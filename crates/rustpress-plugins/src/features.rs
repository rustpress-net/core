//! Plugin Features: Configuration Export/Import, A/B Testing, Feature Flags, Inter-communication
//!
//! Advanced plugin features for configuration management and experimentation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{debug, info};
use rand::Rng;

// ============================================================================
// Configuration Export/Import (Point 182)
// ============================================================================

/// Configuration manager for export/import
pub struct ConfigManager {
    /// Plugin configurations
    configs: Arc<RwLock<HashMap<String, PluginConfig>>>,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub plugin_id: String,
    pub version: String,
    pub settings: HashMap<String, serde_json::Value>,
    pub enabled_features: Vec<String>,
    pub custom_data: Option<serde_json::Value>,
    pub exported_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Export format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    Toml,
    Yaml,
}

/// Import result
#[derive(Debug, Clone)]
pub struct ImportResult {
    pub plugin_id: String,
    pub success: bool,
    pub imported_settings: usize,
    pub skipped_settings: Vec<String>,
    pub errors: Vec<String>,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            configs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Store configuration
    pub fn store(&self, config: PluginConfig) {
        self.configs.write().insert(config.plugin_id.clone(), config);
    }

    /// Export plugin configuration
    pub fn export(&self, plugin_id: &str, format: ExportFormat) -> Result<String, ConfigError> {
        let configs = self.configs.read();
        let config = configs
            .get(plugin_id)
            .ok_or_else(|| ConfigError::NotFound(plugin_id.to_string()))?;

        let mut export_config = config.clone();
        export_config.exported_at = Some(chrono::Utc::now());

        match format {
            ExportFormat::Json => {
                serde_json::to_string_pretty(&export_config)
                    .map_err(|e| ConfigError::SerializationFailed(e.to_string()))
            }
            ExportFormat::Toml => {
                toml::to_string_pretty(&export_config)
                    .map_err(|e| ConfigError::SerializationFailed(e.to_string()))
            }
            ExportFormat::Yaml => {
                // Would use serde_yaml in real implementation
                Err(ConfigError::UnsupportedFormat("yaml".to_string()))
            }
        }
    }

    /// Export multiple plugins
    pub fn export_all(&self, plugin_ids: &[String], format: ExportFormat) -> Result<String, ConfigError> {
        let configs = self.configs.read();
        let mut export_data: HashMap<String, PluginConfig> = HashMap::new();

        for id in plugin_ids {
            if let Some(config) = configs.get(id) {
                let mut export_config = config.clone();
                export_config.exported_at = Some(chrono::Utc::now());
                export_data.insert(id.clone(), export_config);
            }
        }

        match format {
            ExportFormat::Json => {
                serde_json::to_string_pretty(&export_data)
                    .map_err(|e| ConfigError::SerializationFailed(e.to_string()))
            }
            ExportFormat::Toml => {
                toml::to_string_pretty(&export_data)
                    .map_err(|e| ConfigError::SerializationFailed(e.to_string()))
            }
            ExportFormat::Yaml => {
                Err(ConfigError::UnsupportedFormat("yaml".to_string()))
            }
        }
    }

    /// Import plugin configuration
    pub fn import(&self, data: &str, format: ExportFormat) -> Result<ImportResult, ConfigError> {
        let config: PluginConfig = match format {
            ExportFormat::Json => {
                serde_json::from_str(data)
                    .map_err(|e| ConfigError::DeserializationFailed(e.to_string()))?
            }
            ExportFormat::Toml => {
                toml::from_str(data)
                    .map_err(|e| ConfigError::DeserializationFailed(e.to_string()))?
            }
            ExportFormat::Yaml => {
                return Err(ConfigError::UnsupportedFormat("yaml".to_string()));
            }
        };

        let plugin_id = config.plugin_id.clone();
        let settings_count = config.settings.len();

        self.configs.write().insert(plugin_id.clone(), config);

        info!("Imported configuration for plugin: {}", plugin_id);

        Ok(ImportResult {
            plugin_id,
            success: true,
            imported_settings: settings_count,
            skipped_settings: Vec::new(),
            errors: Vec::new(),
        })
    }

