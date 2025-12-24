//! Plugin system for RustPress using trait objects and dynamic dispatch.
//!
//! Allows extending functionality through a WordPress-like plugin architecture.

use crate::context::AppContext;
use crate::error::Result;
use async_trait::async_trait;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// Metadata about a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Unique identifier for the plugin
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Plugin version
    pub version: Version,
    /// Plugin description
    pub description: String,
    /// Plugin author
    pub author: String,
    /// Author URL
    pub author_url: Option<String>,
    /// Plugin homepage
    pub homepage: Option<String>,
    /// License
    pub license: String,
    /// Dependencies on other plugins
    pub dependencies: Vec<PluginDependency>,
    /// Minimum RustPress version required
    pub min_rustpress_version: Option<Version>,
    /// Tags for categorization
    pub tags: Vec<String>,
}

impl PluginInfo {
    pub fn new(id: impl Into<String>, name: impl Into<String>, version: Version) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version,
            description: String::new(),
            author: String::new(),
            author_url: None,
            homepage: None,
            license: "MIT".to_string(),
            dependencies: Vec::new(),
            min_rustpress_version: None,
            tags: Vec::new(),
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = author.into();
        self
    }

    pub fn with_dependency(mut self, dep: PluginDependency) -> Self {
        self.dependencies.push(dep);
        self
    }
}

/// A dependency on another plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    /// ID of the required plugin
    pub plugin_id: String,
    /// Version requirement (semver)
    pub version_req: String,
    /// Whether this dependency is optional
    pub optional: bool,
}

impl PluginDependency {
    pub fn new(plugin_id: impl Into<String>, version_req: impl Into<String>) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            version_req: version_req.into(),
            optional: false,
        }
    }

    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
}

/// Plugin state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginState {
    /// Plugin is registered but not activated
    Inactive,
    /// Plugin is currently activating
    Activating,
    /// Plugin is active and running
    Active,
    /// Plugin is deactivating
    Deactivating,
    /// Plugin encountered an error
    Error,
}

/// The main Plugin trait that all plugins must implement
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Get plugin information
    fn info(&self) -> &PluginInfo;

    /// Called when the plugin is activated
    async fn activate(&self, ctx: &AppContext) -> Result<()>;

    /// Called when the plugin is deactivated
    async fn deactivate(&self, ctx: &AppContext) -> Result<()>;

    /// Called during application startup (after all plugins are loaded)
    async fn on_startup(&self, _ctx: &AppContext) -> Result<()> {
        Ok(())
    }

    /// Called during application shutdown
    async fn on_shutdown(&self, _ctx: &AppContext) -> Result<()> {
        Ok(())
    }

    /// Check if this plugin is compatible with the current environment
    fn is_compatible(&self) -> bool {
        true
    }

    /// Get plugin configuration schema (if any)
    fn config_schema(&self) -> Option<serde_json::Value> {
        None
    }

    /// Get current plugin state
    fn state(&self) -> PluginState {
        PluginState::Inactive
    }
}

/// A registered plugin with its runtime state
struct RegisteredPlugin {
    plugin: Arc<dyn Plugin>,
    state: PluginState,
    error: Option<String>,
}

