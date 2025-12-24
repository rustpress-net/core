//! # RustPress Plugins
//!
//! Plugin system for extending RustPress CMS functionality.
//!
//! This crate provides comprehensive infrastructure for loading, managing,
//! sandboxing, and executing plugins with WordPress-like capabilities.
//!
//! ## Features
//!
//! - **Plugin Manifest** - TOML-based plugin configuration
//! - **Discovery & Loading** - Automatic plugin discovery with hot reload
//! - **Dependency Resolution** - DAG-based topological sort for load order
//! - **WebAssembly Sandbox** - Secure plugin execution with Wasmtime
//! - **Lifecycle Management** - Activate, deactivate, upgrade, uninstall
//! - **Settings API** - Typed settings with schema validation
//! - **Database Migrations** - Version-tracked schema changes
//! - **Asset Management** - CSS/JS registration with dependencies
//! - **REST API Registration** - Custom endpoint registration
//! - **Admin Pages** - Admin menu and page registration
//! - **Shortcodes** - WordPress-compatible shortcode system
//! - **Blocks** - Gutenberg-style block registration
//! - **Widgets** - Dashboard and sidebar widgets
//! - **CLI Commands** - Custom CLI command registration
//! - **Cron Jobs** - Scheduled task registration
//! - **Capabilities** - Role-based permission system
//! - **Update Checking** - Automatic update detection
//! - **Marketplace** - Plugin marketplace integration
//! - **Conflict Detection** - Plugin incompatibility detection
//! - **Performance Monitoring** - Resource usage tracking
//! - **Error Isolation** - Automatic error containment
//! - **Config Export/Import** - Configuration portability
//! - **A/B Testing** - Experiment support for plugins
//! - **Feature Flags** - Gradual feature rollout
//! - **Inter-Plugin Communication** - Pub/sub messaging
//! - **Must-Use Plugins** - Auto-loaded essential plugins
//! - **Network Plugins** - Multi-site plugin support
//! - **Code Signing** - Plugin verification
//! - **Rollback** - Version rollback support
//! - **Usage Analytics** - Anonymous usage tracking

// Core modules (existing)
pub mod loader;
pub mod registry;
pub mod sandbox;

// Plugin manifest and discovery (Points 161-163)
pub mod manifest;
pub mod discovery;
pub mod dependencies;

// Plugin lifecycle and settings (Points 165-166)
pub mod lifecycle;
pub mod settings;

// Database and assets (Points 167-168)
pub mod migrations;
pub mod assets;

// API and admin (Points 169-170)
pub mod api;
pub mod admin;

// Registration systems (Points 171-176)
pub mod registrations;

// Updates and marketplace (Points 177-179)
pub mod updates;

// Monitoring and isolation (Points 180-181)
pub mod monitoring;

// Advanced features (Points 182-185)
pub mod features;

// Network and security (Points 186-190)
pub mod network;

// Cryptographic utilities for secrets and API keys
pub mod crypto;
// Re-export core types from existing modules
pub use loader::PluginLoader;
pub use registry::PluginRegistry;
pub use sandbox::PluginSandbox;

// Re-export manifest types (Point 161)
pub use manifest::{PluginManifest, PluginMeta, ManifestError};

// Re-export discovery types (Point 162)
pub use discovery::{PluginDiscovery, DiscoveredPlugin, PluginLoader as DiscoveryLoader};

// Re-export dependency types (Point 163)
pub use dependencies::DependencyResolver;

// Re-export sandbox types (Point 164)
pub use sandbox::{WasmPluginSandbox, WasmSandboxConfig, SandboxError, WasmValue};

// Re-export lifecycle types (Point 165)
pub use lifecycle::{LifecycleManager, PluginState, HookRegistry};

// Re-export settings types (Point 166)
pub use settings::{SettingsManager, SettingValue, SettingsSchema};

// Re-export migration types (Point 167)
pub use migrations::{MigrationManager, Migration, MigrationError};

// Re-export asset types (Point 168)
pub use assets::AssetManager;

// Re-export API types (Point 169)
pub use api::ApiRegistry;
pub use manifest::ApiEndpoint;

// Re-export admin types (Point 170)
pub use admin::AdminRegistry;
pub use manifest::AdminPage;

// Re-export registration types (Points 171-176)
pub use registrations::{
    ShortcodeRegistry,
    BlockRegistry,
    WidgetRegistry,
    CliRegistry,
    CronRegistry,
    CapabilityRegistry,
};

// Re-export update types (Points 177-179)
pub use updates::{
    UpdateChecker, UpdateInfo, UpdateConfig,
    MarketplaceClient, MarketplacePlugin,
    ConflictDetector, DetectedConflict,
};

// Re-export monitoring types (Points 180-181)
pub use monitoring::{
    PerformanceMonitor, PluginMetrics,
    ErrorIsolator,
};

// Re-export feature types (Points 182-185)
pub use features::{
    ConfigManager, PluginConfig,
    ABTestManager,
    FeatureFlagManager, FeatureFlag,
    PluginHub,
};

// Re-export network types (Points 186-190)
pub use network::{
    MustUseManager, MustUsePlugin,
    NetworkPluginManager, NetworkPlugin,
    SigningVerifier, VerificationResult,
    RollbackManager, PluginBackup,
    AnalyticsCollector, PluginUsage,
};

// Re-export crypto types for API key encryption
pub use crypto::{
    EncryptionKey, ApiKeyEncryptor, EncryptedValue,
    CryptoError, CryptoResult,
};

use rustpress_core::plugin::{Plugin, PluginInfo};
use std::sync::Arc;

/// Re-export core plugin types
pub use rustpress_core::plugin::PluginManager;

/// Plugin metadata stored in the registry
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub info: PluginInfo,
    pub path: Option<String>,
    pub enabled: bool,
    pub settings: serde_json::Value,
}

impl PluginMetadata {
    pub fn new(info: PluginInfo) -> Self {
        Self {
            info,
            path: None,
            enabled: false,
            settings: serde_json::Value::Object(Default::default()),
        }
    }

    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn enabled(mut self) -> Self {
        self.enabled = true;
        self
    }
}
