//! # RustCloudflare
//!
//! Seamless Cloudflare integration for RustPress - CDN, DNS, Security, Workers, R2, KV, D1, and more.
//!
//! This plugin provides comprehensive Cloudflare management capabilities directly from your
//! RustPress admin dashboard, including:
//!
//! - **CDN Management**: Cache control, purging, and optimization
//! - **DNS Management**: Full DNS record CRUD operations
//! - **Security**: WAF rules, firewall, bot management, DDoS protection
//! - **SSL/TLS**: Certificate management and configuration
//! - **Workers**: Deploy and manage Cloudflare Workers
//! - **KV Storage**: Workers KV namespace and key management
//! - **R2 Storage**: S3-compatible object storage for media
//! - **D1 Database**: Edge SQL database integration
//! - **Stream**: Video hosting and live streaming
//! - **Analytics**: Traffic, performance, and security analytics

pub mod api;
pub mod client;
pub mod config;
pub mod error;
pub mod hooks;
pub mod middleware;
pub mod models;
pub mod services;
pub mod workers;

use async_trait::async_trait;
use axum::Router;
use rustpress_core::{
    error::Result,
    plugin::{Plugin, PluginInfo, PluginState},
    AppContext,
};
use crate::error::CloudflareResult;
use rustpress_plugins::lifecycle::{
    ActivationContext, DeactivationContext, HookError, InitContext, LifecycleHook,
    LoadContext, ShutdownContext, UninstallContext, UpgradeContext,
};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::client::CloudflareClient;
use crate::config::CloudflareConfig;
use crate::services::CloudflareServices;

/// Current plugin version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Plugin identifier
pub const PLUGIN_ID: &str = "rustcloudflare";

/// Main RustCloudflare plugin struct
pub struct RustCloudflarePlugin {
    info: PluginInfo,
    state: RwLock<PluginState>,
    config: RwLock<Option<CloudflareConfig>>,
    client: RwLock<Option<Arc<CloudflareClient>>>,
    services: RwLock<Option<Arc<CloudflareServices>>>,
    db_pool: RwLock<Option<PgPool>>,
}

impl RustCloudflarePlugin {
    /// Create a new instance of the plugin
    pub fn new() -> Self {
        Self {
            info: PluginInfo {
                id: PLUGIN_ID.to_string(),
                name: "RustCloudflare".to_string(),
                version: semver::Version::parse(VERSION).unwrap_or_else(|_| {
                    semver::Version::new(1, 0, 0)
                }),
                description: "Seamless Cloudflare integration for RustPress".to_string(),
                author: "RustPress Team".to_string(),
                author_url: Some("https://rustpress.io".to_string()),
                homepage: Some("https://rustpress.io/plugins/rustcloudflare".to_string()),
                license: "MIT".to_string(),
                dependencies: vec![],
                min_rustpress_version: Some(semver::Version::new(1, 0, 0)),
                tags: vec![
                    "cloudflare".to_string(),
                    "cdn".to_string(),
                    "security".to_string(),
                    "performance".to_string(),
                    "dns".to_string(),
                    "workers".to_string(),
                    "edge".to_string(),
                ],
            },
            state: RwLock::new(PluginState::Inactive),
            config: RwLock::new(None),
            client: RwLock::new(None),
            services: RwLock::new(None),
            db_pool: RwLock::new(None),
        }
    }

    /// Initialize the plugin with a database pool
    pub async fn init_with_pool(&self, pool: PgPool) -> CloudflareResult<()> {
        *self.db_pool.write().await = Some(pool.clone());

        // Try to load configuration from environment
        match CloudflareConfig::from_env() {
            Ok(config) => {
                // Create the Cloudflare API client
                let client = Arc::new(CloudflareClient::new(&config)?);

                // Verify connection
                client.verify_connection().await?;

                // Create services layer
                let services = Arc::new(CloudflareServices::new(
                    Arc::clone(&client),
                    pool,
                ));

                // Store in plugin state
                *self.config.write().await = Some(config);
                *self.client.write().await = Some(client);
                *self.services.write().await = Some(services);

                info!("Cloudflare client initialized from environment");
            }
            Err(e) => {
                warn!("Cloudflare not configured from environment: {}", e);
                // Create unconfigured services for settings management
                let services = Arc::new(CloudflareServices::new_unconfigured(pool));
                *self.services.write().await = Some(services);
            }
        }

        Ok(())
    }

    /// Initialize client with stored settings
    pub async fn init_with_settings(&self, settings: serde_json::Value) -> CloudflareResult<()> {
        let config = CloudflareConfig::from_settings(settings)?;

        let pool = self.db_pool.read().await.clone()
            .ok_or_else(|| error::CloudflareError::NotConfigured)?;

        // Create the Cloudflare API client
        let client = Arc::new(CloudflareClient::new(&config)?);

        // Verify connection
        client.verify_connection().await?;

        // Create services layer
        let services = Arc::new(CloudflareServices::new(
            Arc::clone(&client),
            pool,
        ));

        // Store in plugin state
        *self.config.write().await = Some(config);
        *self.client.write().await = Some(client);
        *self.services.write().await = Some(services);

        info!("Cloudflare client initialized from settings");
        Ok(())
    }

