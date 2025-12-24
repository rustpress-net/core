//! Taxonomy management (categories, tags, custom taxonomies)
//!
//! Provides hierarchical and flat taxonomy systems for content organization.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{ContentError, ContentResult};

/// Taxonomy definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Taxonomy {
    /// URL slug (must be unique)
    pub slug: String,

    /// Display name (plural)
    pub name: String,

    /// Display name (singular)
    pub singular_name: String,

    /// Description
    pub description: Option<String>,

    /// Supports hierarchy (like categories)
    pub hierarchical: bool,

    /// Is publicly visible
    pub public: bool,

    /// Show in admin menu
    pub show_in_menu: bool,

    /// Custom URL rewrite slug
    pub rewrite_slug: Option<String>,

    /// Post types this taxonomy applies to
    pub post_types: Vec<String>,

    /// Is system taxonomy (cannot be deleted)
    pub is_system: bool,

    /// When created
    pub created_at: DateTime<Utc>,
}

impl Taxonomy {
    /// Create a new taxonomy
    pub fn new(slug: &str, name: &str, singular_name: &str) -> Self {
        Self {
            slug: slug.to_string(),
            name: name.to_string(),
            singular_name: singular_name.to_string(),
            description: None,
            hierarchical: false,
            public: true,
            show_in_menu: true,
            rewrite_slug: None,
            post_types: vec!["post".to_string()],
            is_system: false,
            created_at: Utc::now(),
        }
    }

    /// Create the default 'category' taxonomy
    pub fn category() -> Self {
        let mut tax = Self::new("category", "Categories", "Category");
        tax.hierarchical = true;
        tax.is_system = true;
        tax
    }

    /// Create the default 'tag' taxonomy
    pub fn tag() -> Self {
        let mut tax = Self::new("tag", "Tags", "Tag");
        tax.is_system = true;
        tax
    }

    /// Create a product category taxonomy
    pub fn product_category() -> Self {
        let mut tax = Self::new("product_category", "Product Categories", "Product Category");
        tax.hierarchical = true;
        tax.post_types = vec!["product".to_string()];
        tax
    }

    /// Create a product tag taxonomy
    pub fn product_tag() -> Self {
        let mut tax = Self::new("product_tag", "Product Tags", "Product Tag");
        tax.post_types = vec!["product".to_string()];
        tax
    }

    /// Validate taxonomy
    pub fn validate(&self) -> ContentResult<()> {
        let slug_regex = regex::Regex::new(r"^[a-z][a-z0-9_-]*$").unwrap();
        if !slug_regex.is_match(&self.slug) {
            return Err(ContentError::Validation(
                "Taxonomy slug must start with a letter and contain only lowercase letters, numbers, underscores, and hyphens".to_string(),
            ));
        }

        if self.slug.len() > 32 {
            return Err(ContentError::Validation(
                "Taxonomy slug must be 32 characters or less".to_string(),
            ));
        }

        if self.name.is_empty() {
            return Err(ContentError::Validation(
                "Taxonomy name is required".to_string(),
            ));
        }

        Ok(())
    }
}

/// Term (individual category, tag, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Term {
    /// Term ID
    pub id: Uuid,

    /// Taxonomy slug
    pub taxonomy: String,

    /// Term name
    pub name: String,

    /// URL slug
    pub slug: String,

    /// Description
    pub description: Option<String>,

    /// Parent term ID (for hierarchical taxonomies)
    pub parent_id: Option<Uuid>,

    /// Custom metadata
    pub meta: serde_json::Value,

    /// Content count using this term
    pub count: i32,

    /// When created
    pub created_at: DateTime<Utc>,
}

