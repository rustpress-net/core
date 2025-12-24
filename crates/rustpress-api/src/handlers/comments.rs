//! Comment handlers.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Comment status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommentStatus {
    Pending,
    Approved,
    Spam,
    Trash,
}

/// Create comment request
#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    pub post_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub content: String,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub author_url: Option<String>,
}

/// Update comment request
#[derive(Debug, Deserialize)]
pub struct UpdateCommentRequest {
    pub content: Option<String>,
    pub status: Option<CommentStatus>,
}

/// Comment response
#[derive(Debug, Serialize)]
pub struct CommentResponse {
    pub id: Uuid,
    pub post_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub content: String,
    pub status: CommentStatus,
    pub author: CommentAuthor,
    pub replies_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Comment author info
#[derive(Debug, Serialize)]
pub struct CommentAuthor {
    pub id: Option<Uuid>,
    pub name: String,
    pub email: Option<String>,
    pub url: Option<String>,
    pub avatar_url: Option<String>,
    pub is_registered: bool,
}

/// Comment list query parameters
#[derive(Debug, Deserialize)]
pub struct CommentListQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub post_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub status: Option<CommentStatus>,
    pub search: Option<String>,
    pub order_by: Option<String>,
}

/// Batch update comments request
#[derive(Debug, Deserialize)]
pub struct BatchUpdateCommentsRequest {
    pub comment_ids: Vec<Uuid>,
    pub status: CommentStatus,
}
