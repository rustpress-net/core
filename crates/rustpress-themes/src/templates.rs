//! Template Hierarchy System and Template Engine
//!
//! WordPress-compatible template hierarchy with Tera template engine.

use crate::manifest::TemplateSection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;
use tera::{Context, Tera};
use tracing::{debug, error, info};

// ============================================================================
// Template Hierarchy (Point 192)
// ============================================================================

/// Template hierarchy resolver
pub struct TemplateHierarchy {
    /// Custom hierarchy overrides
    overrides: HashMap<String, Vec<String>>,
    /// Available templates
    available: Arc<RwLock<HashMap<String, TemplateInfo>>>,
}

/// Template information
#[derive(Debug, Clone)]
pub struct TemplateInfo {
    pub name: String,
    pub path: PathBuf,
    pub template_type: TemplateType,
    pub post_types: Vec<String>,
}

/// Template type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateType {
    Index,
    Home,
    FrontPage,
    Single,
    Page,
    Archive,
    Category,
    Tag,
    Taxonomy,
    Author,
    Date,
    Search,
    NotFound,
    Attachment,
    Singular,
    Custom,
    Part,
}

/// Query context for template resolution
#[derive(Debug, Clone, Default)]
pub struct QueryContext {
    pub is_home: bool,
    pub is_front_page: bool,
    pub is_single: bool,
    pub is_page: bool,
    pub is_archive: bool,
    pub is_category: bool,
    pub is_tag: bool,
    pub is_tax: bool,
    pub is_author: bool,
    pub is_date: bool,
    pub is_search: bool,
    pub is_404: bool,
    pub is_attachment: bool,
    pub post_type: Option<String>,
    pub post_id: Option<i64>,
    pub post_slug: Option<String>,
    pub taxonomy: Option<String>,
    pub term_slug: Option<String>,
    pub term_id: Option<i64>,
    pub author_id: Option<i64>,
    pub author_slug: Option<String>,
    pub page_template: Option<String>,
}

impl TemplateHierarchy {
    pub fn new() -> Self {
        Self {
            overrides: HashMap::new(),
            available: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Configure from manifest
    pub fn from_config(config: &TemplateSection) -> Self {
        let mut hierarchy = Self::new();
        hierarchy.overrides = config.hierarchy.clone();
        hierarchy
    }

    /// Register available template
    pub fn register_template(&self, info: TemplateInfo) {
        self.available.write().insert(info.name.clone(), info);
    }

    /// Scan directory for templates
    pub fn scan_directory(&self, dir: &Path, extension: &str) -> Result<Vec<TemplateInfo>, TemplateError> {
        let mut templates = Vec::new();

        if !dir.exists() {
            return Ok(templates);
        }

        for entry in walkdir::WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == extension {
                        let name = path
                            .strip_prefix(dir)
                            .unwrap_or(path)
                            .with_extension("")
                            .to_string_lossy()
                            .replace(std::path::MAIN_SEPARATOR, "/");

                        let template_type = self.detect_template_type(&name);
                        let info = TemplateInfo {
                            name: name.clone(),
                            path: path.to_path_buf(),
                            template_type,
                            post_types: Vec::new(),
                        };

                        templates.push(info.clone());
                        self.available.write().insert(name, info);
                    }
                }
            }
        }

