//! RustPress Media Management System
//!
//! This crate provides comprehensive media management functionality including:
//! - Image optimization (WebP, AVIF conversion)
//! - Lazy loading support
//! - Responsive image srcsets
//! - Media library with folders
//! - Drag-and-drop upload support
//! - Image editing (crop, resize, filters)
//! - Video transcoding
//! - Audio player support

pub mod image_optimizer;
pub mod lazy_loading;
pub mod srcset;
pub mod library;
pub mod upload;
pub mod editor;
pub mod video;
pub mod audio;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

// Re-exports
pub use image_optimizer::*;
pub use lazy_loading::*;
pub use srcset::*;
pub use library::*;
pub use upload::*;
pub use editor::*;
pub use video::*;
pub use audio::*;

/// Media errors
#[derive(Error, Debug)]
pub enum MediaError {
    #[error("Media not found: {0}")]
    NotFound(Uuid),

    #[error("Invalid media type: {0}")]
    InvalidType(String),

    #[error("File too large: {size} bytes (max: {max} bytes)")]
    FileTooLarge { size: u64, max: u64 },

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Processing error: {0}")]
    ProcessingError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Image processing error: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type MediaResult<T> = Result<T, MediaError>;

/// Media item types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "media_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Image,
    Video,
    Audio,
    Document,
    Archive,
    Other,
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaType::Image => write!(f, "image"),
            MediaType::Video => write!(f, "video"),
            MediaType::Audio => write!(f, "audio"),
            MediaType::Document => write!(f, "document"),
            MediaType::Archive => write!(f, "archive"),
            MediaType::Other => write!(f, "other"),
        }
    }
}

impl MediaType {
    /// Determine media type from MIME type
    pub fn from_mime(mime: &str) -> Self {
        if mime.starts_with("image/") {
            Self::Image
        } else if mime.starts_with("video/") {
            Self::Video
        } else if mime.starts_with("audio/") {
            Self::Audio
        } else if mime.starts_with("application/pdf")
            || mime.starts_with("application/msword")
            || mime.starts_with("application/vnd.")
            || mime.starts_with("text/")
        {
            Self::Document
        } else if mime.starts_with("application/zip")
            || mime.starts_with("application/x-rar")
            || mime.starts_with("application/x-tar")
            || mime.starts_with("application/gzip")
        {
            Self::Archive
        } else {
            Self::Other
        }
    }
}

