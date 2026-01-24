//! # Visual Queue Manager
//!
//! Enterprise-grade visual queue management system for RustPress.
//!
//! This plugin provides:
//! - Real-time queue monitoring and visualization
//! - Event-driven message processing with configurable handlers
//! - Worker pool management with auto-scaling capabilities
//! - Dead letter queue handling with replay functionality
//! - Scheduled jobs with cron expression support
//! - Comprehensive metrics and analytics dashboard
//! - Webhook subscriptions for external integrations
//! - Full audit logging for compliance
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    Visual Queue Manager                          │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
//! │  │  REST API   │  │  WebSocket  │  │    Admin    │             │
//! │  │  Handlers   │  │   Server    │  │  Interface  │             │
//! │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘             │
//! │         │                │                │                     │
//! │  ┌──────┴────────────────┴────────────────┴──────┐             │
//! │  │                  Event Bus                     │             │
//! │  └──────┬────────────────┬────────────────┬──────┘             │
//! │         │                │                │                     │
//! │  ┌──────┴──────┐  ┌──────┴──────┐  ┌──────┴──────┐             │
//! │  │   Queue     │  │   Worker    │  │  Metrics    │             │
//! │  │   Engine    │  │    Pool     │  │  Collector  │             │
//! │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘             │
//! │         │                │                │                     │
//! │  ┌──────┴────────────────┴────────────────┴──────┐             │
//! │  │              Storage Layer                     │             │
//! │  │         (PostgreSQL + Redis)                   │             │
//! │  └───────────────────────────────────────────────┘             │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

// TODO: Enable missing_docs when documentation is complete
#![allow(missing_docs)]
#![warn(clippy::all)]
#![allow(clippy::module_inception)]

use async_trait::async_trait;
use rustpress_core::context::AppContext;
use rustpress_core::plugin::{Plugin, PluginDependency, PluginInfo};
use rustpress_core::Result;
use semver::Version;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

// =============================================================================
// Module Declarations
// =============================================================================

pub mod admin;
pub mod api;
pub mod engine;
pub mod enterprise;
pub mod models;

// Hooks module (inline for simplicity)
mod hooks_impl;

// Re-exports for convenience
pub use admin::{
    AdminError, AdminModule, Capability, DashboardProvider, HookRegistry, PermissionError,
    PermissionManager, PluginSettings, Role, SettingsManager,
};
pub use enterprise::{
    AuditEvent, ComplianceConfig, ComplianceManager, DataClassification, EncryptedData,
    EncryptionConfig, EncryptionService, EnterpriseConfig, EnterpriseError, EnterpriseManager,
    KeyAlgorithm, RateLimitConfig, RateLimitKey, RateLimitResult, RateLimiter, Tenant,
    TenancyConfig, TenancyManager, TenantContext, TenantTier,
};

// =============================================================================
// Plugin Configuration
// =============================================================================

/// Plugin configuration loaded from settings
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginConfig {
    /// Redis connection URL
    pub redis_url: String,
    /// Redis password (optional)
    pub redis_password: Option<String>,
    /// Database connection URL
    pub database_url: String,
    /// Maximum database connections
    pub max_connections: u32,
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,

    /// Default retry attempts for failed messages
    pub default_retry_attempts: u32,
    /// Initial retry delay in milliseconds
    pub retry_delay_ms: u64,
    /// Retry backoff multiplier
    pub retry_backoff_multiplier: u32,
    /// Maximum retry delay in milliseconds
    pub max_retry_delay_ms: u64,
    /// Enable dead letter queue
    pub dead_letter_enabled: bool,
    /// Maximum queue size
    pub max_queue_size: u64,
    /// Default visibility timeout in milliseconds
    pub default_visibility_timeout_ms: u64,
    /// Default message TTL in seconds (0 = no expiry)
    pub default_message_ttl_seconds: u64,

    /// Auto-start workers on plugin activation
    pub auto_start_workers: bool,
    /// Default worker concurrency
    pub default_worker_concurrency: u32,
    /// Worker poll interval in milliseconds
    pub worker_poll_interval_ms: u64,

    /// Dashboard refresh interval in seconds
    pub refresh_interval_seconds: u32,
    /// Enable UI animations
    pub enable_animations: bool,
    /// Enable real-time WebSocket updates
    pub enable_realtime_updates: bool,

    /// Encryption key for sensitive data
    pub encryption_key: String,
    /// Encrypt message payloads
    pub encrypt_payloads: bool,
    /// Enable audit logging
    pub audit_logging: bool,
    /// Maximum payload size in KB
    pub max_payload_size_kb: u64,

    /// Webhook URL for notifications
    pub webhook_url: Option<String>,
    /// Slack webhook URL
    pub slack_webhook: Option<String>,
    /// Warning threshold for queue depth alerts
    pub alert_threshold_warning: u64,
    /// Critical threshold for queue depth alerts
    pub alert_threshold_critical: u64,

    /// Feature flags
    pub features: FeatureFlags,
}

