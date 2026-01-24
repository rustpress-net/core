//! Editor API Types
//!
//! Request and response types for the editor REST API.

use crate::blocks::{Block, BlockId, BlockType};
use crate::post::{
    Author, FeaturedMedia, PostDocument, PostStats,
    PublishStatus,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Create a new post request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePostRequest {
    /// Post title
    pub title: String,

    /// Post type (post, page, etc.)
    #[serde(default = "default_post_type")]
    pub post_type: String,

    /// Initial content (optional)
    pub content: Option<String>,

    /// Initial blocks (optional)
    pub blocks: Option<Vec<Block>>,

    /// Author ID
    pub author_id: Option<i64>,

    /// Initial status
    #[serde(default)]
    pub status: PublishStatus,

    /// Category IDs
    #[serde(default)]
    pub category_ids: Vec<i64>,

    /// Tag IDs
    #[serde(default)]
    pub tag_ids: Vec<i64>,
}

fn default_post_type() -> String {
    "post".to_string()
}

/// Update post request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePostRequest {
    /// Updated title
    pub title: Option<String>,

    /// Updated slug
    pub slug: Option<String>,

    /// Updated blocks
    pub blocks: Option<Vec<Block>>,

    /// Updated excerpt
    pub excerpt: Option<String>,

    /// Updated status
    pub status: Option<PublishStatus>,

    /// Updated metadata
    pub metadata: Option<UpdateMetadataRequest>,

    /// Updated SEO settings
    pub seo: Option<UpdateSeoRequest>,

    /// Updated featured media
    pub featured_media: Option<FeaturedMediaRequest>,

    /// Updated publishing settings
    pub publishing: Option<UpdatePublishingRequest>,

    /// Create revision
    #[serde(default = "default_true")]
    pub create_revision: bool,
}

fn default_true() -> bool {
    true
}

/// Update metadata request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMetadataRequest {
    pub author_id: Option<i64>,
    pub category_ids: Option<Vec<i64>>,
    pub tag_ids: Option<Vec<i64>>,
    pub custom_fields: Option<serde_json::Value>,
}

/// Update SEO settings request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSeoRequest {
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub focus_keyword: Option<String>,
    pub canonical_url: Option<String>,
    pub robots_noindex: Option<bool>,
    pub robots_nofollow: Option<bool>,
    pub open_graph: Option<OpenGraphRequest>,
    pub twitter_card: Option<TwitterCardRequest>,
}

/// Open Graph request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenGraphRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub og_type: Option<String>,
}

/// Twitter Card request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitterCardRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub card_type: Option<String>,
}

/// Featured media request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturedMediaRequest {
    pub media_id: i64,
    pub alt: Option<String>,
    pub caption: Option<String>,
    pub focal_point_x: Option<f32>,
    pub focal_point_y: Option<f32>,
}

/// Update publishing request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePublishingRequest {
    pub status: Option<PublishStatus>,
    pub publish_at: Option<DateTime<Utc>>,
    pub password: Option<String>,
    pub visibility: Option<String>,
}

/// Post response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostResponse {
    pub id: i64,
    pub uuid: Uuid,
    pub post_type: String,
    pub title: String,
    pub slug: String,
    pub permalink: String,
    pub content: ContentResponse,
    pub excerpt: Option<String>,
    pub metadata: MetadataResponse,
    pub featured_media: Option<FeaturedMedia>,
    pub seo: SeoResponse,
    pub publishing: PublishingResponse,
    pub stats: PostStats,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub revision_count: u32,
}

