//! Design Tokens System
//!
//! Color palette (201), typography (202), and layout (203) management.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;

/// Color definition with accessibility metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color {
    pub slug: String,
    pub name: String,
    pub color: String,
    /// Computed luminance for contrast calculations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub luminance: Option<f64>,
    /// Suggested foreground color for this background
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contrast_color: Option<String>,
}

impl Color {
    pub fn new(slug: &str, name: &str, color: &str) -> Self {
        let mut c = Self {
            slug: slug.to_string(),
            name: name.to_string(),
            color: color.to_string(),
            luminance: None,
            contrast_color: None,
        };
        c.compute_luminance();
        c.compute_contrast_color();
        c
    }

    fn compute_luminance(&mut self) {
        if let Some(rgb) = parse_hex_color(&self.color) {
            // Calculate relative luminance per WCAG 2.1
            let r = srgb_to_linear(rgb.0 as f64 / 255.0);
            let g = srgb_to_linear(rgb.1 as f64 / 255.0);
            let b = srgb_to_linear(rgb.2 as f64 / 255.0);
            self.luminance = Some(0.2126 * r + 0.7152 * g + 0.0722 * b);
        }
    }

    fn compute_contrast_color(&mut self) {
        if let Some(lum) = self.luminance {
            // Use white for dark backgrounds, black for light
            self.contrast_color = Some(if lum > 0.179 {
                "#000000".to_string()
            } else {
                "#ffffff".to_string()
            });
        }
    }

    /// Calculate contrast ratio with another color
    pub fn contrast_ratio(&self, other: &Color) -> Option<f64> {
        if let (Some(l1), Some(l2)) = (self.luminance, other.luminance) {
            let lighter = l1.max(l2);
            let darker = l1.min(l2);
            Some((lighter + 0.05) / (darker + 0.05))
        } else {
            None
        }
    }

    /// Check WCAG AA compliance (4.5:1 for normal text)
    pub fn passes_aa(&self, other: &Color) -> bool {
        self.contrast_ratio(other).map_or(false, |r| r >= 4.5)
    }

    /// Check WCAG AAA compliance (7:1 for normal text)
    pub fn passes_aaa(&self, other: &Color) -> bool {
        self.contrast_ratio(other).map_or(false, |r| r >= 7.0)
    }
}

fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');

    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some((r, g, b))
        }
        6 | 8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some((r, g, b))
        }
        _ => None,
    }
}

fn srgb_to_linear(value: f64) -> f64 {
    if value <= 0.03928 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

/// Color palette manager
pub struct ColorPalette {
    colors: Arc<RwLock<Vec<Color>>>,
    gradients: Arc<RwLock<Vec<Gradient>>>,
    #[allow(dead_code)]
    duotones: Arc<RwLock<Vec<Duotone>>>,
}

/// Gradient definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gradient {
    pub slug: String,
    pub name: String,
    pub gradient: String,
}

/// Duotone filter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Duotone {
    pub slug: String,
    pub name: String,
    pub colors: Vec<String>,
}

