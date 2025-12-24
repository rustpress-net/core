//! Public Theme Rendering Service
//!
//! Renders the public-facing website using the active theme's templates.
//! Handles WordPress-like template hierarchy for different content types.

use chrono::{DateTime, Utc};
use rustpress_core::error::{Error, Result};
use rustpress_themes::templates::{QueryContext, TemplateEngine};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tera::Context;
use tokio::sync::RwLock;

use super::ThemeService;

/// Database row for posts
#[derive(Debug, FromRow)]
struct PostRow {
    id: Uuid,
    title: String,
    slug: String,
    content: Option<String>,
    excerpt: Option<String>,
    post_type: String,
    status: String,
    author_id: Uuid,
    featured_media_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    published_at: Option<DateTime<Utc>>,
    author_name: Option<String>,
    author_slug: String,
    author_bio: Option<String>,
    author_avatar: Option<String>,
    comment_count: Option<i64>,
}

/// Database row for terms
#[derive(Debug, FromRow)]
struct TermRow {
    id: Uuid,
    name: String,
    slug: String,
    description: Option<String>,
    taxonomy: String,
    count: Option<i64>,
}

/// Database row for authors
#[derive(Debug, FromRow)]
struct AuthorRow {
    id: Uuid,
    name: Option<String>,
    slug: String,
    bio: Option<String>,
    avatar_url: Option<String>,
    url: Option<String>,
}

/// Database row for media
#[derive(Debug, FromRow)]
struct MediaRow {
    id: Uuid,
    url: String,
    alt: Option<String>,
    title: Option<String>,
    width: Option<i32>,
    height: Option<i32>,
    mime_type: String,
}

/// Database row for menus
#[derive(Debug, FromRow)]
struct MenuRow {
    id: Uuid,
    name: String,
    slug: String,
}

/// Database row for menu items
#[derive(Debug, FromRow)]
struct MenuItemRow {
    id: Uuid,
    title: String,
    url: String,
    target: Option<String>,
    css_classes: Option<String>,
    parent_id: Option<Uuid>,
    #[allow(dead_code)]
    position: i32,
}

/// Database row for widget areas
#[derive(Debug, FromRow)]
struct WidgetAreaRow {
    id: Uuid,
    slug: String,
    name: String,
}

/// Database row for widgets
#[derive(Debug, FromRow)]
struct WidgetRow {
    id: Uuid,
    widget_type: String,
    title: Option<String>,
    content: Option<String>,
    settings: serde_json::Value,
}

/// Site metadata for templates
#[derive(Debug, Clone, Serialize)]
pub struct SiteInfo {
    pub name: String,
    pub description: String,
    pub url: String,
    pub language: String,
    pub charset: String,
    pub default_image: String,
    pub author: String,
}

/// Post/content data for templates
#[derive(Debug, Clone, Serialize)]
pub struct PostData {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub post_type: String,
    pub status: String,
    pub author: AuthorData,
    pub featured_image: Option<MediaData>,
    pub categories: Vec<TermData>,
    pub tags: Vec<TermData>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub comment_count: i32,
    pub meta: HashMap<String, serde_json::Value>,
}

/// Author data for templates
#[derive(Debug, Clone, Serialize)]
pub struct AuthorData {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub url: Option<String>,
}

/// Media/attachment data
#[derive(Debug, Clone, Serialize)]
pub struct MediaData {
    pub id: String,
    pub url: String,
    pub alt: Option<String>,
    pub title: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub mime_type: String,
}

/// Term (category/tag/taxonomy) data
#[derive(Debug, Clone, Serialize)]
pub struct TermData {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub count: i32,
    pub taxonomy: String,
}

/// Menu item data
#[derive(Debug, Clone, Serialize)]
pub struct MenuItemData {
    pub id: String,
    pub title: String,
    pub url: String,
    pub target: Option<String>,
    pub classes: Vec<String>,
    pub parent_id: Option<String>,
    pub children: Vec<MenuItemData>,
}

/// Menu data
#[derive(Debug, Clone, Serialize)]
pub struct MenuData {
    pub name: String,
    pub slug: String,
    pub items: Vec<MenuItemData>,
}

/// Widget data
#[derive(Debug, Clone, Serialize)]
pub struct WidgetData {
    pub id: String,
    pub widget_type: String,
    pub title: Option<String>,
    pub content: String,
    pub settings: serde_json::Value,
}

/// Widget area data
#[derive(Debug, Clone, Serialize)]
pub struct WidgetAreaData {
    pub slug: String,
    pub name: String,
    pub widgets: Vec<WidgetData>,
}

/// Pagination data
#[derive(Debug, Clone, Serialize)]
pub struct PaginationData {
    pub current_page: i32,
    pub total_pages: i32,
    pub total_items: i64,
    pub per_page: i32,
    pub has_previous: bool,
    pub has_next: bool,
    pub previous_url: Option<String>,
    pub next_url: Option<String>,
}

