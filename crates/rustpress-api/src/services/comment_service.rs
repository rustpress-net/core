//! Comment service for handling comment-related business logic.

use chrono::{DateTime, Utc};
use rustpress_core::error::{Error, Result};
use rustpress_database::repository::comments::{
    CommentRow, CommentStatus, CommentWithAuthor, CommentsRepository,
    CreateComment, UpdateComment, CommentListParams,
};
use rustpress_database::repository::options::OptionsRepository;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

/// Comment author info for API response
#[derive(Debug, Clone, Serialize)]
pub struct CommentAuthorResponse {
    pub id: Option<Uuid>,
    pub name: String,
    pub email: Option<String>,
    pub url: Option<String>,
    pub avatar_url: Option<String>,
    pub is_registered: bool,
}

/// Comment response for API
#[derive(Debug, Clone, Serialize)]
pub struct CommentResponse {
    pub id: Uuid,
    pub post_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub depth: i32,
    pub content: String,
    pub content_html: Option<String>,
    pub status: CommentStatus,
    pub author: CommentAuthorResponse,
    pub likes_count: i32,
    pub replies_count: i32,
    pub is_edited: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Comment with nested replies
#[derive(Debug, Clone, Serialize)]
pub struct CommentTreeNode {
    #[serde(flatten)]
    pub comment: CommentResponse,
    pub replies: Vec<CommentTreeNode>,
}

/// Paginated comments response
#[derive(Debug, Clone, Serialize)]
pub struct CommentsListResponse {
    pub comments: Vec<CommentResponse>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
    pub counts: HashMap<String, i64>,
}

/// Create comment request
#[derive(Debug, Clone, Deserialize)]
pub struct CreateCommentRequest {
    pub post_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub content: String,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub author_url: Option<String>,
}

/// Update comment request
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateCommentRequest {
    pub content: Option<String>,
    pub status: Option<CommentStatus>,
}

/// Batch moderation request
#[derive(Debug, Clone, Deserialize)]
pub struct BatchModerateRequest {
    pub comment_ids: Vec<Uuid>,
    pub status: CommentStatus,
}

impl From<CommentRow> for CommentResponse {
    fn from(row: CommentRow) -> Self {
        let author = CommentAuthorResponse {
            id: row.user_id,
            name: row.author_name.unwrap_or_else(|| "Anonymous".to_string()),
            email: row.author_email,
            url: row.author_url,
            avatar_url: None, // Would need to be joined from users table
            is_registered: row.user_id.is_some(),
        };

        Self {
            id: row.id,
            post_id: row.post_id,
            parent_id: row.parent_id,
            depth: row.depth,
            content: row.content,
            content_html: row.content_html,
            status: row.status,
            author,
            likes_count: row.likes_count,
            replies_count: row.replies_count,
            is_edited: row.is_edited,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<CommentWithAuthor> for CommentResponse {
    fn from(row: CommentWithAuthor) -> Self {
        let author = CommentAuthorResponse {
            id: row.user_id,
            name: row.user_display_name
                .or(row.author_name)
                .unwrap_or_else(|| "Anonymous".to_string()),
            email: row.author_email,
            url: row.author_url,
            avatar_url: row.user_avatar_url,
            is_registered: row.user_id.is_some(),
        };

        Self {
            id: row.id,
            post_id: row.post_id,
            parent_id: row.parent_id,
            depth: row.depth,
            content: row.content,
            content_html: row.content_html,
            status: row.status,
            author,
            likes_count: row.likes_count,
            replies_count: row.replies_count,
            is_edited: false, // Not included in CommentWithAuthor
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// Comment service for handling comment operations
#[derive(Clone)]
pub struct CommentService {
    pool: PgPool,
    site_id: Option<Uuid>,
}

impl CommentService {
    /// Create a new comment service
    pub fn new(pool: PgPool) -> Self {
        Self { pool, site_id: None }
    }

    /// Set the site ID for multi-site support
    pub fn with_site(mut self, site_id: Uuid) -> Self {
        self.site_id = Some(site_id);
        self
    }

    /// Get the repository instance
    fn repo(&self) -> CommentsRepository {
        let repo = CommentsRepository::new(self.pool.clone());
        match self.site_id {
            Some(id) => repo.with_site(id),
            None => repo,
        }
    }

    /// Get an options repository instance
    fn options_repo(&self) -> OptionsRepository {
        let repo = OptionsRepository::new(self.pool.clone());
        match self.site_id {
            Some(id) => repo.with_site(id),
            None => repo,
        }
    }

    /// Get a boolean option value with default
    async fn get_bool_option(&self, name: &str, default: bool) -> bool {
        match self.options_repo().get(name).await {
            Ok(Some(value)) => {
                // Handle different formats: "true"/"false" string or boolean
                if let Some(b) = value.as_bool() {
                    b
                } else if let Some(s) = value.as_str() {
                    s == "true" || s == "1"
                } else {
                    default
                }
            }
            _ => default,
        }
    }

    /// Determine the initial comment status based on settings and author history
    async fn determine_initial_status(
        &self,
        spam_score: f64,
        user_id: Option<Uuid>,
        author_email: &Option<String>,
    ) -> CommentStatus {
        // High spam score - mark as spam
        if spam_score > 0.7 {
            return CommentStatus::Spam;
        }

        // Medium spam score - hold for moderation
        if spam_score > 0.4 {
            return CommentStatus::Pending;
        }

        // Check if manual moderation is required for all comments
        let require_moderation = self.get_bool_option("comment_moderation", false).await;
        if require_moderation {
            return CommentStatus::Pending;
        }

        // Registered users: check if they have previously approved comments
        if let Some(uid) = user_id {
            if self.repo().has_approved_comment_by_user(uid).await.unwrap_or(false) {
                return CommentStatus::Approved;
            }
        }

        // Guest users: check if their email has previously approved comments
        let auto_approve_previous = self.get_bool_option("comment_previously_approved", true).await;
        if auto_approve_previous {
            if let Some(email) = author_email {
                if !email.trim().is_empty() {
                    if self.repo().has_approved_comment_by_email(email).await.unwrap_or(false) {
                        return CommentStatus::Approved;
                    }
                }
            }
        }

        // Default: hold for moderation
        CommentStatus::Pending
    }

    /// Submit a new comment
    pub async fn submit_comment(
        &self,
        request: CreateCommentRequest,
        user_id: Option<Uuid>,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<CommentResponse> {
        // Validate content
        if request.content.trim().is_empty() {
            return Err(Error::validation("Comment content cannot be empty"));
        }

        // For guest comments, require name and email
        if user_id.is_none() {
            if request.author_name.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true) {
                return Err(Error::validation("Name is required for guest comments"));
            }
            if request.author_email.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true) {
                return Err(Error::validation("Email is required for guest comments"));
            }
        }

        // Check spam (simple heuristics for now)
        let spam_score = self.calculate_spam_score(&request, &ip, &user_agent);

        // Determine initial status based on settings and author history
        let initial_status = self.determine_initial_status(
            spam_score,
            user_id,
            &request.author_email,
        ).await;

        // Render content to HTML (basic for now)
        let content_html = Some(self.render_content(&request.content));

        let create = CreateComment {
            post_id: request.post_id,
            parent_id: request.parent_id,
            user_id,
            author_name: request.author_name,
            author_email: request.author_email,
            author_url: request.author_url,
            author_ip: ip,
            content: request.content,
            content_html,
            user_agent,
            status: initial_status,
        };

        let comment = self.repo().create(&create).await?;

        // Update spam score if detected
        if spam_score > 0.0 {
            let reasons = serde_json::json!({
                "score": spam_score,
                "auto_detected": true
            });
            let _ = self.repo().update_spam_score(comment.id, spam_score, Some(reasons)).await;
        }

        Ok(CommentResponse::from(comment))
    }

    /// List comments with filtering and pagination
    pub async fn list_comments(
        &self,
        page: u64,
        per_page: u64,
        post_id: Option<Uuid>,
        status: Option<CommentStatus>,
        search: Option<String>,
    ) -> Result<CommentsListResponse> {
        let params = CommentListParams {
            page: page.max(1),
            per_page: per_page.min(100).max(1),
            post_id,
            status,
            search,
            order_desc: true,
            ..Default::default()
        };

        let (comments, total) = self.repo().list(&params).await?;
        let counts = self.repo().count_by_status().await?;

        let total_pages = (total as f64 / params.per_page as f64).ceil() as u64;

        Ok(CommentsListResponse {
            comments: comments.into_iter().map(CommentResponse::from).collect(),
            total,
            page: params.page,
            per_page: params.per_page,
            total_pages,
            counts,
        })
    }

    /// Get a single comment by ID
    pub async fn get_comment(&self, id: Uuid) -> Result<Option<CommentResponse>> {
        let comment = self.repo().find_by_id(id).await?;
        Ok(comment.map(CommentResponse::from))
    }

    /// Update a comment
    pub async fn update_comment(&self, id: Uuid, request: UpdateCommentRequest) -> Result<CommentResponse> {
        let updates = UpdateComment {
            content: request.content.clone(),
            content_html: request.content.map(|c| self.render_content(&c)),
            status: request.status,
        };

        let comment = self.repo().update(id, &updates).await?;
        Ok(CommentResponse::from(comment))
    }

    /// Delete a comment (soft delete)
    pub async fn delete_comment(&self, id: Uuid) -> Result<bool> {
        self.repo().soft_delete(id).await
    }

    /// Approve a comment
    pub async fn approve_comment(&self, id: Uuid, moderator_id: Uuid) -> Result<CommentResponse> {
        let comment = self.repo().update_status(id, CommentStatus::Approved, moderator_id, None).await?;
        Ok(CommentResponse::from(comment))
    }

    /// Mark a comment as spam
    pub async fn mark_as_spam(&self, id: Uuid, moderator_id: Uuid) -> Result<CommentResponse> {
        let comment = self.repo().update_status(id, CommentStatus::Spam, moderator_id, None).await?;
        Ok(CommentResponse::from(comment))
    }

    /// Move a comment to trash
    pub async fn trash_comment(&self, id: Uuid, moderator_id: Uuid) -> Result<CommentResponse> {
        let comment = self.repo().update_status(id, CommentStatus::Trash, moderator_id, None).await?;
        Ok(CommentResponse::from(comment))
    }

    /// Batch moderate comments
    pub async fn batch_moderate(
        &self,
        request: BatchModerateRequest,
        moderator_id: Uuid,
    ) -> Result<u64> {
        self.repo()
            .batch_update_status(&request.comment_ids, request.status, moderator_id)
            .await
    }

    /// Toggle like on a comment
    pub async fn toggle_like(&self, comment_id: Uuid, user_id: Uuid) -> Result<bool> {
        let has_liked = self.repo().has_liked(comment_id, user_id).await?;

        if has_liked {
            self.repo().remove_like(comment_id, user_id).await?;
            Ok(false)
        } else {
            self.repo().add_like(comment_id, user_id).await?;
            Ok(true)
        }
    }

    /// Get comments for a post as a tree
    pub async fn get_comment_tree(&self, post_id: Uuid) -> Result<Vec<CommentTreeNode>> {
        let comments = self.repo()
            .list_by_post_with_authors(post_id, Some(CommentStatus::Approved))
            .await?;

        Ok(self.build_tree(comments))
    }

    /// Get comment counts by status
    pub async fn get_counts(&self) -> Result<HashMap<String, i64>> {
        self.repo().count_by_status().await
    }

    // =====================
    // Helper methods
    // =====================

    /// Build a nested tree from flat comments
    fn build_tree(&self, comments: Vec<CommentWithAuthor>) -> Vec<CommentTreeNode> {
        let mut nodes: HashMap<Uuid, CommentTreeNode> = HashMap::new();
        let mut root_ids: Vec<Uuid> = Vec::new();

        // First pass: create all nodes
        for comment in comments {
            let id = comment.id;
            let parent_id = comment.parent_id;

            let node = CommentTreeNode {
                comment: CommentResponse::from(comment),
                replies: Vec::new(),
            };

            nodes.insert(id, node);

            if parent_id.is_none() {
                root_ids.push(id);
            }
        }

        // Second pass: build tree (iterative to avoid borrowing issues)
        let mut parent_child_pairs: Vec<(Uuid, Uuid)> = Vec::new();
        for (id, node) in &nodes {
            if let Some(parent_id) = node.comment.parent_id {
                parent_child_pairs.push((parent_id, *id));
            }
        }

        for (parent_id, child_id) in parent_child_pairs {
            if let Some(child) = nodes.remove(&child_id) {
                if let Some(parent) = nodes.get_mut(&parent_id) {
                    parent.replies.push(child);
                }
            }
        }

        // Extract root nodes
        root_ids
            .into_iter()
            .filter_map(|id| nodes.remove(&id))
            .collect()
    }

    /// Calculate spam score (0.0 - 1.0)
    fn calculate_spam_score(
        &self,
        request: &CreateCommentRequest,
        _ip: &Option<String>,
        _user_agent: &Option<String>,
    ) -> f64 {
        let mut score = 0.0;
        let content = &request.content;

        // Check for common spam patterns
        let spam_keywords = ["viagra", "casino", "lottery", "click here", "buy now", "free money"];
        let content_lower = content.to_lowercase();
        for keyword in &spam_keywords {
            if content_lower.contains(keyword) {
                score += 0.3;
            }
        }

        // Check for excessive links
        let link_count = content.matches("http").count();
        if link_count > 3 {
            score += 0.2 * (link_count - 3) as f64;
        }

        // Check for all caps
        let caps_ratio = content.chars().filter(|c| c.is_uppercase()).count() as f64
            / content.chars().filter(|c| c.is_alphabetic()).count().max(1) as f64;
        if caps_ratio > 0.7 {
            score += 0.2;
        }

        // Very short or very long comments
        let len = content.len();
        if len < 10 || len > 5000 {
            score += 0.1;
        }

        score.min(1.0)
    }

    /// Render markdown content to HTML
    /// Supports limited markdown for comments: bold, italic, links, code, lists
    fn render_content(&self, content: &str) -> String {
        use pulldown_cmark::{html, Event, Options, Parser, Tag};

        // Enable common markdown extensions (but not raw HTML for security)
        let options = Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES;

        // Parse markdown
        let parser = Parser::new_ext(content, options);

        // Filter out potentially dangerous elements
        let parser = parser.filter_map(|event| {
            match event {
                // Block images in comments (potential abuse)
                Event::Start(Tag::Image(..)) => None,
                Event::End(Tag::Image(..)) => None,
                // Block raw HTML
                Event::Html(html) => {
                    // Escape any HTML tags
                    Some(Event::Text(html.replace('<', "&lt;").replace('>', "&gt;").into()))
                }
                // Pass through other events
                other => Some(other),
            }
        });

        // Render to HTML
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        // Post-process to add nofollow to links for SEO safety
        html_output = html_output.replace("<a href=", "<a rel=\"nofollow ugc\" href=");

        html_output
    }
}

impl Default for CommentService {
    fn default() -> Self {
        panic!("CommentService requires a database pool")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spam_score_clean() {
        let service = CommentService {
            pool: unsafe { std::mem::zeroed() }, // Just for test
            site_id: None,
        };

        let request = CreateCommentRequest {
            post_id: Uuid::new_v4(),
            parent_id: None,
            content: "This is a helpful comment.".to_string(),
            author_name: Some("John".to_string()),
            author_email: Some("john@example.com".to_string()),
            author_url: None,
        };

        let score = service.calculate_spam_score(&request, &None, &None);
        assert!(score < 0.3);
    }

    #[test]
    fn test_spam_score_spammy() {
        let service = CommentService {
            pool: unsafe { std::mem::zeroed() },
            site_id: None,
        };

        let request = CreateCommentRequest {
            post_id: Uuid::new_v4(),
            parent_id: None,
            content: "BUY VIAGRA NOW! Click here http://spam.com http://more.spam http://even.more http://lots.of.links".to_string(),
            author_name: Some("Spammer".to_string()),
            author_email: Some("spam@spam.com".to_string()),
            author_url: None,
        };

        let score = service.calculate_spam_score(&request, &None, &None);
        assert!(score > 0.5);
    }
}
