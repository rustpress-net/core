//! theme.json Support
//!
//! WordPress-compatible theme.json configuration for block themes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;
use tokio::fs;

/// theme.json errors
#[derive(Debug, Error)]
pub enum ThemeJsonError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("Validation error: {0}")]
    Validation(String),
}

/// theme.json schema version
pub const SCHEMA_VERSION: u32 = 2;

/// theme.json configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeJson {
    /// Schema URL
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    /// Schema version
    pub version: u32,

    /// Theme settings
    #[serde(default)]
    pub settings: ThemeJsonSettings,

    /// Theme styles
    #[serde(default)]
    pub styles: ThemeJsonStyles,

    /// Template part areas
    #[serde(default)]
    pub template_parts: Vec<TemplatePartDefinition>,

    /// Custom templates
    #[serde(default)]
    pub custom_templates: Vec<CustomTemplateDefinition>,

    /// Block patterns
    #[serde(default)]
    pub patterns: Vec<String>,
}

impl Default for ThemeJson {
    fn default() -> Self {
        Self {
            schema: Some("https://schemas.wp.org/trunk/theme.json".to_string()),
            version: SCHEMA_VERSION,
            settings: ThemeJsonSettings::default(),
            styles: ThemeJsonStyles::default(),
            template_parts: Vec::new(),
            custom_templates: Vec::new(),
            patterns: Vec::new(),
        }
    }
}

/// Theme settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeJsonSettings {
    #[serde(default)]
    pub appear_tools: AppearTools,

    #[serde(default)]
    pub border: BorderSettings,

    #[serde(default)]
    pub color: ColorSettings,

    #[serde(default)]
    pub layout: LayoutSettings,

    #[serde(default)]
    pub spacing: SpacingSettings,

    #[serde(default)]
    pub typography: TypographySettings,

    #[serde(default)]
    pub blocks: HashMap<String, BlockSettings>,

    #[serde(default)]
    pub custom: HashMap<String, serde_json::Value>,
}

/// Appearance tools settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppearTools {
    #[serde(default = "default_true")]
    pub background_image: bool,
}

fn default_true() -> bool {
    true
}

/// Border settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BorderSettings {
    #[serde(default = "default_true")]
    pub color: bool,
    #[serde(default = "default_true")]
    pub radius: bool,
    #[serde(default = "default_true")]
    pub style: bool,
    #[serde(default = "default_true")]
    pub width: bool,
}

/// Color settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorSettings {
    #[serde(default = "default_true")]
    pub background: bool,
    #[serde(default = "default_true")]
    pub custom: bool,
    #[serde(default = "default_true")]
    pub custom_duotone: bool,
    #[serde(default = "default_true")]
    pub custom_gradient: bool,
    #[serde(default)]
    pub default_duotone: bool,
    #[serde(default)]
    pub default_gradients: bool,
    #[serde(default)]
    pub default_palette: bool,
    #[serde(default)]
    pub duotone: Vec<DuotonePreset>,
    #[serde(default)]
    pub gradients: Vec<GradientPreset>,
    #[serde(default = "default_true")]
    pub link: bool,
    #[serde(default)]
    pub palette: Vec<ColorPreset>,
    #[serde(default = "default_true")]
    pub text: bool,
}

impl Default for ColorSettings {
    fn default() -> Self {
        Self {
            background: true,
            custom: true,
            custom_duotone: true,
            custom_gradient: true,
            default_duotone: true,
            default_gradients: true,
            default_palette: true,
            duotone: Vec::new(),
            gradients: Vec::new(),
            link: true,
            palette: Vec::new(),
            text: true,
        }
    }
}

/// Color preset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPreset {
    pub slug: String,
    pub name: String,
    pub color: String,
}

/// Gradient preset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientPreset {
    pub slug: String,
    pub name: String,
    pub gradient: String,
}

/// Duotone preset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuotonePreset {
    pub slug: String,
    pub name: String,
    pub colors: Vec<String>,
}

/// Layout settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wide_size: Option<String>,
    #[serde(default = "default_true")]
    pub allow_editing: bool,
    #[serde(default = "default_true")]
    pub allow_custom_content_and_wide_size: bool,
}

impl Default for LayoutSettings {
    fn default() -> Self {
        Self {
            content_size: Some("650px".to_string()),
            wide_size: Some("1200px".to_string()),
            allow_editing: true,
            allow_custom_content_and_wide_size: true,
        }
    }
}

/// Spacing settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpacingSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_gap: Option<serde_json::Value>,
    #[serde(default = "default_true")]
    pub margin: bool,
    #[serde(default = "default_true")]
    pub padding: bool,
    #[serde(default)]
    pub units: Vec<String>,
    #[serde(default)]
    pub spacing_sizes: Vec<SpacingPreset>,
    #[serde(default = "default_true")]
    pub custom_spacing_size: bool,
}

