//! Plugin Discovery and Loading Mechanism
//!
//! Discovers plugins from configured directories and loads them.

use crate::manifest::{ManifestError, PluginManifest, PluginType};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{debug, error, info, warn};
use walkdir::WalkDir;
use notify::{Watcher, RecursiveMode, Event, EventKind};
use tokio::sync::mpsc;

/// Plugin discovery configuration
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Plugin directories to scan
    pub plugin_dirs: Vec<PathBuf>,

    /// Must-use plugins directory
    pub must_use_dir: Option<PathBuf>,

    /// Drop-in plugins directory
    pub dropin_dir: Option<PathBuf>,

    /// File patterns to recognize plugins
    pub manifest_filename: String,

    /// Enable file watching for hot reload
    pub watch_enabled: bool,

    /// Scan depth limit
    pub max_depth: usize,

    /// Include hidden directories
    pub include_hidden: bool,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            plugin_dirs: vec![PathBuf::from("plugins")],
            must_use_dir: Some(PathBuf::from("mu-plugins")),
            dropin_dir: Some(PathBuf::from("dropin")),
            manifest_filename: "plugin.toml".to_string(),
            watch_enabled: false,
            max_depth: 2,
            include_hidden: false,
        }
    }
}

/// Discovered plugin information
#[derive(Debug, Clone)]
pub struct DiscoveredPlugin {
    /// Plugin manifest
    pub manifest: PluginManifest,

    /// Path to plugin directory
    pub path: PathBuf,

    /// Path to manifest file
    pub manifest_path: PathBuf,

    /// Plugin source type
    pub source: PluginSource,

    /// Discovery timestamp
    pub discovered_at: chrono::DateTime<chrono::Utc>,

    /// File checksum for change detection
    pub checksum: String,
}

/// Plugin source type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginSource {
    /// Regular plugin directory
    Standard,
    /// Must-use plugin (always activated)
    MustUse,
    /// Drop-in plugin (special functionality)
    DropIn,
    /// Network-installed plugin
    Network,
}

/// Plugin discovery events
#[derive(Debug, Clone)]
pub enum DiscoveryEvent {
    /// New plugin discovered
    Discovered(DiscoveredPlugin),
    /// Plugin files changed
    Changed(String, PathBuf),
    /// Plugin removed
    Removed(String, PathBuf),
    /// Discovery error
    Error(String, String),
}

/// Plugin discovery service
pub struct PluginDiscovery {
    config: DiscoveryConfig,
    plugins: Arc<RwLock<HashMap<String, DiscoveredPlugin>>>,
    event_tx: Option<mpsc::Sender<DiscoveryEvent>>,
    watcher: Option<notify::RecommendedWatcher>,
}

impl PluginDiscovery {
    /// Create a new plugin discovery service
    pub fn new(config: DiscoveryConfig) -> Self {
        Self {
            config,
            plugins: Arc::new(RwLock::new(HashMap::new())),
            event_tx: None,
            watcher: None,
        }
    }

    /// Get event receiver for discovery events
    pub fn subscribe(&mut self) -> mpsc::Receiver<DiscoveryEvent> {
        let (tx, rx) = mpsc::channel(100);
        self.event_tx = Some(tx);
        rx
    }

    /// Scan all plugin directories
    pub fn scan_all(&self) -> Result<Vec<DiscoveredPlugin>, DiscoveryError> {
        let mut discovered = Vec::new();

        // Scan must-use plugins first
        if let Some(ref mu_dir) = self.config.must_use_dir {
            if mu_dir.exists() {
                let plugins = self.scan_directory(mu_dir, PluginSource::MustUse)?;
                discovered.extend(plugins);
            }
        }

        // Scan drop-in plugins
        if let Some(ref dropin_dir) = self.config.dropin_dir {
            if dropin_dir.exists() {
                let plugins = self.scan_directory(dropin_dir, PluginSource::DropIn)?;
                discovered.extend(plugins);
            }
        }

        // Scan standard plugin directories
        for dir in &self.config.plugin_dirs {
            if dir.exists() {
                let plugins = self.scan_directory(dir, PluginSource::Standard)?;
                discovered.extend(plugins);
            }
        }

        // Update internal cache
        {
            let mut cache = self.plugins.write();
            cache.clear();
            for plugin in &discovered {
                cache.insert(plugin.manifest.plugin.id.clone(), plugin.clone());
            }
        }

        info!("Discovered {} plugins", discovered.len());
        Ok(discovered)
    }

