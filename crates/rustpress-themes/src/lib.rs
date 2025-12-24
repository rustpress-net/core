//! RustPress Themes
//!
//! A comprehensive theme system for the RustPress CMS, providing:
//!
//! - Theme manifest format and loading
//! - Template hierarchy and engine (Tera-based)
//! - Theme customizer with live preview
//! - Asset compilation (CSS/SCSS/JS)
//! - Responsive images
//! - Block patterns
//! - Full-site editing support
//! - Theme variations and dark mode
//! - Accessibility and performance tools

pub mod manifest;
pub mod templates;
pub mod customizer;
pub mod settings;
pub mod assets;
pub mod images;
pub mod critical_css;
pub mod patterns;
pub mod design_tokens;
pub mod child_theme;
pub mod manager;
pub mod starter_content;
pub mod export;
pub mod marketplace;
pub mod fse;
pub mod theme_json;
pub mod variations;
pub mod quality;
pub mod docs;

// Re-exports for convenience
pub use manifest::ThemeManifest;
pub use templates::{TemplateEngine, TemplateHierarchy, TemplatePartManager};
pub use customizer::ThemeCustomizer;
pub use settings::{ThemeSettings, GlobalSettingsRegistry};
pub use assets::{AssetCompiler, AssetConfig};
pub use images::{ResponsiveImageGenerator, ImageSize};
pub use critical_css::{CriticalCssExtractor, CriticalCssConfig};
pub use patterns::{BlockPattern, PatternRegistry};
pub use design_tokens::{ColorPalette, TypographySettings, LayoutSettings, DesignTokens};
pub use child_theme::{ThemeInheritance, ChildThemeBuilder};
pub use manager::{ThemeManager, ThemePreview, RegisteredTheme};
pub use starter_content::StarterContent;
pub use export::{ThemeExporter, ThemeImporter, ExportOptions};
pub use marketplace::{MarketplaceClient, MarketplaceConfig, ThemeListing};
pub use fse::{FseManager, FseTemplate, TemplatePart};
pub use theme_json::ThemeJson;
pub use variations::{StyleVariation, VariationManager, DarkModeConfig};
pub use quality::{AmpCompatibility, AccessibilityChecker, PerformanceScorer};
pub use docs::{DocGenerator, ScreenshotGenerator};

use thiserror::Error;

/// Theme system errors
#[derive(Debug, Error)]
pub enum ThemeError {
    #[error("Theme not found: {0}")]
    NotFound(String),

    #[error("Invalid theme manifest: {0}")]
    InvalidManifest(String),

    #[error("Template error: {0}")]
    Template(String),

    #[error("Asset error: {0}")]
    Asset(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Theme type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeType {
    /// Classic PHP-style theme
    Classic,
    /// Block-based theme
    Block,
    /// Hybrid (supports both)
    Hybrid,
}

/// Prelude for common imports
pub mod prelude {
    pub use crate::ThemeError;
    pub use crate::ThemeType;
    pub use crate::ThemeManifest;
    pub use crate::ThemeManager;
    pub use crate::TemplateEngine;
    pub use crate::ThemeCustomizer;
    pub use crate::ThemeSettings;
    pub use crate::AssetCompiler;
    pub use crate::PatternRegistry;
    pub use crate::DesignTokens;
}