impl Default for SpacingSettings {
    fn default() -> Self {
        Self {
            block_gap: None,
            margin: true,
            padding: true,
            units: vec!["px".to_string(), "em".to_string(), "rem".to_string(), "%".to_string(), "vw".to_string()],
            spacing_sizes: Vec::new(),
            custom_spacing_size: true,
        }
    }
}

/// Spacing preset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacingPreset {
    pub slug: String,
    pub name: String,
    pub size: String,
}

/// Typography settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypographySettings {
    #[serde(default = "default_true")]
    pub custom_font_size: bool,
    #[serde(default = "default_true")]
    pub drop_cap: bool,
    #[serde(default)]
    pub fluid: Option<FluidTypography>,
    #[serde(default)]
    pub font_families: Vec<FontFamilyPreset>,
    #[serde(default)]
    pub font_sizes: Vec<FontSizePreset>,
    #[serde(default = "default_true")]
    pub font_style: bool,
    #[serde(default = "default_true")]
    pub font_weight: bool,
    #[serde(default = "default_true")]
    pub letter_spacing: bool,
    #[serde(default = "default_true")]
    pub line_height: bool,
    #[serde(default = "default_true")]
    pub text_decoration: bool,
    #[serde(default = "default_true")]
    pub text_transform: bool,
    #[serde(default = "default_true")]
    pub writing_mode: bool,
}

impl Default for TypographySettings {
    fn default() -> Self {
        Self {
            custom_font_size: true,
            drop_cap: true,
            fluid: Some(FluidTypography::default()),
            font_families: Vec::new(),
            font_sizes: Vec::new(),
            font_style: true,
            font_weight: true,
            letter_spacing: true,
            line_height: true,
            text_decoration: true,
            text_transform: true,
            writing_mode: true,
        }
    }
}

/// Fluid typography config
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FluidTypography {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_viewport_width: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_viewport_width: Option<String>,
}

/// Font family preset
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontFamilyPreset {
    pub slug: String,
    pub name: String,
    pub font_family: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub font_face: Vec<FontFace>,
}

/// Font face definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontFace {
    pub font_family: String,
    pub font_weight: String,
    pub font_style: String,
    #[serde(default)]
    pub font_stretch: Option<String>,
    pub src: Vec<String>,
}

/// Font size preset
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontSizePreset {
    pub slug: String,
    pub name: String,
    pub size: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fluid: Option<FluidFontSize>,
}

/// Fluid font size config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluidFontSize {
    pub min: String,
    pub max: String,
}

/// Per-block settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockSettings {
    #[serde(flatten)]
    pub settings: HashMap<String, serde_json::Value>,
}

/// Theme styles
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeJsonStyles {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border: Option<BorderStyles>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<ColorStyles>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing: Option<SpacingStyles>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typography: Option<TypographyStyles>,
    #[serde(default)]
    pub elements: HashMap<String, ElementStyles>,
    #[serde(default)]
    pub blocks: HashMap<String, BlockStyles>,
}

/// Border styles
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BorderStyles {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radius: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,
}

/// Color styles
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorStyles {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gradient: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// Spacing styles
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpacingStyles {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_gap: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin: Option<SpacingValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding: Option<SpacingValue>,
}

/// Spacing value (can be single or per-side)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SpacingValue {
    Single(String),
    Sides {
        #[serde(skip_serializing_if = "Option::is_none")]
        top: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        right: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        bottom: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        left: Option<String>,
    },
}

/// Typography styles
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypographyStyles {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub letter_spacing: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_height: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_decoration: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_transform: Option<String>,
}

/// Element styles
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementStyles {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border: Option<BorderStyles>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<ColorStyles>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing: Option<SpacingStyles>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typography: Option<TypographyStyles>,
}

/// Block styles
pub type BlockStyles = ElementStyles;

/// Template part definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplatePartDefinition {
    pub name: String,
    pub title: String,
    pub area: String,
}

/// Custom template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomTemplateDefinition {
    pub name: String,
    pub title: String,
    #[serde(default)]
    pub post_types: Vec<String>,
}

impl ThemeJson {
    /// Load from file
    pub async fn load(path: &Path) -> Result<Self, ThemeJsonError> {
        let content = fs::read_to_string(path).await?;
        let theme_json: Self = serde_json::from_str(&content)?;
        Ok(theme_json)
    }

    /// Load from theme directory
    pub async fn load_from_theme(theme_path: &Path) -> Result<Option<Self>, ThemeJsonError> {
        let json_path = theme_path.join("theme.json");

        if json_path.exists() {
            Ok(Some(Self::load(&json_path).await?))
        } else {
            Ok(None)
        }
    }