        info!("Scanned {} templates from {:?}", templates.len(), dir);
        Ok(templates)
    }

    /// Detect template type from name
    fn detect_template_type(&self, name: &str) -> TemplateType {
        match name {
            "index" => TemplateType::Index,
            "home" => TemplateType::Home,
            "front-page" | "frontpage" => TemplateType::FrontPage,
            n if n.starts_with("single") => TemplateType::Single,
            n if n.starts_with("page") => TemplateType::Page,
            n if n.starts_with("archive") => TemplateType::Archive,
            n if n.starts_with("category") => TemplateType::Category,
            n if n.starts_with("tag") => TemplateType::Tag,
            n if n.starts_with("taxonomy") => TemplateType::Taxonomy,
            n if n.starts_with("author") => TemplateType::Author,
            n if n.starts_with("date") => TemplateType::Date,
            "search" => TemplateType::Search,
            "404" => TemplateType::NotFound,
            n if n.starts_with("attachment") => TemplateType::Attachment,
            "singular" => TemplateType::Singular,
            _ => TemplateType::Custom,
        }
    }

    /// Resolve template hierarchy for query
    pub fn resolve(&self, query: &QueryContext) -> Vec<String> {
        let mut hierarchy = Vec::new();

        // Check for custom page template first
        if let Some(ref template) = query.page_template {
            hierarchy.push(template.clone());
        }

        // 404 page
        if query.is_404 {
            hierarchy.push("404".to_string());
        }
        // Search results
        else if query.is_search {
            hierarchy.push("search".to_string());
        }
        // Front page
        else if query.is_front_page {
            if query.is_home {
                hierarchy.push("front-page".to_string());
                hierarchy.push("home".to_string());
            } else {
                hierarchy.push("front-page".to_string());
            }
        }
        // Home (blog posts page)
        else if query.is_home {
            hierarchy.push("home".to_string());
        }
        // Attachment
        else if query.is_attachment {
            // attachment-{mime-type}.html
            // attachment.html
            hierarchy.push("attachment".to_string());
        }
        // Single post
        else if query.is_single {
            if let Some(ref post_type) = query.post_type {
                if let Some(ref slug) = query.post_slug {
                    hierarchy.push(format!("single-{}-{}", post_type, slug));
                }
                hierarchy.push(format!("single-{}", post_type));
            }
            hierarchy.push("single".to_string());
            hierarchy.push("singular".to_string());
        }
        // Page
        else if query.is_page {
            if let Some(ref slug) = query.post_slug {
                hierarchy.push(format!("page-{}", slug));
            }
            if let Some(id) = query.post_id {
                hierarchy.push(format!("page-{}", id));
            }
            hierarchy.push("page".to_string());
            hierarchy.push("singular".to_string());
        }
        // Category archive
        else if query.is_category {
            if let Some(ref slug) = query.term_slug {
                hierarchy.push(format!("category-{}", slug));
            }
            if let Some(id) = query.term_id {
                hierarchy.push(format!("category-{}", id));
            }
            hierarchy.push("category".to_string());
            hierarchy.push("archive".to_string());
        }
        // Tag archive
        else if query.is_tag {
            if let Some(ref slug) = query.term_slug {
                hierarchy.push(format!("tag-{}", slug));
            }
            if let Some(id) = query.term_id {
                hierarchy.push(format!("tag-{}", id));
            }
            hierarchy.push("tag".to_string());
            hierarchy.push("archive".to_string());
        }
        // Custom taxonomy archive
        else if query.is_tax {
            if let Some(ref taxonomy) = query.taxonomy {
                if let Some(ref term) = query.term_slug {
                    hierarchy.push(format!("taxonomy-{}-{}", taxonomy, term));
                }
                hierarchy.push(format!("taxonomy-{}", taxonomy));
            }
            hierarchy.push("taxonomy".to_string());
            hierarchy.push("archive".to_string());
        }
        // Author archive
        else if query.is_author {
            if let Some(ref slug) = query.author_slug {
                hierarchy.push(format!("author-{}", slug));
            }
            if let Some(id) = query.author_id {
                hierarchy.push(format!("author-{}", id));
            }
            hierarchy.push("author".to_string());
            hierarchy.push("archive".to_string());
        }
        // Date archive
        else if query.is_date {
            hierarchy.push("date".to_string());
            hierarchy.push("archive".to_string());
        }
        // Generic archive
        else if query.is_archive {
            if let Some(ref post_type) = query.post_type {
                hierarchy.push(format!("archive-{}", post_type));
            }
            hierarchy.push("archive".to_string());
        }

        // Always fall back to index
        hierarchy.push("index".to_string());

        // Apply custom overrides
        for template in &hierarchy {
            if let Some(override_list) = self.overrides.get(template) {
                let mut new_hierarchy = override_list.clone();
                new_hierarchy.extend(hierarchy.iter().skip(1).cloned());
                return new_hierarchy;
            }
        }

        hierarchy
    }

    /// Find first available template from hierarchy
    pub fn find_template(&self, query: &QueryContext) -> Option<TemplateInfo> {
        let hierarchy = self.resolve(query);
        let available = self.available.read();

        for template_name in hierarchy {
            if let Some(info) = available.get(&template_name) {
                debug!("Resolved template: {}", template_name);
                return Some(info.clone());
            }
        }

        None
    }

    /// Get all available templates
    pub fn get_available(&self) -> Vec<TemplateInfo> {
        self.available.read().values().cloned().collect()
    }

    /// Check if template exists
    pub fn template_exists(&self, name: &str) -> bool {
        self.available.read().contains_key(name)
    }
}