    /// Scan a single directory for plugins
    fn scan_directory(
        &self,
        dir: &Path,
        source: PluginSource,
    ) -> Result<Vec<DiscoveredPlugin>, DiscoveryError> {
        let mut plugins = Vec::new();

        for entry in WalkDir::new(dir)
            .max_depth(self.config.max_depth)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| {
                self.config.include_hidden
                    || !e
                        .file_name()
                        .to_str()
                        .map(|s| s.starts_with('.'))
                        .unwrap_or(false)
            })
        {
            let entry = entry.map_err(|e| DiscoveryError::Scan(e.to_string()))?;

            if entry.file_type().is_file()
                && entry.file_name().to_str() == Some(&self.config.manifest_filename)
            {
                match self.load_plugin(entry.path(), source) {
                    Ok(plugin) => {
                        debug!("Found plugin: {} at {:?}", plugin.manifest.plugin.id, plugin.path);
                        plugins.push(plugin);
                    }
                    Err(e) => {
                        warn!("Failed to load plugin at {:?}: {}", entry.path(), e);
                    }
                }
            }
        }

        Ok(plugins)
    }

    /// Load a single plugin from manifest path
    fn load_plugin(
        &self,
        manifest_path: &Path,
        source: PluginSource,
    ) -> Result<DiscoveredPlugin, DiscoveryError> {
        let manifest = PluginManifest::from_file(manifest_path)
            .map_err(|e| DiscoveryError::Manifest(e))?;

        // Validate manifest
        if let Err(errors) = manifest.validate() {
            return Err(DiscoveryError::Validation(
                errors.into_iter().map(|e| format!("{}: {}", e.field, e.message)).collect(),
            ));
        }

        let plugin_dir = manifest_path
            .parent()
            .ok_or_else(|| DiscoveryError::InvalidPath(manifest_path.to_path_buf()))?
            .to_path_buf();

        // Calculate checksum of manifest for change detection
        let checksum = self.calculate_checksum(manifest_path)?;

        Ok(DiscoveredPlugin {
            manifest,
            path: plugin_dir,
            manifest_path: manifest_path.to_path_buf(),
            source,
            discovered_at: chrono::Utc::now(),
            checksum,
        })
    }

    /// Calculate file checksum
    fn calculate_checksum(&self, path: &Path) -> Result<String, DiscoveryError> {
        let content = std::fs::read(path)
            .map_err(|e| DiscoveryError::Io(e.to_string()))?;
        let hash = blake3::hash(&content);
        Ok(hash.to_hex().to_string())
    }

    /// Get a discovered plugin by ID
    pub fn get(&self, plugin_id: &str) -> Option<DiscoveredPlugin> {
        self.plugins.read().get(plugin_id).cloned()
    }

    /// Get all discovered plugins
    pub fn get_all(&self) -> Vec<DiscoveredPlugin> {
        self.plugins.read().values().cloned().collect()
    }

    /// Get must-use plugins
    pub fn get_must_use(&self) -> Vec<DiscoveredPlugin> {
        self.plugins
            .read()
            .values()
            .filter(|p| p.source == PluginSource::MustUse || p.manifest.plugin.must_use)
            .cloned()
            .collect()
    }

    /// Get plugins by type
    pub fn get_by_type(&self, plugin_type: PluginType) -> Vec<DiscoveredPlugin> {
        self.plugins
            .read()
            .values()
            .filter(|p| p.manifest.plugin.plugin_type == plugin_type)
            .cloned()
            .collect()
    }

    /// Start watching for file changes
    pub fn start_watching(&mut self) -> Result<(), DiscoveryError> {
        if !self.config.watch_enabled {
            return Ok(());
        }

        let plugins = Arc::clone(&self.plugins);
        let event_tx = self.event_tx.clone();

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    if let Some(ref tx) = event_tx {
                        match event.kind {
                            EventKind::Create(_) | EventKind::Modify(_) => {
                                for path in event.paths {
                                    if path.file_name().map(|n| n == "plugin.toml").unwrap_or(false) {
                                        // Check if this is a known plugin
                                        let plugins_guard = plugins.read();
                                        let plugin_id = plugins_guard
                                            .values()
                                            .find(|p| p.manifest_path == path)
                                            .map(|p| p.manifest.plugin.id.clone());
                                        drop(plugins_guard);

                                        if let Some(id) = plugin_id {
                                            let _ = tx.blocking_send(DiscoveryEvent::Changed(id, path));
                                        } else {
                                            let _ = tx.blocking_send(DiscoveryEvent::Discovered(
                                                DiscoveredPlugin {
                                                    manifest: PluginManifest::from_file(&path).unwrap(),
                                                    path: path.parent().unwrap().to_path_buf(),
                                                    manifest_path: path.clone(),
                                                    source: PluginSource::Standard,
                                                    discovered_at: chrono::Utc::now(),
                                                    checksum: String::new(),
                                                },
                                            ));
                                        }
                                    }
                                }
                            }
                            EventKind::Remove(_) => {
                                for path in event.paths {
                                    let plugins_guard = plugins.read();
                                    let plugin_id = plugins_guard
                                        .values()
                                        .find(|p| p.manifest_path == path)
                                        .map(|p| p.manifest.plugin.id.clone());
                                    drop(plugins_guard);

                                    if let Some(id) = plugin_id {
                                        let _ = tx.blocking_send(DiscoveryEvent::Removed(id, path));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    error!("File watch error: {}", e);
                }
            }
        }).map_err(|e| DiscoveryError::Watch(e.to_string()))?;

        // Watch all plugin directories
        for dir in &self.config.plugin_dirs {
            if dir.exists() {
                watcher.watch(dir, RecursiveMode::Recursive)
                    .map_err(|e| DiscoveryError::Watch(e.to_string()))?;
            }
        }

        if let Some(ref mu_dir) = self.config.must_use_dir {
            if mu_dir.exists() {
                watcher.watch(mu_dir, RecursiveMode::Recursive)
                    .map_err(|e| DiscoveryError::Watch(e.to_string()))?;
            }
        }

        self.watcher = Some(watcher);
        info!("Started watching plugin directories for changes");
        Ok(())
    }

    /// Stop watching for file changes
    pub fn stop_watching(&mut self) {
        self.watcher = None;
    }

    /// Check for plugin updates by comparing checksums
    pub fn check_for_changes(&self) -> Vec<(String, ChangeType)> {
        let mut changes = Vec::new();
        let plugins = self.plugins.read();

        for (id, plugin) in plugins.iter() {
            if let Ok(current_checksum) = self.calculate_checksum(&plugin.manifest_path) {
                if current_checksum != plugin.checksum {
                    changes.push((id.clone(), ChangeType::Modified));
                }
            } else {
                changes.push((id.clone(), ChangeType::Removed));
            }
        }

        changes
    }

    /// Refresh a specific plugin
    pub fn refresh_plugin(&self, plugin_id: &str) -> Result<DiscoveredPlugin, DiscoveryError> {
        let current = self.plugins.read().get(plugin_id).cloned();

        if let Some(current) = current {
            let updated = self.load_plugin(&current.manifest_path, current.source)?;
            self.plugins.write().insert(plugin_id.to_string(), updated.clone());
            Ok(updated)
        } else {
            Err(DiscoveryError::NotFound(plugin_id.to_string()))
        }
    }
}