/// Manages all registered plugins
pub struct PluginManager {
    plugins: RwLock<HashMap<String, RegisteredPlugin>>,
    load_order: RwLock<Vec<String>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            load_order: RwLock::new(Vec::new()),
        }
    }

    /// Register a plugin
    pub fn register(&self, plugin: Arc<dyn Plugin>) -> Result<()> {
        let info = plugin.info();
        let id = info.id.clone();

        let mut plugins = self.plugins.write();
        if plugins.contains_key(&id) {
            return Err(crate::error::Error::Plugin {
                plugin_id: id,
                message: "Plugin already registered".to_string(),
            });
        }

        plugins.insert(
            id.clone(),
            RegisteredPlugin {
                plugin,
                state: PluginState::Inactive,
                error: None,
            },
        );

        self.load_order.write().push(id);
        Ok(())
    }

    /// Unregister a plugin
    pub fn unregister(&self, plugin_id: &str) -> Result<()> {
        let mut plugins = self.plugins.write();
        if !plugins.contains_key(plugin_id) {
            return Err(crate::error::Error::PluginNotFound {
                plugin_id: plugin_id.to_string(),
            });
        }

        let registered = plugins.get(plugin_id).unwrap();
        if registered.state == PluginState::Active {
            return Err(crate::error::Error::Plugin {
                plugin_id: plugin_id.to_string(),
                message: "Cannot unregister active plugin. Deactivate first.".to_string(),
            });
        }

        plugins.remove(plugin_id);
        self.load_order.write().retain(|id| id != plugin_id);
        Ok(())
    }

    /// Activate a plugin
    pub async fn activate(&self, plugin_id: &str, ctx: &AppContext) -> Result<()> {
        // Check dependencies first
        self.check_dependencies(plugin_id)?;

        // Update state to activating
        {
            let mut plugins = self.plugins.write();
            let registered = plugins.get_mut(plugin_id).ok_or_else(|| {
                crate::error::Error::PluginNotFound {
                    plugin_id: plugin_id.to_string(),
                }
            })?;
            registered.state = PluginState::Activating;
        }

        // Get plugin reference
        let plugin = {
            let plugins = self.plugins.read();
            plugins.get(plugin_id).map(|r| r.plugin.clone())
        }
        .ok_or_else(|| crate::error::Error::PluginNotFound {
            plugin_id: plugin_id.to_string(),
        })?;

        // Activate
        match plugin.activate(ctx).await {
            Ok(()) => {
                let mut plugins = self.plugins.write();
                if let Some(registered) = plugins.get_mut(plugin_id) {
                    registered.state = PluginState::Active;
                    registered.error = None;
                }
                Ok(())
            }
            Err(e) => {
                let mut plugins = self.plugins.write();
                if let Some(registered) = plugins.get_mut(plugin_id) {
                    registered.state = PluginState::Error;
                    registered.error = Some(e.to_string());
                }
                Err(e)
            }
        }
    }

    /// Deactivate a plugin
    pub async fn deactivate(&self, plugin_id: &str, ctx: &AppContext) -> Result<()> {
        // Check if other plugins depend on this one
        self.check_dependents(plugin_id)?;

        // Update state
        {
            let mut plugins = self.plugins.write();
            let registered = plugins.get_mut(plugin_id).ok_or_else(|| {
                crate::error::Error::PluginNotFound {
                    plugin_id: plugin_id.to_string(),
                }
            })?;
            registered.state = PluginState::Deactivating;
        }

        // Get plugin reference
        let plugin = {
            let plugins = self.plugins.read();
            plugins.get(plugin_id).map(|r| r.plugin.clone())
        }
        .ok_or_else(|| crate::error::Error::PluginNotFound {
            plugin_id: plugin_id.to_string(),
        })?;

        // Deactivate
        match plugin.deactivate(ctx).await {
            Ok(()) => {
                let mut plugins = self.plugins.write();
                if let Some(registered) = plugins.get_mut(plugin_id) {
                    registered.state = PluginState::Inactive;
                }
                Ok(())
            }
            Err(e) => {
                let mut plugins = self.plugins.write();
                if let Some(registered) = plugins.get_mut(plugin_id) {
                    registered.state = PluginState::Error;
                    registered.error = Some(e.to_string());
                }
                Err(e)
            }
        }
    }

    /// Get a plugin by ID
    pub fn get(&self, plugin_id: &str) -> Option<Arc<dyn Plugin>> {
        self.plugins.read().get(plugin_id).map(|r| r.plugin.clone())
    }

    /// Get plugin state
    pub fn state(&self, plugin_id: &str) -> Option<PluginState> {
        self.plugins.read().get(plugin_id).map(|r| r.state)
    }

    /// List all registered plugins
    pub fn list(&self) -> Vec<PluginInfo> {
        self.plugins
            .read()
            .values()
            .map(|r| r.plugin.info().clone())
            .collect()
    }

    /// List active plugins
    pub fn list_active(&self) -> Vec<PluginInfo> {
        self.plugins
            .read()
            .values()
            .filter(|r| r.state == PluginState::Active)
            .map(|r| r.plugin.info().clone())
            .collect()
    }

    /// Check if a plugin's dependencies are satisfied
    fn check_dependencies(&self, plugin_id: &str) -> Result<()> {
        let plugins = self.plugins.read();
        let registered = plugins.get(plugin_id).ok_or_else(|| {
            crate::error::Error::PluginNotFound {
                plugin_id: plugin_id.to_string(),
            }
        })?;

        for dep in &registered.plugin.info().dependencies {
            if dep.optional {
                continue;
            }

            let dep_plugin = plugins.get(&dep.plugin_id);
            match dep_plugin {
                None => {
                    return Err(crate::error::Error::PluginDependency {
                        plugin_id: plugin_id.to_string(),
                        dependency: dep.plugin_id.clone(),
                    });
                }
                Some(p) if p.state != PluginState::Active => {
                    return Err(crate::error::Error::PluginDependency {
                        plugin_id: plugin_id.to_string(),
                        dependency: format!("{} (not active)", dep.plugin_id),
                    });
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Check if any active plugins depend on this one
    fn check_dependents(&self, plugin_id: &str) -> Result<()> {
        let plugins = self.plugins.read();

        for (id, registered) in plugins.iter() {
            if registered.state != PluginState::Active {
                continue;
            }

            for dep in &registered.plugin.info().dependencies {
                if dep.plugin_id == plugin_id && !dep.optional {
                    return Err(crate::error::Error::Plugin {
                        plugin_id: plugin_id.to_string(),
                        message: format!("Plugin {} depends on this plugin", id),
                    });
                }
            }
        }

        Ok(())
    }

    /// Startup all active plugins
    pub async fn startup(&self, ctx: &AppContext) -> Result<()> {
        let order = self.load_order.read().clone();

        for plugin_id in order {
            let plugin = {
                let plugins = self.plugins.read();
                plugins
                    .get(&plugin_id)
                    .filter(|r| r.state == PluginState::Active)
                    .map(|r| r.plugin.clone())
            };

            if let Some(plugin) = plugin {
                plugin.on_startup(ctx).await?;
            }
        }

        Ok(())
    }

    /// Shutdown all active plugins (in reverse order)
    pub async fn shutdown(&self, ctx: &AppContext) -> Result<()> {
        let order: Vec<_> = self.load_order.read().iter().rev().cloned().collect();

        for plugin_id in order {
            let plugin = {
                let plugins = self.plugins.read();
                plugins
                    .get(&plugin_id)
                    .filter(|r| r.state == PluginState::Active)
                    .map(|r| r.plugin.clone())
            };

            if let Some(plugin) = plugin {
                if let Err(e) = plugin.on_shutdown(ctx).await {
                    tracing::error!(
                        plugin_id = %plugin_id,
                        error = %e,
                        "Plugin shutdown error"
                    );
                }
            }
        }

        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin {
        info: PluginInfo,
    }

    impl TestPlugin {
        fn new(id: &str) -> Self {
            Self {
                info: PluginInfo::new(id, id, Version::new(1, 0, 0)),
            }
        }
    }

    #[async_trait]
    impl Plugin for TestPlugin {
        fn info(&self) -> &PluginInfo {
            &self.info
        }

        async fn activate(&self, _ctx: &AppContext) -> Result<()> {
            Ok(())
        }

        async fn deactivate(&self, _ctx: &AppContext) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_plugin_registration() {
        let manager = PluginManager::new();
        let plugin = Arc::new(TestPlugin::new("test-plugin"));

        manager.register(plugin).unwrap();
        assert!(manager.get("test-plugin").is_some());
        assert_eq!(manager.state("test-plugin"), Some(PluginState::Inactive));
    }

    #[test]
    fn test_duplicate_registration() {
        let manager = PluginManager::new();
        let plugin1 = Arc::new(TestPlugin::new("test-plugin"));
        let plugin2 = Arc::new(TestPlugin::new("test-plugin"));

        manager.register(plugin1).unwrap();
        assert!(manager.register(plugin2).is_err());
    }

    #[tokio::test]
    async fn test_plugin_activation() {
        let manager = PluginManager::new();
        let plugin = Arc::new(TestPlugin::new("test-plugin"));
        let config = crate::config::AppConfig::default();
        let ctx = AppContext::new(config);

        manager.register(plugin).unwrap();
        manager.activate("test-plugin", &ctx).await.unwrap();

        assert_eq!(manager.state("test-plugin"), Some(PluginState::Active));
    }
}
