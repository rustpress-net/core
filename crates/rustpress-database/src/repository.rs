//! Generic repository implementations for database operations.

use async_trait::async_trait;
use rustpress_core::error::{Error, Result};
use rustpress_core::id::TenantId;
use rustpress_core::repository::{QueryableRepository, Repository};
use rustpress_core::service::{ListParams, ListResult, SortOrder};
use serde::{de::DeserializeOwned, Serialize};
use sqlx::{FromRow, PgPool, Postgres, Row};
use std::marker::PhantomData;
use uuid::Uuid;

/// Generic PostgreSQL repository
pub struct PgRepository<T> {
    pool: PgPool,
    table_name: String,
    tenant_id: Option<TenantId>,
    _phantom: PhantomData<T>,
}

impl<T> PgRepository<T> {
    pub fn new(pool: PgPool, table_name: impl Into<String>) -> Self {
        Self {
            pool,
            table_name: table_name.into(),
            tenant_id: None,
            _phantom: PhantomData,
        }
    }

    pub fn with_tenant(mut self, tenant_id: TenantId) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    /// Build a WHERE clause with tenant filtering
    fn tenant_filter(&self, alias: Option<&str>) -> String {
        if let Some(tenant_id) = &self.tenant_id {
            let prefix = alias.map(|a| format!("{}.", a)).unwrap_or_default();
            format!("{}tenant_id = '{}'", prefix, tenant_id)
        } else {
            "1=1".to_string()
        }
    }
}

impl<T> Clone for PgRepository<T> {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            table_name: self.table_name.clone(),
            tenant_id: self.tenant_id,
            _phantom: PhantomData,
        }
    }
}

/// Helper trait for entities with ID
pub trait Entity: Send + Sync {
    fn id(&self) -> Uuid;
}

/// Helper to build dynamic queries
pub struct QueryHelper;

impl QueryHelper {
    /// Build ORDER BY clause
    pub fn order_by(sort_by: Option<&str>, sort_order: SortOrder) -> String {
        let field = sort_by.unwrap_or("created_at");
        let direction = match sort_order {
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        };
        format!("ORDER BY {} {}", field, direction)
    }

    /// Build LIMIT/OFFSET clause
    pub fn pagination(params: &ListParams) -> String {
        format!(
            "LIMIT {} OFFSET {}",
            params.per_page,
            params.offset()
        )
    }

    /// Build search condition
    pub fn search_condition(search: &str, fields: &[&str]) -> String {
        let conditions: Vec<String> = fields
            .iter()
            .map(|f| format!("{} ILIKE '%{}%'", f, search.replace('\'', "''")))
            .collect();

        if conditions.is_empty() {
            "1=1".to_string()
        } else {
            format!("({})", conditions.join(" OR "))
        }
    }

    /// Escape string for LIKE
    pub fn escape_like(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    }
}

/// User repository implementation
pub mod users {
    use super::*;
    use chrono::{DateTime, Utc};

    #[derive(Debug, Clone, sqlx::FromRow)]
    pub struct UserRow {
        pub id: Uuid,
        pub email: String,
        pub username: String,
        pub password_hash: String,
        pub display_name: Option<String>,
        pub status: String,
        pub role: String,
        pub avatar_url: Option<String>,
        pub locale: Option<String>,
        pub timezone: Option<String>,
        pub email_verified_at: Option<DateTime<Utc>>,
        pub last_login_at: Option<DateTime<Utc>>,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
        pub deleted_at: Option<DateTime<Utc>>,
    }

    pub struct UserRepository {
        pool: PgPool,
    }

    impl UserRepository {
        pub fn new(pool: PgPool) -> Self {
            Self { pool }
        }

        pub async fn find_by_email(&self, email: &str) -> Result<Option<UserRow>> {
            sqlx::query_as::<_, UserRow>(
                "SELECT * FROM users WHERE email = $1 AND deleted_at IS NULL",
            )
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to find user by email", e))
        }

        pub async fn find_by_username(&self, username: &str) -> Result<Option<UserRow>> {
            sqlx::query_as::<_, UserRow>(
                "SELECT * FROM users WHERE username = $1 AND deleted_at IS NULL",
            )
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to find user by username", e))
        }

        pub async fn create(&self, user: &UserRow) -> Result<UserRow> {
            sqlx::query_as::<_, UserRow>(
                r#"
                INSERT INTO users (id, email, username, password_hash, display_name, status, role, avatar_url, locale, timezone, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                RETURNING *
                "#,
            )
            .bind(user.id)
            .bind(&user.email)
            .bind(&user.username)
            .bind(&user.password_hash)
            .bind(&user.display_name)
            .bind(&user.status)
            .bind(&user.role)
            .bind(&user.avatar_url)
            .bind(&user.locale)
            .bind(&user.timezone)
            .bind(user.created_at)
            .bind(user.updated_at)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to create user", e))
        }

        pub async fn update_last_login(&self, user_id: Uuid) -> Result<()> {
            sqlx::query("UPDATE users SET last_login_at = NOW(), updated_at = NOW() WHERE id = $1")
                .bind(user_id)
                .execute(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to update last login", e))?;
            Ok(())
        }

        pub async fn verify_email(&self, user_id: Uuid) -> Result<()> {
            sqlx::query(
                "UPDATE users SET email_verified_at = NOW(), status = 'active', updated_at = NOW() WHERE id = $1",
            )
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to verify email", e))?;
            Ok(())
        }
    }
}

/// Post repository implementation
pub mod posts {
    use super::*;
    use chrono::{DateTime, Utc};

    #[derive(Debug, Clone, sqlx::FromRow)]
    pub struct PostRow {
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

    impl PostRow {
        /// Columns to select (excludes search_vector, casts enums to text)
        pub const COLUMNS: &'static str = "id, site_id, post_type::text as post_type, author_id, title, slug, content, excerpt, status::text as status, visibility, password, parent_id, menu_order, template, featured_image_id, comment_status, comment_count, ping_status, meta_title, meta_description, canonical_url, published_at, scheduled_at, created_at, updated_at, deleted_at";
    }

    pub struct PostRepository {
        pool: PgPool,
        site_id: Option<Uuid>,
    }

    impl PostRepository {
        pub fn new(pool: PgPool) -> Self {
            Self {
                pool,
                site_id: None,
            }
        }

        pub fn with_site(mut self, site_id: Uuid) -> Self {
            self.site_id = Some(site_id);
            self
        }

        fn site_condition(&self) -> String {
            match self.site_id {
                Some(id) => format!("site_id = '{}'", id),
                None => "site_id IS NULL".to_string(),
            }
        }