/// Change type for plugin files
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    Modified,
    Removed,
}

/// Plugin loader for different plugin types
pub struct PluginLoader {
    /// Loaded native libraries
    native_libs: HashMap<String, libloading::Library>,
}

impl PluginLoader {
    pub fn new() -> Self {
        Self {
            native_libs: HashMap::new(),
        }
    }

    /// Load a plugin based on its type
    pub fn load(&mut self, plugin: &DiscoveredPlugin) -> Result<LoadedPlugin, LoadError> {
        match plugin.manifest.plugin.plugin_type {
            PluginType::Wasm => self.load_wasm(plugin),
            PluginType::Native => self.load_native(plugin),
            PluginType::Script => self.load_script(plugin),
        }
    }

    /// Load a WebAssembly plugin
    fn load_wasm(&self, plugin: &DiscoveredPlugin) -> Result<LoadedPlugin, LoadError> {
        let wasm_path = plugin.path.join(&plugin.manifest.plugin.entry);

        if !wasm_path.exists() {
            return Err(LoadError::EntryNotFound(wasm_path));
        }

        let wasm_bytes = std::fs::read(&wasm_path)
            .map_err(|e| LoadError::Io(e.to_string()))?;

        Ok(LoadedPlugin {
            id: plugin.manifest.plugin.id.clone(),
            plugin_type: PluginType::Wasm,
            data: PluginData::Wasm(wasm_bytes),
        })
    }

