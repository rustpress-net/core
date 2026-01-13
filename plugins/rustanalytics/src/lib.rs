//! RustAnalytics - Enterprise Google Analytics Integration for RustPress
//!
//! This plugin provides comprehensive Google Analytics integration with:
//! - Real-time analytics dashboard
//! - Audience insights and demographics
//! - Acquisition and traffic analysis
//! - Behavior and content analytics
//! - Conversion and goal tracking
//! - E-commerce analytics
//! - Custom reports and scheduled reporting
//!
//! # Features
//!
//! - Full Google Analytics 4 (GA4) Data API integration
//! - Real-time visitor tracking and monitoring
//! - Enterprise dashboard with comprehensive metrics
//! - Custom report builder
//! - Scheduled email reports
//! - Data caching for performance
//! - Privacy-compliant tracking (GDPR, CCPA)

pub mod models;
pub mod services;
pub mod handlers;
pub mod admin;

use std::sync::Arc;

use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::models::{AnalyticsSettings, ConnectionStatus};
use crate::services::client::GoogleAnalyticsClient;

/// Plugin version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Plugin ID
pub const PLUGIN_ID: &str = "rustanalytics";

/// Plugin name
pub const PLUGIN_NAME: &str = "RustAnalytics - Google Analytics Integration";

/// Plugin state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginState {
    Inactive,
    Activating,
    Active,
    Deactivating,
    Error,
}

/// Plugin information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub author_uri: Option<String>,
    pub plugin_uri: Option<String>,
    pub license: String,
    pub dependencies: Vec<String>,
    pub tags: Vec<String>,
    pub min_rustpress_version: String,
}

/// Plugin trait for RustPress integration
#[async_trait]
pub trait Plugin: Send + Sync {
    fn info(&self) -> &PluginInfo;
    fn state(&self) -> PluginState;
    fn config_schema(&self) -> Option<serde_json::Value>;
}

/// RustAnalytics Plugin
///
/// Enterprise-grade Google Analytics integration for RustPress
pub struct RustAnalyticsPlugin {
    /// Plugin information
    info: PluginInfo,
    /// Current plugin state
    state: RwLock<PluginState>,
    /// Plugin settings
    settings: RwLock<AnalyticsSettings>,
    /// Google Analytics API client
    ga_client: RwLock<Option<Arc<GoogleAnalyticsClient>>>,
    /// Connection status
    connection_status: RwLock<ConnectionStatus>,
}

impl RustAnalyticsPlugin {
    /// Create a new RustAnalytics plugin instance
    pub fn new() -> Self {
        let info = PluginInfo {
            id: PLUGIN_ID.to_string(),
            name: PLUGIN_NAME.to_string(),
            version: VERSION.to_string(),
            description: "Enterprise-grade Google Analytics integration with comprehensive dashboards, real-time monitoring, and advanced reporting".to_string(),
            author: "RustPress Team".to_string(),
            author_uri: Some("https://rustpress.io".to_string()),
            plugin_uri: Some("https://rustpress.io/plugins/rustanalytics".to_string()),
            license: "MIT".to_string(),
            dependencies: vec![],
            tags: vec![
                "analytics".to_string(),
                "google-analytics".to_string(),
                "ga4".to_string(),
                "enterprise".to_string(),
                "dashboard".to_string(),
                "reporting".to_string(),
                "real-time".to_string(),
                "ecommerce".to_string(),
            ],
            min_rustpress_version: "1.0.0".to_string(),
        };

        Self {
            info,
            state: RwLock::new(PluginState::Inactive),
            settings: RwLock::new(AnalyticsSettings::default()),
            ga_client: RwLock::new(None),
            connection_status: RwLock::new(ConnectionStatus {
                connected: false,
                property_name: None,
                property_id: None,
                account_name: None,
                last_sync: None,
                error: None,
            }),
        }
    }

    /// Get current settings
    pub fn settings(&self) -> AnalyticsSettings {
        self.settings.read().clone()
    }

    /// Update settings
    pub fn update_settings(&self, settings: AnalyticsSettings) {
        *self.settings.write() = settings;
    }

    /// Get connection status
    pub fn connection_status(&self) -> ConnectionStatus {
        self.connection_status.read().clone()
    }

    /// Get the Google Analytics client
    pub fn ga_client(&self) -> Option<Arc<GoogleAnalyticsClient>> {
        self.ga_client.read().clone()
    }

    /// Initialize the Google Analytics client
    pub async fn initialize_client(&self) -> Result<(), String> {
        let settings = self.settings();

        if settings.ga_property_id.is_empty() {
            return Err("GA Property ID is not configured".to_string());
        }

        match GoogleAnalyticsClient::new(
            settings.ga_property_id.clone(),
            settings.service_account_json.clone(),
        ).await {
            Ok(client) => {
                *self.ga_client.write() = Some(Arc::new(client));
                *self.connection_status.write() = ConnectionStatus {
                    connected: true,
                    property_id: Some(settings.ga_property_id.clone()),
                    property_name: None,
                    account_name: None,
                    last_sync: Some(chrono::Utc::now()),
                    error: None,
                };
                info!("RustAnalytics: Successfully connected to Google Analytics");
                Ok(())
            }
            Err(e) => {
                warn!("RustAnalytics: Failed to initialize GA client: {}", e);
                *self.connection_status.write() = ConnectionStatus {
                    connected: false,
                    property_id: Some(settings.ga_property_id.clone()),
                    property_name: None,
                    account_name: None,
                    last_sync: None,
                    error: Some(e.to_string()),
                };
                Err(e.to_string())
            }
        }
    }