        pub async fn find_by_id(&self, id: Uuid) -> Result<Option<PostRow>> {
            let query = format!(
                "SELECT {} FROM posts WHERE id = $1 AND {} AND deleted_at IS NULL",
                PostRow::COLUMNS,
                self.site_condition()
            );

            sqlx::query_as::<_, PostRow>(&query)
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to find post", e))
        }

        pub async fn find_by_slug(&self, slug: &str) -> Result<Option<PostRow>> {
            let query = format!(
                "SELECT {} FROM posts WHERE slug = $1 AND {} AND deleted_at IS NULL",
                PostRow::COLUMNS,
                self.site_condition()
            );

            sqlx::query_as::<_, PostRow>(&query)
                .bind(slug)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to find post by slug", e))
        }

        pub async fn list(&self, params: &ListParams) -> Result<ListResult<PostRow>> {
            let search_condition = params
                .search
                .as_ref()
                .map(|s| QueryHelper::search_condition(s, &["title", "content"]))
                .unwrap_or_else(|| "1=1".to_string());

            let count_query = format!(
                "SELECT COUNT(*) as count FROM posts WHERE {} AND {} AND deleted_at IS NULL",
                self.site_condition(),
                search_condition
            );

            let total: (i64,) = sqlx::query_as(&count_query)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to count posts", e))?;

            let query = format!(
                "SELECT {} FROM posts WHERE {} AND {} AND deleted_at IS NULL {} {}",
                PostRow::COLUMNS,
                self.site_condition(),
                search_condition,
                QueryHelper::order_by(params.sort_by.as_deref(), params.sort_order),
                QueryHelper::pagination(params)
            );

            let posts = sqlx::query_as::<_, PostRow>(&query)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to list posts", e))?;

            Ok(ListResult::new(posts, total.0 as u64, params))
        }

        pub async fn list_published(&self, params: &ListParams) -> Result<ListResult<PostRow>> {
            let search_condition = params
                .search
                .as_ref()
                .map(|s| QueryHelper::search_condition(s, &["title", "content"]))
                .unwrap_or_else(|| "1=1".to_string());

            let count_query = format!(
                "SELECT COUNT(*) as count FROM posts WHERE {} AND {} AND status = 'published' AND deleted_at IS NULL",
                self.site_condition(),
                search_condition
            );

            let total: (i64,) = sqlx::query_as(&count_query)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to count posts", e))?;

            let query = format!(
                "SELECT {} FROM posts WHERE {} AND {} AND status = 'published' AND deleted_at IS NULL {} {}",
                PostRow::COLUMNS,
                self.site_condition(),
                search_condition,
                QueryHelper::order_by(Some("published_at"), SortOrder::Desc),
                QueryHelper::pagination(params)
            );

            let posts = sqlx::query_as::<_, PostRow>(&query)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to list published posts", e))?;

            Ok(ListResult::new(posts, total.0 as u64, params))
        }

        pub async fn create(&self, post: &PostRow) -> Result<PostRow> {
            let query = format!(
                r#"
                INSERT INTO posts (id, site_id, post_type, author_id, title, slug, content, excerpt, status, visibility, password, parent_id, menu_order, template, featured_image_id, comment_status, comment_count, ping_status, meta_title, meta_description, canonical_url, published_at, scheduled_at, created_at, updated_at)
                VALUES ($1, $2, $3::post_type, $4, $5, $6, $7, $8, $9::post_status, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25)
                RETURNING {}
                "#,
                PostRow::COLUMNS
            );
            sqlx::query_as::<_, PostRow>(&query)
            .bind(post.id)
            .bind(post.site_id)
            .bind(&post.post_type)
            .bind(post.author_id)
            .bind(&post.title)
            .bind(&post.slug)
            .bind(&post.content)
            .bind(&post.excerpt)
            .bind(&post.status)
            .bind(&post.visibility)
            .bind(&post.password)
            .bind(post.parent_id)
            .bind(post.menu_order)
            .bind(&post.template)
            .bind(post.featured_image_id)
            .bind(&post.comment_status)
            .bind(post.comment_count)
            .bind(&post.ping_status)
            .bind(&post.meta_title)
            .bind(&post.meta_description)
            .bind(&post.canonical_url)
            .bind(post.published_at)
            .bind(post.scheduled_at)
            .bind(post.created_at)
            .bind(post.updated_at)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to create post", e))
        }

        pub async fn update(&self, post: &PostRow) -> Result<PostRow> {
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
                    comment_status = $13,
                    ping_status = $14,
                    meta_title = $15,
                    meta_description = $16,
                    canonical_url = $17,
                    published_at = $18,
                    scheduled_at = $19,
                    updated_at = NOW()
                WHERE id = $1
                RETURNING {}
                "#,
                PostRow::COLUMNS
            );
            sqlx::query_as::<_, PostRow>(&query)
            .bind(post.id)
            .bind(&post.title)
            .bind(&post.slug)
            .bind(&post.content)
            .bind(&post.excerpt)
            .bind(&post.status)
            .bind(&post.visibility)
            .bind(&post.password)
            .bind(post.parent_id)
            .bind(post.menu_order)
            .bind(&post.template)
            .bind(post.featured_image_id)
            .bind(&post.comment_status)
            .bind(&post.ping_status)
            .bind(&post.meta_title)
            .bind(&post.meta_description)
            .bind(&post.canonical_url)
            .bind(post.published_at)
            .bind(post.scheduled_at)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to update post", e))
        }

        pub async fn soft_delete(&self, id: Uuid) -> Result<()> {
            sqlx::query("UPDATE posts SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1")
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to delete post", e))?;
            Ok(())
        }

        pub async fn restore(&self, id: Uuid) -> Result<()> {
            sqlx::query("UPDATE posts SET deleted_at = NULL, updated_at = NOW() WHERE id = $1")
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to restore post", e))?;
            Ok(())
        }
    }
}

/// Options repository for site settings
pub mod options {
    use super::*;
    use chrono::{DateTime, Utc};

    /// Full option row matching database schema
    #[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
    pub struct OptionRow {
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

    /// Grouped settings response
    #[derive(Debug, Clone, serde::Serialize)]
    pub struct SettingsGroup {
        pub group: String,
        pub settings: Vec<OptionRow>,
    }

    pub struct OptionsRepository {
        pool: PgPool,
        site_id: Option<Uuid>,
    }

    impl OptionsRepository {
        pub fn new(pool: PgPool) -> Self {
            Self {
                pool,
                site_id: None,
            }
        }

        pub fn with_site(mut self, site_id: Uuid) -> Self {
            self.site_id = Some(site_id);
            self
        }

        fn site_condition(&self) -> String {
            match self.site_id {
                Some(id) => format!("site_id = '{}'", id),
                None => "site_id IS NULL".to_string(),
            }
        }