impl From<PostDocument> for PostResponse {
    fn from(doc: PostDocument) -> Self {
        let permalink = doc.get_permalink("");
        let html = doc.content.get_html();
        let plain_text = doc.content.get_plain_text();
        Self {
            id: doc.id,
            uuid: doc.uuid,
            post_type: doc.post_type,
            title: doc.title,
            slug: doc.slug.clone(),
            permalink,
            content: ContentResponse {
                blocks: doc.content.blocks,
                html,
                plain_text,
            },
            excerpt: doc.excerpt,
            metadata: MetadataResponse {
                author: doc.metadata.author,
                categories: doc.metadata.categories.iter().map(|c| c.id).collect(),
                tags: doc.metadata.tags.iter().map(|t| t.id).collect(),
            },
            featured_media: doc.featured_media,
            seo: SeoResponse {
                meta_title: doc.seo.meta_title,
                meta_description: doc.seo.meta_description,
                focus_keyword: doc.seo.focus_keyword,
                canonical_url: doc.seo.canonical_url,
            },
            publishing: PublishingResponse {
                status: doc.publishing.status,
                published_at: doc.publishing.published_at,
                scheduled_at: doc.publishing.scheduled_at,
                visibility: format!("{:?}", doc.publishing.visibility).to_lowercase(),
            },
            stats: doc.stats,
            created_at: doc.created_at,
            updated_at: doc.modified_at,
            revision_count: doc.revisions.len() as u32,
        }
    }
}

/// Content response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentResponse {
    pub blocks: Vec<Block>,
    pub html: String,
    pub plain_text: String,
}

/// Metadata response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataResponse {
    pub author: Option<Author>,
    pub categories: Vec<i64>,
    pub tags: Vec<i64>,
}

/// SEO response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeoResponse {
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub focus_keyword: Option<String>,
    pub canonical_url: Option<String>,
}

/// Publishing response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishingResponse {
    pub status: PublishStatus,
    pub published_at: Option<DateTime<Utc>>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub visibility: String,
}

/// List posts request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPostsRequest {
    /// Page number (1-indexed)
    #[serde(default = "default_page")]
    pub page: u32,

    /// Items per page
    #[serde(default = "default_per_page")]
    pub per_page: u32,

    /// Post type filter
    pub post_type: Option<String>,

    /// Status filter
    pub status: Option<PublishStatus>,

    /// Author ID filter
    pub author_id: Option<i64>,

    /// Category ID filter
    pub category_id: Option<i64>,

    /// Tag ID filter
    pub tag_id: Option<i64>,

    /// Search query
    pub search: Option<String>,

    /// Sort field
    #[serde(default = "default_sort")]
    pub sort: String,

    /// Sort order
    #[serde(default = "default_order")]
    pub order: String,

    /// Date range start
    pub date_from: Option<DateTime<Utc>>,

    /// Date range end
    pub date_to: Option<DateTime<Utc>>,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    20
}

fn default_sort() -> String {
    "updated_at".to_string()
}

fn default_order() -> String {
    "desc".to_string()
}

/// List posts response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPostsResponse {
    pub posts: Vec<PostListItem>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

/// Post list item (summary)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostListItem {
    pub id: i64,
    pub uuid: Uuid,
    pub post_type: String,
    pub title: String,
    pub slug: String,
    pub status: PublishStatus,
    pub author: Option<AuthorSummary>,
    pub featured_image_url: Option<String>,
    pub excerpt: Option<String>,
    pub word_count: u32,
    pub reading_time: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

/// Author summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorSummary {
    pub id: i64,
    pub name: String,
    pub avatar_url: Option<String>,
}

// Block operations

/// Add block request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddBlockRequest {
    /// Block to add
    pub block: Block,

    /// Parent block ID (for nested blocks)
    pub parent_id: Option<BlockId>,

    /// Position within parent (or root if no parent)
    pub position: usize,
}

/// Move block request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveBlockRequest {
    /// Block ID to move
    pub block_id: BlockId,

    /// New parent ID (None for root level)
    pub new_parent_id: Option<BlockId>,

    /// New position
    pub new_position: usize,
}

/// Update block request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBlockRequest {
    /// Block ID
    pub block_id: BlockId,

    /// Updated content
    pub content: Option<String>,

    /// Updated attributes
    pub attributes: Option<serde_json::Value>,

    /// Updated styles
    pub styles: Option<serde_json::Value>,
}

/// Delete block request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteBlockRequest {
    /// Block ID to delete
    pub block_id: BlockId,

    /// Whether to delete children or move them up
    #[serde(default)]
    pub delete_children: bool,
}

/// Transform block request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformBlockRequest {
    /// Block ID to transform
    pub block_id: BlockId,

    /// Target block type
    pub target_type: BlockType,
}

/// Duplicate block request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateBlockRequest {
    /// Block ID to duplicate
    pub block_id: BlockId,

    /// Whether to duplicate children
    #[serde(default = "default_true")]
    pub include_children: bool,
}