    /// Generate the Google Analytics tracking script
    pub fn generate_tracking_script(&self) -> Option<String> {
        let settings = self.settings();

        if !settings.enable_tracking || settings.ga_measurement_id.is_empty() {
            return None;
        }

        let config_options = if settings.anonymize_ip {
            "{ 'anonymize_ip': true }"
        } else {
            "{}"
        };

        let enhanced_link = if settings.enhanced_link_attribution {
            "gtag('require', 'linkid');"
        } else {
            ""
        };

        Some(format!(
            r#"<!-- Google Analytics (RustAnalytics) -->
<script async src="https://www.googletagmanager.com/gtag/js?id={}"></script>
<script>
  window.dataLayer = window.dataLayer || [];
  function gtag(){{dataLayer.push(arguments);}}
  gtag('js', new Date());
  {}
  gtag('config', '{}', {});
</script>
<!-- End Google Analytics -->"#,
            settings.ga_measurement_id,
            enhanced_link,
            settings.ga_measurement_id,
            config_options
        ))
    }
}

impl Default for RustAnalyticsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for RustAnalyticsPlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }

    fn state(&self) -> PluginState {
        *self.state.read()
    }

    fn config_schema(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "title": "RustAnalytics Settings",
            "properties": {
                "ga_property_id": {
                    "type": "string",
                    "title": "GA4 Property ID",
                    "description": "Your Google Analytics 4 property ID (e.g., 123456789)"
                },
                "ga_measurement_id": {
                    "type": "string",
                    "title": "Measurement ID",
                    "description": "Your GA4 measurement ID (e.g., G-XXXXXXXXXX)"
                },
                "enable_tracking": {
                    "type": "boolean",
                    "title": "Enable Tracking",
                    "description": "Enable Google Analytics tracking on your site",
                    "default": true
                },
                "anonymize_ip": {
                    "type": "boolean",
                    "title": "Anonymize IP",
                    "description": "Anonymize visitor IP addresses for privacy",
                    "default": true
                },
                "track_logged_in_users": {
                    "type": "boolean",
                    "title": "Track Logged-in Users",
                    "description": "Track visits from logged-in users",
                    "default": true
                },
                "track_admin_users": {
                    "type": "boolean",
                    "title": "Track Admin Users",
                    "description": "Track visits from administrators",
                    "default": false
                },
                "default_date_range": {
                    "type": "string",
                    "title": "Default Date Range",
                    "enum": ["today", "yesterday", "last_7_days", "last_30_days", "last_90_days", "this_month", "last_month"],
                    "default": "last_30_days"
                },
                "cache_duration_minutes": {
                    "type": "integer",
                    "title": "Cache Duration (minutes)",
                    "minimum": 1,
                    "maximum": 60,
                    "default": 15
                },
                "gdpr_compliant": {
                    "type": "boolean",
                    "title": "GDPR Compliant Mode",
                    "description": "Enable GDPR-compliant tracking",
                    "default": true
                },
                "ccpa_compliant": {
                    "type": "boolean",
                    "title": "CCPA Compliant Mode",
                    "description": "Enable CCPA-compliant tracking",
                    "default": true
                }
            },
            "required": ["ga_property_id"]
        }))
    }
}

/// Create the plugin instance
///
/// This is the entry point for the plugin system
pub fn create_plugin() -> Arc<dyn Plugin> {
    Arc::new(RustAnalyticsPlugin::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = RustAnalyticsPlugin::new();
        assert_eq!(plugin.info().id, PLUGIN_ID);
        assert_eq!(plugin.info().name, PLUGIN_NAME);
        assert_eq!(plugin.state(), PluginState::Inactive);
    }

    #[test]
    fn test_default_settings() {
        let plugin = RustAnalyticsPlugin::new();
        let settings = plugin.settings();
        assert!(settings.enable_tracking);
        assert!(settings.anonymize_ip);
        assert!(!settings.track_admin_users);
        assert_eq!(settings.cache_duration_minutes, 15);
    }

    #[test]
    fn test_tracking_script_generation() {
        let plugin = RustAnalyticsPlugin::new();

        // No measurement ID, should return None
        assert!(plugin.generate_tracking_script().is_none());

        // With measurement ID
        let mut settings = plugin.settings();
        settings.ga_measurement_id = "G-TEST123".to_string();
        plugin.update_settings(settings);

        let script = plugin.generate_tracking_script();
        assert!(script.is_some());
        assert!(script.unwrap().contains("G-TEST123"));
    }
}
