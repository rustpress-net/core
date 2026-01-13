//! Settings service for Cloudflare credential management

use crate::error::{CloudflareError, CloudflareResult};
use crate::hooks::AutoPurgeConfig;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use tracing::{debug, info};

/// Cloudflare credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareCredentials {
    pub api_token: String,
    pub account_id: String,
    pub zone_id: String,
}

/// Plugin settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginSettings {
    pub cdn_enabled: bool,
    pub cache_level: String,
    pub security_level: String,
    pub ssl_mode: String,
    pub auto_purge_on_update: bool,
    pub development_mode: bool,
}

/// Extended plugin settings with all configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedPluginSettings {
    // Credentials (masked for security)
    pub api_token_set: bool,
    pub account_id: Option<String>,
    pub zone_id: Option<String>,

    // Auto-purge settings
    pub auto_purge_enabled: bool,
    pub auto_purge_on_post_update: bool,
    pub auto_purge_on_page_update: bool,
    pub auto_purge_on_media_upload: bool,
    pub auto_purge_on_theme_change: bool,
    pub auto_purge_on_menu_update: bool,
    pub auto_purge_entire_site: bool,
    pub auto_purge_homepage: bool,
    pub auto_purge_archives: bool,
    pub auto_purge_custom_urls: Option<String>,
    pub auto_purge_delay_ms: u32,

    // Cache warming
    pub cache_warming_enabled: bool,
    pub cache_warming_schedule: String,

    // Notifications
    pub security_email_alerts: bool,
    pub security_slack_webhook: Option<String>,

    // Advanced settings
    pub development_mode_duration: u32,
    pub analytics_retention_days: u32,
    pub r2_default_bucket: Option<String>,
    pub workers_enabled: bool,
}

impl Default for ExtendedPluginSettings {
    fn default() -> Self {
        Self {
            api_token_set: false,
            account_id: None,
            zone_id: None,
            auto_purge_enabled: true,
            auto_purge_on_post_update: true,
            auto_purge_on_page_update: true,
            auto_purge_on_media_upload: true,
            auto_purge_on_theme_change: true,
            auto_purge_on_menu_update: true,
            auto_purge_entire_site: false,
            auto_purge_homepage: true,
            auto_purge_archives: true,
            auto_purge_custom_urls: None,
            auto_purge_delay_ms: 500,
            cache_warming_enabled: false,
            cache_warming_schedule: "immediate".to_string(),
            security_email_alerts: false,
            security_slack_webhook: None,
            development_mode_duration: 180,
            analytics_retention_days: 30,
            r2_default_bucket: None,
            workers_enabled: true,
        }
    }
}

/// Settings service for managing plugin configuration
#[derive(Clone)]
pub struct SettingsService {
    pool: Pool<Postgres>,
}

impl SettingsService {
    /// Create a new settings service
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    /// Get Cloudflare credentials from database
    pub async fn get_credentials(&self) -> CloudflareResult<Option<CloudflareCredentials>> {
        let api_token = self.get_setting("api_token").await?;
        let account_id = self.get_setting("account_id").await?;
        let zone_id = self.get_setting("zone_id").await?;

        match (api_token, account_id, zone_id) {
            (Some(token), Some(account), Some(zone)) => {
                // Decrypt token if needed
                let token_str = token.as_str().unwrap_or("").trim_matches('"').to_string();
                let account_str = account.as_str().unwrap_or("").trim_matches('"').to_string();
                let zone_str = zone.as_str().unwrap_or("").trim_matches('"').to_string();

                if token_str.is_empty() || account_str.is_empty() || zone_str.is_empty() {
                    return Ok(None);
                }

                Ok(Some(CloudflareCredentials {
                    api_token: token_str,
                    account_id: account_str,
                    zone_id: zone_str,
                }))
            }
            _ => Ok(None),
        }
    }

    /// Save Cloudflare credentials to database
    pub async fn save_credentials(&self, credentials: &CloudflareCredentials) -> CloudflareResult<()> {
        self.set_setting("api_token", &serde_json::json!(credentials.api_token)).await?;
        self.set_setting("account_id", &serde_json::json!(credentials.account_id)).await?;
        self.set_setting("zone_id", &serde_json::json!(credentials.zone_id)).await?;
        info!("Cloudflare credentials saved");
        Ok(())
    }

    /// Delete Cloudflare credentials from database
    pub async fn delete_credentials(&self) -> CloudflareResult<()> {
        self.delete_setting("api_token").await?;
        self.delete_setting("account_id").await?;
        self.delete_setting("zone_id").await?;
        info!("Cloudflare credentials deleted");
        Ok(())
    }

