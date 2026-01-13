//! Theme Starter Content
//!
//! Pre-defined content that gets installed when a theme is activated.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;
use tokio::fs;

/// Starter content errors
#[derive(Debug, Error)]
pub enum StarterContentError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Content creation failed: {0}")]
    CreationFailed(String),
}

/// Starter content definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarterContent {
    /// Posts to create
    #[serde(default)]
    pub posts: Vec<StarterPost>,
    /// Pages to create
    #[serde(default)]
    pub pages: Vec<StarterPage>,
    /// Attachments (media)
    #[serde(default)]
    pub attachments: Vec<StarterAttachment>,
    /// Theme locations (menus assigned to locations)
    #[serde(default)]
    pub theme_locations: HashMap<String, String>,
    /// Nav menus to create
    #[serde(default)]
    pub nav_menus: HashMap<String, StarterNavMenu>,
    /// Widgets to configure
    #[serde(default)]
    pub widgets: HashMap<String, Vec<StarterWidget>>,
    /// Theme mods to set
    #[serde(default)]
    pub theme_mods: HashMap<String, serde_json::Value>,
    /// Options to set
    #[serde(default)]
    pub options: HashMap<String, serde_json::Value>,
}

/// Starter post
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarterPost {
    pub id: String,
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub excerpt: Option<String>,
    #[serde(default)]
    pub post_type: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub featured_image: Option<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub meta: HashMap<String, serde_json::Value>,
}

impl Default for StarterPost {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            content: String::new(),
            excerpt: None,
            post_type: "post".to_string(),
            status: "publish".to_string(),
            featured_image: None,
            categories: Vec::new(),
            tags: Vec::new(),
            meta: HashMap::new(),
        }
    }
}

/// Starter page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarterPage {
    pub id: String,
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub menu_order: i32,
    #[serde(default)]
    pub featured_image: Option<String>,
    #[serde(default)]
    pub meta: HashMap<String, serde_json::Value>,
}

/// Starter attachment (media)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarterAttachment {
    pub id: String,
    pub file: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub alt: Option<String>,
    #[serde(default)]
    pub caption: Option<String>,
}

/// Starter navigation menu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarterNavMenu {
    pub name: String,
    pub items: Vec<StarterMenuItem>,
}

/// Starter menu item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarterMenuItem {
    pub title: String,
    #[serde(flatten)]
    pub item_type: MenuItemType,
    #[serde(default)]
    pub children: Vec<StarterMenuItem>,
}

/// Menu item type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MenuItemType {
    Page { page_id: String },
    Post { post_id: String },
    Custom { url: String },
    Category { category_id: String },
    Archive { post_type: String },
    Home,
}

/// Starter widget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarterWidget {
    pub widget_type: String,
    #[serde(default)]
    pub settings: HashMap<String, serde_json::Value>,
}

impl StarterContent {
    pub fn new() -> Self {
        Self {
            posts: Vec::new(),
            pages: Vec::new(),
            attachments: Vec::new(),
            theme_locations: HashMap::new(),
            nav_menus: HashMap::new(),
            widgets: HashMap::new(),
            theme_mods: HashMap::new(),
            options: HashMap::new(),
        }
    }

    /// Load from file
    pub async fn load_from_file(path: &Path) -> Result<Self, StarterContentError> {
        let content = fs::read_to_string(path).await?;

        if path.extension().map_or(false, |e| e == "json") {
            serde_json::from_str(&content)
                .map_err(|e| StarterContentError::Parse(e.to_string()))
        } else {
            toml::from_str(&content)
                .map_err(|e| StarterContentError::Parse(e.to_string()))
        }
    }

    /// Load from theme directory
    pub async fn load_from_theme(theme_path: &Path) -> Result<Option<Self>, StarterContentError> {
        let starter_toml = theme_path.join("starter-content.toml");
        let starter_json = theme_path.join("starter-content.json");

        if starter_toml.exists() {
            Ok(Some(Self::load_from_file(&starter_toml).await?))
        } else if starter_json.exists() {
            Ok(Some(Self::load_from_file(&starter_json).await?))
        } else {
            Ok(None)
        }
    }

    /// Add a page
    pub fn add_page(&mut self, page: StarterPage) -> &mut Self {
        self.pages.push(page);
        self
    }

    /// Add a post
    pub fn add_post(&mut self, post: StarterPost) -> &mut Self {
        self.posts.push(post);
        self
    }

    /// Add an attachment
    pub fn add_attachment(&mut self, attachment: StarterAttachment) -> &mut Self {
        self.attachments.push(attachment);
        self
    }