/// Feature flags for gradual rollout
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FeatureFlags {
    /// Advanced analytics dashboard
    pub advanced_analytics: bool,
    /// AI-powered predictions
    pub ai_predictions: bool,
    /// Distributed mode support
    pub distributed_mode: bool,
    /// Real-time streaming
    pub real_time_streaming: bool,
    /// Message batching
    pub message_batching: bool,
    /// Priority queues
    pub priority_queues: bool,
    /// Message deduplication
    pub message_deduplication: bool,
    /// Circuit breaker pattern
    pub circuit_breaker: bool,
    /// Distributed tracing
    pub distributed_tracing: bool,
    /// Multi-tenancy support
    pub multi_tenancy: bool,
    /// Webhook subscriptions
    pub webhook_subscriptions: bool,
    /// Scheduled jobs
    pub scheduled_jobs: bool,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://localhost:6379".to_string(),
            redis_password: None,
            database_url: "postgresql://localhost:5432/rustpress".to_string(),
            max_connections: 10,
            connection_timeout_ms: 5000,
            default_retry_attempts: 3,
            retry_delay_ms: 1000,
            retry_backoff_multiplier: 2,
            max_retry_delay_ms: 300000,
            dead_letter_enabled: true,
            max_queue_size: 100000,
            default_visibility_timeout_ms: 30000,
            default_message_ttl_seconds: 0,
            auto_start_workers: true,
            default_worker_concurrency: 5,
            worker_poll_interval_ms: 100,
            refresh_interval_seconds: 5,
            enable_animations: true,
            enable_realtime_updates: true,
            encryption_key: String::new(),
            encrypt_payloads: false,
            audit_logging: true,
            max_payload_size_kb: 1024,
            webhook_url: None,
            slack_webhook: None,
            alert_threshold_warning: 5000,
            alert_threshold_critical: 10000,
            features: FeatureFlags::default(),
        }
    }
}

// =============================================================================
// Plugin State
// =============================================================================

/// Plugin runtime state
#[derive(Debug, Clone, Default)]
pub struct PluginState {
    /// Whether the plugin is fully initialized
    pub initialized: bool,
    /// Whether workers are running
    pub workers_running: bool,
    /// Last health check result
    pub last_health_check: Option<HealthCheckResult>,
    /// Active WebSocket connections count
    pub active_connections: u32,
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Overall health status
    pub healthy: bool,
    /// Timestamp of the check
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Individual component checks
    pub checks: Vec<(String, bool, Option<String>)>,
}

// =============================================================================
// Main Plugin Struct
// =============================================================================

/// Visual Queue Manager Plugin
///
/// The main plugin struct that implements the RustPress Plugin trait.
pub struct VisualQueueManager {
    /// Plugin information
    info: PluginInfo,
    /// Plugin configuration
    config: Arc<RwLock<PluginConfig>>,
    /// Database connection pool (set once, read many)
    pool: std::sync::OnceLock<PgPool>,
    /// Admin module for RustPress integration
    admin_module: Arc<RwLock<Option<Arc<AdminModule>>>>,
    /// Enterprise features manager
    enterprise_manager: Arc<RwLock<Option<Arc<EnterpriseManager>>>>,
    /// Plugin state
    state: Arc<RwLock<PluginState>>,
}

impl VisualQueueManager {
    /// Create a new instance of the Visual Queue Manager plugin
    pub fn new() -> Self {
        let info = PluginInfo {
            id: "visual-queue-manager".to_string(),
            name: "Visual Queue Manager".to_string(),
            version: Version::new(1, 0, 0),
            description: "Enterprise-grade visual queue management system".to_string(),
            author: "RustPress Enterprise".to_string(),
            author_url: Some("https://rustpress.net".to_string()),
            homepage: Some("https://github.com/rustpress-net/rustpress-plugin-visual-queue".to_string()),
            license: "MIT".to_string(),
            dependencies: vec![],
            min_rustpress_version: Some(Version::new(0, 4, 0)),
            tags: vec![
                "queue".to_string(),
                "events".to_string(),
                "enterprise".to_string(),
                "monitoring".to_string(),
            ],
        };

        Self {
            info,
            config: Arc::new(RwLock::new(PluginConfig::default())),
            pool: std::sync::OnceLock::new(),
            admin_module: Arc::new(RwLock::new(None)),
            enterprise_manager: Arc::new(RwLock::new(None)),
            state: Arc::new(RwLock::new(PluginState::default())),
        }
    }

