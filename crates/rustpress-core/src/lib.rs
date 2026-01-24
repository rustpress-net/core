//! # RustPress Core
//!
//! Core types, traits, and interfaces for the RustPress CMS platform.
//! This crate defines all shared abstractions used across the system.

pub mod api;
pub mod config;
pub mod context;
pub mod discovery;
pub mod error;
pub mod health;
pub mod hook;
pub mod id;
pub mod middleware;
pub mod plugin;
pub mod plugin_loader;
pub mod repository;
pub mod service;
pub mod tenant;
pub mod types;

// Re-exports for convenience
pub use config::AppConfig;
pub use context::{AppContext, RequestContext};
pub use error::{Error, Result};
pub use hook::{Action, Filter, Hook, HookRegistry};
pub use id::TenantId;
pub use id::{EntityId, Id};
pub use plugin::{Plugin, PluginInfo, PluginManager};
pub use plugin_loader::{PluginLoader, PluginManifest, LoadResult};
pub use discovery::{DiscoveryService, DiscoverySource, DiscoveryConfig, ComponentType, ComponentManifest};
pub use tenant::Tenant;

/// The current version of RustPress
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Feature flags for conditional compilation
pub mod features {
    /// Whether metrics collection is enabled
    #[cfg(feature = "metrics")]
    pub const METRICS_ENABLED: bool = true;
    #[cfg(not(feature = "metrics"))]
    pub const METRICS_ENABLED: bool = false;
}