    /// Get configuration
    pub fn get(&self, plugin_id: &str) -> Option<PluginConfig> {
        self.configs.read().get(plugin_id).cloned()
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration error
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Serialization failed: {0}")]
    SerializationFailed(String),

    #[error("Deserialization failed: {0}")]
    DeserializationFailed(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: String, actual: String },
}

// ============================================================================
// A/B Testing Support (Point 183)
// ============================================================================

/// A/B testing manager
pub struct ABTestManager {
    /// Active experiments
    experiments: Arc<RwLock<HashMap<String, Experiment>>>,
    /// User assignments
    assignments: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
    /// Results
    results: Arc<RwLock<HashMap<String, ExperimentResults>>>,
}

/// Experiment definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experiment {
    pub id: String,
    pub plugin_id: String,
    pub name: String,
    pub description: Option<String>,
    pub variants: Vec<Variant>,
    pub status: ExperimentStatus,
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
    pub targeting: Option<Targeting>,
    pub goal: Option<String>,
}

/// Experiment variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variant {
    pub id: String,
    pub name: String,
    pub weight: u32,
    pub config: HashMap<String, serde_json::Value>,
}

/// Experiment status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExperimentStatus {
    Draft,
    Running,
    Paused,
    Completed,
    Archived,
}

/// Targeting rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Targeting {
    pub user_roles: Option<Vec<String>>,
    pub user_ids: Option<Vec<i64>>,
    pub percentage: Option<u8>,
    pub conditions: Option<Vec<TargetingCondition>>,
}

/// Targeting condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetingCondition {
    pub field: String,
    pub operator: String,
    pub value: serde_json::Value,
}

/// Experiment results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExperimentResults {
    pub experiment_id: String,
    pub variant_stats: HashMap<String, VariantStats>,
    pub total_participants: u64,
    pub conversions: u64,
    pub statistical_significance: Option<f64>,
}

/// Variant statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VariantStats {
    pub participants: u64,
    pub conversions: u64,
    pub conversion_rate: f64,
}

