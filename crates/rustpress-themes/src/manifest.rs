//! Theme Manifest Format (TOML-based)
//!
//! Comprehensive theme configuration and metadata.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Theme manifest - the main configuration file for a theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeManifest {
    /// Theme metadata
    pub theme: ThemeMeta,

    /// Parent theme (for child themes)
    #[serde(default)]
    pub parent: Option<ParentTheme>,

    /// Template configuration
    #[serde(default)]
    pub templates: TemplateSection,

    /// Assets configuration
    #[serde(default)]
    pub assets: AssetSection,

    /// Theme settings/customizer
    #[serde(default)]
    pub settings: SettingsSection,

    /// Color palette
    #[serde(default)]
    pub colors: ColorSection,

    /// Typography settings
    #[serde(default)]
    pub typography: TypographySection,

    /// Layout configuration
    #[serde(default)]
    pub layout: LayoutSection,

    /// Block patterns
    #[serde(default)]
    pub patterns: Vec<PatternDefinition>,

    /// Block theme configuration
    #[serde(default)]
    pub blocks: BlockThemeSection,

    /// Theme variations
    #[serde(default)]
    pub variations: Vec<ThemeVariation>,

    /// Starter content
    #[serde(default)]
    pub starter_content: Option<StarterContent>,

    /// Support declarations
    #[serde(default)]
    pub supports: ThemeSupports,

    /// Feature flags
    #[serde(default)]
    pub features: HashMap<String, bool>,
}

/// Theme metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeMeta {
    /// Unique theme identifier
    pub id: String,

    /// Display name
    pub name: String,

    /// Theme version
    pub version: String,

    /// Short description
    pub description: String,

    /// Author name
    pub author: String,

    /// Author URL
    #[serde(default)]
    pub author_url: Option<String>,

    /// Theme homepage
    #[serde(default)]
    pub homepage: Option<String>,

    /// License identifier
    #[serde(default = "default_license")]
    pub license: String,

    /// License URL
    #[serde(default)]
    pub license_url: Option<String>,

    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,

    /// Screenshot path
    #[serde(default)]
    pub screenshot: Option<String>,

    /// Minimum RustPress version
    #[serde(default)]
    pub requires_rustpress: Option<String>,

    /// Tested up to version
    #[serde(default)]
    pub tested_up_to: Option<String>,

    /// Text domain for translations
    #[serde(default)]
    pub text_domain: Option<String>,

    /// Domain path for translations
    #[serde(default)]
    pub domain_path: Option<String>,

    /// Theme type
    #[serde(default)]
    pub theme_type: ThemeType,
}

fn default_license() -> String {
    "GPL-2.0-or-later".to_string()
}

/// Theme type
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeType {
    /// Traditional PHP-style theme
    #[default]
    Classic,
    /// Full-site editing block theme
    Block,
    /// Hybrid theme with both modes
    Hybrid,
}

/// Parent theme reference for child themes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentTheme {
    /// Parent theme ID
    pub id: String,

    /// Minimum parent version
    #[serde(default)]
    pub min_version: Option<String>,
}

/// Template configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemplateSection {
    /// Template directory
    #[serde(default = "default_template_dir")]
    pub directory: String,

    /// Template file extension
    #[serde(default = "default_template_ext")]
    pub extension: String,

    /// Custom template hierarchy overrides
    #[serde(default)]
    pub hierarchy: HashMap<String, Vec<String>>,

    /// Template parts directory
    #[serde(default = "default_parts_dir")]
    pub parts_directory: String,

    /// Locked templates (cannot be edited)
    #[serde(default)]
    pub locked: Vec<String>,

    /// Template definitions
    #[serde(default)]
    pub definitions: Vec<TemplateDefinition>,
}

fn default_template_dir() -> String {
    "templates".to_string()
}

fn default_template_ext() -> String {
    "html".to_string()
}

fn default_parts_dir() -> String {
    "parts".to_string()
}

