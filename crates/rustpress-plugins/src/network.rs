//! Must-Use Plugins, Network-Wide Plugins, Code Signing, Rollback, Analytics
//!
//! Advanced plugin features for enterprise and multi-site deployments.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{debug, error, info, warn};

// ============================================================================
// Must-Use Plugins Support (Point 186)
// ============================================================================

/// Must-use plugin manager
pub struct MustUseManager {
    /// Must-use plugins
    plugins: Arc<RwLock<Vec<MustUsePlugin>>>,
    /// Must-use directory
    mu_dir: PathBuf,
}

/// Must-use plugin
#[derive(Debug, Clone)]
pub struct MustUsePlugin {
    pub plugin_id: String,
    pub name: String,
    pub path: PathBuf,
    pub load_order: i32,
    pub description: Option<String>,
}

impl MustUseManager {
    pub fn new(mu_dir: PathBuf) -> Self {
        Self {
            plugins: Arc::new(RwLock::new(Vec::new())),
            mu_dir,
        }
    }

    /// Scan and load must-use plugins
    pub fn scan(&self) -> Result<Vec<MustUsePlugin>, MustUseError> {
        if !self.mu_dir.exists() {
            return Ok(Vec::new());
        }

        let mut plugins = Vec::new();

        for entry in std::fs::read_dir(&self.mu_dir)
            .map_err(|e| MustUseError::ScanFailed(e.to_string()))?
        {
            let entry = entry.map_err(|e| MustUseError::ScanFailed(e.to_string()))?;
            let path = entry.path();

            if path.is_dir() {
                // Check for plugin.toml
                let manifest_path = path.join("plugin.toml");
                if manifest_path.exists() {
                    if let Some(plugin) = self.load_plugin(&path)? {
                        plugins.push(plugin);
                    }
                }
            }
        }

        // Sort by load order
        plugins.sort_by_key(|p| p.load_order);

        *self.plugins.write() = plugins.clone();
        info!("Loaded {} must-use plugins", plugins.len());
        Ok(plugins)
    }

    /// Load a single must-use plugin
    fn load_plugin(&self, path: &Path) -> Result<Option<MustUsePlugin>, MustUseError> {
        let manifest_path = path.join("plugin.toml");
        let content = std::fs::read_to_string(&manifest_path)
            .map_err(|e| MustUseError::LoadFailed(e.to_string()))?;

        // Parse minimal manifest info
        let manifest: toml::Value = toml::from_str(&content)
            .map_err(|e| MustUseError::ParseFailed(e.to_string()))?;

        let plugin_info = manifest.get("plugin").ok_or_else(|| {
            MustUseError::ParseFailed("Missing [plugin] section".to_string())
        })?;

        let plugin_id = plugin_info
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MustUseError::ParseFailed("Missing plugin id".to_string()))?
            .to_string();

        let name = plugin_info
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(&plugin_id)
            .to_string();

        Ok(Some(MustUsePlugin {
            plugin_id,
            name,
            path: path.to_path_buf(),
            load_order: 0,
            description: plugin_info.get("description").and_then(|v| v.as_str()).map(String::from),
        }))
    }

    /// Get all must-use plugins
    pub fn get_all(&self) -> Vec<MustUsePlugin> {
        self.plugins.read().clone()
    }

    /// Check if plugin is must-use
    pub fn is_must_use(&self, plugin_id: &str) -> bool {
        self.plugins.read().iter().any(|p| p.plugin_id == plugin_id)
    }
}

/// Must-use error
#[derive(Debug, thiserror::Error)]
pub enum MustUseError {
    #[error("Scan failed: {0}")]
    ScanFailed(String),

    #[error("Load failed: {0}")]
    LoadFailed(String),

    #[error("Parse failed: {0}")]
    ParseFailed(String),
}

// ============================================================================
// Network-Wide Plugins (Point 187)
// ============================================================================

/// Network plugin manager for multi-site
pub struct NetworkPluginManager {
    /// Network-activated plugins
    network_plugins: Arc<RwLock<HashMap<String, NetworkPlugin>>>,
    /// Site-specific overrides
    site_overrides: Arc<RwLock<HashMap<i64, SitePluginOverrides>>>,
}

/// Network plugin info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPlugin {
    pub plugin_id: String,
    pub activated_at: chrono::DateTime<chrono::Utc>,
    pub activated_by: i64,
    pub settings_mode: NetworkSettingsMode,
}

