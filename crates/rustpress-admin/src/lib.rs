//! RustPress Admin Dashboard
//!
//! Visual server management and administration interface for RustPress CMS.
//!
//! # Features
//!
//! - System status overview
//! - Database management
//! - Cache management
//! - CDN configuration
//! - Backup/restore operations
//! - User management
//! - Log viewer
//! - Performance monitoring

pub mod dashboard;
pub mod dbmanager;
pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod templates;
pub mod widgets;
pub mod functions;

pub use dashboard::*;
pub use routes::admin_router;
// Re-export PgPool for convenience
pub use sqlx::PgPool;
// Export dbmanager init function for persistence tables
pub use dbmanager::init_dbmanager_tables;

use serde::{Deserialize, Serialize};

/// Admin configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdminConfig {
    /// Admin panel path prefix
    #[serde(default = "default_prefix")]
    pub prefix: String,

    /// Enable admin panel
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Require authentication
    #[serde(default = "default_require_auth")]
    pub require_auth: bool,

    /// Allowed IP addresses (empty = all allowed)
    #[serde(default)]
    pub allowed_ips: Vec<String>,

    /// Session timeout in seconds
    #[serde(default = "default_session_timeout")]
    pub session_timeout: u64,

    /// Enable dark mode
    #[serde(default)]
    pub dark_mode: bool,
}

fn default_prefix() -> String {
    "/admin".to_string()
}

fn default_enabled() -> bool {
    true
}

fn default_require_auth() -> bool {
    true
}

fn default_session_timeout() -> u64 {
    3600
}

impl Default for AdminConfig {
    fn default() -> Self {
        Self {
            prefix: default_prefix(),
            enabled: default_enabled(),
            require_auth: default_require_auth(),
            allowed_ips: Vec::new(),
            session_timeout: default_session_timeout(),
            dark_mode: false,
        }
    }
}
