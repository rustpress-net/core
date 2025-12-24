//! Common types used throughout RustPress.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Post status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PostStatus {
    Draft,
    Pending,
    Published,
    Private,
    Trash,
    Scheduled,
}

impl Default for PostStatus {
    fn default() -> Self {
        Self::Draft
    }
}

impl PostStatus {
    pub fn is_public(&self) -> bool {
        matches!(self, Self::Published)
    }

    pub fn is_visible(&self) -> bool {
        matches!(self, Self::Published | Self::Private)
    }
}

/// Comment status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommentStatus {
    Pending,
    Approved,
    Spam,
    Trash,
}

impl Default for CommentStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// User status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
    Pending,
}

impl Default for UserStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Media type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Image,
    Video,
    Audio,
    Document,
    Archive,
    Other,
}

impl MediaType {
    pub fn from_mime(mime: &str) -> Self {
        if mime.starts_with("image/") {
            Self::Image
        } else if mime.starts_with("video/") {
            Self::Video
        } else if mime.starts_with("audio/") {
            Self::Audio
        } else if mime.starts_with("application/pdf")
            || mime.starts_with("application/msword")
            || mime.starts_with("application/vnd")
            || mime.starts_with("text/")
        {
            Self::Document
        } else if mime.contains("zip") || mime.contains("tar") || mime.contains("gzip") {
            Self::Archive
        } else {
            Self::Other
        }
    }
}

/// Timestamps mixin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timestamps {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for Timestamps {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            updated_at: now,
        }
    }
}

impl Timestamps {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

/// Soft delete mixin
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SoftDelete {
    pub deleted_at: Option<DateTime<Utc>>,
}

impl SoftDelete {
    pub fn new() -> Self {
        Self { deleted_at: None }
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    pub fn delete(&mut self) {
        self.deleted_at = Some(Utc::now());
    }

    pub fn restore(&mut self) {
        self.deleted_at = None;
    }
}

/// Slug for URL-friendly identifiers
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Slug(String);

impl Slug {
    pub fn new(value: impl Into<String>) -> Self {
        Self(Self::slugify(&value.into()))
    }

    pub fn from_raw(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    fn slugify(s: &str) -> String {
        s.to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c
                } else if c.is_whitespace() || c == '-' || c == '_' {
                    '-'
                } else {
                    '\0'
                }
            })
            .filter(|&c| c != '\0')
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Slug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Slug {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Pagination info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
    pub total_pages: u32,
}

impl Pagination {
    pub fn new(page: u32, per_page: u32, total: u64) -> Self {
        let total_pages = (total as f64 / per_page as f64).ceil() as u32;
        Self {
            page,
            per_page,
            total,
            total_pages,
        }
    }

    pub fn has_next(&self) -> bool {
        self.page < self.total_pages
    }

    pub fn has_prev(&self) -> bool {
        self.page > 1
    }

    pub fn next_page(&self) -> Option<u32> {
        if self.has_next() {
            Some(self.page + 1)
        } else {
            None
        }
    }

    pub fn prev_page(&self) -> Option<u32> {
        if self.has_prev() {
            Some(self.page - 1)
        } else {
            None
        }
    }
}

/// Content format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ContentFormat {
    #[default]
    Html,
    Markdown,
    PlainText,
    Blocks, // Gutenberg-style blocks
}

/// Visibility setting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    #[default]
    Public,
    Private,
    Password,
    Internal,
}

/// Permission
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    pub resource: String,
    pub action: String,
}

impl Permission {
    pub fn new(resource: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            action: action.into(),
        }
    }

    pub fn read(resource: impl Into<String>) -> Self {
        Self::new(resource, "read")
    }

    pub fn write(resource: impl Into<String>) -> Self {
        Self::new(resource, "write")
    }

    pub fn delete(resource: impl Into<String>) -> Self {
        Self::new(resource, "delete")
    }

    pub fn admin(resource: impl Into<String>) -> Self {
        Self::new(resource, "admin")
    }

    pub fn matches(&self, resource: &str, action: &str) -> bool {
        (self.resource == "*" || self.resource == resource)
            && (self.action == "*" || self.action == action)
    }
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.resource, self.action)
    }
}

/// Locale/language code
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Locale(String);

impl Locale {
    pub fn new(code: impl Into<String>) -> Self {
        Self(code.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn language(&self) -> &str {
        self.0.split(&['-', '_'][..]).next().unwrap_or(&self.0)
    }

    pub fn region(&self) -> Option<&str> {
        self.0.split(&['-', '_'][..]).nth(1)
    }
}

impl Default for Locale {
    fn default() -> Self {
        Self("en".to_string())
    }
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slug() {
        assert_eq!(Slug::new("Hello World").as_str(), "hello-world");
        assert_eq!(Slug::new("Test  Post!").as_str(), "test-post");
        assert_eq!(Slug::new("My_Great_Post").as_str(), "my-great-post");
        assert_eq!(Slug::new("--test--").as_str(), "test");
    }

    #[test]
    fn test_pagination() {
        let p = Pagination::new(2, 10, 45);
        assert_eq!(p.total_pages, 5);
        assert!(p.has_next());
        assert!(p.has_prev());
        assert_eq!(p.next_page(), Some(3));
        assert_eq!(p.prev_page(), Some(1));
    }

    #[test]
    fn test_permission() {
        let perm = Permission::new("posts", "write");
        assert!(perm.matches("posts", "write"));
        assert!(!perm.matches("posts", "delete"));

        let admin = Permission::new("*", "*");
        assert!(admin.matches("posts", "delete"));
        assert!(admin.matches("users", "admin"));
    }

    #[test]
    fn test_locale() {
        let locale = Locale::new("en-US");
        assert_eq!(locale.language(), "en");
        assert_eq!(locale.region(), Some("US"));

        let simple = Locale::new("fr");
        assert_eq!(simple.language(), "fr");
        assert_eq!(simple.region(), None);
    }

    #[test]
    fn test_media_type() {
        assert_eq!(MediaType::from_mime("image/png"), MediaType::Image);
        assert_eq!(MediaType::from_mime("video/mp4"), MediaType::Video);
        assert_eq!(MediaType::from_mime("application/pdf"), MediaType::Document);
        assert_eq!(MediaType::from_mime("application/zip"), MediaType::Archive);
    }

    #[test]
    fn test_timestamps() {
        let mut ts = Timestamps::new();
        let original = ts.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(10));
        ts.touch();
        assert!(ts.updated_at > original);
    }

    #[test]
    fn test_soft_delete() {
        let mut sd = SoftDelete::new();
        assert!(!sd.is_deleted());

        sd.delete();
        assert!(sd.is_deleted());

        sd.restore();
        assert!(!sd.is_deleted());
    }
}
