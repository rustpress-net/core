//! Child Theme Support
//!
//! Inheritance and override system for child themes.

use crate::manifest::ThemeManifest;
use crate::templates::TemplateEngine;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;
use thiserror::Error;
use tokio::fs;

/// Child theme errors
#[derive(Debug, Error)]
pub enum ChildThemeError {
    #[error("Parent theme not found: {0}")]
    ParentNotFound(String),

    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Manifest error: {0}")]
    Manifest(String),

    #[error("Incompatible parent version: required {required}, found {found}")]
    IncompatibleVersion { required: String, found: String },
}

/// Theme inheritance resolver
pub struct ThemeInheritance {
    /// Theme directory base path
    themes_dir: PathBuf,
    /// Loaded themes
    themes: Arc<RwLock<HashMap<String, LoadedTheme>>>,
    /// Parent-child relationships
    hierarchy: Arc<RwLock<HashMap<String, String>>>,
}

/// Loaded theme information
#[derive(Debug, Clone)]
pub struct LoadedTheme {
    pub id: String,
    pub manifest: ThemeManifest,
    pub path: PathBuf,
    pub parent_id: Option<String>,
    pub is_active: bool,
}

impl ThemeInheritance {
    pub fn new(themes_dir: PathBuf) -> Self {
        Self {
            themes_dir,
            themes: Arc::new(RwLock::new(HashMap::new())),
            hierarchy: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load a theme and its parent chain
    pub fn load_theme<'a>(&'a self, theme_id: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<LoadedTheme, ChildThemeError>> + Send + 'a>> {
        Box::pin(async move {
            // Check if already loaded
            if let Some(theme) = self.themes.read().get(theme_id) {
                return Ok(theme.clone());
            }

            // Load manifest
            let theme_path = self.themes_dir.join(theme_id);
            let manifest_path = theme_path.join("theme.toml");

            if !manifest_path.exists() {
                return Err(ChildThemeError::Manifest(format!(
                    "Manifest not found at {}",
                    manifest_path.display()
                )));
            }

            let manifest_content = fs::read_to_string(&manifest_path).await?;
            let manifest: ThemeManifest = toml::from_str(&manifest_content)
                .map_err(|e| ChildThemeError::Manifest(e.to_string()))?;

            // Check for parent theme
            let parent_id = manifest.parent.as_ref().map(|p| p.id.clone());

            // Detect circular dependencies
            if let Some(ref parent) = parent_id {
                self.check_circular_dependency(theme_id, parent)?;
            }

            // Load parent if exists
            if let Some(ref parent) = parent_id {
                // Load parent first
                self.load_theme(parent).await?;

                // Verify parent version compatibility
                if let Some(parent_info) = &manifest.parent {
                    if let Some(required_version) = &parent_info.min_version {
                        let parent_theme = self.themes.read().get(parent).cloned();
                        if let Some(pt) = parent_theme {
                            let parent_version = &pt.manifest.theme.version;
                            if !is_version_compatible(parent_version, required_version) {
                                return Err(ChildThemeError::IncompatibleVersion {
                                    required: required_version.clone(),
                                    found: parent_version.clone(),
                                });
                            }
                        }
                    }
                }

                // Record hierarchy
                self.hierarchy.write().insert(theme_id.to_string(), parent.clone());
            }

            let theme = LoadedTheme {
                id: theme_id.to_string(),
                manifest,
                path: theme_path,
                parent_id,
                is_active: false,
            };

            self.themes.write().insert(theme_id.to_string(), theme.clone());

            Ok(theme)
        })
    }

    fn check_circular_dependency(&self, theme_id: &str, parent_id: &str) -> Result<(), ChildThemeError> {
        let hierarchy = self.hierarchy.read();
        let mut current = Some(parent_id.to_string());
        let mut chain = vec![theme_id.to_string()];

        while let Some(ref id) = current {
            if id == theme_id {
                chain.push(id.clone());
                return Err(ChildThemeError::CircularDependency(chain.join(" -> ")));
            }
            chain.push(id.clone());
            current = hierarchy.get(id).cloned();
        }

        Ok(())
    }

    /// Get the full inheritance chain for a theme
    pub fn get_inheritance_chain(&self, theme_id: &str) -> Vec<String> {
        let hierarchy = self.hierarchy.read();
        let mut chain = vec![theme_id.to_string()];
        let mut current = hierarchy.get(theme_id);

        while let Some(parent_id) = current {
            chain.push(parent_id.clone());
            current = hierarchy.get(parent_id);
        }

        chain
    }

    /// Resolve a template, checking child theme first then parent chain
    pub async fn resolve_template(&self, theme_id: &str, template_name: &str) -> Option<PathBuf> {
        let chain = self.get_inheritance_chain(theme_id);
        let themes = self.themes.read();

        for id in chain {
            if let Some(theme) = themes.get(&id) {
                let template_path = theme.path.join("templates").join(template_name);
                if template_path.exists() {
                    return Some(template_path);
                }
            }
        }

        None
    }

    /// Resolve a template part
    pub async fn resolve_template_part(&self, theme_id: &str, part_name: &str) -> Option<PathBuf> {
        let chain = self.get_inheritance_chain(theme_id);
        let themes = self.themes.read();

        for id in chain {
            if let Some(theme) = themes.get(&id) {
                // Check parts directory
                let part_path = theme.path.join("parts").join(format!("{}.html", part_name));
                if part_path.exists() {
                    return Some(part_path);
                }

                // Check template-parts directory (WordPress compat)
                let part_path = theme.path.join("template-parts").join(format!("{}.html", part_name));
                if part_path.exists() {
                    return Some(part_path);
                }
            }
        }

        None
    }

    /// Resolve a pattern
    pub async fn resolve_pattern(&self, theme_id: &str, pattern_name: &str) -> Option<PathBuf> {
        let chain = self.get_inheritance_chain(theme_id);
        let themes = self.themes.read();

        for id in chain {
            if let Some(theme) = themes.get(&id) {
                let pattern_path = theme.path.join("patterns").join(format!("{}.html", pattern_name));
                if pattern_path.exists() {
                    return Some(pattern_path);
                }
            }
        }

        None
    }

    /// Resolve an asset (CSS, JS, images)
    pub fn resolve_asset(&self, theme_id: &str, asset_path: &str) -> Option<PathBuf> {
        let chain = self.get_inheritance_chain(theme_id);
        let themes = self.themes.read();

        for id in chain {
            if let Some(theme) = themes.get(&id) {
                let full_path = theme.path.join(asset_path);
                if full_path.exists() {
                    return Some(full_path);
                }
            }
        }

        None
    }

    /// Merge theme settings (child overrides parent)
    pub fn merge_settings(&self, theme_id: &str) -> serde_json::Value {
        let chain = self.get_inheritance_chain(theme_id);
        let themes = self.themes.read();

        // Start with empty object
        let mut merged = serde_json::json!({});

        // Merge from root parent to child (child overrides)
        for id in chain.into_iter().rev() {
            if let Some(theme) = themes.get(&id) {
                // In a real implementation, merge manifest.settings
                // For now, just mark which theme contributed
                merged[&id] = serde_json::json!({"loaded": true});
            }
        }

        merged
    }

    /// Get merged color palette
    pub fn merge_colors(&self, theme_id: &str) -> Vec<crate::manifest::ColorDefinition> {
        let chain = self.get_inheritance_chain(theme_id);
        let themes = self.themes.read();
        let mut colors = Vec::new();
        let mut seen_slugs = std::collections::HashSet::new();

        // Child colors first (override parent)
        for id in chain {
            if let Some(theme) = themes.get(&id) {
                for color in &theme.manifest.colors.palette {
                    if !seen_slugs.contains(&color.slug) {
                        seen_slugs.insert(color.slug.clone());
                        colors.push(color.clone());
                    }
                }
            }
        }

        colors
    }

    /// Get merged typography settings
    pub fn merge_typography(&self, theme_id: &str) -> crate::manifest::TypographySection {
        let chain = self.get_inheritance_chain(theme_id);
        let themes = self.themes.read();

        // Start with defaults
        let mut merged = crate::manifest::TypographySection::default();

        // Merge from parent to child
        for id in chain.into_iter().rev() {
            if let Some(theme) = themes.get(&id) {
                // Child font families override/extend parent
                for family in &theme.manifest.typography.font_families {
                    if !merged.font_families.iter().any(|f| f.slug == family.slug) {
                        merged.font_families.push(family.clone());
                    }
                }

                // Child font sizes override/extend parent
                for size in &theme.manifest.typography.font_sizes {
                    if let Some(existing) = merged.font_sizes.iter_mut().find(|s| s.slug == size.slug) {
                        *existing = size.clone();
                    } else {
                        merged.font_sizes.push(size.clone());
                    }
                }

                merged.fluid = theme.manifest.typography.fluid.clone();
            }
        }

        merged
    }

    /// Unload a theme
    pub fn unload(&self, theme_id: &str) {
        self.themes.write().remove(theme_id);
        self.hierarchy.write().remove(theme_id);
    }
}

/// Check if a version is compatible with a requirement
fn is_version_compatible(version: &str, requirement: &str) -> bool {
    // Simple version checking - in production use semver crate
    let req = requirement.trim_start_matches('^').trim_start_matches('~');

    let version_parts: Vec<u32> = version
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    let req_parts: Vec<u32> = req
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    if requirement.starts_with('^') {
        // Caret: compatible with same major version
        version_parts.get(0) == req_parts.get(0)
    } else if requirement.starts_with('~') {
        // Tilde: compatible with same minor version
        version_parts.get(0) == req_parts.get(0) && version_parts.get(1) == req_parts.get(1)
    } else {
        // Exact match
        version == requirement
    }
}

/// Child theme builder for creating child themes
pub struct ChildThemeBuilder {
    parent_id: String,
    child_id: String,
    name: String,
    description: Option<String>,
    author: Option<String>,
}

impl ChildThemeBuilder {
    pub fn new(parent_id: &str, child_id: &str, name: &str) -> Self {
        Self {
            parent_id: parent_id.to_string(),
            child_id: child_id.to_string(),
            name: name.to_string(),
            description: None,
            author: None,
        }
    }

