//! DNS API handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use crate::error::CloudflareResult;
use crate::models::{CreateDnsRecord, DnsListParams, UpdateDnsRecord};
use crate::services::CloudflareServices;

/// Query parameters for listing DNS records
#[derive(Debug, Deserialize)]
pub struct ListDnsQuery {
    #[serde(rename = "type")]
    pub record_type: Option<String>,
    pub name: Option<String>,
    pub content: Option<String>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

/// Import zone file request
#[derive(Debug, Deserialize)]
pub struct ImportZoneRequest {
    pub zone_file: String,
}

/// List all DNS records
pub async fn list_records(
    State(services): State<Arc<CloudflareServices>>,
    Query(query): Query<ListDnsQuery>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let params = if query.record_type.is_some() || query.name.is_some() || query.content.is_some() {
        Some(DnsListParams {
            record_type: query.record_type,
            name: query.name,
            content: query.content,
            page: query.page,
            per_page: query.per_page,
            ..Default::default()
        })
    } else {
        None
    };

    let records = services.dns.list(params).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": records,
        "total": records.len()
    })))
}

/// Get a single DNS record by ID
pub async fn get_record(
    State(services): State<Arc<CloudflareServices>>,
    Path(id): Path<String>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let record = services.dns.get(&id).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": record
    })))
}

/// Create a new DNS record
pub async fn create_record(
    State(services): State<Arc<CloudflareServices>>,
    Json(req): Json<CreateDnsRecord>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let record = services.dns.create(req).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": record,
        "message": format!("DNS record {} created successfully", record.name)
    })))
}

/// Update an existing DNS record
pub async fn update_record(
    State(services): State<Arc<CloudflareServices>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateDnsRecord>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let record = services.dns.update(&id, req).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": record,
        "message": format!("DNS record {} updated successfully", record.name)
    })))
}

/// Delete a DNS record
pub async fn delete_record(
    State(services): State<Arc<CloudflareServices>>,
    Path(id): Path<String>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let result = services.dns.delete(&id).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "id": result.id
        },
        "message": "DNS record deleted successfully"
    })))
}

/// Export zone file
pub async fn export_zone(
    State(services): State<Arc<CloudflareServices>>,
) -> CloudflareResult<String> {
    let zone_file = services.dns.export_zone_file().await?;
    Ok(zone_file)
}

/// Import zone file (placeholder - Cloudflare doesn't support direct import via API)
pub async fn import_zone(
    State(_services): State<Arc<CloudflareServices>>,
    Json(_req): Json<ImportZoneRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    // Note: Cloudflare doesn't support direct zone file import via API
    // This would need to parse the zone file and create records individually
    Ok(Json(serde_json::json!({
        "success": false,
        "message": "Zone file import is not yet supported. Please use the Cloudflare dashboard."
    })))
}

/// Sync DNS records from Cloudflare to local database
pub async fn sync_records(
    State(services): State<Arc<CloudflareServices>>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let result = services.dns.full_sync().await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "total": result.total,
            "synced": result.synced,
            "errors": result.errors
        },
        "message": format!("Synced {} records ({} errors)", result.synced, result.errors)
    })))
}