    /// Get the Cloudflare client
    pub async fn client(&self) -> Option<Arc<CloudflareClient>> {
        self.client.read().await.clone()
    }

    /// Get the services layer
    pub async fn services(&self) -> Option<Arc<CloudflareServices>> {
        self.services.read().await.clone()
    }

    /// Get the current configuration
    pub async fn config(&self) -> Option<CloudflareConfig> {
        self.config.read().await.clone()
    }

    /// Get the API router for this plugin
    /// This can be mounted at /api/plugins/rustcloudflare
    pub async fn api_router(&self) -> Option<Router> {
        let services = self.services.read().await.clone()?;
        Some(api::create_router(services))
    }

    /// Check if the plugin is configured
    pub async fn is_configured(&self) -> bool {
        self.config.read().await.is_some()
    }
}

impl Default for RustCloudflarePlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for RustCloudflarePlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }

    async fn activate(&self, _ctx: &AppContext) -> Result<()> {
        info!("Activating RustCloudflare plugin v{}", VERSION);

        *self.state.write().await = PluginState::Activating;

        // Note: Database migrations should be run via the migration system
        // API routes are exposed via api_router() method

        *self.state.write().await = PluginState::Active;
        info!("RustCloudflare plugin activated successfully");

        Ok(())
    }

    async fn deactivate(&self, _ctx: &AppContext) -> Result<()> {
        info!("Deactivating RustCloudflare plugin");

        *self.state.write().await = PluginState::Deactivating;

        // Cleanup resources
        *self.client.write().await = None;
        *self.services.write().await = None;
        *self.config.write().await = None;

        *self.state.write().await = PluginState::Inactive;
        info!("RustCloudflare plugin deactivated");

        Ok(())
    }

    async fn on_startup(&self, _ctx: &AppContext) -> Result<()> {
        info!("RustCloudflare plugin starting up");
        Ok(())
    }

    async fn on_shutdown(&self, _ctx: &AppContext) -> Result<()> {
        info!("RustCloudflare plugin shutting down");
        Ok(())
    }

    fn is_compatible(&self) -> bool {
        true
    }

    fn config_schema(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "type": "object",
            "properties": {
                "api_token": {
                    "type": "string",
                    "title": "API Token",
                    "description": "Cloudflare API token"
                },
                "account_id": {
                    "type": "string",
                    "title": "Account ID"
                },
                "zone_id": {
                    "type": "string",
                    "title": "Zone ID"
                }
            },
            "required": ["api_token", "account_id", "zone_id"]
        }))
    }

    fn state(&self) -> PluginState {
        // This is a sync method, so we use try_read
        self.state.try_read().map(|s| s.clone()).unwrap_or(PluginState::Inactive)
    }
}

#[async_trait]
impl LifecycleHook for RustCloudflarePlugin {
    async fn on_activate(&self, _context: &ActivationContext) -> std::result::Result<(), HookError> {
        info!("RustCloudflare lifecycle: on_activate");
        Ok(())
    }

    async fn on_deactivate(&self, _context: &DeactivationContext) -> std::result::Result<(), HookError> {
        info!("RustCloudflare lifecycle: on_deactivate");
        Ok(())
    }

    async fn on_upgrade(&self, context: &UpgradeContext) -> std::result::Result<(), HookError> {
        info!(
            "RustCloudflare lifecycle: upgrading from {} to {}",
            context.from_version, context.to_version
        );
        Ok(())
    }

    async fn on_uninstall(&self, context: &UninstallContext) -> std::result::Result<(), HookError> {
        info!("RustCloudflare lifecycle: on_uninstall");
        // Clean up plugin data if requested
        if context.delete_data {
            // Remove plugin tables and data
        }
        Ok(())
    }

    async fn on_init(&self, _context: &InitContext) -> std::result::Result<(), HookError> {
        info!("RustCloudflare lifecycle: on_init");
        Ok(())
    }

    async fn on_load(&self, _context: &LoadContext) -> std::result::Result<(), HookError> {
        info!("RustCloudflare lifecycle: on_load");
        Ok(())
    }

    async fn on_shutdown(&self, _context: &ShutdownContext) -> std::result::Result<(), HookError> {
        info!("RustCloudflare lifecycle: on_shutdown");
        Ok(())
    }
}

/// Plugin entry point - called by RustPress plugin loader
#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn create_plugin() -> Box<dyn Plugin> {
    Box::new(RustCloudflarePlugin::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = RustCloudflarePlugin::new();
        assert_eq!(plugin.info().id, PLUGIN_ID);
        assert_eq!(plugin.info().name, "RustCloudflare");
    }

    #[test]
    fn test_plugin_state() {
        let plugin = RustCloudflarePlugin::new();
        assert!(matches!(plugin.state(), PluginState::Inactive));
    }
}
