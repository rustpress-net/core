//! Auto-purge integration hooks for RustPress events
//!
//! This module provides hooks that automatically purge Cloudflare cache
//! when content changes in RustPress, ensuring visitors always see fresh content.

use crate::error::{CloudflareError, CloudflareResult};
use crate::services::CloudflareServices;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Auto-purge configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AutoPurgeConfig {
    /// Master switch for auto-purge functionality
    pub enabled: bool,
    /// Purge when posts are published or updated
    pub on_post_update: bool,
    /// Purge when pages are published or updated
    pub on_page_update: bool,
    /// Purge when media files are uploaded or deleted
    pub on_media_change: bool,
    /// Purge when themes are changed or updated
    pub on_theme_change: bool,
    /// Purge when menus are updated
    pub on_menu_update: bool,
    /// Purge when widgets/sidebars are updated
    pub on_widget_update: bool,
    /// Purge when settings are changed
    pub on_settings_change: bool,
    /// Purge entire site (purge_all) instead of specific URLs
    pub purge_entire_site: bool,
    /// Also purge homepage when content changes
    pub always_purge_homepage: bool,
    /// Also purge archive/listing pages
    pub purge_archives: bool,
    /// Custom URLs to always purge (comma-separated patterns)
    pub custom_purge_urls: Option<String>,
    /// Delay before purging (milliseconds) - allows batching
    pub purge_delay_ms: u32,
}

impl AutoPurgeConfig {
    /// Create a new auto-purge config with sensible defaults
    pub fn new() -> Self {
        Self {
            enabled: true,
            on_post_update: true,
            on_page_update: true,
            on_media_change: true,
            on_theme_change: true,
            on_menu_update: true,
            on_widget_update: true,
            on_settings_change: false, // Off by default as it can be noisy
            purge_entire_site: false,
            always_purge_homepage: true,
            purge_archives: true,
            custom_purge_urls: None,
            purge_delay_ms: 500, // Small delay to batch rapid changes
        }
    }
}

/// Content types that can trigger auto-purge
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    Post,
    Page,
    Media,
    Theme,
    Menu,
    Widget,
    Settings,
    Plugin,
    User,
    Comment,
    Category,
    Tag,
    Custom(String),
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentType::Post => write!(f, "post"),
            ContentType::Page => write!(f, "page"),
            ContentType::Media => write!(f, "media"),
            ContentType::Theme => write!(f, "theme"),
            ContentType::Menu => write!(f, "menu"),
            ContentType::Widget => write!(f, "widget"),
            ContentType::Settings => write!(f, "settings"),
            ContentType::Plugin => write!(f, "plugin"),
            ContentType::User => write!(f, "user"),
            ContentType::Comment => write!(f, "comment"),
            ContentType::Category => write!(f, "category"),
            ContentType::Tag => write!(f, "tag"),
            ContentType::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Event actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventAction {
    Created,
    Updated,
    Deleted,
    Published,
    Unpublished,
    Trashed,
    Restored,
}

impl std::fmt::Display for EventAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventAction::Created => write!(f, "created"),
            EventAction::Updated => write!(f, "updated"),
            EventAction::Deleted => write!(f, "deleted"),
            EventAction::Published => write!(f, "published"),
            EventAction::Unpublished => write!(f, "unpublished"),
            EventAction::Trashed => write!(f, "trashed"),
            EventAction::Restored => write!(f, "restored"),
        }
    }
}

