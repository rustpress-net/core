//! Settings API handlers

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::error::CloudflareResult;
use crate::services::CloudflareServices;
use crate::services::settings::ExtendedPluginSettings;

/// API response wrapper
#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }

    pub fn error(message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            message: Some(message.into()),
        }
    }
}

#[derive(Deserialize)]
pub struct UpdateSettingsRequest {
    pub settings: serde_json::Value,
}

#[derive(Deserialize)]
pub struct ToggleDevModeRequest {
    pub enabled: bool,
}

#[derive(Deserialize)]
pub struct UpdateAutoPurgeRequest {
    pub auto_purge_enabled: Option<bool>,
    pub auto_purge_on_post_update: Option<bool>,
    pub auto_purge_on_page_update: Option<bool>,
    pub auto_purge_on_media_upload: Option<bool>,
    pub auto_purge_on_theme_change: Option<bool>,
    pub auto_purge_on_menu_update: Option<bool>,
    pub auto_purge_entire_site: Option<bool>,
    pub auto_purge_homepage: Option<bool>,
    pub auto_purge_archives: Option<bool>,
    pub auto_purge_custom_urls: Option<String>,
    pub auto_purge_delay_ms: Option<u32>,
}

#[derive(Deserialize)]
pub struct UpdateCacheWarmingRequest {
    pub cache_warming_enabled: Option<bool>,
    pub cache_warming_schedule: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateNotificationsRequest {
    pub security_email_alerts: Option<bool>,
    pub security_slack_webhook: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateAdvancedRequest {
    pub development_mode_duration: Option<u32>,
    pub analytics_retention_days: Option<u32>,
    pub r2_default_bucket: Option<String>,
    pub workers_enabled: Option<bool>,
}

/// Get all plugin settings
pub async fn get_settings(
    State(services): State<Arc<CloudflareServices>>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let settings = services.settings.get_extended_settings().await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": settings
    })))
}

/// Update plugin settings
pub async fn update_settings(
    State(services): State<Arc<CloudflareServices>>,
    Json(req): Json<ExtendedPluginSettings>,
) -> CloudflareResult<Json<serde_json::Value>> {
    services.settings.update_extended_settings(&req).await?;

    let updated = services.settings.get_extended_settings().await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": updated,
        "message": "Settings updated successfully"
    })))
}

/// Get auto-purge settings
pub async fn get_auto_purge_settings(
    State(services): State<Arc<CloudflareServices>>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let config = services.settings.get_auto_purge_config().await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": config
    })))
}

/// Update auto-purge settings
pub async fn update_auto_purge_settings(
    State(services): State<Arc<CloudflareServices>>,
    Json(req): Json<UpdateAutoPurgeRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let mut settings = services.settings.get_extended_settings().await?;

    // Apply updates
    if let Some(v) = req.auto_purge_enabled { settings.auto_purge_enabled = v; }
    if let Some(v) = req.auto_purge_on_post_update { settings.auto_purge_on_post_update = v; }
    if let Some(v) = req.auto_purge_on_page_update { settings.auto_purge_on_page_update = v; }
    if let Some(v) = req.auto_purge_on_media_upload { settings.auto_purge_on_media_upload = v; }
    if let Some(v) = req.auto_purge_on_theme_change { settings.auto_purge_on_theme_change = v; }
    if let Some(v) = req.auto_purge_on_menu_update { settings.auto_purge_on_menu_update = v; }
    if let Some(v) = req.auto_purge_entire_site { settings.auto_purge_entire_site = v; }
    if let Some(v) = req.auto_purge_homepage { settings.auto_purge_homepage = v; }
    if let Some(v) = req.auto_purge_archives { settings.auto_purge_archives = v; }
    if req.auto_purge_custom_urls.is_some() { settings.auto_purge_custom_urls = req.auto_purge_custom_urls; }
    if let Some(v) = req.auto_purge_delay_ms { settings.auto_purge_delay_ms = v; }

    services.settings.update_extended_settings(&settings).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "auto_purge_enabled": settings.auto_purge_enabled,
            "auto_purge_on_post_update": settings.auto_purge_on_post_update,
            "auto_purge_on_page_update": settings.auto_purge_on_page_update,
            "auto_purge_on_media_upload": settings.auto_purge_on_media_upload,
            "auto_purge_on_theme_change": settings.auto_purge_on_theme_change,
            "auto_purge_on_menu_update": settings.auto_purge_on_menu_update,
            "auto_purge_entire_site": settings.auto_purge_entire_site,
            "auto_purge_homepage": settings.auto_purge_homepage,
            "auto_purge_archives": settings.auto_purge_archives,
            "auto_purge_custom_urls": settings.auto_purge_custom_urls,
            "auto_purge_delay_ms": settings.auto_purge_delay_ms,
        },
        "message": "Auto-purge settings updated"
    })))
}

