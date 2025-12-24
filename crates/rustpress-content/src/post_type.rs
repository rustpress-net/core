//! Post Type Registration System (Point 81)
//!
//! Provides a flexible system for registering custom content types
//! similar to WordPress's register_post_type functionality.

use chrono::{DateTime, Utc};
use rustpress_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

/// Post type identifier
pub type PostTypeId = String;

/// Post type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostType {
    /// Unique identifier (e.g., "post", "page", "product")
    pub name: PostTypeId,
    /// Singular label
    pub singular_label: String,
    /// Plural label
    pub plural_label: String,
    /// Description
    pub description: Option<String>,
    /// Whether this post type is public
    pub public: bool,
    /// Show in admin UI
    pub show_ui: bool,
    /// Show in navigation menus
    pub show_in_nav_menus: bool,
    /// Show in REST API
    pub show_in_rest: bool,
    /// REST API base slug
    pub rest_base: Option<String>,
    /// Hierarchical (like pages) or flat (like posts)
    pub hierarchical: bool,
    /// Supports which features
    pub supports: PostTypeSupports,
    /// URL rewrite settings
    pub rewrite: Option<RewriteConfig>,
    /// Taxonomies to associate
    pub taxonomies: Vec<String>,
    /// Menu icon
    pub menu_icon: Option<String>,
    /// Menu position
    pub menu_position: Option<i32>,
    /// Capability type for permissions
    pub capability_type: String,
    /// Custom capabilities mapping
    pub capabilities: PostTypeCapabilities,
    /// Whether to enable archives
    pub has_archive: bool,
    /// Archive slug
    pub archive_slug: Option<String>,
    /// Exclude from search
    pub exclude_from_search: bool,
    /// Publicly queryable
    pub publicly_queryable: bool,
    /// Can export
    pub can_export: bool,
    /// Delete with user (when user is deleted)
    pub delete_with_user: bool,
    /// Built-in type (cannot be unregistered)
    pub builtin: bool,
    /// Custom labels
    pub labels: PostTypeLabels,
    /// Registration timestamp
    pub registered_at: DateTime<Utc>,
}

impl PostType {
    /// Create a new post type builder
    pub fn builder(name: impl Into<String>) -> PostTypeBuilder {
        PostTypeBuilder::new(name)
    }

    /// Check if post type supports a feature
    pub fn supports_feature(&self, feature: &str) -> bool {
        match feature {
            "title" => self.supports.title,
            "editor" => self.supports.editor,
            "author" => self.supports.author,
            "thumbnail" => self.supports.thumbnail,
            "excerpt" => self.supports.excerpt,
            "trackbacks" => self.supports.trackbacks,
            "custom-fields" => self.supports.custom_fields,
            "comments" => self.supports.comments,
            "revisions" => self.supports.revisions,
            "page-attributes" => self.supports.page_attributes,
            "post-formats" => self.supports.post_formats,
            _ => self.supports.custom.contains(&feature.to_string()),
        }
    }
}

/// Post type feature support flags
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PostTypeSupports {
    pub title: bool,
    pub editor: bool,
    pub author: bool,
    pub thumbnail: bool,
    pub excerpt: bool,
    pub trackbacks: bool,
    pub custom_fields: bool,
    pub comments: bool,
    pub revisions: bool,
    pub page_attributes: bool,
    pub post_formats: bool,
    /// Custom feature flags
    pub custom: Vec<String>,
}

impl PostTypeSupports {
    /// Default supports for posts
    pub fn post_defaults() -> Self {
        Self {
            title: true,
            editor: true,
            author: true,
            thumbnail: true,
            excerpt: true,
            trackbacks: false,
            custom_fields: true,
            comments: true,
            revisions: true,
            page_attributes: false,
            post_formats: true,
            custom: Vec::new(),
        }
    }

    /// Default supports for pages
    pub fn page_defaults() -> Self {
        Self {
            title: true,
            editor: true,
            author: true,
            thumbnail: true,
            excerpt: false,
            trackbacks: false,
            custom_fields: true,
            comments: true,
            revisions: true,
            page_attributes: true,
            post_formats: false,
            custom: Vec::new(),
        }
    }

    /// Minimal supports
    pub fn minimal() -> Self {
        Self {
            title: true,
            editor: true,
            ..Default::default()
        }
    }
}

/// URL rewrite configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewriteConfig {
    /// URL slug
    pub slug: String,
    /// Include front base
    pub with_front: bool,
    /// Enable feeds
    pub feeds: bool,
    /// Enable pagination
    pub pages: bool,
    /// Custom endpoints
    pub endpoints: Vec<EndpointConfig>,
}

