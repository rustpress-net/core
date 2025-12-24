//! Settings handlers.

use serde::{Deserialize, Serialize};

/// Update setting request
#[derive(Debug, Deserialize)]
pub struct UpdateSettingRequest {
    pub value: serde_json::Value,
}

/// Setting response
#[derive(Debug, Serialize)]
pub struct SettingResponse {
    pub key: String,
    pub value: serde_json::Value,
    pub group: String,
    pub description: Option<String>,
}

/// Settings group response
#[derive(Debug, Serialize)]
pub struct SettingsGroupResponse {
    pub group: String,
    pub settings: Vec<SettingResponse>,
}

/// All settings response
#[derive(Debug, Serialize)]
pub struct AllSettingsResponse {
    pub groups: Vec<SettingsGroupResponse>,
}

/// Predefined setting keys
pub mod keys {
    // General
    pub const SITE_TITLE: &str = "site_title";
    pub const SITE_TAGLINE: &str = "site_tagline";
    pub const SITE_URL: &str = "site_url";
    pub const ADMIN_EMAIL: &str = "admin_email";
    pub const TIMEZONE: &str = "timezone";
    pub const DATE_FORMAT: &str = "date_format";
    pub const TIME_FORMAT: &str = "time_format";
    pub const LANGUAGE: &str = "language";

    // Reading
    pub const POSTS_PER_PAGE: &str = "posts_per_page";
    pub const SHOW_ON_FRONT: &str = "show_on_front";
    pub const PAGE_ON_FRONT: &str = "page_on_front";
    pub const PAGE_FOR_POSTS: &str = "page_for_posts";

    // Discussion
    pub const COMMENTS_ENABLED: &str = "comments_enabled";
    pub const COMMENT_MODERATION: &str = "comment_moderation";
    pub const COMMENT_REGISTRATION: &str = "comment_registration";
    pub const COMMENTS_NOTIFY: &str = "comments_notify";
    pub const COMMENTS_NESTED: &str = "comments_nested";
    pub const COMMENTS_NESTED_DEPTH: &str = "comments_nested_depth";

    // Media
    pub const THUMBNAIL_SIZE_W: &str = "thumbnail_size_w";
    pub const THUMBNAIL_SIZE_H: &str = "thumbnail_size_h";
    pub const MEDIUM_SIZE_W: &str = "medium_size_w";
    pub const MEDIUM_SIZE_H: &str = "medium_size_h";
    pub const LARGE_SIZE_W: &str = "large_size_w";
    pub const LARGE_SIZE_H: &str = "large_size_h";
    pub const UPLOADS_PATH: &str = "uploads_path";

    // Permalinks
    pub const PERMALINK_STRUCTURE: &str = "permalink_structure";
    pub const CATEGORY_BASE: &str = "category_base";
    pub const TAG_BASE: &str = "tag_base";
}