/// Template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateDefinition {
    /// Template name/slug
    pub name: String,

    /// Display title
    pub title: String,

    /// Description
    #[serde(default)]
    pub description: Option<String>,

    /// Post types this template applies to
    #[serde(default)]
    pub post_types: Vec<String>,

    /// Template file path
    pub file: String,
}

/// Asset configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssetSection {
    /// CSS files
    #[serde(default)]
    pub css: Vec<AssetFile>,

    /// JavaScript files
    #[serde(default)]
    pub js: Vec<AssetFile>,

    /// SCSS source files
    #[serde(default)]
    pub scss: Vec<ScssConfig>,

    /// Asset output directory
    #[serde(default = "default_asset_output")]
    pub output_directory: String,

    /// Enable asset minification
    #[serde(default = "default_true")]
    pub minify: bool,

    /// Enable source maps
    #[serde(default)]
    pub sourcemaps: bool,

    /// Asset versioning strategy
    #[serde(default)]
    pub versioning: AssetVersioning,

    /// Critical CSS configuration
    #[serde(default)]
    pub critical_css: Option<CriticalCssConfig>,

    /// Print stylesheet
    #[serde(default)]
    pub print_stylesheet: Option<String>,
}

fn default_asset_output() -> String {
    "dist".to_string()
}

fn default_true() -> bool {
    true
}

/// Asset file configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetFile {
    /// File path
    pub path: String,

    /// Handle/identifier
    pub handle: String,

    /// Dependencies
    #[serde(default)]
    pub deps: Vec<String>,

    /// Load in footer (for JS)
    #[serde(default)]
    pub footer: bool,

    /// Media query (for CSS)
    #[serde(default)]
    pub media: Option<String>,

    /// Conditional loading
    #[serde(default)]
    pub condition: Option<String>,

    /// Inline asset content
    #[serde(default)]
    pub inline: bool,

    /// Async/defer loading
    #[serde(default)]
    pub loading: AssetLoading,
}

/// Asset loading strategy
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetLoading {
    #[default]
    Normal,
    Async,
    Defer,
    Module,
}

/// Asset versioning strategy
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetVersioning {
    #[default]
    ThemeVersion,
    ContentHash,
    Timestamp,
    None,
}

/// SCSS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScssConfig {
    /// Source file
    pub source: String,

    /// Output file
    pub output: String,

    /// Include paths
    #[serde(default)]
    pub include_paths: Vec<String>,
}

/// Critical CSS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalCssConfig {
    /// Enable critical CSS
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Viewport widths to analyze
    #[serde(default = "default_viewports")]
    pub viewports: Vec<ViewportConfig>,

    /// Maximum critical CSS size (bytes)
    #[serde(default = "default_critical_size")]
    pub max_size: usize,

    /// Cache critical CSS
    #[serde(default = "default_true")]
    pub cache: bool,
}

fn default_viewports() -> Vec<ViewportConfig> {
    vec![
        ViewportConfig { width: 375, height: 667, name: "mobile".to_string() },
        ViewportConfig { width: 1024, height: 768, name: "tablet".to_string() },
        ViewportConfig { width: 1440, height: 900, name: "desktop".to_string() },
    ]
}

fn default_critical_size() -> usize {
    14000 // ~14KB
}

/// Viewport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportConfig {
    pub width: u32,
    pub height: u32,
    pub name: String,
}

/// Settings/customizer section
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SettingsSection {
    /// Setting panels
    #[serde(default)]
    pub panels: Vec<SettingPanel>,

    /// Setting sections
    #[serde(default)]
    pub sections: Vec<SettingSection>,

    /// Individual settings
    #[serde(default)]
    pub settings: Vec<SettingDefinition>,
}

/// Setting panel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingPanel {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub priority: i32,
}

/// Setting section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingSection {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub panel: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub priority: i32,
}

