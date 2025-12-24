//! Theme Variations (213), Dark Mode (214), Template Locking (212)
//!
//! Style variations, dark mode support, and template locking.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;
use thiserror::Error;
use tokio::fs;

/// Variation errors
#[derive(Debug, Error)]
pub enum VariationError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Variation not found: {0}")]
    NotFound(String),

    #[error("Parse error: {0}")]
    Parse(String),
}

/// Style variation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleVariation {
    /// Variation slug
    pub slug: String,
    /// Display title
    pub title: String,
    /// Variation settings (partial theme.json)
    pub settings: serde_json::Value,
    /// Variation styles (partial theme.json)
    pub styles: serde_json::Value,
    /// Preview image
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
}

/// Style variation manager
pub struct VariationManager {
    theme_path: PathBuf,
    variations: Arc<RwLock<HashMap<String, StyleVariation>>>,
    active_variation: Arc<RwLock<Option<String>>>,
}

impl VariationManager {
    pub fn new(theme_path: PathBuf) -> Self {
        Self {
            theme_path,
            variations: Arc::new(RwLock::new(HashMap::new())),
            active_variation: Arc::new(RwLock::new(None)),
        }
    }

    /// Load variations from theme
    pub async fn load(&self) -> Result<(), VariationError> {
        let styles_dir = self.theme_path.join("styles");

        if !styles_dir.exists() {
            return Ok(());
        }

        let mut entries = fs::read_dir(&styles_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.extension().map_or(false, |e| e == "json") {
                let content = fs::read_to_string(&path).await?;
                let variation: StyleVariation = serde_json::from_str(&content)
                    .map_err(|e| VariationError::Parse(e.to_string()))?;

                self.variations.write().insert(variation.slug.clone(), variation);
            }
        }

        Ok(())
    }

    /// Get all variations
    pub fn get_all(&self) -> Vec<StyleVariation> {
        self.variations.read().values().cloned().collect()
    }

    /// Get a specific variation
    pub fn get(&self, slug: &str) -> Option<StyleVariation> {
        self.variations.read().get(slug).cloned()
    }

    /// Set active variation
    pub fn set_active(&self, slug: &str) -> Result<(), VariationError> {
        if !self.variations.read().contains_key(slug) {
            return Err(VariationError::NotFound(slug.to_string()));
        }

        *self.active_variation.write() = Some(slug.to_string());
        Ok(())
    }

    /// Get active variation
    pub fn get_active(&self) -> Option<StyleVariation> {
        let active = self.active_variation.read();
        active.as_ref().and_then(|slug| self.get(slug))
    }

    /// Clear active variation
    pub fn clear_active(&self) {
        *self.active_variation.write() = None;
    }

    /// Register a variation
    pub fn register(&self, variation: StyleVariation) {
        self.variations.write().insert(variation.slug.clone(), variation);
    }

    /// Generate CSS for active variation
    pub fn generate_css(&self) -> Option<String> {
        let variation = self.get_active()?;

        // Generate CSS from variation styles
        let mut css = String::new();

        if let Some(obj) = variation.styles.as_object() {
            if let Some(color) = obj.get("color") {
                if let Some(bg) = color.get("background").and_then(|v| v.as_str()) {
                    css.push_str(&format!("body {{ background-color: {}; }}\n", bg));
                }
                if let Some(text) = color.get("text").and_then(|v| v.as_str()) {
                    css.push_str(&format!("body {{ color: {}; }}\n", text));
                }
            }
        }

        Some(css)
    }
}

/// Dark mode support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DarkModeConfig {
    /// Enable dark mode
    pub enabled: bool,
    /// Default mode (light, dark, auto)
    pub default_mode: ColorMode,
    /// Dark mode colors
    pub dark_colors: HashMap<String, String>,
    /// Light mode colors
    pub light_colors: HashMap<String, String>,
    /// CSS custom properties to toggle
    pub toggle_properties: Vec<String>,
    /// Respect system preference
    pub respect_system: bool,
    /// Transition duration
    pub transition_duration: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorMode {
    Light,
    Dark,
    Auto,
}

impl Default for ColorMode {
    fn default() -> Self {
        Self::Auto
    }
}

impl Default for DarkModeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_mode: ColorMode::Auto,
            dark_colors: HashMap::new(),
            light_colors: HashMap::new(),
            toggle_properties: vec![
                "--wp--preset--color--background".to_string(),
                "--wp--preset--color--foreground".to_string(),
            ],
            respect_system: true,
            transition_duration: "0.3s".to_string(),
        }
    }
}

