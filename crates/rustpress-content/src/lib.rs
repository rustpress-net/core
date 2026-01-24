//! RustPress Content Management System
//!
//! This crate provides comprehensive content management features:
//! - Visual block editor (Gutenberg-style)
//! - Elementor compatibility layer
//! - Markdown editor with live preview
//! - Content versioning and revision history
//! - Scheduled publishing
//! - Draft autosave
//! - Content templates
//! - Custom post types
//! - Taxonomy management

pub mod access;
pub mod autosave;
pub mod blocks;
pub mod bulk;
pub mod elementor;
pub mod excerpt;
pub mod featured;
pub mod fields;
pub mod i18n;
pub mod markdown;
pub mod media;
pub mod oembed;
pub mod post_types;
pub mod related;
pub mod revision;
pub mod sanitize;
pub mod scheduler;
pub mod shortcode;
pub mod taxonomy;
pub mod templates;
pub mod toc;
pub mod trash;
pub mod versioning;
pub mod workflow;
pub mod wxr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub use access::*;
pub use autosave::*;
pub use blocks::*;
pub use bulk::*;
pub use elementor::*;
pub use excerpt::*;
pub use featured::*;
pub use fields::*;
pub use i18n::*;
pub use markdown::*;
pub use media::*;
pub use oembed::*;
pub use post_types::*;
pub use related::*;
pub use revision::*;
pub use sanitize::*;
pub use scheduler::*;
pub use shortcode::*;
pub use taxonomy::*;
pub use templates::*;
pub use toc::*;
pub use trash::*;
pub use versioning::*;
pub use workflow::*;
pub use wxr::*;

/// Content management errors
#[derive(Debug, Error)]
pub enum ContentError {
    #[error("Content not found: {0}")]
    NotFound(String),

    #[error("Invalid content: {0}")]
    Invalid(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Version conflict: {0}")]
    VersionConflict(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Template error: {0}")]
    Template(String),

    #[error("Scheduler error: {0}")]
    Scheduler(String),
}

pub type ContentResult<T> = Result<T, ContentError>;

/// Content status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContentStatus {
    Draft,
    PendingReview,
    Scheduled,
    Published,
    Private,
    Archived,
    Trash,
}

impl Default for ContentStatus {
    fn default() -> Self {
        Self::Draft
    }
}

/// Content format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContentFormat {
    /// Block-based content (Gutenberg-style)
    Blocks,
    /// Elementor page builder format
    Elementor,
    /// Plain markdown
    Markdown,
    /// Raw HTML
    Html,
    /// Plain text
    Text,
}

impl Default for ContentFormat {
    fn default() -> Self {
        Self::Blocks
    }
}

/// Core content structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    /// Unique identifier
    pub id: Uuid,

    /// Post type (post, page, custom)
    pub post_type: String,

    /// Content title
    pub title: String,

    /// URL slug
    pub slug: String,

    /// Content body (format depends on content_format)
    pub content: String,

    /// Parsed/rendered content blocks
    #[serde(default)]
    pub blocks: Vec<Block>,

    /// Content format
    pub format: ContentFormat,

    /// Excerpt/summary
    pub excerpt: Option<String>,

    /// Featured image ID
    pub featured_image: Option<Uuid>,

    /// Current status
    pub status: ContentStatus,

    /// Author user ID
    pub author_id: Uuid,

    /// Parent content ID (for hierarchical content)
    pub parent_id: Option<Uuid>,

    /// Menu order for sorting
    pub menu_order: i32,

    /// Allow comments
    pub comment_status: bool,

    /// Allow pingbacks
    pub ping_status: bool,

    /// Custom fields/meta
    #[serde(default)]
    pub meta: serde_json::Value,

    /// Template name
    pub template: Option<String>,

    /// Current revision number
    pub revision: i32,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Publication timestamp
    pub published_at: Option<DateTime<Utc>>,

    /// Scheduled publication time
    pub scheduled_at: Option<DateTime<Utc>>,

    /// Taxonomy terms (category IDs, tag IDs, etc.)
    #[serde(default)]
    pub terms: Vec<Uuid>,
}