/// Setting definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingDefinition {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub section: Option<String>,
    pub setting_type: SettingType,
    #[serde(default)]
    pub default: Option<serde_json::Value>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub transport: SettingTransport,
    #[serde(default)]
    pub choices: Option<Vec<SettingChoice>>,
    #[serde(default)]
    pub sanitize: Option<String>,
    #[serde(default)]
    pub live_preview: bool,
}

/// Setting type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettingType {
    Text,
    Textarea,
    Number,
    Range,
    Color,
    Image,
    File,
    Checkbox,
    Radio,
    Select,
    Dropdown,
    Font,
    Spacing,
    Background,
    Code,
    Html,
}

/// Setting transport
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SettingTransport {
    #[default]
    Refresh,
    PostMessage,
}

/// Setting choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingChoice {
    pub value: String,
    pub label: String,
}

/// Color section
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ColorSection {
    /// Color palette
    #[serde(default)]
    pub palette: Vec<ColorDefinition>,

    /// Gradients
    #[serde(default)]
    pub gradients: Vec<GradientDefinition>,

    /// Duotone presets
    #[serde(default)]
    pub duotone: Vec<DuotoneDefinition>,

    /// Custom color picker enabled
    #[serde(default = "default_true")]
    pub custom_colors: bool,

    /// Custom gradients enabled
    #[serde(default = "default_true")]
    pub custom_gradients: bool,

    /// Link color support
    #[serde(default)]
    pub link_color: bool,

    /// Background color support
    #[serde(default = "default_true")]
    pub background: bool,

    /// Text color support
    #[serde(default = "default_true")]
    pub text: bool,
}

/// Color definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorDefinition {
    pub slug: String,
    pub name: String,
    pub color: String,
}

/// Gradient definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientDefinition {
    pub slug: String,
    pub name: String,
    pub gradient: String,
}

/// Duotone definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuotoneDefinition {
    pub slug: String,
    pub name: String,
    pub colors: [String; 2],
}

/// Typography section
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TypographySection {
    /// Font families
    #[serde(default)]
    pub font_families: Vec<FontFamily>,

    /// Font sizes
    #[serde(default)]
    pub font_sizes: Vec<FontSize>,

    /// Custom font sizes enabled
    #[serde(default = "default_true")]
    pub custom_font_sizes: bool,

    /// Line height support
    #[serde(default)]
    pub line_height: bool,

    /// Letter spacing support
    #[serde(default)]
    pub letter_spacing: bool,

    /// Text decoration support
    #[serde(default)]
    pub text_decoration: bool,

    /// Text transform support
    #[serde(default)]
    pub text_transform: bool,

    /// Drop cap support
    #[serde(default)]
    pub drop_cap: bool,

    /// Fluid typography
    #[serde(default)]
    pub fluid: Option<FluidTypography>,
}

/// Font family
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontFamily {
    pub slug: String,
    pub name: String,
    pub font_family: String,
    #[serde(default)]
    pub font_face: Vec<FontFace>,
}

/// Font face definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontFace {
    pub font_family: String,
    pub font_weight: String,
    #[serde(default = "default_font_style")]
    pub font_style: String,
    pub src: Vec<String>,
    #[serde(default)]
    pub font_display: Option<String>,
}

fn default_font_style() -> String {
    "normal".to_string()
}

/// Font size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSize {
    pub slug: String,
    pub name: String,
    pub size: String,
    #[serde(default)]
    pub fluid: Option<FluidFontSize>,
}

/// Fluid font size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluidFontSize {
    pub min: String,
    pub max: String,
}

/// Fluid typography configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluidTypography {
    #[serde(default = "default_min_viewport")]
    pub min_viewport_width: String,
    #[serde(default = "default_max_viewport")]
    pub max_viewport_width: String,
}

fn default_min_viewport() -> String {
    "320px".to_string()
}