impl DarkModeConfig {
    /// Generate CSS for dark mode
    pub fn generate_css(&self) -> String {
        if !self.enabled {
            return String::new();
        }

        let mut css = String::new();

        // Light mode (default)
        css.push_str(":root {\n");
        for (prop, value) in &self.light_colors {
            css.push_str(&format!("  {}: {};\n", prop, value));
        }
        css.push_str("}\n\n");

        // Dark mode via attribute
        css.push_str("[data-theme=\"dark\"] {\n");
        for (prop, value) in &self.dark_colors {
            css.push_str(&format!("  {}: {};\n", prop, value));
        }
        css.push_str("}\n\n");

        // Dark mode via system preference
        if self.respect_system {
            css.push_str("@media (prefers-color-scheme: dark) {\n");
            css.push_str("  :root:not([data-theme=\"light\"]) {\n");
            for (prop, value) in &self.dark_colors {
                css.push_str(&format!("    {}: {};\n", prop, value));
            }
            css.push_str("  }\n");
            css.push_str("}\n\n");
        }

        // Transition
        css.push_str(&format!(
            ":root {{ color-scheme: light dark; transition: background-color {0}, color {0}; }}\n",
            self.transition_duration
        ));

        css
    }

    /// Generate JavaScript for dark mode toggle
    pub fn generate_js(&self) -> String {
        format!(
            r#"(function() {{
  const prefersDark = window.matchMedia('(prefers-color-scheme: dark)');
  const stored = localStorage.getItem('theme');

  function setTheme(theme) {{
    document.documentElement.setAttribute('data-theme', theme);
    localStorage.setItem('theme', theme);
  }}

  function getPreferredTheme() {{
    if (stored) return stored;
    if ({}) return prefersDark.matches ? 'dark' : 'light';
    return '{}';
  }}

  setTheme(getPreferredTheme());

  prefersDark.addEventListener('change', (e) => {{
    if (!stored) setTheme(e.matches ? 'dark' : 'light');
  }});

  window.toggleTheme = function() {{
    const current = document.documentElement.getAttribute('data-theme');
    setTheme(current === 'dark' ? 'light' : 'dark');
  }};
}})();"#,
            self.respect_system,
            match self.default_mode {
                ColorMode::Light => "light",
                ColorMode::Dark => "dark",
                ColorMode::Auto => "light",
            }
        )
    }
}

/// Template locking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateLock {
    /// Lock type
    pub lock_type: LockType,
    /// Whether to prevent removal
    pub remove: bool,
    /// Whether to prevent movement
    #[serde(rename = "move")]
    pub move_lock: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LockType {
    /// All locks enabled
    All,
    /// Only insertion locked
    Insert,
    /// Content editing locked
    ContentOnly,
}

impl Default for TemplateLock {
    fn default() -> Self {
        Self {
            lock_type: LockType::All,
            remove: true,
            move_lock: true,
        }
    }
}

/// Template locking manager
pub struct TemplateLockManager {
    locks: Arc<RwLock<HashMap<String, TemplateLock>>>,
}

