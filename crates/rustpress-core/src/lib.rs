//! # RustPress Core
//!
//! Core types, traits, and interfaces for the RustPress CMS platform.
//! This crate defines all shared abstractions used across the system.

pub mod config;
pub mod error;
pub mod id;
pub mod context;
pub mod plugin;
pub mod hook;
pub mod service;
pub mod repository;
pub mod types;
pub mod tenant;
pub mod api;
pub mod health;
pub mod middleware;

// Re-exports for convenience
pub use config::AppConfig;
pub use error::{Error, Result};
pub use id::{Id, EntityId};
pub use context::{RequestContext, AppContext};
pub use plugin::{Plugin, PluginInfo, PluginManager};
pub use hook::{Hook, HookRegistry, Filter, Action};
pub use tenant::Tenant;
pub use id::TenantId;

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
