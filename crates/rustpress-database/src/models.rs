//! Database model definitions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Re-export repository models
pub use crate::repository::posts::PostRow;
pub use crate::repository::users::UserRow;
pub use crate::repository::options::{OptionRow, SettingsGroup};
pub use crate::repository::comments::{
    CommentRow, CommentMetaRow, CommentLikeRow, CommentWithAuthor,
    CommentStatus, CreateComment, UpdateComment, CommentListParams,
    CommentsRepository
};
pub use crate::repository::themes::{
    ThemeRow, ThemeOptionRow, ThemePreviewRow,
    ThemeMenuAssignmentRow, ThemeWidgetAssignmentRow, ThemeRepository
};

/// Media model
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct MediaRow {
    pub id: Uuid,
    #[sqlx(rename = "site_id")]
    pub tenant_id: Option<Uuid>,
    pub uploader_id: Option<Uuid>,
    pub filename: String,
    pub original_filename: String,
    pub mime_type: String,
    pub file_size: i64,
    pub storage_path: String,
    pub storage_backend: Option<String>,
    pub alt_text: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration: Option<i32>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Page model - stored in posts table with post_type='page'
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct PageRow {
    pub id: Uuid,
    pub site_id: Option<Uuid>,
    pub post_type: String,
    pub author_id: Uuid,
    pub title: String,
    pub slug: String,
    pub content: Option<String>,
    pub excerpt: Option<String>,
    pub status: String,
    pub visibility: String,
    pub password: Option<String>,
    pub parent_id: Option<Uuid>,
    pub menu_order: i32,
    pub template: Option<String>,
    pub featured_image_id: Option<Uuid>,
    pub comment_status: String,
    pub comment_count: i32,
    pub ping_status: String,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub canonical_url: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl PageRow {
    /// Columns to select from posts table (excludes search_vector, casts enums to text)
    pub const COLUMNS: &'static str = "id, site_id, post_type::text as post_type, author_id, title, slug, content, excerpt, status::text as status, visibility, password, parent_id, menu_order, template, featured_image_id, comment_status, comment_count, ping_status, meta_title, meta_description, canonical_url, published_at, scheduled_at, created_at, updated_at, deleted_at";
}

/// Taxonomy model
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct TaxonomyRow {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub hierarchical: bool,
}

/// Term model
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct TermRow {
    pub id: Uuid,
    pub taxonomy_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub term_order: Option<i32>,
    pub count: Option<i32>,
}

/// Term relationship model
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct TermRelationshipRow {
    pub object_id: Uuid,
    pub object_type: String,
    pub term_id: Uuid,
    pub term_order: Option<i32>,
}

/// Session model
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct SessionRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub last_active_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Tenant model
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct TenantRow {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub domain: Option<String>,
    pub status: String,
    pub plan: Option<String>,
    pub owner_id: Option<Uuid>,
    pub settings: serde_json::Value,
    pub quotas: serde_json::Value,
    pub trial_ends_at: Option<DateTime<Utc>>,
    pub subscription_ends_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Job model
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct JobRow {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub queue: String,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub priority: Option<i32>,
    pub attempts: Option<i32>,
    pub max_attempts: Option<i32>,
    pub last_error: Option<String>,
    pub available_at: DateTime<Utc>,
    pub reserved_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Webhook model
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct WebhookRow {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub name: String,
    pub url: String,
    pub secret: Option<String>,
    pub events: Vec<String>,
    pub is_active: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub failure_count: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Webhook delivery model
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct WebhookDeliveryRow {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub response_status: Option<i32>,
    pub response_body: Option<String>,
    pub duration_ms: Option<i32>,
    pub success: Option<bool>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Audit log model
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct AuditLogRow {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<Uuid>,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}