// Revision operations

/// List revisions response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRevisionsResponse {
    pub revisions: Vec<RevisionSummary>,
}

/// Revision summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisionSummary {
    pub id: Uuid,
    pub version: u32,
    pub author_id: Option<i64>,
    pub author_name: Option<String>,
    pub title: String,
    pub word_count: u32,
    pub created_at: DateTime<Utc>,
    pub change_summary: Option<String>,
}

/// Compare revisions response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareRevisionsResponse {
    pub old_version: u32,
    pub new_version: u32,
    pub title_changed: bool,
    pub content_diff: Vec<DiffChunk>,
    pub blocks_added: u32,
    pub blocks_removed: u32,
    pub blocks_modified: u32,
}

/// Diff chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffChunk {
    pub change_type: DiffChangeType,
    pub old_text: Option<String>,
    pub new_text: Option<String>,
}

/// Diff change type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiffChangeType {
    Equal,
    Insert,
    Delete,
    Replace,
}

// Media operations

/// Upload media request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadMediaRequest {
    pub filename: String,
    pub content_type: String,
    pub alt: Option<String>,
    pub title: Option<String>,
    pub caption: Option<String>,
}

/// Upload media response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadMediaResponse {
    pub id: i64,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub file_size: u64,
    pub mime_type: String,
}

/// Media library request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaLibraryRequest {
    pub page: u32,
    pub per_page: u32,
    pub media_type: Option<String>,
    pub search: Option<String>,
}

/// Media library response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaLibraryResponse {
    pub items: Vec<MediaItem>,
    pub total: u64,
    pub page: u32,
    pub total_pages: u32,
}

/// Media item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    pub id: i64,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub filename: String,
    pub alt: Option<String>,
    pub title: Option<String>,
    pub caption: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub file_size: u64,
    pub mime_type: String,
    pub uploaded_at: DateTime<Utc>,
}

// Analysis operations

/// SEO analysis response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeoAnalysisResponse {
    pub score: u32,
    pub grade: String,
    pub checks: Vec<SeoCheckResponse>,
    pub suggestions: Vec<String>,
}

/// SEO check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeoCheckResponse {
    pub name: String,
    pub passed: bool,
    pub message: String,
    pub category: String,
}

/// Readability analysis response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadabilityAnalysisResponse {
    pub score: u32,
    pub grade: String,
    pub flesch_reading_ease: f32,
    pub grade_level: f32,
    pub reading_level: String,
    pub issues: Vec<ReadabilityIssueResponse>,
    pub suggestions: Vec<String>,
}

/// Readability issue response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadabilityIssueResponse {
    pub issue_type: String,
    pub severity: String,
    pub message: String,
    pub count: usize,
}

// Autosave

/// Autosave request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutosaveRequest {
    pub post_id: i64,
    pub title: Option<String>,
    pub blocks: Vec<Block>,
    pub client_timestamp: DateTime<Utc>,
}

/// Autosave response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutosaveResponse {
    pub autosave_id: Uuid,
    pub saved_at: DateTime<Utc>,
}

// Publish operations

/// Publish post request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishPostRequest {
    /// Publish immediately or schedule
    pub publish_at: Option<DateTime<Utc>>,

    /// Password protection
    pub password: Option<String>,

    /// Visibility (public, private, password)
    pub visibility: Option<String>,
}

/// Schedule post request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulePostRequest {
    pub publish_at: DateTime<Utc>,
}

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    pub fn not_found(message: &str) -> Self {
        Self {
            code: "NOT_FOUND".to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    pub fn validation_error(message: &str, details: serde_json::Value) -> Self {
        Self {
            code: "VALIDATION_ERROR".to_string(),
            message: message.to_string(),
            details: Some(details),
        }
    }

    pub fn internal_error(message: &str) -> Self {
        Self {
            code: "INTERNAL_ERROR".to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    pub fn unauthorized() -> Self {
        Self {
            code: "UNAUTHORIZED".to_string(),
            message: "Authentication required".to_string(),
            details: None,
        }
    }

    pub fn forbidden() -> Self {
        Self {
            code: "FORBIDDEN".to_string(),
            message: "You do not have permission to perform this action".to_string(),
            details: None,
        }
    }
}
