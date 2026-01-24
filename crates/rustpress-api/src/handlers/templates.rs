//! Template Library handlers.
//!
//! Handlers for template operations including listing templates,
//! managing user templates, ratings, and template usage tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Template category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateCategory {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
}

/// Template variable definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    #[serde(rename = "type")]
    pub variable_type: TemplateVariableType,
    #[serde(rename = "defaultValue")]
    pub default_value: String,
    pub placeholder: String,
}

/// Template variable types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemplateVariableType {
    Text,
    Image,
    Color,
    Link,
}

impl std::fmt::Display for TemplateVariableType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text => write!(f, "text"),
            Self::Image => write!(f, "image"),
            Self::Color => write!(f, "color"),
            Self::Link => write!(f, "link"),
        }
    }
}

/// Template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub category_id: Option<String>,
    pub thumbnail_url: Option<String>,
    pub html_content: String,
    pub css_content: Option<String>,
    pub variables: Vec<TemplateVariable>,
    pub is_pro: bool,
    pub is_system: bool,
    pub author_id: Option<Uuid>,
    pub downloads: i32,
    pub rating: f32,
    pub rating_count: i32,
    pub is_public: bool,
    pub tags: Vec<String>,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Template list item (for grid display)
#[derive(Debug, Clone, Serialize)]
pub struct TemplateListItem {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub category_id: Option<String>,
    pub thumbnail_url: Option<String>,
    pub is_pro: bool,
    pub is_system: bool,
    pub author_name: Option<String>,
    pub downloads: i32,
    pub rating: f32,
    pub rating_count: i32,
    pub tags: Vec<String>,
}

/// Template detail response (full content)
#[derive(Debug, Clone, Serialize)]
pub struct TemplateDetailResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub category: Option<TemplateCategory>,
    pub thumbnail_url: Option<String>,
    pub html_content: String,
    pub css_content: Option<String>,
    pub variables: Vec<TemplateVariable>,
    pub is_pro: bool,
    pub is_system: bool,
    pub author: Option<TemplateAuthor>,
    pub downloads: i32,
    pub rating: f32,
    pub rating_count: i32,
    pub is_public: bool,
    pub tags: Vec<String>,
    pub version: String,
    pub user_rating: Option<i32>,
    pub is_favorited: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Template author info
#[derive(Debug, Clone, Serialize)]
pub struct TemplateAuthor {
    pub id: Uuid,
    pub name: String,
    pub avatar_url: Option<String>,
}

/// Create template request
#[derive(Debug, Deserialize)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<String>,
    pub thumbnail_url: Option<String>,
    pub html_content: String,
    pub css_content: Option<String>,
    pub variables: Option<Vec<TemplateVariable>>,
    pub is_public: Option<bool>,
    pub tags: Option<Vec<String>>,
}

/// Update template request
#[derive(Debug, Deserialize)]
pub struct UpdateTemplateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category_id: Option<String>,
    pub thumbnail_url: Option<String>,
    pub html_content: Option<String>,
    pub css_content: Option<String>,
    pub variables: Option<Vec<TemplateVariable>>,
    pub is_public: Option<bool>,
    pub tags: Option<Vec<String>>,
}

/// Template list query parameters
#[derive(Debug, Deserialize, Default)]
pub struct TemplateListQuery {
    pub category: Option<String>,
    pub search: Option<String>,
    pub tags: Option<String>,
    pub is_system: Option<bool>,
    pub is_pro: Option<bool>,
    pub sort_by: Option<TemplateSortBy>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

/// Template sort options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemplateSortBy {
    Newest,
    Popular,
    Rating,
    Name,
    Downloads,
}

impl Default for TemplateSortBy {
    fn default() -> Self {
        Self::Popular
    }
}

/// Paginated template list response
#[derive(Debug, Serialize)]
pub struct TemplateListResponse {
    pub items: Vec<TemplateListItem>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

/// Rate template request
#[derive(Debug, Deserialize)]
pub struct RateTemplateRequest {
    pub rating: i32,
    pub review: Option<String>,
}

/// Template rating response
#[derive(Debug, Serialize)]
pub struct TemplateRatingResponse {
    pub template_id: Uuid,
    pub rating: i32,
    pub new_average: f32,
    pub total_ratings: i32,
}

/// Track template usage request
#[derive(Debug, Deserialize)]
pub struct TrackTemplateUsageRequest {
    pub template_id: Uuid,
    pub action: TemplateUsageAction,
    pub post_id: Option<Uuid>,
    pub variables_used: Option<serde_json::Value>,
}

/// Template usage action types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemplateUsageAction {
    Preview,
    Insert,
    Download,
}

impl std::fmt::Display for TemplateUsageAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Preview => write!(f, "preview"),
            Self::Insert => write!(f, "insert"),
            Self::Download => write!(f, "download"),
        }
    }
}

