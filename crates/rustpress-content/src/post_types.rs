//! Custom post types
//!
//! Allows creation of custom content types beyond standard posts and pages.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{ContentError, ContentResult};

/// Custom post type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostType {
    /// URL slug (must be unique)
    pub slug: String,

    /// Display name (plural)
    pub name: String,

    /// Display name (singular)
    pub singular_name: String,

    /// Description
    pub description: Option<String>,

    /// Is publicly visible
    pub public: bool,

    /// Supports parent/child hierarchy
    pub hierarchical: bool,

    /// Has archive page
    pub has_archive: bool,

    /// Custom URL rewrite slug
    pub rewrite_slug: Option<String>,

    /// Supported features
    pub supports: Vec<PostTypeSupport>,

    /// Associated taxonomies
    pub taxonomies: Vec<String>,

    /// Menu icon (icon name or URL)
    pub menu_icon: Option<String>,

    /// Position in admin menu
    pub menu_position: Option<i32>,

    /// Is system post type (cannot be deleted)
    pub is_system: bool,

    /// When created
    pub created_at: DateTime<Utc>,
}

/// Post type supported features
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PostTypeSupport {
    Title,
    Editor,
    Author,
    Thumbnail,
    Excerpt,
    Trackbacks,
    CustomFields,
    Comments,
    Revisions,
    PageAttributes,
    PostFormats,
}

impl PostType {
    /// Create a new post type
    pub fn new(slug: &str, name: &str, singular_name: &str) -> Self {
        Self {
            slug: slug.to_string(),
            name: name.to_string(),
            singular_name: singular_name.to_string(),
            description: None,
            public: true,
            hierarchical: false,
            has_archive: true,
            rewrite_slug: None,
            supports: vec![
                PostTypeSupport::Title,
                PostTypeSupport::Editor,
                PostTypeSupport::Author,
                PostTypeSupport::Thumbnail,
                PostTypeSupport::Excerpt,
            ],
            taxonomies: vec!["category".to_string(), "tag".to_string()],
            menu_icon: None,
            menu_position: None,
            is_system: false,
            created_at: Utc::now(),
        }
    }

    /// Create the default 'post' type
    pub fn post() -> Self {
        let mut pt = Self::new("post", "Posts", "Post");
        pt.is_system = true;
        pt.supports = vec![
            PostTypeSupport::Title,
            PostTypeSupport::Editor,
            PostTypeSupport::Author,
            PostTypeSupport::Thumbnail,
            PostTypeSupport::Excerpt,
            PostTypeSupport::Trackbacks,
            PostTypeSupport::CustomFields,
            PostTypeSupport::Comments,
            PostTypeSupport::Revisions,
            PostTypeSupport::PostFormats,
        ];
        pt
    }

    /// Create the default 'page' type
    pub fn page() -> Self {
        let mut pt = Self::new("page", "Pages", "Page");
        pt.hierarchical = true;
        pt.has_archive = false;
        pt.is_system = true;
        pt.supports = vec![
            PostTypeSupport::Title,
            PostTypeSupport::Editor,
            PostTypeSupport::Author,
            PostTypeSupport::Thumbnail,
            PostTypeSupport::Excerpt,
            PostTypeSupport::PageAttributes,
            PostTypeSupport::CustomFields,
            PostTypeSupport::Comments,
            PostTypeSupport::Revisions,
        ];
        pt.taxonomies = Vec::new();
        pt
    }

    /// Create a product post type (e-commerce)
    pub fn product() -> Self {
        let mut pt = Self::new("product", "Products", "Product");
        pt.description = Some("E-commerce products".to_string());
        pt.menu_icon = Some("cart".to_string());
        pt.taxonomies = vec!["product_category".to_string(), "product_tag".to_string()];
        pt
    }

    /// Create a portfolio post type
    pub fn portfolio() -> Self {
        let mut pt = Self::new("portfolio", "Portfolio", "Portfolio Item");
        pt.description = Some("Portfolio showcase items".to_string());
        pt.menu_icon = Some("grid".to_string());
        pt.taxonomies = vec!["portfolio_category".to_string()];
        pt
    }

    /// Create a testimonial post type
    pub fn testimonial() -> Self {
        let mut pt = Self::new("testimonial", "Testimonials", "Testimonial");
        pt.description = Some("Customer testimonials".to_string());
        pt.menu_icon = Some("quote".to_string());
        pt.has_archive = false;
        pt.taxonomies = Vec::new();
        pt.supports = vec![
            PostTypeSupport::Title,
            PostTypeSupport::Editor,
            PostTypeSupport::Thumbnail,
            PostTypeSupport::CustomFields,
        ];
        pt
    }

