//! OAuth API handlers for Cloudflare SSO

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::error::CloudflareResult;
use crate::services::{CloudflareServices, TokenResources};
use tracing::{info, error};

/// OAuth callback query parameters
#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

/// Token verification request
#[derive(Debug, Deserialize)]
pub struct VerifyTokenRequest {
    pub api_token: String,
}

/// Token verification response
#[derive(Debug, Serialize)]
pub struct VerifyTokenResponse {
    pub success: bool,
    pub valid: bool,
    pub message: Option<String>,
    pub resources: Option<TokenResources>,
}

/// Save credentials request
#[derive(Debug, Deserialize)]
pub struct SaveCredentialsRequest {
    pub api_token: String,
    pub account_id: String,
    pub zone_id: String,
}

/// SSO Connect request - for one-step SSO flow
#[derive(Debug, Deserialize)]
pub struct SsoConnectRequest {
    pub access_token: String,
    pub account_id: Option<String>,
    pub zone_id: Option<String>,
}

/// SSO Connect response
#[derive(Debug, Serialize)]
pub struct SsoConnectResponse {
    pub success: bool,
    pub connected: bool,
    pub message: String,
    pub needs_selection: bool,
    pub resources: Option<TokenResources>,
    pub auto_selected: Option<AutoSelectedResources>,
}

/// Auto-selected resources when there's only one option
#[derive(Debug, Serialize)]
pub struct AutoSelectedResources {
    pub account_id: String,
    pub account_name: String,
    pub zone_id: String,
    pub zone_name: String,
}

/// API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

/// Get OAuth authorization URL
pub async fn get_auth_url(
    State(services): State<Arc<CloudflareServices>>,
) -> CloudflareResult<Json<ApiResponse<String>>> {
    // Generate a random state for CSRF protection
    let state = uuid::Uuid::new_v4().to_string();

    match services.oauth.get_auth_url(&state) {
        Ok(url) => Ok(Json(ApiResponse {
            success: true,
            data: Some(url),
            error: None,
        })),
        Err(e) => Ok(Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("OAuth not configured: {}", e)),
        })),
    }
}

/// Handle OAuth callback - one-step SSO flow
/// Automatically exchanges code, verifies token, gets resources, and saves credentials
pub async fn oauth_callback(
    State(services): State<Arc<CloudflareServices>>,
    Query(params): Query<OAuthCallbackQuery>,
) -> impl IntoResponse {
    // Check for errors
    if let Some(error) = params.error {
        let description = params.error_description.unwrap_or_default();
        error!("OAuth error: {} - {}", error, description);
        return Redirect::to(&format!(
            "/admin/cloudflare/settings?error={}",
            urlencoding::encode(&format!("{}: {}", error, description))
        ));
    }

    // Get authorization code
    let code = match params.code {
        Some(c) => c,
        None => {
            return Redirect::to("/admin/cloudflare/settings?error=No%20authorization%20code%20received");
        }
    };

    // Exchange code for tokens
    let tokens = match services.oauth.exchange_code(&code).await {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to exchange OAuth code: {}", e);
            return Redirect::to(&format!(
                "/admin/cloudflare/settings?error={}",
                urlencoding::encode(&e.to_string())
            ));
        }
    };

    info!("OAuth code exchanged successfully");

    // Get resources (accounts and zones) using the access token
    let resources = match services.oauth.get_token_resources(&tokens.access_token).await {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to get token resources: {}", e);
            return Redirect::to(&format!(
                "/admin/cloudflare/settings?error={}",
                urlencoding::encode(&format!("Failed to get account info: {}", e))
            ));
        }
    };

    // Check if auto-selection is possible (single account and single zone)
    let can_auto_select = resources.accounts.len() == 1 && resources.zones.len() == 1;

    if can_auto_select {
        // Auto-select and save credentials in one step
        let account = &resources.accounts[0];
        let zone = &resources.zones[0];

        let credentials = crate::services::CloudflareCredentials {
            api_token: tokens.access_token.clone(),
            account_id: account.id.clone(),
            zone_id: zone.id.clone(),
        };

        match services.settings.save_credentials(&credentials).await {
            Ok(_) => {
                info!("SSO credentials saved automatically for zone: {}", zone.name);
                Redirect::to(&format!(
                    "/admin/cloudflare/settings?sso_success=true&zone={}",
                    urlencoding::encode(&zone.name)
                ))
            }
            Err(e) => {
                error!("Failed to save SSO credentials: {}", e);
                Redirect::to(&format!(
                    "/admin/cloudflare/settings?error={}",
                    urlencoding::encode(&format!("Failed to save credentials: {}", e))
                ))
            }
        }
    } else {
        // Multiple accounts/zones - pass token to frontend for selection
        // Encode account and zone info for the frontend
        let accounts_json = serde_json::to_string(&resources.accounts).unwrap_or_default();
        let zones_json = serde_json::to_string(&resources.zones).unwrap_or_default();

        Redirect::to(&format!(
            "/admin/cloudflare/settings?sso_token={}&sso_accounts={}&sso_zones={}",
            urlencoding::encode(&tokens.access_token),
            urlencoding::encode(&accounts_json),
            urlencoding::encode(&zones_json)
        ))
    }
}