impl ColorPalette {
    pub fn new() -> Self {
        Self {
            colors: Arc::new(RwLock::new(Vec::new())),
            gradients: Arc::new(RwLock::new(Vec::new())),
            duotones: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add default colors
    pub fn with_defaults(self) -> Self {
        let defaults = vec![
            Color::new("black", "Black", "#000000"),
            Color::new("white", "White", "#ffffff"),
            Color::new("primary", "Primary", "#0073aa"),
            Color::new("secondary", "Secondary", "#23282d"),
            Color::new("accent", "Accent", "#00a0d2"),
            Color::new("success", "Success", "#46b450"),
            Color::new("warning", "Warning", "#ffb900"),
            Color::new("error", "Error", "#dc3232"),
            Color::new("gray-100", "Gray 100", "#f7f7f7"),
            Color::new("gray-200", "Gray 200", "#eeeeee"),
            Color::new("gray-300", "Gray 300", "#dddddd"),
            Color::new("gray-400", "Gray 400", "#cccccc"),
            Color::new("gray-500", "Gray 500", "#aaaaaa"),
            Color::new("gray-600", "Gray 600", "#888888"),
            Color::new("gray-700", "Gray 700", "#666666"),
            Color::new("gray-800", "Gray 800", "#444444"),
            Color::new("gray-900", "Gray 900", "#222222"),
        ];

        *self.colors.write() = defaults;

        let gradients = vec![
            Gradient {
                slug: "vivid-cyan-blue-to-vivid-purple".to_string(),
                name: "Vivid cyan blue to vivid purple".to_string(),
                gradient: "linear-gradient(135deg,rgba(6,147,227,1) 0%,rgb(155,81,224) 100%)".to_string(),
            },
            Gradient {
                slug: "light-green-cyan-to-vivid-green-cyan".to_string(),
                name: "Light green cyan to vivid green cyan".to_string(),
                gradient: "linear-gradient(135deg,rgb(122,220,180) 0%,rgb(0,208,130) 100%)".to_string(),
            },
        ];

        *self.gradients.write() = gradients;

        self
    }

    /// Add a color
    pub fn add_color(&self, color: Color) {
        self.colors.write().push(color);
    }

    /// Remove a color by slug
    pub fn remove_color(&self, slug: &str) {
        self.colors.write().retain(|c| c.slug != slug);
    }

    /// Get color by slug
    pub fn get_color(&self, slug: &str) -> Option<Color> {
        self.colors.read().iter().find(|c| c.slug == slug).cloned()
    }

    /// Get all colors
    pub fn get_colors(&self) -> Vec<Color> {
        self.colors.read().clone()
    }

    /// Add a gradient
    pub fn add_gradient(&self, gradient: Gradient) {
        self.gradients.write().push(gradient);
    }

    /// Get all gradients
    pub fn get_gradients(&self) -> Vec<Gradient> {
        self.gradients.read().clone()
    }

    /// Generate CSS custom properties
    pub fn generate_css_variables(&self) -> String {
        let mut css = String::from(":root {\n");

        for color in self.colors.read().iter() {
            css.push_str(&format!(
                "  --wp--preset--color--{}: {};\n",
                color.slug, color.color
            ));
        }

        for gradient in self.gradients.read().iter() {
            css.push_str(&format!(
                "  --wp--preset--gradient--{}: {};\n",
                gradient.slug, gradient.gradient
            ));
        }

        css.push_str("}\n");
        css
    }

    /// Generate utility classes
    pub fn generate_utility_classes(&self) -> String {
        let mut css = String::new();

        for color in self.colors.read().iter() {
            // Background color
            css.push_str(&format!(
                ".has-{}-background-color {{ background-color: var(--wp--preset--color--{}) !important; }}\n",
                color.slug, color.slug
            ));

            // Text color
            css.push_str(&format!(
                ".has-{}-color {{ color: var(--wp--preset--color--{}) !important; }}\n",
                color.slug, color.slug
            ));

            // Border color
            css.push_str(&format!(
                ".has-{}-border-color {{ border-color: var(--wp--preset--color--{}) !important; }}\n",
                color.slug, color.slug
            ));
        }

        for gradient in self.gradients.read().iter() {
            css.push_str(&format!(
                ".has-{}-gradient-background {{ background: var(--wp--preset--gradient--{}) !important; }}\n",
                gradient.slug, gradient.slug
            ));
        }

        css
    }
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self::new()
    }
}

/// Typography settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypographySettings {
    /// Font families
    pub font_families: Vec<FontFamily>,
    /// Font sizes
    pub font_sizes: Vec<FontSize>,
    /// Line heights
    pub line_heights: Vec<LineHeight>,
    /// Letter spacings
    pub letter_spacings: Vec<LetterSpacing>,
    /// Default font family
    pub default_font_family: Option<String>,
    /// Fluid typography enabled
    pub fluid: bool,
}

/// Font family definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontFamily {
    pub slug: String,
    pub name: String,
    pub font_family: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_face: Option<Vec<FontFace>>,
}

/// Font face for custom fonts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontFace {
    pub font_family: String,
    pub font_style: String,
    pub font_weight: String,
    pub src: Vec<String>,
}