        /// Get a single option value by name
        pub async fn get(&self, name: &str) -> Result<Option<serde_json::Value>> {
            let query = format!(
                "SELECT option_value FROM options WHERE option_name = $1 AND {}",
                self.site_condition()
            );

            let result: Option<(Option<serde_json::Value>,)> = sqlx::query_as(&query)
                .bind(name)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to get option", e))?;

            Ok(result.and_then(|(v,)| v))
        }

        /// Get full option row by name
        pub async fn get_full(&self, name: &str) -> Result<Option<OptionRow>> {
            let query = format!(
                "SELECT * FROM options WHERE option_name = $1 AND {}",
                self.site_condition()
            );

            sqlx::query_as::<_, OptionRow>(&query)
                .bind(name)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to get option", e))
        }

        /// Set an option value (upsert)
        pub async fn set(&self, name: &str, value: serde_json::Value) -> Result<()> {
            let id = Uuid::now_v7();

            sqlx::query(
                r#"
                INSERT INTO options (id, site_id, option_name, option_value)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (option_name, site_id) DO UPDATE SET
                    option_value = EXCLUDED.option_value,
                    updated_at = NOW()
                "#,
            )
            .bind(id)
            .bind(self.site_id)
            .bind(name)
            .bind(value)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to set option", e))?;

            Ok(())
        }

        /// Delete an option (only non-system options)
        pub async fn delete(&self, name: &str) -> Result<bool> {
            let query = format!(
                "DELETE FROM options WHERE option_name = $1 AND {} AND is_system = false",
                self.site_condition()
            );

            let result = sqlx::query(&query)
                .bind(name)
                .execute(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to delete option", e))?;

            Ok(result.rows_affected() > 0)
        }

        /// Get all options that should be autoloaded
        pub async fn get_autoload(&self) -> Result<Vec<OptionRow>> {
            let query = format!(
                "SELECT * FROM options WHERE autoload = true AND ({} OR site_id IS NULL) ORDER BY option_group, option_name",
                self.site_condition()
            );

            sqlx::query_as::<_, OptionRow>(&query)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to get autoload options", e))
        }

        /// Get all options
        pub async fn get_all(&self) -> Result<Vec<OptionRow>> {
            let query = format!(
                "SELECT * FROM options WHERE ({} OR site_id IS NULL) ORDER BY option_group, option_name",
                self.site_condition()
            );

            sqlx::query_as::<_, OptionRow>(&query)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to get all options", e))
        }

        /// Get options by group
        pub async fn get_by_group(&self, group: &str) -> Result<Vec<OptionRow>> {
            let query = format!(
                "SELECT * FROM options WHERE option_group = $1 AND ({} OR site_id IS NULL) ORDER BY option_name",
                self.site_condition()
            );

            sqlx::query_as::<_, OptionRow>(&query)
                .bind(group)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to get options by group", e))
        }

        /// Get all options grouped by their option_group
        pub async fn get_all_grouped(&self) -> Result<Vec<SettingsGroup>> {
            let options = self.get_all().await?;

            let mut groups: std::collections::HashMap<String, Vec<OptionRow>> = std::collections::HashMap::new();
            for option in options {
                groups.entry(option.option_group.clone())
                    .or_default()
                    .push(option);
            }

            let mut result: Vec<SettingsGroup> = groups
                .into_iter()
                .map(|(group, settings)| SettingsGroup { group, settings })
                .collect();

            // Sort groups in a logical order
            let group_order = ["general", "reading", "discussion", "media", "permalinks", "privacy", "security"];
            result.sort_by(|a, b| {
                let a_idx = group_order.iter().position(|&g| g == a.group).unwrap_or(999);
                let b_idx = group_order.iter().position(|&g| g == b.group).unwrap_or(999);
                a_idx.cmp(&b_idx)
            });

            Ok(result)
        }

        /// Batch update multiple options
        pub async fn batch_update(&self, updates: Vec<(String, serde_json::Value)>) -> Result<u64> {
            let mut updated = 0u64;

            for (name, value) in updates {
                sqlx::query(
                    r#"
                    UPDATE options
                    SET option_value = $1, updated_at = NOW()
                    WHERE option_name = $2 AND (site_id = $3 OR ($3 IS NULL AND site_id IS NULL))
                    "#,
                )
                .bind(&value)
                .bind(&name)
                .bind(self.site_id)
                .execute(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to batch update options", e))?;

                updated += 1;
            }

            Ok(updated)
        }

        /// Get available option groups
        pub async fn get_groups(&self) -> Result<Vec<String>> {
            let query = format!(
                "SELECT DISTINCT option_group FROM options WHERE ({} OR site_id IS NULL) ORDER BY option_group",
                self.site_condition()
            );

            let rows: Vec<(String,)> = sqlx::query_as(&query)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to get option groups", e))?;

            Ok(rows.into_iter().map(|(g,)| g).collect())
        }
    }
}

/// Comments repository for comment management
pub mod comments {
    use super::*;
    use chrono::{DateTime, Utc};
    use std::collections::HashMap;

    /// Comment status enum matching database
    #[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, serde::Serialize, serde::Deserialize)]
    #[sqlx(type_name = "comment_status", rename_all = "lowercase")]
    #[serde(rename_all = "lowercase")]
    pub enum CommentStatus {
        Pending,
        Approved,
        Spam,
        Trash,
    }

    impl Default for CommentStatus {
        fn default() -> Self {
            Self::Pending
        }
    }

    impl std::fmt::Display for CommentStatus {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Pending => write!(f, "pending"),
                Self::Approved => write!(f, "approved"),
                Self::Spam => write!(f, "spam"),
                Self::Trash => write!(f, "trash"),
            }
        }
    }

    impl std::str::FromStr for CommentStatus {
        type Err = String;
        fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
            match s.to_lowercase().as_str() {
                "pending" => Ok(Self::Pending),
                "approved" => Ok(Self::Approved),
                "spam" => Ok(Self::Spam),
                "trash" => Ok(Self::Trash),
                _ => Err(format!("Invalid comment status: {}", s)),
            }
        }
    }

    /// Full comment row matching database schema
    #[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
    pub struct CommentRow {
        pub id: Uuid,
        pub site_id: Option<Uuid>,
        pub post_id: Uuid,
        pub parent_id: Option<Uuid>,
        pub depth: i32,
        // path is ltree type, we'll handle it as Option<String> for now
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

    /// Comment meta row
    #[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
    pub struct CommentMetaRow {
        pub id: Uuid,
        pub comment_id: Uuid,
        pub meta_key: String,
        pub meta_value: Option<serde_json::Value>,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }

