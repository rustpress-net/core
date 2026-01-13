//! Post service for handling post-related business logic.

use chrono::{DateTime, Utc};
use rustpress_core::error::{Error, Result};
use rustpress_core::service::{ListParams, SortOrder};
use rustpress_database::repository::posts::{PostRepository, PostRow};
use rustpress_admin::functions::EventDispatcher;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Post status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PostStatus {
    Draft,
    Pending,
    Published,
    Private,
    Trash,
}

impl Default for PostStatus {
    fn default() -> Self {
        Self::Draft
    }
}

impl std::fmt::Display for PostStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Pending => write!(f, "pending"),
            Self::Published => write!(f, "published"),
            Self::Private => write!(f, "private"),
            Self::Trash => write!(f, "trash"),
        }
    }
}

impl std::str::FromStr for PostStatus {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(Self::Draft),
            "pending" => Ok(Self::Pending),
            "published" => Ok(Self::Published),
            "private" => Ok(Self::Private),
            "trash" => Ok(Self::Trash),
            _ => Err(format!("Invalid post status: {}", s)),
        }
    }
}

/// Post author info for API response
#[derive(Debug, Clone, Serialize)]
pub struct PostAuthorResponse {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

/// Post response for API
#[derive(Debug, Clone, Serialize)]
pub struct PostResponse {
    pub id: Uuid,
    pub author_id: Uuid,
    pub author: Option<PostAuthorResponse>,
    pub title: String,
    pub slug: String,
    pub excerpt: Option<String>,
    pub content: Option<String>,
    pub content_format: Option<String>,
    pub status: String,
    pub visibility: Option<String>,
    pub featured_image_id: Option<Uuid>,
    pub featured_image_url: Option<String>,
    pub comment_status: Option<String>,
    pub ping_status: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub categories: Vec<TermResponse>,
    pub tags: Vec<TermResponse>,
}

/// Term response (category/tag)
#[derive(Debug, Clone, Serialize)]
pub struct TermResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

/// Paginated posts response
#[derive(Debug, Clone, Serialize)]
pub struct PostsListResponse {
    pub posts: Vec<PostResponse>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

/// Create post request
#[derive(Debug, Clone, Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub slug: Option<String>,
    pub excerpt: Option<String>,
    pub content: Option<String>,
    pub content_format: Option<String>,
    pub status: Option<String>,
    pub visibility: Option<String>,
    pub password: Option<String>,
    pub featured_image_id: Option<Uuid>,
    pub comment_status: Option<String>,
    pub ping_status: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub category_ids: Option<Vec<Uuid>>,
    pub tag_ids: Option<Vec<Uuid>>,
}

/// Update post request
#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePostRequest {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub excerpt: Option<String>,
    pub content: Option<String>,
    pub content_format: Option<String>,
    pub status: Option<String>,
    pub visibility: Option<String>,
    pub password: Option<String>,
    pub featured_image_id: Option<Uuid>,
    pub comment_status: Option<String>,
    pub ping_status: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub category_ids: Option<Vec<Uuid>>,
    pub tag_ids: Option<Vec<Uuid>>,
}

/// Post list query parameters
#[derive(Debug, Clone, Deserialize, Default)]
pub struct PostListParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub status: Option<String>,
    pub author_id: Option<Uuid>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

impl From<PostRow> for PostResponse {
    fn from(row: PostRow) -> Self {
        Self {
            id: row.id,
            author_id: row.author_id,
            author: None, // Will be populated separately
            title: row.title,
            slug: row.slug,
            excerpt: row.excerpt,
            content: row.content,
            content_format: Some("html".to_string()), // Default format
            status: row.status,
            visibility: Some(row.visibility),
            featured_image_id: row.featured_image_id,
            featured_image_url: None, // Will be populated separately
            comment_status: Some(row.comment_status),
            ping_status: Some(row.ping_status),
            published_at: row.published_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
            categories: vec![],
            tags: vec![],
        }
    }
}