/// Font size definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSize {
    pub slug: String,
    pub name: String,
    pub size: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fluid: Option<FluidFontSize>,
}

/// Fluid font size configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluidFontSize {
    pub min: String,
    pub max: String,
}

/// Line height definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineHeight {
    pub slug: String,
    pub name: String,
    pub value: String,
}

/// Letter spacing definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LetterSpacing {
    pub slug: String,
    pub name: String,
    pub value: String,
}

impl TypographySettings {
    pub fn new() -> Self {
        Self {
            font_families: Vec::new(),
            font_sizes: Vec::new(),
            line_heights: Vec::new(),
            letter_spacings: Vec::new(),
            default_font_family: None,
            fluid: true,
        }
    }

    /// Add default typography settings
    pub fn with_defaults(mut self) -> Self {
        self.font_families = vec![
            FontFamily {
                slug: "system-font".to_string(),
                name: "System Font".to_string(),
                font_family: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen-Sans, Ubuntu, Cantarell, 'Helvetica Neue', sans-serif".to_string(),
                font_face: None,
            },
            FontFamily {
                slug: "serif".to_string(),
                name: "Serif".to_string(),
                font_family: "Georgia, 'Times New Roman', Times, serif".to_string(),
                font_face: None,
            },
            FontFamily {
                slug: "monospace".to_string(),
                name: "Monospace".to_string(),
                font_family: "Consolas, Monaco, 'Andale Mono', 'Ubuntu Mono', monospace".to_string(),
                font_face: None,
            },
        ];

        self.font_sizes = vec![
            FontSize {
                slug: "small".to_string(),
                name: "Small".to_string(),
                size: "0.875rem".to_string(),
                fluid: Some(FluidFontSize {
                    min: "0.75rem".to_string(),
                    max: "0.875rem".to_string(),
                }),
            },
            FontSize {
                slug: "medium".to_string(),
                name: "Medium".to_string(),
                size: "1rem".to_string(),
                fluid: None,
            },
            FontSize {
                slug: "large".to_string(),
                name: "Large".to_string(),
                size: "1.25rem".to_string(),
                fluid: Some(FluidFontSize {
                    min: "1.125rem".to_string(),
                    max: "1.25rem".to_string(),
                }),
            },
            FontSize {
                slug: "x-large".to_string(),
                name: "Extra Large".to_string(),
                size: "1.5rem".to_string(),
                fluid: Some(FluidFontSize {
                    min: "1.25rem".to_string(),
                    max: "1.5rem".to_string(),
                }),
            },
            FontSize {
                slug: "xx-large".to_string(),
                name: "Huge".to_string(),
                size: "2.25rem".to_string(),
                fluid: Some(FluidFontSize {
                    min: "1.75rem".to_string(),
                    max: "2.25rem".to_string(),
                }),
            },
        ];

        self.line_heights = vec![
            LineHeight { slug: "tight".to_string(), name: "Tight".to_string(), value: "1.1".to_string() },
            LineHeight { slug: "normal".to_string(), name: "Normal".to_string(), value: "1.5".to_string() },
            LineHeight { slug: "relaxed".to_string(), name: "Relaxed".to_string(), value: "1.75".to_string() },
            LineHeight { slug: "loose".to_string(), name: "Loose".to_string(), value: "2".to_string() },
        ];

        self.default_font_family = Some("system-font".to_string());

        self
    }

    /// Generate CSS custom properties
    pub fn generate_css_variables(&self) -> String {
        let mut css = String::from(":root {\n");

        for family in &self.font_families {
            css.push_str(&format!(
                "  --wp--preset--font-family--{}: {};\n",
                family.slug, family.font_family
            ));
        }

        for size in &self.font_sizes {
            if self.fluid {
                if let Some(fluid) = &size.fluid {
                    css.push_str(&format!(
                        "  --wp--preset--font-size--{}: clamp({}, 2.5vw, {});\n",
                        size.slug, fluid.min, fluid.max
                    ));
                } else {
                    css.push_str(&format!(
                        "  --wp--preset--font-size--{}: {};\n",
                        size.slug, size.size
                    ));
                }
            } else {
                css.push_str(&format!(
                    "  --wp--preset--font-size--{}: {};\n",
                    size.slug, size.size
                ));
            }
        }

        css.push_str("}\n");
        css
    }