impl Default for RewriteConfig {
    fn default() -> Self {
        Self {
            slug: String::new(),
            with_front: true,
            feeds: true,
            pages: true,
            endpoints: Vec::new(),
        }
    }
}

/// Custom endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    pub name: String,
    pub mask: u32,
}

/// Post type capabilities mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostTypeCapabilities {
    pub edit_post: String,
    pub read_post: String,
    pub delete_post: String,
    pub edit_posts: String,
    pub edit_others_posts: String,
    pub delete_posts: String,
    pub publish_posts: String,
    pub read_private_posts: String,
    pub create_posts: String,
    pub delete_private_posts: String,
    pub delete_published_posts: String,
    pub delete_others_posts: String,
    pub edit_private_posts: String,
    pub edit_published_posts: String,
}

impl PostTypeCapabilities {
    pub fn for_type(capability_type: &str) -> Self {
        Self {
            edit_post: format!("edit_{}", capability_type),
            read_post: format!("read_{}", capability_type),
            delete_post: format!("delete_{}", capability_type),
            edit_posts: format!("edit_{}s", capability_type),
            edit_others_posts: format!("edit_others_{}s", capability_type),
            delete_posts: format!("delete_{}s", capability_type),
            publish_posts: format!("publish_{}s", capability_type),
            read_private_posts: format!("read_private_{}s", capability_type),
            create_posts: format!("create_{}s", capability_type),
            delete_private_posts: format!("delete_private_{}s", capability_type),
            delete_published_posts: format!("delete_published_{}s", capability_type),
            delete_others_posts: format!("delete_others_{}s", capability_type),
            edit_private_posts: format!("edit_private_{}s", capability_type),
            edit_published_posts: format!("edit_published_{}s", capability_type),
        }
    }
}

/// Post type labels for UI
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PostTypeLabels {
    pub name: String,
    pub singular_name: String,
    pub add_new: String,
    pub add_new_item: String,
    pub edit_item: String,
    pub new_item: String,
    pub view_item: String,
    pub view_items: String,
    pub search_items: String,
    pub not_found: String,
    pub not_found_in_trash: String,
    pub parent_item_colon: Option<String>,
    pub all_items: String,
    pub archives: String,
    pub attributes: String,
    pub insert_into_item: String,
    pub uploaded_to_this_item: String,
    pub featured_image: String,
    pub set_featured_image: String,
    pub remove_featured_image: String,
    pub use_featured_image: String,
    pub menu_name: String,
    pub filter_items_list: String,
    pub items_list_navigation: String,
    pub items_list: String,
    pub item_published: String,
    pub item_published_privately: String,
    pub item_reverted_to_draft: String,
    pub item_scheduled: String,
    pub item_updated: String,
}

impl PostTypeLabels {
    pub fn generate(singular: &str, plural: &str) -> Self {
        Self {
            name: plural.to_string(),
            singular_name: singular.to_string(),
            add_new: "Add New".to_string(),
            add_new_item: format!("Add New {}", singular),
            edit_item: format!("Edit {}", singular),
            new_item: format!("New {}", singular),
            view_item: format!("View {}", singular),
            view_items: format!("View {}", plural),
            search_items: format!("Search {}", plural),
            not_found: format!("No {} found", plural.to_lowercase()),
            not_found_in_trash: format!("No {} found in Trash", plural.to_lowercase()),
            parent_item_colon: Some(format!("Parent {}:", singular)),
            all_items: format!("All {}", plural),
            archives: format!("{} Archives", singular),
            attributes: format!("{} Attributes", singular),
            insert_into_item: format!("Insert into {}", singular.to_lowercase()),
            uploaded_to_this_item: format!("Uploaded to this {}", singular.to_lowercase()),
            featured_image: "Featured image".to_string(),
            set_featured_image: "Set featured image".to_string(),
            remove_featured_image: "Remove featured image".to_string(),
            use_featured_image: "Use as featured image".to_string(),
            menu_name: plural.to_string(),
            filter_items_list: format!("Filter {} list", plural.to_lowercase()),
            items_list_navigation: format!("{} list navigation", plural),
            items_list: format!("{} list", plural),
            item_published: format!("{} published.", singular),
            item_published_privately: format!("{} published privately.", singular),
            item_reverted_to_draft: format!("{} reverted to draft.", singular),
            item_scheduled: format!("{} scheduled.", singular),
            item_updated: format!("{} updated.", singular),
        }
    }
}