    pub fn description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    pub fn author(mut self, author: &str) -> Self {
        self.author = Some(author.to_string());
        self
    }

    /// Create the child theme directory structure
    pub async fn create(&self, themes_dir: &Path) -> Result<PathBuf, ChildThemeError> {
        let child_path = themes_dir.join(&self.child_id);

        // Create directories
        fs::create_dir_all(&child_path).await?;
        fs::create_dir_all(child_path.join("templates")).await?;
        fs::create_dir_all(child_path.join("parts")).await?;
        fs::create_dir_all(child_path.join("patterns")).await?;
        fs::create_dir_all(child_path.join("assets")).await?;
        fs::create_dir_all(child_path.join("assets/css")).await?;
        fs::create_dir_all(child_path.join("assets/js")).await?;

        // Create manifest
        let manifest = format!(
            r#"[theme]
id = "{}"
name = "{}"
version = "1.0.0"
description = "{}"
author = "{}"

[parent]
id = "{}"
min_version = "1.0.0"

[supports]
block_editor = true
custom_logo = true
post_thumbnails = true

[colors]
palette = []
gradients = []

[typography]
font_families = []
font_sizes = []

[layout]
content_width = "inherit"
wide_width = "inherit"

[settings]
sections = []
"#,
            self.child_id,
            self.name,
            self.description.as_deref().unwrap_or("A child theme"),
            self.author.as_deref().unwrap_or("Theme Author"),
            self.parent_id,
        );

        fs::write(child_path.join("theme.toml"), manifest).await?;

        // Create empty style.css for overrides
        let style_css = format!(
            r#"/*
Theme Name: {}
Template: {}
Description: {}
*/

/* Add your custom styles here */
"#,
            self.name,
            self.parent_id,
            self.description.as_deref().unwrap_or("A child theme"),
        );

        fs::write(child_path.join("assets/css/style.css"), style_css).await?;

        // Create functions file (for hooks)
        let functions = r#"// Child theme functions
// Add your customizations here
"#;

        fs::write(child_path.join("functions.rs"), functions).await?;

        Ok(child_path)
    }
}

/// Theme override tracking
#[derive(Debug, Clone)]
pub struct ThemeOverride {
    pub file_path: PathBuf,
    pub parent_path: Option<PathBuf>,
    pub override_type: OverrideType,
}

/// Types of overrides
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverrideType {
    Template,
    TemplatePart,
    Pattern,
    Asset,
    Function,
    Style,
}

/// Analyze overrides in a child theme
pub async fn analyze_overrides(
    inheritance: &ThemeInheritance,
    theme_id: &str,
) -> Result<Vec<ThemeOverride>, ChildThemeError> {
    let themes = inheritance.themes.read();
    let theme = themes.get(theme_id).ok_or_else(|| {
        ChildThemeError::Manifest(format!("Theme not found: {}", theme_id))
    })?;

    let parent_id = theme.parent_id.as_ref().ok_or_else(|| {
        ChildThemeError::Manifest("Not a child theme".to_string())
    })?;

    let parent = themes.get(parent_id).ok_or_else(|| {
        ChildThemeError::ParentNotFound(parent_id.clone())
    })?;

    let mut overrides = Vec::new();

    // Check template overrides
    let templates_dir = theme.path.join("templates");
    if templates_dir.exists() {
        for entry in walkdir::WalkDir::new(&templates_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let relative = entry.path().strip_prefix(&templates_dir).unwrap();
            let parent_path = parent.path.join("templates").join(relative);

            overrides.push(ThemeOverride {
                file_path: entry.path().to_path_buf(),
                parent_path: if parent_path.exists() { Some(parent_path) } else { None },
                override_type: OverrideType::Template,
            });
        }
    }

    // Check parts overrides
    let parts_dir = theme.path.join("parts");
    if parts_dir.exists() {
        for entry in walkdir::WalkDir::new(&parts_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let relative = entry.path().strip_prefix(&parts_dir).unwrap();
            let parent_path = parent.path.join("parts").join(relative);

            overrides.push(ThemeOverride {
                file_path: entry.path().to_path_buf(),
                parent_path: if parent_path.exists() { Some(parent_path) } else { None },
                override_type: OverrideType::TemplatePart,
            });
        }
    }

    Ok(overrides)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_compatibility() {
        assert!(is_version_compatible("1.2.3", "^1.0.0"));
        assert!(is_version_compatible("1.5.0", "^1.0.0"));
        assert!(!is_version_compatible("2.0.0", "^1.0.0"));

        assert!(is_version_compatible("1.2.3", "~1.2.0"));
        assert!(!is_version_compatible("1.3.0", "~1.2.0"));

        assert!(is_version_compatible("1.2.3", "1.2.3"));
        assert!(!is_version_compatible("1.2.4", "1.2.3"));
    }

    #[test]
    fn test_inheritance_chain() {
        let inheritance = ThemeInheritance::new(PathBuf::from("/themes"));

        // Manually add hierarchy for testing
        inheritance.hierarchy.write().insert("child".to_string(), "parent".to_string());
        inheritance.hierarchy.write().insert("grandchild".to_string(), "child".to_string());

        let chain = inheritance.get_inheritance_chain("grandchild");
        assert_eq!(chain, vec!["grandchild", "child", "parent"]);
    }
}
