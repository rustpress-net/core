//! Post handlers.

use chrono::{DateTime, Utc};
use rustpress_core::types::PostStatus;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Create post request
#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub slug: Option<String>,
    pub status: Option<PostStatus>,
    pub featured_image_id: Option<Uuid>,
    pub category_ids: Option<Vec<Uuid>>,
    pub tag_ids: Option<Vec<Uuid>>,
    pub meta: Option<serde_json::Value>,
}

/// Update post request
#[derive(Debug, Deserialize)]
pub struct UpdatePostRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub excerpt: Option<String>,
    pub slug: Option<String>,
    pub status: Option<PostStatus>,
    pub featured_image_id: Option<Uuid>,
    pub category_ids: Option<Vec<Uuid>>,
    pub tag_ids: Option<Vec<Uuid>>,
    pub meta: Option<serde_json::Value>,
}

/// Post response
#[derive(Debug, Serialize)]
pub struct PostResponse {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub status: PostStatus,
    pub author: AuthorInfo,
    pub featured_image: Option<MediaInfo>,
    pub categories: Vec<TaxonomyInfo>,
    pub tags: Vec<TaxonomyInfo>,
    pub meta: serde_json::Value,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Author info for post response
#[derive(Debug, Serialize)]
pub struct AuthorInfo {
    pub id: Uuid,
    pub name: String,
    pub avatar_url: Option<String>,
}

/// Media info for featured image
#[derive(Debug, Serialize)]
pub struct MediaInfo {
    pub id: Uuid,
    pub url: String,
    pub alt_text: Option<String>,
}

/// Taxonomy info (categories, tags)
#[derive(Debug, Serialize)]
pub struct TaxonomyInfo {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

/// Post list query parameters
#[derive(Debug, Deserialize)]
pub struct PostListQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub status: Option<PostStatus>,
    pub author_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub tag_id: Option<Uuid>,
    pub search: Option<String>,
    pub order_by: Option<String>,
    pub order_dir: Option<String>,
}
