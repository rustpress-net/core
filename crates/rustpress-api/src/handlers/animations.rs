//! Animation Library handlers.
//!
//! Handlers for animation operations including listing animations,
//! managing favorites, custom animations, and presets.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Animation category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnimationCategory {
    Entrance,
    Exit,
    Emphasis,
    Scroll,
    Rotation,
    Scale,
    Motion,
    Custom,
}

impl std::fmt::Display for AnimationCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Entrance => write!(f, "entrance"),
            Self::Exit => write!(f, "exit"),
            Self::Emphasis => write!(f, "emphasis"),
            Self::Scroll => write!(f, "scroll"),
            Self::Rotation => write!(f, "rotation"),
            Self::Scale => write!(f, "scale"),
            Self::Motion => write!(f, "motion"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

/// Animation definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationDefinition {
    pub id: String,
    pub name: String,
    pub category: String,
    pub duration: i32,
    pub preview_description: Option<String>,
    pub css_keyframes: String,
    pub css_class: String,
    pub is_pro: bool,
    pub sort_order: i32,
}

/// Animation settings for customization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationSettings {
    #[serde(default = "default_duration")]
    pub duration: i32,
    #[serde(default)]
    pub delay: i32,
    #[serde(default = "default_easing")]
    pub easing: String,
    #[serde(default = "default_repeat")]
    pub repeat: i32,
    #[serde(default)]
    pub repeat_delay: i32,
    #[serde(default = "default_direction")]
    pub direction: String,
    #[serde(default = "default_fill_mode")]
    pub fill_mode: String,
    #[serde(default = "default_trigger")]
    pub trigger: String,
    #[serde(default)]
    pub scroll_offset: i32,
}

fn default_duration() -> i32 { 500 }
fn default_easing() -> String { "ease".to_string() }
fn default_repeat() -> i32 { 1 }
fn default_direction() -> String { "normal".to_string() }
fn default_fill_mode() -> String { "forwards".to_string() }
fn default_trigger() -> String { "load".to_string() }

impl Default for AnimationSettings {
    fn default() -> Self {
        Self {
            duration: 500,
            delay: 0,
            easing: "ease".to_string(),
            repeat: 1,
            repeat_delay: 0,
            direction: "normal".to_string(),
            fill_mode: "forwards".to_string(),
            trigger: "load".to_string(),
            scroll_offset: 100,
        }
    }
}

/// User animation preferences response
#[derive(Debug, Clone, Serialize)]
pub struct UserAnimationPreferencesResponse {
    pub user_id: Uuid,
    pub favorite_animations: Vec<String>,
    pub recent_animations: Vec<String>,
    pub default_settings: AnimationSettings,
    pub updated_at: DateTime<Utc>,
}

/// Custom animation response
#[derive(Debug, Clone, Serialize)]
pub struct CustomAnimationResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub category: String,
    pub duration: i32,
    pub css_keyframes: String,
    pub css_class: String,
    pub settings: AnimationSettings,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create custom animation request
#[derive(Debug, Deserialize)]
pub struct CreateCustomAnimationRequest {
    pub name: String,
    pub category: Option<String>,
    pub duration: Option<i32>,
    pub css_keyframes: String,
    pub css_class: String,
    pub settings: Option<AnimationSettings>,
    pub is_public: Option<bool>,
}

/// Update custom animation request
#[derive(Debug, Deserialize)]
pub struct UpdateCustomAnimationRequest {
    pub name: Option<String>,
    pub category: Option<String>,
    pub duration: Option<i32>,
    pub css_keyframes: Option<String>,
    pub css_class: Option<String>,
    pub settings: Option<AnimationSettings>,
    pub is_public: Option<bool>,
}

/// Animation preset - combination of animations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationPreset {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<AnimationStep>,
    pub is_system: bool,
    pub user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Animation step within a preset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationStep {
    pub animation_id: String,
    pub delay: i32,
    pub target: Option<String>,
    pub settings: Option<AnimationSettings>,
}

