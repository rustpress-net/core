//! Full-Site Editing (FSE)
//!
//! Block-based site editing with templates and template parts.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;
use thiserror::Error;
use tokio::fs;

/// FSE errors
#[derive(Debug, Error)]
pub enum FseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Invalid template: {0}")]
    InvalidTemplate(String),
}

/// FSE template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FseTemplate {
    /// Template slug
    pub slug: String,
    /// Template title
    pub title: String,
    /// Template description
    #[serde(default)]
    pub description: Option<String>,
    /// Template content (block markup)
    pub content: String,
    /// Template type
    pub template_type: TemplateType,
    /// Post types this template applies to
    #[serde(default)]
    pub post_types: Vec<String>,
    /// Associated template parts
    #[serde(default)]
    pub template_parts: Vec<String>,
    /// Is this template modified by user
    #[serde(default)]
    pub is_custom: bool,
    /// Source theme ID
    #[serde(default)]
    pub source: Option<String>,
}

/// Template types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TemplateType {
    Index,
    Home,
    FrontPage,
    Singular,
    Single,
    Page,
    Archive,
    Author,
    Category,
    Tag,
    Taxonomy,
    Date,
    Search,
    NotFound,
    Attachment,
    PrivacyPolicy,
    Custom,
}

impl TemplateType {
    pub fn slug(&self) -> &'static str {
        match self {
            Self::Index => "index",
            Self::Home => "home",
            Self::FrontPage => "front-page",
            Self::Singular => "singular",
            Self::Single => "single",
            Self::Page => "page",
            Self::Archive => "archive",
            Self::Author => "author",
            Self::Category => "category",
            Self::Tag => "tag",
            Self::Taxonomy => "taxonomy",
            Self::Date => "date",
            Self::Search => "search",
            Self::NotFound => "404",
            Self::Attachment => "attachment",
            Self::PrivacyPolicy => "privacy-policy",
            Self::Custom => "custom",
        }
    }
}

/// FSE template part
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplatePart {
    /// Part slug
    pub slug: String,
    /// Part title
    pub title: String,
    /// Part area (header, footer, sidebar, general)
    pub area: TemplatePartArea,
    /// Part content (block markup)
    pub content: String,
    /// Is this part modified by user
    #[serde(default)]
    pub is_custom: bool,
    /// Source theme ID
    #[serde(default)]
    pub source: Option<String>,
}

/// Template part areas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemplatePartArea {
    Header,
    Footer,
    Sidebar,
    General,
    Uncategorized,
}

impl Default for TemplatePartArea {
    fn default() -> Self {
        Self::Uncategorized
    }
}

/// FSE manager
pub struct FseManager {
    /// Theme directory
    theme_path: PathBuf,
    /// Templates (from theme and user customizations)
    templates: Arc<RwLock<HashMap<String, FseTemplate>>>,
    /// Template parts
    parts: Arc<RwLock<HashMap<String, TemplatePart>>>,
    /// User templates directory
    custom_templates_dir: PathBuf,
}

impl FseManager {
    pub fn new(theme_path: PathBuf, custom_templates_dir: PathBuf) -> Self {
        Self {
            theme_path,
            templates: Arc::new(RwLock::new(HashMap::new())),
            parts: Arc::new(RwLock::new(HashMap::new())),
            custom_templates_dir,
        }
    }

    /// Load templates from theme
    pub async fn load_templates(&self) -> Result<(), FseError> {
        // Load theme templates
        let templates_dir = self.theme_path.join("templates");
        if templates_dir.exists() {
            self.load_templates_from_dir(&templates_dir, false).await?;
        }

        // Load custom templates (override theme)
        if self.custom_templates_dir.exists() {
            self.load_templates_from_dir(&self.custom_templates_dir, true).await?;
        }

        Ok(())
    }

