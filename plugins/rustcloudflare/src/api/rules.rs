//! Page Rules API handlers

use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use crate::error::CloudflareResult;
use crate::models::{CreatePageRule, UpdatePageRule};
use crate::services::CloudflareServices;

/// List all page rules
pub async fn list_page_rules(
    State(_services): State<Arc<CloudflareServices>>,
) -> CloudflareResult<Json<serde_json::Value>> {
    // Would call client.list_page_rules().await?;
    // Page rules are zone-level settings for URL pattern matching

    Ok(Json(serde_json::json!({
        "success": true,
        "data": [],
        "total": 0,
        "available_quota": 3  // Free plan allows 3 page rules
    })))
}

/// Get a specific page rule
pub async fn get_page_rule(
    State(_services): State<Arc<CloudflareServices>>,
    Path(id): Path<String>,
) -> CloudflareResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "id": id
        }
    })))
}

/// Create a new page rule
pub async fn create_page_rule(
    State(_services): State<Arc<CloudflareServices>>,
    Json(req): Json<CreatePageRule>,
) -> CloudflareResult<Json<serde_json::Value>> {
    // Validate targets
    if req.targets.is_empty() {
        return Ok(Json(serde_json::json!({
            "success": false,
            "error": "At least one target URL pattern is required"
        })));
    }

    // Validate actions
    if req.actions.is_empty() {
        return Ok(Json(serde_json::json!({
            "success": false,
            "error": "At least one action is required"
        })));
    }

    // Would call client.create_page_rule(req).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "id": uuid::Uuid::new_v4().to_string(),
            "targets": req.targets,
            "actions": req.actions,
            "status": req.status.unwrap_or_else(|| "active".to_string()),
            "priority": req.priority.unwrap_or(1)
        },
        "message": "Page rule created successfully"
    })))
}

/// Update an existing page rule
pub async fn update_page_rule(
    State(_services): State<Arc<CloudflareServices>>,
    Path(id): Path<String>,
    Json(req): Json<UpdatePageRule>,
) -> CloudflareResult<Json<serde_json::Value>> {
    // Would call client.update_page_rule(&id, req).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "id": id,
            "targets": req.targets,
            "actions": req.actions,
            "status": req.status,
            "priority": req.priority
        },
        "message": "Page rule updated successfully"
    })))
}

/// Delete a page rule
pub async fn delete_page_rule(
    State(_services): State<Arc<CloudflareServices>>,
    Path(id): Path<String>,
) -> CloudflareResult<Json<serde_json::Value>> {
    // Would call client.delete_page_rule(&id).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "id": id
        },
        "message": "Page rule deleted successfully"
    })))
}

/// Get available page rule actions
pub async fn get_available_actions() -> CloudflareResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({
        "success": true,
        "data": [
            {
                "id": "always_online",
                "name": "Always Online",
                "description": "If your server is unreachable, Cloudflare will serve cached pages"
            },
            {
                "id": "browser_cache_ttl",
                "name": "Browser Cache TTL",
                "description": "Override the browser cache TTL"
            },
            {
                "id": "browser_check",
                "name": "Browser Integrity Check",
                "description": "Check browser for basic integrity"
            },
            {
                "id": "cache_level",
                "name": "Cache Level",
                "description": "Set the cache level for this rule"
            },
            {
                "id": "edge_cache_ttl",
                "name": "Edge Cache TTL",
                "description": "Set the edge cache TTL"
            },
            {
                "id": "forwarding_url",
                "name": "Forwarding URL",
                "description": "Forward requests to another URL"
            },
            {
                "id": "always_use_https",
                "name": "Always Use HTTPS",
                "description": "Redirect HTTP to HTTPS"
            },
            {
                "id": "disable_apps",
                "name": "Disable Apps",
                "description": "Disable Cloudflare Apps"
            },
            {
                "id": "disable_performance",
                "name": "Disable Performance",
                "description": "Disable all performance features"
            },
            {
                "id": "disable_security",
                "name": "Disable Security",
                "description": "Disable all security features"
            },
            {
                "id": "security_level",
                "name": "Security Level",
                "description": "Set the security level"
            },
            {
                "id": "ssl",
                "name": "SSL",
                "description": "Set the SSL mode"
            }
        ]
    })))
}

/// Test a page rule pattern against URLs
#[derive(Debug, Deserialize)]
pub struct TestPatternRequest {
    pub pattern: String,
    pub urls: Vec<String>,
}

pub async fn test_pattern(
    Json(req): Json<TestPatternRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let pattern_regex = pattern_to_regex(&req.pattern);
    let regex = match regex::Regex::new(&pattern_regex) {
        Ok(r) => r,
        Err(e) => {
            return Ok(Json(serde_json::json!({
                "success": false,
                "error": format!("Invalid pattern: {}", e)
            })));
        }
    };

    let results: Vec<serde_json::Value> = req.urls.iter().map(|url| {
        serde_json::json!({
            "url": url,
            "matches": regex.is_match(url)
        })
    }).collect();

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "pattern": req.pattern,
            "results": results
        }
    })))
}

/// Convert Cloudflare URL pattern to regex
fn pattern_to_regex(pattern: &str) -> String {
    let mut regex = String::from("^");

    for ch in pattern.chars() {
        match ch {
            '*' => regex.push_str(".*"),
            '?' => regex.push('.'),
            '.' | '+' | '^' | '$' | '{' | '}' | '[' | ']' | '|' | '\\' | '(' | ')' => {
                regex.push('\\');
                regex.push(ch);
            }
            _ => regex.push(ch),
        }
    }

    regex.push('$');
    regex
}