/// Create preset request
#[derive(Debug, Deserialize)]
pub struct CreatePresetRequest {
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<AnimationStep>,
}

/// Update preset request
#[derive(Debug, Deserialize)]
pub struct UpdatePresetRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub steps: Option<Vec<AnimationStep>>,
}

/// Animation list query parameters
#[derive(Debug, Deserialize, Default)]
pub struct AnimationListQuery {
    pub category: Option<String>,
    pub search: Option<String>,
    pub include_custom: Option<bool>,
    pub include_pro: Option<bool>,
}

/// Custom animation list query
#[derive(Debug, Deserialize, Default)]
pub struct CustomAnimationListQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub category: Option<String>,
    pub search: Option<String>,
    pub is_public: Option<bool>,
}

/// Update recent animations request
#[derive(Debug, Deserialize)]
pub struct UpdateRecentAnimationsRequest {
    pub animation_id: String,
}

/// Toggle favorite animation request
#[derive(Debug, Deserialize)]
pub struct ToggleFavoriteAnimationRequest {
    pub animation_id: String,
}

/// Toggle favorite response
#[derive(Debug, Serialize)]
pub struct ToggleFavoriteAnimationResponse {
    pub animation_id: String,
    pub is_favorite: bool,
}

/// Track animation usage request
#[derive(Debug, Deserialize)]
pub struct TrackAnimationUsageRequest {
    pub animation_id: String,
    pub animation_type: Option<String>,
    pub post_id: Option<Uuid>,
    pub action: Option<String>,
    pub settings: Option<AnimationSettings>,
}

/// Animation analytics response
#[derive(Debug, Serialize)]
pub struct AnimationAnalyticsResponse {
    pub total_applications: i64,
    pub unique_animations_used: i64,
    pub most_used_animations: Vec<AnimationUsageEntry>,
    pub recent_activity: Vec<AnimationUsageEntry>,
    pub category_breakdown: Vec<CategoryUsage>,
}

/// Animation usage entry
#[derive(Debug, Clone, Serialize)]
pub struct AnimationUsageEntry {
    pub animation_id: String,
    pub animation_type: String,
    pub usage_count: i64,
    pub last_used: DateTime<Utc>,
}

/// Category usage stats
#[derive(Debug, Clone, Serialize)]
pub struct CategoryUsage {
    pub category: String,
    pub count: i64,
}

/// Paginated custom animation list
#[derive(Debug, Serialize)]
pub struct CustomAnimationListResponse {
    pub items: Vec<CustomAnimationResponse>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

/// Paginated preset list
#[derive(Debug, Serialize)]
pub struct PresetListResponse {
    pub items: Vec<AnimationPreset>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

/// Animation library response
#[derive(Debug, Serialize)]
pub struct AnimationLibraryResponse {
    pub animations: Vec<AnimationDefinition>,
    pub categories: Vec<AnimationCategoryInfo>,
    pub custom_animations: Vec<CustomAnimationResponse>,
    pub presets: Vec<AnimationPreset>,
    pub recent_animations: Vec<String>,
    pub favorite_animations: Vec<String>,
    pub default_settings: AnimationSettings,
}

/// Animation category info
#[derive(Debug, Clone, Serialize)]
pub struct AnimationCategoryInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub count: i32,
}

/// Update default settings request
#[derive(Debug, Deserialize)]
pub struct UpdateDefaultSettingsRequest {
    pub settings: AnimationSettings,
}

/// Generated animation output
#[derive(Debug, Serialize)]
pub struct AnimationOutput {
    pub inline_style: String,
    pub css_class: String,
    pub data_attributes: String,
    pub css_keyframes: String,
    pub js_trigger_code: Option<String>,
}

/// Generate animation output request
#[derive(Debug, Deserialize)]
pub struct GenerateOutputRequest {
    pub animation_id: String,
    pub settings: AnimationSettings,
    pub output_format: String,
    pub include_keyframes: Option<bool>,
}
