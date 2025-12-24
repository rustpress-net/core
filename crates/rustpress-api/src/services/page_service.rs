//! Page service for handling page-related business logic.

use chrono::{DateTime, Utc};
use rustpress_core::error::{Error, Result};
use rustpress_core::service::SortOrder;
use rustpress_database::models::PageRow;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Page status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PageStatus {
    Draft,
    Pending,
    Published,
    Private,
    Trash,
}

impl Default for PageStatus {
    fn default() -> Self {
        Self::Draft
    }
}

impl std::fmt::Display for PageStatus {
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

impl std::str::FromStr for PageStatus {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(Self::Draft),
            "pending" => Ok(Self::Pending),
            "published" => Ok(Self::Published),
            "private" => Ok(Self::Private),
            "trash" => Ok(Self::Trash),
            _ => Err(format!("Invalid page status: {}", s)),
        }
    }
}

/// Page author info for API response
#[derive(Debug, Clone, Serialize)]
pub struct PageAuthorResponse {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

/// Page response for API
#[derive(Debug, Clone, Serialize)]
pub struct PageResponse {
    pub id: Uuid,
    pub author_id: Uuid,
    pub author: Option<PageAuthorResponse>,
    pub parent_id: Option<Uuid>,
    pub title: String,
    pub slug: String,
    pub content: Option<String>,
    pub content_format: Option<String>,
    pub status: String,
    pub visibility: Option<String>,
    pub template: Option<String>,
    pub menu_order: Option<i32>,
    pub featured_image_id: Option<Uuid>,
    pub featured_image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub children: Vec<PageResponse>,
}

/// Paginated pages response
#[derive(Debug, Clone, Serialize)]
pub struct PagesListResponse {
    pub pages: Vec<PageResponse>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

/// Create page request
#[derive(Debug, Clone, Deserialize)]
pub struct CreatePageRequest {
    pub title: String,
    pub slug: Option<String>,
    pub content: Option<String>,
    pub content_format: Option<String>,
    pub status: Option<String>,
    pub visibility: Option<String>,
    pub password: Option<String>,
    pub parent_id: Option<Uuid>,
    pub template: Option<String>,
    pub menu_order: Option<i32>,
    pub featured_image_id: Option<Uuid>,
}

/// Update page request
#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePageRequest {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub content: Option<String>,
    pub content_format: Option<String>,
    pub status: Option<String>,
    pub visibility: Option<String>,
    pub password: Option<String>,
    pub parent_id: Option<Uuid>,
    pub template: Option<String>,
    pub menu_order: Option<i32>,
    pub featured_image_id: Option<Uuid>,
}

/// Page list query parameters
#[derive(Debug, Clone, Deserialize, Default)]
pub struct PageListParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub status: Option<String>,
    pub parent_id: Option<Uuid>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub hierarchical: Option<bool>,
}

impl From<PageRow> for PageResponse {
    fn from(row: PageRow) -> Self {
        Self {
            id: row.id,
            author_id: row.author_id,
            author: None,
            parent_id: row.parent_id,
            title: row.title,
            slug: row.slug,
            content: row.content,
            content_format: Some("html".to_string()), // Default format
            status: row.status,
            visibility: Some(row.visibility),
            template: row.template,
            menu_order: Some(row.menu_order),
            featured_image_id: row.featured_image_id,
            featured_image_url: None,
            created_at: row.created_at,
            updated_at: row.updated_at,
            children: vec![],
        }
    }
}

/// Page service for handling page operations
#[derive(Clone)]
pub struct PageService {
    pool: PgPool,
    site_id: Option<Uuid>,
}

impl PageService {
    /// Create a new page service
    pub fn new(pool: PgPool) -> Self {
        Self { pool, site_id: None }
    }

    /// Set the site ID for multi-site support
    pub fn with_site(mut self, site_id: Uuid) -> Self {
        self.site_id = Some(site_id);
        self
    }

    /// Site condition for queries
    fn site_condition(&self) -> String {
        match self.site_id {
            Some(id) => format!("site_id = '{}'", id),
            None => "site_id IS NULL".to_string(),
        }
    }

