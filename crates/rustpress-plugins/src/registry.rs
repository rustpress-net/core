//! Plugin registry for tracking installed plugins.

use crate::PluginMetadata;
use rustpress_core::error::{Error, Result};
use rustpress_core::plugin::PluginInfo;
use std::collections::HashMap;
use std::sync::RwLock;

/// Plugin registry for managing plugin metadata
pub struct PluginRegistry {
    /// Registered plugins
    plugins: RwLock<HashMap<String, PluginMetadata>>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
        }
    }

    /// Register a plugin
    pub fn register(&self, metadata: PluginMetadata) -> Result<()> {
        let mut plugins = self.plugins.write().map_err(|_| Error::Internal {
            message: "Failed to acquire write lock".to_string(),
            request_id: None,
        })?;

        let name = metadata.info.name.clone();
        plugins.insert(name, metadata);
        Ok(())
    }

    /// Unregister a plugin
    pub fn unregister(&self, name: &str) -> Result<PluginMetadata> {
        let mut plugins = self.plugins.write().map_err(|_| Error::Internal {
            message: "Failed to acquire write lock".to_string(),
            request_id: None,
        })?;

        plugins.remove(name).ok_or_else(|| Error::NotFound {
            entity_type: "Plugin".to_string(),
            id: name.to_string(),
        })
    }

    /// Get plugin metadata
    pub fn get(&self, name: &str) -> Result<Option<PluginMetadata>> {
        let plugins = self.plugins.read().map_err(|_| Error::Internal {
            message: "Failed to acquire read lock".to_string(),
            request_id: None,
        })?;

        Ok(plugins.get(name).cloned())
    }

    /// List all plugins
    pub fn list(&self) -> Result<Vec<PluginMetadata>> {
        let plugins = self.plugins.read().map_err(|_| Error::Internal {
            message: "Failed to acquire read lock".to_string(),
            request_id: None,
        })?;

        Ok(plugins.values().cloned().collect())
    }

    /// List enabled plugins
    pub fn enabled(&self) -> Result<Vec<PluginMetadata>> {
        let plugins = self.plugins.read().map_err(|_| Error::Internal {
            message: "Failed to acquire read lock".to_string(),
            request_id: None,
        })?;

        Ok(plugins.values().filter(|p| p.enabled).cloned().collect())
    }

    /// Enable a plugin
    pub fn enable(&self, name: &str) -> Result<()> {
        let mut plugins = self.plugins.write().map_err(|_| Error::Internal {
            message: "Failed to acquire write lock".to_string(),
            request_id: None,
        })?;

        let plugin = plugins.get_mut(name).ok_or_else(|| Error::NotFound {
            entity_type: "Plugin".to_string(),
            id: name.to_string(),
        })?;

        plugin.enabled = true;
        Ok(())
    }

    /// Disable a plugin
    pub fn disable(&self, name: &str) -> Result<()> {
        let mut plugins = self.plugins.write().map_err(|_| Error::Internal {
            message: "Failed to acquire write lock".to_string(),
            request_id: None,
        })?;

        let plugin = plugins.get_mut(name).ok_or_else(|| Error::NotFound {
            entity_type: "Plugin".to_string(),
            id: name.to_string(),
        })?;

        plugin.enabled = false;
        Ok(())
    }

    /// Update plugin settings
    pub fn update_settings(&self, name: &str, settings: serde_json::Value) -> Result<()> {
        let mut plugins = self.plugins.write().map_err(|_| Error::Internal {
            message: "Failed to acquire write lock".to_string(),
            request_id: None,
        })?;

        let plugin = plugins.get_mut(name).ok_or_else(|| Error::NotFound {
            entity_type: "Plugin".to_string(),
            id: name.to_string(),
        })?;

        plugin.settings = settings;
        Ok(())
    }

    /// Check if a plugin is registered
    pub fn is_registered(&self, name: &str) -> bool {
        self.plugins
            .read()
            .map(|p| p.contains_key(name))
            .unwrap_or(false)
    }

    /// Check if a plugin is enabled
    pub fn is_enabled(&self, name: &str) -> bool {
        self.plugins
            .read()
            .map(|p| p.get(name).map(|m| m.enabled).unwrap_or(false))
            .unwrap_or(false)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustpress_core::plugin::PluginInfo;

    fn test_plugin_info() -> PluginInfo {
        PluginInfo {
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            dependencies: vec![],
        }
    }

    #[test]
    fn test_register_plugin() {
        let registry = PluginRegistry::new();
        let metadata = PluginMetadata::new(test_plugin_info());

        registry.register(metadata).unwrap();

        assert!(registry.is_registered("test-plugin"));
    }

    #[test]
    fn test_enable_disable() {
        let registry = PluginRegistry::new();
        let metadata = PluginMetadata::new(test_plugin_info());

        registry.register(metadata).unwrap();

        assert!(!registry.is_enabled("test-plugin"));

        registry.enable("test-plugin").unwrap();
        assert!(registry.is_enabled("test-plugin"));

        registry.disable("test-plugin").unwrap();
        assert!(!registry.is_enabled("test-plugin"));
    }
}