    /// Comment like row
    #[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
    pub struct CommentLikeRow {
        pub comment_id: Uuid,
        pub user_id: Uuid,
        pub created_at: DateTime<Utc>,
    }

    /// Parameters for creating a comment
    #[derive(Debug, Clone)]
    pub struct CreateComment {
        pub post_id: Uuid,
        pub parent_id: Option<Uuid>,
        pub user_id: Option<Uuid>,
        pub author_name: Option<String>,
        pub author_email: Option<String>,
        pub author_url: Option<String>,
        pub author_ip: Option<String>,
        pub content: String,
        pub content_html: Option<String>,
        pub user_agent: Option<String>,
        pub status: CommentStatus,
    }

    /// Parameters for updating a comment
    #[derive(Debug, Clone, Default)]
    pub struct UpdateComment {
        pub content: Option<String>,
        pub content_html: Option<String>,
        pub status: Option<CommentStatus>,
    }

    /// Parameters for listing comments
    #[derive(Debug, Clone, Default)]
    pub struct CommentListParams {
        pub page: u64,
        pub per_page: u64,
        pub post_id: Option<Uuid>,
        pub parent_id: Option<Uuid>,
        pub user_id: Option<Uuid>,
        pub status: Option<CommentStatus>,
        pub search: Option<String>,
        pub include_deleted: bool,
        pub order_by: Option<String>,
        pub order_desc: bool,
    }

    impl CommentListParams {
        pub fn offset(&self) -> u64 {
            (self.page.saturating_sub(1)) * self.per_page
        }
    }

    /// Comment with author info joined
    #[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
    pub struct CommentWithAuthor {
        pub id: Uuid,
        pub post_id: Uuid,
        pub parent_id: Option<Uuid>,
        pub depth: i32,
        pub content: String,
        pub content_html: Option<String>,
        pub status: CommentStatus,
        pub likes_count: i32,
        pub replies_count: i32,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
        // Author info
        pub user_id: Option<Uuid>,
        pub author_name: Option<String>,
        pub author_email: Option<String>,
        pub author_url: Option<String>,
        // Joined user data
        pub user_display_name: Option<String>,
        pub user_avatar_url: Option<String>,
    }

    pub struct CommentsRepository {
        pool: PgPool,
        site_id: Option<Uuid>,
    }

    impl CommentsRepository {
        pub fn new(pool: PgPool) -> Self {
            Self {
                pool,
                site_id: None,
            }
        }

        pub fn with_site(mut self, site_id: Uuid) -> Self {
            self.site_id = Some(site_id);
            self
        }

        fn site_condition(&self) -> String {
            match self.site_id {
                Some(id) => format!("site_id = '{}'", id),
                None => "site_id IS NULL".to_string(),
            }
        }

        /// Create a new comment
        pub async fn create(&self, comment: &CreateComment) -> Result<CommentRow> {
            let id = Uuid::now_v7();

            // Note: path and depth are auto-set by DB trigger
            sqlx::query_as::<_, CommentRow>(
                r#"
                INSERT INTO comments (
                    id, site_id, post_id, parent_id, user_id,
                    author_name, author_email, author_url, author_ip,
                    content, content_html, status, user_agent, created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, NOW(), NOW())
                RETURNING id, site_id, post_id, parent_id, depth, user_id,
                    author_name, author_email, author_url, author_ip,
                    content, content_html, status, is_edited, edited_at,
                    moderated_by, moderated_at, moderation_note,
                    spam_score, spam_reasons, likes_count, replies_count,
                    user_agent, created_at, updated_at, deleted_at
                "#,
            )
            .bind(id)
            .bind(self.site_id)
            .bind(comment.post_id)
            .bind(comment.parent_id)
            .bind(comment.user_id)
            .bind(&comment.author_name)
            .bind(&comment.author_email)
            .bind(&comment.author_url)
            .bind(&comment.author_ip)
            .bind(&comment.content)
            .bind(&comment.content_html)
            .bind(comment.status)
            .bind(&comment.user_agent)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to create comment", e))
        }

        /// Find a comment by ID
        pub async fn find_by_id(&self, id: Uuid) -> Result<Option<CommentRow>> {
            let query = format!(
                r#"
                SELECT id, site_id, post_id, parent_id, depth, user_id,
                    author_name, author_email, author_url, author_ip,
                    content, content_html, status, is_edited, edited_at,
                    moderated_by, moderated_at, moderation_note,
                    spam_score, spam_reasons, likes_count, replies_count,
                    user_agent, created_at, updated_at, deleted_at
                FROM comments
                WHERE id = $1 AND {} AND deleted_at IS NULL
                "#,
                self.site_condition()
            );

            sqlx::query_as::<_, CommentRow>(&query)
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to find comment", e))
        }