    /// Create an event post type
    pub fn event() -> Self {
        let mut pt = Self::new("event", "Events", "Event");
        pt.description = Some("Calendar events".to_string());
        pt.menu_icon = Some("calendar".to_string());
        pt.taxonomies = vec!["event_category".to_string()];
        pt
    }

    /// Create a FAQ post type
    pub fn faq() -> Self {
        let mut pt = Self::new("faq", "FAQs", "FAQ");
        pt.description = Some("Frequently asked questions".to_string());
        pt.menu_icon = Some("help-circle".to_string());
        pt.has_archive = false;
        pt.taxonomies = vec!["faq_category".to_string()];
        pt.supports = vec![
            PostTypeSupport::Title,
            PostTypeSupport::Editor,
            PostTypeSupport::CustomFields,
        ];
        pt
    }

    /// Validate post type
    pub fn validate(&self) -> ContentResult<()> {
        // Validate slug format
        let slug_regex = regex::Regex::new(r"^[a-z][a-z0-9_-]*$").unwrap();
        if !slug_regex.is_match(&self.slug) {
            return Err(ContentError::Validation(
                "Post type slug must start with a letter and contain only lowercase letters, numbers, underscores, and hyphens".to_string(),
            ));
        }

        if self.slug.len() > 20 {
            return Err(ContentError::Validation(
                "Post type slug must be 20 characters or less".to_string(),
            ));
        }

        if self.name.is_empty() {
            return Err(ContentError::Validation(
                "Post type name is required".to_string(),
            ));
        }

        if self.singular_name.is_empty() {
            return Err(ContentError::Validation(
                "Post type singular name is required".to_string(),
            ));
        }

        // Reserved slugs
        let reserved = ["attachment", "revision", "nav_menu_item", "custom_css", "customize_changeset"];
        if reserved.contains(&self.slug.as_str()) {
            return Err(ContentError::Validation(
                format!("Post type slug '{}' is reserved", self.slug),
            ));
        }

        Ok(())
    }

    /// Get URL path for this post type
    pub fn get_url_path(&self) -> String {
        self.rewrite_slug.clone().unwrap_or_else(|| self.slug.clone())
    }

    /// Check if post type supports a feature
    pub fn has_support(&self, feature: PostTypeSupport) -> bool {
        self.supports.contains(&feature)
    }
}

/// Post type service
pub struct PostTypeService {
    pool: sqlx::PgPool,
}