fn default_max_viewport() -> String {
    "1600px".to_string()
}

/// Layout section
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LayoutSection {
    /// Content width
    #[serde(default)]
    pub content_width: Option<String>,

    /// Wide width
    #[serde(default)]
    pub wide_width: Option<String>,

    /// Sidebar positions
    #[serde(default)]
    pub sidebars: Vec<SidebarDefinition>,

    /// Widget areas
    #[serde(default)]
    pub widget_areas: Vec<WidgetArea>,

    /// Menu locations
    #[serde(default)]
    pub menu_locations: Vec<MenuLocation>,

    /// Layout presets
    #[serde(default)]
    pub presets: Vec<LayoutPreset>,

    /// Spacing presets
    #[serde(default)]
    pub spacing: SpacingConfig,
}

/// Sidebar definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidebarDefinition {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub position: SidebarPosition,
}

/// Sidebar position
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SidebarPosition {
    #[default]
    Right,
    Left,
    None,
}

/// Widget area
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetArea {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub before_widget: Option<String>,
    #[serde(default)]
    pub after_widget: Option<String>,
    #[serde(default)]
    pub before_title: Option<String>,
    #[serde(default)]
    pub after_title: Option<String>,
}

/// Menu location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuLocation {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// Layout preset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutPreset {
    pub slug: String,
    pub name: String,
    pub sidebar: SidebarPosition,
    #[serde(default)]
    pub content_width: Option<String>,
}

/// Spacing configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpacingConfig {
    #[serde(default)]
    pub units: Vec<String>,
    #[serde(default)]
    pub presets: Vec<SpacingPreset>,
    #[serde(default)]
    pub custom_spacing: bool,
    #[serde(default)]
    pub block_gap: Option<String>,
}

/// Spacing preset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacingPreset {
    pub slug: String,
    pub name: String,
    pub size: String,
}

/// Block pattern definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternDefinition {
    pub name: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub content: String,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub viewport_width: Option<u32>,
    #[serde(default)]
    pub block_types: Vec<String>,
    #[serde(default)]
    pub inserter: bool,
}

/// Block theme section
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlockThemeSection {
    /// Enable full-site editing
    #[serde(default)]
    pub full_site_editing: bool,

    /// Template parts
    #[serde(default)]
    pub template_parts: Vec<TemplatePartDefinition>,

    /// Custom template types
    #[serde(default)]
    pub custom_templates: Vec<CustomTemplateDefinition>,

    /// Block styles
    #[serde(default)]
    pub styles: Vec<BlockStyleDefinition>,

    /// Block variations
    #[serde(default)]
    pub variations: Vec<BlockVariation>,
}

/// Template part definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplatePartDefinition {
    pub name: String,
    pub title: String,
    pub area: TemplatePartArea,
    #[serde(default)]
    pub file: Option<String>,
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
}

/// Custom template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTemplateDefinition {
    pub name: String,
    pub title: String,
    #[serde(default)]
    pub post_types: Vec<String>,
}

/// Block style definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockStyleDefinition {
    pub block: String,
    pub name: String,
    pub label: String,
    #[serde(default)]
    pub is_default: bool,
    #[serde(default)]
    pub style_handle: Option<String>,
    #[serde(default)]
    pub inline_style: Option<String>,
}

/// Block variation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockVariation {
    pub block: String,
    pub name: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub attributes: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub inner_blocks: Vec<serde_json::Value>,
    #[serde(default)]
    pub is_default: bool,
}

/// Theme variation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeVariation {
    pub slug: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    /// Settings overrides
    #[serde(default)]
    pub settings: HashMap<String, serde_json::Value>,
    /// Style overrides
    #[serde(default)]
    pub styles: HashMap<String, serde_json::Value>,
}

