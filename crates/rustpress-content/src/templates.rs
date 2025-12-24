//! Content templates
//!
//! Provides reusable content templates for quick content creation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Block, ContentError, ContentFormat, ContentResult};

/// Content template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentTemplate {
    /// Template ID
    pub id: Uuid,

    /// Template name
    pub name: String,

    /// Template description
    pub description: Option<String>,

    /// Post type this template applies to
    pub post_type: String,

    /// Template content
    pub content: String,

    /// Template blocks
    pub blocks: Vec<Block>,

    /// Content format
    pub format: ContentFormat,

    /// Template metadata
    pub meta: serde_json::Value,

    /// Is system template (cannot be deleted)
    pub is_system: bool,

    /// When template was created
    pub created_at: DateTime<Utc>,

    /// When template was last updated
    pub updated_at: DateTime<Utc>,
}

impl ContentTemplate {
    /// Create a new template
    pub fn new(name: &str, post_type: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: None,
            post_type: post_type.to_string(),
            content: String::new(),
            blocks: Vec::new(),
            format: ContentFormat::Blocks,
            meta: serde_json::json!({}),
            is_system: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create blank template
    pub fn blank() -> Self {
        Self::new("Blank", "post")
    }

    /// Create blog post template
    pub fn blog_post() -> Self {
        let mut template = Self::new("Blog Post", "post");
        template.description = Some("Standard blog post with introduction, body, and conclusion".to_string());
        template.blocks = vec![
            Block::heading("Post Title", 1),
            Block::paragraph("Introduction paragraph - hook your readers here..."),
            Block::separator(None),
            Block::heading("Main Section", 2),
            Block::paragraph("Main content goes here..."),
            Block::heading("Conclusion", 2),
            Block::paragraph("Wrap up your thoughts..."),
        ];
        template.is_system = true;
        template
    }

    /// Create landing page template
    pub fn landing_page() -> Self {
        let mut template = Self::new("Landing Page", "page");
        template.description = Some("Marketing landing page with hero, features, and CTA".to_string());
        template.blocks = vec![
            Block::cover("/placeholder-hero.jpg", "primary", vec![
                Block::heading("Welcome to Our Site", 1),
                Block::paragraph("Your compelling tagline goes here"),
                Block::button("Get Started", "#cta", Some("is-style-fill")),
            ]),
            Block::spacer(60),
            Block::heading("Features", 2),
            Block::columns(vec![
                Block::column(vec![
                    Block::heading("Feature 1", 3),
                    Block::paragraph("Description of feature 1"),
                ]),
                Block::column(vec![
                    Block::heading("Feature 2", 3),
                    Block::paragraph("Description of feature 2"),
                ]),
                Block::column(vec![
                    Block::heading("Feature 3", 3),
                    Block::paragraph("Description of feature 3"),
                ]),
            ]),
            Block::spacer(60),
            Block::group(vec![
                Block::heading("Ready to get started?", 2),
                Block::button("Sign Up Now", "#signup", Some("is-style-fill")),
            ], Some("constrained")),
        ];
        template.is_system = true;
        template
    }

    /// Create about page template
    pub fn about_page() -> Self {
        let mut template = Self::new("About Page", "page");
        template.description = Some("Company or personal about page".to_string());
        template.blocks = vec![
            Block::heading("About Us", 1),
            Block::paragraph("Tell your story here..."),
            Block::heading("Our Mission", 2),
            Block::paragraph("What drives us..."),
            Block::heading("Our Team", 2),
            Block::columns(vec![
                Block::column(vec![
                    Block::image("/placeholder-team-1.jpg", "Team member 1", None),
                    Block::heading("Team Member 1", 3),
                    Block::paragraph("Role description"),
                ]),
                Block::column(vec![
                    Block::image("/placeholder-team-2.jpg", "Team member 2", None),
                    Block::heading("Team Member 2", 3),
                    Block::paragraph("Role description"),
                ]),
            ]),
        ];
        template.is_system = true;
        template
    }

    /// Create contact page template
    pub fn contact_page() -> Self {
        let mut template = Self::new("Contact Page", "page");
        template.description = Some("Contact information and form".to_string());
        template.blocks = vec![
            Block::heading("Contact Us", 1),
            Block::paragraph("We'd love to hear from you!"),
            Block::columns(vec![
                Block::column(vec![
                    Block::heading("Get in Touch", 3),
                    Block::paragraph("Email: contact@example.com"),
                    Block::paragraph("Phone: (555) 123-4567"),
                    Block::paragraph("Address: 123 Main St, City, State"),
                ]),
                Block::column(vec![
                    Block::heading("Send a Message", 3),
                    // Form would go here
                    Block::paragraph("[Contact form placeholder]"),
                ]),
            ]),
        ];
        template.is_system = true;
        template
    }

    /// Create portfolio item template
    pub fn portfolio_item() -> Self {
        let mut template = Self::new("Portfolio Item", "portfolio");
        template.description = Some("Showcase a portfolio project".to_string());
        template.blocks = vec![
            Block::image("/placeholder-portfolio.jpg", "Project image", None),
            Block::heading("Project Title", 1),
            Block::columns(vec![
                Block::column(vec![
                    Block::heading("Overview", 3),
                    Block::paragraph("Project description..."),
                ]),
                Block::column(vec![
                    Block::heading("Details", 3),
                    Block::list(&["Client: ", "Year: ", "Services: "], false),
                ]),
            ]),
            Block::heading("The Challenge", 2),
            Block::paragraph("What problem did this solve..."),
            Block::heading("The Solution", 2),
            Block::paragraph("How we approached it..."),
            Block::gallery(vec![], 3),
        ];
        template.is_system = true;
        template
    }

    /// Create product page template
    pub fn product_page() -> Self {
        let mut template = Self::new("Product Page", "product");
        template.description = Some("E-commerce product listing".to_string());
        template.blocks = vec![
            Block::columns(vec![
                Block::column(vec![
                    Block::gallery(vec![], 1),
                ]),
                Block::column(vec![
                    Block::heading("Product Name", 1),
                    Block::paragraph("$99.99"),
                    Block::paragraph("Product description..."),
                    Block::button("Add to Cart", "#add-to-cart", Some("is-style-fill")),
                ]),
            ]),
            Block::separator(None),
            Block::heading("Product Details", 2),
            Block::paragraph("Detailed product information..."),
        ];
        template.is_system = true;
        template
    }
}

/// Template service
pub struct TemplateService {
    pool: sqlx::PgPool,
}

impl TemplateService {
    /// Create new template service
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// Initialize system templates
    pub async fn init_system_templates(&self) -> ContentResult<()> {
        let templates = vec![
            ContentTemplate::blank(),
            ContentTemplate::blog_post(),
            ContentTemplate::landing_page(),
            ContentTemplate::about_page(),
            ContentTemplate::contact_page(),
            ContentTemplate::portfolio_item(),
            ContentTemplate::product_page(),
        ];

        for template in templates {
            // Insert only if not exists
            sqlx::query(
                r#"
                INSERT INTO content_templates (
                    id, name, description, post_type, content, blocks,
                    format, meta, is_system, created_at, updated_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                ON CONFLICT (name) DO NOTHING
                "#,
            )
            .bind(template.id)
            .bind(&template.name)
            .bind(&template.description)
            .bind(&template.post_type)
            .bind(&template.content)
            .bind(serde_json::to_value(&template.blocks)?)
            .bind(serde_json::to_string(&template.format)?)
            .bind(&template.meta)
            .bind(template.is_system)
            .bind(template.created_at)
            .bind(template.updated_at)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Create a new template
    pub async fn create(&self, template: ContentTemplate) -> ContentResult<ContentTemplate> {
        sqlx::query(
            r#"
            INSERT INTO content_templates (
                id, name, description, post_type, content, blocks,
                format, meta, is_system, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(template.id)
        .bind(&template.name)
        .bind(&template.description)
        .bind(&template.post_type)
        .bind(&template.content)
        .bind(serde_json::to_value(&template.blocks)?)
        .bind(serde_json::to_string(&template.format)?)
        .bind(&template.meta)
        .bind(template.is_system)
        .bind(template.created_at)
        .bind(template.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(template)
    }

    /// Get template by ID
    pub async fn get(&self, id: Uuid) -> ContentResult<ContentTemplate> {
        let row = sqlx::query_as::<_, TemplateRow>(
            "SELECT * FROM content_templates WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ContentError::NotFound(id.to_string()))?;

        row.try_into()
    }

    /// Get template by name
    pub async fn get_by_name(&self, name: &str) -> ContentResult<ContentTemplate> {
        let row = sqlx::query_as::<_, TemplateRow>(
            "SELECT * FROM content_templates WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ContentError::NotFound(name.to_string()))?;

        row.try_into()
    }

    /// List all templates
    pub async fn list(&self) -> ContentResult<Vec<ContentTemplate>> {
        let rows = sqlx::query_as::<_, TemplateRow>(
            "SELECT * FROM content_templates ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    /// List templates by post type
    pub async fn list_by_post_type(&self, post_type: &str) -> ContentResult<Vec<ContentTemplate>> {
        let rows = sqlx::query_as::<_, TemplateRow>(
            "SELECT * FROM content_templates WHERE post_type = $1 ORDER BY name",
        )
        .bind(post_type)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    /// Update template
    pub async fn update(&self, mut template: ContentTemplate) -> ContentResult<ContentTemplate> {
        if template.is_system {
            return Err(ContentError::PermissionDenied(
                "Cannot modify system templates".to_string(),
            ));
        }

        template.updated_at = Utc::now();

        sqlx::query(
            r#"
            UPDATE content_templates SET
                name = $2, description = $3, post_type = $4, content = $5,
                blocks = $6, format = $7, meta = $8, updated_at = $9
            WHERE id = $1 AND is_system = false
            "#,
        )
        .bind(template.id)
        .bind(&template.name)
        .bind(&template.description)
        .bind(&template.post_type)
        .bind(&template.content)
        .bind(serde_json::to_value(&template.blocks)?)
        .bind(serde_json::to_string(&template.format)?)
        .bind(&template.meta)
        .bind(template.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(template)
    }

    /// Delete template
    pub async fn delete(&self, id: Uuid) -> ContentResult<()> {
        let result = sqlx::query(
            "DELETE FROM content_templates WHERE id = $1 AND is_system = false",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(ContentError::PermissionDenied(
                "Cannot delete system templates".to_string(),
            ));
        }

        Ok(())
    }

    /// Duplicate template
    pub async fn duplicate(&self, id: Uuid, new_name: &str) -> ContentResult<ContentTemplate> {
        let original = self.get(id).await?;

        let duplicate = ContentTemplate {
            id: Uuid::new_v4(),
            name: new_name.to_string(),
            description: original.description,
            post_type: original.post_type,
            content: original.content,
            blocks: original.blocks,
            format: original.format,
            meta: original.meta,
            is_system: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.create(duplicate).await
    }
}

/// Database row
#[derive(Debug, sqlx::FromRow)]
struct TemplateRow {
    id: Uuid,
    name: String,
    description: Option<String>,
    post_type: String,
    content: String,
    blocks: serde_json::Value,
    format: String,
    meta: serde_json::Value,
    is_system: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<TemplateRow> for ContentTemplate {
    type Error = ContentError;

    fn try_from(row: TemplateRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            name: row.name,
            description: row.description,
            post_type: row.post_type,
            content: row.content,
            blocks: serde_json::from_value(row.blocks)?,
            format: serde_json::from_str(&row.format)?,
            meta: row.meta,
            is_system: row.is_system,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_template() {
        let template = ContentTemplate::new("Test", "post");
        assert_eq!(template.name, "Test");
        assert_eq!(template.post_type, "post");
    }

    #[test]
    fn test_blog_post_template() {
        let template = ContentTemplate::blog_post();
        assert_eq!(template.name, "Blog Post");
        assert!(template.is_system);
        assert!(!template.blocks.is_empty());
    }

    #[test]
    fn test_landing_page_template() {
        let template = ContentTemplate::landing_page();
        assert_eq!(template.name, "Landing Page");
        assert_eq!(template.post_type, "page");
    }
}