impl Default for TemplateHierarchy {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Template Engine (Point 193)
// ============================================================================

/// Template engine using Tera
pub struct TemplateEngine {
    /// Tera instance
    tera: Arc<RwLock<Tera>>,
    /// Template hierarchy
    hierarchy: Arc<TemplateHierarchy>,
    /// Theme directory
    theme_dir: PathBuf,
    /// Template extension
    extension: String,
    /// Global context
    global_context: Arc<RwLock<Context>>,
    /// Template cache
    cache: Arc<RwLock<HashMap<String, String>>>,
    /// Enable caching
    cache_enabled: bool,
}

impl TemplateEngine {
    /// Create new template engine
    pub fn new(theme_dir: PathBuf, extension: &str) -> Result<Self, TemplateError> {
        let template_glob = format!("{}/**/*.{}", theme_dir.display(), extension);
        let tera = Tera::new(&template_glob)
            .map_err(|e| TemplateError::InitError(e.to_string()))?;

        Ok(Self {
            tera: Arc::new(RwLock::new(tera)),
            hierarchy: Arc::new(TemplateHierarchy::new()),
            theme_dir,
            extension: extension.to_string(),
            global_context: Arc::new(RwLock::new(Context::new())),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_enabled: true,
        })
    }

    /// Create with custom hierarchy
    pub fn with_hierarchy(mut self, hierarchy: TemplateHierarchy) -> Self {
        self.hierarchy = Arc::new(hierarchy);
        self
    }

    /// Set caching enabled
    pub fn with_caching(mut self, enabled: bool) -> Self {
        self.cache_enabled = enabled;
        self
    }

    /// Initialize templates
    pub fn init(&self) -> Result<(), TemplateError> {
        let templates_dir = self.theme_dir.join("templates");
        self.hierarchy.scan_directory(&templates_dir, &self.extension)?;

        let parts_dir = self.theme_dir.join("parts");
        if parts_dir.exists() {
            self.hierarchy.scan_directory(&parts_dir, &self.extension)?;
        }

        // Register custom filters and functions
        self.register_filters()?;
        self.register_functions()?;

        Ok(())
    }

    /// Register custom filters
    fn register_filters(&self) -> Result<(), TemplateError> {
        let mut tera = self.tera.write();

        // Truncate filter
        tera.register_filter("truncate_words", |value: &tera::Value, args: &HashMap<String, tera::Value>| {
            let s = tera::try_get_value!("truncate_words", "value", String, value);
            let length = match args.get("length") {
                Some(val) => tera::try_get_value!("truncate_words", "length", usize, val),
                None => 20,
            };
            let words: Vec<&str> = s.split_whitespace().take(length).collect();
            Ok(tera::Value::String(words.join(" ")))
        });

        // Excerpt filter
        tera.register_filter("excerpt", |value: &tera::Value, args: &HashMap<String, tera::Value>| {
            let s = tera::try_get_value!("excerpt", "value", String, value);
            let length = match args.get("length") {
                Some(val) => tera::try_get_value!("excerpt", "length", usize, val),
                None => 55,
            };
            // Strip HTML tags
            let text = regex::Regex::new(r"<[^>]*>")
                .unwrap()
                .replace_all(&s, "")
                .to_string();
            let words: Vec<&str> = text.split_whitespace().take(length).collect();
            let mut result = words.join(" ");
            if text.split_whitespace().count() > length {
                result.push_str("...");
            }
            Ok(tera::Value::String(result))
        });

        // Date format filter
        tera.register_filter("wp_date", |value: &tera::Value, args: &HashMap<String, tera::Value>| {
            let date_str = tera::try_get_value!("wp_date", "value", String, value);
            let _format = match args.get("format") {
                Some(val) => tera::try_get_value!("wp_date", "format", String, val),
                None => "%B %d, %Y".to_string(),
            };
            // In real implementation, would parse and reformat date
            Ok(tera::Value::String(date_str))
        });

        // Sanitize HTML
        tera.register_filter("sanitize_html", |value: &tera::Value, _args: &HashMap<String, tera::Value>| {
            let s = tera::try_get_value!("sanitize_html", "value", String, value);
            let escaped = s
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('"', "&quot;")
                .replace('\'', "&#x27;");
            Ok(tera::Value::String(escaped))
        });

        Ok(())
    }