/// Media item representing a file in the media library
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MediaItem {
    /// Unique identifier
    pub id: Uuid,

    /// Original filename
    pub filename: String,

    /// Title/display name
    pub title: String,

    /// Alt text for accessibility
    pub alt_text: String,

    /// Caption
    pub caption: String,

    /// Description
    pub description: String,

    /// Media type
    pub media_type: MediaType,

    /// MIME type
    pub mime_type: String,

    /// File size in bytes
    pub file_size: i64,

    /// Storage path (relative)
    pub path: String,

    /// URL to access the media
    pub url: String,

    /// Thumbnail URL (if applicable)
    pub thumbnail_url: Option<String>,

    /// Image width (if applicable)
    pub width: Option<i32>,

    /// Image height (if applicable)
    pub height: Option<i32>,

    /// Duration in seconds (for audio/video)
    pub duration: Option<f64>,

    /// File hash for deduplication
    pub file_hash: String,

    /// Folder ID (for organization)
    pub folder_id: Option<Uuid>,

    /// Metadata (EXIF, etc.)
    pub metadata: serde_json::Value,

    /// Upload user ID
    pub uploaded_by: Uuid,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

impl MediaItem {
    /// Check if this is an image
    pub fn is_image(&self) -> bool {
        self.media_type == MediaType::Image
    }

    /// Check if this is a video
    pub fn is_video(&self) -> bool {
        self.media_type == MediaType::Video
    }

    /// Check if this is audio
    pub fn is_audio(&self) -> bool {
        self.media_type == MediaType::Audio
    }

    /// Get dimensions as tuple
    pub fn dimensions(&self) -> Option<(i32, i32)> {
        match (self.width, self.height) {
            (Some(w), Some(h)) => Some((w, h)),
            _ => None,
        }
    }

    /// Get aspect ratio
    pub fn aspect_ratio(&self) -> Option<f64> {
        self.dimensions()
            .map(|(w, h)| w as f64 / h as f64)
    }

    /// Get human-readable file size
    pub fn human_file_size(&self) -> String {
        let size = self.file_size as f64;
        if size < 1024.0 {
            format!("{} B", self.file_size)
        } else if size < 1024.0 * 1024.0 {
            format!("{:.1} KB", size / 1024.0)
        } else if size < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.1} MB", size / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", size / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

/// Media folder for organization
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MediaFolder {
    /// Folder ID
    pub id: Uuid,

    /// Folder name
    pub name: String,

    /// Parent folder ID (for nested folders)
    pub parent_id: Option<Uuid>,

    /// Folder path
    pub path: String,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Media service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaConfig {
    /// Base storage path
    pub storage_path: String,

    /// Base URL for media access
    pub base_url: String,

    /// Maximum upload size in bytes
    pub max_upload_size: u64,

    /// Allowed MIME types
    pub allowed_types: Vec<String>,

    /// Enable image optimization
    pub optimize_images: bool,

    /// Generate WebP versions
    pub generate_webp: bool,

    /// Generate AVIF versions
    pub generate_avif: bool,

    /// Thumbnail sizes to generate
    pub thumbnail_sizes: Vec<(u32, u32)>,

    /// Enable lazy loading
    pub enable_lazy_loading: bool,

    /// Enable srcset generation
    pub enable_srcset: bool,
}

impl Default for MediaConfig {
    fn default() -> Self {
        Self {
            storage_path: "./uploads".to_string(),
            base_url: "/uploads".to_string(),
            max_upload_size: 100 * 1024 * 1024, // 100 MB
            allowed_types: vec![
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "image/gif".to_string(),
                "image/webp".to_string(),
                "image/svg+xml".to_string(),
                "video/mp4".to_string(),
                "video/webm".to_string(),
                "audio/mpeg".to_string(),
                "audio/wav".to_string(),
                "audio/ogg".to_string(),
                "application/pdf".to_string(),
            ],
            optimize_images: true,
            generate_webp: true,
            generate_avif: false, // AVIF requires additional dependencies
            thumbnail_sizes: vec![
                (150, 150),   // Thumbnail
                (300, 300),   // Small
                (600, 600),   // Medium
                (1200, 1200), // Large
            ],
            enable_lazy_loading: true,
            enable_srcset: true,
        }
    }
}

/// Main media service
pub struct MediaService {
    pool: PgPool,
    config: MediaConfig,
}

impl MediaService {
    /// Create new media service
    pub fn new(pool: PgPool, config: MediaConfig) -> Self {
        Self { pool, config }
    }

    /// Get media item by ID
    pub async fn get(&self, id: Uuid) -> MediaResult<MediaItem> {
        let media: Option<MediaItem> = sqlx::query_as(
            r#"
            SELECT
                id, filename, title, alt_text, caption, description,
                media_type, mime_type, file_size, path, url, thumbnail_url,
                width, height, duration, file_hash, folder_id,
                metadata, uploaded_by, created_at, updated_at
            FROM media_items
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        media.ok_or(MediaError::NotFound(id))
    }

    /// List media items with pagination
    pub async fn list(
        &self,
        folder_id: Option<Uuid>,
        media_type: Option<MediaType>,
        limit: i64,
        offset: i64,
    ) -> MediaResult<Vec<MediaItem>> {
        let media: Vec<MediaItem> = sqlx::query_as(
            r#"
            SELECT
                id, filename, title, alt_text, caption, description,
                media_type, mime_type, file_size, path, url, thumbnail_url,
                width, height, duration, file_hash, folder_id,
                metadata, uploaded_by, created_at, updated_at
            FROM media_items
            WHERE ($1::uuid IS NULL OR folder_id = $1)
            AND ($2::text IS NULL OR media_type::text = $2)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#
        )
        .bind(folder_id)
        .bind(media_type.map(|t| t.to_string()))
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(media)
    }

    /// Search media items
    pub async fn search(&self, query: &str, limit: i64) -> MediaResult<Vec<MediaItem>> {
        let search_pattern = format!("%{}%", query);

        let media: Vec<MediaItem> = sqlx::query_as(
            r#"
            SELECT
                id, filename, title, alt_text, caption, description,
                media_type, mime_type, file_size, path, url, thumbnail_url,
                width, height, duration, file_hash, folder_id,
                metadata, uploaded_by, created_at, updated_at
            FROM media_items
            WHERE title ILIKE $1
            OR filename ILIKE $1
            OR alt_text ILIKE $1
            OR caption ILIKE $1
            ORDER BY created_at DESC
            LIMIT $2
            "#
        )
        .bind(&search_pattern)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(media)
    }

    /// Update media item metadata
    pub async fn update(&self, id: Uuid, title: &str, alt_text: &str, caption: &str, description: &str) -> MediaResult<MediaItem> {
        let media: Option<MediaItem> = sqlx::query_as(
            r#"
            UPDATE media_items
            SET title = $2, alt_text = $3, caption = $4, description = $5, updated_at = NOW()
            WHERE id = $1
            RETURNING
                id, filename, title, alt_text, caption, description,
                media_type, mime_type, file_size, path, url, thumbnail_url,
                width, height, duration, file_hash, folder_id,
                metadata, uploaded_by, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(title)
        .bind(alt_text)
        .bind(caption)
        .bind(description)
        .fetch_optional(&self.pool)
        .await?;

        media.ok_or(MediaError::NotFound(id))
    }

    /// Delete media item
    pub async fn delete(&self, id: Uuid) -> MediaResult<()> {
        // Get the media item first to delete files
        let media = self.get(id).await?;

        // Delete the file from storage
        let file_path = format!("{}/{}", self.config.storage_path, media.path);
        if tokio::fs::metadata(&file_path).await.is_ok() {
            tokio::fs::remove_file(&file_path).await?;
        }

        // Delete from database
        sqlx::query("DELETE FROM media_items WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Move media to folder
    pub async fn move_to_folder(&self, id: Uuid, folder_id: Option<Uuid>) -> MediaResult<()> {
        sqlx::query("UPDATE media_items SET folder_id = $2, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(folder_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Get config
    pub fn config(&self) -> &MediaConfig {
        &self.config
    }
}

/// SQL migrations for media tables
pub const MEDIA_MIGRATIONS: &str = r#"
-- Media type enum
DO $$ BEGIN
    CREATE TYPE media_type AS ENUM ('image', 'video', 'audio', 'document', 'archive', 'other');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Media folders table
CREATE TABLE IF NOT EXISTS media_folders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    parent_id UUID REFERENCES media_folders(id) ON DELETE CASCADE,
    path VARCHAR(1000) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_media_folders_parent ON media_folders(parent_id);
CREATE INDEX IF NOT EXISTS idx_media_folders_path ON media_folders(path);

-- Media items table
CREATE TABLE IF NOT EXISTS media_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    filename VARCHAR(255) NOT NULL,
    title VARCHAR(500) NOT NULL DEFAULT '',
    alt_text VARCHAR(500) NOT NULL DEFAULT '',
    caption TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    media_type media_type NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    file_size BIGINT NOT NULL,
    path VARCHAR(1000) NOT NULL,
    url VARCHAR(1000) NOT NULL,
    thumbnail_url VARCHAR(1000),
    width INTEGER,
    height INTEGER,
    duration DOUBLE PRECISION,
    file_hash VARCHAR(64) NOT NULL,
    folder_id UUID REFERENCES media_folders(id) ON DELETE SET NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    uploaded_by UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_media_items_folder ON media_items(folder_id);
CREATE INDEX IF NOT EXISTS idx_media_items_type ON media_items(media_type);
CREATE INDEX IF NOT EXISTS idx_media_items_hash ON media_items(file_hash);
CREATE INDEX IF NOT EXISTS idx_media_items_created ON media_items(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_media_items_search ON media_items USING gin(to_tsvector('english', title || ' ' || filename || ' ' || alt_text));

-- Image variants table (for srcset)
CREATE TABLE IF NOT EXISTS media_variants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    media_id UUID NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    variant_type VARCHAR(50) NOT NULL, -- 'thumbnail', 'small', 'medium', 'large', 'webp', 'avif'
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    file_size BIGINT NOT NULL,
    path VARCHAR(1000) NOT NULL,
    url VARCHAR(1000) NOT NULL,
    format VARCHAR(20) NOT NULL, -- 'jpeg', 'png', 'webp', 'avif'
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_media_variants_media ON media_variants(media_id);
CREATE INDEX IF NOT EXISTS idx_media_variants_type ON media_variants(variant_type);

-- Video metadata table
CREATE TABLE IF NOT EXISTS video_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    media_id UUID NOT NULL UNIQUE REFERENCES media_items(id) ON DELETE CASCADE,
    codec VARCHAR(50),
    bitrate INTEGER,
    framerate DOUBLE PRECISION,
    resolution VARCHAR(20),
    has_audio BOOLEAN DEFAULT TRUE,
    poster_url VARCHAR(1000),
    transcoded_versions JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Audio metadata table
CREATE TABLE IF NOT EXISTS audio_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    media_id UUID NOT NULL UNIQUE REFERENCES media_items(id) ON DELETE CASCADE,
    codec VARCHAR(50),
    bitrate INTEGER,
    sample_rate INTEGER,
    channels INTEGER,
    artist VARCHAR(255),
    album VARCHAR(255),
    title VARCHAR(255),
    genre VARCHAR(100),
    year INTEGER,
    cover_art_url VARCHAR(1000),
    waveform_data JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_type_from_mime() {
        assert_eq!(MediaType::from_mime("image/jpeg"), MediaType::Image);
        assert_eq!(MediaType::from_mime("video/mp4"), MediaType::Video);
        assert_eq!(MediaType::from_mime("audio/mpeg"), MediaType::Audio);
        assert_eq!(MediaType::from_mime("application/pdf"), MediaType::Document);
        assert_eq!(MediaType::from_mime("application/zip"), MediaType::Archive);
        assert_eq!(MediaType::from_mime("application/octet-stream"), MediaType::Other);
    }

    #[test]
    fn test_human_file_size() {
        let item = MediaItem {
            id: Uuid::new_v4(),
            filename: "test.jpg".to_string(),
            title: "Test".to_string(),
            alt_text: String::new(),
            caption: String::new(),
            description: String::new(),
            media_type: MediaType::Image,
            mime_type: "image/jpeg".to_string(),
            file_size: 1536,
            path: String::new(),
            url: String::new(),
            thumbnail_url: None,
            width: None,
            height: None,
            duration: None,
            file_hash: String::new(),
            folder_id: None,
            metadata: serde_json::json!({}),
            uploaded_by: Uuid::new_v4(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(item.human_file_size(), "1.5 KB");
    }
}