    /// Create a new page
    pub async fn create_page(
        &self,
        request: CreatePageRequest,
        author_id: Uuid,
    ) -> Result<PageResponse> {
        // Validate title
        if request.title.trim().is_empty() {
            return Err(Error::validation("Page title cannot be empty"));
        }

        // Generate slug if not provided
        let slug = request.slug.unwrap_or_else(|| self.generate_slug(&request.title));

        // Check if slug is unique
        if let Some(_) = self.find_by_slug(&slug).await? {
            return Err(Error::validation("A page with this slug already exists"));
        }

        // Validate parent exists if provided
        if let Some(parent_id) = request.parent_id {
            if self.find_by_id(parent_id).await?.is_none() {
                return Err(Error::validation("Parent page not found"));
            }
        }

        let status = request.status.unwrap_or_else(|| "draft".to_string());
        let now = Utc::now();

        let page = PageRow {
            id: Uuid::now_v7(),
            site_id: self.site_id,
            post_type: "page".to_string(),
            author_id,
            title: request.title,
            slug,
            content: request.content,
            excerpt: None,
            status,
            visibility: request.visibility.unwrap_or_else(|| "public".to_string()),
            password: request.password,
            parent_id: request.parent_id,
            menu_order: request.menu_order.unwrap_or(0),
            template: request.template,
            featured_image_id: request.featured_image_id,
            comment_status: "closed".to_string(),
            comment_count: 0,
            ping_status: "closed".to_string(),
            meta_title: None,
            meta_description: None,
            canonical_url: None,
            published_at: None,
            scheduled_at: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created = self.create(&page).await?;
        Ok(PageResponse::from(created))
    }

    /// List pages with pagination and filtering
    pub async fn list_pages(&self, params: PageListParams) -> Result<PagesListResponse> {
        let page = params.page.unwrap_or(1).max(1);
        let per_page = params.per_page.unwrap_or(20).min(100).max(1);

        let sort_order = match params.sort_order.as_deref() {
            Some("asc") => SortOrder::Asc,
            _ => SortOrder::Desc,
        };

        let mut conditions = vec![self.site_condition()];
        conditions.push("post_type = 'page'".to_string());
        conditions.push("deleted_at IS NULL".to_string());

        if let Some(ref status) = params.status {
            conditions.push(format!("status = '{}'", status.replace('\'', "''")));
        }

        if let Some(parent_id) = params.parent_id {
            conditions.push(format!("parent_id = '{}'", parent_id));
        }

        if let Some(ref search) = params.search {
            let escaped = search.replace('\'', "''");
            conditions.push(format!(
                "(title ILIKE '%{}%' OR content ILIKE '%{}%')",
                escaped, escaped
            ));
        }

        let where_clause = conditions.join(" AND ");
        let order_by = params.sort_by.as_deref().unwrap_or("menu_order");
        let order_dir = if sort_order == SortOrder::Desc { "DESC" } else { "ASC" };

        // Count query
        let count_query = format!(
            "SELECT COUNT(*) as count FROM posts WHERE {}",
            where_clause
        );
        let total: (i64,) = sqlx::query_as(&count_query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to count pages", e))?;

        // Data query
        let offset = (page - 1) * per_page;
        let data_query = format!(
            "SELECT {} FROM posts WHERE {} ORDER BY {} {} LIMIT {} OFFSET {}",
            PageRow::COLUMNS, where_clause, order_by, order_dir, per_page, offset
        );

        let rows: Vec<PageRow> = sqlx::query_as(&data_query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to list pages", e))?;

        let pages: Vec<PageResponse> = rows.into_iter().map(PageResponse::from).collect();

        // Optionally build hierarchy
        let pages = if params.hierarchical.unwrap_or(false) {
            self.build_hierarchy(pages)
        } else {
            pages
        };

        let total_pages = ((total.0 as f64) / (per_page as f64)).ceil() as u64;

        Ok(PagesListResponse {
            pages,
            total: total.0 as u64,
            page: page.into(),
            per_page: per_page.into(),
            total_pages,
        })
    }

    /// Get a single page by ID
    pub async fn get_page(&self, id: Uuid) -> Result<Option<PageResponse>> {
        let page = self.find_by_id(id).await?;

        match page {
            Some(row) => {
                let mut response = PageResponse::from(row);
                response.author = self.get_author(response.author_id).await?;
                Ok(Some(response))
            }
            None => Ok(None),
        }
    }

    /// Get a page by slug
    pub async fn get_page_by_slug(&self, slug: &str) -> Result<Option<PageResponse>> {
        let page = self.find_by_slug(slug).await?;

        match page {
            Some(row) => {
                let mut response = PageResponse::from(row);
                response.author = self.get_author(response.author_id).await?;
                Ok(Some(response))
            }
            None => Ok(None),
        }
    }

    /// Update a page
    pub async fn update_page(&self, id: Uuid, request: UpdatePageRequest) -> Result<PageResponse> {
        let existing = self.find_by_id(id).await?
            .ok_or_else(|| Error::not_found("Page", id.to_string()))?;

        // Check slug uniqueness if changed
        if let Some(ref new_slug) = request.slug {
            if new_slug != &existing.slug {
                if let Some(_) = self.find_by_slug(new_slug).await? {
                    return Err(Error::validation("A page with this slug already exists"));
                }
            }
        }

        // Validate parent if changed
        if let Some(parent_id) = request.parent_id {
            if parent_id == id {
                return Err(Error::validation("A page cannot be its own parent"));
            }
            if self.find_by_id(parent_id).await?.is_none() {
                return Err(Error::validation("Parent page not found"));
            }
        }

        let updated_page = PageRow {
            id: existing.id,
            site_id: existing.site_id,
            post_type: existing.post_type,
            author_id: existing.author_id,
            title: request.title.unwrap_or(existing.title),
            slug: request.slug.unwrap_or(existing.slug),
            content: request.content.or(existing.content),
            excerpt: existing.excerpt,
            status: request.status.unwrap_or(existing.status),
            visibility: request.visibility.unwrap_or(existing.visibility),
            password: request.password.or(existing.password),
            parent_id: request.parent_id.or(existing.parent_id),
            menu_order: request.menu_order.unwrap_or(existing.menu_order),
            template: request.template.or(existing.template),
            featured_image_id: request.featured_image_id.or(existing.featured_image_id),
            comment_status: existing.comment_status,
            comment_count: existing.comment_count,
            ping_status: existing.ping_status,
            meta_title: existing.meta_title,
            meta_description: existing.meta_description,
            canonical_url: existing.canonical_url,
            published_at: existing.published_at,
            scheduled_at: existing.scheduled_at,
            created_at: existing.created_at,
            updated_at: Utc::now(),
            deleted_at: existing.deleted_at,
        };

        let updated = self.update(&updated_page).await?;
        Ok(PageResponse::from(updated))
    }

    /// Delete a page (soft delete)
    pub async fn delete_page(&self, id: Uuid) -> Result<bool> {
        let _ = self.find_by_id(id).await?
            .ok_or_else(|| Error::not_found("Page", id.to_string()))?;

        self.soft_delete(id).await?;
        Ok(true)
    }

    // =====================
    // Database operations
    // =====================

    async fn find_by_id(&self, id: Uuid) -> Result<Option<PageRow>> {
        let query = format!(
            "SELECT {} FROM posts WHERE id = $1 AND {} AND post_type = 'page' AND deleted_at IS NULL",
            PageRow::COLUMNS,
            self.site_condition()
        );

        sqlx::query_as::<_, PageRow>(&query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to find page", e))
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<PageRow>> {
        let query = format!(
            "SELECT {} FROM posts WHERE slug = $1 AND {} AND post_type = 'page' AND deleted_at IS NULL",
            PageRow::COLUMNS,
            self.site_condition()
        );

        sqlx::query_as::<_, PageRow>(&query)
            .bind(slug)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to find page by slug", e))
    }

    async fn create(&self, page: &PageRow) -> Result<PageRow> {
        let query = format!(
            r#"
            INSERT INTO posts (
                id, site_id, post_type, author_id, title, slug, content, excerpt,
                status, visibility, password, parent_id, menu_order, template,
                featured_image_id, comment_status, comment_count, ping_status,
                meta_title, meta_description, canonical_url, published_at, scheduled_at,
                created_at, updated_at
            )
            VALUES ($1, $2, $3::post_type, $4, $5, $6, $7, $8, $9::post_status, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25)
            RETURNING {}
            "#,
            PageRow::COLUMNS
        );
        sqlx::query_as::<_, PageRow>(&query)
        .bind(page.id)
        .bind(page.site_id)
        .bind(&page.post_type)
        .bind(page.author_id)
        .bind(&page.title)
        .bind(&page.slug)
        .bind(&page.content)
        .bind(&page.excerpt)
        .bind(&page.status)
        .bind(&page.visibility)
        .bind(&page.password)
        .bind(page.parent_id)
        .bind(page.menu_order)
        .bind(&page.template)
        .bind(page.featured_image_id)
        .bind(&page.comment_status)
        .bind(page.comment_count)
        .bind(&page.ping_status)
        .bind(&page.meta_title)
        .bind(&page.meta_description)
        .bind(&page.canonical_url)
        .bind(page.published_at)
        .bind(page.scheduled_at)
        .bind(page.created_at)
        .bind(page.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to create page", e))
    }

    async fn update(&self, page: &PageRow) -> Result<PageRow> {
        let query = format!(
            r#"
            UPDATE posts SET
                title = $2,
                slug = $3,
                content = $4,
                excerpt = $5,
                status = $6::post_status,
                visibility = $7,
                password = $8,
                parent_id = $9,
                menu_order = $10,
                template = $11,
                featured_image_id = $12,
                meta_title = $13,
                meta_description = $14,
                canonical_url = $15,
                published_at = $16,
                updated_at = NOW()
            WHERE id = $1 AND post_type = 'page'
            RETURNING {}
            "#,
            PageRow::COLUMNS
        );
        sqlx::query_as::<_, PageRow>(&query)
        .bind(page.id)
        .bind(&page.title)
        .bind(&page.slug)
        .bind(&page.content)
        .bind(&page.excerpt)
        .bind(&page.status)
        .bind(&page.visibility)
        .bind(&page.password)
        .bind(page.parent_id)
        .bind(page.menu_order)
        .bind(&page.template)
        .bind(page.featured_image_id)
        .bind(&page.meta_title)
        .bind(&page.meta_description)
        .bind(&page.canonical_url)
        .bind(page.published_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to update page", e))
    }

    async fn soft_delete(&self, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE posts SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1 AND post_type = 'page'")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to delete page", e))?;
        Ok(())
    }

    // =====================
    // Helper methods
    // =====================

    fn generate_slug(&self, title: &str) -> String {
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

    fn build_hierarchy(&self, pages: Vec<PageResponse>) -> Vec<PageResponse> {
        use std::collections::HashMap;

        let mut by_id: HashMap<Uuid, PageResponse> = HashMap::new();
        let mut children_map: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        let mut root_ids: Vec<Uuid> = Vec::new();

        // First pass: collect all pages and parent-child relationships
        for page in pages {
            if let Some(parent_id) = page.parent_id {
                children_map.entry(parent_id).or_default().push(page.id);
            } else {
                root_ids.push(page.id);
            }
            by_id.insert(page.id, page);
        }

        // Recursive function to build tree
        fn build_tree(
            id: Uuid,
            by_id: &mut HashMap<Uuid, PageResponse>,
            children_map: &HashMap<Uuid, Vec<Uuid>>,
        ) -> Option<PageResponse> {
            let mut page = by_id.remove(&id)?;

            if let Some(child_ids) = children_map.get(&id) {
                for child_id in child_ids {
                    if let Some(child) = build_tree(*child_id, by_id, children_map) {
                        page.children.push(child);
                    }
                }
            }

            // Sort children by menu_order
            page.children.sort_by(|a, b| {
                a.menu_order.unwrap_or(0).cmp(&b.menu_order.unwrap_or(0))
            });

            Some(page)
        }

        // Build tree from roots
        let mut result = Vec::new();
        for root_id in root_ids {
            if let Some(tree) = build_tree(root_id, &mut by_id, &children_map) {
                result.push(tree);
            }
        }

        result
    }

    async fn get_author(&self, author_id: Uuid) -> Result<Option<PageAuthorResponse>> {
        let query = "SELECT id, display_name, email, avatar_url FROM users WHERE id = $1 AND deleted_at IS NULL";

        let row: Option<(Uuid, Option<String>, String, Option<String>)> = sqlx::query_as(query)
            .bind(author_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to get author", e))?;

        Ok(row.map(|(id, display_name, email, avatar_url)| PageAuthorResponse {
            id,
            name: display_name.unwrap_or_else(|| "Unknown".to_string()),
            email: Some(email),
            avatar_url,
        }))
    }
}

impl Default for PageService {
    fn default() -> Self {
        panic!("PageService requires a database pool")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_slug() {
        let service = PageService {
            pool: unsafe { std::mem::zeroed() },
            site_id: None,
        };

        assert_eq!(service.generate_slug("About Us"), "about-us");
        assert_eq!(service.generate_slug("Contact & Support"), "contact-support");
    }

    #[test]
    fn test_page_status_display() {
        assert_eq!(PageStatus::Draft.to_string(), "draft");
        assert_eq!(PageStatus::Published.to_string(), "published");
    }
}