    /// Generate @font-face rules
    pub fn generate_font_faces(&self) -> String {
        let mut css = String::new();

        for family in &self.font_families {
            if let Some(faces) = &family.font_face {
                for face in faces {
                    css.push_str("@font-face {\n");
                    css.push_str(&format!("  font-family: '{}';\n", face.font_family));
                    css.push_str(&format!("  font-style: {};\n", face.font_style));
                    css.push_str(&format!("  font-weight: {};\n", face.font_weight));

                    let src: Vec<String> = face.src.iter().map(|s| {
                        if s.ends_with(".woff2") {
                            format!("url('{}') format('woff2')", s)
                        } else if s.ends_with(".woff") {
                            format!("url('{}') format('woff')", s)
                        } else if s.ends_with(".ttf") {
                            format!("url('{}') format('truetype')", s)
                        } else {
                            format!("url('{}')", s)
                        }
                    }).collect();

                    css.push_str(&format!("  src: {};\n", src.join(", ")));
                    css.push_str("  font-display: swap;\n");
                    css.push_str("}\n\n");
                }
            }
        }

        css
    }

    /// Generate utility classes
    pub fn generate_utility_classes(&self) -> String {
        let mut css = String::new();

        for family in &self.font_families {
            css.push_str(&format!(
                ".has-{}-font-family {{ font-family: var(--wp--preset--font-family--{}) !important; }}\n",
                family.slug, family.slug
            ));
        }

        for size in &self.font_sizes {
            css.push_str(&format!(
                ".has-{}-font-size {{ font-size: var(--wp--preset--font-size--{}) !important; }}\n",
                size.slug, size.slug
            ));
        }

        css
    }
}

impl Default for TypographySettings {
    fn default() -> Self {
        Self::new()
    }
}

/// Layout settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutSettings {
    /// Content width
    pub content_size: String,
    /// Wide width
    pub wide_size: String,
    /// Spacing presets
    pub spacing: Vec<SpacingPreset>,
    /// Custom spacing enabled
    pub custom_spacing: bool,
    /// Spacing units
    pub spacing_units: Vec<String>,
    /// Block gap (default spacing between blocks)
    pub block_gap: Option<String>,
}

/// Spacing preset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacingPreset {
    pub slug: String,
    pub name: String,
    pub size: String,
}

impl LayoutSettings {
    pub fn new() -> Self {
        Self {
            content_size: "650px".to_string(),
            wide_size: "1200px".to_string(),
            spacing: Vec::new(),
            custom_spacing: true,
            spacing_units: vec!["px".to_string(), "em".to_string(), "rem".to_string(), "%".to_string(), "vw".to_string()],
            block_gap: Some("2rem".to_string()),
        }
    }

    /// Add default spacing presets
    pub fn with_defaults(mut self) -> Self {
        self.spacing = vec![
            SpacingPreset { slug: "20".to_string(), name: "1".to_string(), size: "min(1.5rem, 2vw)".to_string() },
            SpacingPreset { slug: "30".to_string(), name: "2".to_string(), size: "min(2.5rem, 3vw)".to_string() },
            SpacingPreset { slug: "40".to_string(), name: "3".to_string(), size: "min(4rem, 5vw)".to_string() },
            SpacingPreset { slug: "50".to_string(), name: "4".to_string(), size: "min(6.5rem, 8vw)".to_string() },
            SpacingPreset { slug: "60".to_string(), name: "5".to_string(), size: "min(10.5rem, 13vw)".to_string() },
            SpacingPreset { slug: "70".to_string(), name: "6".to_string(), size: "min(12rem, 15vw)".to_string() },
            SpacingPreset { slug: "80".to_string(), name: "7".to_string(), size: "min(14rem, 18vw)".to_string() },
        ];

        self
    }

