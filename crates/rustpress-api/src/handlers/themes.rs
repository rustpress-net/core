//! Theme handlers for theme activation, customization, and management.
//!
//! Themes are presentation-only - all content (posts, pages, menus, widgets)
//! remains intact when switching themes, just like WordPress.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============ Request Types ============

/// Request to activate a theme
#[derive(Debug, Deserialize)]
pub struct ActivateThemeRequest {
    /// Whether to copy menu/widget assignments from current theme
    pub copy_assignments: Option<bool>,
}

/// Request to update theme settings/customizations
#[derive(Debug, Deserialize)]
pub struct UpdateThemeSettingsRequest {
    /// Theme customization settings (colors, fonts, etc.)
    pub settings: serde_json::Value,
}

/// Request to update a specific theme option
#[derive(Debug, Deserialize)]
pub struct UpdateThemeOptionRequest {
    pub value: serde_json::Value,
    pub option_type: Option<String>,
}

/// Request to assign a menu to a theme location
#[derive(Debug, Deserialize)]
pub struct AssignMenuRequest {
    pub location_slug: String,
    pub menu_id: Uuid,
}

/// Request to assign a widget to a theme area
#[derive(Debug, Deserialize)]
pub struct AssignWidgetRequest {
    pub area_slug: String,
    pub widget_id: Uuid,
    pub position: Option<i32>,
}

/// Request to create a theme preview session
#[derive(Debug, Deserialize)]
pub struct CreatePreviewRequest {
    /// Duration in minutes (default: 30, max: 120)
    pub duration_minutes: Option<i32>,
    /// Optional preview settings to apply
    pub settings: Option<serde_json::Value>,
}

/// Theme list query parameters
#[derive(Debug, Deserialize)]
pub struct ThemeListQuery {
    /// Filter by installed status
    pub installed: Option<bool>,
    /// Filter by active status
    pub active: Option<bool>,
    /// Search by name or description
    pub search: Option<String>,
    /// Filter by tag
    pub tag: Option<String>,
}

// ============ Response Types ============

/// Theme response with full details
#[derive(Debug, Serialize)]
pub struct ThemeResponse {
    pub id: Uuid,
    pub theme_id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub author_url: Option<String>,
    pub license: Option<String>,
    pub is_active: bool,
    pub is_installed: bool,
    pub parent_theme_id: Option<String>,
    pub screenshot_url: Option<String>,
    pub homepage_url: Option<String>,
    pub tags: Vec<String>,
    pub supports: ThemeSupports,
    pub menu_locations: serde_json::Value,
    pub widget_areas: serde_json::Value,
    pub template_count: i32,
    pub activated_at: Option<DateTime<Utc>>,
    pub installed_at: Option<DateTime<Utc>>,
}

/// Theme capabilities/features
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ThemeSupports {
    #[serde(default)]
    pub post_formats: Vec<String>,
    #[serde(default)]
    pub post_thumbnails: bool,
    #[serde(default)]
    pub custom_logo: bool,
    #[serde(default)]
    pub custom_header: bool,
    #[serde(default)]
    pub custom_background: bool,
    #[serde(default)]
    pub menus: bool,
    #[serde(default)]
    pub widgets: bool,
    #[serde(default)]
    pub editor_styles: bool,
    #[serde(default)]
    pub dark_mode: bool,
    #[serde(default)]
    pub block_templates: bool,
}

/// Theme list response
#[derive(Debug, Serialize)]
pub struct ThemeListResponse {
    pub themes: Vec<ThemeResponse>,
    pub total: i64,
    pub active_theme: Option<String>,
}

/// Theme settings response
#[derive(Debug, Serialize)]
pub struct ThemeSettingsResponse {
    pub theme_id: String,
    pub customizer_schema: serde_json::Value,
    pub settings: serde_json::Value,
    pub options: Vec<ThemeOptionResponse>,
}

/// Individual theme option
#[derive(Debug, Serialize)]
pub struct ThemeOptionResponse {
    pub name: String,
    pub value: serde_json::Value,
    pub option_type: Option<String>,
}

/// Menu assignment response
#[derive(Debug, Serialize)]
pub struct MenuAssignmentResponse {
    pub location_slug: String,
    pub location_name: String,
    pub menu_id: Option<Uuid>,
    pub menu_name: Option<String>,
}

/// Widget area with assigned widgets
#[derive(Debug, Serialize)]
pub struct WidgetAreaResponse {
    pub area_slug: String,
    pub area_name: String,
    pub widgets: Vec<AssignedWidgetResponse>,
}