/// Toggle favorite request
#[derive(Debug, Deserialize)]
pub struct ToggleFavoriteRequest {
    pub template_id: Uuid,
}

/// Toggle favorite response
#[derive(Debug, Serialize)]
pub struct ToggleFavoriteResponse {
    pub template_id: Uuid,
    pub is_favorited: bool,
}

/// User template favorites response
#[derive(Debug, Serialize)]
pub struct UserFavoritesResponse {
    pub favorites: Vec<TemplateListItem>,
    pub total: u64,
}

/// Process template request (replace variables)
#[derive(Debug, Deserialize)]
pub struct ProcessTemplateRequest {
    pub template_id: Uuid,
    pub variables: serde_json::Value,
    pub include_css: Option<bool>,
}

/// Process template response
#[derive(Debug, Serialize)]
pub struct ProcessTemplateResponse {
    pub html: String,
    pub css: Option<String>,
    pub variables_replaced: Vec<String>,
}

/// Template analytics response
#[derive(Debug, Serialize)]
pub struct TemplateAnalyticsResponse {
    pub total_templates: i64,
    pub total_downloads: i64,
    pub total_inserts: i64,
    pub most_popular: Vec<TemplatePopularityEntry>,
    pub recent_usage: Vec<TemplateUsageEntry>,
    pub category_breakdown: Vec<CategoryUsageStats>,
}

/// Template popularity entry
#[derive(Debug, Clone, Serialize)]
pub struct TemplatePopularityEntry {
    pub template_id: Uuid,
    pub template_name: String,
    pub downloads: i64,
    pub rating: f32,
}

/// Template usage entry
#[derive(Debug, Clone, Serialize)]
pub struct TemplateUsageEntry {
    pub template_id: Uuid,
    pub template_name: String,
    pub action: String,
    pub used_at: DateTime<Utc>,
}

/// Category usage statistics
#[derive(Debug, Clone, Serialize)]
pub struct CategoryUsageStats {
    pub category_id: String,
    pub category_name: String,
    pub template_count: i64,
    pub total_downloads: i64,
}

/// Template library response (full data for modal)
#[derive(Debug, Serialize)]
pub struct TemplateLibraryResponse {
    pub templates: Vec<TemplateListItem>,
    pub categories: Vec<TemplateCategory>,
    pub user_templates: Vec<TemplateListItem>,
    pub favorites: Vec<Uuid>,
    pub recent_used: Vec<Uuid>,
}

/// Save as template request (from editor content)
#[derive(Debug, Deserialize)]
pub struct SaveAsTemplateRequest {
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<String>,
    pub html_content: String,
    pub css_content: Option<String>,
    pub extract_variables: Option<bool>,
    pub is_public: Option<bool>,
    pub tags: Option<Vec<String>>,
}

/// Extracted variable from content
#[derive(Debug, Clone, Serialize)]
pub struct ExtractedVariable {
    pub name: String,
    pub suggested_type: TemplateVariableType,
    pub current_value: String,
    pub occurrences: i32,
}

/// Extract variables response
#[derive(Debug, Serialize)]
pub struct ExtractVariablesResponse {
    pub variables: Vec<ExtractedVariable>,
    pub html_with_placeholders: String,
}

/// Duplicate template request
#[derive(Debug, Deserialize)]
pub struct DuplicateTemplateRequest {
    pub template_id: Uuid,
    pub new_name: Option<String>,
}

/// Template settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateLibrarySettings {
    #[serde(default = "default_view_mode")]
    pub view_mode: ViewMode,
    #[serde(default = "default_thumbnail_size")]
    pub thumbnail_size: ThumbnailSize,
    #[serde(default = "default_true")]
    pub show_ratings: bool,
    #[serde(default = "default_true")]
    pub show_downloads: bool,
    #[serde(default)]
    pub replace_content: bool,
    #[serde(default = "default_true")]
    pub process_variables: bool,
    #[serde(default = "default_max_templates")]
    pub max_user_templates: i32,
    #[serde(default)]
    pub allow_public_templates: bool,
    #[serde(default = "default_true")]
    pub show_pro_templates: bool,
}

fn default_view_mode() -> ViewMode { ViewMode::Grid }
fn default_thumbnail_size() -> ThumbnailSize { ThumbnailSize::Medium }
fn default_true() -> bool { true }
fn default_max_templates() -> i32 { 50 }

/// View mode options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ViewMode {
    Grid,
    List,
}

/// Thumbnail size options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThumbnailSize {
    Small,
    Medium,
    Large,
}

impl Default for TemplateLibrarySettings {
    fn default() -> Self {
        Self {
            view_mode: ViewMode::Grid,
            thumbnail_size: ThumbnailSize::Medium,
            show_ratings: true,
            show_downloads: true,
            replace_content: false,
            process_variables: true,
            max_user_templates: 50,
            allow_public_templates: false,
            show_pro_templates: true,
        }
    }
}

/// Update settings request
#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub settings: TemplateLibrarySettings,
}