    /// Add a nav menu
    pub fn add_nav_menu(&mut self, location: &str, menu: StarterNavMenu) -> &mut Self {
        let menu_name = menu.name.clone();
        self.nav_menus.insert(location.to_string(), menu);
        self.theme_locations.insert(location.to_string(), menu_name);
        self
    }

    /// Set a theme mod
    pub fn set_theme_mod(&mut self, key: &str, value: serde_json::Value) -> &mut Self {
        self.theme_mods.insert(key.to_string(), value);
        self
    }

    /// Set an option
    pub fn set_option(&mut self, key: &str, value: serde_json::Value) -> &mut Self {
        self.options.insert(key.to_string(), value);
        self
    }
}

impl Default for StarterContent {
    fn default() -> Self {
        Self::new()
    }
}

/// Starter content installer
pub struct StarterContentInstaller {
    content: StarterContent,
    #[allow(dead_code)]
    theme_path: std::path::PathBuf,
    dry_run: bool,
}

/// Installation result
#[derive(Debug, Clone)]
pub struct InstallationResult {
    pub posts_created: Vec<String>,
    pub pages_created: Vec<String>,
    pub attachments_created: Vec<String>,
    pub menus_created: Vec<String>,
    pub widgets_configured: Vec<String>,
    pub errors: Vec<String>,
}

impl StarterContentInstaller {
    pub fn new(content: StarterContent, theme_path: std::path::PathBuf) -> Self {
        Self {
            content,
            theme_path,
            dry_run: false,
        }
    }

    pub fn dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// Install all starter content
    pub async fn install(&self) -> Result<InstallationResult, StarterContentError> {
        let mut result = InstallationResult {
            posts_created: Vec::new(),
            pages_created: Vec::new(),
            attachments_created: Vec::new(),
            menus_created: Vec::new(),
            widgets_configured: Vec::new(),
            errors: Vec::new(),
        };

        // Install attachments first (they might be referenced by posts/pages)
        for attachment in &self.content.attachments {
            match self.install_attachment(attachment).await {
                Ok(_) => result.attachments_created.push(attachment.id.clone()),
                Err(e) => result.errors.push(format!("Attachment {}: {}", attachment.id, e)),
            }
        }

        // Install pages
        for page in &self.content.pages {
            match self.install_page(page).await {
                Ok(_) => result.pages_created.push(page.id.clone()),
                Err(e) => result.errors.push(format!("Page {}: {}", page.id, e)),
            }
        }

        // Install posts
        for post in &self.content.posts {
            match self.install_post(post).await {
                Ok(_) => result.posts_created.push(post.id.clone()),
                Err(e) => result.errors.push(format!("Post {}: {}", post.id, e)),
            }
        }

        // Install menus
        for (location, menu) in &self.content.nav_menus {
            match self.install_menu(location, menu).await {
                Ok(_) => result.menus_created.push(location.clone()),
                Err(e) => result.errors.push(format!("Menu {}: {}", location, e)),
            }
        }

        // Configure widgets
        for (sidebar, widgets) in &self.content.widgets {
            match self.configure_widgets(sidebar, widgets).await {
                Ok(_) => result.widgets_configured.push(sidebar.clone()),
                Err(e) => result.errors.push(format!("Sidebar {}: {}", sidebar, e)),
            }
        }

        Ok(result)
    }

    async fn install_attachment(&self, attachment: &StarterAttachment) -> Result<(), StarterContentError> {
        if self.dry_run {
            return Ok(());
        }

        // In a real implementation, this would:
        // 1. Copy the file from theme assets to uploads
        // 2. Create an attachment post
        // 3. Generate thumbnails

        tracing::debug!("Installing attachment: {}", attachment.id);
        Ok(())
    }

    async fn install_page(&self, page: &StarterPage) -> Result<(), StarterContentError> {
        if self.dry_run {
            return Ok(());
        }

        // In a real implementation, this would create the page
        tracing::debug!("Installing page: {} - {}", page.id, page.title);
        Ok(())
    }

    async fn install_post(&self, post: &StarterPost) -> Result<(), StarterContentError> {
        if self.dry_run {
            return Ok(());
        }

        // In a real implementation, this would create the post
        tracing::debug!("Installing post: {} - {}", post.id, post.title);
        Ok(())
    }

    async fn install_menu(&self, _location: &str, menu: &StarterNavMenu) -> Result<(), StarterContentError> {
        if self.dry_run {
            return Ok(());
        }

        // In a real implementation, this would create the menu and items
        tracing::debug!("Installing menu: {}", menu.name);
        Ok(())
    }

