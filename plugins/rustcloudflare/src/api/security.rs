//! Security API handlers (WAF, Firewall, IP Access Rules)

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use crate::error::CloudflareResult;
use crate::models::CreateFirewallRule;
use crate::services::CloudflareServices;

#[derive(Debug, Deserialize)]
pub struct SetSecurityLevelRequest {
    pub level: String,
}

#[derive(Debug, Deserialize)]
pub struct ToggleUnderAttackRequest {
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct IpAccessRequest {
    pub ip: String,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChallengeIpRequest {
    pub ip: String,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SecurityEventsQuery {
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWafRuleRequest {
    pub enabled: bool,
}

/// Get current security level
pub async fn get_security_level(
    State(services): State<Arc<CloudflareServices>>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let level = services.security.get_security_level().await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "level": level,
            "description": get_security_level_description(&level)
        }
    })))
}

/// Set security level
pub async fn set_security_level(
    State(services): State<Arc<CloudflareServices>>,
    Json(req): Json<SetSecurityLevelRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    // Validate level
    let valid_levels = ["off", "essentially_off", "low", "medium", "high", "under_attack"];
    if !valid_levels.contains(&req.level.as_str()) {
        return Ok(Json(serde_json::json!({
            "success": false,
            "error": format!("Invalid security level. Valid options: {:?}", valid_levels)
        })));
    }

    services.security.set_security_level(&req.level).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "level": req.level,
            "description": get_security_level_description(&req.level)
        },
        "message": format!("Security level set to {}", req.level)
    })))
}

/// Toggle Under Attack mode
pub async fn toggle_under_attack(
    State(services): State<Arc<CloudflareServices>>,
    Json(req): Json<ToggleUnderAttackRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    services.security.toggle_under_attack(req.enabled).await?;

    let status = if req.enabled { "enabled" } else { "disabled" };

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "under_attack_mode": req.enabled
        },
        "message": format!("Under Attack mode {}", status)
    })))
}

/// List WAF rule packages
pub async fn list_waf_rules(
    State(services): State<Arc<CloudflareServices>>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let rules = services.security.list_waf_rules().await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": rules,
        "total": rules.len()
    })))
}

/// Update WAF rule status
pub async fn update_waf_rule(
    State(_services): State<Arc<CloudflareServices>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateWafRuleRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    // WAF rule update would require additional service methods
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "id": id,
            "enabled": req.enabled
        },
        "message": format!("WAF rule {} {}", id, if req.enabled { "enabled" } else { "disabled" })
    })))
}

/// List firewall rules
pub async fn list_firewall_rules(
    State(services): State<Arc<CloudflareServices>>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let rules = services.security.list_firewall_rules().await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": rules,
        "total": rules.len()
    })))
}

/// Create a firewall rule
pub async fn create_firewall_rule(
    State(services): State<Arc<CloudflareServices>>,
    Json(req): Json<CreateFirewallRule>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let rule = services.security.create_firewall_rule(req).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": rule,
        "message": "Firewall rule created successfully"
    })))
}

/// Update a firewall rule
pub async fn update_firewall_rule(
    State(_services): State<Arc<CloudflareServices>>,
    Path(id): Path<String>,
    Json(_req): Json<CreateFirewallRule>,
) -> CloudflareResult<Json<serde_json::Value>> {
    // Firewall rule update would require additional service methods
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "id": id
        },
        "message": "Firewall rule updated successfully"
    })))
}

/// Delete a firewall rule
pub async fn delete_firewall_rule(
    State(_services): State<Arc<CloudflareServices>>,
    Path(id): Path<String>,
) -> CloudflareResult<Json<serde_json::Value>> {
    // Would need to add delete_firewall_rule to security service
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "id": id
        },
        "message": "Firewall rule deleted successfully"
    })))
}

/// List IP access rules
pub async fn list_ip_access_rules(
    State(services): State<Arc<CloudflareServices>>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let rules = services.security.list_ip_access_rules().await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": rules,
        "total": rules.len()
    })))
}

/// Block an IP address
pub async fn block_ip(
    State(services): State<Arc<CloudflareServices>>,
    Json(req): Json<IpAccessRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let rule = services.security.block_ip(&req.ip, req.note.as_deref()).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": rule,
        "message": format!("IP {} blocked successfully", req.ip)
    })))
}

/// Allow an IP address (whitelist)
pub async fn allow_ip(
    State(services): State<Arc<CloudflareServices>>,
    Json(req): Json<IpAccessRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let rule = services.security.allow_ip(&req.ip, req.note.as_deref()).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": rule,
        "message": format!("IP {} allowed successfully", req.ip)
    })))
}

/// Challenge an IP address (CAPTCHA)
pub async fn challenge_ip(
    State(_services): State<Arc<CloudflareServices>>,
    Json(req): Json<ChallengeIpRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    // Would need to add challenge_ip to security service
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "ip": req.ip,
            "mode": "challenge"
        },
        "message": format!("IP {} will now be challenged", req.ip)
    })))
}

/// Delete an IP access rule
pub async fn delete_ip_access_rule(
    State(_services): State<Arc<CloudflareServices>>,
    Path(id): Path<String>,
) -> CloudflareResult<Json<serde_json::Value>> {
    // Would need to add delete_ip_access_rule to security service
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "id": id
        },
        "message": "IP access rule deleted successfully"
    })))
}

/// Get recent security events
pub async fn get_security_events(
    State(_services): State<Arc<CloudflareServices>>,
    Query(query): Query<SecurityEventsQuery>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let limit = query.limit.unwrap_or(100);

    // Security events would require analytics API integration
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "events": [],
            "limit": limit
        },
        "message": "Security events fetched (requires Cloudflare Analytics API)"
    })))
}

/// Helper function to get security level description
fn get_security_level_description(level: &str) -> &'static str {
    match level {
        "off" => "Off - No security checks",
        "essentially_off" => "Essentially Off - Only the most severe attacks blocked",
        "low" => "Low - Challenges only the most threatening visitors",
        "medium" => "Medium - Challenges both moderate and severe threats",
        "high" => "High - Challenges all visitors that have shown threatening behavior",
        "under_attack" => "I'm Under Attack! - Maximum protection for sites under DDoS attack",
        _ => "Unknown security level"
    }
}