/// Post type builder for fluent API
pub struct PostTypeBuilder {
    name: String,
    singular_label: String,
    plural_label: String,
    description: Option<String>,
    public: bool,
    show_ui: bool,
    show_in_nav_menus: bool,
    show_in_rest: bool,
    rest_base: Option<String>,
    hierarchical: bool,
    supports: PostTypeSupports,
    rewrite: Option<RewriteConfig>,
    taxonomies: Vec<String>,
    menu_icon: Option<String>,
    menu_position: Option<i32>,
    capability_type: String,
    has_archive: bool,
    archive_slug: Option<String>,
    exclude_from_search: bool,
    publicly_queryable: bool,
    can_export: bool,
    delete_with_user: bool,
}

impl PostTypeBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            name: name.clone(),
            singular_label: name.clone(),
            plural_label: format!("{}s", name),
            description: None,
            public: true,
            show_ui: true,
            show_in_nav_menus: true,
            show_in_rest: true,
            rest_base: None,
            hierarchical: false,
            supports: PostTypeSupports::post_defaults(),
            rewrite: None,
            taxonomies: Vec::new(),
            menu_icon: None,
            menu_position: None,
            capability_type: "post".to_string(),
            has_archive: true,
            archive_slug: None,
            exclude_from_search: false,
            publicly_queryable: true,
            can_export: true,
            delete_with_user: false,
        }
    }

    pub fn labels(mut self, singular: impl Into<String>, plural: impl Into<String>) -> Self {
        self.singular_label = singular.into();
        self.plural_label = plural.into();
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn public(mut self, public: bool) -> Self {
        self.public = public;
        if !public {
            self.show_ui = false;
            self.show_in_nav_menus = false;
            self.publicly_queryable = false;
        }
        self
    }

    pub fn show_ui(mut self, show: bool) -> Self {
        self.show_ui = show;
        self
    }

    pub fn show_in_rest(mut self, show: bool) -> Self {
        self.show_in_rest = show;
        self
    }

    pub fn rest_base(mut self, base: impl Into<String>) -> Self {
        self.rest_base = Some(base.into());
        self
    }

    pub fn hierarchical(mut self, hierarchical: bool) -> Self {
        self.hierarchical = hierarchical;
        if hierarchical {
            self.supports = PostTypeSupports::page_defaults();
        }
        self
    }

    pub fn supports(mut self, supports: PostTypeSupports) -> Self {
        self.supports = supports;
        self
    }

    pub fn add_support(mut self, feature: &str) -> Self {
        match feature {
            "title" => self.supports.title = true,
            "editor" => self.supports.editor = true,
            "author" => self.supports.author = true,
            "thumbnail" => self.supports.thumbnail = true,
            "excerpt" => self.supports.excerpt = true,
            "trackbacks" => self.supports.trackbacks = true,
            "custom-fields" => self.supports.custom_fields = true,
            "comments" => self.supports.comments = true,
            "revisions" => self.supports.revisions = true,
            "page-attributes" => self.supports.page_attributes = true,
            "post-formats" => self.supports.post_formats = true,
            _ => self.supports.custom.push(feature.to_string()),
        }
        self
    }

    pub fn remove_support(mut self, feature: &str) -> Self {
        match feature {
            "title" => self.supports.title = false,
            "editor" => self.supports.editor = false,
            "author" => self.supports.author = false,
            "thumbnail" => self.supports.thumbnail = false,
            "excerpt" => self.supports.excerpt = false,
            "trackbacks" => self.supports.trackbacks = false,
            "custom-fields" => self.supports.custom_fields = false,
            "comments" => self.supports.comments = false,
            "revisions" => self.supports.revisions = false,
            "page-attributes" => self.supports.page_attributes = false,
            "post-formats" => self.supports.post_formats = false,
            _ => self.supports.custom.retain(|f| f != feature),
        }
        self
    }

    pub fn rewrite(mut self, slug: impl Into<String>) -> Self {
        self.rewrite = Some(RewriteConfig {
            slug: slug.into(),
            ..Default::default()
        });
        self
    }

    pub fn rewrite_config(mut self, config: RewriteConfig) -> Self {
        self.rewrite = Some(config);
        self
    }

    pub fn taxonomies(mut self, taxonomies: Vec<String>) -> Self {
        self.taxonomies = taxonomies;
        self
    }

    pub fn add_taxonomy(mut self, taxonomy: impl Into<String>) -> Self {
        self.taxonomies.push(taxonomy.into());
        self
    }

    pub fn menu_icon(mut self, icon: impl Into<String>) -> Self {
        self.menu_icon = Some(icon.into());
        self
    }

    pub fn menu_position(mut self, position: i32) -> Self {
        self.menu_position = Some(position);
        self
    }

    pub fn capability_type(mut self, cap_type: impl Into<String>) -> Self {
        self.capability_type = cap_type.into();
        self
    }

    pub fn has_archive(mut self, has_archive: bool) -> Self {
        self.has_archive = has_archive;
        self
    }

    pub fn archive_slug(mut self, slug: impl Into<String>) -> Self {
        self.archive_slug = Some(slug.into());
        self.has_archive = true;
        self
    }

    pub fn exclude_from_search(mut self, exclude: bool) -> Self {
        self.exclude_from_search = exclude;
        self
    }

    pub fn can_export(mut self, can_export: bool) -> Self {
        self.can_export = can_export;
        self
    }

    pub fn delete_with_user(mut self, delete: bool) -> Self {
        self.delete_with_user = delete;
        self
    }

    pub fn build(self) -> PostType {
        let labels = PostTypeLabels::generate(&self.singular_label, &self.plural_label);
        let capabilities = PostTypeCapabilities::for_type(&self.capability_type);

        PostType {
            name: self.name.clone(),
            singular_label: self.singular_label,
            plural_label: self.plural_label,
            description: self.description,
            public: self.public,
            show_ui: self.show_ui,
            show_in_nav_menus: self.show_in_nav_menus,
            show_in_rest: self.show_in_rest,
            rest_base: self.rest_base.or_else(|| Some(self.name.clone())),
            hierarchical: self.hierarchical,
            supports: self.supports,
            rewrite: self.rewrite.or_else(|| Some(RewriteConfig {
                slug: self.name.clone(),
                ..Default::default()
            })),
            taxonomies: self.taxonomies,
            menu_icon: self.menu_icon,
            menu_position: self.menu_position,
            capability_type: self.capability_type,
            capabilities,
            has_archive: self.has_archive,
            archive_slug: self.archive_slug,
            exclude_from_search: self.exclude_from_search,
            publicly_queryable: self.publicly_queryable,
            can_export: self.can_export,
            delete_with_user: self.delete_with_user,
            builtin: false,
            labels,
            registered_at: Utc::now(),
        }
    }
}

