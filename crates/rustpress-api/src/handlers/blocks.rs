//! Block Library handlers.
//!
//! Handlers for content block operations including listing blocks,
//! managing favorites, recent blocks, and custom blocks.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Block definition for the library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub category: String,
    pub html_template: String,
    pub default_attributes: serde_json::Value,
    pub supports: BlockSupports,
}

/// Block capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlockSupports {
    pub alignment: bool,
    pub color: bool,
    pub typography: bool,
    pub spacing: bool,
    pub html: bool,
    pub reusable: bool,
}

/// Block category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockCategory {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub is_system: bool,
}

/// User block preferences response
#[derive(Debug, Clone, Serialize)]
pub struct UserBlockPreferencesResponse {
    pub user_id: Uuid,
    pub recent_blocks: Vec<String>,
    pub favorite_blocks: Vec<String>,
    pub block_settings: serde_json::Value,
    pub updated_at: DateTime<Utc>,
}

/// Update recent blocks request
#[derive(Debug, Deserialize)]
pub struct UpdateRecentBlocksRequest {
    pub block_id: String,
}

/// Toggle favorite block request
#[derive(Debug, Deserialize)]
pub struct ToggleFavoriteBlockRequest {
    pub block_id: String,
}

/// Toggle favorite response
#[derive(Debug, Serialize)]
pub struct ToggleFavoriteResponse {
    pub block_id: String,
    pub is_favorite: bool,
}

/// Custom block response
#[derive(Debug, Clone, Serialize)]
pub struct CustomBlockResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub icon: String,
    pub content: String,
    pub preview_html: Option<String>,
    pub settings: serde_json::Value,
    pub is_global: bool,
    pub usage_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create custom block request
#[derive(Debug, Deserialize)]
pub struct CreateCustomBlockRequest {
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub icon: Option<String>,
    pub content: String,
    pub preview_html: Option<String>,
    pub settings: Option<serde_json::Value>,
    pub is_global: Option<bool>,
}

/// Update custom block request
#[derive(Debug, Deserialize)]
pub struct UpdateCustomBlockRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub icon: Option<String>,
    pub content: Option<String>,
    pub preview_html: Option<String>,
    pub settings: Option<serde_json::Value>,
    pub is_global: Option<bool>,
}

/// Block list query parameters
#[derive(Debug, Deserialize, Default)]
pub struct BlockListQuery {
    pub category: Option<String>,
    pub search: Option<String>,
    pub include_custom: Option<bool>,
}

/// Custom block list query parameters
#[derive(Debug, Deserialize, Default)]
pub struct CustomBlockListQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub category: Option<String>,
    pub search: Option<String>,
    pub is_global: Option<bool>,
}

/// Block usage analytics entry
#[derive(Debug, Clone, Serialize)]
pub struct BlockUsageEntry {
    pub block_id: String,
    pub block_type: String,
    pub usage_count: i64,
    pub last_used: DateTime<Utc>,
}

/// Track block usage request
#[derive(Debug, Deserialize)]
pub struct TrackBlockUsageRequest {
    pub block_id: String,
    pub block_type: String,
    pub post_id: Option<Uuid>,
    pub action: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Block analytics response
#[derive(Debug, Serialize)]
pub struct BlockAnalyticsResponse {
    pub total_insertions: i64,
    pub unique_blocks_used: i64,
    pub most_used_blocks: Vec<BlockUsageEntry>,
    pub recent_activity: Vec<BlockUsageEntry>,
}

/// Paginated custom block list response
#[derive(Debug, Serialize)]
pub struct CustomBlockListResponse {
    pub items: Vec<CustomBlockResponse>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

/// Block library response with all blocks
#[derive(Debug, Serialize)]
pub struct BlockLibraryResponse {
    pub blocks: Vec<BlockDefinition>,
    pub categories: Vec<BlockCategory>,
    pub custom_blocks: Vec<CustomBlockResponse>,
    pub recent_blocks: Vec<String>,
    pub favorite_blocks: Vec<String>,
}

/// Update block settings request
#[derive(Debug, Deserialize)]
pub struct UpdateBlockSettingsRequest {
    pub settings: serde_json::Value,
}