/// Content change event for triggering auto-purge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentChangeEvent {
    /// Type of content that changed
    pub content_type: ContentType,
    /// What happened to the content
    pub action: EventAction,
    /// ID of the content item
    pub content_id: Option<String>,
    /// URL of the content (if applicable)
    pub url: Option<String>,
    /// Slug of the content
    pub slug: Option<String>,
    /// Title of the content
    pub title: Option<String>,
    /// Additional URLs to purge
    pub related_urls: Vec<String>,
    /// User who made the change
    pub user_id: Option<String>,
    /// Timestamp of the event
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ContentChangeEvent {
    pub fn new(content_type: ContentType, action: EventAction) -> Self {
        Self {
            content_type,
            action,
            content_id: None,
            url: None,
            slug: None,
            title: None,
            related_urls: Vec::new(),
            user_id: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.content_id = Some(id.into());
        self
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn with_slug(mut self, slug: impl Into<String>) -> Self {
        self.slug = Some(slug.into());
        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_related_urls(mut self, urls: Vec<String>) -> Self {
        self.related_urls = urls;
        self
    }

    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }
}

/// Auto-purge hooks manager
pub struct AutoPurgeHooks {
    services: Option<Arc<CloudflareServices>>,
    config: RwLock<AutoPurgeConfig>,
    db: PgPool,
    site_url: String,
}

impl AutoPurgeHooks {
    /// Create a new hooks manager
    pub fn new(db: PgPool, site_url: String) -> Self {
        Self {
            services: None,
            config: RwLock::new(AutoPurgeConfig::new()),
            db,
            site_url,
        }
    }

    /// Set the Cloudflare services instance
    pub fn set_services(&mut self, services: Arc<CloudflareServices>) {
        self.services = Some(services);
    }

    /// Update the auto-purge configuration
    pub async fn update_config(&self, config: AutoPurgeConfig) {
        *self.config.write().await = config;
    }

    /// Get the current configuration
    pub async fn get_config(&self) -> AutoPurgeConfig {
        self.config.read().await.clone()
    }

    /// Load configuration from database
    pub async fn load_config(&self) -> CloudflareResult<()> {
        let result: Option<(Option<serde_json::Value>,)> = sqlx::query_as(
            r#"SELECT value FROM cloudflare_settings WHERE key = 'auto_purge_config'"#,
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| CloudflareError::DatabaseError(e.to_string()))?;

        if let Some((Some(value),)) = result {
            if let Ok(config) = serde_json::from_value::<AutoPurgeConfig>(value) {
                *self.config.write().await = config;
            }
        }

        Ok(())
    }

    /// Save configuration to database
    pub async fn save_config(&self) -> CloudflareResult<()> {
        let config = self.config.read().await.clone();
        let value = serde_json::to_value(&config)
            .map_err(|e| CloudflareError::ConfigError(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO cloudflare_settings (key, value, updated_at)
            VALUES ('auto_purge_config', $1, NOW())
            ON CONFLICT (key) DO UPDATE SET value = $1, updated_at = NOW()
            "#,
        )
        .bind(&value)
        .execute(&self.db)
        .await
        .map_err(|e| CloudflareError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Handle a content change event
    pub async fn handle_event(&self, event: ContentChangeEvent) -> CloudflareResult<()> {
        let config = self.config.read().await.clone();

        if !config.enabled {
            debug!("Auto-purge disabled, skipping event: {:?}", event);
            return Ok(());
        }

        // Check if we should handle this content type
        let should_purge = match event.content_type {
            ContentType::Post => config.on_post_update,
            ContentType::Page => config.on_page_update,
            ContentType::Media => config.on_media_change,
            ContentType::Theme => config.on_theme_change,
            ContentType::Menu => config.on_menu_update,
            ContentType::Widget => config.on_widget_update,
            ContentType::Settings => config.on_settings_change,
            ContentType::Comment | ContentType::Category | ContentType::Tag => config.on_post_update,
            _ => false,
        };

        if !should_purge {
            debug!("Auto-purge not configured for content type: {}", event.content_type);
            return Ok(());
        }

        // Log the event
        self.log_event(&event).await?;

        // Get the services
        let services = match &self.services {
            Some(s) => s,
            None => {
                warn!("Cloudflare services not configured, skipping auto-purge");
                return Ok(());
            }
        };

        // Apply delay if configured
        if config.purge_delay_ms > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(config.purge_delay_ms as u64)).await;
        }

        // Purge entire site or specific URLs
        if config.purge_entire_site {
            info!("Auto-purging entire site cache due to {} {}", event.content_type, event.action);
            services.cache.purge_all().await?;
        } else {
            // Collect URLs to purge
            let mut urls_to_purge = self.collect_urls_to_purge(&event, &config).await;

            if urls_to_purge.is_empty() {
                debug!("No URLs to purge for event");
                return Ok(());
            }

            // Deduplicate
            urls_to_purge.sort();
            urls_to_purge.dedup();

            info!(
                "Auto-purging {} URLs due to {} {}",
                urls_to_purge.len(),
                event.content_type,
                event.action
            );

            // Cloudflare allows up to 30 URLs per purge request
            for chunk in urls_to_purge.chunks(30) {
                services.cache.purge_urls(chunk.to_vec()).await?;
            }
        }

        Ok(())
    }

    /// Collect URLs to purge based on the event
    async fn collect_urls_to_purge(&self, event: &ContentChangeEvent, config: &AutoPurgeConfig) -> Vec<String> {
        let mut urls = Vec::new();
        let site_url = self.site_url.trim_end_matches('/');

        // Add the content URL if available
        if let Some(url) = &event.url {
            urls.push(url.clone());
            // Also add with trailing slash
            if !url.ends_with('/') {
                urls.push(format!("{}/", url));
            }
        }

        // Add related URLs
        urls.extend(event.related_urls.clone());

        // Add homepage if configured
        if config.always_purge_homepage {
            urls.push(format!("{}/", site_url));
            urls.push(site_url.to_string());
        }

        // Add archive pages if configured
        if config.purge_archives {
            match event.content_type {
                ContentType::Post => {
                    urls.push(format!("{}/blog/", site_url));
                    urls.push(format!("{}/blog", site_url));
                    urls.push(format!("{}/posts/", site_url));
                    urls.push(format!("{}/posts", site_url));
                    // Year/month archives could be added based on post date
                }
                ContentType::Category => {
                    urls.push(format!("{}/category/", site_url));
                    if let Some(slug) = &event.slug {
                        urls.push(format!("{}/category/{}/", site_url, slug));
                        urls.push(format!("{}/category/{}", site_url, slug));
                    }
                }
                ContentType::Tag => {
                    urls.push(format!("{}/tag/", site_url));
                    if let Some(slug) = &event.slug {
                        urls.push(format!("{}/tag/{}/", site_url, slug));
                        urls.push(format!("{}/tag/{}", site_url, slug));
                    }
                }
                _ => {}
            }
        }

        // Add custom URLs if configured
        if let Some(custom) = &config.custom_purge_urls {
            for pattern in custom.split(',') {
                let pattern = pattern.trim();
                if !pattern.is_empty() {
                    if pattern.starts_with("http") {
                        urls.push(pattern.to_string());
                    } else {
                        urls.push(format!("{}{}", site_url, pattern));
                    }
                }
            }
        }

        // Add sitemap URLs (good practice to purge these)
        urls.push(format!("{}/sitemap.xml", site_url));
        urls.push(format!("{}/sitemap_index.xml", site_url));
        urls.push(format!("{}/feed/", site_url));
        urls.push(format!("{}/rss/", site_url));

        urls
    }

    /// Log a purge event to the database
    async fn log_event(&self, event: &ContentChangeEvent) -> CloudflareResult<()> {
        let details = serde_json::to_value(event)
            .map_err(|e| CloudflareError::ConfigError(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO cloudflare_cache_events (event_type, details, created_at)
            VALUES ($1, $2, NOW())
            "#,
        )
        .bind(format!("auto_purge_{}", event.content_type))
        .bind(&details)
        .execute(&self.db)
        .await
        .map_err(|e| CloudflareError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

// Convenience functions for creating events
impl ContentChangeEvent {
    /// Create a post published event
    pub fn post_published(id: &str, url: &str, title: &str) -> Self {
        Self::new(ContentType::Post, EventAction::Published)
            .with_id(id)
            .with_url(url)
            .with_title(title)
    }

    /// Create a post updated event
    pub fn post_updated(id: &str, url: &str, title: &str) -> Self {
        Self::new(ContentType::Post, EventAction::Updated)
            .with_id(id)
            .with_url(url)
            .with_title(title)
    }

    /// Create a page published event
    pub fn page_published(id: &str, url: &str, title: &str) -> Self {
        Self::new(ContentType::Page, EventAction::Published)
            .with_id(id)
            .with_url(url)
            .with_title(title)
    }

    /// Create a media uploaded event
    pub fn media_uploaded(id: &str, url: &str) -> Self {
        Self::new(ContentType::Media, EventAction::Created)
            .with_id(id)
            .with_url(url)
    }

    /// Create a media deleted event
    pub fn media_deleted(id: &str) -> Self {
        Self::new(ContentType::Media, EventAction::Deleted)
            .with_id(id)
    }

    /// Create a theme changed event
    pub fn theme_changed(theme_name: &str) -> Self {
        Self::new(ContentType::Theme, EventAction::Updated)
            .with_title(theme_name)
    }

    /// Create a menu updated event
    pub fn menu_updated(menu_id: &str, menu_name: &str) -> Self {
        Self::new(ContentType::Menu, EventAction::Updated)
            .with_id(menu_id)
            .with_title(menu_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_purge_config_defaults() {
        let config = AutoPurgeConfig::new();
        assert!(config.enabled);
        assert!(config.on_post_update);
        assert!(config.always_purge_homepage);
    }

    #[test]
    fn test_content_change_event_builder() {
        let event = ContentChangeEvent::post_published("123", "https://example.com/post/123", "Test Post");
        assert_eq!(event.content_type, ContentType::Post);
        assert_eq!(event.action, EventAction::Published);
        assert_eq!(event.content_id, Some("123".to_string()));
        assert_eq!(event.url, Some("https://example.com/post/123".to_string()));
    }
}
