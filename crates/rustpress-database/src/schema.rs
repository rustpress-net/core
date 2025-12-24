//! Database schema definitions and model types.
//!
//! This module contains all the Rust struct definitions that map to
//! the PostgreSQL database tables.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// =============================================================================
// Sites (Multi-site support - Point 55)
// =============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Site {
    pub id: Uuid,
    pub domain: String,
    pub path: String,
    pub subdomain: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub is_primary: bool,
    pub is_public: bool,
    pub language: String,
    pub timezone: String,
    pub date_format: Option<String>,
    pub time_format: Option<String>,
    pub storage_quota_bytes: Option<i64>,
    pub storage_used_bytes: i64,
    pub max_users: Option<i32>,
    pub max_posts: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SiteMeta {
    pub id: Uuid,
    pub site_id: Uuid,
    pub meta_key: String,
    pub meta_value: Option<serde_json::Value>,
    pub is_autoload: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// Users (Point 34 - with Argon2 password hashing)
// =============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub site_id: Option<Uuid>,
    pub email: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub display_name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub nickname: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub website_url: Option<String>,
    pub phone: Option<String>,
    pub locale: Option<String>,
    pub timezone: Option<String>,
    pub status: String,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub two_factor_enabled: bool,
    #[serde(skip_serializing)]
    pub two_factor_secret: Option<String>,
    #[serde(skip_serializing)]
    pub two_factor_recovery_codes: Option<serde_json::Value>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub last_login_ip: Option<String>,
    pub failed_login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub password_changed_at: Option<DateTime<Utc>>,
    pub must_change_password: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserMeta {
    pub id: Uuid,
    pub user_id: Uuid,
    pub meta_key: String,
    pub meta_value: Option<serde_json::Value>,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// Roles & Permissions (Point 35)
// =============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Role {
    pub id: Uuid,
    pub site_id: Option<Uuid>,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub parent_role_id: Option<Uuid>,
    pub priority: i32,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Capability {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub resource: String,
    pub action: String,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct RoleCapability {
    pub role_id: Uuid,
    pub capability_id: Uuid,
    pub granted: bool,
    pub restrictions: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserRole {
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub site_id: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// Posts (Point 31 - UUID primary keys)
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "post_type", rename_all = "lowercase")]
pub enum PostType {
    Post,
    Page,
    Attachment,
    Revision,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "post_status", rename_all = "lowercase")]
pub enum PostStatus {
    Draft,
    Pending,
    Private,
    Published,
    Scheduled,
    Trash,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Post {
    pub id: Uuid,
    pub site_id: Option<Uuid>,
    pub post_type: PostType,
    pub author_id: Uuid,
    pub title: String,
    pub slug: String,
    pub content: Option<String>,
    pub excerpt: Option<String>,
    pub status: PostStatus,
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

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PostMeta {
    pub id: Uuid,
    pub post_id: Uuid,
    pub meta_key: String,
    pub meta_value: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// Revisions (Point 39)
// =============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Revision {
    pub id: Uuid,
    pub post_id: Uuid,
    pub author_id: Uuid,
    pub revision_number: i32,
    pub title: String,
    pub content: Option<String>,
    pub excerpt: Option<String>,
    pub meta_snapshot: Option<serde_json::Value>,
    pub change_summary: Option<String>,
    pub changes: Option<serde_json::Value>,
    pub revision_type: String,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// Taxonomies (Point 33)
// =============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Taxonomy {
    pub id: Uuid,
    pub site_id: Option<Uuid>,
    pub name: String,
    pub singular_name: String,
    pub plural_name: String,
    pub is_hierarchical: bool,
    pub is_public: bool,
    pub show_in_menu: bool,
    pub show_in_rest: bool,
    pub post_types: serde_json::Value,
    pub slug: Option<String>,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Term {
    pub id: Uuid,
    pub site_id: Option<Uuid>,
    pub taxonomy_id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub term_order: i32,
    pub count: i32,
    pub image_id: Option<Uuid>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TermMeta {
    pub id: Uuid,
    pub term_id: Uuid,
    pub meta_key: String,
    pub meta_value: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PostTerm {
    pub post_id: Uuid,
    pub term_id: Uuid,
    pub term_order: i32,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// Media (Point 37)
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "media_type", rename_all = "lowercase")]
pub enum MediaType {
    Image,
    Video,
    Audio,
    Document,
    Archive,
    Other,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Media {
    pub id: Uuid,
    pub site_id: Option<Uuid>,
    pub uploader_id: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub mime_type: String,
    pub media_type: MediaType,
    pub file_size: i64,
    pub file_hash: Option<String>,
    pub storage_path: String,
    pub storage_driver: String,
    pub url: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration: Option<i32>,
    pub title: Option<String>,
    pub alt_text: Option<String>,
    pub caption: Option<String>,
    pub description: Option<String>,
    pub exif_data: Option<serde_json::Value>,
    pub thumbnails: Option<serde_json::Value>,
    pub attached_to_post_id: Option<Uuid>,
    pub status: String,
    pub focus_x: Option<f64>,
    pub focus_y: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct MediaMeta {
    pub id: Uuid,
    pub media_id: Uuid,
    pub meta_key: String,
    pub meta_value: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// Comments (Point 36 - Nested replies)
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "comment_status", rename_all = "lowercase")]
pub enum CommentStatus {
    Pending,
    Approved,
    Spam,
    Trash,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Comment {
    pub id: Uuid,
    pub site_id: Option<Uuid>,
    pub post_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub depth: i32,
    pub user_id: Option<Uuid>,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub author_url: Option<String>,
    pub author_ip: Option<String>,
    pub content: String,
    pub content_html: Option<String>,
    pub status: CommentStatus,
    pub is_edited: bool,
    pub edited_at: Option<DateTime<Utc>>,
    pub moderated_by: Option<Uuid>,
    pub moderated_at: Option<DateTime<Utc>>,
    pub moderation_note: Option<String>,
    pub spam_score: Option<f64>,
    pub spam_reasons: Option<serde_json::Value>,
    pub likes_count: i32,
    pub replies_count: i32,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct CommentMeta {
    pub id: Uuid,
    pub comment_id: Uuid,
    pub meta_key: String,
    pub meta_value: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// Options/Settings (Point 38)
// =============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SiteOption {
    pub id: Uuid,
    pub site_id: Option<Uuid>,
    pub option_name: String,
    pub option_value: Option<serde_json::Value>,
    pub option_group: String,
    pub autoload: bool,
    pub is_system: bool,
    pub value_type: Option<String>,
    pub validation: Option<serde_json::Value>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// Sessions (Point 40)
// =============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub site_id: Option<Uuid>,
    pub user_id: Uuid,
    #[serde(skip_serializing)]
    pub token_hash: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_type: Option<String>,
    pub device_name: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub data: serde_json::Value,
    pub is_revoked: bool,
    pub revoked_reason: Option<String>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub last_active_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// Menus (Point 46)
// =============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Menu {
    pub id: Uuid,
    pub site_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct MenuItem {
    pub id: Uuid,
    pub menu_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub position: i32,
    pub depth: i32,
    pub item_type: String,
    pub object_id: Option<Uuid>,
    pub object_type: Option<String>,
    pub url: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub target: Option<String>,
    pub css_classes: Option<String>,
    pub xfn: Option<String>,
    pub icon: Option<String>,
    pub icon_position: Option<String>,
    pub is_visible: bool,
    pub visible_to_logged_in: Option<bool>,
    pub visible_to_roles: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// Widgets/Blocks (Point 47)
// =============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct WidgetArea {
    pub id: Uuid,
    pub site_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub theme: Option<String>,
    pub area_type: String,
    pub before_widget: Option<String>,
    pub after_widget: Option<String>,
    pub before_title: Option<String>,
    pub after_title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Widget {
    pub id: Uuid,
    pub site_id: Option<Uuid>,
    pub area_id: Option<Uuid>,
    pub position: i32,
    pub widget_type: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub settings: serde_json::Value,
    pub display_rules: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct BlockTemplate {
    pub id: Uuid,
    pub site_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub content: String,
    pub content_parsed: Option<serde_json::Value>,
    pub block_type: String,
    pub category: Option<String>,
    pub keywords: serde_json::Value,
    pub usage_count: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// =============================================================================
// Relationships (Point 50)
// =============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct RelationshipType {
    pub id: Uuid,
    pub site_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub from_post_types: serde_json::Value,
    pub to_post_types: serde_json::Value,
    pub cardinality: String,
    pub from_label: Option<String>,
    pub to_label: Option<String>,
    pub is_bidirectional: bool,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PostRelationship {
    pub id: Uuid,
    pub type_id: Uuid,
    pub from_post_id: Uuid,
    pub to_post_id: Uuid,
    pub sort_order: i32,
    pub meta: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// Notifications (Point 51)
// =============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub site_id: Option<Uuid>,
    pub user_id: Uuid,
    pub r#type: String,
    pub title: String,
    pub message: Option<String>,
    pub data: serde_json::Value,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub action_url: Option<String>,
    pub action_text: Option<String>,
    pub sender_id: Option<Uuid>,
    pub is_read: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub group_key: Option<String>,
    pub is_grouped: bool,
    pub group_count: i32,
    pub priority: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// Jobs (Point 52)
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "job_status", rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub site_id: Option<Uuid>,
    pub queue: String,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub priority: i32,
    pub status: JobStatus,
    pub attempts: i32,
    pub max_attempts: i32,
    pub last_error: Option<String>,
    pub error_details: Option<serde_json::Value>,
    pub available_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub timeout_seconds: Option<i32>,
    pub worker_id: Option<String>,
    pub locked_at: Option<DateTime<Utc>>,
    pub locked_until: Option<DateTime<Utc>>,
    pub depends_on: Option<Vec<Uuid>>,
    pub batch_id: Option<Uuid>,
    pub result: Option<serde_json::Value>,
    pub tags: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// Audit Logs (Point 41)
// =============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub site_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub user_email: Option<String>,
    pub user_name: Option<String>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<Uuid>,
    pub entity_title: Option<String>,
    pub description: Option<String>,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub changes: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<Uuid>,
    pub request_url: Option<String>,
    pub request_method: Option<String>,
    pub severity: String,
    pub category: Option<String>,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// Redirects (Point 54)
// =============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Redirect {
    pub id: Uuid,
    pub site_id: Option<Uuid>,
    pub source_url: String,
    pub source_url_hash: String,
    pub match_type: String,
    pub is_case_sensitive: bool,
    pub target_url: String,
    pub redirect_type: String,
    pub preserve_query_string: bool,
    pub conditions: serde_json::Value,
    pub is_active: bool,
    pub priority: i32,
    pub hit_count: i64,
    pub last_hit_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// =============================================================================
// Cache (Point 53)
// =============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub site_id: Option<Uuid>,
    pub value: serde_json::Value,
    pub value_type: Option<String>,
    pub tags: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub hit_count: i64,
    pub last_hit_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct CacheTag {
    pub id: Uuid,
    pub site_id: Option<Uuid>,
    pub tag: String,
    pub version: i64,
    pub invalidated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