/// Update cache warming settings
pub async fn update_cache_warming_settings(
    State(services): State<Arc<CloudflareServices>>,
    Json(req): Json<UpdateCacheWarmingRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let mut settings = services.settings.get_extended_settings().await?;

    if let Some(v) = req.cache_warming_enabled { settings.cache_warming_enabled = v; }
    if let Some(v) = req.cache_warming_schedule { settings.cache_warming_schedule = v; }

    services.settings.update_extended_settings(&settings).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "cache_warming_enabled": settings.cache_warming_enabled,
            "cache_warming_schedule": settings.cache_warming_schedule,
        },
        "message": "Cache warming settings updated"
    })))
}

/// Update notification settings
pub async fn update_notification_settings(
    State(services): State<Arc<CloudflareServices>>,
    Json(req): Json<UpdateNotificationsRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let mut settings = services.settings.get_extended_settings().await?;

    if let Some(v) = req.security_email_alerts { settings.security_email_alerts = v; }
    if req.security_slack_webhook.is_some() { settings.security_slack_webhook = req.security_slack_webhook; }

    services.settings.update_extended_settings(&settings).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "security_email_alerts": settings.security_email_alerts,
            "security_slack_webhook": settings.security_slack_webhook,
        },
        "message": "Notification settings updated"
    })))
}

/// Update advanced settings
pub async fn update_advanced_settings(
    State(services): State<Arc<CloudflareServices>>,
    Json(req): Json<UpdateAdvancedRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let mut settings = services.settings.get_extended_settings().await?;

    if let Some(v) = req.development_mode_duration { settings.development_mode_duration = v; }
    if let Some(v) = req.analytics_retention_days { settings.analytics_retention_days = v; }
    if req.r2_default_bucket.is_some() { settings.r2_default_bucket = req.r2_default_bucket; }
    if let Some(v) = req.workers_enabled { settings.workers_enabled = v; }

    services.settings.update_extended_settings(&settings).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "development_mode_duration": settings.development_mode_duration,
            "analytics_retention_days": settings.analytics_retention_days,
            "r2_default_bucket": settings.r2_default_bucket,
            "workers_enabled": settings.workers_enabled,
        },
        "message": "Advanced settings updated"
    })))
}

/// Get zone info
pub async fn get_zone_info(
    State(_services): State<Arc<CloudflareServices>>,
) -> CloudflareResult<Json<serde_json::Value>> {
    // Would fetch from Cloudflare API
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "id": "",
            "name": "",
            "status": "pending"
        }
    })))
}

/// Get zone settings from Cloudflare (placeholder)
pub async fn get_zone_settings(
    State(_services): State<Arc<CloudflareServices>>,
) -> CloudflareResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "success": true, "data": [] })))
}

/// Update zone settings on Cloudflare (placeholder)
pub async fn update_zone_settings(
    State(_services): State<Arc<CloudflareServices>>,
    Json(req): Json<serde_json::Value>,
) -> CloudflareResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "success": true, "data": req })))
}

/// Toggle development mode
pub async fn toggle_dev_mode(
    State(services): State<Arc<CloudflareServices>>,
    Json(req): Json<ToggleDevModeRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    services.settings.set_setting("development_mode", &serde_json::json!(req.enabled)).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "development_mode": req.enabled
        },
        "message": if req.enabled {
            "Development mode enabled"
        } else {
            "Development mode disabled"
        }
    })))
}