    /// Register custom functions
    fn register_functions(&self) -> Result<(), TemplateError> {
        let mut tera = self.tera.write();

        // Get template part
        tera.register_function("get_template_part", |args: &HashMap<String, tera::Value>| {
            let slug = args.get("slug")
                .and_then(|v| v.as_str())
                .ok_or_else(|| tera::Error::msg("Missing 'slug' argument"))?;
            let name = args.get("name").and_then(|v| v.as_str());

            let part_name = if let Some(n) = name {
                format!("parts/{}-{}", slug, n)
            } else {
                format!("parts/{}", slug)
            };

            // Return marker for later processing
            Ok(tera::Value::String(format!("{{{{ include \"{}\" }}}}", part_name)))
        });

        // Check if is_single, etc
        tera.register_function("is_single", |_args: &HashMap<String, tera::Value>| {
            // Would check context in real implementation
            Ok(tera::Value::Bool(false))
        });

        tera.register_function("is_page", |_args: &HashMap<String, tera::Value>| {
            Ok(tera::Value::Bool(false))
        });

        tera.register_function("is_home", |_args: &HashMap<String, tera::Value>| {
            Ok(tera::Value::Bool(false))
        });

        tera.register_function("is_front_page", |_args: &HashMap<String, tera::Value>| {
            Ok(tera::Value::Bool(false))
        });

        // Body class
        tera.register_function("body_class", |args: &HashMap<String, tera::Value>| {
            let extra = args.get("class").and_then(|v| v.as_str()).unwrap_or("");
            let classes = format!("rustpress-body {}", extra);
            Ok(tera::Value::String(classes.trim().to_string()))
        });

        // Post class
        tera.register_function("post_class", |args: &HashMap<String, tera::Value>| {
            let extra = args.get("class").and_then(|v| v.as_str()).unwrap_or("");
            let classes = format!("post {}", extra);
            Ok(tera::Value::String(classes.trim().to_string()))
        });

        Ok(())
    }

    /// Set global context value
    pub fn set_global(&self, key: &str, value: impl Serialize) {
        let mut context = self.global_context.write();
        context.insert(key, &value);
    }

    /// Render template by name
    pub fn render(&self, template_name: &str, context: &Context) -> Result<String, TemplateError> {
        // Check cache
        let cache_key = format!("{}:{:?}", template_name, context);
        if self.cache_enabled {
            if let Some(cached) = self.cache.read().get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        // Merge with global context
        let mut merged = self.global_context.read().clone();
        for (key, value) in context.clone().into_json().as_object().unwrap() {
            merged.insert(key, value);
        }

        // Render
        let tera = self.tera.read();
        // Templates from hierarchy are stored as "home", "single", etc. but Tera registers them
        // with their relative path from theme_dir, so we need "templates/home.html", etc.
        let template_file = format!("templates/{}.{}", template_name, self.extension);
        let result = tera.render(&template_file, &merged)
            .map_err(|e| {
                error!("Tera render error for template '{}': {:?}", template_file, e);
                TemplateError::RenderError(format!("{:?}", e))
            })?;

        // Cache result
        if self.cache_enabled {
            self.cache.write().insert(cache_key, result.clone());
        }

        Ok(result)
    }

    /// Render template for query
    pub fn render_for_query(&self, query: &QueryContext, context: &Context) -> Result<String, TemplateError> {
        let template = self.hierarchy.find_template(query)
            .ok_or_else(|| TemplateError::NotFound("No matching template".to_string()))?;

        self.render(&template.name, context)
    }

    /// Render string template
    pub fn render_string(&self, template: &str, context: &Context) -> Result<String, TemplateError> {
        let mut merged = self.global_context.read().clone();
        for (key, value) in context.clone().into_json().as_object().unwrap() {
            merged.insert(key, value);
        }

        let mut tera = self.tera.write();
        tera.render_str(template, &merged)
            .map_err(|e| TemplateError::RenderError(e.to_string()))
    }

    /// Reload templates
    pub fn reload(&self) -> Result<(), TemplateError> {
        let mut tera = self.tera.write();
        tera.full_reload()
            .map_err(|e| TemplateError::InitError(e.to_string()))?;

        self.cache.write().clear();
        info!("Templates reloaded");
        Ok(())
    }

    /// Clear cache
    pub fn clear_cache(&self) {
        self.cache.write().clear();
    }

    /// Get hierarchy
    pub fn hierarchy(&self) -> &TemplateHierarchy {
        &self.hierarchy
    }
}

// ============================================================================
// Template Parts (Point 194)
// ============================================================================

/// Template part manager
pub struct TemplatePartManager {
    /// Parts directory
    parts_dir: PathBuf,
    /// Registered parts
    parts: Arc<RwLock<HashMap<String, TemplatePart>>>,
    /// Part cache
    cache: Arc<RwLock<HashMap<String, String>>>,
}

/// Template part
#[derive(Debug, Clone)]
pub struct TemplatePart {
    pub slug: String,
    pub name: Option<String>,
    pub path: PathBuf,
    pub area: TemplatePartArea,
}

/// Template part area
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemplatePartArea {
    #[default]
    Uncategorized,
    Header,
    Footer,
    Sidebar,
    Content,
}

impl TemplatePartManager {
    pub fn new(parts_dir: PathBuf) -> Self {
        Self {
            parts_dir,
            parts: Arc::new(RwLock::new(HashMap::new())),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Scan and register parts
    pub fn scan(&self, extension: &str) -> Result<(), TemplateError> {
        if !self.parts_dir.exists() {
            return Ok(());
        }

        for entry in walkdir::WalkDir::new(&self.parts_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == extension {
                        let relative = path
                            .strip_prefix(&self.parts_dir)
                            .unwrap_or(path)
                            .with_extension("");

                        let slug = relative.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/");

                        // Detect area from path
                        let area = if slug.starts_with("header") {
                            TemplatePartArea::Header
                        } else if slug.starts_with("footer") {
                            TemplatePartArea::Footer
                        } else if slug.starts_with("sidebar") {
                            TemplatePartArea::Sidebar
                        } else if slug.starts_with("content") {
                            TemplatePartArea::Content
                        } else {
                            TemplatePartArea::Uncategorized
                        };

                        let part = TemplatePart {
                            slug: slug.clone(),
                            name: None,
                            path: path.to_path_buf(),
                            area,
                        };

                        self.parts.write().insert(slug, part);
                    }
                }
            }
        }

        Ok(())
    }