    /// Save to file
    pub async fn save(&self, path: &Path) -> Result<(), ThemeJsonError> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content).await?;
        Ok(())
    }

    /// Generate CSS from theme.json
    pub fn generate_css(&self) -> String {
        let mut css = String::new();

        // Generate CSS custom properties
        css.push_str(":root {\n");

        // Colors
        for color in &self.settings.color.palette {
            css.push_str(&format!(
                "  --wp--preset--color--{}: {};\n",
                color.slug, color.color
            ));
        }

        // Gradients
        for gradient in &self.settings.color.gradients {
            css.push_str(&format!(
                "  --wp--preset--gradient--{}: {};\n",
                gradient.slug, gradient.gradient
            ));
        }

        // Font families
        for family in &self.settings.typography.font_families {
            css.push_str(&format!(
                "  --wp--preset--font-family--{}: {};\n",
                family.slug, family.font_family
            ));
        }

        // Font sizes
        for size in &self.settings.typography.font_sizes {
            let value = if let Some(fluid) = &size.fluid {
                format!("clamp({}, 2.5vw, {})", fluid.min, fluid.max)
            } else {
                size.size.clone()
            };
            css.push_str(&format!(
                "  --wp--preset--font-size--{}: {};\n",
                size.slug, value
            ));
        }

        // Spacing
        for spacing in &self.settings.spacing.spacing_sizes {
            css.push_str(&format!(
                "  --wp--preset--spacing--{}: {};\n",
                spacing.slug, spacing.size
            ));
        }

        // Layout
        if let Some(ref content_size) = self.settings.layout.content_size {
            css.push_str(&format!(
                "  --wp--style--global--content-size: {};\n",
                content_size
            ));
        }

        if let Some(ref wide_size) = self.settings.layout.wide_size {
            css.push_str(&format!(
                "  --wp--style--global--wide-size: {};\n",
                wide_size
            ));
        }

        css.push_str("}\n\n");

        // Global styles
        css.push_str("body {\n");
        if let Some(ref color_styles) = self.styles.color {
            if let Some(ref bg) = color_styles.background {
                css.push_str(&format!("  background-color: {};\n", bg));
            }
            if let Some(ref text) = color_styles.text {
                css.push_str(&format!("  color: {};\n", text));
            }
        }
        if let Some(ref typography) = self.styles.typography {
            if let Some(ref family) = typography.font_family {
                css.push_str(&format!("  font-family: {};\n", family));
            }
            if let Some(ref size) = typography.font_size {
                css.push_str(&format!("  font-size: {};\n", size));
            }
            if let Some(ref line_height) = typography.line_height {
                css.push_str(&format!("  line-height: {};\n", line_height));
            }
        }
        css.push_str("}\n");

        css
    }

    /// Merge with another theme.json (for child themes)
    pub fn merge(&mut self, other: &ThemeJson) {
        // Merge color palette
        for color in &other.settings.color.palette {
            if !self.settings.color.palette.iter().any(|c| c.slug == color.slug) {
                self.settings.color.palette.push(color.clone());
            }
        }

        // Merge font families
        for family in &other.settings.typography.font_families {
            if !self.settings.typography.font_families.iter().any(|f| f.slug == family.slug) {
                self.settings.typography.font_families.push(family.clone());
            }
        }

        // Merge font sizes (child overrides parent)
        for size in &other.settings.typography.font_sizes {
            if let Some(existing) = self.settings.typography.font_sizes.iter_mut()
                .find(|s| s.slug == size.slug)
            {
                *existing = size.clone();
            } else {
                self.settings.typography.font_sizes.push(size.clone());
            }
        }

        // Merge template parts
        for part in &other.template_parts {
            if !self.template_parts.iter().any(|p| p.name == part.name) {
                self.template_parts.push(part.clone());
            }
        }

        // Merge custom templates
        for template in &other.custom_templates {
            if !self.custom_templates.iter().any(|t| t.name == template.name) {
                self.custom_templates.push(template.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_json_default() {
        let theme_json = ThemeJson::default();
        assert_eq!(theme_json.version, SCHEMA_VERSION);
    }

    #[test]
    fn test_theme_json_serialize() {
        let theme_json = ThemeJson::default();
        let json = serde_json::to_string_pretty(&theme_json).unwrap();
        assert!(json.contains("\"version\""));
        assert!(json.contains("\"settings\""));
    }

    #[test]
    fn test_generate_css() {
        let mut theme_json = ThemeJson::default();
        theme_json.settings.color.palette.push(ColorPreset {
            slug: "primary".to_string(),
            name: "Primary".to_string(),
            color: "#0073aa".to_string(),
        });

        let css = theme_json.generate_css();
        assert!(css.contains("--wp--preset--color--primary: #0073aa"));
    }

    #[test]
    fn test_merge() {
        let mut parent = ThemeJson::default();
        parent.settings.color.palette.push(ColorPreset {
            slug: "primary".to_string(),
            name: "Primary".to_string(),
            color: "#0073aa".to_string(),
        });

        let mut child = ThemeJson::default();
        child.settings.color.palette.push(ColorPreset {
            slug: "secondary".to_string(),
            name: "Secondary".to_string(),
            color: "#23282d".to_string(),
        });

        parent.merge(&child);
        assert_eq!(parent.settings.color.palette.len(), 2);
    }
}