/// Post type registry
pub struct PostTypeRegistry {
    types: RwLock<HashMap<PostTypeId, PostType>>,
}

impl PostTypeRegistry {
    pub fn new() -> Self {
        let registry = Self {
            types: RwLock::new(HashMap::new()),
        };
        registry.register_builtin_types();
        registry
    }

    /// Register built-in post types
    fn register_builtin_types(&self) {
        // Post type
        let post = {
            let mut pt = PostType::builder("post")
                .labels("Post", "Posts")
                .description("Standard blog posts")
                .supports(PostTypeSupports::post_defaults())
                .add_taxonomy("category")
                .add_taxonomy("tag")
                .menu_icon("dashicons-admin-post")
                .menu_position(5)
                .build();
            pt.builtin = true;
            pt
        };

        // Page type
        let page = {
            let mut pt = PostType::builder("page")
                .labels("Page", "Pages")
                .description("Static pages")
                .hierarchical(true)
                .supports(PostTypeSupports::page_defaults())
                .menu_icon("dashicons-admin-page")
                .menu_position(20)
                .has_archive(false)
                .build();
            pt.builtin = true;
            pt
        };

        // Attachment type
        let attachment = {
            let mut pt = PostType::builder("attachment")
                .labels("Media", "Media")
                .description("Uploaded media files")
                .public(false)
                .show_ui(true)
                .supports(PostTypeSupports {
                    title: true,
                    author: true,
                    comments: true,
                    ..Default::default()
                })
                .build();
            pt.builtin = true;
            pt
        };

        // Revision type (internal)
        let revision = {
            let mut pt = PostType::builder("revision")
                .labels("Revision", "Revisions")
                .description("Post revisions")
                .public(false)
                .show_ui(false)
                .show_in_rest(false)
                .supports(PostTypeSupports::minimal())
                .build();
            pt.builtin = true;
            pt
        };

        // Nav menu item (internal)
        let nav_menu_item = {
            let mut pt = PostType::builder("nav_menu_item")
                .labels("Navigation Menu Item", "Navigation Menu Items")
                .description("Navigation menu items")
                .public(false)
                .show_ui(false)
                .show_in_rest(false)
                .supports(PostTypeSupports::minimal())
                .build();
            pt.builtin = true;
            pt
        };

        let mut types = self.types.write().unwrap();
        types.insert("post".to_string(), post);
        types.insert("page".to_string(), page);
        types.insert("attachment".to_string(), attachment);
        types.insert("revision".to_string(), revision);
        types.insert("nav_menu_item".to_string(), nav_menu_item);
    }

