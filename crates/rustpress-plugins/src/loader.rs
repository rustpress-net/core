//! Plugin loading infrastructure.

use rustpress_core::error::{Error, Result};
use rustpress_core::plugin::{Plugin, PluginInfo};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tracing::{info, warn};

/// Plugin loader for discovering and loading plugins
pub struct PluginLoader {
    /// Directory where plugins are stored
    plugins_dir: String,
    /// Loaded plugins
    loaded: HashMap<String, Arc<dyn Plugin>>,
}

impl PluginLoader {
    /// Create a new plugin loader
    pub fn new(plugins_dir: impl Into<String>) -> Self {
        Self {
            plugins_dir: plugins_dir.into(),
            loaded: HashMap::new(),
        }
    }

    /// Discover plugins in the plugins directory
    pub fn discover(&self) -> Result<Vec<PluginInfo>> {
        let path = Path::new(&self.plugins_dir);

        if !path.exists() {
            return Ok(Vec::new());
        }

        let mut plugins = Vec::new();

        // Look for plugin manifests
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();

                // Check for plugin.json manifest
                let manifest_path = entry_path.join("plugin.json");
                if manifest_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                        if let Ok(info) = serde_json::from_str::<PluginInfo>(&content) {
                            info!("Discovered plugin: {} v{}", info.name, info.version);
                            plugins.push(info);
                        }
                    }
                }
            }
        }

        Ok(plugins)
    }

    /// Load a plugin by name
    pub fn load(&mut self, name: &str) -> Result<Arc<dyn Plugin>> {
        if let Some(plugin) = self.loaded.get(name) {
            return Ok(Arc::clone(plugin));
        }

        let plugin_path = Path::new(&self.plugins_dir).join(name);

        if !plugin_path.exists() {
            return Err(Error::NotFound {
                entity_type: "Plugin".to_string(),
                id: name.to_string(),
            });
        }

        // For now, we only support Rust plugins compiled as dynamic libraries
        // In the future, we could support WASM plugins
        let lib_name = if cfg!(target_os = "windows") {
            format!("{}.dll", name)
        } else if cfg!(target_os = "macos") {
            format!("lib{}.dylib", name)
        } else {
            format!("lib{}.so", name)
        };

        let lib_path = plugin_path.join(&lib_name);

        if !lib_path.exists() {
            warn!("Plugin library not found: {:?}", lib_path);
            return Err(Error::Plugin {
                plugin_id: name.to_string(),
                message: format!("Plugin library not found: {}", lib_name),
            });
        }

        // Dynamic loading would happen here
        // For safety reasons, we're just providing the infrastructure
        // Actual dynamic loading should be carefully implemented

        Err(Error::Plugin {
            plugin_id: name.to_string(),
            message: "Dynamic plugin loading not yet implemented".to_string(),
        })
    }

    /// Unload a plugin
    pub fn unload(&mut self, name: &str) -> Result<()> {
        if self.loaded.remove(name).is_some() {
            info!("Unloaded plugin: {}", name);
            Ok(())
        } else {
            Err(Error::NotFound {
                entity_type: "Plugin".to_string(),
                id: name.to_string(),
            })
        }
    }

    /// Get a loaded plugin
    pub fn get(&self, name: &str) -> Option<Arc<dyn Plugin>> {
        self.loaded.get(name).cloned()
    }

    /// List loaded plugins
    pub fn loaded_plugins(&self) -> Vec<String> {
        self.loaded.keys().cloned().collect()
    }

    /// Register a plugin instance (for built-in plugins)
    pub fn register(&mut self, plugin: Arc<dyn Plugin>) {
        let name = plugin.info().name.clone();
        info!("Registered plugin: {}", name);
        self.loaded.insert(name, plugin);
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new("./plugins")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_loader_creation() {
        let loader = PluginLoader::new("/tmp/plugins");
        assert_eq!(loader.plugins_dir, "/tmp/plugins");
    }

    #[test]
    fn test_discover_empty_dir() {
        let loader = PluginLoader::new("/nonexistent/path");
        let result = loader.discover();
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
