//! Plugin Loader - Automatic plugin discovery and registration
//!
//! Scans the plugins directory for plugin.toml manifests and automatically
//! registers discovered plugins with the PluginManager.

use crate::context::AppContext;
use crate::error::Result;
use crate::plugin::{Plugin, PluginManager};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Plugin manifest loaded from plugin.toml
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub author_url: Option<String>,
    pub license: Option<String>,
    pub min_rustpress_version: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub dependencies: PluginDependencies,
    #[serde(default)]
    pub settings: PluginSettingsSchema,
    #[serde(default)]
    pub hooks: PluginHooks,
    #[serde(default)]
    pub api: PluginApi,
    #[serde(default)]
    pub admin: PluginAdmin,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct PluginDependencies {
    #[serde(default)]
    pub plugins: HashMap<String, String>,
    #[serde(default)]
    pub conflicts: Vec<String>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct PluginSettingsSchema {
    #[serde(default)]
    pub schema: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct PluginHooks {
    pub activate: Option<String>,
    pub deactivate: Option<String>,
    pub uninstall: Option<String>,
    #[serde(default)]
    pub actions: Vec<HookAction>,
    #[serde(default)]
    pub filters: Vec<HookFilter>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct HookAction {
    pub hook: String,
    pub callback: String,
    #[serde(default = "default_priority")]
    pub priority: i32,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct HookFilter {
    pub hook: String,
    pub callback: String,
    #[serde(default = "default_priority")]
    pub priority: i32,
}

fn default_priority() -> i32 {
    10
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct PluginApi {
    pub namespace: Option<String>,
    pub version: Option<String>,
    #[serde(default)]
    pub endpoints: Vec<ApiEndpoint>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApiEndpoint {
    pub path: String,
    pub method: String,
    pub handler: String,
    pub permission: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct PluginAdmin {
    #[serde(default)]
    pub menu: Vec<AdminMenuItem>,
    #[serde(default)]
    pub pages: Vec<AdminPage>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AdminMenuItem {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    #[serde(default)]
    pub position: i32,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AdminPage {
    pub id: String,
    pub title: String,
    pub handler: String,
    pub capability: Option<String>,
    pub parent: Option<String>,
}

/// Result of scanning the plugins directory
#[derive(Debug, Default)]
pub struct PluginScanResult {
    /// Successfully discovered plugins
    pub discovered: Vec<PluginManifest>,
    /// Errors encountered during scanning
    pub errors: Vec<String>,
    /// Plugins that were skipped
    pub skipped: Vec<String>,
}

/// Plugin loader that discovers and registers plugins
pub struct PluginLoader {
    plugins_dir: PathBuf,
    /// Registry of plugin factory functions
    factories: HashMap<String, Box<dyn Fn() -> Arc<dyn Plugin> + Send + Sync>>,
}

impl PluginLoader {
    /// Create a new plugin loader for the given plugins directory
    pub fn new(plugins_dir: impl Into<PathBuf>) -> Self {
        Self {
            plugins_dir: plugins_dir.into(),
            factories: HashMap::new(),
        }
    }

    /// Register a plugin factory function
    ///
    /// This is used to map plugin IDs to their Rust implementations.
    /// Since Rust plugins are statically compiled, we need to register
    /// factory functions that create plugin instances.
    pub fn register_factory<F>(&mut self, plugin_id: &str, factory: F)
    where
        F: Fn() -> Arc<dyn Plugin> + Send + Sync + 'static,
    {
        self.factories.insert(plugin_id.to_string(), Box::new(factory));
    }

    /// Scan the plugins directory for plugin.toml manifests
    pub fn scan(&self) -> PluginScanResult {
        let mut result = PluginScanResult::default();

        if !self.plugins_dir.exists() {
            warn!(path = ?self.plugins_dir, "Plugins directory does not exist");
            return result;
        }

        info!(path = ?self.plugins_dir, "Scanning plugins directory");

        // Read directory entries
        let entries = match std::fs::read_dir(&self.plugins_dir) {
            Ok(entries) => entries,
            Err(e) => {
                result.errors.push(format!("Failed to read plugins directory: {}", e));
                return result;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();

            // Skip if not a directory
            if !path.is_dir() {
                continue;
            }

            // Look for plugin.toml
            let manifest_path = path.join("plugin.toml");
            if !manifest_path.exists() {
                debug!(path = ?path, "No plugin.toml found, skipping");
                result.skipped.push(path.display().to_string());
                continue;
            }

            // Load and parse the manifest
            match self.load_manifest(&manifest_path) {
                Ok(manifest) => {
                    info!(
                        plugin_id = %manifest.id,
                        plugin_name = %manifest.name,
                        version = %manifest.version,
                        "Discovered plugin"
                    );
                    result.discovered.push(manifest);
                }
                Err(e) => {
                    result.errors.push(format!(
                        "Failed to load {}: {}",
                        manifest_path.display(),
                        e
                    ));
                }
            }
        }

        info!(
            discovered = result.discovered.len(),
            errors = result.errors.len(),
            skipped = result.skipped.len(),
            "Plugin scan complete"
        );

        result
    }

    /// Load a plugin manifest from a file
    fn load_manifest(&self, path: &Path) -> Result<PluginManifest> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::error::Error::Internal {
                message: format!("Failed to read manifest: {}", e),
                request_id: None,
            })?;

        let manifest: PluginManifest = toml::from_str(&content)
            .map_err(|e| crate::error::Error::Internal {
                message: format!("Failed to parse manifest: {}", e),
                request_id: None,
            })?;

        Ok(manifest)
    }

    /// Load and register all discovered plugins with the plugin manager
    pub async fn load_all(
        &self,
        manager: Arc<RwLock<PluginManager>>,
        ctx: &AppContext,
    ) -> Result<LoadResult> {
        let scan_result = self.scan();
        let mut load_result = LoadResult::default();

        for manifest in scan_result.discovered {
            let plugin_id = manifest.id.clone();

            // Check if we have a factory for this plugin
            if let Some(factory) = self.factories.get(&plugin_id) {
                match self
                    .load_plugin(manager.clone(), ctx, &manifest, factory)
                    .await
                {
                    Ok(()) => {
                        load_result.loaded.push(plugin_id);
                    }
                    Err(e) => {
                        load_result.errors.push(format!("{}: {}", plugin_id, e));
                    }
                }
            } else {
                debug!(plugin_id = %plugin_id, "No factory registered for plugin");
                load_result.skipped.push(plugin_id);
            }
        }

        load_result.scan_errors = scan_result.errors;

        Ok(load_result)
    }

    /// Load and register a single plugin
    async fn load_plugin(
        &self,
        manager: Arc<RwLock<PluginManager>>,
        ctx: &AppContext,
        manifest: &PluginManifest,
        factory: &(dyn Fn() -> Arc<dyn Plugin> + Send + Sync),
    ) -> Result<()> {
        info!(plugin_id = %manifest.id, "Loading plugin");

        // Create plugin instance
        let plugin = factory();

        // Register with manager (short-lived lock)
        {
            let mgr = manager.read().await;
            mgr.register(plugin)?;
        }

        // Activate the plugin (short-lived lock per operation inside PluginManager)
        {
            let mgr = manager.read().await;
            mgr.activate(&manifest.id, ctx).await?;
        }

        info!(plugin_id = %manifest.id, "Plugin loaded and activated");

        Ok(())
    }
}

/// Result of loading plugins
#[derive(Debug, Default)]
pub struct LoadResult {
    /// Successfully loaded plugins
    pub loaded: Vec<String>,
    /// Plugins that were skipped (no factory registered)
    pub skipped: Vec<String>,
    /// Errors during loading
    pub errors: Vec<String>,
    /// Errors from the scan phase
    pub scan_errors: Vec<String>,
}

impl LoadResult {
    pub fn is_success(&self) -> bool {
        self.errors.is_empty() && self.scan_errors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_scan_empty_directory() {
        let dir = tempdir().unwrap();
        let loader = PluginLoader::new(dir.path());
        let result = loader.scan();

        assert!(result.discovered.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_scan_with_manifest() {
        let dir = tempdir().unwrap();
        let plugin_dir = dir.path().join("test-plugin");
        std::fs::create_dir(&plugin_dir).unwrap();

        let manifest = r#"
            id = "test-plugin"
            name = "Test Plugin"
            version = "1.0.0"
        "#;
        std::fs::write(plugin_dir.join("plugin.toml"), manifest).unwrap();

        let loader = PluginLoader::new(dir.path());
        let result = loader.scan();

        assert_eq!(result.discovered.len(), 1);
        assert_eq!(result.discovered[0].id, "test-plugin");
    }
}