/// Post service for handling post operations
#[derive(Clone)]
pub struct PostService {
    pool: PgPool,
    site_id: Option<Uuid>,
    dispatcher: EventDispatcher,
}

impl PostService {
    /// Create a new post service
    pub fn new(pool: PgPool) -> Self {
        let dispatcher = EventDispatcher::new(pool.clone());
        Self { pool, site_id: None, dispatcher }
    }

    /// Set the site ID for multi-site support
    pub fn with_site(mut self, site_id: Uuid) -> Self {
        self.site_id = Some(site_id);
        self
    }

    /// Get the repository instance
    fn repo(&self) -> PostRepository {
        let repo = PostRepository::new(self.pool.clone());
        match self.site_id {
            Some(id) => repo.with_site(id),
            None => repo,
        }
    }

    /// Create a new post
    pub async fn create_post(
        &self,
        request: CreatePostRequest,
        author_id: Uuid,
    ) -> Result<PostResponse> {
        // Validate title
        if request.title.trim().is_empty() {
            return Err(Error::validation("Post title cannot be empty"));
        }

        // Generate slug if not provided
        let slug = request.slug.clone().unwrap_or_else(|| self.generate_slug(&request.title));

        // Check if slug is unique
        if let Some(_) = self.repo().find_by_slug(&slug).await? {
            return Err(Error::validation("A post with this slug already exists"));
        }

        let status = request.status.clone().unwrap_or_else(|| "draft".to_string());
        let now = Utc::now();

        // Prepare event data for hooks
        let event_data = serde_json::json!({
            "title": request.title,
            "slug": slug,
            "content": request.content,
            "excerpt": request.excerpt,
            "status": status,
            "author_id": author_id.to_string(),
            "category_ids": request.category_ids,
            "tag_ids": request.tag_ids,
        });

        // Execute BEFORE hooks - can modify data or cancel the action
        let before_result = self.dispatcher.dispatch_before("post_created", &event_data).await;

        if !before_result.proceed {
            return Err(Error::validation(
                before_result.cancel_reason.unwrap_or_else(|| "Post creation cancelled by hook".to_string())
            ));
        }

        // Use modified data if provided by hooks
        let final_data = before_result.modified_data.unwrap_or(event_data);
        let final_title = final_data.get("title").and_then(|v| v.as_str()).unwrap_or(&request.title).to_string();
        let final_slug = final_data.get("slug").and_then(|v| v.as_str()).unwrap_or(&slug).to_string();
        let final_content = final_data.get("content").and_then(|v| v.as_str()).map(|s| s.to_string()).or(request.content.clone());
        let final_excerpt = final_data.get("excerpt").and_then(|v| v.as_str()).map(|s| s.to_string()).or(request.excerpt.clone());

        // Set published_at if status is published and not already set
        let published_at = if status == "published" {
            request.published_at.or(Some(now))
        } else {
            request.published_at
        };

        let post = PostRow {
            id: Uuid::now_v7(),
            site_id: self.site_id,
            post_type: "post".to_string(),
            author_id,
            title: final_title,
            slug: final_slug,
            content: final_content,
            excerpt: final_excerpt,
            status: status.clone(),
            visibility: request.visibility.unwrap_or_else(|| "public".to_string()),
            password: request.password,
            parent_id: None,
            menu_order: 0,
            template: None,
            featured_image_id: request.featured_image_id,
            comment_status: request.comment_status.unwrap_or_else(|| "open".to_string()),
            comment_count: 0,
            ping_status: request.ping_status.unwrap_or_else(|| "open".to_string()),
            meta_title: None,
            meta_description: None,
            canonical_url: None,
            published_at,
            scheduled_at: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created = self.repo().create(&post).await?;

        // Handle categories and tags
        if let Some(category_ids) = request.category_ids {
            self.set_terms(created.id, "category", &category_ids).await?;
        }
        if let Some(tag_ids) = request.tag_ids {
            self.set_terms(created.id, "post_tag", &tag_ids).await?;
        }

        let mut response = PostResponse::from(created);
        response.categories = self.get_post_terms(response.id, "category").await?;
        response.tags = self.get_post_terms(response.id, "post_tag").await?;

        // Execute AFTER hooks - for notifications, logging, etc.
        let after_event_data = serde_json::json!({
            "post_id": response.id.to_string(),
            "title": response.title,
            "slug": response.slug,
            "status": response.status,
            "author_id": response.author_id.to_string(),
            "created_at": response.created_at.to_rfc3339(),
        });
        let _ = self.dispatcher.dispatch_after("post_created", &after_event_data).await;

        // If post was published, also trigger post_published event
        if status == "published" {
            let _ = self.dispatcher.dispatch_after("post_published", &after_event_data).await;
        }

        Ok(response)
    }

    /// List posts with pagination and filtering
    pub async fn list_posts(&self, params: PostListParams) -> Result<PostsListResponse> {
        let page = params.page.unwrap_or(1).max(1);
        let per_page = params.per_page.unwrap_or(20).min(100).max(1);

        let sort_order = match params.sort_order.as_deref() {
            Some("asc") => SortOrder::Asc,
            _ => SortOrder::Desc,
        };

        let _list_params = ListParams {
            page,
            per_page,
            search: params.search.clone(),
            sort_by: params.sort_by.clone(),
            sort_order,
            ..Default::default()
        };

        // Build custom query with status and author filters
        let mut conditions = vec![];

        if let Some(site_id) = self.site_id {
            conditions.push(format!("site_id = '{}'", site_id));
        } else {
            conditions.push("site_id IS NULL".to_string());
        }

        conditions.push("deleted_at IS NULL".to_string());

        if let Some(ref status) = params.status {
            conditions.push(format!("status = '{}'", status.replace('\'', "''")));
        }

        if let Some(author_id) = params.author_id {
            conditions.push(format!("author_id = '{}'", author_id));
        }

        if let Some(ref search) = params.search {
            let escaped = search.replace('\'', "''");
            conditions.push(format!(
                "(title ILIKE '%{}%' OR content ILIKE '%{}%' OR excerpt ILIKE '%{}%')",
                escaped, escaped, escaped
            ));
        }

        let where_clause = conditions.join(" AND ");
        let order_by = params.sort_by.as_deref().unwrap_or("created_at");
        let order_dir = if sort_order == SortOrder::Desc { "DESC" } else { "ASC" };

        // Count query
        let count_query = format!(
            "SELECT COUNT(*) as count FROM posts WHERE {}",
            where_clause
        );
        let total: (i64,) = sqlx::query_as(&count_query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to count posts", e))?;

        // Data query
        let offset = (page - 1) * per_page;
        let data_query = format!(
            "SELECT {} FROM posts WHERE {} ORDER BY {} {} LIMIT {} OFFSET {}",
            PostRow::COLUMNS, where_clause, order_by, order_dir, per_page, offset
        );

        let rows: Vec<PostRow> = sqlx::query_as(&data_query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to list posts", e))?;

        let mut posts = Vec::with_capacity(rows.len());
        for row in rows {
            let mut post = PostResponse::from(row);
            post.categories = self.get_post_terms(post.id, "category").await?;
            post.tags = self.get_post_terms(post.id, "post_tag").await?;
            posts.push(post);
        }

        let total_pages = ((total.0 as f64) / (per_page as f64)).ceil() as u64;

        Ok(PostsListResponse {
            posts,
            total: total.0 as u64,
            page: page.into(),
            per_page: per_page.into(),
            total_pages,
        })
    }

    /// Get a single post by ID
    pub async fn get_post(&self, id: Uuid) -> Result<Option<PostResponse>> {
        let post = self.repo().find_by_id(id).await?;

        match post {
            Some(row) => {
                let mut response = PostResponse::from(row);
                response.categories = self.get_post_terms(response.id, "category").await?;
                response.tags = self.get_post_terms(response.id, "post_tag").await?;
                response.author = self.get_author(response.author_id).await?;
                Ok(Some(response))
            }
            None => Ok(None),
        }
    }

    /// Get a post by slug
    pub async fn get_post_by_slug(&self, slug: &str) -> Result<Option<PostResponse>> {
        let post = self.repo().find_by_slug(slug).await?;

        match post {
            Some(row) => {
                let mut response = PostResponse::from(row);
                response.categories = self.get_post_terms(response.id, "category").await?;
                response.tags = self.get_post_terms(response.id, "post_tag").await?;
                response.author = self.get_author(response.author_id).await?;
                Ok(Some(response))
            }
            None => Ok(None),
        }
    }

    /// Update a post
    pub async fn update_post(&self, id: Uuid, request: UpdatePostRequest) -> Result<PostResponse> {
        // Get existing post
        let existing = self.repo().find_by_id(id).await?
            .ok_or_else(|| Error::not_found("Post", id.to_string()))?;

        // Check slug uniqueness if changed
        if let Some(ref new_slug) = request.slug {
            if new_slug != &existing.slug {
                if let Some(_) = self.repo().find_by_slug(new_slug).await? {
                    return Err(Error::validation("A post with this slug already exists"));
                }
            }
        }

        let was_published = existing.status == "published";
        let new_status = request.status.as_ref().unwrap_or(&existing.status);
        let is_publishing = !was_published && new_status == "published";

        // Prepare event data for BEFORE hooks
        let event_data = serde_json::json!({
            "post_id": id.to_string(),
            "title": request.title.as_ref().unwrap_or(&existing.title),
            "slug": request.slug.as_ref().unwrap_or(&existing.slug),
            "content": request.content.as_ref().or(existing.content.as_ref()),
            "excerpt": request.excerpt.as_ref().or(existing.excerpt.as_ref()),
            "status": new_status,
            "old_status": existing.status.clone(),
            "author_id": existing.author_id.to_string(),
            "is_publishing": is_publishing,
        });

        // Execute BEFORE hooks - can modify data or cancel the action
        let before_result = self.dispatcher.dispatch_before("post_updated", &event_data).await;

        if !before_result.proceed {
            return Err(Error::validation(
                before_result.cancel_reason.unwrap_or_else(|| "Post update cancelled by hook".to_string())
            ));
        }

        // Use modified data if provided by hooks
        let final_data = before_result.modified_data.unwrap_or(event_data.clone());
        let final_title = final_data.get("title").and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or(request.title.clone())
            .unwrap_or_else(|| existing.title.clone());
        let final_slug = final_data.get("slug").and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or(request.slug.clone())
            .unwrap_or_else(|| existing.slug.clone());
        let final_content = final_data.get("content").and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or(request.content.clone())
            .or(existing.content.clone());
        let final_excerpt = final_data.get("excerpt").and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or(request.excerpt.clone())
            .or(existing.excerpt.clone());

        // Set published_at if publishing for the first time
        let published_at = if is_publishing && existing.published_at.is_none() {
            Some(Utc::now())
        } else {
            request.published_at.or(existing.published_at)
        };

        let updated_post = PostRow {
            id: existing.id,
            site_id: existing.site_id,
            post_type: existing.post_type,
            author_id: existing.author_id,
            title: final_title,
            slug: final_slug,
            content: final_content,
            excerpt: final_excerpt,
            status: request.status.clone().unwrap_or(existing.status),
            visibility: request.visibility.unwrap_or(existing.visibility),
            password: request.password.or(existing.password),
            parent_id: existing.parent_id,
            menu_order: existing.menu_order,
            template: existing.template,
            featured_image_id: request.featured_image_id.or(existing.featured_image_id),
            comment_status: request.comment_status.unwrap_or(existing.comment_status),
            comment_count: existing.comment_count,
            ping_status: request.ping_status.unwrap_or(existing.ping_status),
            meta_title: existing.meta_title,
            meta_description: existing.meta_description,
            canonical_url: existing.canonical_url,
            published_at,
            scheduled_at: existing.scheduled_at,
            created_at: existing.created_at,
            updated_at: Utc::now(),
            deleted_at: existing.deleted_at,
        };

        let updated = self.repo().update(&updated_post).await?;

        // Handle categories and tags
        if let Some(category_ids) = request.category_ids {
            self.set_terms(id, "category", &category_ids).await?;
        }
        if let Some(tag_ids) = request.tag_ids {
            self.set_terms(id, "post_tag", &tag_ids).await?;
        }

        let mut response = PostResponse::from(updated);
        response.categories = self.get_post_terms(response.id, "category").await?;
        response.tags = self.get_post_terms(response.id, "post_tag").await?;

        // Execute AFTER hooks
        let after_event_data = serde_json::json!({
            "post_id": response.id.to_string(),
            "title": response.title,
            "slug": response.slug,
            "status": response.status,
            "author_id": response.author_id.to_string(),
            "updated_at": response.updated_at.to_rfc3339(),
        });
        let _ = self.dispatcher.dispatch_after("post_updated", &after_event_data).await;

        // If post was just published, also trigger post_published event
        if is_publishing {
            let _ = self.dispatcher.dispatch_after("post_published", &after_event_data).await;
        }

        Ok(response)
    }

    /// Delete a post (soft delete)
    pub async fn delete_post(&self, id: Uuid) -> Result<bool> {
        // Get post details for hooks
        let existing = self.repo().find_by_id(id).await?
            .ok_or_else(|| Error::not_found("Post", id.to_string()))?;

        // Prepare event data for BEFORE hooks
        let event_data = serde_json::json!({
            "post_id": id.to_string(),
            "title": existing.title,
            "slug": existing.slug,
            "status": existing.status,
            "author_id": existing.author_id.to_string(),
        });

        // Execute BEFORE hooks - can cancel the deletion
        let before_result = self.dispatcher.dispatch_before("post_deleted", &event_data).await;

        if !before_result.proceed {
            return Err(Error::validation(
                before_result.cancel_reason.unwrap_or_else(|| "Post deletion cancelled by hook".to_string())
            ));
        }

        self.repo().soft_delete(id).await?;

        // Execute AFTER hooks
        let after_event_data = serde_json::json!({
            "post_id": id.to_string(),
            "title": existing.title,
            "slug": existing.slug,
            "deleted_at": Utc::now().to_rfc3339(),
        });
        let _ = self.dispatcher.dispatch_after("post_deleted", &after_event_data).await;
        let _ = self.dispatcher.dispatch_after("post_trashed", &after_event_data).await;

        Ok(true)
    }

    /// Restore a deleted post
    pub async fn restore_post(&self, id: Uuid) -> Result<PostResponse> {
        // Execute BEFORE hooks
        let event_data = serde_json::json!({
            "post_id": id.to_string(),
        });
        let before_result = self.dispatcher.dispatch_before("post_restored", &event_data).await;

        if !before_result.proceed {
            return Err(Error::validation(
                before_result.cancel_reason.unwrap_or_else(|| "Post restoration cancelled by hook".to_string())
            ));
        }

        self.repo().restore(id).await?;
        let post = self.get_post(id).await?
            .ok_or_else(|| Error::not_found("Post", id.to_string()))?;

        // Execute AFTER hooks
        let after_event_data = serde_json::json!({
            "post_id": post.id.to_string(),
            "title": post.title,
            "slug": post.slug,
            "restored_at": Utc::now().to_rfc3339(),
        });
        let _ = self.dispatcher.dispatch_after("post_restored", &after_event_data).await;

        Ok(post)
    }

    /// Publish a post
    pub async fn publish_post(&self, id: Uuid) -> Result<PostResponse> {
        let existing = self.repo().find_by_id(id).await?
            .ok_or_else(|| Error::not_found("Post", id.to_string()))?;

        // Prepare event data for BEFORE hooks
        let event_data = serde_json::json!({
            "post_id": id.to_string(),
            "title": existing.title,
            "slug": existing.slug,
            "old_status": existing.status,
            "author_id": existing.author_id.to_string(),
        });

        // Execute BEFORE hooks - can cancel publishing
        let before_result = self.dispatcher.dispatch_before("post_published", &event_data).await;

        if !before_result.proceed {
            return Err(Error::validation(
                before_result.cancel_reason.unwrap_or_else(|| "Post publishing cancelled by hook".to_string())
            ));
        }

        let old_status = existing.status.clone();
        let published_at = existing.published_at.or(Some(Utc::now()));

        let updated_post = PostRow {
            status: "published".to_string(),
            published_at,
            updated_at: Utc::now(),
            ..existing
        };

        let updated = self.repo().update(&updated_post).await?;

        let mut response = PostResponse::from(updated);
        response.categories = self.get_post_terms(response.id, "category").await?;
        response.tags = self.get_post_terms(response.id, "post_tag").await?;

        // Execute AFTER hooks
        let after_event_data = serde_json::json!({
            "post_id": response.id.to_string(),
            "title": response.title,
            "slug": response.slug,
            "published_at": response.published_at.map(|d| d.to_rfc3339()),
            "author_id": response.author_id.to_string(),
        });
        let _ = self.dispatcher.dispatch_after("post_published", &after_event_data).await;
        let _ = self.dispatcher.dispatch_after("post_status_changed", &serde_json::json!({
            "post_id": response.id.to_string(),
            "old_status": old_status,
            "new_status": "published",
        })).await;

        Ok(response)
    }

    /// Unpublish a post (revert to draft)
    pub async fn unpublish_post(&self, id: Uuid) -> Result<PostResponse> {
        let existing = self.repo().find_by_id(id).await?
            .ok_or_else(|| Error::not_found("Post", id.to_string()))?;

        // Prepare event data for BEFORE hooks
        let event_data = serde_json::json!({
            "post_id": id.to_string(),
            "title": existing.title,
            "slug": existing.slug,
            "old_status": existing.status,
            "author_id": existing.author_id.to_string(),
        });

        // Execute BEFORE hooks - can cancel unpublishing
        let before_result = self.dispatcher.dispatch_before("post_unpublished", &event_data).await;

        if !before_result.proceed {
            return Err(Error::validation(
                before_result.cancel_reason.unwrap_or_else(|| "Post unpublishing cancelled by hook".to_string())
            ));
        }

        let old_status = existing.status.clone();
        let updated_post = PostRow {
            status: "draft".to_string(),
            updated_at: Utc::now(),
            ..existing
        };

        let updated = self.repo().update(&updated_post).await?;

        let mut response = PostResponse::from(updated);
        response.categories = self.get_post_terms(response.id, "category").await?;
        response.tags = self.get_post_terms(response.id, "post_tag").await?;

        // Execute AFTER hooks
        let after_event_data = serde_json::json!({
            "post_id": response.id.to_string(),
            "title": response.title,
            "slug": response.slug,
            "author_id": response.author_id.to_string(),
        });
        let _ = self.dispatcher.dispatch_after("post_unpublished", &after_event_data).await;
        let _ = self.dispatcher.dispatch_after("post_status_changed", &serde_json::json!({
            "post_id": response.id.to_string(),
            "old_status": old_status,
            "new_status": "draft",
        })).await;

        Ok(response)
    }

    /// Get counts by status
    pub async fn get_counts(&self) -> Result<std::collections::HashMap<String, i64>> {
        let site_condition = match self.site_id {
            Some(id) => format!("site_id = '{}'", id),
            None => "site_id IS NULL".to_string(),
        };

        let query = format!(
            r#"
            SELECT status, COUNT(*) as count
            FROM posts
            WHERE {} AND deleted_at IS NULL
            GROUP BY status
            "#,
            site_condition
        );

        let rows: Vec<(String, i64)> = sqlx::query_as(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to count posts by status", e))?;

        Ok(rows.into_iter().collect())
    }

    // =====================
    // Helper methods
    // =====================

    /// Generate a URL-friendly slug from a title
    fn generate_slug(&self, title: &str) -> String {
        generate_slug_impl(title)
    }
}

/// Standalone slug generation (for testing without database)
fn generate_slug_impl(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

impl PostService {

    /// Get terms (categories/tags) for a post
    async fn get_post_terms(&self, post_id: Uuid, taxonomy: &str) -> Result<Vec<TermResponse>> {
        let query = r#"
            SELECT t.id, t.name, t.slug
            FROM terms t
            JOIN term_relationships tr ON t.id = tr.term_id
            JOIN taxonomies tax ON t.taxonomy_id = tax.id
            WHERE tr.object_id = $1 AND tr.object_type = 'post' AND tax.slug = $2
            ORDER BY t.name
        "#;

        // Use Result to catch any errors gracefully - return empty vec on error
        let rows: Vec<(Uuid, String, String)> = match sqlx::query_as(query)
            .bind(post_id)
            .bind(taxonomy)
            .fetch_all(&self.pool)
            .await
        {
            Ok(r) => r,
            Err(e) => {
                tracing::debug!("get_post_terms error (returning empty): {}", e);
                return Ok(vec![]);
            }
        };

        Ok(rows.into_iter().map(|(id, name, slug)| TermResponse { id, name, slug }).collect())
    }

    /// Set terms for a post (replaces existing)
    async fn set_terms(&self, post_id: Uuid, taxonomy: &str, term_ids: &[Uuid]) -> Result<()> {
        // First, remove existing term relationships for this taxonomy
        let delete_query = r#"
            DELETE FROM term_relationships
            WHERE object_id = $1 AND object_type = 'post'
            AND term_id IN (
                SELECT t.id FROM terms t
                JOIN taxonomies tax ON t.taxonomy_id = tax.id
                WHERE tax.slug = $2
            )
        "#;

        sqlx::query(delete_query)
            .bind(post_id)
            .bind(taxonomy)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to remove term relationships", e))?;

        // Insert new relationships
        for (order, term_id) in term_ids.iter().enumerate() {
            let insert_query = r#"
                INSERT INTO term_relationships (object_id, object_type, term_id, term_order)
                VALUES ($1, 'post', $2, $3)
                ON CONFLICT (object_id, object_type, term_id) DO NOTHING
            "#;

            sqlx::query(insert_query)
                .bind(post_id)
                .bind(term_id)
                .bind(order as i32)
                .execute(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to add term relationship", e))?;
        }

        // Update term counts
        for term_id in term_ids {
            let update_count = r#"
                UPDATE terms SET count = (
                    SELECT COUNT(*) FROM term_relationships WHERE term_id = $1
                ) WHERE id = $1
            "#;

            let _ = sqlx::query(update_count)
                .bind(term_id)
                .execute(&self.pool)
                .await;
        }

        Ok(())
    }

    /// Get author info
    async fn get_author(&self, author_id: Uuid) -> Result<Option<PostAuthorResponse>> {
        let query = "SELECT id, display_name, email, avatar_url FROM users WHERE id = $1 AND deleted_at IS NULL";

        let row: Option<(Uuid, Option<String>, String, Option<String>)> = sqlx::query_as(query)
            .bind(author_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to get author", e))?;

        Ok(row.map(|(id, display_name, email, avatar_url)| PostAuthorResponse {
            id,
            name: display_name.unwrap_or_else(|| "Unknown".to_string()),
            email: Some(email),
            avatar_url,
        }))
    }
}

impl Default for PostService {
    fn default() -> Self {
        panic!("PostService requires a database pool")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_slug() {
        assert_eq!(generate_slug_impl("Hello World"), "hello-world");
        assert_eq!(generate_slug_impl("  Multiple   Spaces  "), "multiple-spaces");
        assert_eq!(generate_slug_impl("Special!@#$%Characters"), "special-characters");
    }

    #[test]
    fn test_post_status_display() {
        assert_eq!(PostStatus::Draft.to_string(), "draft");
        assert_eq!(PostStatus::Published.to_string(), "published");
    }

    #[test]
    fn test_post_status_from_str() {
        assert_eq!("draft".parse::<PostStatus>().unwrap(), PostStatus::Draft);
        assert_eq!("published".parse::<PostStatus>().unwrap(), PostStatus::Published);
        assert!("invalid".parse::<PostStatus>().is_err());
    }
}