    /// Load a native (dynamic library) plugin
    fn load_native(&mut self, plugin: &DiscoveredPlugin) -> Result<LoadedPlugin, LoadError> {
        let lib_name = if cfg!(windows) {
            format!("{}.dll", plugin.manifest.plugin.entry.trim_end_matches(".dll"))
        } else if cfg!(target_os = "macos") {
            format!("lib{}.dylib", plugin.manifest.plugin.entry.trim_start_matches("lib").trim_end_matches(".dylib"))
        } else {
            format!("lib{}.so", plugin.manifest.plugin.entry.trim_start_matches("lib").trim_end_matches(".so"))
        };

        let lib_path = plugin.path.join(&lib_name);

        if !lib_path.exists() {
            return Err(LoadError::EntryNotFound(lib_path));
        }

        // Load the library
        let lib = unsafe {
            libloading::Library::new(&lib_path)
                .map_err(|e| LoadError::NativeLoad(e.to_string()))?
        };

        self.native_libs.insert(plugin.manifest.plugin.id.clone(), lib);

        Ok(LoadedPlugin {
            id: plugin.manifest.plugin.id.clone(),
            plugin_type: PluginType::Native,
            data: PluginData::Native(lib_path),
        })
    }

    /// Load a script plugin
    fn load_script(&self, plugin: &DiscoveredPlugin) -> Result<LoadedPlugin, LoadError> {
        let script_path = plugin.path.join(&plugin.manifest.plugin.entry);

        if !script_path.exists() {
            return Err(LoadError::EntryNotFound(script_path));
        }

        let script_content = std::fs::read_to_string(&script_path)
            .map_err(|e| LoadError::Io(e.to_string()))?;

        Ok(LoadedPlugin {
            id: plugin.manifest.plugin.id.clone(),
            plugin_type: PluginType::Script,
            data: PluginData::Script(script_content),
        })
    }

    /// Unload a native plugin
    pub fn unload(&mut self, plugin_id: &str) {
        self.native_libs.remove(plugin_id);
    }

    /// Get a function from a native plugin
    pub unsafe fn get_native_function<T>(
        &self,
        plugin_id: &str,
        name: &str,
    ) -> Result<libloading::Symbol<T>, LoadError> {
        let lib = self.native_libs.get(plugin_id)
            .ok_or_else(|| LoadError::NotLoaded(plugin_id.to_string()))?;

        lib.get(name.as_bytes())
            .map_err(|e| LoadError::SymbolNotFound(name.to_string(), e.to_string()))
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Loaded plugin data
pub struct LoadedPlugin {
    pub id: String,
    pub plugin_type: PluginType,
    pub data: PluginData,
}

/// Plugin data by type
pub enum PluginData {
    Wasm(Vec<u8>),
    Native(PathBuf),
    Script(String),
}

/// Discovery error
#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    #[error("Scan error: {0}")]
    Scan(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Manifest error: {0}")]
    Manifest(#[from] ManifestError),

    #[error("Validation errors: {0:?}")]
    Validation(Vec<String>),

    #[error("Invalid path: {0}")]
    InvalidPath(PathBuf),

    #[error("Watch error: {0}")]
    Watch(String),

    #[error("Plugin not found: {0}")]
    NotFound(String),
}

/// Load error
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("Entry point not found: {0}")]
    EntryNotFound(PathBuf),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Native library load error: {0}")]
    NativeLoad(String),

    #[error("Plugin not loaded: {0}")]
    NotLoaded(String),

    #[error("Symbol not found: {0} - {1}")]
    SymbolNotFound(String, String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DiscoveryConfig::default();
        assert_eq!(config.manifest_filename, "plugin.toml");
        assert!(!config.watch_enabled);
        assert_eq!(config.max_depth, 2);
    }

    #[test]
    fn test_plugin_source_equality() {
        assert_eq!(PluginSource::Standard, PluginSource::Standard);
        assert_ne!(PluginSource::Standard, PluginSource::MustUse);
    }
}