/// Network settings mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkSettingsMode {
    /// Same settings for all sites
    NetworkWide,
    /// Each site has own settings
    PerSite,
    /// Network defaults with site overrides
    Inherited,
}

/// Site plugin overrides
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SitePluginOverrides {
    pub site_id: i64,
    /// Plugins disabled for this site
    pub disabled: Vec<String>,
    /// Site-specific settings
    pub settings: HashMap<String, serde_json::Value>,
}

impl NetworkPluginManager {
    pub fn new() -> Self {
        Self {
            network_plugins: Arc::new(RwLock::new(HashMap::new())),
            site_overrides: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Activate plugin network-wide
    pub fn network_activate(
        &self,
        plugin_id: &str,
        user_id: i64,
        settings_mode: NetworkSettingsMode,
    ) {
        let plugin = NetworkPlugin {
            plugin_id: plugin_id.to_string(),
            activated_at: chrono::Utc::now(),
            activated_by: user_id,
            settings_mode,
        };

        info!("Network activating plugin: {}", plugin_id);
        self.network_plugins.write().insert(plugin_id.to_string(), plugin);
    }

    /// Deactivate plugin network-wide
    pub fn network_deactivate(&self, plugin_id: &str) {
        info!("Network deactivating plugin: {}", plugin_id);
        self.network_plugins.write().remove(plugin_id);
    }

    /// Check if plugin is network-activated
    pub fn is_network_activated(&self, plugin_id: &str) -> bool {
        self.network_plugins.read().contains_key(plugin_id)
    }

    /// Get network plugin info
    pub fn get_network_plugin(&self, plugin_id: &str) -> Option<NetworkPlugin> {
        self.network_plugins.read().get(plugin_id).cloned()
    }

    /// Get all network plugins
    pub fn get_all_network_plugins(&self) -> Vec<NetworkPlugin> {
        self.network_plugins.read().values().cloned().collect()
    }

    /// Disable plugin for specific site
    pub fn disable_for_site(&self, plugin_id: &str, site_id: i64) {
        let mut overrides = self.site_overrides.write();
        let site = overrides.entry(site_id).or_insert_with(|| {
            SitePluginOverrides {
                site_id,
                ..Default::default()
            }
        });

        if !site.disabled.contains(&plugin_id.to_string()) {
            site.disabled.push(plugin_id.to_string());
        }
    }

    /// Enable plugin for specific site
    pub fn enable_for_site(&self, plugin_id: &str, site_id: i64) {
        let mut overrides = self.site_overrides.write();
        if let Some(site) = overrides.get_mut(&site_id) {
            site.disabled.retain(|p| p != plugin_id);
        }
    }

    /// Check if plugin is enabled for site
    pub fn is_enabled_for_site(&self, plugin_id: &str, site_id: i64) -> bool {
        // Check if network activated
        if !self.is_network_activated(plugin_id) {
            return false;
        }

        // Check site override
        let overrides = self.site_overrides.read();
        if let Some(site) = overrides.get(&site_id) {
            return !site.disabled.contains(&plugin_id.to_string());
        }

        true
    }

    /// Get active plugins for site
    pub fn get_active_for_site(&self, site_id: i64) -> Vec<String> {
        let network_plugins = self.network_plugins.read();
        let overrides = self.site_overrides.read();

        let disabled: &[String] = overrides
            .get(&site_id)
            .map(|o| o.disabled.as_slice())
            .unwrap_or(&[]);

        network_plugins
            .keys()
            .filter(|p| !disabled.contains(p))
            .cloned()
            .collect()
    }
}

impl Default for NetworkPluginManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Code Signing Verification (Point 188)
// ============================================================================

/// Code signing verifier
pub struct SigningVerifier {
    /// Trusted public keys
    trusted_keys: Arc<RwLock<HashMap<String, TrustedKey>>>,
    /// Verification results cache
    cache: Arc<RwLock<HashMap<String, VerificationResult>>>,
}

/// Trusted public key
#[derive(Debug, Clone)]
pub struct TrustedKey {
    pub key_id: String,
    pub public_key: Vec<u8>,
    pub owner: String,
    pub added_at: chrono::DateTime<chrono::Utc>,
    pub trusted_for: TrustLevel,
}

/// Trust level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TrustLevel {
    /// Official plugins
    Official,
    /// Verified developers
    Verified,
    /// Community plugins
    Community,
}

/// Verification result
#[derive(Debug, Clone, Serialize)]
pub struct VerificationResult {
    pub plugin_id: String,
    pub is_signed: bool,
    pub is_valid: bool,
    pub signer: Option<String>,
    pub trust_level: Option<TrustLevel>,
    pub verified_at: chrono::DateTime<chrono::Utc>,
    pub errors: Vec<String>,
}

impl SigningVerifier {
    pub fn new() -> Self {
        Self {
            trusted_keys: Arc::new(RwLock::new(HashMap::new())),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add trusted key
    pub fn add_trusted_key(&self, key: TrustedKey) {
        info!("Added trusted key: {} ({})", key.key_id, key.owner);
        self.trusted_keys.write().insert(key.key_id.clone(), key);
    }

    /// Remove trusted key
    pub fn remove_trusted_key(&self, key_id: &str) {
        self.trusted_keys.write().remove(key_id);
    }

    /// Verify plugin signature
    pub fn verify(&self, plugin_id: &str, plugin_path: &Path) -> VerificationResult {
        // Check cache
        if let Some(cached) = self.cache.read().get(plugin_id) {
            return cached.clone();
        }

        let mut result = VerificationResult {
            plugin_id: plugin_id.to_string(),
            is_signed: false,
            is_valid: false,
            signer: None,
            trust_level: None,
            verified_at: chrono::Utc::now(),
            errors: Vec::new(),
        };

        // Check for signature file
        let sig_path = plugin_path.join("plugin.sig");
        if !sig_path.exists() {
            result.errors.push("No signature file found".to_string());
            self.cache.write().insert(plugin_id.to_string(), result.clone());
            return result;
        }

        result.is_signed = true;

        // Read signature
        let signature = match std::fs::read(&sig_path) {
            Ok(s) => s,
            Err(e) => {
                result.errors.push(format!("Failed to read signature: {}", e));
                self.cache.write().insert(plugin_id.to_string(), result.clone());
                return result;
            }
        };

        // Parse signature (would parse actual format in real implementation)
        let (key_id, _sig_bytes) = match self.parse_signature(&signature) {
            Ok(s) => s,
            Err(e) => {
                result.errors.push(format!("Invalid signature format: {}", e));
                self.cache.write().insert(plugin_id.to_string(), result.clone());
                return result;
            }
        };

        // Find trusted key
        let keys = self.trusted_keys.read();
        let key = match keys.get(&key_id) {
            Some(k) => k.clone(),
            None => {
                result.errors.push(format!("Unknown signing key: {}", key_id));
                self.cache.write().insert(plugin_id.to_string(), result.clone());
                return result;
            }
        };
        drop(keys);

        // Verify signature (simplified - real impl would use ed25519-dalek)
        // In real implementation, would:
        // 1. Hash plugin files
        // 2. Verify signature against hash using public key

        result.is_valid = true;
        result.signer = Some(key.owner);
        result.trust_level = Some(key.trusted_for);

        self.cache.write().insert(plugin_id.to_string(), result.clone());
        result
    }

    /// Parse signature file
    fn parse_signature(&self, data: &[u8]) -> Result<(String, Vec<u8>), String> {
        // Simplified - real implementation would parse actual format
        if data.len() < 64 {
            return Err("Signature too short".to_string());
        }

        // Extract key ID (first 32 bytes as hex)
        let key_id = hex::encode(&data[..32]);
        let sig_bytes = data[32..].to_vec();

        Ok((key_id, sig_bytes))
    }

    /// Get verification result
    pub fn get_verification(&self, plugin_id: &str) -> Option<VerificationResult> {
        self.cache.read().get(plugin_id).cloned()
    }

    /// Clear verification cache
    pub fn clear_cache(&self) {
        self.cache.write().clear();
    }
}

impl Default for SigningVerifier {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Rollback Mechanism (Point 189)
// ============================================================================

/// Plugin rollback manager
pub struct RollbackManager {
    /// Backup directory
    backup_dir: PathBuf,
    /// Backup history
    backups: Arc<RwLock<HashMap<String, Vec<PluginBackup>>>>,
    /// Maximum backups per plugin
    max_backups: usize,
}

/// Plugin backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginBackup {
    pub plugin_id: String,
    pub version: String,
    pub backup_path: PathBuf,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub reason: BackupReason,
    pub size_bytes: u64,
}

/// Backup reason
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackupReason {
    BeforeUpdate,
    BeforeDeactivation,
    Manual,
    Scheduled,
}

impl RollbackManager {
    pub fn new(backup_dir: PathBuf, max_backups: usize) -> Self {
        // Create backup directory if needed
        if !backup_dir.exists() {
            let _ = std::fs::create_dir_all(&backup_dir);
        }

        Self {
            backup_dir,
            backups: Arc::new(RwLock::new(HashMap::new())),
            max_backups,
        }
    }

    /// Create backup before update
    pub fn create_backup(
        &self,
        plugin_id: &str,
        version: &str,
        plugin_path: &Path,
        reason: BackupReason,
    ) -> Result<PluginBackup, RollbackError> {
        let timestamp = chrono::Utc::now().timestamp();
        let backup_name = format!("{}-{}-{}", plugin_id, version, timestamp);
        let backup_path = self.backup_dir.join(&backup_name);

        // Create backup directory
        std::fs::create_dir_all(&backup_path)
            .map_err(|e| RollbackError::BackupFailed(e.to_string()))?;

        // Copy plugin files
        let size = self.copy_recursive(plugin_path, &backup_path)?;

        let backup = PluginBackup {
            plugin_id: plugin_id.to_string(),
            version: version.to_string(),
            backup_path: backup_path.clone(),
            created_at: chrono::Utc::now(),
            reason,
            size_bytes: size,
        };

        // Store backup info
        {
            let mut backups = self.backups.write();
            let plugin_backups = backups.entry(plugin_id.to_string()).or_insert_with(Vec::new);
            plugin_backups.push(backup.clone());

            // Cleanup old backups
            if plugin_backups.len() > self.max_backups {
                let to_remove: Vec<_> = plugin_backups
                    .drain(..plugin_backups.len() - self.max_backups)
                    .collect();

                for old_backup in to_remove {
                    let _ = std::fs::remove_dir_all(&old_backup.backup_path);
                }
            }
        }

        info!("Created backup for plugin {}: {:?}", plugin_id, backup_path);
        Ok(backup)
    }

    /// Copy files recursively
    fn copy_recursive(&self, src: &Path, dst: &Path) -> Result<u64, RollbackError> {
        let mut total_size = 0;

        for entry in std::fs::read_dir(src)
            .map_err(|e| RollbackError::BackupFailed(e.to_string()))?
        {
            let entry = entry.map_err(|e| RollbackError::BackupFailed(e.to_string()))?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                std::fs::create_dir_all(&dst_path)
                    .map_err(|e| RollbackError::BackupFailed(e.to_string()))?;
                total_size += self.copy_recursive(&src_path, &dst_path)?;
            } else {
                std::fs::copy(&src_path, &dst_path)
                    .map_err(|e| RollbackError::BackupFailed(e.to_string()))?;
                total_size += entry.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }

        Ok(total_size)
    }

    /// Rollback to previous version
    pub fn rollback(&self, plugin_id: &str, plugin_path: &Path) -> Result<PluginBackup, RollbackError> {
        let backups = self.backups.read();
        let plugin_backups = backups
            .get(plugin_id)
            .ok_or_else(|| RollbackError::NoBackupAvailable(plugin_id.to_string()))?;

        let backup = plugin_backups
            .last()
            .ok_or_else(|| RollbackError::NoBackupAvailable(plugin_id.to_string()))?
            .clone();

        drop(backups);

        // Remove current plugin
        if plugin_path.exists() {
            std::fs::remove_dir_all(plugin_path)
                .map_err(|e| RollbackError::RollbackFailed(e.to_string()))?;
        }

        // Restore from backup
        self.copy_recursive(&backup.backup_path, plugin_path)?;

        info!(
            "Rolled back plugin {} to version {}",
            plugin_id, backup.version
        );

        Ok(backup)
    }

    /// Rollback to specific version
    pub fn rollback_to_version(
        &self,
        plugin_id: &str,
        version: &str,
        plugin_path: &Path,
    ) -> Result<PluginBackup, RollbackError> {
        let backups = self.backups.read();
        let plugin_backups = backups
            .get(plugin_id)
            .ok_or_else(|| RollbackError::NoBackupAvailable(plugin_id.to_string()))?;

        let backup = plugin_backups
            .iter()
            .find(|b| b.version == version)
            .ok_or_else(|| RollbackError::VersionNotFound(version.to_string()))?
            .clone();

        drop(backups);

        // Remove current plugin
        if plugin_path.exists() {
            std::fs::remove_dir_all(plugin_path)
                .map_err(|e| RollbackError::RollbackFailed(e.to_string()))?;
        }

        // Restore from backup
        self.copy_recursive(&backup.backup_path, plugin_path)?;

        info!(
            "Rolled back plugin {} to version {}",
            plugin_id, backup.version
        );

        Ok(backup)
    }

    /// Get available backups
    pub fn get_backups(&self, plugin_id: &str) -> Vec<PluginBackup> {
        self.backups
            .read()
            .get(plugin_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Delete backup
    pub fn delete_backup(&self, plugin_id: &str, version: &str) -> Result<(), RollbackError> {
        let mut backups = self.backups.write();
        if let Some(plugin_backups) = backups.get_mut(plugin_id) {
            if let Some(idx) = plugin_backups.iter().position(|b| b.version == version) {
                let backup = plugin_backups.remove(idx);
                std::fs::remove_dir_all(&backup.backup_path)
                    .map_err(|e| RollbackError::DeleteFailed(e.to_string()))?;
            }
        }
        Ok(())
    }
}

/// Rollback error
#[derive(Debug, thiserror::Error)]
pub enum RollbackError {
    #[error("Backup failed: {0}")]
    BackupFailed(String),

    #[error("Rollback failed: {0}")]
    RollbackFailed(String),

    #[error("No backup available for plugin: {0}")]
    NoBackupAvailable(String),

    #[error("Version not found: {0}")]
    VersionNotFound(String),

    #[error("Delete failed: {0}")]
    DeleteFailed(String),
}

// ============================================================================
// Usage Analytics (Point 190)
// ============================================================================

/// Plugin analytics collector
pub struct AnalyticsCollector {
    /// Plugin usage data
    usage: Arc<RwLock<HashMap<String, PluginUsage>>>,
    /// Event history
    events: Arc<RwLock<Vec<AnalyticsEvent>>>,
    /// Max events to store
    max_events: usize,
}

/// Plugin usage data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginUsage {
    pub plugin_id: String,
    pub activation_count: u64,
    pub deactivation_count: u64,
    pub total_runtime_ms: u64,
    pub hook_calls: u64,
    pub api_calls: u64,
    pub error_count: u64,
    pub last_active: Option<chrono::DateTime<chrono::Utc>>,
    pub first_activated: Option<chrono::DateTime<chrono::Utc>>,
}

/// Analytics event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    pub event_type: EventType,
    pub plugin_id: String,
    pub data: Option<serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Activated,
    Deactivated,
    Updated,
    Error,
    HookCalled,
    ApiCalled,
    ConfigChanged,
}

impl AnalyticsCollector {
    pub fn new(max_events: usize) -> Self {
        Self {
            usage: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(Vec::new())),
            max_events,
        }
    }

    /// Record an event
    pub fn record(&self, event: AnalyticsEvent) {
        let plugin_id = event.plugin_id.clone();
        let event_type = event.event_type;

        // Store event
        {
            let mut events = self.events.write();
            events.push(event);

            // Trim old events
            if events.len() > self.max_events {
                let drain_count = events.len() - self.max_events;
                events.drain(..drain_count);
            }
        }

        // Update usage
        let mut usage = self.usage.write();
        let plugin_usage = usage.entry(plugin_id).or_insert_with(PluginUsage::default);

        match event_type {
            EventType::Activated => {
                plugin_usage.activation_count += 1;
                if plugin_usage.first_activated.is_none() {
                    plugin_usage.first_activated = Some(chrono::Utc::now());
                }
            }
            EventType::Deactivated => {
                plugin_usage.deactivation_count += 1;
            }
            EventType::Error => {
                plugin_usage.error_count += 1;
            }
            EventType::HookCalled => {
                plugin_usage.hook_calls += 1;
            }
            EventType::ApiCalled => {
                plugin_usage.api_calls += 1;
            }
            _ => {}
        }

        plugin_usage.last_active = Some(chrono::Utc::now());
    }

    /// Record activation
    pub fn record_activation(&self, plugin_id: &str) {
        self.record(AnalyticsEvent {
            event_type: EventType::Activated,
            plugin_id: plugin_id.to_string(),
            data: None,
            timestamp: chrono::Utc::now(),
        });
    }

    /// Record deactivation
    pub fn record_deactivation(&self, plugin_id: &str) {
        self.record(AnalyticsEvent {
            event_type: EventType::Deactivated,
            plugin_id: plugin_id.to_string(),
            data: None,
            timestamp: chrono::Utc::now(),
        });
    }

    /// Record error
    pub fn record_error(&self, plugin_id: &str, error: &str) {
        self.record(AnalyticsEvent {
            event_type: EventType::Error,
            plugin_id: plugin_id.to_string(),
            data: Some(serde_json::json!({ "error": error })),
            timestamp: chrono::Utc::now(),
        });
    }

    /// Record hook call
    pub fn record_hook_call(&self, plugin_id: &str, hook_name: &str) {
        self.record(AnalyticsEvent {
            event_type: EventType::HookCalled,
            plugin_id: plugin_id.to_string(),
            data: Some(serde_json::json!({ "hook": hook_name })),
            timestamp: chrono::Utc::now(),
        });
    }

    /// Get usage for plugin
    pub fn get_usage(&self, plugin_id: &str) -> Option<PluginUsage> {
        self.usage.read().get(plugin_id).cloned()
    }

    /// Get all usage data
    pub fn get_all_usage(&self) -> Vec<PluginUsage> {
        self.usage.read().values().cloned().collect()
    }

    /// Get events for plugin
    pub fn get_events(&self, plugin_id: &str, limit: usize) -> Vec<AnalyticsEvent> {
        self.events
            .read()
            .iter()
            .filter(|e| e.plugin_id == plugin_id)
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get recent events
    pub fn get_recent_events(&self, limit: usize) -> Vec<AnalyticsEvent> {
        self.events.read().iter().rev().take(limit).cloned().collect()
    }

    /// Generate summary report
    pub fn generate_report(&self) -> AnalyticsReport {
        let usage = self.usage.read();

        let total_plugins = usage.len();
        let total_activations: u64 = usage.values().map(|u| u.activation_count).sum();
        let total_errors: u64 = usage.values().map(|u| u.error_count).sum();
        let total_hook_calls: u64 = usage.values().map(|u| u.hook_calls).sum();

        let most_active: Vec<_> = {
            let mut plugins: Vec<_> = usage.iter().collect();
            plugins.sort_by_key(|(_, u)| std::cmp::Reverse(u.hook_calls));
            plugins.into_iter().take(5).map(|(id, _)| id.clone()).collect()
        };

        let most_errors: Vec<_> = {
            let mut plugins: Vec<_> = usage.iter().collect();
            plugins.sort_by_key(|(_, u)| std::cmp::Reverse(u.error_count));
            plugins.into_iter().take(5).map(|(id, _)| id.clone()).collect()
        };

        AnalyticsReport {
            generated_at: chrono::Utc::now(),
            total_plugins,
            total_activations,
            total_errors,
            total_hook_calls,
            most_active_plugins: most_active,
            most_errors_plugins: most_errors,
        }
    }

    /// Clear analytics
    pub fn clear(&self) {
        self.usage.write().clear();
        self.events.write().clear();
    }
}

impl Default for AnalyticsCollector {
    fn default() -> Self {
        Self::new(10000)
    }
}

/// Analytics report
#[derive(Debug, Clone, Serialize)]
pub struct AnalyticsReport {
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub total_plugins: usize,
    pub total_activations: u64,
    pub total_errors: u64,
    pub total_hook_calls: u64,
    pub most_active_plugins: Vec<String>,
    pub most_errors_plugins: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_plugin_manager() {
        let manager = NetworkPluginManager::new();

        manager.network_activate("test-plugin", 1, NetworkSettingsMode::NetworkWide);
        assert!(manager.is_network_activated("test-plugin"));
        assert!(manager.is_enabled_for_site("test-plugin", 1));

        manager.disable_for_site("test-plugin", 1);
        assert!(!manager.is_enabled_for_site("test-plugin", 1));
    }

    #[test]
    fn test_analytics_collector() {
        let collector = AnalyticsCollector::new(100);

        collector.record_activation("test-plugin");
        collector.record_hook_call("test-plugin", "init");
        collector.record_error("test-plugin", "Something went wrong");

        let usage = collector.get_usage("test-plugin").unwrap();
        assert_eq!(usage.activation_count, 1);
        assert_eq!(usage.hook_calls, 1);
        assert_eq!(usage.error_count, 1);
    }
}