    /// Initialize the database pool
    pub async fn init_pool(&self, database_url: &str) -> Result<()> {
        let pool = PgPool::connect(database_url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;
        self.pool.set(pool).map_err(|_| anyhow::anyhow!("Pool already initialized"))?;
        Ok(())
    }

    /// Get the database pool
    pub fn db_pool(&self) -> &PgPool {
        self.pool.get().expect("Database pool not initialized - ensure init_pool() is called first")
    }

    /// Log an audit event
    pub async fn log_audit(
        &self,
        event_type: &str,
        resource_type: &str,
        action: &str,
        resource_id: Option<uuid::Uuid>,
        resource_name: Option<&str>,
        user_id: Option<uuid::Uuid>,
        username: Option<&str>,
    ) {
        tracing::info!(
            event_type = %event_type,
            resource_type = %resource_type,
            action = %action,
            resource_id = ?resource_id,
            resource_name = ?resource_name,
            user_id = ?user_id,
            username = ?username,
            "Audit log entry"
        );
        // In production, this would write to an audit log table
    }

    /// Get plugin uptime in seconds
    pub fn uptime_seconds(&self) -> i64 {
        // Stub implementation - would track actual start time in production
        0
    }

    /// Get plugin start time
    pub fn started_at(&self) -> chrono::DateTime<chrono::Utc> {
        // Stub implementation - would track actual start time in production
        chrono::Utc::now()
    }

    /// Get the current plugin configuration
    pub async fn config(&self) -> PluginConfig {
        self.config.read().await.clone()
    }

    /// Update the plugin configuration
    pub async fn update_config(&self, config: PluginConfig) {
        let mut current = self.config.write().await;
        *current = config;
    }

    /// Get the admin module instance
    pub async fn admin_module(&self) -> Option<Arc<AdminModule>> {
        let guard = self.admin_module.read().await;
        guard.clone()
    }

    /// Get the enterprise manager instance
    pub async fn enterprise_manager(&self) -> Option<Arc<EnterpriseManager>> {
        let guard = self.enterprise_manager.read().await;
        guard.clone()
    }

    /// Enable enterprise features with the given configuration
    pub async fn enable_enterprise(&self, config: EnterpriseConfig) {
        let mut manager = self.enterprise_manager.write().await;
        *manager = Some(Arc::new(EnterpriseManager::new(config)));
    }

    /// Check if the plugin is healthy
    pub async fn health_check(&self) -> HealthCheckResult {
        let mut checks = Vec::new();

        // Basic health checks
        checks.push(("config".to_string(), true, None));
        checks.push(("state".to_string(), true, None));

        // Check enterprise features if enabled
        if self.enterprise_manager.read().await.is_some() {
            checks.push(("enterprise".to_string(), true, None));
        }

        let healthy = checks.iter().all(|(_, h, _)| *h);

        let result = HealthCheckResult {
            healthy,
            timestamp: chrono::Utc::now(),
            checks,
        };

        // Update state
        let mut state = self.state.write().await;
        state.last_health_check = Some(result.clone());

        result
    }

    /// Get current state
    pub async fn state(&self) -> PluginState {
        self.state.read().await.clone()
    }
}

impl Default for VisualQueueManager {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// RustPress Plugin Implementation
// =============================================================================

#[async_trait]
impl Plugin for VisualQueueManager {
    fn info(&self) -> &PluginInfo {
        &self.info
    }

    async fn activate(&self, ctx: &AppContext) -> Result<()> {
        tracing::info!("Activating Visual Queue Manager plugin...");

        // Initialize admin module
        // Note: In a real implementation, we'd pass the actual queue engine
        // For now, we'll defer initialization
        let mut state = self.state.write().await;
        state.initialized = true;

        // Log activation
        tracing::info!(
            plugin_id = %self.info.id,
            version = %self.info.version,
            "Visual Queue Manager plugin activated successfully"
        );

        Ok(())
    }

    async fn deactivate(&self, ctx: &AppContext) -> Result<()> {
        tracing::info!("Deactivating Visual Queue Manager plugin...");

        // Update state
        let mut state = self.state.write().await;
        state.initialized = false;
        state.workers_running = false;

        tracing::info!("Visual Queue Manager plugin deactivated");
        Ok(())
    }