/// Complete SSO connection with selected account and zone
/// Called from frontend when user selects from multiple accounts/zones
pub async fn sso_complete(
    State(services): State<Arc<CloudflareServices>>,
    Json(req): Json<SsoConnectRequest>,
) -> CloudflareResult<Json<SsoConnectResponse>> {
    // Verify the token is still valid
    let valid = services.oauth.verify_token(&req.access_token).await?;

    if !valid {
        return Ok(Json(SsoConnectResponse {
            success: false,
            connected: false,
            message: "Token is invalid or expired. Please try SSO login again.".to_string(),
            needs_selection: false,
            resources: None,
            auto_selected: None,
        }));
    }

    // Get resources to validate selection
    let resources = services.oauth.get_token_resources(&req.access_token).await?;

    // Determine account and zone IDs
    let account_id = req.account_id.or_else(|| {
        if resources.accounts.len() == 1 {
            Some(resources.accounts[0].id.clone())
        } else {
            None
        }
    });

    let zone_id = req.zone_id.or_else(|| {
        if resources.zones.len() == 1 {
            Some(resources.zones[0].id.clone())
        } else {
            None
        }
    });

    // Check if we have all required info
    let (account_id, zone_id) = match (account_id, zone_id) {
        (Some(a), Some(z)) => (a, z),
        _ => {
            return Ok(Json(SsoConnectResponse {
                success: true,
                connected: false,
                message: "Please select an account and zone".to_string(),
                needs_selection: true,
                resources: Some(resources),
                auto_selected: None,
            }));
        }
    };

    // Validate the selected account and zone exist
    let account = resources.accounts.iter().find(|a| a.id == account_id);
    let zone = resources.zones.iter().find(|z| z.id == zone_id);

    if account.is_none() || zone.is_none() {
        return Ok(Json(SsoConnectResponse {
            success: false,
            connected: false,
            message: "Invalid account or zone selection".to_string(),
            needs_selection: true,
            resources: Some(resources),
            auto_selected: None,
        }));
    }

    let account = account.unwrap();
    let zone = zone.unwrap();

    // Save credentials
    let credentials = crate::services::CloudflareCredentials {
        api_token: req.access_token,
        account_id: account_id.clone(),
        zone_id: zone_id.clone(),
    };

    services.settings.save_credentials(&credentials).await?;

    info!("SSO connection completed for zone: {}", zone.name);

    Ok(Json(SsoConnectResponse {
        success: true,
        connected: true,
        message: format!("Successfully connected to {}", zone.name),
        needs_selection: false,
        resources: None,
        auto_selected: Some(AutoSelectedResources {
            account_id,
            account_name: account.name.clone(),
            zone_id,
            zone_name: zone.name.clone(),
        }),
    }))
}

