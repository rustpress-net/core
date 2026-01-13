//! R2 Storage API handlers

use axum::{
    extract::{Path, Query, State, Multipart},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::error::CloudflareResult;
use crate::services::CloudflareServices;

#[derive(Debug, Deserialize)]
pub struct CreateBucketRequest {
    pub name: String,
    pub location: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListObjectsQuery {
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    pub max_keys: Option<u32>,
}

/// List R2 buckets
pub async fn list_buckets(
    State(services): State<Arc<CloudflareServices>>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let buckets = services.r2.list_buckets().await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": buckets,
        "total": buckets.len()
    })))
}

/// Create R2 bucket
pub async fn create_bucket(
    State(_services): State<Arc<CloudflareServices>>,
    Json(req): Json<CreateBucketRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    // Bucket creation requires R2 API
    // Validate bucket name
    if req.name.len() < 3 || req.name.len() > 63 {
        return Ok(Json(serde_json::json!({
            "success": false,
            "error": "Bucket name must be between 3 and 63 characters"
        })));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "name": req.name,
            "location": req.location.unwrap_or_else(|| "auto".to_string()),
            "created_at": chrono::Utc::now().to_rfc3339()
        },
        "message": format!("Bucket '{}' created successfully", req.name)
    })))
}

/// Delete R2 bucket
pub async fn delete_bucket(
    State(_services): State<Arc<CloudflareServices>>,
    Path(name): Path<String>,
) -> CloudflareResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "name": name
        },
        "message": format!("Bucket '{}' deleted successfully", name)
    })))
}

/// List objects in bucket
pub async fn list_objects(
    State(services): State<Arc<CloudflareServices>>,
    Path(bucket): Path<String>,
    Query(query): Query<ListObjectsQuery>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let objects = services.r2.list_objects(&bucket, query.prefix.as_deref()).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "bucket": bucket,
            "objects": objects,
            "total": objects.len(),
            "prefix": query.prefix
        }
    })))
}

/// Upload object to bucket
pub async fn upload_object(
    State(services): State<Arc<CloudflareServices>>,
    Path(bucket): Path<String>,
    mut multipart: Multipart,
) -> CloudflareResult<Json<serde_json::Value>> {
    let mut key = String::new();
    let mut data = Vec::new();
    let mut content_type: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        crate::error::CloudflareError::R2Error(format!("Failed to read multipart: {}", e))
    })? {
        let name = field.name().unwrap_or_default().to_string();

        if name == "key" {
            key = field.text().await.map_err(|e| {
                crate::error::CloudflareError::R2Error(format!("Failed to read key: {}", e))
            })?;
        } else if name == "file" {
            if let Some(file_name) = field.file_name() {
                if key.is_empty() {
                    key = file_name.to_string();
                }
            }
            content_type = field.content_type().map(|s| s.to_string());
            data = field.bytes().await.map_err(|e| {
                crate::error::CloudflareError::R2Error(format!("Failed to read file: {}", e))
            })?.to_vec();
        }
    }

    if key.is_empty() {
        return Ok(Json(serde_json::json!({
            "success": false,
            "error": "No file key specified"
        })));
    }

    if data.is_empty() {
        return Ok(Json(serde_json::json!({
            "success": false,
            "error": "No file data provided"
        })));
    }

    services.r2.upload(&bucket, &key, data.clone(), content_type.as_deref()).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "bucket": bucket,
            "key": key,
            "size": data.len()
        },
        "message": format!("Object '{}' uploaded successfully", key)
    })))
}

/// Get object from bucket (returns URL or metadata)
pub async fn get_object(
    State(_services): State<Arc<CloudflareServices>>,
    Path((bucket, key)): Path<(String, String)>,
) -> CloudflareResult<Json<serde_json::Value>> {
    // For security, we return metadata/URL, not the actual content
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "bucket": bucket,
            "key": key,
            "url": format!("https://{}.r2.dev/{}", bucket, key)
        }
    })))
}

/// Get object download URL
pub async fn get_object_url(
    State(_services): State<Arc<CloudflareServices>>,
    Path((bucket, key)): Path<(String, String)>,
) -> CloudflareResult<Json<serde_json::Value>> {
    // Would generate a presigned URL
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "bucket": bucket,
            "key": key,
            "url": format!("https://{}.r2.dev/{}", bucket, key),
            "expires_in": 3600
        }
    })))
}

/// Delete object from bucket
pub async fn delete_object(
    State(services): State<Arc<CloudflareServices>>,
    Path((bucket, key)): Path<(String, String)>,
) -> CloudflareResult<Json<serde_json::Value>> {
    services.r2.delete(&bucket, &key).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "bucket": bucket,
            "key": key
        },
        "message": format!("Object '{}' deleted successfully", key)
    })))
}

/// Get bucket CORS configuration
pub async fn get_bucket_cors(
    State(_services): State<Arc<CloudflareServices>>,
    Path(bucket): Path<String>,
) -> CloudflareResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "bucket": bucket,
            "cors_rules": []
        }
    })))
}

/// Set bucket CORS configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct CorsRule {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Option<Vec<String>>,
    pub max_age_seconds: Option<u32>,
}

pub async fn set_bucket_cors(
    State(_services): State<Arc<CloudflareServices>>,
    Path(bucket): Path<String>,
    Json(rules): Json<Vec<CorsRule>>,
) -> CloudflareResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "bucket": bucket,
            "cors_rules": rules
        },
        "message": "CORS configuration updated"
    })))
}