impl ABTestManager {
    pub fn new() -> Self {
        Self {
            experiments: Arc::new(RwLock::new(HashMap::new())),
            assignments: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create an experiment
    pub fn create_experiment(&self, experiment: Experiment) -> String {
        let id = experiment.id.clone();
        self.experiments.write().insert(id.clone(), experiment);
        self.results.write().insert(
            id.clone(),
            ExperimentResults {
                experiment_id: id.clone(),
                ..Default::default()
            },
        );
        id
    }

    /// Get experiment
    pub fn get_experiment(&self, experiment_id: &str) -> Option<Experiment> {
        self.experiments.read().get(experiment_id).cloned()
    }

    /// Start experiment
    pub fn start_experiment(&self, experiment_id: &str) -> Result<(), ABTestError> {
        let mut experiments = self.experiments.write();
        let experiment = experiments
            .get_mut(experiment_id)
            .ok_or_else(|| ABTestError::NotFound(experiment_id.to_string()))?;

        experiment.status = ExperimentStatus::Running;
        experiment.start_date = Some(chrono::Utc::now());
        Ok(())
    }

    /// Stop experiment
    pub fn stop_experiment(&self, experiment_id: &str) -> Result<(), ABTestError> {
        let mut experiments = self.experiments.write();
        let experiment = experiments
            .get_mut(experiment_id)
            .ok_or_else(|| ABTestError::NotFound(experiment_id.to_string()))?;

        experiment.status = ExperimentStatus::Completed;
        experiment.end_date = Some(chrono::Utc::now());
        Ok(())
    }

    /// Get variant for user
    pub fn get_variant(&self, experiment_id: &str, user_id: &str) -> Option<Variant> {
        // Check existing assignment
        if let Some(variant_id) = self.get_assignment(experiment_id, user_id) {
            let experiments = self.experiments.read();
            if let Some(experiment) = experiments.get(experiment_id) {
                return experiment.variants.iter().find(|v| v.id == variant_id).cloned();
            }
        }

        // Assign new variant
        let experiments = self.experiments.read();
        let experiment = experiments.get(experiment_id)?;

        if experiment.status != ExperimentStatus::Running {
            return None;
        }

        // Check targeting
        if let Some(targeting) = &experiment.targeting {
            if let Some(percentage) = targeting.percentage {
                let hash = Self::hash_user(user_id);
                if hash > percentage {
                    return None; // User not in experiment
                }
            }
        }

        // Weighted random assignment
        let variant = self.weighted_random_variant(&experiment.variants)?;
        drop(experiments);

        self.set_assignment(experiment_id, user_id, &variant.id);
        Some(variant)
    }

    /// Get user assignment
    fn get_assignment(&self, experiment_id: &str, user_id: &str) -> Option<String> {
        self.assignments
            .read()
            .get(user_id)?
            .get(experiment_id)
            .cloned()
    }

    /// Set user assignment
    fn set_assignment(&self, experiment_id: &str, user_id: &str, variant_id: &str) {
        let mut assignments = self.assignments.write();
        assignments
            .entry(user_id.to_string())
            .or_insert_with(HashMap::new)
            .insert(experiment_id.to_string(), variant_id.to_string());
    }

    /// Hash user for percentage targeting
    fn hash_user(user_id: &str) -> u8 {
        let hash = blake3::hash(user_id.as_bytes());
        hash.as_bytes()[0] % 100
    }

    /// Weighted random variant selection
    fn weighted_random_variant(&self, variants: &[Variant]) -> Option<Variant> {
        let total_weight: u32 = variants.iter().map(|v| v.weight).sum();
        if total_weight == 0 {
            return variants.first().cloned();
        }

        let mut rng = rand::thread_rng();
        let roll: u32 = rng.gen_range(0..total_weight);

        let mut cumulative = 0;
        for variant in variants {
            cumulative += variant.weight;
            if roll < cumulative {
                return Some(variant.clone());
            }
        }

        variants.first().cloned()
    }

    /// Record conversion
    pub fn record_conversion(&self, experiment_id: &str, user_id: &str) {
        let variant_id = match self.get_assignment(experiment_id, user_id) {
            Some(v) => v,
            None => return,
        };

        let mut results = self.results.write();
        if let Some(result) = results.get_mut(experiment_id) {
            result.conversions += 1;
            let variant_stats = result
                .variant_stats
                .entry(variant_id)
                .or_insert_with(VariantStats::default);
            variant_stats.conversions += 1;
            if variant_stats.participants > 0 {
                variant_stats.conversion_rate =
                    variant_stats.conversions as f64 / variant_stats.participants as f64;
            }
        }
    }

    /// Get experiment results
    pub fn get_results(&self, experiment_id: &str) -> Option<ExperimentResults> {
        self.results.read().get(experiment_id).cloned()
    }
}

impl Default for ABTestManager {
    fn default() -> Self {
        Self::new()
    }
}

/// A/B test error
#[derive(Debug, thiserror::Error)]
pub enum ABTestError {
    #[error("Experiment not found: {0}")]
    NotFound(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),
}

// ============================================================================
// Feature Flags (Point 184)
// ============================================================================

/// Feature flag manager
pub struct FeatureFlagManager {
    /// Feature flags
    flags: Arc<RwLock<HashMap<String, FeatureFlag>>>,
    /// User overrides
    overrides: Arc<RwLock<HashMap<String, HashMap<String, bool>>>>,
}

/// Feature flag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    pub id: String,
    pub plugin_id: String,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub rollout_percentage: Option<u8>,
    pub conditions: Vec<FlagCondition>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Flag condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagCondition {
    pub condition_type: FlagConditionType,
    pub value: serde_json::Value,
}

/// Condition type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlagConditionType {
    UserRole,
    UserId,
    Environment,
    Custom,
}

impl FeatureFlagManager {
    pub fn new() -> Self {
        Self {
            flags: Arc::new(RwLock::new(HashMap::new())),
            overrides: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a feature flag
    pub fn register(&self, flag: FeatureFlag) {
        debug!("Registered feature flag: {}", flag.id);
        self.flags.write().insert(flag.id.clone(), flag);
    }

    /// Check if feature is enabled
    pub fn is_enabled(&self, flag_id: &str, context: &FeatureContext) -> bool {
        // Check user override first
        if let Some(override_value) = self.get_override(flag_id, &context.user_id) {
            return override_value;
        }

        let flags = self.flags.read();
        let flag = match flags.get(flag_id) {
            Some(f) => f,
            None => return false,
        };

        if !flag.enabled {
            return false;
        }

        // Check rollout percentage
        if let Some(percentage) = flag.rollout_percentage {
            let hash = Self::hash_context(&context.user_id, flag_id);
            if hash >= percentage {
                return false;
            }
        }

        // Check conditions
        for condition in &flag.conditions {
            if !self.evaluate_condition(condition, context) {
                return false;
            }
        }

        true
    }

    /// Evaluate a condition
    fn evaluate_condition(&self, condition: &FlagCondition, context: &FeatureContext) -> bool {
        match &condition.condition_type {
            FlagConditionType::UserRole => {
                if let Some(roles) = condition.value.as_array() {
                    let role_strings: Vec<&str> = roles
                        .iter()
                        .filter_map(|r| r.as_str())
                        .collect();
                    return context.user_roles.iter().any(|r| role_strings.contains(&r.as_str()));
                }
                false
            }
            FlagConditionType::UserId => {
                if let Some(ids) = condition.value.as_array() {
                    return ids.iter().any(|id| {
                        id.as_str() == Some(&context.user_id) ||
                        id.as_i64().map(|i| i.to_string()) == Some(context.user_id.clone())
                    });
                }
                false
            }
            FlagConditionType::Environment => {
                condition.value.as_str() == Some(&context.environment)
            }
            FlagConditionType::Custom => {
                // Would call custom evaluator
                true
            }
        }
    }

    /// Hash context for rollout
    fn hash_context(user_id: &str, flag_id: &str) -> u8 {
        let combined = format!("{}:{}", user_id, flag_id);
        let hash = blake3::hash(combined.as_bytes());
        hash.as_bytes()[0] % 100
    }

    /// Set user override
    pub fn set_override(&self, flag_id: &str, user_id: &str, enabled: bool) {
        let mut overrides = self.overrides.write();
        overrides
            .entry(user_id.to_string())
            .or_insert_with(HashMap::new)
            .insert(flag_id.to_string(), enabled);
    }

    /// Get user override
    fn get_override(&self, flag_id: &str, user_id: &str) -> Option<bool> {
        self.overrides
            .read()
            .get(user_id)?
            .get(flag_id)
            .copied()
    }

    /// Clear user override
    pub fn clear_override(&self, flag_id: &str, user_id: &str) {
        let mut overrides = self.overrides.write();
        if let Some(user_overrides) = overrides.get_mut(user_id) {
            user_overrides.remove(flag_id);
        }
    }

    /// Get all flags
    pub fn get_all(&self) -> Vec<FeatureFlag> {
        self.flags.read().values().cloned().collect()
    }

    /// Get flags for plugin
    pub fn get_for_plugin(&self, plugin_id: &str) -> Vec<FeatureFlag> {
        self.flags
            .read()
            .values()
            .filter(|f| f.plugin_id == plugin_id)
            .cloned()
            .collect()
    }

    /// Update flag
    pub fn update(&self, flag_id: &str, enabled: bool) {
        let mut flags = self.flags.write();
        if let Some(flag) = flags.get_mut(flag_id) {
            flag.enabled = enabled;
            flag.updated_at = chrono::Utc::now();
        }
    }

    /// Delete flag
    pub fn delete(&self, flag_id: &str) {
        self.flags.write().remove(flag_id);
    }
}

impl Default for FeatureFlagManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Feature context
#[derive(Debug, Clone, Default)]
pub struct FeatureContext {
    pub user_id: String,
    pub user_roles: Vec<String>,
    pub environment: String,
    pub custom: HashMap<String, serde_json::Value>,
}

// ============================================================================
// Inter-communication API (Point 185)
// ============================================================================

/// Plugin communication hub
pub struct PluginHub {
    /// Message channels
    channels: Arc<RwLock<HashMap<String, Vec<Subscriber>>>>,
    /// Registered services
    services: Arc<RwLock<HashMap<String, RegisteredService>>>,
    /// Message history
    history: Arc<RwLock<Vec<Message>>>,
}

/// Subscriber
#[derive(Clone)]
struct Subscriber {
    plugin_id: String,
    callback_id: String,
}

/// Registered service
#[derive(Debug, Clone)]
pub struct RegisteredService {
    pub plugin_id: String,
    pub service_name: String,
    pub version: String,
    pub methods: Vec<String>,
}

/// Message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub from_plugin: String,
    pub channel: String,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl PluginHub {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            services: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Subscribe to a channel
    pub fn subscribe(&self, channel: &str, plugin_id: &str, callback_id: &str) {
        let mut channels = self.channels.write();
        let subscribers = channels.entry(channel.to_string()).or_insert_with(Vec::new);
        subscribers.push(Subscriber {
            plugin_id: plugin_id.to_string(),
            callback_id: callback_id.to_string(),
        });
        debug!("Plugin {} subscribed to channel {}", plugin_id, channel);
    }

    /// Unsubscribe from a channel
    pub fn unsubscribe(&self, channel: &str, plugin_id: &str) {
        let mut channels = self.channels.write();
        if let Some(subscribers) = channels.get_mut(channel) {
            subscribers.retain(|s| s.plugin_id != plugin_id);
        }
    }

    /// Publish message to channel
    pub fn publish(&self, from_plugin: &str, channel: &str, payload: serde_json::Value) -> Vec<String> {
        let message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            from_plugin: from_plugin.to_string(),
            channel: channel.to_string(),
            payload,
            timestamp: chrono::Utc::now(),
        };

        // Store in history
        self.history.write().push(message.clone());

        // Get subscribers
        let subscribers = match self.channels.read().get(channel).cloned() {
            Some(s) => s,
            None => return Vec::new(),
        };

        // Return callback IDs for execution
        subscribers
            .iter()
            .filter(|s| s.plugin_id != from_plugin)
            .map(|s| s.callback_id.clone())
            .collect()
    }

    /// Register a service
    pub fn register_service(&self, service: RegisteredService) {
        let key = format!("{}:{}", service.plugin_id, service.service_name);
        info!(
            "Plugin {} registered service: {}",
            service.plugin_id, service.service_name
        );
        self.services.write().insert(key, service);
    }

    /// Get service
    pub fn get_service(&self, plugin_id: &str, service_name: &str) -> Option<RegisteredService> {
        let key = format!("{}:{}", plugin_id, service_name);
        self.services.read().get(&key).cloned()
    }

    /// Find services by name
    pub fn find_services(&self, service_name: &str) -> Vec<RegisteredService> {
        self.services
            .read()
            .values()
            .filter(|s| s.service_name == service_name)
            .cloned()
            .collect()
    }

    /// Unregister plugin services
    pub fn unregister_plugin(&self, plugin_id: &str) {
        self.services
            .write()
            .retain(|_, s| s.plugin_id != plugin_id);

        let mut channels = self.channels.write();
        for subscribers in channels.values_mut() {
            subscribers.retain(|s| s.plugin_id != plugin_id);
        }
    }

    /// Get message history
    pub fn get_history(&self, channel: Option<&str>, limit: usize) -> Vec<Message> {
        let history = self.history.read();
        let filtered: Vec<_> = if let Some(ch) = channel {
            history.iter().filter(|m| m.channel == ch).cloned().collect()
        } else {
            history.iter().cloned().collect()
        };

        filtered.into_iter().rev().take(limit).collect()
    }
}

impl Default for PluginHub {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_flag_basic() {
        let manager = FeatureFlagManager::new();

        manager.register(FeatureFlag {
            id: "test-flag".to_string(),
            plugin_id: "test-plugin".to_string(),
            name: "Test Feature".to_string(),
            description: None,
            enabled: true,
            rollout_percentage: None,
            conditions: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        });

        let context = FeatureContext::default();
        assert!(manager.is_enabled("test-flag", &context));
    }

    #[test]
    fn test_feature_flag_disabled() {
        let manager = FeatureFlagManager::new();

        manager.register(FeatureFlag {
            id: "disabled-flag".to_string(),
            plugin_id: "test-plugin".to_string(),
            name: "Disabled Feature".to_string(),
            description: None,
            enabled: false,
            rollout_percentage: None,
            conditions: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        });

        let context = FeatureContext::default();
        assert!(!manager.is_enabled("disabled-flag", &context));
    }

    #[test]
    fn test_plugin_hub_pubsub() {
        let hub = PluginHub::new();

        hub.subscribe("events", "plugin-b", "on_event");
        let callbacks = hub.publish(
            "plugin-a",
            "events",
            serde_json::json!({"type": "test"}),
        );

        assert_eq!(callbacks.len(), 1);
        assert_eq!(callbacks[0], "on_event");
    }
}