/// Starter content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarterContent {
    /// Starter posts
    #[serde(default)]
    pub posts: Vec<StarterPost>,
    /// Starter pages
    #[serde(default)]
    pub pages: Vec<StarterPage>,
    /// Starter menus
    #[serde(default)]
    pub menus: HashMap<String, Vec<StarterMenuItem>>,
    /// Starter widgets
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
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub excerpt: Option<String>,
    #[serde(default)]
    pub thumbnail: Option<String>,
}

/// Starter page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarterPage {
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default)]
    pub is_front_page: bool,
    #[serde(default)]
    pub is_posts_page: bool,
}

/// Starter menu item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarterMenuItem {
    pub title: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub page: Option<String>,
    #[serde(default)]
    pub children: Vec<StarterMenuItem>,
}

/// Starter widget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarterWidget {
    pub widget_type: String,
    #[serde(default)]
    pub settings: HashMap<String, serde_json::Value>,
}

/// Theme support declarations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThemeSupports {
    /// Block editor (Gutenberg) support
    #[serde(default)]
    pub block_editor: bool,

    /// Full-site editing support
    #[serde(default)]
    pub full_site_editing: bool,

    /// Responsive embeds
    #[serde(default)]
    pub responsive_embeds: bool,

    /// Editor styles
    #[serde(default)]
    pub editor_styles: bool,

    /// Dark editor style
    #[serde(default)]
    pub dark_editor_style: bool,

    /// Align wide
    #[serde(default)]
    pub align_wide: bool,

    /// HTML5 features
    #[serde(default)]
    pub html5: Vec<String>,

    /// Post thumbnails
    #[serde(default)]
    pub post_thumbnails: bool,

    /// Custom logo
    #[serde(default)]
    pub custom_logo: Option<CustomLogoConfig>,

    /// Custom header
    #[serde(default)]
    pub custom_header: Option<CustomHeaderConfig>,

    /// Custom background
    #[serde(default)]
    pub custom_background: Option<CustomBackgroundConfig>,

    /// Automatic feed links
    #[serde(default)]
    pub automatic_feed_links: bool,

    /// Title tag
    #[serde(default = "default_true")]
    pub title_tag: bool,

    /// Post formats
    #[serde(default)]
    pub post_formats: Vec<String>,

    /// AMP support
    #[serde(default)]
    pub amp: bool,

    /// Dark mode
    #[serde(default)]
    pub dark_mode: bool,
}

/// Custom logo configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomLogoConfig {
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub flex_width: bool,
    #[serde(default)]
    pub flex_height: bool,
    #[serde(default)]
    pub header_text: Vec<String>,
    #[serde(default)]
    pub unlink_homepage_logo: bool,
}

/// Custom header configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomHeaderConfig {
    #[serde(default)]
    pub default_image: Option<String>,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub flex_width: bool,
    #[serde(default)]
    pub flex_height: bool,
    #[serde(default)]
    pub header_text: bool,
    #[serde(default)]
    pub default_text_color: Option<String>,
    #[serde(default)]
    pub video: bool,
}

/// Custom background configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomBackgroundConfig {
    #[serde(default)]
    pub default_color: Option<String>,
    #[serde(default)]
    pub default_image: Option<String>,
    #[serde(default)]
    pub default_repeat: Option<String>,
    #[serde(default)]
    pub default_position_x: Option<String>,
    #[serde(default)]
    pub default_position_y: Option<String>,
    #[serde(default)]
    pub default_size: Option<String>,
    #[serde(default)]
    pub default_attachment: Option<String>,
}

impl ThemeManifest {
    /// Parse manifest from TOML content
    pub fn from_toml(content: &str) -> Result<Self, ManifestError> {
        toml::from_str(content).map_err(|e| ManifestError::ParseError(e.to_string()))
    }

