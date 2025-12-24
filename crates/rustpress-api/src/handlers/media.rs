//! Media handlers.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Update media request
#[derive(Debug, Deserialize)]
pub struct UpdateMediaRequest {
    pub title: Option<String>,
    pub alt_text: Option<String>,
    pub caption: Option<String>,
    pub description: Option<String>,
}

/// Media response
#[derive(Debug, Serialize)]
pub struct MediaResponse {
    pub id: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub mime_type: String,
    pub size: i64,
    pub url: String,
    pub title: Option<String>,
    pub alt_text: Option<String>,
    pub caption: Option<String>,
    pub description: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub thumbnails: Vec<ThumbnailInfo>,
    pub uploader: UploaderInfo,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Thumbnail info
#[derive(Debug, Serialize)]
pub struct ThumbnailInfo {
    pub size: String,
    pub url: String,
    pub width: i32,
    pub height: i32,
}

/// Uploader info
#[derive(Debug, Serialize)]
pub struct UploaderInfo {
    pub id: Uuid,
    pub name: String,
}

/// Media list query parameters
#[derive(Debug, Deserialize)]
pub struct MediaListQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub mime_type: Option<String>,
    pub uploader_id: Option<Uuid>,
    pub search: Option<String>,
    pub order_by: Option<String>,
}

/// Upload response
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub id: Uuid,
    pub url: String,
    pub filename: String,
    pub mime_type: String,
    pub size: i64,
}
