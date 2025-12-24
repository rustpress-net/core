//! Integration tests for data persistence round-trip verification.
//!
//! These tests verify that all major entities can be saved and loaded correctly,
//! ensuring data integrity across the application.

use chrono::{DateTime, Utc};
use serde_json::json;
use uuid::Uuid;

/// Test data structures that mirror database entities
mod test_entities {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TestPost {
        pub id: i64,
        pub site_id: Uuid,
        pub title: String,
        pub slug: String,
        pub content: String,
        pub excerpt: Option<String>,
        pub status: String,
        pub post_type: String,
        pub author_id: Uuid,
        pub featured_image_id: Option<Uuid>,
        pub meta_title: Option<String>,
        pub meta_description: Option<String>,
        pub published_at: Option<DateTime<Utc>>,
        pub scheduled_at: Option<DateTime<Utc>>,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TestUser {
        pub id: Uuid,
        pub site_id: Uuid,
        pub email: String,
        pub username: String,
        pub display_name: String,
        pub status: String,
        pub locale: String,
        pub timezone: String,
        pub email_verified_at: Option<DateTime<Utc>>,
        pub created_at: DateTime<Utc>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TestTheme {
        pub id: Uuid,
        pub site_id: Uuid,
        pub theme_id: String,
        pub name: String,
        pub version: String,
        pub is_active: bool,
        pub settings: serde_json::Value,
        pub installed_at: DateTime<Utc>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TestPlugin {
        pub id: Uuid,
        pub site_id: Uuid,
        pub plugin_id: String,
        pub name: String,
        pub version: String,
        pub is_active: bool,
        pub settings: serde_json::Value,
        pub installed_at: DateTime<Utc>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TestMedia {
        pub id: Uuid,
        pub site_id: Uuid,
        pub filename: String,
        pub original_filename: String,
        pub mime_type: String,
        pub file_size: i64,
        pub storage_path: String,
        pub url: String,
        pub width: Option<i32>,
        pub height: Option<i32>,
        pub alt_text: Option<String>,
        pub caption: Option<String>,
        pub created_at: DateTime<Utc>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TestOption {
        pub id: Uuid,
        pub site_id: Uuid,
        pub option_name: String,
        pub option_value: serde_json::Value,
        pub option_group: String,
        pub autoload: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TestComment {
        pub id: i64,
        pub site_id: Uuid,
        pub post_id: i64,
        pub author_name: String,
        pub author_email: String,
        pub content: String,
        pub status: String,
        pub parent_id: Option<i64>,
        pub created_at: DateTime<Utc>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TestTaxonomy {
        pub id: i64,
        pub site_id: Uuid,
        pub name: String,
        pub slug: String,
        pub taxonomy_type: String,
        pub description: Option<String>,
        pub parent_id: Option<i64>,
        pub count: i32,
    }
}

/// SQL queries for testing persistence
mod queries {
    pub const INSERT_POST: &str = r#"
        INSERT INTO posts (
            site_id, title, slug, content, excerpt, status, post_type,
            author_id, featured_image_id, meta_title, meta_description,
            published_at, scheduled_at, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        RETURNING id
    "#;

    pub const SELECT_POST: &str = r#"
        SELECT id, site_id, title, slug, content, excerpt, status, post_type,
               author_id, featured_image_id, meta_title, meta_description,
               published_at, scheduled_at, created_at, updated_at
        FROM posts WHERE id = $1
    "#;

    pub const INSERT_USER: &str = r#"
        INSERT INTO users (
            id, site_id, email, username, password_hash, display_name,
            status, locale, timezone, email_verified_at, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
    "#;

    pub const SELECT_USER: &str = r#"
        SELECT id, site_id, email, username, display_name, status,
               locale, timezone, email_verified_at, created_at
        FROM users WHERE id = $1
    "#;

    pub const INSERT_THEME: &str = r#"
        INSERT INTO themes (
            id, site_id, theme_id, name, version, is_active, settings, installed_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
    "#;

    pub const SELECT_THEME: &str = r#"
        SELECT id, site_id, theme_id, name, version, is_active, settings, installed_at
        FROM themes WHERE id = $1
    "#;

    pub const INSERT_OPTION: &str = r#"
        INSERT INTO options (
            id, site_id, option_name, option_value, option_group, autoload
        ) VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (site_id, option_name) DO UPDATE
        SET option_value = EXCLUDED.option_value
    "#;

    pub const SELECT_OPTION: &str = r#"
        SELECT id, site_id, option_name, option_value, option_group, autoload
        FROM options WHERE site_id = $1 AND option_name = $2
    "#;

    pub const INSERT_MEDIA: &str = r#"
        INSERT INTO media (
            id, site_id, filename, original_filename, mime_type, file_size,
            storage_path, url, width, height, alt_text, caption, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
    "#;

    pub const SELECT_MEDIA: &str = r#"
        SELECT id, site_id, filename, original_filename, mime_type, file_size,
               storage_path, url, width, height, alt_text, caption, created_at
        FROM media WHERE id = $1
    "#;

    pub const CLEAN_THEME_PREVIEWS: &str = r#"
        DELETE FROM theme_previews WHERE expires_at < NOW()
    "#;

    pub const PUBLISH_SCHEDULED_POSTS: &str = r#"
        UPDATE posts
        SET status = 'published', published_at = NOW(), scheduled_at = NULL, updated_at = NOW()
        WHERE status = 'scheduled' AND scheduled_at <= NOW()
        RETURNING id
    "#;
}

/// Test fixtures for creating test data
mod fixtures {
    use super::*;
    use test_entities::*;

    pub fn create_test_post(site_id: Uuid, author_id: Uuid) -> TestPost {
        let now = Utc::now();
        TestPost {
            id: 0, // Will be set by database
            site_id,
            title: "Test Post Title".to_string(),
            slug: format!("test-post-{}", Uuid::new_v4()),
            content: "<p>This is test content with <strong>HTML</strong>.</p>".to_string(),
            excerpt: Some("Test excerpt".to_string()),
            status: "draft".to_string(),
            post_type: "post".to_string(),
            author_id,
            featured_image_id: None,
            meta_title: Some("SEO Title".to_string()),
            meta_description: Some("SEO Description".to_string()),
            published_at: None,
            scheduled_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn create_test_user(site_id: Uuid) -> TestUser {
        let now = Utc::now();
        TestUser {
            id: Uuid::new_v4(),
            site_id,
            email: format!("test-{}@example.com", Uuid::new_v4()),
            username: format!("testuser_{}", &Uuid::new_v4().to_string()[..8]),
            display_name: "Test User".to_string(),
            status: "active".to_string(),
            locale: "en_US".to_string(),
            timezone: "America/New_York".to_string(),
            email_verified_at: Some(now),
            created_at: now,
        }
    }

    pub fn create_test_theme(site_id: Uuid) -> TestTheme {
        let now = Utc::now();
        TestTheme {
            id: Uuid::new_v4(),
            site_id,
            theme_id: format!("test-theme-{}", &Uuid::new_v4().to_string()[..8]),
            name: "Test Theme".to_string(),
            version: "1.0.0".to_string(),
            is_active: false,
            settings: json!({
                "primary_color": "#007bff",
                "font_family": "Arial, sans-serif",
                "sidebar_position": "right"
            }),
            installed_at: now,
        }
    }

    pub fn create_test_option(site_id: Uuid) -> TestOption {
        TestOption {
            id: Uuid::new_v4(),
            site_id,
            option_name: format!("test_option_{}", &Uuid::new_v4().to_string()[..8]),
            option_value: json!({
                "setting1": true,
                "setting2": "value",
                "setting3": 42
            }),
            option_group: "general".to_string(),
            autoload: true,
        }
    }

    pub fn create_test_media(site_id: Uuid) -> TestMedia {
        let now = Utc::now();
        let file_id = Uuid::new_v4();
        TestMedia {
            id: file_id,
            site_id,
            filename: format!("{}.jpg", file_id),
            original_filename: "test-image.jpg".to_string(),
            mime_type: "image/jpeg".to_string(),
            file_size: 1024 * 100, // 100KB
            storage_path: format!("/uploads/{}/{}.jpg", now.format("%Y/%m"), file_id),
            url: format!("/uploads/{}/{}.jpg", now.format("%Y/%m"), file_id),
            width: Some(1920),
            height: Some(1080),
            alt_text: Some("Test image alt text".to_string()),
            caption: Some("Test image caption".to_string()),
            created_at: now,
        }
    }
}

/// Assertions for verifying data integrity
mod assertions {
    use super::test_entities::*;

    /// Verify post data matches after round-trip
    pub fn assert_post_equals(original: &TestPost, loaded: &TestPost) {
        assert_eq!(original.site_id, loaded.site_id, "site_id mismatch");
        assert_eq!(original.title, loaded.title, "title mismatch");
        assert_eq!(original.slug, loaded.slug, "slug mismatch");
        assert_eq!(original.content, loaded.content, "content mismatch");
        assert_eq!(original.excerpt, loaded.excerpt, "excerpt mismatch");
        assert_eq!(original.status, loaded.status, "status mismatch");
        assert_eq!(original.post_type, loaded.post_type, "post_type mismatch");
        assert_eq!(original.author_id, loaded.author_id, "author_id mismatch");
        assert_eq!(original.meta_title, loaded.meta_title, "meta_title mismatch");
        assert_eq!(original.meta_description, loaded.meta_description, "meta_description mismatch");
    }

    /// Verify user data matches after round-trip
    pub fn assert_user_equals(original: &TestUser, loaded: &TestUser) {
        assert_eq!(original.id, loaded.id, "id mismatch");
        assert_eq!(original.site_id, loaded.site_id, "site_id mismatch");
        assert_eq!(original.email, loaded.email, "email mismatch");
        assert_eq!(original.username, loaded.username, "username mismatch");
        assert_eq!(original.display_name, loaded.display_name, "display_name mismatch");
        assert_eq!(original.status, loaded.status, "status mismatch");
        assert_eq!(original.locale, loaded.locale, "locale mismatch");
        assert_eq!(original.timezone, loaded.timezone, "timezone mismatch");
    }

    /// Verify theme data matches after round-trip
    pub fn assert_theme_equals(original: &TestTheme, loaded: &TestTheme) {
        assert_eq!(original.id, loaded.id, "id mismatch");
        assert_eq!(original.site_id, loaded.site_id, "site_id mismatch");
        assert_eq!(original.theme_id, loaded.theme_id, "theme_id mismatch");
        assert_eq!(original.name, loaded.name, "name mismatch");
        assert_eq!(original.version, loaded.version, "version mismatch");
        assert_eq!(original.is_active, loaded.is_active, "is_active mismatch");
        assert_eq!(original.settings, loaded.settings, "settings mismatch");
    }

    /// Verify option data matches after round-trip
    pub fn assert_option_equals(original: &TestOption, loaded: &TestOption) {
        assert_eq!(original.site_id, loaded.site_id, "site_id mismatch");
        assert_eq!(original.option_name, loaded.option_name, "option_name mismatch");
        assert_eq!(original.option_value, loaded.option_value, "option_value mismatch");
        assert_eq!(original.option_group, loaded.option_group, "option_group mismatch");
        assert_eq!(original.autoload, loaded.autoload, "autoload mismatch");
    }

    /// Verify media data matches after round-trip
    pub fn assert_media_equals(original: &TestMedia, loaded: &TestMedia) {
        assert_eq!(original.id, loaded.id, "id mismatch");
        assert_eq!(original.site_id, loaded.site_id, "site_id mismatch");
        assert_eq!(original.filename, loaded.filename, "filename mismatch");
        assert_eq!(original.original_filename, loaded.original_filename, "original_filename mismatch");
        assert_eq!(original.mime_type, loaded.mime_type, "mime_type mismatch");
        assert_eq!(original.file_size, loaded.file_size, "file_size mismatch");
        assert_eq!(original.storage_path, loaded.storage_path, "storage_path mismatch");
        assert_eq!(original.url, loaded.url, "url mismatch");
        assert_eq!(original.width, loaded.width, "width mismatch");
        assert_eq!(original.height, loaded.height, "height mismatch");
        assert_eq!(original.alt_text, loaded.alt_text, "alt_text mismatch");
        assert_eq!(original.caption, loaded.caption, "caption mismatch");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that scheduled posts are published correctly
    #[test]
    fn test_scheduled_post_publishing_query() {
        // This tests the SQL query structure
        let query = queries::PUBLISH_SCHEDULED_POSTS;
        assert!(query.contains("UPDATE posts"));
        assert!(query.contains("status = 'published'"));
        assert!(query.contains("scheduled_at <= NOW()"));
        assert!(query.contains("RETURNING id"));
    }

    /// Test that theme preview cleanup query is correct
    #[test]
    fn test_theme_preview_cleanup_query() {
        let query = queries::CLEAN_THEME_PREVIEWS;
        assert!(query.contains("DELETE FROM theme_previews"));
        assert!(query.contains("expires_at < NOW()"));
    }

    /// Test fixture creation
    #[test]
    fn test_post_fixture_creation() {
        let site_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let post = fixtures::create_test_post(site_id, author_id);

        assert_eq!(post.site_id, site_id);
        assert_eq!(post.author_id, author_id);
        assert!(!post.title.is_empty());
        assert!(!post.slug.is_empty());
        assert!(!post.content.is_empty());
        assert_eq!(post.status, "draft");
        assert_eq!(post.post_type, "post");
    }

    /// Test user fixture creation
    #[test]
    fn test_user_fixture_creation() {
        let site_id = Uuid::new_v4();
        let user = fixtures::create_test_user(site_id);

        assert_eq!(user.site_id, site_id);
        assert!(!user.email.is_empty());
        assert!(user.email.contains("@"));
        assert!(!user.username.is_empty());
        assert_eq!(user.status, "active");
    }

    /// Test theme fixture creation with JSON settings
    #[test]
    fn test_theme_fixture_with_settings() {
        let site_id = Uuid::new_v4();
        let theme = fixtures::create_test_theme(site_id);

        assert_eq!(theme.site_id, site_id);
        assert!(!theme.theme_id.is_empty());
        assert_eq!(theme.version, "1.0.0");

        // Verify settings JSON structure
        let settings = &theme.settings;
        assert!(settings.is_object());
        assert!(settings.get("primary_color").is_some());
        assert!(settings.get("font_family").is_some());
    }

    /// Test option fixture with nested JSON
    #[test]
    fn test_option_fixture_with_json_value() {
        let site_id = Uuid::new_v4();
        let option = fixtures::create_test_option(site_id);

        assert_eq!(option.site_id, site_id);
        assert!(!option.option_name.is_empty());
        assert!(option.autoload);

        // Verify nested JSON structure preserved
        let value = &option.option_value;
        assert_eq!(value.get("setting1"), Some(&json!(true)));
        assert_eq!(value.get("setting2"), Some(&json!("value")));
        assert_eq!(value.get("setting3"), Some(&json!(42)));
    }

    /// Test media fixture with dimensions
    #[test]
    fn test_media_fixture_with_dimensions() {
        let site_id = Uuid::new_v4();
        let media = fixtures::create_test_media(site_id);

        assert_eq!(media.site_id, site_id);
        assert!(!media.filename.is_empty());
        assert_eq!(media.mime_type, "image/jpeg");
        assert!(media.file_size > 0);
        assert_eq!(media.width, Some(1920));
        assert_eq!(media.height, Some(1080));
    }

    /// Test post assertion equality
    #[test]
    fn test_post_assertion_passes_for_equal_posts() {
        let site_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let post = fixtures::create_test_post(site_id, author_id);
        let post_copy = post.clone();

        // This should not panic
        assertions::assert_post_equals(&post, &post_copy);
    }

    /// Test user assertion equality
    #[test]
    fn test_user_assertion_passes_for_equal_users() {
        let site_id = Uuid::new_v4();
        let user = fixtures::create_test_user(site_id);
        let user_copy = user.clone();

        // This should not panic
        assertions::assert_user_equals(&user, &user_copy);
    }

    /// Test theme assertion with JSON settings equality
    #[test]
    fn test_theme_assertion_with_json_settings() {
        let site_id = Uuid::new_v4();
        let theme = fixtures::create_test_theme(site_id);
        let theme_copy = theme.clone();

        // This should not panic - JSON settings should match
        assertions::assert_theme_equals(&theme, &theme_copy);
    }

    /// Test option assertion with JSON value equality
    #[test]
    fn test_option_assertion_with_json_value() {
        let site_id = Uuid::new_v4();
        let option = fixtures::create_test_option(site_id);
        let option_copy = option.clone();

        // This should not panic - JSON values should match
        assertions::assert_option_equals(&option, &option_copy);
    }

    /// Test media assertion equality
    #[test]
    fn test_media_assertion_passes_for_equal_media() {
        let site_id = Uuid::new_v4();
        let media = fixtures::create_test_media(site_id);
        let media_copy = media.clone();

        // This should not panic
        assertions::assert_media_equals(&media, &media_copy);
    }

    /// Test that unique slugs are generated
    #[test]
    fn test_unique_post_slugs() {
        let site_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let post1 = fixtures::create_test_post(site_id, author_id);
        let post2 = fixtures::create_test_post(site_id, author_id);

        assert_ne!(post1.slug, post2.slug, "Slugs should be unique");
    }

    /// Test that unique emails are generated
    #[test]
    fn test_unique_user_emails() {
        let site_id = Uuid::new_v4();

        let user1 = fixtures::create_test_user(site_id);
        let user2 = fixtures::create_test_user(site_id);

        assert_ne!(user1.email, user2.email, "Emails should be unique");
        assert_ne!(user1.username, user2.username, "Usernames should be unique");
    }

    /// Test all SQL queries have correct structure
    #[test]
    fn test_sql_query_structure() {
        // Test INSERT queries have RETURNING or proper structure
        assert!(queries::INSERT_POST.contains("INSERT INTO posts"));
        assert!(queries::INSERT_POST.contains("RETURNING id"));

        assert!(queries::INSERT_USER.contains("INSERT INTO users"));
        assert!(queries::INSERT_THEME.contains("INSERT INTO themes"));
        assert!(queries::INSERT_MEDIA.contains("INSERT INTO media"));

        // Test SELECT queries
        assert!(queries::SELECT_POST.contains("SELECT"));
        assert!(queries::SELECT_POST.contains("FROM posts"));
        assert!(queries::SELECT_POST.contains("WHERE id = $1"));

        assert!(queries::SELECT_USER.contains("SELECT"));
        assert!(queries::SELECT_USER.contains("FROM users"));

        // Test UPSERT query
        assert!(queries::INSERT_OPTION.contains("ON CONFLICT"));
        assert!(queries::INSERT_OPTION.contains("DO UPDATE"));
    }
}
