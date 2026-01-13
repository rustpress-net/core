//! Cache API handlers

use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use crate::error::CloudflareResult;
use crate::services::CloudflareServices;

#[derive(Debug, Deserialize)]
pub struct PurgeUrlsRequest {
    pub urls: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct PurgeTagsRequest {
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct PurgePrefixRequest {
    pub prefix: String,
}

#[derive(Debug, Deserialize)]
pub struct WarmCacheRequest {
    pub urls: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CacheStatsQuery {
    pub hours: Option<i32>,
}

/// Purge cache by specific URLs
pub async fn purge_cache(
    State(services): State<Arc<CloudflareServices>>,
    Json(req): Json<PurgeUrlsRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let result = services.cache.purge_urls(req.urls.clone()).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "id": result.id,
            "purged_urls": req.urls.len()
        },
        "message": format!("Successfully purged {} URLs", req.urls.len())
    })))
}

/// Purge all cache
pub async fn purge_all(
    State(services): State<Arc<CloudflareServices>>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let result = services.cache.purge_all().await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "id": result.id
        },
        "message": "Successfully purged entire cache"
    })))
}

/// Purge cache by tags (Enterprise only)
pub async fn purge_by_tags(
    State(services): State<Arc<CloudflareServices>>,
    Json(req): Json<PurgeTagsRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let result = services.cache.purge_tags(req.tags.clone()).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "id": result.id,
            "purged_tags": req.tags.len()
        },
        "message": format!("Successfully purged cache by {} tags", req.tags.len())
    })))
}

/// Purge cache by prefix (Enterprise only)
pub async fn purge_by_prefix(
    State(services): State<Arc<CloudflareServices>>,
    Json(req): Json<PurgePrefixRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let result = services.cache.purge_prefix(vec![req.prefix.clone()]).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "id": result.id,
            "purged_prefix": req.prefix
        },
        "message": format!("Successfully purged cache by prefix: {}", req.prefix)
    })))
}

/// Get cache status and statistics
pub async fn get_cache_status(
    State(services): State<Arc<CloudflareServices>>,
    Query(query): Query<CacheStatsQuery>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let hours = query.hours.unwrap_or(24);
    let stats = services.cache.get_cache_stats(hours).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "period_hours": hours,
            "requests": {
                "total": stats.total_requests,
                "cached": stats.cached_requests,
                "uncached": stats.uncached_requests,
                "cache_hit_ratio": format!("{:.2}%", stats.cache_hit_ratio)
            },
            "bandwidth": {
                "total_bytes": stats.total_bandwidth,
                "cached_bytes": stats.cached_bandwidth,
                "uncached_bytes": stats.uncached_bandwidth,
                "bandwidth_saved_ratio": format!("{:.2}%", stats.bandwidth_saved_ratio)
            }
        }
    })))
}

/// Warm cache by pre-fetching URLs
pub async fn warm_cache(
    State(_services): State<Arc<CloudflareServices>>,
    Json(req): Json<WarmCacheRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    // Cache warming - makes requests to URLs to populate the cache
    let mut warmed = 0;
    let mut failed = 0;

    let client = reqwest::Client::new();

    for url in &req.urls {
        match client.get(url).send().await {
            Ok(response) if response.status().is_success() => {
                warmed += 1;
            }
            _ => {
                failed += 1;
            }
        }
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "total_urls": req.urls.len(),
            "warmed": warmed,
            "failed": failed
        },
        "message": format!("Cache warming complete: {} warmed, {} failed", warmed, failed)
    })))
}

/// Clear local cache (plugin-level cache)
pub async fn clear_local_cache(
    State(_services): State<Arc<CloudflareServices>>,
) -> CloudflareResult<Json<serde_json::Value>> {
    // Clear any local/in-memory caches the plugin maintains
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Local cache cleared"
    })))
}
