//! Analytics API handlers

use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use crate::error::CloudflareResult;
use crate::services::CloudflareServices;

#[derive(Debug, Deserialize)]
pub struct AnalyticsQuery {
    pub time_range: Option<String>,
    pub hours: Option<i32>,
}

/// Get dashboard analytics
pub async fn get_analytics(
    State(services): State<Arc<CloudflareServices>>,
    Query(query): Query<AnalyticsQuery>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let hours = query.hours.unwrap_or(24);
    let analytics = services.analytics.get_dashboard(hours).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": analytics
    })))
}

/// Get zone analytics summary
pub async fn get_zone_analytics(
    State(services): State<Arc<CloudflareServices>>,
    Query(query): Query<AnalyticsQuery>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let hours = query.hours.unwrap_or(24);
    let analytics = services.analytics.get_dashboard(hours).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "zone": analytics,
            "period_hours": hours
        }
    })))
}

/// Get traffic summary
pub async fn get_traffic_summary(
    State(services): State<Arc<CloudflareServices>>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let summary = services.analytics.get_traffic_summary().await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "total_requests": summary.total_requests,
            "cached_requests": summary.cached_requests,
            "uncached_requests": summary.uncached_requests,
            "cache_hit_rate": format!("{:.2}%", summary.cache_hit_rate),
            "bandwidth": {
                "total_bytes": summary.total_bandwidth_bytes,
                "cached_bytes": summary.cached_bandwidth_bytes,
                "total_formatted": format_bytes(summary.total_bandwidth_bytes),
                "cached_formatted": format_bytes(summary.cached_bandwidth_bytes)
            },
            "threats_blocked": summary.threats_blocked
        }
    })))
}

/// Get bandwidth statistics
pub async fn get_bandwidth_stats(
    State(services): State<Arc<CloudflareServices>>,
    Query(query): Query<AnalyticsQuery>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let hours = query.hours.unwrap_or(24);
    let analytics = services.analytics.get_dashboard(hours).await?;
    let totals = analytics.totals.unwrap_or_default();
    let bandwidth = totals.bandwidth.unwrap_or_default();

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "period_hours": hours,
            "total_bytes": bandwidth.all,
            "cached_bytes": bandwidth.cached,
            "uncached_bytes": bandwidth.uncached,
            "total_formatted": format_bytes(bandwidth.all),
            "cached_formatted": format_bytes(bandwidth.cached),
            "uncached_formatted": format_bytes(bandwidth.uncached),
            "savings_percentage": if bandwidth.all > 0 {
                format!("{:.2}%", (bandwidth.cached as f64 / bandwidth.all as f64) * 100.0)
            } else {
                "0.00%".to_string()
            }
        }
    })))
}

/// Get security analytics summary
pub async fn get_security_summary(
    State(services): State<Arc<CloudflareServices>>,
    Query(query): Query<AnalyticsQuery>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let hours = query.hours.unwrap_or(24);
    let analytics = services.analytics.get_dashboard(hours).await?;
    let totals = analytics.totals.unwrap_or_default();
    let threats = totals.threats.unwrap_or_default();

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "period_hours": hours,
            "threats_blocked": threats.all,
            "threats_by_country": threats.country,
            "threats_by_type": threats.threat_type
        }
    })))
}

/// Get cache analytics
pub async fn get_cache_analytics(
    State(services): State<Arc<CloudflareServices>>,
    Query(query): Query<AnalyticsQuery>,
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
                "hit_ratio": format!("{:.2}%", stats.cache_hit_ratio)
            },
            "bandwidth": {
                "total_bytes": stats.total_bandwidth,
                "cached_bytes": stats.cached_bandwidth,
                "uncached_bytes": stats.uncached_bandwidth,
                "savings_ratio": format!("{:.2}%", stats.bandwidth_saved_ratio),
                "total_formatted": format_bytes(stats.total_bandwidth),
                "cached_formatted": format_bytes(stats.cached_bandwidth)
            }
        }
    })))
}

/// Get performance metrics
pub async fn get_performance_metrics(
    State(services): State<Arc<CloudflareServices>>,
    Query(query): Query<AnalyticsQuery>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let hours = query.hours.unwrap_or(24);
    let analytics = services.analytics.get_dashboard(hours).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "period_hours": hours,
            "totals": analytics.totals,
            "timeseries": analytics.timeseries
        }
    })))
}

/// Format bytes to human-readable string
fn format_bytes(bytes: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = KB * 1024;
    const GB: i64 = MB * 1024;
    const TB: i64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