/// Archive data
#[derive(Debug, Clone, Serialize)]
pub struct ArchiveData {
    pub title: String,
    pub description: Option<String>,
    pub archive_type: String, // category, tag, author, date, custom
    pub term: Option<TermData>,
    pub author: Option<AuthorData>,
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>,
}

/// Rendered page response
#[derive(Debug)]
pub struct RenderedPage {
    pub html: String,
    pub status_code: u16,
    pub cache_control: String,
    pub content_type: String,
}

/// Public rendering service
pub struct RenderService {
    pool: PgPool,
    theme_service: Arc<ThemeService>,
    themes_dir: PathBuf,
    template_engines: Arc<RwLock<HashMap<String, Arc<TemplateEngine>>>>,
    site_info: Arc<RwLock<SiteInfo>>,
}

impl RenderService {
    /// Create a new render service
    pub fn new(
        pool: PgPool,
        theme_service: Arc<ThemeService>,
        themes_dir: PathBuf,
    ) -> Self {
        Self {
            pool,
            theme_service,
            themes_dir,
            template_engines: Arc::new(RwLock::new(HashMap::new())),
            site_info: Arc::new(RwLock::new(SiteInfo {
                name: "RustPress Site".to_string(),
                description: "Powered by RustPress".to_string(),
                url: "http://localhost:3000".to_string(),
                language: "en-US".to_string(),
                charset: "UTF-8".to_string(),
                default_image: "/themes/rustpress-enterprise/assets/images/og-default.jpg".to_string(),
                author: "RustPress".to_string(),
            })),
        }
    }

    /// Update site info from settings
    pub async fn update_site_info(&self, info: SiteInfo) {
        *self.site_info.write().await = info;
    }

    /// Get or create template engine for a theme
    async fn get_engine(&self, theme_id: &str) -> Result<Arc<TemplateEngine>> {
        // Check cache
        {
            let engines = self.template_engines.read().await;
            if let Some(engine) = engines.get(theme_id) {
                return Ok(engine.clone());
            }
        }

        // Create new engine
        let theme_dir = self.themes_dir.join(theme_id);
        let engine = TemplateEngine::new(theme_dir, "html")
            .map_err(|e| Error::internal(format!("Failed to create template engine: {}", e)))?;

        engine.init()
            .map_err(|e| Error::internal(format!("Failed to initialize templates: {}", e)))?;

        let engine = Arc::new(engine);

        // Cache it
        {
            let mut engines = self.template_engines.write().await;
            engines.insert(theme_id.to_string(), engine.clone());
        }

        Ok(engine)
    }

    /// Build base context for all templates
    async fn build_base_context(&self, theme_id: &str) -> Context {
        let mut context = Context::new();

        // Site info
        let site_info = self.site_info.read().await.clone();
        context.insert("site", &site_info);

        // Theme info
        if let Ok(Some(theme)) = self.theme_service.get_theme(theme_id).await {
            context.insert("theme", &serde_json::json!({
                "id": theme.theme_id,
                "name": theme.name,
                "version": theme.version,
                "author": theme.author,
            }));
        }

        // Current year for copyright
        context.insert("current_year", &Utc::now().format("%Y").to_string());

        // Menus - would load from database
        let menus = self.load_menus(theme_id).await.unwrap_or_default();
        context.insert("menus", &menus);

        // Widget areas - would load from database
        let widget_areas = self.load_widget_areas(theme_id).await.unwrap_or_default();
        context.insert("sidebars", &widget_areas);

        context
    }