/// Verify an API token and get available resources
pub async fn verify_token(
    State(services): State<Arc<CloudflareServices>>,
    Json(req): Json<VerifyTokenRequest>,
) -> CloudflareResult<Json<VerifyTokenResponse>> {
    // Verify the token
    let valid = services.oauth.verify_token(&req.api_token).await?;

    if !valid {
        return Ok(Json(VerifyTokenResponse {
            success: true,
            valid: false,
            message: Some("Invalid API token".to_string()),
            resources: None,
        }));
    }

    // Get available resources (accounts and zones)
    match services.oauth.get_token_resources(&req.api_token).await {
        Ok(resources) => Ok(Json(VerifyTokenResponse {
            success: true,
            valid: true,
            message: Some("Token verified successfully".to_string()),
            resources: Some(resources),
        })),
        Err(e) => Ok(Json(VerifyTokenResponse {
            success: true,
            valid: true,
            message: Some(format!("Token valid but couldn't fetch resources: {}", e)),
            resources: None,
        })),
    }
}

/// Save Cloudflare credentials
pub async fn save_credentials(
    State(services): State<Arc<CloudflareServices>>,
    Json(req): Json<SaveCredentialsRequest>,
) -> CloudflareResult<Json<ApiResponse<()>>> {
    // Verify the token first
    let valid = services.oauth.verify_token(&req.api_token).await?;

    if !valid {
        return Ok(Json(ApiResponse {
            success: false,
            data: None,
            error: Some("Invalid API token".to_string()),
        }));
    }

    // Save credentials
    let credentials = crate::services::CloudflareCredentials {
        api_token: req.api_token,
        account_id: req.account_id,
        zone_id: req.zone_id,
    };

    services.settings.save_credentials(&credentials).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(()),
        error: None,
    }))
}

/// Get connection status
pub async fn get_connection_status(
    State(services): State<Arc<CloudflareServices>>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let has_creds = services.settings.has_credentials().await?;

    if !has_creds {
        return Ok(Json(serde_json::json!({
            "success": true,
            "connected": false,
            "message": "No credentials configured"
        })));
    }

    // Get credentials and verify
    let creds = services.settings.get_credentials().await?;

    if let Some(creds) = creds {
        let valid = services.oauth.verify_token(&creds.api_token).await.unwrap_or(false);

        if valid {
            // Get zone info
            match services.oauth.list_zones(&creds.api_token, Some(&creds.account_id)).await {
                Ok(zones) => {
                    let zone = zones.iter().find(|z| z.id == creds.zone_id);
                    Ok(Json(serde_json::json!({
                        "success": true,
                        "connected": true,
                        "account_id": creds.account_id,
                        "zone_id": creds.zone_id,
                        "zone_name": zone.map(|z| z.name.clone()),
                        "zone_status": zone.map(|z| z.status.clone()),
                    })))
                }
                Err(_) => Ok(Json(serde_json::json!({
                    "success": true,
                    "connected": true,
                    "account_id": creds.account_id,
                    "zone_id": creds.zone_id,
                }))),
            }
        } else {
            Ok(Json(serde_json::json!({
                "success": true,
                "connected": false,
                "message": "API token is invalid or expired"
            })))
        }
    } else {
        Ok(Json(serde_json::json!({
            "success": true,
            "connected": false,
            "message": "No credentials found"
        })))
    }
}

/// Disconnect (delete credentials)
pub async fn disconnect(
    State(services): State<Arc<CloudflareServices>>,
) -> CloudflareResult<Json<ApiResponse<()>>> {
    services.settings.delete_credentials().await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(()),
        error: None,
    }))
}

/// List available accounts for the current token
pub async fn list_accounts(
    State(services): State<Arc<CloudflareServices>>,
    Json(req): Json<VerifyTokenRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let accounts = services.oauth.list_accounts(&req.api_token).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": accounts
    })))
}

/// List available zones for an account
#[derive(Debug, Deserialize)]
pub struct ListZonesRequest {
    pub api_token: String,
    pub account_id: Option<String>,
}

pub async fn list_zones(
    State(services): State<Arc<CloudflareServices>>,
    Json(req): Json<ListZonesRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let zones = services.oauth.list_zones(&req.api_token, req.account_id.as_deref()).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": zones
    })))
}