    /// Parse manifest from JSON content (supports simple/flat JSON format)
    pub fn from_json(content: &str, theme_id: &str) -> Result<Self, ManifestError> {
        // First try parsing as full ThemeManifest
        if let Ok(manifest) = serde_json::from_str::<ThemeManifest>(content) {
            return Ok(manifest);
        }

        // Otherwise parse as simple JSON and convert
        let json: serde_json::Value = serde_json::from_str(content)
            .map_err(|e| ManifestError::ParseError(e.to_string()))?;

        // Build ThemeMeta from flat JSON
        let theme = ThemeMeta {
            id: json.get("id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| theme_id.to_string()),
            name: json.get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| theme_id.to_string()),
            version: json.get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "1.0.0".to_string()),
            description: json.get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default(),
            author: json.get("author")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
            author_url: json.get("author_url")
                .or_else(|| json.get("authorUrl"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            homepage: json.get("homepage")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            license: json.get("license")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "MIT".to_string()),
            license_url: json.get("license_url")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            tags: json.get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default(),
            screenshot: json.get("screenshot")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            requires_rustpress: json.get("requires_rustpress")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            tested_up_to: json.get("tested_up_to")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            text_domain: json.get("text_domain")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            domain_path: json.get("domain_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            theme_type: ThemeType::Classic,
        };

        // Parse colors from simple map format: {"primary": "#6366F1", ...}
        let colors = if let Some(colors_obj) = json.get("colors").and_then(|v| v.as_object()) {
            let palette: Vec<ColorDefinition> = colors_obj.iter()
                .filter_map(|(key, value)| {
                    value.as_str().map(|color| ColorDefinition {
                        slug: key.clone(),
                        name: key.replace('_', " ").to_string(),
                        color: color.to_string(),
                    })
                })
                .collect();
            ColorSection {
                palette,
                ..Default::default()
            }
        } else {
            ColorSection::default()
        };

        // Parse fonts from simple map format: {"heading": "Space Grotesk", ...}
        let typography = if let Some(fonts_obj) = json.get("fonts").and_then(|v| v.as_object()) {
            let font_families: Vec<FontFamily> = fonts_obj.iter()
                .filter_map(|(key, value)| {
                    value.as_str().map(|font| FontFamily {
                        slug: key.clone(),
                        name: font.to_string(),
                        font_family: font.to_string(),
                        font_face: vec![],
                    })
                })
                .collect();
            TypographySection {
                font_families,
                ..Default::default()
            }
        } else {
            TypographySection::default()
        };

        // Parse features from map format: {"darkMode": true, ...}
        let features: HashMap<String, bool> = json.get("features")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(key, value)| {
                        value.as_bool().map(|b| (key.clone(), b))
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Parse layout settings
        let layout = if let Some(layout_obj) = json.get("layout").and_then(|v| v.as_object()) {
            LayoutSection {
                content_width: layout_obj.get("contentWidth")
                    .or_else(|| layout_obj.get("content_width"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                wide_width: layout_obj.get("wideWidth")
                    .or_else(|| layout_obj.get("wide_width"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                ..Default::default()
            }
        } else {
            LayoutSection::default()
        };

        // Parse menu locations
        let menu_locations = if let Some(menus) = json.get("menuLocations").or_else(|| json.get("menu_locations")) {
            if let Some(arr) = menus.as_array() {
                arr.iter().filter_map(|v| {
                    let id = v.get("id").and_then(|v| v.as_str())?;
                    let name = v.get("name").and_then(|v| v.as_str()).unwrap_or(id);
                    Some(MenuLocation {
                        id: id.to_string(),
                        name: name.to_string(),
                        description: v.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    })
                }).collect()
            } else if let Some(obj) = menus.as_object() {
                obj.iter().map(|(id, name)| MenuLocation {
                    id: id.clone(),
                    name: name.as_str().unwrap_or(id).to_string(),
                    description: None,
                }).collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        // Parse widget areas
        let widget_areas = if let Some(widgets) = json.get("widgetAreas").or_else(|| json.get("widget_areas")) {
            if let Some(arr) = widgets.as_array() {
                arr.iter().filter_map(|v| {
                    let id = v.get("id").and_then(|v| v.as_str())?;
                    let name = v.get("name").and_then(|v| v.as_str()).unwrap_or(id);
                    Some(WidgetArea {
                        id: id.to_string(),
                        name: name.to_string(),
                        description: v.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        before_widget: None,
                        after_widget: None,
                        before_title: None,
                        after_title: None,
                    })
                }).collect()
            } else if let Some(obj) = widgets.as_object() {
                obj.iter().map(|(id, name)| WidgetArea {
                    id: id.clone(),
                    name: name.as_str().unwrap_or(id).to_string(),
                    description: None,
                    before_widget: None,
                    after_widget: None,
                    before_title: None,
                    after_title: None,
                }).collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        let layout_with_locations = LayoutSection {
            menu_locations,
            widget_areas,
            ..layout
        };

        // Build supports from features
        let supports = ThemeSupports {
            dark_mode: features.get("darkMode").or_else(|| features.get("dark_mode")).copied().unwrap_or(false),
            block_editor: features.get("blockEditor").or_else(|| features.get("block_editor")).copied().unwrap_or(true),
            post_thumbnails: features.get("postThumbnails").or_else(|| features.get("post_thumbnails")).copied().unwrap_or(true),
            ..Default::default()
        };

        Ok(ThemeManifest {
            theme,
            parent: None,
            templates: TemplateSection::default(),
            assets: AssetSection::default(),
            settings: SettingsSection::default(),
            colors,
            typography,
            layout: layout_with_locations,
            patterns: vec![],
            blocks: BlockThemeSection::default(),
            variations: vec![],
            starter_content: None,
            supports,
            features,
        })
    }

    /// Serialize to TOML
    pub fn to_toml(&self) -> Result<String, ManifestError> {
        toml::to_string_pretty(self).map_err(|e| ManifestError::SerializeError(e.to_string()))
    }

    /// Load from file path
    pub fn load(path: &std::path::Path) -> Result<Self, ManifestError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ManifestError::IoError(e.to_string()))?;
        Self::from_toml(&content)
    }

    /// Check if this is a child theme
    pub fn is_child_theme(&self) -> bool {
        self.parent.is_some()
    }

    /// Check if this is a block theme
    pub fn is_block_theme(&self) -> bool {
        matches!(self.theme.theme_type, ThemeType::Block | ThemeType::Hybrid)
    }

    /// Get all color slugs
    pub fn color_slugs(&self) -> Vec<&str> {
        self.colors.palette.iter().map(|c| c.slug.as_str()).collect()
    }

    /// Get all font size slugs
    pub fn font_size_slugs(&self) -> Vec<&str> {
        self.typography.font_sizes.iter().map(|f| f.slug.as_str()).collect()
    }
}

/// Manifest error
#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Serialize error: {0}")]
    SerializeError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_manifest() {
        let toml = r##"
[theme]
id = "my-theme"
name = "My Theme"
version = "1.0.0"
description = "A test theme"
author = "Test Author"

[[colors.palette]]
slug = "primary"
name = "Primary"
color = "#0073aa"

[[typography.font_sizes]]
slug = "small"
name = "Small"
size = "13px"
"##;

        let manifest = ThemeManifest::from_toml(toml).unwrap();
        assert_eq!(manifest.theme.id, "my-theme");
        assert_eq!(manifest.colors.palette.len(), 1);
        assert_eq!(manifest.typography.font_sizes.len(), 1);
    }

    #[test]
    fn test_child_theme_detection() {
        let toml = r##"
[theme]
id = "child-theme"
name = "Child Theme"
version = "1.0.0"
description = "A child theme"
author = "Test Author"

[parent]
id = "parent-theme"
min_version = "2.0.0"
"##;

        let manifest = ThemeManifest::from_toml(toml).unwrap();
        assert!(manifest.is_child_theme());
    }
}