    /// Get template part
    pub fn get(&self, slug: &str, name: Option<&str>) -> Option<TemplatePart> {
        let parts = self.parts.read();

        // Try specific name first
        if let Some(n) = name {
            let key = format!("{}-{}", slug, n);
            if let Some(part) = parts.get(&key) {
                return Some(part.clone());
            }
        }

        // Fall back to base slug
        parts.get(slug).cloned()
    }

    /// Get parts by area
    pub fn get_by_area(&self, area: TemplatePartArea) -> Vec<TemplatePart> {
        self.parts
            .read()
            .values()
            .filter(|p| p.area == area)
            .cloned()
            .collect()
    }

    /// Register custom part
    pub fn register(&self, part: TemplatePart) {
        self.parts.write().insert(part.slug.clone(), part);
    }

    /// Get all parts
    pub fn all(&self) -> Vec<TemplatePart> {
        self.parts.read().values().cloned().collect()
    }

    /// Clear cache
    pub fn clear_cache(&self) {
        self.cache.write().clear();
    }
}

/// Template error
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("Initialization error: {0}")]
    InitError(String),

    #[error("Template not found: {0}")]
    NotFound(String),

    #[error("Render error: {0}")]
    RenderError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hierarchy_single_post() {
        let hierarchy = TemplateHierarchy::new();
        let query = QueryContext {
            is_single: true,
            post_type: Some("post".to_string()),
            post_slug: Some("hello-world".to_string()),
            ..Default::default()
        };

        let result = hierarchy.resolve(&query);
        assert!(result.contains(&"single-post-hello-world".to_string()));
        assert!(result.contains(&"single-post".to_string()));
        assert!(result.contains(&"single".to_string()));
        assert!(result.contains(&"singular".to_string()));
        assert!(result.last() == Some(&"index".to_string()));
    }

    #[test]
    fn test_hierarchy_category() {
        let hierarchy = TemplateHierarchy::new();
        let query = QueryContext {
            is_category: true,
            is_archive: true,
            term_slug: Some("news".to_string()),
            term_id: Some(5),
            ..Default::default()
        };

        let result = hierarchy.resolve(&query);
        assert!(result.contains(&"category-news".to_string()));
        assert!(result.contains(&"category-5".to_string()));
        assert!(result.contains(&"category".to_string()));
        assert!(result.contains(&"archive".to_string()));
    }

    #[test]
    fn test_hierarchy_front_page() {
        let hierarchy = TemplateHierarchy::new();
        let query = QueryContext {
            is_front_page: true,
            is_home: true,
            ..Default::default()
        };

        let result = hierarchy.resolve(&query);
        assert_eq!(result[0], "front-page");
        assert_eq!(result[1], "home");
    }

    #[test]
    fn test_template_part_area_detection() {
        let manager = TemplatePartManager::new(PathBuf::from("/tmp/parts"));

        // Would need actual files to test fully
        assert_eq!(TemplatePartArea::default(), TemplatePartArea::Uncategorized);
    }
}
