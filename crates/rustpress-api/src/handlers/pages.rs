//! Page handlers.

use chrono::{DateTime, Utc};
use rustpress_core::types::PostStatus;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Create page request
#[derive(Debug, Deserialize)]
pub struct CreatePageRequest {
    pub title: String,
    pub content: String,
    pub slug: Option<String>,
    pub status: Option<PostStatus>,
    pub parent_id: Option<Uuid>,
    pub template: Option<String>,
    pub menu_order: Option<i32>,
    pub meta: Option<serde_json::Value>,
}

/// Update page request
#[derive(Debug, Deserialize)]
pub struct UpdatePageRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub slug: Option<String>,
    pub status: Option<PostStatus>,
    pub parent_id: Option<Uuid>,
    pub template: Option<String>,
    pub menu_order: Option<i32>,
    pub meta: Option<serde_json::Value>,
}

/// Page response
#[derive(Debug, Serialize)]
pub struct PageResponse {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub status: PostStatus,
    pub parent_id: Option<Uuid>,
    pub template: Option<String>,
    pub menu_order: i32,
    pub author: super::posts::AuthorInfo,
    pub meta: serde_json::Value,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Page list query parameters
#[derive(Debug, Deserialize)]
pub struct PageListQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub status: Option<PostStatus>,
    pub parent_id: Option<Uuid>,
    pub search: Option<String>,
    pub order_by: Option<String>,
}