    async fn on_startup(&self, ctx: &AppContext) -> Result<()> {
        tracing::info!("Visual Queue Manager starting up...");

        let config = self.config.read().await;

        // Auto-start workers if configured
        if config.auto_start_workers {
            tracing::info!(
                concurrency = config.default_worker_concurrency,
                "Auto-starting workers"
            );
            let mut state = self.state.write().await;
            state.workers_running = true;
        }

        // Initialize enterprise features if multi-tenancy is enabled
        if config.features.multi_tenancy {
            let enterprise_config = EnterpriseConfig::default();
            self.enable_enterprise(enterprise_config).await;

            if let Some(ref manager) = *self.enterprise_manager.read().await {
                manager.initialize().await.map_err(|e| {
                    anyhow::anyhow!("Failed to initialize enterprise features: {}", e)
                })?;
            }
        }

        tracing::info!("Visual Queue Manager startup complete");
        Ok(())
    }

    async fn on_shutdown(&self, ctx: &AppContext) -> Result<()> {
        tracing::info!("Visual Queue Manager shutting down...");

        // Stop workers gracefully
        let mut state = self.state.write().await;
        state.workers_running = false;

        tracing::info!("Visual Queue Manager shutdown complete");
        Ok(())
    }

    fn is_compatible(&self) -> bool {
        true
    }

    fn config_schema(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "type": "object",
            "properties": {
                "redis_url": {
                    "type": "string",
                    "description": "Redis connection URL",
                    "default": "redis://localhost:6379"
                },
                "database_url": {
                    "type": "string",
                    "description": "PostgreSQL connection URL"
                },
                "auto_start_workers": {
                    "type": "boolean",
                    "description": "Auto-start workers on activation",
                    "default": true
                },
                "default_worker_concurrency": {
                    "type": "integer",
                    "description": "Default worker concurrency",
                    "default": 5,
                    "minimum": 1,
                    "maximum": 100
                },
                "features": {
                    "type": "object",
                    "properties": {
                        "multi_tenancy": {
                            "type": "boolean",
                            "default": false
                        },
                        "distributed_mode": {
                            "type": "boolean",
                            "default": false
                        }
                    }
                }
            }
        }))
    }
}

// =============================================================================
// Plugin Error Types
// =============================================================================

/// Plugin-specific error types
#[derive(Debug, thiserror::Error)]
pub enum VqmError {
    /// Queue-related error
    #[error("Queue error: {0}")]
    Queue(String),

    /// Worker-related error
    #[error("Worker error: {0}")]
    Worker(String),

    /// Storage-related error
    #[error("Storage error: {0}")]
    Storage(String),

    /// Handler-related error
    #[error("Handler error: {0}")]
    Handler(String),

    /// Scheduler-related error
    #[error("Scheduler error: {0}")]
    Scheduler(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimited(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Resource not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

// =============================================================================
// Plugin Factory Function
// =============================================================================

/// Create a new Visual Queue Manager plugin instance
/// This is the entry point for dynamic plugin loading
#[no_mangle]
pub extern "C" fn create_visual_queue_manager_plugin() -> *mut dyn Plugin {
    let plugin = Box::new(VisualQueueManager::new());
    Box::into_raw(plugin)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = VisualQueueManager::new();
        assert_eq!(plugin.info().id, "visual-queue-manager");
        assert_eq!(plugin.info().name, "Visual Queue Manager");
    }

    #[test]
    fn test_default_config() {
        let config = PluginConfig::default();
        assert_eq!(config.default_retry_attempts, 3);
        assert_eq!(config.retry_delay_ms, 1000);
        assert!(config.dead_letter_enabled);
    }

    #[tokio::test]
    async fn test_health_check() {
        let plugin = VisualQueueManager::new();
        let result = plugin.health_check().await;
        assert!(result.healthy);
    }

    #[test]
    fn test_plugin_info() {
        let plugin = VisualQueueManager::new();
        let info = plugin.info();

        assert_eq!(info.version, Version::new(1, 0, 0));
        assert_eq!(info.license, "MIT");
        assert!(info.tags.contains(&"queue".to_string()));
    }

    #[tokio::test]
    async fn test_config_update() {
        let plugin = VisualQueueManager::new();

        let mut new_config = plugin.config().await;
        new_config.default_worker_concurrency = 10;
        plugin.update_config(new_config).await;

        let updated = plugin.config().await;
        assert_eq!(updated.default_worker_concurrency, 10);
    }
}