impl Content {
    /// Create new content with just post type (uses defaults for other fields)
    pub fn new(post_type: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            post_type: post_type.to_string(),
            title: String::new(),
            slug: String::new(),
            content: String::new(),
            blocks: Vec::new(),
            format: ContentFormat::default(),
            excerpt: None,
            featured_image: None,
            status: ContentStatus::Draft,
            author_id: Uuid::nil(),
            parent_id: None,
            menu_order: 0,
            comment_status: true,
            ping_status: true,
            meta: serde_json::json!({}),
            template: None,
            revision: 1,
            created_at: now,
            updated_at: now,
            published_at: None,
            scheduled_at: None,
            terms: Vec::new(),
        }
    }

    /// Create new content with title and author
    pub fn with_author_and_title(post_type: &str, title: &str, author_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            post_type: post_type.to_string(),
            title: title.to_string(),
            slug: slug::slugify(title),
            content: String::new(),
            blocks: Vec::new(),
            format: ContentFormat::default(),
            excerpt: None,
            featured_image: None,
            status: ContentStatus::Draft,
            author_id,
            parent_id: None,
            menu_order: 0,
            comment_status: true,
            ping_status: true,
            meta: serde_json::json!({}),
            template: None,
            revision: 1,
            created_at: now,
            updated_at: now,
            published_at: None,
            scheduled_at: None,
            terms: Vec::new(),
        }
    }

    /// Builder: set title
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        if self.slug.is_empty() {
            self.slug = slug::slugify(title);
        }
        self
    }

    /// Builder: set slug
    pub fn with_slug(mut self, slug: &str) -> Self {
        self.slug = slug.to_string();
        self
    }

    /// Builder: set content
    pub fn with_content(mut self, content: &str) -> Self {
        self.content = content.to_string();
        self
    }

    /// Builder: set author
    pub fn with_author(mut self, author_id: Uuid) -> Self {
        self.author_id = author_id;
        self
    }

    /// Builder: set parent
    pub fn with_parent(mut self, parent_id: Uuid) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Builder: set excerpt
    pub fn with_excerpt(mut self, excerpt: &str) -> Self {
        self.excerpt = Some(excerpt.to_string());
        self
    }

    /// Builder: set blocks
    pub fn with_blocks(mut self, blocks: Vec<Block>) -> Self {
        self.blocks = blocks;
        self
    }

    /// Builder: set format
    pub fn with_format(mut self, format: ContentFormat) -> Self {
        self.format = format;
        self
    }

    /// Builder: set status
    pub fn with_status(mut self, status: ContentStatus) -> Self {
        self.status = status;
        self
    }

    /// Check if content is published
    pub fn is_published(&self) -> bool {
        self.status == ContentStatus::Published
    }

    /// Check if content is scheduled
    pub fn is_scheduled(&self) -> bool {
        self.status == ContentStatus::Scheduled && self.scheduled_at.is_some()
    }

    /// Check if scheduled content should be published
    pub fn should_publish(&self) -> bool {
        if let Some(scheduled) = self.scheduled_at {
            self.status == ContentStatus::Scheduled && scheduled <= Utc::now()
        } else {
            false
        }
    }

    /// Publish content
    pub fn publish(&mut self) {
        self.status = ContentStatus::Published;
        self.published_at = Some(Utc::now());
        self.scheduled_at = None;
        self.updated_at = Utc::now();
    }

    /// Schedule content for publication
    pub fn schedule(&mut self, publish_at: DateTime<Utc>) {
        self.status = ContentStatus::Scheduled;
        self.scheduled_at = Some(publish_at);
        self.updated_at = Utc::now();
    }

    /// Move to trash
    pub fn trash(&mut self) {
        self.status = ContentStatus::Trash;
        self.updated_at = Utc::now();
    }

    /// Render content to HTML
    pub fn render_html(&self) -> String {
        match self.format {
            ContentFormat::Blocks => BlockRenderer::new().render_blocks(&self.blocks),
            ContentFormat::Elementor => ElementorRenderer::new().render(&self.content),
            ContentFormat::Markdown => MarkdownProcessor::new().to_html(&self.content),
            ContentFormat::Html => self.content.clone(),
            ContentFormat::Text => {
                format!("<p>{}</p>", ammonia::clean(&self.content))
            }
        }
    }

    /// Generate excerpt from content
    pub fn generate_excerpt(&self, max_length: usize) -> String {
        if let Some(ref excerpt) = self.excerpt {
            if !excerpt.is_empty() {
                return excerpt.clone();
            }
        }

        // Strip HTML and truncate
        let text = ammonia::clean(&self.render_html());
        let text = text.replace('\n', " ").trim().to_string();

        if text.len() <= max_length {
            text
        } else {
            let truncated: String = text.chars().take(max_length).collect();
            format!("{}...", truncated.trim_end())
        }
    }
}