    /// Generate CSS custom properties
    pub fn generate_css_variables(&self) -> String {
        let mut css = String::from(":root {\n");

        css.push_str(&format!(
            "  --wp--style--global--content-size: {};\n",
            self.content_size
        ));

        css.push_str(&format!(
            "  --wp--style--global--wide-size: {};\n",
            self.wide_size
        ));

        for spacing in &self.spacing {
            css.push_str(&format!(
                "  --wp--preset--spacing--{}: {};\n",
                spacing.slug, spacing.size
            ));
        }

        if let Some(gap) = &self.block_gap {
            css.push_str(&format!("  --wp--style--block-gap: {};\n", gap));
        }

        css.push_str("}\n");
        css
    }

    /// Generate layout utility classes
    pub fn generate_utility_classes(&self) -> String {
        let mut css = String::new();

        // Alignment classes
        css.push_str(".alignwide { max-width: var(--wp--style--global--wide-size); margin-left: auto; margin-right: auto; }\n");
        css.push_str(".alignfull { max-width: none; margin-left: calc(-50vw + 50%); margin-right: calc(-50vw + 50%); width: 100vw; }\n");

        // Spacing classes
        for spacing in &self.spacing {
            css.push_str(&format!(
                ".has-{}-padding {{ padding: var(--wp--preset--spacing--{}) !important; }}\n",
                spacing.slug, spacing.slug
            ));
            css.push_str(&format!(
                ".has-{}-margin {{ margin: var(--wp--preset--spacing--{}) !important; }}\n",
                spacing.slug, spacing.slug
            ));
        }

        css
    }
}

impl Default for LayoutSettings {
    fn default() -> Self {
        Self::new()
    }
}

/// Combined design tokens
pub struct DesignTokens {
    pub colors: ColorPalette,
    pub typography: TypographySettings,
    pub layout: LayoutSettings,
}

impl DesignTokens {
    pub fn new() -> Self {
        Self {
            colors: ColorPalette::new().with_defaults(),
            typography: TypographySettings::new().with_defaults(),
            layout: LayoutSettings::new().with_defaults(),
        }
    }

    /// Generate complete CSS
    pub fn generate_css(&self) -> String {
        let mut css = String::new();

        // Font faces first
        css.push_str(&self.typography.generate_font_faces());

        // Custom properties
        css.push_str(&self.colors.generate_css_variables());
        css.push_str(&self.typography.generate_css_variables());
        css.push_str(&self.layout.generate_css_variables());

        // Utility classes
        css.push_str("\n/* Color Utilities */\n");
        css.push_str(&self.colors.generate_utility_classes());

        css.push_str("\n/* Typography Utilities */\n");
        css.push_str(&self.typography.generate_utility_classes());

        css.push_str("\n/* Layout Utilities */\n");
        css.push_str(&self.layout.generate_utility_classes());

        css
    }
}

impl Default for DesignTokens {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_contrast() {
        let white = Color::new("white", "White", "#ffffff");
        let black = Color::new("black", "Black", "#000000");

        let ratio = white.contrast_ratio(&black).unwrap();
        assert!((ratio - 21.0).abs() < 0.01); // Max contrast ratio is 21:1

        assert!(white.passes_aaa(&black));
    }

    #[test]
    fn test_parse_hex() {
        assert_eq!(parse_hex_color("#fff"), Some((255, 255, 255)));
        assert_eq!(parse_hex_color("#000000"), Some((0, 0, 0)));
        assert_eq!(parse_hex_color("#ff5733"), Some((255, 87, 51)));
    }

    #[test]
    fn test_color_palette_css() {
        let palette = ColorPalette::new().with_defaults();
        let css = palette.generate_css_variables();

        assert!(css.contains("--wp--preset--color--primary"));
        assert!(css.contains("--wp--preset--color--white"));
    }

    #[test]
    fn test_typography_css() {
        let typography = TypographySettings::new().with_defaults();
        let css = typography.generate_css_variables();

        assert!(css.contains("--wp--preset--font-family--system-font"));
        assert!(css.contains("--wp--preset--font-size--medium"));
    }

    #[test]
    fn test_layout_css() {
        let layout = LayoutSettings::new().with_defaults();
        let css = layout.generate_css_variables();

        assert!(css.contains("--wp--style--global--content-size"));
        assert!(css.contains("--wp--preset--spacing--40"));
    }
}