    async fn load_templates_from_dir(&self, dir: &Path, is_custom: bool) -> Result<(), FseError> {
        let mut entries = fs::read_dir(dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.extension().map_or(false, |e| e == "html") {
                let content = fs::read_to_string(&path).await?;
                let slug = path.file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| FseError::InvalidTemplate("Invalid filename".to_string()))?
                    .to_string();

                let template = self.parse_template(&slug, &content, is_custom)?;
                self.templates.write().insert(slug, template);
            }
        }

        Ok(())
    }

    fn parse_template(&self, slug: &str, content: &str, is_custom: bool) -> Result<FseTemplate, FseError> {
        // Extract metadata from HTML comments if present
        let (title, description, post_types) = self.extract_template_metadata(content);

        let template_type = self.infer_template_type(slug);

        Ok(FseTemplate {
            slug: slug.to_string(),
            title: title.unwrap_or_else(|| self.format_title(slug)),
            description,
            content: content.to_string(),
            template_type,
            post_types,
            template_parts: self.find_template_parts(content),
            is_custom,
            source: if is_custom { None } else { Some("theme".to_string()) },
        })
    }

    fn extract_template_metadata(&self, content: &str) -> (Option<String>, Option<String>, Vec<String>) {
        let mut title = None;
        let mut description = None;
        let mut post_types = Vec::new();

        // Look for metadata comment at start of file
        // Format: <!-- wp:template {"title":"...","description":"...","postTypes":["post"]} -->
        if let Some(start) = content.find("<!-- wp:template ") {
            if let Some(end) = content[start..].find("-->") {
                let comment = &content[start..start + end + 3];
                if let Some(json_start) = comment.find('{') {
                    if let Some(json_end) = comment.rfind('}') {
                        let json_str = &comment[json_start..=json_end];
                        if let Ok(meta) = serde_json::from_str::<serde_json::Value>(json_str) {
                            if let Some(t) = meta.get("title").and_then(|v| v.as_str()) {
                                title = Some(t.to_string());
                            }
                            if let Some(d) = meta.get("description").and_then(|v| v.as_str()) {
                                description = Some(d.to_string());
                            }
                            if let Some(pt) = meta.get("postTypes").and_then(|v| v.as_array()) {
                                post_types = pt.iter()
                                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                    .collect();
                            }
                        }
                    }
                }
            }
        }

        (title, description, post_types)
    }

    fn infer_template_type(&self, slug: &str) -> TemplateType {
        match slug {
            "index" => TemplateType::Index,
            "home" => TemplateType::Home,
            "front-page" => TemplateType::FrontPage,
            "singular" => TemplateType::Singular,
            "single" => TemplateType::Single,
            "page" => TemplateType::Page,
            "archive" => TemplateType::Archive,
            "author" => TemplateType::Author,
            "category" => TemplateType::Category,
            "tag" => TemplateType::Tag,
            "taxonomy" => TemplateType::Taxonomy,
            "date" => TemplateType::Date,
            "search" => TemplateType::Search,
            "404" => TemplateType::NotFound,
            "attachment" => TemplateType::Attachment,
            "privacy-policy" => TemplateType::PrivacyPolicy,
            _ => TemplateType::Custom,
        }
    }

    fn format_title(&self, slug: &str) -> String {
        slug.replace('-', " ")
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn find_template_parts(&self, content: &str) -> Vec<String> {
        let mut parts = Vec::new();

        // Find wp:template-part blocks
        let re = regex::Regex::new(r#"<!-- wp:template-part \{[^}]*"slug":"([^"]+)"[^}]*\}"#)
            .unwrap();

        for cap in re.captures_iter(content) {
            parts.push(cap[1].to_string());
        }

        parts
    }

    /// Load template parts from theme
    pub async fn load_parts(&self) -> Result<(), FseError> {
        // Load theme parts
        let parts_dir = self.theme_path.join("parts");
        if parts_dir.exists() {
            self.load_parts_from_dir(&parts_dir, false).await?;
        }

        // Load custom parts
        let custom_parts_dir = self.custom_templates_dir.join("parts");
        if custom_parts_dir.exists() {
            self.load_parts_from_dir(&custom_parts_dir, true).await?;
        }

        Ok(())
    }

    async fn load_parts_from_dir(&self, dir: &Path, is_custom: bool) -> Result<(), FseError> {
        let mut entries = fs::read_dir(dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.extension().map_or(false, |e| e == "html") {
                let content = fs::read_to_string(&path).await?;
                let slug = path.file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| FseError::InvalidTemplate("Invalid filename".to_string()))?
                    .to_string();

                let part = self.parse_part(&slug, &content, is_custom)?;
                self.parts.write().insert(slug, part);
            }
        }

        Ok(())
    }

    fn parse_part(&self, slug: &str, content: &str, is_custom: bool) -> Result<TemplatePart, FseError> {
        let (title, area) = self.extract_part_metadata(content, slug);

        Ok(TemplatePart {
            slug: slug.to_string(),
            title: title.unwrap_or_else(|| self.format_title(slug)),
            area,
            content: content.to_string(),
            is_custom,
            source: if is_custom { None } else { Some("theme".to_string()) },
        })
    }

    fn extract_part_metadata(&self, content: &str, slug: &str) -> (Option<String>, TemplatePartArea) {
        let mut title = None;
        let mut area = self.infer_part_area(slug);

        // Look for metadata in HTML comment
        if let Some(start) = content.find("<!-- wp:template-part ") {
            if let Some(end) = content[start..].find("-->") {
                let comment = &content[start..start + end + 3];
                if let Some(json_start) = comment.find('{') {
                    if let Some(json_end) = comment.rfind('}') {
                        let json_str = &comment[json_start..=json_end];
                        if let Ok(meta) = serde_json::from_str::<serde_json::Value>(json_str) {
                            if let Some(t) = meta.get("title").and_then(|v| v.as_str()) {
                                title = Some(t.to_string());
                            }
                            if let Some(a) = meta.get("area").and_then(|v| v.as_str()) {
                                area = match a {
                                    "header" => TemplatePartArea::Header,
                                    "footer" => TemplatePartArea::Footer,
                                    "sidebar" => TemplatePartArea::Sidebar,
                                    _ => TemplatePartArea::General,
                                };
                            }
                        }
                    }
                }
            }
        }

        (title, area)
    }

    fn infer_part_area(&self, slug: &str) -> TemplatePartArea {
        if slug.contains("header") {
            TemplatePartArea::Header
        } else if slug.contains("footer") {
            TemplatePartArea::Footer
        } else if slug.contains("sidebar") {
            TemplatePartArea::Sidebar
        } else {
            TemplatePartArea::General
        }
    }

    /// Get a template
    pub fn get_template(&self, slug: &str) -> Option<FseTemplate> {
        self.templates.read().get(slug).cloned()
    }

    /// Get all templates
    pub fn get_all_templates(&self) -> Vec<FseTemplate> {
        self.templates.read().values().cloned().collect()
    }

    /// Get templates by type
    pub fn get_templates_by_type(&self, template_type: TemplateType) -> Vec<FseTemplate> {
        self.templates
            .read()
            .values()
            .filter(|t| t.template_type == template_type)
            .cloned()
            .collect()
    }

    /// Get a template part
    pub fn get_part(&self, slug: &str) -> Option<TemplatePart> {
        self.parts.read().get(slug).cloned()
    }

    /// Get all template parts
    pub fn get_all_parts(&self) -> Vec<TemplatePart> {
        self.parts.read().values().cloned().collect()
    }

    /// Get parts by area
    pub fn get_parts_by_area(&self, area: TemplatePartArea) -> Vec<TemplatePart> {
        self.parts
            .read()
            .values()
            .filter(|p| p.area == area)
            .cloned()
            .collect()
    }

    /// Save a custom template
    pub async fn save_template(&self, template: FseTemplate) -> Result<(), FseError> {
        // Save to custom templates directory
        fs::create_dir_all(&self.custom_templates_dir).await?;

        let path = self.custom_templates_dir.join(format!("{}.html", template.slug));
        fs::write(&path, &template.content).await?;

        // Update cache
        let mut templates = self.templates.write();
        templates.insert(template.slug.clone(), FseTemplate {
            is_custom: true,
            ..template
        });

        Ok(())
    }

    /// Save a custom template part
    pub async fn save_part(&self, part: TemplatePart) -> Result<(), FseError> {
        let parts_dir = self.custom_templates_dir.join("parts");
        fs::create_dir_all(&parts_dir).await?;

        let path = parts_dir.join(format!("{}.html", part.slug));
        fs::write(&path, &part.content).await?;

        // Update cache
        let mut parts = self.parts.write();
        parts.insert(part.slug.clone(), TemplatePart {
            is_custom: true,
            ..part
        });

        Ok(())
    }

    /// Delete a custom template (reverts to theme version)
    pub async fn delete_custom_template(&self, slug: &str) -> Result<(), FseError> {
        let path = self.custom_templates_dir.join(format!("{}.html", slug));

        if path.exists() {
            fs::remove_file(&path).await?;
        }

        // Reload from theme
        let theme_path = self.theme_path.join("templates").join(format!("{}.html", slug));
        if theme_path.exists() {
            let content = fs::read_to_string(&theme_path).await?;
            let template = self.parse_template(slug, &content, false)?;
            self.templates.write().insert(slug.to_string(), template);
        } else {
            self.templates.write().remove(slug);
        }

        Ok(())
    }

    /// Delete a custom template part
    pub async fn delete_custom_part(&self, slug: &str) -> Result<(), FseError> {
        let path = self.custom_templates_dir.join("parts").join(format!("{}.html", slug));

        if path.exists() {
            fs::remove_file(&path).await?;
        }

        // Reload from theme
        let theme_path = self.theme_path.join("parts").join(format!("{}.html", slug));
        if theme_path.exists() {
            let content = fs::read_to_string(&theme_path).await?;
            let part = self.parse_part(slug, &content, false)?;
            self.parts.write().insert(slug.to_string(), part);
        } else {
            self.parts.write().remove(slug);
        }

        Ok(())
    }

    /// Render a template (resolving template parts)
    pub fn render(&self, template_slug: &str) -> Result<String, FseError> {
        let template = self.get_template(template_slug)
            .ok_or_else(|| FseError::TemplateNotFound(template_slug.to_string()))?;

        self.render_with_parts(&template.content)
    }

    fn render_with_parts(&self, content: &str) -> Result<String, FseError> {
        let mut result = content.to_string();

        // Replace template-part blocks with their content
        let re = regex::Regex::new(
            r#"<!-- wp:template-part \{[^}]*"slug":"([^"]+)"[^}]*\} /-->"#
        ).unwrap();

        for cap in re.captures_iter(content) {
            let full_match = cap.get(0).unwrap().as_str();
            let slug = &cap[1];

            if let Some(part) = self.get_part(slug) {
                result = result.replace(full_match, &part.content);
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_template_type_slug() {
        assert_eq!(TemplateType::Index.slug(), "index");
        assert_eq!(TemplateType::FrontPage.slug(), "front-page");
        assert_eq!(TemplateType::NotFound.slug(), "404");
    }

    #[tokio::test]
    async fn test_fse_manager_creation() {
        let dir = tempdir().unwrap();
        let manager = FseManager::new(
            dir.path().to_path_buf(),
            dir.path().join("custom"),
        );

        assert!(manager.get_all_templates().is_empty());
        assert!(manager.get_all_parts().is_empty());
    }

    #[test]
    fn test_format_title() {
        let dir = tempdir().unwrap();
        let manager = FseManager::new(
            dir.path().to_path_buf(),
            dir.path().join("custom"),
        );

        assert_eq!(manager.format_title("front-page"), "Front Page");
        assert_eq!(manager.format_title("single-post"), "Single Post");
    }
}