/// Content service for managing content operations
pub struct ContentService {
    pool: sqlx::PgPool,
    versioning: VersioningService,
    scheduler: scheduler::PublishScheduler,
    autosave: AutosaveService,
}

impl ContentService {
    /// Create new content service
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            pool: pool.clone(),
            versioning: VersioningService::new(pool.clone()),
            scheduler: scheduler::PublishScheduler::new(pool.clone()),
            autosave: AutosaveService::new(pool),
        }
    }

    /// Create new content
    pub async fn create(&self, content: Content) -> ContentResult<Content> {
        // Validate content
        self.validate(&content)?;

        // Save to database
        sqlx::query(
            r#"
            INSERT INTO contents (
                id, post_type, title, slug, content, blocks, format,
                excerpt, featured_image, status, author_id, parent_id,
                menu_order, comment_status, ping_status, meta, template,
                revision, created_at, updated_at, published_at, scheduled_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12,
                $13, $14, $15, $16, $17, $18, $19, $20, $21, $22
            )
            "#,
        )
        .bind(content.id)
        .bind(&content.post_type)
        .bind(&content.title)
        .bind(&content.slug)
        .bind(&content.content)
        .bind(serde_json::to_value(&content.blocks)?)
        .bind(serde_json::to_string(&content.format)?)
        .bind(&content.excerpt)
        .bind(content.featured_image)
        .bind(serde_json::to_string(&content.status)?)
        .bind(content.author_id)
        .bind(content.parent_id)
        .bind(content.menu_order)
        .bind(content.comment_status)
        .bind(content.ping_status)
        .bind(&content.meta)
        .bind(&content.template)
        .bind(content.revision)
        .bind(content.created_at)
        .bind(content.updated_at)
        .bind(content.published_at)
        .bind(content.scheduled_at)
        .execute(&self.pool)
        .await?;

        // Create initial revision
        self.versioning.create_revision(&content).await?;

        Ok(content)
    }

    /// Update existing content
    pub async fn update(&self, mut content: Content) -> ContentResult<Content> {
        // Validate content
        self.validate(&content)?;

        // Increment revision
        content.revision += 1;
        content.updated_at = Utc::now();

        // Save to database
        sqlx::query(
            r#"
            UPDATE contents SET
                title = $2, slug = $3, content = $4, blocks = $5,
                format = $6, excerpt = $7, featured_image = $8,
                status = $9, parent_id = $10, menu_order = $11,
                comment_status = $12, ping_status = $13, meta = $14,
                template = $15, revision = $16, updated_at = $17,
                published_at = $18, scheduled_at = $19
            WHERE id = $1
            "#,
        )
        .bind(content.id)
        .bind(&content.title)
        .bind(&content.slug)
        .bind(&content.content)
        .bind(serde_json::to_value(&content.blocks)?)
        .bind(serde_json::to_string(&content.format)?)
        .bind(&content.excerpt)
        .bind(content.featured_image)
        .bind(serde_json::to_string(&content.status)?)
        .bind(content.parent_id)
        .bind(content.menu_order)
        .bind(content.comment_status)
        .bind(content.ping_status)
        .bind(&content.meta)
        .bind(&content.template)
        .bind(content.revision)
        .bind(content.updated_at)
        .bind(content.published_at)
        .bind(content.scheduled_at)
        .execute(&self.pool)
        .await?;

        // Create new revision
        self.versioning.create_revision(&content).await?;

        Ok(content)
    }

    /// Get content by ID
    pub async fn get(&self, id: Uuid) -> ContentResult<Content> {
        let row = sqlx::query_as::<_, ContentRow>("SELECT * FROM contents WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| ContentError::NotFound(id.to_string()))?;

        row.into_content()
    }

    /// Get content by slug
    pub async fn get_by_slug(&self, slug: &str, post_type: &str) -> ContentResult<Content> {
        let row = sqlx::query_as::<_, ContentRow>(
            "SELECT * FROM contents WHERE slug = $1 AND post_type = $2",
        )
        .bind(slug)
        .bind(post_type)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ContentError::NotFound(slug.to_string()))?;

        row.into_content()
    }

    /// List content with filters
    pub async fn list(&self, filter: ContentFilter) -> ContentResult<Vec<Content>> {
        let mut query = String::from("SELECT * FROM contents WHERE 1=1");
        let mut params: Vec<String> = Vec::new();

        if let Some(ref post_type) = filter.post_type {
            params.push(post_type.clone());
            query.push_str(&format!(" AND post_type = ${}", params.len()));
        }

        if let Some(ref status) = filter.status {
            params.push(serde_json::to_string(status)?);
            query.push_str(&format!(" AND status = ${}", params.len()));
        }

        if let Some(author_id) = filter.author_id {
            params.push(author_id.to_string());
            query.push_str(&format!(" AND author_id = ${}", params.len()));
        }

        query.push_str(&format!(
            " ORDER BY {} {} LIMIT {} OFFSET {}",
            filter.order_by.unwrap_or_else(|| "created_at".to_string()),
            if filter.order_desc.unwrap_or(true) {
                "DESC"
            } else {
                "ASC"
            },
            filter.limit.unwrap_or(20),
            filter.offset.unwrap_or(0)
        ));

        // Note: This is a simplified implementation. In production,
        // use proper parameterized queries with sqlx::query_builder
        let rows = sqlx::query_as::<_, ContentRow>(&query)
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter().map(|r| r.into_content()).collect()
    }

    /// Delete content
    pub async fn delete(&self, id: Uuid) -> ContentResult<()> {
        sqlx::query("DELETE FROM contents WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Validate content
    fn validate(&self, content: &Content) -> ContentResult<()> {
        if content.title.is_empty() {
            return Err(ContentError::Validation("Title is required".to_string()));
        }

        if content.slug.is_empty() {
            return Err(ContentError::Validation("Slug is required".to_string()));
        }

        // Validate slug format
        let slug_regex = regex::Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").unwrap();
        if !slug_regex.is_match(&content.slug) {
            return Err(ContentError::Validation(
                "Invalid slug format. Use lowercase letters, numbers, and hyphens only."
                    .to_string(),
            ));
        }

        Ok(())
    }

    /// Get versioning service
    pub fn versioning(&self) -> &VersioningService {
        &self.versioning
    }

    /// Get scheduler
    pub fn scheduler(&self) -> &scheduler::PublishScheduler {
        &self.scheduler
    }

    /// Get autosave service
    pub fn autosave(&self) -> &AutosaveService {
        &self.autosave
    }
}

/// Content filter for listing
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ContentFilter {
    pub post_type: Option<String>,
    pub status: Option<ContentStatus>,
    pub author_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub search: Option<String>,
    pub term_ids: Option<Vec<Uuid>>,
    pub order_by: Option<String>,
    pub order_desc: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Database row representation
#[derive(Debug, sqlx::FromRow)]
struct ContentRow {
    id: Uuid,
    post_type: String,
    title: String,
    slug: String,
    content: String,
    blocks: serde_json::Value,
    format: String,
    excerpt: Option<String>,
    featured_image: Option<Uuid>,
    status: String,
    author_id: Uuid,
    parent_id: Option<Uuid>,
    menu_order: i32,
    comment_status: bool,
    ping_status: bool,
    meta: serde_json::Value,
    template: Option<String>,
    revision: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    published_at: Option<DateTime<Utc>>,
    scheduled_at: Option<DateTime<Utc>>,
}

impl ContentRow {
    fn into_content(self) -> ContentResult<Content> {
        Ok(Content {
            id: self.id,
            post_type: self.post_type,
            title: self.title,
            slug: self.slug,
            content: self.content,
            blocks: serde_json::from_value(self.blocks)?,
            format: serde_json::from_str(&self.format)?,
            excerpt: self.excerpt,
            featured_image: self.featured_image,
            status: serde_json::from_str(&self.status)?,
            author_id: self.author_id,
            parent_id: self.parent_id,
            menu_order: self.menu_order,
            comment_status: self.comment_status,
            ping_status: self.ping_status,
            meta: self.meta,
            template: self.template,
            revision: self.revision,
            created_at: self.created_at,
            updated_at: self.updated_at,
            published_at: self.published_at,
            scheduled_at: self.scheduled_at,
            terms: Vec::new(), // Loaded separately
        })
    }
}

/// SQL migrations for content tables
pub const CONTENT_MIGRATIONS: &str = r#"
-- Contents table
CREATE TABLE IF NOT EXISTS contents (
    id UUID PRIMARY KEY,
    post_type VARCHAR(50) NOT NULL DEFAULT 'post',
    title VARCHAR(500) NOT NULL,
    slug VARCHAR(500) NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    blocks JSONB NOT NULL DEFAULT '[]',
    format VARCHAR(20) NOT NULL DEFAULT 'blocks',
    excerpt TEXT,
    featured_image UUID,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    author_id UUID NOT NULL,
    parent_id UUID REFERENCES contents(id) ON DELETE SET NULL,
    menu_order INTEGER NOT NULL DEFAULT 0,
    comment_status BOOLEAN NOT NULL DEFAULT true,
    ping_status BOOLEAN NOT NULL DEFAULT true,
    meta JSONB NOT NULL DEFAULT '{}',
    template VARCHAR(100),
    revision INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at TIMESTAMPTZ,
    scheduled_at TIMESTAMPTZ,
    UNIQUE(slug, post_type)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_contents_post_type ON contents(post_type);
CREATE INDEX IF NOT EXISTS idx_contents_status ON contents(status);
CREATE INDEX IF NOT EXISTS idx_contents_author ON contents(author_id);
CREATE INDEX IF NOT EXISTS idx_contents_slug ON contents(slug);
CREATE INDEX IF NOT EXISTS idx_contents_created ON contents(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_contents_published ON contents(published_at DESC);
CREATE INDEX IF NOT EXISTS idx_contents_scheduled ON contents(scheduled_at) WHERE scheduled_at IS NOT NULL;

-- Full text search
CREATE INDEX IF NOT EXISTS idx_contents_search ON contents USING gin(
    to_tsvector('english', title || ' ' || content)
);

-- Content revisions
CREATE TABLE IF NOT EXISTS content_revisions (
    id UUID PRIMARY KEY,
    content_id UUID NOT NULL REFERENCES contents(id) ON DELETE CASCADE,
    revision INTEGER NOT NULL,
    title VARCHAR(500) NOT NULL,
    content TEXT NOT NULL,
    blocks JSONB NOT NULL DEFAULT '[]',
    author_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    change_summary TEXT,
    UNIQUE(content_id, revision)
);

CREATE INDEX IF NOT EXISTS idx_revisions_content ON content_revisions(content_id, revision DESC);

-- Autosaves
CREATE TABLE IF NOT EXISTS content_autosaves (
    id UUID PRIMARY KEY,
    content_id UUID REFERENCES contents(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    title VARCHAR(500) NOT NULL,
    content TEXT NOT NULL,
    blocks JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(content_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_autosaves_content ON content_autosaves(content_id);
CREATE INDEX IF NOT EXISTS idx_autosaves_user ON content_autosaves(user_id);

-- Content templates
CREATE TABLE IF NOT EXISTS content_templates (
    id UUID PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    post_type VARCHAR(50) NOT NULL DEFAULT 'post',
    content TEXT NOT NULL DEFAULT '',
    blocks JSONB NOT NULL DEFAULT '[]',
    format VARCHAR(20) NOT NULL DEFAULT 'blocks',
    meta JSONB NOT NULL DEFAULT '{}',
    is_system BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Custom post types
CREATE TABLE IF NOT EXISTS post_types (
    slug VARCHAR(50) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    singular_name VARCHAR(100) NOT NULL,
    description TEXT,
    public BOOLEAN NOT NULL DEFAULT true,
    hierarchical BOOLEAN NOT NULL DEFAULT false,
    has_archive BOOLEAN NOT NULL DEFAULT true,
    rewrite_slug VARCHAR(100),
    supports JSONB NOT NULL DEFAULT '["title", "editor", "author", "thumbnail", "excerpt"]',
    taxonomies JSONB NOT NULL DEFAULT '["category", "tag"]',
    menu_icon VARCHAR(100),
    menu_position INTEGER,
    is_system BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert default post types
INSERT INTO post_types (slug, name, singular_name, is_system) VALUES
    ('post', 'Posts', 'Post', true),
    ('page', 'Pages', 'Page', true)
ON CONFLICT (slug) DO NOTHING;

-- Taxonomies
CREATE TABLE IF NOT EXISTS taxonomies (
    slug VARCHAR(50) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    singular_name VARCHAR(100) NOT NULL,
    description TEXT,
    hierarchical BOOLEAN NOT NULL DEFAULT false,
    public BOOLEAN NOT NULL DEFAULT true,
    show_in_menu BOOLEAN NOT NULL DEFAULT true,
    rewrite_slug VARCHAR(100),
    post_types JSONB NOT NULL DEFAULT '["post"]',
    is_system BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert default taxonomies
INSERT INTO taxonomies (slug, name, singular_name, hierarchical, is_system) VALUES
    ('category', 'Categories', 'Category', true, true),
    ('tag', 'Tags', 'Tag', false, true)
ON CONFLICT (slug) DO NOTHING;

-- Terms (categories, tags, etc.)
CREATE TABLE IF NOT EXISTS terms (
    id UUID PRIMARY KEY,
    taxonomy VARCHAR(50) NOT NULL REFERENCES taxonomies(slug) ON DELETE CASCADE,
    name VARCHAR(200) NOT NULL,
    slug VARCHAR(200) NOT NULL,
    description TEXT,
    parent_id UUID REFERENCES terms(id) ON DELETE SET NULL,
    meta JSONB NOT NULL DEFAULT '{}',
    count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(taxonomy, slug)
);

CREATE INDEX IF NOT EXISTS idx_terms_taxonomy ON terms(taxonomy);
CREATE INDEX IF NOT EXISTS idx_terms_parent ON terms(parent_id);

-- Content-Term relationships
CREATE TABLE IF NOT EXISTS content_terms (
    content_id UUID NOT NULL REFERENCES contents(id) ON DELETE CASCADE,
    term_id UUID NOT NULL REFERENCES terms(id) ON DELETE CASCADE,
    PRIMARY KEY (content_id, term_id)
);

CREATE INDEX IF NOT EXISTS idx_content_terms_term ON content_terms(term_id);

-- Scheduled publish queue
CREATE TABLE IF NOT EXISTS scheduled_publishes (
    id UUID PRIMARY KEY,
    content_id UUID NOT NULL REFERENCES contents(id) ON DELETE CASCADE,
    scheduled_at TIMESTAMPTZ NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    attempts INTEGER NOT NULL DEFAULT 0,
    last_attempt TIMESTAMPTZ,
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_scheduled_pending ON scheduled_publishes(scheduled_at)
    WHERE status = 'pending';
"#;