impl TemplateLockManager {
    pub fn new() -> Self {
        Self {
            locks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Lock a template
    pub fn lock(&self, template_slug: &str, lock: TemplateLock) {
        self.locks.write().insert(template_slug.to_string(), lock);
    }

    /// Unlock a template
    pub fn unlock(&self, template_slug: &str) {
        self.locks.write().remove(template_slug);
    }

    /// Get lock for template
    pub fn get_lock(&self, template_slug: &str) -> Option<TemplateLock> {
        self.locks.read().get(template_slug).cloned()
    }

    /// Check if template is locked
    pub fn is_locked(&self, template_slug: &str) -> bool {
        self.locks.read().contains_key(template_slug)
    }

    /// Check if removal is allowed
    pub fn can_remove(&self, template_slug: &str) -> bool {
        self.locks
            .read()
            .get(template_slug)
            .map_or(true, |lock| !lock.remove)
    }

    /// Check if moving is allowed
    pub fn can_move(&self, template_slug: &str) -> bool {
        self.locks
            .read()
            .get(template_slug)
            .map_or(true, |lock| !lock.move_lock)
    }
}

impl Default for TemplateLockManager {
    fn default() -> Self {
        Self::new()
    }
}

/// CSS custom properties system (211)
pub struct CssCustomProperties {
    properties: Arc<RwLock<HashMap<String, String>>>,
}

impl CssCustomProperties {
    pub fn new() -> Self {
        Self {
            properties: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set a property
    pub fn set(&self, name: &str, value: &str) {
        let name = if name.starts_with("--") {
            name.to_string()
        } else {
            format!("--{}", name)
        };

        self.properties.write().insert(name, value.to_string());
    }

    /// Get a property
    pub fn get(&self, name: &str) -> Option<String> {
        let name = if name.starts_with("--") {
            name.to_string()
        } else {
            format!("--{}", name)
        };

        self.properties.read().get(&name).cloned()
    }

    /// Generate CSS
    pub fn generate_css(&self) -> String {
        let props = self.properties.read();
        if props.is_empty() {
            return String::new();
        }

        let mut css = String::from(":root {\n");
        for (name, value) in props.iter() {
            css.push_str(&format!("  {}: {};\n", name, value));
        }
        css.push_str("}\n");
        css
    }

    /// Set from theme.json colors
    pub fn from_theme_json_colors(&mut self, colors: &[crate::theme_json::ColorPreset]) {
        for color in colors {
            self.set(
                &format!("--wp--preset--color--{}", color.slug),
                &color.color,
            );
        }
    }

    /// Set from theme.json font sizes
    pub fn from_theme_json_font_sizes(&mut self, sizes: &[crate::theme_json::FontSizePreset]) {
        for size in sizes {
            self.set(
                &format!("--wp--preset--font-size--{}", size.slug),
                &size.size,
            );
        }
    }
}

impl Default for CssCustomProperties {
    fn default() -> Self {
        Self::new()
    }
}

/// Print stylesheet (215)
pub fn generate_print_stylesheet() -> String {
    r##"@media print {
  /* Basic print styles */
  *, *::before, *::after {
    background: transparent !important;
    color: #000 !important;
    box-shadow: none !important;
    text-shadow: none !important;
  }

  body {
    font-size: 12pt;
    line-height: 1.5;
  }

  /* Links */
  a, a:visited {
    text-decoration: underline;
  }

  a[href]::after {
    content: " (" attr(href) ")";
    font-size: 0.8em;
  }

  a[href^="#"]::after,
  a[href^="javascript:"]::after {
    content: "";
  }

  /* Images */
  img {
    max-width: 100% !important;
    page-break-inside: avoid;
  }

  /* Headings */
  h1, h2, h3, h4, h5, h6 {
    page-break-after: avoid;
    page-break-inside: avoid;
  }

  /* Paragraphs */
  p, blockquote {
    orphans: 3;
    widows: 3;
  }

  /* Hide non-essential elements */
  nav, .no-print, .site-header, .site-footer,
  .comments, .comment-respond, .sidebar,
  button, input, select, textarea {
    display: none !important;
  }

  /* Page breaks */
  article {
    page-break-before: always;
  }

  /* Tables */
  table {
    border-collapse: collapse;
  }

  table, th, td {
    border: 1px solid #000;
  }

  th, td {
    padding: 0.5rem;
  }

  /* Preformatted text */
  pre, code {
    font-family: monospace;
    font-size: 10pt;
  }

  pre {
    white-space: pre-wrap;
    word-wrap: break-word;
  }
}
"##.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dark_mode_config() {
        let config = DarkModeConfig::default();
        assert!(config.enabled);
        assert!(config.respect_system);
    }

    #[test]
    fn test_dark_mode_css() {
        let mut config = DarkModeConfig::default();
        config.dark_colors.insert("--background".to_string(), "#1a1a1a".to_string());
        config.light_colors.insert("--background".to_string(), "#ffffff".to_string());

        let css = config.generate_css();
        assert!(css.contains("prefers-color-scheme: dark"));
        assert!(css.contains("#1a1a1a"));
    }

    #[test]
    fn test_template_lock() {
        let manager = TemplateLockManager::new();

        manager.lock("header", TemplateLock::default());
        assert!(manager.is_locked("header"));
        assert!(!manager.can_remove("header"));

        manager.unlock("header");
        assert!(!manager.is_locked("header"));
    }

    #[test]
    fn test_css_custom_properties() {
        let mut props = CssCustomProperties::new();
        props.set("--primary-color", "#0073aa");

        assert_eq!(props.get("--primary-color"), Some("#0073aa".to_string()));

        let css = props.generate_css();
        assert!(css.contains("--primary-color: #0073aa"));
    }
}