impl Term {
    /// Create a new term
    pub fn new(taxonomy: &str, name: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            taxonomy: taxonomy.to_string(),
            name: name.to_string(),
            slug: slug::slugify(name),
            description: None,
            parent_id: None,
            meta: serde_json::json!({}),
            count: 0,
            created_at: Utc::now(),
        }
    }

    /// Validate term
    pub fn validate(&self) -> ContentResult<()> {
        if self.name.is_empty() {
            return Err(ContentError::Validation(
                "Term name is required".to_string(),
            ));
        }

        if self.slug.is_empty() {
            return Err(ContentError::Validation(
                "Term slug is required".to_string(),
            ));
        }

        let slug_regex = regex::Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").unwrap();
        if !slug_regex.is_match(&self.slug) {
            return Err(ContentError::Validation(
                "Invalid slug format".to_string(),
            ));
        }

        Ok(())
    }
}

/// Taxonomy service
pub struct TaxonomyService {
    pool: sqlx::PgPool,
}

impl TaxonomyService {
    /// Create new service
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// Initialize system taxonomies
    pub async fn init_system_taxonomies(&self) -> ContentResult<()> {
        let taxonomies = vec![Taxonomy::category(), Taxonomy::tag()];

        for tax in taxonomies {
            sqlx::query(
                r#"
                INSERT INTO taxonomies (
                    slug, name, singular_name, description, hierarchical,
                    public, show_in_menu, rewrite_slug, post_types, is_system, created_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                ON CONFLICT (slug) DO NOTHING
                "#,
            )
            .bind(&tax.slug)
            .bind(&tax.name)
            .bind(&tax.singular_name)
            .bind(&tax.description)
            .bind(tax.hierarchical)
            .bind(tax.public)
            .bind(tax.show_in_menu)
            .bind(&tax.rewrite_slug)
            .bind(serde_json::to_value(&tax.post_types)?)
            .bind(tax.is_system)
            .bind(tax.created_at)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Register a taxonomy
    pub async fn register_taxonomy(&self, taxonomy: Taxonomy) -> ContentResult<Taxonomy> {
        taxonomy.validate()?;

        sqlx::query(
            r#"
            INSERT INTO taxonomies (
                slug, name, singular_name, description, hierarchical,
                public, show_in_menu, rewrite_slug, post_types, is_system, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(&taxonomy.slug)
        .bind(&taxonomy.name)
        .bind(&taxonomy.singular_name)
        .bind(&taxonomy.description)
        .bind(taxonomy.hierarchical)
        .bind(taxonomy.public)
        .bind(taxonomy.show_in_menu)
        .bind(&taxonomy.rewrite_slug)
        .bind(serde_json::to_value(&taxonomy.post_types)?)
        .bind(taxonomy.is_system)
        .bind(taxonomy.created_at)
        .execute(&self.pool)
        .await?;

        Ok(taxonomy)
    }

    /// Get taxonomy by slug
    pub async fn get_taxonomy(&self, slug: &str) -> ContentResult<Taxonomy> {
        let row = sqlx::query_as::<_, TaxonomyRow>(
            "SELECT * FROM taxonomies WHERE slug = $1",
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ContentError::NotFound(slug.to_string()))?;

        row.try_into()
    }

    /// List all taxonomies
    pub async fn list_taxonomies(&self) -> ContentResult<Vec<Taxonomy>> {
        let rows = sqlx::query_as::<_, TaxonomyRow>(
            "SELECT * FROM taxonomies ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    /// List taxonomies for a post type
    pub async fn taxonomies_for_post_type(&self, post_type: &str) -> ContentResult<Vec<Taxonomy>> {
        let rows = sqlx::query_as::<_, TaxonomyRow>(
            "SELECT * FROM taxonomies WHERE post_types ? $1 ORDER BY name",
        )
        .bind(post_type)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    /// Create a term
    pub async fn create_term(&self, term: Term) -> ContentResult<Term> {
        term.validate()?;

        // Verify taxonomy exists
        let _ = self.get_taxonomy(&term.taxonomy).await?;

        sqlx::query(
            r#"
            INSERT INTO terms (id, taxonomy, name, slug, description, parent_id, meta, count, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(term.id)
        .bind(&term.taxonomy)
        .bind(&term.name)
        .bind(&term.slug)
        .bind(&term.description)
        .bind(term.parent_id)
        .bind(&term.meta)
        .bind(term.count)
        .bind(term.created_at)
        .execute(&self.pool)
        .await?;

        Ok(term)
    }

    /// Get term by ID
    pub async fn get_term(&self, id: Uuid) -> ContentResult<Term> {
        let row = sqlx::query_as::<_, TermRow>(
            "SELECT * FROM terms WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ContentError::NotFound(id.to_string()))?;

        row.try_into()
    }

    /// Get term by slug and taxonomy
    pub async fn get_term_by_slug(&self, taxonomy: &str, slug: &str) -> ContentResult<Term> {
        let row = sqlx::query_as::<_, TermRow>(
            "SELECT * FROM terms WHERE taxonomy = $1 AND slug = $2",
        )
        .bind(taxonomy)
        .bind(slug)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ContentError::NotFound(format!("{}:{}", taxonomy, slug)))?;

        row.try_into()
    }

    /// List terms for a taxonomy
    pub async fn list_terms(&self, taxonomy: &str) -> ContentResult<Vec<Term>> {
        let rows = sqlx::query_as::<_, TermRow>(
            "SELECT * FROM terms WHERE taxonomy = $1 ORDER BY name",
        )
        .bind(taxonomy)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    /// List terms hierarchically
    pub async fn list_terms_tree(&self, taxonomy: &str) -> ContentResult<Vec<TermNode>> {
        let terms = self.list_terms(taxonomy).await?;
        Ok(build_term_tree(&terms, None))
    }

    /// Update term
    pub async fn update_term(&self, term: Term) -> ContentResult<Term> {
        term.validate()?;

        sqlx::query(
            r#"
            UPDATE terms SET
                name = $2, slug = $3, description = $4, parent_id = $5, meta = $6
            WHERE id = $1
            "#,
        )
        .bind(term.id)
        .bind(&term.name)
        .bind(&term.slug)
        .bind(&term.description)
        .bind(term.parent_id)
        .bind(&term.meta)
        .execute(&self.pool)
        .await?;

        Ok(term)
    }

    /// Delete term
    pub async fn delete_term(&self, id: Uuid) -> ContentResult<()> {
        // Update children to have no parent
        sqlx::query("UPDATE terms SET parent_id = NULL WHERE parent_id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        // Remove content associations
        sqlx::query("DELETE FROM content_terms WHERE term_id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        // Delete term
        sqlx::query("DELETE FROM terms WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Assign terms to content
    pub async fn set_content_terms(&self, content_id: Uuid, term_ids: &[Uuid]) -> ContentResult<()> {
        // Remove existing associations
        sqlx::query("DELETE FROM content_terms WHERE content_id = $1")
            .bind(content_id)
            .execute(&self.pool)
            .await?;

        // Add new associations
        for term_id in term_ids {
            sqlx::query(
                "INSERT INTO content_terms (content_id, term_id) VALUES ($1, $2)",
            )
            .bind(content_id)
            .bind(term_id)
            .execute(&self.pool)
            .await?;
        }

        // Update term counts
        self.update_term_counts(term_ids).await?;

        Ok(())
    }

    /// Get terms for content
    pub async fn get_content_terms(&self, content_id: Uuid) -> ContentResult<Vec<Term>> {
        let rows = sqlx::query_as::<_, TermRow>(
            r#"
            SELECT t.* FROM terms t
            JOIN content_terms ct ON t.id = ct.term_id
            WHERE ct.content_id = $1
            ORDER BY t.taxonomy, t.name
            "#,
        )
        .bind(content_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    /// Get content terms by taxonomy
    pub async fn get_content_terms_by_taxonomy(
        &self,
        content_id: Uuid,
        taxonomy: &str,
    ) -> ContentResult<Vec<Term>> {
        let rows = sqlx::query_as::<_, TermRow>(
            r#"
            SELECT t.* FROM terms t
            JOIN content_terms ct ON t.id = ct.term_id
            WHERE ct.content_id = $1 AND t.taxonomy = $2
            ORDER BY t.name
            "#,
        )
        .bind(content_id)
        .bind(taxonomy)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    /// Update term counts
    async fn update_term_counts(&self, term_ids: &[Uuid]) -> ContentResult<()> {
        for term_id in term_ids {
            sqlx::query(
                r#"
                UPDATE terms SET count = (
                    SELECT COUNT(*) FROM content_terms WHERE term_id = $1
                ) WHERE id = $1
                "#,
            )
            .bind(term_id)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Search terms
    pub async fn search_terms(&self, taxonomy: &str, query: &str) -> ContentResult<Vec<Term>> {
        let rows = sqlx::query_as::<_, TermRow>(
            "SELECT * FROM terms WHERE taxonomy = $1 AND name ILIKE $2 ORDER BY name LIMIT 20",
        )
        .bind(taxonomy)
        .bind(format!("%{}%", query))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    /// Get popular terms
    pub async fn get_popular_terms(&self, taxonomy: &str, limit: i64) -> ContentResult<Vec<Term>> {
        let rows = sqlx::query_as::<_, TermRow>(
            "SELECT * FROM terms WHERE taxonomy = $1 AND count > 0 ORDER BY count DESC LIMIT $2",
        )
        .bind(taxonomy)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }
}

/// Term with children (for tree display)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermNode {
    pub term: Term,
    pub children: Vec<TermNode>,
}

/// Build term tree from flat list
fn build_term_tree(terms: &[Term], parent_id: Option<Uuid>) -> Vec<TermNode> {
    terms
        .iter()
        .filter(|t| t.parent_id == parent_id)
        .map(|term| TermNode {
            term: term.clone(),
            children: build_term_tree(terms, Some(term.id)),
        })
        .collect()
}

/// Database row for taxonomy
#[derive(Debug, sqlx::FromRow)]
struct TaxonomyRow {
    slug: String,
    name: String,
    singular_name: String,
    description: Option<String>,
    hierarchical: bool,
    public: bool,
    show_in_menu: bool,
    rewrite_slug: Option<String>,
    post_types: serde_json::Value,
    is_system: bool,
    created_at: DateTime<Utc>,
}

impl TryFrom<TaxonomyRow> for Taxonomy {
    type Error = ContentError;

    fn try_from(row: TaxonomyRow) -> Result<Self, Self::Error> {
        Ok(Self {
            slug: row.slug,
            name: row.name,
            singular_name: row.singular_name,
            description: row.description,
            hierarchical: row.hierarchical,
            public: row.public,
            show_in_menu: row.show_in_menu,
            rewrite_slug: row.rewrite_slug,
            post_types: serde_json::from_value(row.post_types)?,
            is_system: row.is_system,
            created_at: row.created_at,
        })
    }
}

/// Database row for term
#[derive(Debug, sqlx::FromRow)]
struct TermRow {
    id: Uuid,
    taxonomy: String,
    name: String,
    slug: String,
    description: Option<String>,
    parent_id: Option<Uuid>,
    meta: serde_json::Value,
    count: i32,
    created_at: DateTime<Utc>,
}

impl TryFrom<TermRow> for Term {
    type Error = ContentError;

    fn try_from(row: TermRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            taxonomy: row.taxonomy,
            name: row.name,
            slug: row.slug,
            description: row.description,
            parent_id: row.parent_id,
            meta: row.meta,
            count: row.count,
            created_at: row.created_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_taxonomy() {
        let tax = Taxonomy::new("genre", "Genres", "Genre");
        assert_eq!(tax.slug, "genre");
        assert!(!tax.hierarchical);
    }

    #[test]
    fn test_create_term() {
        let term = Term::new("category", "Technology");
        assert_eq!(term.taxonomy, "category");
        assert_eq!(term.slug, "technology");
    }

    #[test]
    fn test_term_tree() {
        let parent = Term::new("category", "Parent");
        let mut child = Term::new("category", "Child");
        child.parent_id = Some(parent.id);

        let terms = vec![parent.clone(), child];
        let tree = build_term_tree(&terms, None);

        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].children.len(), 1);
    }
}