    /// Get plugin settings
    pub async fn get_plugin_settings(&self) -> CloudflareResult<PluginSettings> {
        let cdn_enabled = self.get_setting("cdn_enabled").await?
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let cache_level = self.get_setting("cache_level").await?
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "aggressive".to_string());
        let security_level = self.get_setting("security_level").await?
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "medium".to_string());
        let ssl_mode = self.get_setting("ssl_mode").await?
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "strict".to_string());
        let auto_purge_on_update = self.get_setting("auto_purge_on_update").await?
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let development_mode = self.get_setting("development_mode").await?
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(PluginSettings {
            cdn_enabled,
            cache_level,
            security_level,
            ssl_mode,
            auto_purge_on_update,
            development_mode,
        })
    }

    /// Update plugin settings
    pub async fn update_plugin_settings(&self, settings: &PluginSettings) -> CloudflareResult<()> {
        self.set_setting("cdn_enabled", &serde_json::json!(settings.cdn_enabled)).await?;
        self.set_setting("cache_level", &serde_json::json!(settings.cache_level)).await?;
        self.set_setting("security_level", &serde_json::json!(settings.security_level)).await?;
        self.set_setting("ssl_mode", &serde_json::json!(settings.ssl_mode)).await?;
        self.set_setting("auto_purge_on_update", &serde_json::json!(settings.auto_purge_on_update)).await?;
        self.set_setting("development_mode", &serde_json::json!(settings.development_mode)).await?;
        Ok(())
    }

    /// Get a single setting value
    pub async fn get_setting(&self, key: &str) -> CloudflareResult<Option<serde_json::Value>> {
        let result: Option<(Option<serde_json::Value>,)> = sqlx::query_as(
            r#"SELECT value FROM cloudflare_settings WHERE key = $1"#,
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CloudflareError::DatabaseError(e.to_string()))?;

        Ok(result.and_then(|(v,)| v))
    }

    /// Set a single setting value
    pub async fn set_setting(&self, key: &str, value: &serde_json::Value) -> CloudflareResult<()> {
        sqlx::query(
            r#"
            INSERT INTO cloudflare_settings (key, value, updated_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (key) DO UPDATE SET value = $2, updated_at = NOW()
            "#,
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await
        .map_err(|e| CloudflareError::DatabaseError(e.to_string()))?;

        debug!("Setting '{}' updated", key);
        Ok(())
    }

    /// Delete a setting
    pub async fn delete_setting(&self, key: &str) -> CloudflareResult<()> {
        sqlx::query(r#"DELETE FROM cloudflare_settings WHERE key = $1"#)
            .bind(key)
            .execute(&self.pool)
            .await
            .map_err(|e| CloudflareError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Check if credentials are configured
    pub async fn has_credentials(&self) -> CloudflareResult<bool> {
        let creds = self.get_credentials().await?;
        Ok(creds.is_some())
    }

    /// Get extended plugin settings with all configuration options
    pub async fn get_extended_settings(&self) -> CloudflareResult<ExtendedPluginSettings> {
        let creds = self.get_credentials().await?;

        let mut settings = ExtendedPluginSettings::default();

        // Credentials info (masked)
        if let Some(creds) = creds {
            settings.api_token_set = !creds.api_token.is_empty();
            settings.account_id = Some(creds.account_id);
            settings.zone_id = Some(creds.zone_id);
        }

        // Auto-purge settings
        settings.auto_purge_enabled = self.get_setting("auto_purge_enabled").await?
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        settings.auto_purge_on_post_update = self.get_setting("auto_purge_on_post_update").await?
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        settings.auto_purge_on_page_update = self.get_setting("auto_purge_on_page_update").await?
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        settings.auto_purge_on_media_upload = self.get_setting("auto_purge_on_media_upload").await?
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        settings.auto_purge_on_theme_change = self.get_setting("auto_purge_on_theme_change").await?
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        settings.auto_purge_on_menu_update = self.get_setting("auto_purge_on_menu_update").await?
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        settings.auto_purge_entire_site = self.get_setting("auto_purge_entire_site").await?
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        settings.auto_purge_homepage = self.get_setting("auto_purge_homepage").await?
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        settings.auto_purge_archives = self.get_setting("auto_purge_archives").await?
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        settings.auto_purge_custom_urls = self.get_setting("auto_purge_custom_urls").await?
            .and_then(|v| v.as_str().map(|s| s.to_string()));
        settings.auto_purge_delay_ms = self.get_setting("auto_purge_delay_ms").await?
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(500);

        // Cache warming
        settings.cache_warming_enabled = self.get_setting("cache_warming_enabled").await?
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        settings.cache_warming_schedule = self.get_setting("cache_warming_schedule").await?
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "immediate".to_string());

        // Notifications
        settings.security_email_alerts = self.get_setting("security_email_alerts").await?
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        settings.security_slack_webhook = self.get_setting("security_slack_webhook").await?
            .and_then(|v| v.as_str().map(|s| s.to_string()));

        // Advanced settings
        settings.development_mode_duration = self.get_setting("development_mode_duration").await?
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(180);
        settings.analytics_retention_days = self.get_setting("analytics_retention_days").await?
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(30);
        settings.r2_default_bucket = self.get_setting("r2_default_bucket").await?
            .and_then(|v| v.as_str().map(|s| s.to_string()));
        settings.workers_enabled = self.get_setting("workers_enabled").await?
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        Ok(settings)
    }

    /// Update extended plugin settings
    pub async fn update_extended_settings(&self, settings: &ExtendedPluginSettings) -> CloudflareResult<()> {
        // Auto-purge settings
        self.set_setting("auto_purge_enabled", &serde_json::json!(settings.auto_purge_enabled)).await?;
        self.set_setting("auto_purge_on_post_update", &serde_json::json!(settings.auto_purge_on_post_update)).await?;
        self.set_setting("auto_purge_on_page_update", &serde_json::json!(settings.auto_purge_on_page_update)).await?;
        self.set_setting("auto_purge_on_media_upload", &serde_json::json!(settings.auto_purge_on_media_upload)).await?;
        self.set_setting("auto_purge_on_theme_change", &serde_json::json!(settings.auto_purge_on_theme_change)).await?;
        self.set_setting("auto_purge_on_menu_update", &serde_json::json!(settings.auto_purge_on_menu_update)).await?;
        self.set_setting("auto_purge_entire_site", &serde_json::json!(settings.auto_purge_entire_site)).await?;
        self.set_setting("auto_purge_homepage", &serde_json::json!(settings.auto_purge_homepage)).await?;
        self.set_setting("auto_purge_archives", &serde_json::json!(settings.auto_purge_archives)).await?;
        if let Some(urls) = &settings.auto_purge_custom_urls {
            self.set_setting("auto_purge_custom_urls", &serde_json::json!(urls)).await?;
        }
        self.set_setting("auto_purge_delay_ms", &serde_json::json!(settings.auto_purge_delay_ms)).await?;

        // Cache warming
        self.set_setting("cache_warming_enabled", &serde_json::json!(settings.cache_warming_enabled)).await?;
        self.set_setting("cache_warming_schedule", &serde_json::json!(settings.cache_warming_schedule)).await?;

        // Notifications
        self.set_setting("security_email_alerts", &serde_json::json!(settings.security_email_alerts)).await?;
        if let Some(webhook) = &settings.security_slack_webhook {
            self.set_setting("security_slack_webhook", &serde_json::json!(webhook)).await?;
        }

        // Advanced settings
        self.set_setting("development_mode_duration", &serde_json::json!(settings.development_mode_duration)).await?;
        self.set_setting("analytics_retention_days", &serde_json::json!(settings.analytics_retention_days)).await?;
        if let Some(bucket) = &settings.r2_default_bucket {
            self.set_setting("r2_default_bucket", &serde_json::json!(bucket)).await?;
        }
        self.set_setting("workers_enabled", &serde_json::json!(settings.workers_enabled)).await?;

        info!("Extended plugin settings updated");
        Ok(())
    }

    /// Get auto-purge configuration compatible with hooks module
    pub async fn get_auto_purge_config(&self) -> CloudflareResult<AutoPurgeConfig> {
        let settings = self.get_extended_settings().await?;

        Ok(AutoPurgeConfig {
            enabled: settings.auto_purge_enabled,
            on_post_update: settings.auto_purge_on_post_update,
            on_page_update: settings.auto_purge_on_page_update,
            on_media_change: settings.auto_purge_on_media_upload,
            on_theme_change: settings.auto_purge_on_theme_change,
            on_menu_update: settings.auto_purge_on_menu_update,
            on_widget_update: true,
            on_settings_change: false,
            purge_entire_site: settings.auto_purge_entire_site,
            always_purge_homepage: settings.auto_purge_homepage,
            purge_archives: settings.auto_purge_archives,
            custom_purge_urls: settings.auto_purge_custom_urls,
            purge_delay_ms: settings.auto_purge_delay_ms,
        })
    }
}
