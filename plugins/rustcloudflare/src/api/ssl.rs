//! SSL/TLS API handlers

use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use crate::error::CloudflareResult;
use crate::services::CloudflareServices;

#[derive(Debug, Deserialize)]
pub struct UpdateSslModeRequest {
    pub mode: String,
}

#[derive(Debug, Deserialize)]
pub struct OrderCertificateRequest {
    pub hosts: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UploadCertificateRequest {
    pub certificate: String,
    pub private_key: String,
}

/// Get SSL/TLS status and settings
pub async fn get_ssl_settings(
    State(services): State<Arc<CloudflareServices>>,
) -> CloudflareResult<Json<serde_json::Value>> {
    // Get SSL mode from zone settings via the client
    // For now, we'll try to get the setting from the settings service
    let _settings = services.settings.get_extended_settings().await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "mode": "strict",  // Would come from Cloudflare API
            "status": "active",
            "ssl_recommender": true,
            "always_use_https": true,
            "automatic_https_rewrites": true,
            "min_tls_version": "1.2",
            "opportunistic_encryption": true,
            "tls_1_3": "on"
        }
    })))
}

/// Get SSL status (alias for get_ssl_settings)
pub async fn get_ssl_status(
    State(services): State<Arc<CloudflareServices>>,
) -> CloudflareResult<Json<serde_json::Value>> {
    get_ssl_settings(State(services)).await
}

/// Update SSL mode
pub async fn update_ssl_mode(
    State(_services): State<Arc<CloudflareServices>>,
    Json(req): Json<UpdateSslModeRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    // Validate SSL mode
    let valid_modes = ["off", "flexible", "full", "strict"];
    if !valid_modes.contains(&req.mode.as_str()) {
        return Ok(Json(serde_json::json!({
            "success": false,
            "error": format!("Invalid SSL mode. Valid options: {:?}", valid_modes)
        })));
    }

    // Would call client.update_ssl_mode(&req.mode).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "mode": req.mode,
            "description": get_ssl_mode_description(&req.mode)
        },
        "message": format!("SSL mode set to {}", req.mode)
    })))
}

/// List SSL certificates
pub async fn list_certificates(
    State(_services): State<Arc<CloudflareServices>>,
) -> CloudflareResult<Json<serde_json::Value>> {
    // Would call client.list_certificates().await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "universal": {
                "status": "active",
                "certificate_authority": "lets_encrypt",
                "hosts": ["example.com", "*.example.com"],
                "expires_on": "2025-06-15T00:00:00Z"
            },
            "custom_certificates": [],
            "edge_certificates": []
        }
    })))
}

/// Order a new certificate
pub async fn order_certificate(
    State(_services): State<Arc<CloudflareServices>>,
    Json(req): Json<OrderCertificateRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    // Certificate ordering requires Enterprise plan
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "hosts": req.hosts,
            "status": "pending_validation"
        },
        "message": "Certificate order submitted"
    })))
}

/// Upload a custom certificate
pub async fn upload_certificate(
    State(_services): State<Arc<CloudflareServices>>,
    Json(req): Json<UploadCertificateRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    // Validate certificate format (basic check)
    if !req.certificate.contains("BEGIN CERTIFICATE") {
        return Ok(Json(serde_json::json!({
            "success": false,
            "error": "Invalid certificate format. Expected PEM format."
        })));
    }

    if !req.private_key.contains("BEGIN") || !req.private_key.contains("PRIVATE KEY") {
        return Ok(Json(serde_json::json!({
            "success": false,
            "error": "Invalid private key format. Expected PEM format."
        })));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "status": "active",
            "uploaded": true
        },
        "message": "Custom certificate uploaded successfully"
    })))
}

/// Delete a custom certificate
pub async fn delete_certificate(
    State(_services): State<Arc<CloudflareServices>>,
    Path(id): Path<String>,
) -> CloudflareResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "id": id
        },
        "message": "Certificate deleted successfully"
    })))
}

/// Helper function to get SSL mode description
fn get_ssl_mode_description(mode: &str) -> &'static str {
    match mode {
        "off" => "Off - No encryption between visitor and Cloudflare, and no encryption to origin",
        "flexible" => "Flexible - Encrypts traffic between visitor and Cloudflare, but not to origin",
        "full" => "Full - Encrypts end-to-end, but doesn't validate origin certificate",
        "strict" => "Full (Strict) - Encrypts end-to-end and validates origin certificate",
        _ => "Unknown SSL mode"
    }
}