/// Assigned widget info
#[derive(Debug, Serialize)]
pub struct AssignedWidgetResponse {
    pub widget_id: Uuid,
    pub widget_type: String,
    pub title: Option<String>,
    pub position: i32,
    pub is_active: bool,
}

/// Theme preview session response
#[derive(Debug, Serialize)]
pub struct ThemePreviewResponse {
    pub preview_token: String,
    pub preview_url: String,
    pub theme_id: String,
    pub expires_at: DateTime<Utc>,
}

/// Active theme context for rendering
/// This is passed to templates when rendering the frontend
#[derive(Debug, Serialize)]
pub struct ThemeContext {
    pub theme: ThemeResponse,
    pub settings: serde_json::Value,
    pub menus: serde_json::Value,
    pub widget_areas: serde_json::Value,
}

// ============ Template Context Types ============
// These structures are passed to Tera templates for rendering

/// Site context available in all templates
#[derive(Debug, Serialize)]
pub struct SiteContext {
    pub name: String,
    pub tagline: Option<String>,
    pub url: String,
    pub language: String,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub favicon_url: Option<String>,
    // Social links
    pub social: SocialLinks,
    // Contact info
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
}

/// Social media links
#[derive(Debug, Serialize, Default)]
pub struct SocialLinks {
    pub twitter: Option<String>,
    pub facebook: Option<String>,
    pub linkedin: Option<String>,
    pub instagram: Option<String>,
    pub youtube: Option<String>,
    pub github: Option<String>,
}

/// Menu context for navigation
#[derive(Debug, Serialize)]
pub struct MenuContext {
    pub name: String,
    pub items: Vec<MenuItemContext>,
}

/// Menu item for navigation
#[derive(Debug, Serialize)]
pub struct MenuItemContext {
    pub id: Uuid,
    pub title: String,
    pub url: String,
    pub target: Option<String>,
    pub css_classes: Option<String>,
    pub icon: Option<String>,
    pub is_current: bool,
    pub children: Vec<MenuItemContext>,
}

/// Widget context for sidebars
#[derive(Debug, Serialize)]
pub struct WidgetContext {
    pub id: Uuid,
    pub widget_type: String,
    pub title: Option<String>,
    pub content: serde_json::Value,
}

/// Full page context for rendering
#[derive(Debug, Serialize)]
pub struct PageRenderContext {
    // Site-wide context
    pub site: SiteContext,
    // Current theme info
    pub theme: ThemeRenderInfo,
    // Navigation menus by location
    pub menus: serde_json::Value,
    // Widget areas with widgets
    pub widgets: serde_json::Value,
    // Current page/post content
    pub page: Option<serde_json::Value>,
    pub post: Option<serde_json::Value>,
    pub posts: Option<Vec<serde_json::Value>>,
    // Pagination
    pub pagination: Option<PaginationContext>,
    // User context (if logged in)
    pub user: Option<serde_json::Value>,
    // Additional template variables
    pub extra: serde_json::Value,
}

/// Theme info for templates
#[derive(Debug, Serialize)]
pub struct ThemeRenderInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub supports: ThemeSupports,
}

/// Pagination context
#[derive(Debug, Serialize)]
pub struct PaginationContext {
    pub current_page: u64,
    pub total_pages: u64,
    pub total_items: u64,
    pub per_page: u64,
    pub has_previous: bool,
    pub has_next: bool,
    pub previous_url: Option<String>,
    pub next_url: Option<String>,
}

// ============ Theme Scanner Types ============

/// Theme manifest from theme.json file
#[derive(Debug, Deserialize, Serialize)]
pub struct ThemeManifest {
    pub name: String,
    #[serde(default)]
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub author_url: Option<String>,
    pub license: Option<String>,
    pub homepage: Option<String>,
    pub screenshot: Option<String>,
    pub parent: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub supports: ThemeSupports,
    #[serde(default)]
    pub menu_locations: serde_json::Value,
    #[serde(default)]
    pub widget_areas: serde_json::Value,
    #[serde(default)]
    pub customizer: serde_json::Value,
    #[serde(default)]
    pub colors: serde_json::Value,
    #[serde(default)]
    pub fonts: serde_json::Value,
    #[serde(default)]
    pub pages: Vec<String>,
}

impl Default for ThemeManifest {
    fn default() -> Self {
        Self {
            name: "Unknown Theme".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            author_url: None,
            license: Some("MIT".to_string()),
            homepage: None,
            screenshot: None,
            parent: None,
            tags: vec![],
            supports: ThemeSupports::default(),
            menu_locations: serde_json::json!({}),
            widget_areas: serde_json::json!({}),
            customizer: serde_json::json!({}),
            colors: serde_json::json!({}),
            fonts: serde_json::json!({}),
            pages: vec![],
        }
    }
}