impl PostTypeService {
    /// Create new service
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// Initialize system post types
    pub async fn init_system_types(&self) -> ContentResult<()> {
        let types = vec![PostType::post(), PostType::page()];

        for pt in types {
            sqlx::query(
                r#"
                INSERT INTO post_types (
                    slug, name, singular_name, description, public, hierarchical,
                    has_archive, rewrite_slug, supports, taxonomies, menu_icon,
                    menu_position, is_system, created_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
                ON CONFLICT (slug) DO NOTHING
                "#,
            )
            .bind(&pt.slug)
            .bind(&pt.name)
            .bind(&pt.singular_name)
            .bind(&pt.description)
            .bind(pt.public)
            .bind(pt.hierarchical)
            .bind(pt.has_archive)
            .bind(&pt.rewrite_slug)
            .bind(serde_json::to_value(&pt.supports)?)
            .bind(serde_json::to_value(&pt.taxonomies)?)
            .bind(&pt.menu_icon)
            .bind(pt.menu_position)
            .bind(pt.is_system)
            .bind(pt.created_at)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Register a new post type
    pub async fn register(&self, post_type: PostType) -> ContentResult<PostType> {
        post_type.validate()?;

        sqlx::query(
            r#"
            INSERT INTO post_types (
                slug, name, singular_name, description, public, hierarchical,
                has_archive, rewrite_slug, supports, taxonomies, menu_icon,
                menu_position, is_system, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
        )
        .bind(&post_type.slug)
        .bind(&post_type.name)
        .bind(&post_type.singular_name)
        .bind(&post_type.description)
        .bind(post_type.public)
        .bind(post_type.hierarchical)
        .bind(post_type.has_archive)
        .bind(&post_type.rewrite_slug)
        .bind(serde_json::to_value(&post_type.supports)?)
        .bind(serde_json::to_value(&post_type.taxonomies)?)
        .bind(&post_type.menu_icon)
        .bind(post_type.menu_position)
        .bind(post_type.is_system)
        .bind(post_type.created_at)
        .execute(&self.pool)
        .await?;

        Ok(post_type)
    }

    /// Get post type by slug
    pub async fn get(&self, slug: &str) -> ContentResult<PostType> {
        let row = sqlx::query_as::<_, PostTypeRow>(
            "SELECT * FROM post_types WHERE slug = $1",
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ContentError::NotFound(slug.to_string()))?;

        row.try_into()
    }

    /// List all post types
    pub async fn list(&self) -> ContentResult<Vec<PostType>> {
        let rows = sqlx::query_as::<_, PostTypeRow>(
            "SELECT * FROM post_types ORDER BY menu_position, name",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    /// List public post types
    pub async fn list_public(&self) -> ContentResult<Vec<PostType>> {
        let rows = sqlx::query_as::<_, PostTypeRow>(
            "SELECT * FROM post_types WHERE public = true ORDER BY menu_position, name",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    /// Update post type
    pub async fn update(&self, post_type: PostType) -> ContentResult<PostType> {
        if post_type.is_system {
            return Err(ContentError::PermissionDenied(
                "Cannot modify system post types".to_string(),
            ));
        }

        post_type.validate()?;

        sqlx::query(
            r#"
            UPDATE post_types SET
                name = $2, singular_name = $3, description = $4, public = $5,
                hierarchical = $6, has_archive = $7, rewrite_slug = $8,
                supports = $9, taxonomies = $10, menu_icon = $11, menu_position = $12
            WHERE slug = $1 AND is_system = false
            "#,
        )
        .bind(&post_type.slug)
        .bind(&post_type.name)
        .bind(&post_type.singular_name)
        .bind(&post_type.description)
        .bind(post_type.public)
        .bind(post_type.hierarchical)
        .bind(post_type.has_archive)
        .bind(&post_type.rewrite_slug)
        .bind(serde_json::to_value(&post_type.supports)?)
        .bind(serde_json::to_value(&post_type.taxonomies)?)
        .bind(&post_type.menu_icon)
        .bind(post_type.menu_position)
        .execute(&self.pool)
        .await?;

        Ok(post_type)
    }

    /// Unregister (delete) post type
    pub async fn unregister(&self, slug: &str) -> ContentResult<()> {
        let result = sqlx::query(
            "DELETE FROM post_types WHERE slug = $1 AND is_system = false",
        )
        .bind(slug)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(ContentError::PermissionDenied(
                "Cannot delete system post types".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if post type exists
    pub async fn exists(&self, slug: &str) -> ContentResult<bool> {
        let result: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM post_types WHERE slug = $1)",
        )
        .bind(slug)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.0)
    }
}

/// Database row
#[derive(Debug, sqlx::FromRow)]
struct PostTypeRow {
    slug: String,
    name: String,
    singular_name: String,
    description: Option<String>,
    public: bool,
    hierarchical: bool,
    has_archive: bool,
    rewrite_slug: Option<String>,
    supports: serde_json::Value,
    taxonomies: serde_json::Value,
    menu_icon: Option<String>,
    menu_position: Option<i32>,
    is_system: bool,
    created_at: DateTime<Utc>,
}

impl TryFrom<PostTypeRow> for PostType {
    type Error = ContentError;

    fn try_from(row: PostTypeRow) -> Result<Self, Self::Error> {
        Ok(Self {
            slug: row.slug,
            name: row.name,
            singular_name: row.singular_name,
            description: row.description,
            public: row.public,
            hierarchical: row.hierarchical,
            has_archive: row.has_archive,
            rewrite_slug: row.rewrite_slug,
            supports: serde_json::from_value(row.supports)?,
            taxonomies: serde_json::from_value(row.taxonomies)?,
            menu_icon: row.menu_icon,
            menu_position: row.menu_position,
            is_system: row.is_system,
            created_at: row.created_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_post_type() {
        let pt = PostType::new("review", "Reviews", "Review");
        assert_eq!(pt.slug, "review");
        assert!(!pt.is_system);
    }

    #[test]
    fn test_validate_post_type() {
        let pt = PostType::new("valid_slug", "Valid", "Valid");
        assert!(pt.validate().is_ok());

        let mut invalid = PostType::new("Invalid", "Invalid", "Invalid");
        invalid.slug = "Invalid-Slug".to_string();
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_reserved_slugs() {
        let mut pt = PostType::new("attachment", "Attachments", "Attachment");
        assert!(pt.validate().is_err());
    }

    #[test]
    fn test_has_support() {
        let pt = PostType::post();
        assert!(pt.has_support(PostTypeSupport::Title));
        assert!(pt.has_support(PostTypeSupport::Editor));
    }
}