        /// Update a comment
        pub async fn update(&self, id: Uuid, updates: &UpdateComment) -> Result<CommentRow> {
            let mut set_clauses = vec!["updated_at = NOW()".to_string()];
            let mut param_index = 2; // $1 is id

            if updates.content.is_some() {
                set_clauses.push(format!("content = ${}", param_index));
                set_clauses.push("is_edited = true".to_string());
                set_clauses.push("edited_at = NOW()".to_string());
                param_index += 1;
            }

            if updates.content_html.is_some() {
                set_clauses.push(format!("content_html = ${}", param_index));
                param_index += 1;
            }

            if updates.status.is_some() {
                set_clauses.push(format!("status = ${}", param_index));
            }

            let query = format!(
                r#"
                UPDATE comments SET {}
                WHERE id = $1 AND {}
                RETURNING id, site_id, post_id, parent_id, depth, user_id,
                    author_name, author_email, author_url, author_ip,
                    content, content_html, status, is_edited, edited_at,
                    moderated_by, moderated_at, moderation_note,
                    spam_score, spam_reasons, likes_count, replies_count,
                    user_agent, created_at, updated_at, deleted_at
                "#,
                set_clauses.join(", "),
                self.site_condition()
            );

            let mut query_builder = sqlx::query_as::<_, CommentRow>(&query).bind(id);

            if let Some(ref content) = updates.content {
                query_builder = query_builder.bind(content);
            }
            if let Some(ref html) = updates.content_html {
                query_builder = query_builder.bind(html);
            }
            if let Some(status) = updates.status {
                query_builder = query_builder.bind(status);
            }

            query_builder
                .fetch_one(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to update comment", e))
        }

        /// Update comment status with moderation info
        pub async fn update_status(
            &self,
            id: Uuid,
            status: CommentStatus,
            moderator_id: Uuid,
            note: Option<&str>,
        ) -> Result<CommentRow> {
            let query = format!(
                r#"
                UPDATE comments SET
                    status = $2,
                    moderated_by = $3,
                    moderated_at = NOW(),
                    moderation_note = $4,
                    updated_at = NOW()
                WHERE id = $1 AND {}
                RETURNING id, site_id, post_id, parent_id, depth, user_id,
                    author_name, author_email, author_url, author_ip,
                    content, content_html, status, is_edited, edited_at,
                    moderated_by, moderated_at, moderation_note,
                    spam_score, spam_reasons, likes_count, replies_count,
                    user_agent, created_at, updated_at, deleted_at
                "#,
                self.site_condition()
            );

            sqlx::query_as::<_, CommentRow>(&query)
                .bind(id)
                .bind(status)
                .bind(moderator_id)
                .bind(note)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to update comment status", e))
        }

        /// Batch update status for multiple comments
        pub async fn batch_update_status(
            &self,
            ids: &[Uuid],
            status: CommentStatus,
            moderator_id: Uuid,
        ) -> Result<u64> {
            if ids.is_empty() {
                return Ok(0);
            }

            // Use PostgreSQL ANY() for batch update
            let result = sqlx::query(
                &format!(
                    r#"
                    UPDATE comments SET
                        status = $1,
                        moderated_by = $2,
                        moderated_at = NOW(),
                        updated_at = NOW()
                    WHERE id = ANY($3) AND {}
                    "#,
                    self.site_condition()
                )
            )
            .bind(status)
            .bind(moderator_id)
            .bind(ids)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to batch update comments", e))?;

            Ok(result.rows_affected())
        }

        /// Soft delete a comment
        pub async fn soft_delete(&self, id: Uuid) -> Result<bool> {
            let query = format!(
                "UPDATE comments SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1 AND {} AND deleted_at IS NULL",
                self.site_condition()
            );

            let result = sqlx::query(&query)
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to delete comment", e))?;

            Ok(result.rows_affected() > 0)
        }

        /// Restore a soft-deleted comment
        pub async fn restore(&self, id: Uuid) -> Result<bool> {
            let query = format!(
                "UPDATE comments SET deleted_at = NULL, updated_at = NOW() WHERE id = $1 AND {}",
                self.site_condition()
            );

            let result = sqlx::query(&query)
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to restore comment", e))?;

            Ok(result.rows_affected() > 0)
        }

        /// List comments with pagination and filtering
        pub async fn list(&self, params: &CommentListParams) -> Result<(Vec<CommentRow>, u64)> {
            let mut conditions = vec![self.site_condition()];

            if !params.include_deleted {
                conditions.push("deleted_at IS NULL".to_string());
            }

            if let Some(post_id) = params.post_id {
                conditions.push(format!("post_id = '{}'", post_id));
            }

            if let Some(parent_id) = params.parent_id {
                conditions.push(format!("parent_id = '{}'", parent_id));
            } else if params.post_id.is_some() {
                // If listing for a post without parent filter, show top-level comments
                // Actually, we should NOT add this by default - let caller decide
            }

            if let Some(user_id) = params.user_id {
                conditions.push(format!("user_id = '{}'", user_id));
            }

            if let Some(status) = params.status {
                conditions.push(format!("status = '{}'", status));
            }

            if let Some(ref search) = params.search {
                let escaped = search.replace('\'', "''");
                conditions.push(format!(
                    "(content ILIKE '%{}%' OR author_name ILIKE '%{}%' OR author_email ILIKE '%{}%')",
                    escaped, escaped, escaped
                ));
            }

            let where_clause = conditions.join(" AND ");
            let order_by = params.order_by.as_deref().unwrap_or("created_at");
            let order_dir = if params.order_desc { "DESC" } else { "ASC" };

            // Count query
            let count_query = format!(
                "SELECT COUNT(*) as count FROM comments WHERE {}",
                where_clause
            );
            let total: (i64,) = sqlx::query_as(&count_query)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to count comments", e))?;

            // Data query
            let data_query = format!(
                r#"
                SELECT id, site_id, post_id, parent_id, depth, user_id,
                    author_name, author_email, author_url, author_ip,
                    content, content_html, status, is_edited, edited_at,
                    moderated_by, moderated_at, moderation_note,
                    spam_score, spam_reasons, likes_count, replies_count,
                    user_agent, created_at, updated_at, deleted_at
                FROM comments
                WHERE {}
                ORDER BY {} {}
                LIMIT {} OFFSET {}
                "#,
                where_clause,
                order_by,
                order_dir,
                params.per_page,
                params.offset()
            );

            let comments = sqlx::query_as::<_, CommentRow>(&data_query)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to list comments", e))?;

            Ok((comments, total.0 as u64))
        }

        /// List comments by post with author info
        pub async fn list_by_post_with_authors(&self, post_id: Uuid, status: Option<CommentStatus>) -> Result<Vec<CommentWithAuthor>> {
            let status_condition = match status {
                Some(s) => format!("AND c.status = '{}'", s),
                None => String::new(),
            };

            let query = format!(
                r#"
                SELECT
                    c.id, c.post_id, c.parent_id, c.depth,
                    c.content, c.content_html, c.status,
                    c.likes_count, c.replies_count,
                    c.created_at, c.updated_at,
                    c.user_id, c.author_name, c.author_email, c.author_url,
                    u.display_name as user_display_name,
                    u.avatar_url as user_avatar_url
                FROM comments c
                LEFT JOIN users u ON c.user_id = u.id
                WHERE c.post_id = $1 AND c.deleted_at IS NULL {}
                ORDER BY c.created_at ASC
                "#,
                status_condition
            );

            sqlx::query_as::<_, CommentWithAuthor>(&query)
                .bind(post_id)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to list comments for post", e))
        }

        /// Get comment counts by status
        pub async fn count_by_status(&self) -> Result<HashMap<String, i64>> {
            let query = format!(
                r#"
                SELECT status::text, COUNT(*) as count
                FROM comments
                WHERE {} AND deleted_at IS NULL
                GROUP BY status
                "#,
                self.site_condition()
            );

            let rows: Vec<(String, i64)> = sqlx::query_as(&query)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to count comments by status", e))?;

            Ok(rows.into_iter().collect())
        }

        /// Add a like to a comment
        pub async fn add_like(&self, comment_id: Uuid, user_id: Uuid) -> Result<bool> {
            let result = sqlx::query(
                r#"
                INSERT INTO comment_likes (comment_id, user_id, created_at)
                VALUES ($1, $2, NOW())
                ON CONFLICT (comment_id, user_id) DO NOTHING
                "#,
            )
            .bind(comment_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to add like", e))?;

            Ok(result.rows_affected() > 0)
        }

        /// Remove a like from a comment
        pub async fn remove_like(&self, comment_id: Uuid, user_id: Uuid) -> Result<bool> {
            let result = sqlx::query(
                "DELETE FROM comment_likes WHERE comment_id = $1 AND user_id = $2",
            )
            .bind(comment_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to remove like", e))?;

            Ok(result.rows_affected() > 0)
        }

        /// Check if user has liked a comment
        pub async fn has_liked(&self, comment_id: Uuid, user_id: Uuid) -> Result<bool> {
            let result: Option<(Uuid,)> = sqlx::query_as(
                "SELECT comment_id FROM comment_likes WHERE comment_id = $1 AND user_id = $2",
            )
            .bind(comment_id)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to check like", e))?;

            Ok(result.is_some())
        }

        /// Get likes for a comment
        pub async fn get_likes(&self, comment_id: Uuid) -> Result<Vec<CommentLikeRow>> {
            sqlx::query_as::<_, CommentLikeRow>(
                "SELECT * FROM comment_likes WHERE comment_id = $1 ORDER BY created_at DESC",
            )
            .bind(comment_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to get likes", e))
        }

        /// Get comment meta
        pub async fn get_meta(&self, comment_id: Uuid, key: &str) -> Result<Option<serde_json::Value>> {
            let result: Option<(Option<serde_json::Value>,)> = sqlx::query_as(
                "SELECT meta_value FROM comment_meta WHERE comment_id = $1 AND meta_key = $2",
            )
            .bind(comment_id)
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to get comment meta", e))?;

            Ok(result.and_then(|(v,)| v))
        }

        /// Set comment meta
        pub async fn set_meta(&self, comment_id: Uuid, key: &str, value: serde_json::Value) -> Result<()> {
            let id = Uuid::now_v7();

            sqlx::query(
                r#"
                INSERT INTO comment_meta (id, comment_id, meta_key, meta_value, created_at, updated_at)
                VALUES ($1, $2, $3, $4, NOW(), NOW())
                ON CONFLICT (comment_id, meta_key) DO UPDATE SET
                    meta_value = EXCLUDED.meta_value,
                    updated_at = NOW()
                "#,
            )
            .bind(id)
            .bind(comment_id)
            .bind(key)
            .bind(value)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to set comment meta", e))?;

            Ok(())
        }

        /// Get all meta for a comment
        pub async fn get_all_meta(&self, comment_id: Uuid) -> Result<Vec<CommentMetaRow>> {
            sqlx::query_as::<_, CommentMetaRow>(
                "SELECT * FROM comment_meta WHERE comment_id = $1 ORDER BY meta_key",
            )
            .bind(comment_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to get all comment meta", e))
        }

        /// Update spam score
        pub async fn update_spam_score(&self, id: Uuid, score: f64, reasons: Option<serde_json::Value>) -> Result<()> {
            sqlx::query(
                "UPDATE comments SET spam_score = $2, spam_reasons = $3, updated_at = NOW() WHERE id = $1",
            )
            .bind(id)
            .bind(score)
            .bind(reasons)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to update spam score", e))?;

            Ok(())
        }

        /// Get replies to a comment
        pub async fn get_replies(&self, parent_id: Uuid) -> Result<Vec<CommentRow>> {
            let query = format!(
                r#"
                SELECT id, site_id, post_id, parent_id, depth, user_id,
                    author_name, author_email, author_url, author_ip,
                    content, content_html, status, is_edited, edited_at,
                    moderated_by, moderated_at, moderation_note,
                    spam_score, spam_reasons, likes_count, replies_count,
                    user_agent, created_at, updated_at, deleted_at
                FROM comments
                WHERE parent_id = $1 AND {} AND deleted_at IS NULL
                ORDER BY created_at ASC
                "#,
                self.site_condition()
            );

            sqlx::query_as::<_, CommentRow>(&query)
                .bind(parent_id)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to get replies", e))
        }

        /// Check if an author email has previously had an approved comment
        pub async fn has_approved_comment_by_email(&self, email: &str) -> Result<bool> {
            let query = format!(
                r#"
                SELECT EXISTS(
                    SELECT 1 FROM comments
                    WHERE author_email = $1
                    AND status = 'approved'
                    AND {}
                    AND deleted_at IS NULL
                ) as exists
                "#,
                self.site_condition()
            );

            let result: (bool,) = sqlx::query_as(&query)
                .bind(email)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to check approved comment by email", e))?;

            Ok(result.0)
        }

        /// Check if a user has previously had an approved comment
        pub async fn has_approved_comment_by_user(&self, user_id: Uuid) -> Result<bool> {
            let query = format!(
                r#"
                SELECT EXISTS(
                    SELECT 1 FROM comments
                    WHERE user_id = $1
                    AND status = 'approved'
                    AND {}
                    AND deleted_at IS NULL
                ) as exists
                "#,
                self.site_condition()
            );

            let result: (bool,) = sqlx::query_as(&query)
                .bind(user_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to check approved comment by user", e))?;

            Ok(result.0)
        }
    }
}

/// Theme repository for theme management
/// Themes are presentation-only - content (posts, pages, menus, widgets) is theme-independent
pub mod themes {
    use super::*;
    use chrono::{DateTime, Utc};

    /// Theme database row
    #[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
    pub struct ThemeRow {
        pub id: Uuid,
        pub site_id: Option<Uuid>,
        pub theme_id: String,
        pub name: String,
        pub description: Option<String>,
        pub version: Option<String>,
        pub author: Option<String>,
        pub author_url: Option<String>,
        pub license: Option<String>,
        pub is_active: bool,
        pub is_installed: bool,
        pub parent_theme_id: Option<String>,
        pub screenshot_url: Option<String>,
        pub homepage_url: Option<String>,
        pub tags: Option<Vec<String>>,
        pub supports: serde_json::Value,
        pub menu_locations: serde_json::Value,
        pub widget_areas: serde_json::Value,
        pub customizer_schema: serde_json::Value,
        pub settings: serde_json::Value,
        pub template_count: Option<i32>,
        pub activated_at: Option<DateTime<Utc>>,
        pub installed_at: Option<DateTime<Utc>>,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }

    /// Theme option row
    #[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
    pub struct ThemeOptionRow {
        pub id: Uuid,
        pub site_id: Option<Uuid>,
        pub theme_id: String,
        pub option_name: String,
        pub option_value: Option<serde_json::Value>,
        pub option_type: Option<String>,
        pub is_default: bool,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }

    /// Theme preview session row
    #[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
    pub struct ThemePreviewRow {
        pub id: Uuid,
        pub site_id: Option<Uuid>,
        pub user_id: Uuid,
        pub theme_id: String,
        pub preview_settings: serde_json::Value,
        pub preview_token: String,
        pub expires_at: DateTime<Utc>,
        pub created_at: DateTime<Utc>,
    }

    /// Theme menu assignment row
    #[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
    pub struct ThemeMenuAssignmentRow {
        pub id: Uuid,
        pub site_id: Option<Uuid>,
        pub theme_id: String,
        pub location_slug: String,
        pub menu_id: Uuid,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }

    /// Theme widget assignment row
    #[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
    pub struct ThemeWidgetAssignmentRow {
        pub id: Uuid,
        pub site_id: Option<Uuid>,
        pub theme_id: String,
        pub area_slug: String,
        pub widget_id: Uuid,
        pub position: i32,
        pub is_active: bool,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }

    /// Theme repository for database operations
    pub struct ThemeRepository {
        pool: PgPool,
        site_id: Option<Uuid>,
    }

    impl ThemeRepository {
        pub fn new(pool: PgPool) -> Self {
            Self {
                pool,
                site_id: None,
            }
        }

        pub fn with_site(mut self, site_id: Uuid) -> Self {
            self.site_id = Some(site_id);
            self
        }

        fn site_condition(&self) -> String {
            match self.site_id {
                Some(id) => format!("site_id = '{}'", id),
                None => "site_id IS NULL".to_string(),
            }
        }

        /// Get all installed themes for a site
        pub async fn list(&self) -> Result<Vec<ThemeRow>> {
            let query = format!(
                "SELECT * FROM themes WHERE {} ORDER BY is_active DESC, name ASC",
                self.site_condition()
            );

            sqlx::query_as::<_, ThemeRow>(&query)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to list themes", e))
        }

        /// Get a theme by its theme_id (folder name)
        pub async fn find_by_theme_id(&self, theme_id: &str) -> Result<Option<ThemeRow>> {
            let query = format!(
                "SELECT * FROM themes WHERE theme_id = $1 AND {}",
                self.site_condition()
            );

            sqlx::query_as::<_, ThemeRow>(&query)
                .bind(theme_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to find theme", e))
        }

        /// Get the currently active theme
        pub async fn get_active(&self) -> Result<Option<ThemeRow>> {
            let query = format!(
                "SELECT * FROM themes WHERE is_active = true AND {}",
                self.site_condition()
            );

            sqlx::query_as::<_, ThemeRow>(&query)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to get active theme", e))
        }

        /// Install/register a new theme
        pub async fn install(&self, theme: &ThemeRow) -> Result<ThemeRow> {
            sqlx::query_as::<_, ThemeRow>(
                r#"
                INSERT INTO themes (
                    id, site_id, theme_id, name, description, version, author, author_url,
                    license, is_active, is_installed, parent_theme_id, screenshot_url,
                    homepage_url, tags, supports, menu_locations, widget_areas,
                    customizer_schema, settings, template_count, installed_at, created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, NOW(), NOW(), NOW())
                ON CONFLICT (theme_id, site_id) DO UPDATE SET
                    name = EXCLUDED.name,
                    description = EXCLUDED.description,
                    version = EXCLUDED.version,
                    author = EXCLUDED.author,
                    supports = EXCLUDED.supports,
                    menu_locations = EXCLUDED.menu_locations,
                    widget_areas = EXCLUDED.widget_areas,
                    customizer_schema = EXCLUDED.customizer_schema,
                    template_count = EXCLUDED.template_count,
                    updated_at = NOW()
                RETURNING *
                "#,
            )
            .bind(theme.id)
            .bind(theme.site_id)
            .bind(&theme.theme_id)
            .bind(&theme.name)
            .bind(&theme.description)
            .bind(&theme.version)
            .bind(&theme.author)
            .bind(&theme.author_url)
            .bind(&theme.license)
            .bind(theme.is_active)
            .bind(theme.is_installed)
            .bind(&theme.parent_theme_id)
            .bind(&theme.screenshot_url)
            .bind(&theme.homepage_url)
            .bind(&theme.tags)
            .bind(&theme.supports)
            .bind(&theme.menu_locations)
            .bind(&theme.widget_areas)
            .bind(&theme.customizer_schema)
            .bind(&theme.settings)
            .bind(theme.template_count)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to install theme", e))
        }

        /// Activate a theme (deactivates others automatically via DB trigger)
        pub async fn activate(&self, theme_id: &str) -> Result<ThemeRow> {
            let query = format!(
                r#"
                UPDATE themes
                SET is_active = true, activated_at = NOW(), updated_at = NOW()
                WHERE theme_id = $1 AND {}
                RETURNING *
                "#,
                self.site_condition()
            );

            sqlx::query_as::<_, ThemeRow>(&query)
                .bind(theme_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to activate theme", e))
        }

        /// Deactivate current theme
        pub async fn deactivate(&self, theme_id: &str) -> Result<()> {
            let query = format!(
                "UPDATE themes SET is_active = false, updated_at = NOW() WHERE theme_id = $1 AND {}",
                self.site_condition()
            );

            sqlx::query(&query)
                .bind(theme_id)
                .execute(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to deactivate theme", e))?;
            Ok(())
        }

        /// Update theme settings/customizations
        pub async fn update_settings(&self, theme_id: &str, settings: serde_json::Value) -> Result<ThemeRow> {
            let query = format!(
                r#"
                UPDATE themes
                SET settings = $2, updated_at = NOW()
                WHERE theme_id = $1 AND {}
                RETURNING *
                "#,
                self.site_condition()
            );

            sqlx::query_as::<_, ThemeRow>(&query)
                .bind(theme_id)
                .bind(settings)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to update theme settings", e))
        }

        /// Delete/uninstall a theme
        pub async fn delete(&self, theme_id: &str) -> Result<()> {
            let query = format!(
                "DELETE FROM themes WHERE theme_id = $1 AND {} AND is_active = false",
                self.site_condition()
            );

            let result = sqlx::query(&query)
                .bind(theme_id)
                .execute(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to delete theme", e))?;

            if result.rows_affected() == 0 {
                return Err(Error::validation("Cannot delete active theme or theme not found"));
            }
            Ok(())
        }

        // ============ Theme Options ============

        /// Get all options for a theme
        pub async fn get_options(&self, theme_id: &str) -> Result<Vec<ThemeOptionRow>> {
            let query = format!(
                "SELECT * FROM theme_options WHERE theme_id = $1 AND {} ORDER BY option_name",
                self.site_condition()
            );

            sqlx::query_as::<_, ThemeOptionRow>(&query)
                .bind(theme_id)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to get theme options", e))
        }

        /// Get a specific option
        pub async fn get_option(&self, theme_id: &str, option_name: &str) -> Result<Option<serde_json::Value>> {
            let query = format!(
                "SELECT option_value FROM theme_options WHERE theme_id = $1 AND option_name = $2 AND {}",
                self.site_condition()
            );

            let result: Option<(Option<serde_json::Value>,)> = sqlx::query_as(&query)
                .bind(theme_id)
                .bind(option_name)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to get theme option", e))?;

            Ok(result.and_then(|(v,)| v))
        }

        /// Set a theme option
        pub async fn set_option(&self, theme_id: &str, option_name: &str, value: serde_json::Value, option_type: Option<&str>) -> Result<()> {
            let id = Uuid::now_v7();

            sqlx::query(
                r#"
                INSERT INTO theme_options (id, site_id, theme_id, option_name, option_value, option_type, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
                ON CONFLICT (theme_id, option_name, site_id) DO UPDATE SET
                    option_value = EXCLUDED.option_value,
                    option_type = COALESCE(EXCLUDED.option_type, theme_options.option_type),
                    updated_at = NOW()
                "#,
            )
            .bind(id)
            .bind(self.site_id)
            .bind(theme_id)
            .bind(option_name)
            .bind(value)
            .bind(option_type)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to set theme option", e))?;

            Ok(())
        }

        // ============ Menu Assignments ============

        /// Get menu assignments for a theme
        pub async fn get_menu_assignments(&self, theme_id: &str) -> Result<Vec<ThemeMenuAssignmentRow>> {
            let query = format!(
                "SELECT * FROM theme_menu_assignments WHERE theme_id = $1 AND {}",
                self.site_condition()
            );

            sqlx::query_as::<_, ThemeMenuAssignmentRow>(&query)
                .bind(theme_id)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to get menu assignments", e))
        }

        /// Assign a menu to a theme location
        pub async fn assign_menu(&self, theme_id: &str, location_slug: &str, menu_id: Uuid) -> Result<()> {
            let id = Uuid::now_v7();

            sqlx::query(
                r#"
                INSERT INTO theme_menu_assignments (id, site_id, theme_id, location_slug, menu_id, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
                ON CONFLICT (site_id, theme_id, location_slug) DO UPDATE SET
                    menu_id = EXCLUDED.menu_id,
                    updated_at = NOW()
                "#,
            )
            .bind(id)
            .bind(self.site_id)
            .bind(theme_id)
            .bind(location_slug)
            .bind(menu_id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to assign menu", e))?;

            Ok(())
        }

        /// Remove a menu from a theme location
        pub async fn unassign_menu(&self, theme_id: &str, location_slug: &str) -> Result<()> {
            let query = format!(
                "DELETE FROM theme_menu_assignments WHERE theme_id = $1 AND location_slug = $2 AND {}",
                self.site_condition()
            );

            sqlx::query(&query)
                .bind(theme_id)
                .bind(location_slug)
                .execute(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to unassign menu", e))?;

            Ok(())
        }

        // ============ Widget Assignments ============

        /// Get widget assignments for a theme area
        pub async fn get_widget_assignments(&self, theme_id: &str, area_slug: Option<&str>) -> Result<Vec<ThemeWidgetAssignmentRow>> {
            let query = if let Some(area) = area_slug {
                format!(
                    "SELECT * FROM theme_widget_assignments WHERE theme_id = $1 AND area_slug = '{}' AND {} ORDER BY position",
                    area, self.site_condition()
                )
            } else {
                format!(
                    "SELECT * FROM theme_widget_assignments WHERE theme_id = $1 AND {} ORDER BY area_slug, position",
                    self.site_condition()
                )
            };

            sqlx::query_as::<_, ThemeWidgetAssignmentRow>(&query)
                .bind(theme_id)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to get widget assignments", e))
        }

        /// Assign a widget to a theme area
        pub async fn assign_widget(&self, theme_id: &str, area_slug: &str, widget_id: Uuid, position: i32) -> Result<()> {
            let id = Uuid::now_v7();

            sqlx::query(
                r#"
                INSERT INTO theme_widget_assignments (id, site_id, theme_id, area_slug, widget_id, position, is_active, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, true, NOW(), NOW())
                ON CONFLICT (site_id, theme_id, area_slug, widget_id) DO UPDATE SET
                    position = EXCLUDED.position,
                    is_active = true,
                    updated_at = NOW()
                "#,
            )
            .bind(id)
            .bind(self.site_id)
            .bind(theme_id)
            .bind(area_slug)
            .bind(widget_id)
            .bind(position)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to assign widget", e))?;

            Ok(())
        }

        // ============ Preview Sessions ============

        /// Create a preview session
        pub async fn create_preview(&self, user_id: Uuid, theme_id: &str, token: &str, expires_at: DateTime<Utc>) -> Result<ThemePreviewRow> {
            let id = Uuid::now_v7();

            sqlx::query_as::<_, ThemePreviewRow>(
                r#"
                INSERT INTO theme_previews (id, site_id, user_id, theme_id, preview_token, expires_at, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, NOW())
                ON CONFLICT (user_id, site_id) DO UPDATE SET
                    theme_id = EXCLUDED.theme_id,
                    preview_token = EXCLUDED.preview_token,
                    expires_at = EXCLUDED.expires_at
                RETURNING *
                "#,
            )
            .bind(id)
            .bind(self.site_id)
            .bind(user_id)
            .bind(theme_id)
            .bind(token)
            .bind(expires_at)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to create preview", e))
        }

        /// Get preview by token
        pub async fn get_preview_by_token(&self, token: &str) -> Result<Option<ThemePreviewRow>> {
            sqlx::query_as::<_, ThemePreviewRow>(
                "SELECT * FROM theme_previews WHERE preview_token = $1 AND expires_at > NOW()"
            )
            .bind(token)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to get preview", e))
        }

        /// Delete expired previews
        pub async fn clean_expired_previews(&self) -> Result<u64> {
            let result = sqlx::query("DELETE FROM theme_previews WHERE expires_at < NOW()")
                .execute(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to clean previews", e))?;

            Ok(result.rows_affected())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_helper_order_by() {
        assert_eq!(
            QueryHelper::order_by(Some("title"), SortOrder::Asc),
            "ORDER BY title ASC"
        );
        assert_eq!(
            QueryHelper::order_by(None, SortOrder::Desc),
            "ORDER BY created_at DESC"
        );
    }

    #[test]
    fn test_query_helper_search() {
        let condition = QueryHelper::search_condition("test", &["title", "content"]);
        assert!(condition.contains("title ILIKE"));
        assert!(condition.contains("content ILIKE"));
    }

    #[test]
    fn test_escape_like() {
        assert_eq!(QueryHelper::escape_like("test%"), "test\\%");
        assert_eq!(QueryHelper::escape_like("test_"), "test\\_");
    }
}