    async fn configure_widgets(&self, sidebar: &str, widgets: &[StarterWidget]) -> Result<(), StarterContentError> {
        if self.dry_run {
            return Ok(());
        }

        // In a real implementation, this would configure widgets
        tracing::debug!("Configuring {} widgets for sidebar: {}", widgets.len(), sidebar);
        Ok(())
    }
}

/// Create default starter content for a basic theme
pub fn create_default_starter_content() -> StarterContent {
    let mut content = StarterContent::new();

    // Home page
    content.add_page(StarterPage {
        id: "home".to_string(),
        title: "Home".to_string(),
        content: r#"<!-- wp:heading {"level":1} -->
<h1>Welcome to Your New Site</h1>
<!-- /wp:heading -->

<!-- wp:paragraph -->
<p>This is your homepage. Edit it to add your own content.</p>
<!-- /wp:paragraph -->
"#.to_string(),
        template: Some("front-page".to_string()),
        parent: None,
        menu_order: 0,
        featured_image: None,
        meta: HashMap::new(),
    });

    // About page
    content.add_page(StarterPage {
        id: "about".to_string(),
        title: "About".to_string(),
        content: r#"<!-- wp:heading {"level":1} -->
<h1>About Us</h1>
<!-- /wp:heading -->

<!-- wp:paragraph -->
<p>Tell your visitors about yourself and your site.</p>
<!-- /wp:paragraph -->
"#.to_string(),
        template: None,
        parent: None,
        menu_order: 1,
        featured_image: None,
        meta: HashMap::new(),
    });

    // Contact page
    content.add_page(StarterPage {
        id: "contact".to_string(),
        title: "Contact".to_string(),
        content: r#"<!-- wp:heading {"level":1} -->
<h1>Contact Us</h1>
<!-- /wp:heading -->

<!-- wp:paragraph -->
<p>Get in touch with us using the form below.</p>
<!-- /wp:paragraph -->
"#.to_string(),
        template: None,
        parent: None,
        menu_order: 2,
        featured_image: None,
        meta: HashMap::new(),
    });

    // Blog page
    content.add_page(StarterPage {
        id: "blog".to_string(),
        title: "Blog".to_string(),
        content: String::new(),
        template: Some("blog".to_string()),
        parent: None,
        menu_order: 3,
        featured_image: None,
        meta: HashMap::new(),
    });

    // Sample post
    content.add_post(StarterPost {
        id: "hello-world".to_string(),
        title: "Hello World".to_string(),
        content: r#"<!-- wp:paragraph -->
<p>Welcome to your new site! This is your first post. Edit or delete it, then start writing!</p>
<!-- /wp:paragraph -->
"#.to_string(),
        excerpt: Some("Welcome to your new site!".to_string()),
        categories: vec!["uncategorized".to_string()],
        ..Default::default()
    });

    // Primary menu
    content.add_nav_menu("primary", StarterNavMenu {
        name: "Primary Menu".to_string(),
        items: vec![
            StarterMenuItem {
                title: "Home".to_string(),
                item_type: MenuItemType::Page { page_id: "home".to_string() },
                children: Vec::new(),
            },
            StarterMenuItem {
                title: "About".to_string(),
                item_type: MenuItemType::Page { page_id: "about".to_string() },
                children: Vec::new(),
            },
            StarterMenuItem {
                title: "Blog".to_string(),
                item_type: MenuItemType::Page { page_id: "blog".to_string() },
                children: Vec::new(),
            },
            StarterMenuItem {
                title: "Contact".to_string(),
                item_type: MenuItemType::Page { page_id: "contact".to_string() },
                children: Vec::new(),
            },
        ],
    });

    // Set front page
    content.set_option("show_on_front", serde_json::json!("page"));
    content.set_option("page_on_front", serde_json::json!("home"));
    content.set_option("page_for_posts", serde_json::json!("blog"));

    content
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starter_content_creation() {
        let content = create_default_starter_content();

        assert!(!content.pages.is_empty());
        assert!(!content.posts.is_empty());
        assert!(!content.nav_menus.is_empty());
    }

    #[test]
    fn test_content_builder() {
        let mut content = StarterContent::new();

        content
            .add_page(StarterPage {
                id: "test".to_string(),
                title: "Test Page".to_string(),
                content: "Content".to_string(),
                template: None,
                parent: None,
                menu_order: 0,
                featured_image: None,
                meta: HashMap::new(),
            })
            .set_theme_mod("logo", serde_json::json!("/logo.png"));

        assert_eq!(content.pages.len(), 1);
        assert!(content.theme_mods.contains_key("logo"));
    }
}