    /// Register a new post type
    pub fn register(&self, post_type: PostType) -> Result<()> {
        let mut types = self.types.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;

        if types.contains_key(&post_type.name) {
            return Err(Error::InvalidInput {
                field: "name".to_string(),
                message: format!("Post type '{}' already registered", post_type.name),
            });
        }

        // Validate name
        if post_type.name.is_empty() || post_type.name.len() > 20 {
            return Err(Error::InvalidInput {
                field: "name".to_string(),
                message: "Post type name must be 1-20 characters".to_string(),
            });
        }

        if !post_type.name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(Error::InvalidInput {
                field: "name".to_string(),
                message: "Post type name must be alphanumeric with underscores/hyphens".to_string(),
            });
        }

        types.insert(post_type.name.clone(), post_type);
        Ok(())
    }

    /// Unregister a post type
    pub fn unregister(&self, name: &str) -> Result<()> {
        let mut types = self.types.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;

        if let Some(pt) = types.get(name) {
            if pt.builtin {
                return Err(Error::InvalidInput {
                    field: "name".to_string(),
                    message: "Cannot unregister built-in post type".to_string(),
                });
            }
        }

        types.remove(name);
        Ok(())
    }

    /// Get a post type by name
    pub fn get(&self, name: &str) -> Option<PostType> {
        self.types.read().ok()?.get(name).cloned()
    }

    /// Get all registered post types
    pub fn get_all(&self) -> Vec<PostType> {
        self.types.read().map(|t| t.values().cloned().collect()).unwrap_or_default()
    }

    /// Get public post types
    pub fn get_public(&self) -> Vec<PostType> {
        self.get_all().into_iter().filter(|pt| pt.public).collect()
    }

    /// Get post types that show in REST API
    pub fn get_rest_enabled(&self) -> Vec<PostType> {
        self.get_all().into_iter().filter(|pt| pt.show_in_rest).collect()
    }

    /// Check if a post type exists
    pub fn exists(&self, name: &str) -> bool {
        self.types.read().map(|t| t.contains_key(name)).unwrap_or(false)
    }

    /// Check if post type supports a feature
    pub fn supports(&self, post_type: &str, feature: &str) -> bool {
        self.get(post_type).map(|pt| pt.supports_feature(feature)).unwrap_or(false)
    }
}

impl Default for PostTypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_types() {
        let registry = PostTypeRegistry::new();

        assert!(registry.exists("post"));
        assert!(registry.exists("page"));
        assert!(registry.exists("attachment"));

        let post = registry.get("post").unwrap();
        assert!(post.builtin);
        assert!(post.public);
        assert!(post.supports_feature("title"));
        assert!(post.supports_feature("editor"));
    }

    #[test]
    fn test_register_custom_type() {
        let registry = PostTypeRegistry::new();

        let product = PostType::builder("product")
            .labels("Product", "Products")
            .description("E-commerce products")
            .add_taxonomy("product_category")
            .add_support("price")
            .menu_icon("dashicons-cart")
            .build();

        registry.register(product).unwrap();

        let retrieved = registry.get("product").unwrap();
        assert_eq!(retrieved.singular_label, "Product");
        assert!(retrieved.supports_feature("price"));
    }

    #[test]
    fn test_cannot_unregister_builtin() {
        let registry = PostTypeRegistry::new();

        let result = registry.unregister("post");
        assert!(result.is_err());
    }

    #[test]
    fn test_post_type_builder() {
        let pt = PostType::builder("event")
            .labels("Event", "Events")
            .description("Calendar events")
            .hierarchical(false)
            .has_archive(true)
            .archive_slug("events")
            .menu_position(25)
            .build();

        assert_eq!(pt.name, "event");
        assert_eq!(pt.singular_label, "Event");
        assert!(pt.has_archive);
        assert_eq!(pt.archive_slug, Some("events".to_string()));
    }

    #[test]
    fn test_capabilities_generation() {
        let caps = PostTypeCapabilities::for_type("product");

        assert_eq!(caps.edit_post, "edit_product");
        assert_eq!(caps.edit_posts, "edit_products");
        assert_eq!(caps.publish_posts, "publish_products");
    }
}