    /// Load menus for theme
    async fn load_menus(&self, theme_id: &str) -> Result<HashMap<String, MenuData>> {
        let mut menus = HashMap::new();

        // Get menu assignments for this theme
        let assignments = self.theme_service.get_menu_assignments(theme_id).await?;

        // Load menus from database
        let menu_rows: Vec<MenuRow> = sqlx::query_as(
            "SELECT id, name, slug FROM menus WHERE deleted_at IS NULL"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to load menus", e))?;

        for menu_row in menu_rows {
            // Load menu items for this menu
            let item_rows: Vec<MenuItemRow> = sqlx::query_as(
                r#"
                SELECT id, title, url, target, css_classes, parent_id, position
                FROM menu_items
                WHERE menu_id = $1
                ORDER BY position ASC
                "#
            )
            .bind(menu_row.id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to load menu items", e))?;

            // Build hierarchical menu structure
            let items = self.build_menu_hierarchy(&item_rows);

            // Check if this menu is assigned to a location in the theme
            let location = assignments.as_array()
                .and_then(|arr| {
                    arr.iter()
                        .find(|a| {
                            a.get("menu_id")
                                .and_then(|m| m.as_str())
                                .map(|mid| mid == menu_row.id.to_string())
                                .unwrap_or(false)
                        })
                        .and_then(|a| a.get("location_slug"))
                        .and_then(|l| l.as_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| menu_row.slug.clone());

            menus.insert(location, MenuData {
                name: menu_row.name,
                slug: menu_row.slug,
                items,
            });
        }

        // Add empty placeholders for common menu locations if not assigned
        for loc in ["primary", "footer", "social"] {
            if !menus.contains_key(loc) {
                menus.insert(loc.to_string(), MenuData {
                    name: format!("{} Menu", loc),
                    slug: loc.to_string(),
                    items: vec![],
                });
            }
        }

        Ok(menus)
    }

    /// Build hierarchical menu structure from flat list
    fn build_menu_hierarchy(&self, items: &[MenuItemRow]) -> Vec<MenuItemData> {
        let mut root_items: Vec<MenuItemData> = Vec::new();
        let mut children_map: HashMap<String, Vec<MenuItemData>> = HashMap::new();

        // First pass: create all items and group by parent
        for item in items {
            let menu_item = MenuItemData {
                id: item.id.to_string(),
                title: item.title.clone(),
                url: item.url.clone(),
                target: item.target.clone(),
                classes: item.css_classes.clone()
                    .map(|s| s.split_whitespace().map(|s| s.to_string()).collect())
                    .unwrap_or_else(|| vec!["menu-item".to_string()]),
                parent_id: item.parent_id.map(|id| id.to_string()),
                children: vec![],
            };

            if let Some(parent_id) = &item.parent_id {
                children_map.entry(parent_id.to_string()).or_default().push(menu_item);
            } else {
                root_items.push(menu_item);
            }
        }

        // Second pass: attach children to parents
        fn attach_children(item: &mut MenuItemData, children_map: &HashMap<String, Vec<MenuItemData>>) {
            if let Some(children) = children_map.get(&item.id) {
                item.children = children.clone();
                for child in &mut item.children {
                    attach_children(child, children_map);
                }
            }
        }

        for item in &mut root_items {
            attach_children(item, &children_map);
        }

        root_items
    }

    /// Load widget areas for theme
    async fn load_widget_areas(&self, _theme_id: &str) -> Result<HashMap<String, WidgetAreaData>> {
        let mut areas = HashMap::new();

        // Load widget areas from database
        let area_rows: Vec<WidgetAreaRow> = sqlx::query_as(
            "SELECT id, slug, name FROM widget_areas WHERE is_active = true ORDER BY position"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to load widget areas", e))?;

        for area_row in area_rows {
            // Load widgets for this area
            let widget_rows: Vec<WidgetRow> = sqlx::query_as(
                r#"
                SELECT id, widget_type, title, content, settings
                FROM widgets
                WHERE sidebar_id = $1 AND is_active = true
                ORDER BY position ASC
                "#
            )
            .bind(area_row.id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to load widgets", e))?;

            let widgets: Vec<WidgetData> = widget_rows.into_iter().map(|w| {
                let content = self.render_widget_content(&w.widget_type, &w.content, &w.settings);
                WidgetData {
                    id: w.id.to_string(),
                    widget_type: w.widget_type,
                    title: w.title,
                    content,
                    settings: w.settings,
                }
            }).collect();

            areas.insert(area_row.slug.clone(), WidgetAreaData {
                slug: area_row.slug,
                name: area_row.name,
                widgets,
            });
        }

        // Add default empty areas for common sidebar locations if not present
        for (slug, name) in [
            ("sidebar", "Main Sidebar"),
            ("footer-1", "Footer Column 1"),
            ("footer-2", "Footer Column 2"),
            ("footer-3", "Footer Column 3"),
        ] {
            if !areas.contains_key(slug) {
                areas.insert(slug.to_string(), WidgetAreaData {
                    slug: slug.to_string(),
                    name: name.to_string(),
                    widgets: vec![],
                });
            }
        }

        Ok(areas)
    }

    /// Render widget content based on widget type
    fn render_widget_content(&self, widget_type: &str, content: &Option<String>, settings: &serde_json::Value) -> String {
        match widget_type {
            "search" => r#"<form role="search" method="get" action="/search" class="widget-search">
                <input type="search" name="q" placeholder="Search..." class="search-input">
                <button type="submit" class="search-button">Search</button>
            </form>"#.to_string(),
            "text" | "html" => content.clone().unwrap_or_default(),
            "recent_posts" => {
                // Placeholder - would be populated with actual recent posts
                "<ul class=\"widget-recent-posts\"><!-- Dynamic content --></ul>".to_string()
            }
            "categories" => {
                // Placeholder - would be populated with actual categories
                "<ul class=\"widget-categories\"><!-- Dynamic content --></ul>".to_string()
            }
            "tag_cloud" => {
                "<div class=\"widget-tag-cloud\"><!-- Dynamic content --></div>".to_string()
            }
            _ => content.clone().unwrap_or_default(),
        }
    }

    /// Render the home/front page
    pub async fn render_home(&self, preview_token: Option<&str>) -> Result<RenderedPage> {
        let theme_id = self.get_active_theme_id(preview_token).await?;
        let engine = self.get_engine(&theme_id).await?;

        let mut context = self.build_base_context(&theme_id).await;

        // Load recent posts for home page
        let posts = self.load_recent_posts(10).await?;
        context.insert("posts", &posts);
        context.insert("is_home", &true);
        context.insert("is_front_page", &true);

        // Add page context for templates that expect it
        let site_info = self.site_info.read().await;
        context.insert("page", &serde_json::json!({
            "title": &site_info.name,
            "description": &site_info.description,
            "url": &site_info.url,
            "canonical": &site_info.url,
        }));
        drop(site_info);

        // Build query context
        let query = QueryContext {
            is_home: true,
            is_front_page: true,
            ..Default::default()
        };

        self.render_with_engine(&engine, &query, &context).await
    }

    /// Render a single post
    pub async fn render_post(&self, slug: &str, preview_token: Option<&str>) -> Result<RenderedPage> {
        let theme_id = self.get_active_theme_id(preview_token).await?;
        let engine = self.get_engine(&theme_id).await?;

        let mut context = self.build_base_context(&theme_id).await;

        // Load the post
        let post = self.load_post_by_slug(slug).await?
            .ok_or_else(|| Error::not_found("Post", slug))?;

        context.insert("post", &post);
        context.insert("is_single", &true);

        // Build query context
        let query = QueryContext {
            is_single: true,
            post_type: Some(post.post_type.clone()),
            post_slug: Some(slug.to_string()),
            post_id: None, // We use slug for template hierarchy instead
            ..Default::default()
        };

        self.render_with_engine(&engine, &query, &context).await
    }

    /// Render a page
    pub async fn render_page(&self, slug: &str, preview_token: Option<&str>) -> Result<RenderedPage> {
        let theme_id = self.get_active_theme_id(preview_token).await?;
        let engine = self.get_engine(&theme_id).await?;

        let mut context = self.build_base_context(&theme_id).await;

        // Load the page
        let page = self.load_page_by_slug(slug).await?
            .ok_or_else(|| Error::not_found("Page", slug))?;

        context.insert("page", &page);
        context.insert("post", &page); // WordPress uses 'post' for pages too
        context.insert("is_page", &true);

        // Build query context
        let query = QueryContext {
            is_page: true,
            post_slug: Some(slug.to_string()),
            post_id: None, // We use slug for template hierarchy instead
            page_template: page.meta.get("_wp_page_template")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            ..Default::default()
        };

        self.render_with_engine(&engine, &query, &context).await
    }

    /// Render category archive
    pub async fn render_category(&self, slug: &str, page: i32, preview_token: Option<&str>) -> Result<RenderedPage> {
        let theme_id = self.get_active_theme_id(preview_token).await?;
        let engine = self.get_engine(&theme_id).await?;

        let mut context = self.build_base_context(&theme_id).await;

        // Load category info
        let category = self.load_term_by_slug(slug, "category").await?
            .ok_or_else(|| Error::not_found("Category", slug))?;

        // Load posts in category
        let (posts, total) = self.load_posts_by_term(&category.id, "category", page, 10).await?;
        let pagination = self.build_pagination(page, total, 10, &format!("/category/{}", slug));

        context.insert("term", &category);
        context.insert("category", &category);
        context.insert("posts", &posts);
        context.insert("pagination", &pagination);
        context.insert("is_category", &true);
        context.insert("is_archive", &true);
        context.insert("archive", &ArchiveData {
            title: category.name.clone(),
            description: category.description.clone(),
            archive_type: "category".to_string(),
            term: Some(category.clone()),
            author: None,
            year: None,
            month: None,
            day: None,
        });

        let query = QueryContext {
            is_category: true,
            is_archive: true,
            term_slug: Some(slug.to_string()),
            term_id: None, // We use slug for template hierarchy instead
            taxonomy: Some("category".to_string()),
            ..Default::default()
        };

        self.render_with_engine(&engine, &query, &context).await
    }

    /// Render tag archive
    pub async fn render_tag(&self, slug: &str, page: i32, preview_token: Option<&str>) -> Result<RenderedPage> {
        let theme_id = self.get_active_theme_id(preview_token).await?;
        let engine = self.get_engine(&theme_id).await?;

        let mut context = self.build_base_context(&theme_id).await;

        // Load tag info
        let tag = self.load_term_by_slug(slug, "tag").await?
            .ok_or_else(|| Error::not_found("Tag", slug))?;

        // Load posts with tag
        let (posts, total) = self.load_posts_by_term(&tag.id, "tag", page, 10).await?;
        let pagination = self.build_pagination(page, total, 10, &format!("/tag/{}", slug));

        context.insert("term", &tag);
        context.insert("tag", &tag);
        context.insert("posts", &posts);
        context.insert("pagination", &pagination);
        context.insert("is_tag", &true);
        context.insert("is_archive", &true);
        context.insert("archive", &ArchiveData {
            title: tag.name.clone(),
            description: tag.description.clone(),
            archive_type: "tag".to_string(),
            term: Some(tag.clone()),
            author: None,
            year: None,
            month: None,
            day: None,
        });

        let query = QueryContext {
            is_tag: true,
            is_archive: true,
            term_slug: Some(slug.to_string()),
            term_id: None, // We use slug for template hierarchy instead
            taxonomy: Some("post_tag".to_string()),
            ..Default::default()
        };

        self.render_with_engine(&engine, &query, &context).await
    }

    /// Render author archive
    pub async fn render_author(&self, slug: &str, page: i32, preview_token: Option<&str>) -> Result<RenderedPage> {
        let theme_id = self.get_active_theme_id(preview_token).await?;
        let engine = self.get_engine(&theme_id).await?;

        let mut context = self.build_base_context(&theme_id).await;

        // Load author info
        let author = self.load_author_by_slug(slug).await?
            .ok_or_else(|| Error::not_found("Author", slug))?;

        // Load posts by author
        let (posts, total) = self.load_posts_by_author(&author.id, page, 10).await?;
        let pagination = self.build_pagination(page, total, 10, &format!("/author/{}", slug));

        context.insert("author", &author);
        context.insert("posts", &posts);
        context.insert("pagination", &pagination);
        context.insert("is_author", &true);
        context.insert("is_archive", &true);
        context.insert("archive", &ArchiveData {
            title: author.name.clone(),
            description: author.bio.clone(),
            archive_type: "author".to_string(),
            term: None,
            author: Some(author.clone()),
            year: None,
            month: None,
            day: None,
        });

        let query = QueryContext {
            is_author: true,
            is_archive: true,
            author_slug: Some(slug.to_string()),
            author_id: None, // We use slug for template hierarchy instead
            ..Default::default()
        };

        self.render_with_engine(&engine, &query, &context).await
    }

    /// Render search results
    pub async fn render_search(&self, query_str: &str, page: i32, preview_token: Option<&str>) -> Result<RenderedPage> {
        let theme_id = self.get_active_theme_id(preview_token).await?;
        let engine = self.get_engine(&theme_id).await?;

        let mut context = self.build_base_context(&theme_id).await;

        // Search posts
        let (posts, total) = self.search_posts(query_str, page, 10).await?;
        let pagination = self.build_pagination(page, total, 10, &format!("/search?q={}", urlencoding::encode(query_str)));

        context.insert("search_query", query_str);
        context.insert("posts", &posts);
        context.insert("pagination", &pagination);
        context.insert("is_search", &true);
        context.insert("found_posts", &total);

        let query = QueryContext {
            is_search: true,
            ..Default::default()
        };

        self.render_with_engine(&engine, &query, &context).await
    }

    /// Render 404 page
    pub async fn render_404(&self, preview_token: Option<&str>) -> Result<RenderedPage> {
        let theme_id = self.get_active_theme_id(preview_token).await?;
        let engine = self.get_engine(&theme_id).await?;

        let mut context = self.build_base_context(&theme_id).await;
        context.insert("is_404", &true);

        let query = QueryContext {
            is_404: true,
            ..Default::default()
        };

        let mut result = self.render_with_engine(&engine, &query, &context).await?;
        result.status_code = 404;
        Ok(result)
    }

    /// Get active theme ID (or preview theme if token provided)
    async fn get_active_theme_id(&self, preview_token: Option<&str>) -> Result<String> {
        // Check if preview token is provided and valid
        if let Some(token) = preview_token {
            if !token.is_empty() {
                // Query the database for a valid (non-expired) preview session
                let preview: Option<(String,)> = sqlx::query_as(
                    r#"
                    SELECT theme_id
                    FROM theme_previews
                    WHERE preview_token = $1
                    AND expires_at > NOW()
                    "#
                )
                .bind(token)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| Error::internal(format!("Failed to validate preview token: {}", e)))?;

                if let Some((theme_id,)) = preview {
                    // Verify the theme exists before using it
                    if self.theme_service.get_theme(&theme_id).await?.is_some() {
                        return Ok(theme_id);
                    }
                }
                // Invalid or expired token, fall through to active theme
            }
        }

        self.theme_service.get_active_theme_id().await?
            .ok_or_else(|| Error::internal("No active theme configured"))
    }

    /// Render using template engine
    async fn render_with_engine(&self, engine: &TemplateEngine, query: &QueryContext, context: &Context) -> Result<RenderedPage> {
        let html = engine.render_for_query(query, context)
            .map_err(|e| Error::internal(format!("Template render error: {}", e)))?;

        Ok(RenderedPage {
            html,
            status_code: 200,
            cache_control: "public, max-age=60".to_string(),
            content_type: "text/html; charset=utf-8".to_string(),
        })
    }

    /// Build pagination data
    fn build_pagination(&self, current: i32, total: i64, per_page: i32, base_url: &str) -> PaginationData {
        let total_pages = ((total as f64) / (per_page as f64)).ceil() as i32;
        let separator = if base_url.contains('?') { "&" } else { "?" };

        PaginationData {
            current_page: current,
            total_pages,
            total_items: total,
            per_page,
            has_previous: current > 1,
            has_next: current < total_pages,
            previous_url: if current > 1 { Some(format!("{}{}page={}", base_url, separator, current - 1)) } else { None },
            next_url: if current < total_pages { Some(format!("{}{}page={}", base_url, separator, current + 1)) } else { None },
        }
    }

    // ============================================================================
    // Data Loading Methods
    // ============================================================================

    async fn load_recent_posts(&self, limit: i32) -> Result<Vec<PostData>> {
        let rows = sqlx::query_as::<_, PostRow>(
            r#"
            SELECT p.id, p.title, p.slug, p.content, p.excerpt, p.post_type::text, p.status::text,
                   p.author_id, p.featured_image_id AS featured_media_id, p.created_at, p.updated_at, p.published_at,
                   u.display_name as author_name, u.username as author_slug,
                   u.bio as author_bio, u.avatar_url as author_avatar,
                   (SELECT COUNT(*) FROM comments c WHERE c.post_id = p.id AND c.status = 'approved') as comment_count
            FROM posts p
            JOIN users u ON p.author_id = u.id
            WHERE p.status = 'published' AND p.post_type = 'post' AND p.deleted_at IS NULL
            ORDER BY p.published_at DESC NULLS LAST
            LIMIT $1
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to load posts", e))?;

        let mut posts = Vec::new();
        for row in rows {
            let post = self.row_to_post_data(row).await?;
            posts.push(post);
        }
        Ok(posts)
    }

    async fn load_post_by_slug(&self, slug: &str) -> Result<Option<PostData>> {
        let row = sqlx::query_as::<_, PostRow>(
            r#"
            SELECT p.id, p.title, p.slug, p.content, p.excerpt, p.post_type::text, p.status::text,
                   p.author_id, p.featured_image_id AS featured_media_id, p.created_at, p.updated_at, p.published_at,
                   u.display_name as author_name, u.username as author_slug,
                   u.bio as author_bio, u.avatar_url as author_avatar,
                   (SELECT COUNT(*) FROM comments c WHERE c.post_id = p.id AND c.status = 'approved') as comment_count
            FROM posts p
            JOIN users u ON p.author_id = u.id
            WHERE p.slug = $1 AND p.post_type = 'post' AND p.status = 'published' AND p.deleted_at IS NULL
            "#
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to load post", e))?;

        match row {
            Some(r) => Ok(Some(self.row_to_post_data(r).await?)),
            None => Ok(None),
        }
    }

    async fn load_page_by_slug(&self, slug: &str) -> Result<Option<PostData>> {
        let row = sqlx::query_as::<_, PostRow>(
            r#"
            SELECT p.id, p.title, p.slug, p.content, p.excerpt, p.post_type::text, p.status::text,
                   p.author_id, p.featured_image_id AS featured_media_id, p.created_at, p.updated_at, p.published_at,
                   u.display_name as author_name, u.username as author_slug,
                   u.bio as author_bio, u.avatar_url as author_avatar,
                   (SELECT COUNT(*) FROM comments c WHERE c.post_id = p.id AND c.status = 'approved') as comment_count
            FROM posts p
            JOIN users u ON p.author_id = u.id
            WHERE p.slug = $1 AND p.post_type = 'page' AND p.status = 'published' AND p.deleted_at IS NULL
            "#
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to load page", e))?;

        match row {
            Some(r) => Ok(Some(self.row_to_post_data(r).await?)),
            None => Ok(None),
        }
    }

    async fn load_term_by_slug(&self, slug: &str, taxonomy: &str) -> Result<Option<TermData>> {
        let row = sqlx::query_as::<_, TermRow>(
            r#"
            SELECT t.id, t.name, t.slug, t.description, tx.slug as taxonomy,
                   (SELECT COUNT(DISTINCT pt.post_id) FROM post_terms pt
                    JOIN posts p ON pt.post_id = p.id
                    WHERE pt.term_id = t.id AND p.status = 'published') as count
            FROM terms t
            JOIN taxonomies tx ON tx.id = t.taxonomy_id
            WHERE t.slug = $1 AND tx.slug = $2
            "#
        )
        .bind(slug)
        .bind(taxonomy)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to load term", e))?;

        Ok(row.map(|r| TermData {
            id: r.id.to_string(),
            name: r.name,
            slug: r.slug,
            description: r.description,
            count: r.count.unwrap_or(0) as i32,
            taxonomy: r.taxonomy,
        }))
    }

    async fn load_posts_by_term(&self, term_id_str: &str, _taxonomy: &str, page: i32, per_page: i32) -> Result<(Vec<PostData>, i64)> {
        let term_id = Uuid::parse_str(term_id_str)
            .map_err(|e| Error::validation(format!("Invalid term ID: {}", e)))?;
        let offset = (page - 1) * per_page;

        // Get total count
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT p.id)
            FROM posts p
            JOIN post_terms pt ON pt.post_id = p.id
            WHERE pt.term_id = $1 AND p.status = 'published' AND p.post_type = 'post' AND p.deleted_at IS NULL
            "#
        )
        .bind(term_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to count posts", e))?;

        // Get posts
        let rows = sqlx::query_as::<_, PostRow>(
            r#"
            SELECT p.id, p.title, p.slug, p.content, p.excerpt, p.post_type::text, p.status::text,
                   p.author_id, p.featured_image_id AS featured_media_id, p.created_at, p.updated_at, p.published_at,
                   u.display_name as author_name, u.username as author_slug,
                   u.bio as author_bio, u.avatar_url as author_avatar,
                   (SELECT COUNT(*) FROM comments c WHERE c.post_id = p.id AND c.status = 'approved') as comment_count
            FROM posts p
            JOIN users u ON p.author_id = u.id
            JOIN post_terms pt ON pt.post_id = p.id
            WHERE pt.term_id = $1 AND p.status = 'published' AND p.post_type = 'post' AND p.deleted_at IS NULL
            ORDER BY p.published_at DESC NULLS LAST
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(term_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to load posts", e))?;

        let mut posts = Vec::new();
        for row in rows {
            let post = self.row_to_post_data(row).await?;
            posts.push(post);
        }

        Ok((posts, count.0))
    }

    async fn load_author_by_slug(&self, slug: &str) -> Result<Option<AuthorData>> {
        let row = sqlx::query_as::<_, AuthorRow>(
            r#"
            SELECT id, display_name as name, username as slug, bio, avatar_url, website as url
            FROM users
            WHERE username = $1 AND deleted_at IS NULL
            "#
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to load author", e))?;

        Ok(row.map(|r| AuthorData {
            id: r.id.to_string(),
            name: r.name.unwrap_or_else(|| slug.to_string()),
            slug: r.slug,
            bio: r.bio,
            avatar_url: r.avatar_url,
            url: r.url,
        }))
    }

    async fn load_posts_by_author(&self, author_id_str: &str, page: i32, per_page: i32) -> Result<(Vec<PostData>, i64)> {
        let author_id = Uuid::parse_str(author_id_str)
            .map_err(|e| Error::validation(format!("Invalid author ID: {}", e)))?;
        let offset = (page - 1) * per_page;

        // Get total count
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM posts
            WHERE author_id = $1 AND status = 'published' AND post_type = 'post' AND deleted_at IS NULL
            "#
        )
        .bind(author_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to count posts", e))?;

        // Get posts
        let rows = sqlx::query_as::<_, PostRow>(
            r#"
            SELECT p.id, p.title, p.slug, p.content, p.excerpt, p.post_type::text, p.status::text,
                   p.author_id, p.featured_image_id AS featured_media_id, p.created_at, p.updated_at, p.published_at,
                   u.display_name as author_name, u.username as author_slug,
                   u.bio as author_bio, u.avatar_url as author_avatar,
                   (SELECT COUNT(*) FROM comments c WHERE c.post_id = p.id AND c.status = 'approved') as comment_count
            FROM posts p
            JOIN users u ON p.author_id = u.id
            WHERE p.author_id = $1 AND p.status = 'published' AND p.post_type = 'post' AND p.deleted_at IS NULL
            ORDER BY p.published_at DESC NULLS LAST
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(author_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to load posts", e))?;

        let mut posts = Vec::new();
        for row in rows {
            let post = self.row_to_post_data(row).await?;
            posts.push(post);
        }

        Ok((posts, count.0))
    }

    async fn search_posts(&self, query: &str, page: i32, per_page: i32) -> Result<(Vec<PostData>, i64)> {
        let offset = (page - 1) * per_page;
        let search_pattern = format!("%{}%", query);

        // Get total count
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM posts
            WHERE (title ILIKE $1 OR content ILIKE $1 OR excerpt ILIKE $1)
              AND status = 'published' AND post_type = 'post' AND deleted_at IS NULL
            "#
        )
        .bind(&search_pattern)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to count search results", e))?;

        // Get posts
        let rows = sqlx::query_as::<_, PostRow>(
            r#"
            SELECT p.id, p.title, p.slug, p.content, p.excerpt, p.post_type::text, p.status::text,
                   p.author_id, p.featured_image_id AS featured_media_id, p.created_at, p.updated_at, p.published_at,
                   u.display_name as author_name, u.username as author_slug,
                   u.bio as author_bio, u.avatar_url as author_avatar,
                   (SELECT COUNT(*) FROM comments c WHERE c.post_id = p.id AND c.status = 'approved') as comment_count
            FROM posts p
            JOIN users u ON p.author_id = u.id
            WHERE (p.title ILIKE $1 OR p.content ILIKE $1 OR p.excerpt ILIKE $1)
              AND p.status = 'published' AND p.post_type = 'post' AND p.deleted_at IS NULL
            ORDER BY p.published_at DESC NULLS LAST
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(&search_pattern)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to search posts", e))?;

        let mut posts = Vec::new();
        for row in rows {
            let post = self.row_to_post_data(row).await?;
            posts.push(post);
        }

        Ok((posts, count.0))
    }

    /// Convert database row to PostData with related entities
    async fn row_to_post_data(&self, row: PostRow) -> Result<PostData> {
        // Load categories
        let categories = self.load_post_terms(row.id, "category").await?;

        // Load tags
        let tags = self.load_post_terms(row.id, "post_tag").await?;

        // Load featured image if present
        let featured_image = if let Some(media_id) = row.featured_media_id {
            self.load_media(media_id).await?
        } else {
            None
        };

        // Load post meta
        let meta = self.load_post_meta(row.id).await?;

        Ok(PostData {
            id: row.id.to_string(),
            title: row.title,
            slug: row.slug,
            content: row.content.unwrap_or_default(),
            excerpt: row.excerpt,
            post_type: row.post_type,
            status: row.status,
            author: AuthorData {
                id: row.author_id.to_string(),
                name: row.author_name.unwrap_or_else(|| "Unknown".to_string()),
                slug: row.author_slug,
                bio: row.author_bio,
                avatar_url: row.author_avatar,
                url: None,
            },
            featured_image,
            categories,
            tags,
            created_at: row.created_at,
            updated_at: row.updated_at,
            published_at: row.published_at,
            comment_count: row.comment_count.unwrap_or(0) as i32,
            meta,
        })
    }

    /// Load terms for a post
    async fn load_post_terms(&self, post_id: Uuid, taxonomy: &str) -> Result<Vec<TermData>> {
        let rows = sqlx::query_as::<_, TermRow>(
            r#"
            SELECT t.id, t.name, t.slug, t.description, tx.slug as taxonomy,
                   (SELECT COUNT(*) FROM post_terms WHERE term_id = t.id) as count
            FROM terms t
            JOIN taxonomies tx ON tx.id = t.taxonomy_id
            JOIN post_terms pt ON pt.term_id = t.id
            WHERE pt.post_id = $1 AND tx.slug = $2
            "#
        )
        .bind(post_id)
        .bind(taxonomy)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to load terms", e))?;

        Ok(rows.into_iter().map(|r| TermData {
            id: r.id.to_string(),
            name: r.name,
            slug: r.slug,
            description: r.description,
            count: r.count.unwrap_or(0) as i32,
            taxonomy: r.taxonomy,
        }).collect())
    }

    /// Load media by ID
    async fn load_media(&self, media_id: Uuid) -> Result<Option<MediaData>> {
        let row = sqlx::query_as::<_, MediaRow>(
            r#"
            SELECT id, url, alt_text as alt, title, width, height, mime_type
            FROM media
            WHERE id = $1 AND deleted_at IS NULL
            "#
        )
        .bind(media_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to load media", e))?;

        Ok(row.map(|r| MediaData {
            id: r.id.to_string(),
            url: r.url,
            alt: r.alt,
            title: r.title,
            width: r.width,
            height: r.height,
            mime_type: r.mime_type,
        }))
    }

    /// Load post meta
    async fn load_post_meta(&self, post_id: Uuid) -> Result<HashMap<String, serde_json::Value>> {
        let rows: Vec<(String, serde_json::Value)> = sqlx::query_as(
            "SELECT meta_key, meta_value FROM post_meta WHERE post_id = $1"
        )
        .bind(post_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to load post meta", e))?;

        Ok(rows.into_iter().collect())
    }

    /// Clear template engine cache for a theme
    pub async fn clear_theme_cache(&self, theme_id: &str) {
        let mut engines = self.template_engines.write().await;
        engines.remove(theme_id);
    }

    /// Clear all template caches
    pub async fn clear_all_caches(&self) {
        let mut engines = self.template_engines.write().await;
        engines.clear();
    }
}
