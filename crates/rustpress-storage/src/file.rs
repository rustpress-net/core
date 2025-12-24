//! File types and metadata.

use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Stored file representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredFile {
    /// Unique identifier
    pub id: Uuid,
    /// Storage path
    pub path: String,
    /// Original filename
    pub original_name: String,
    /// Generated filename
    pub filename: String,
    /// MIME type
    pub mime_type: String,
    /// File size in bytes
    pub size: u64,
    /// File metadata
    pub metadata: FileMetadata,
    /// Storage backend used
    pub backend: String,
    /// Public URL (if available)
    pub url: Option<String>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
}

impl StoredFile {
    pub fn new(
        path: impl Into<String>,
        original_name: impl Into<String>,
        mime_type: impl Into<String>,
        size: u64,
    ) -> Self {
        let original = original_name.into();
        let path_str = path.into();
        let filename = Path::new(&path_str)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| original.clone());

        Self {
            id: Uuid::now_v7(),
            path: path_str,
            original_name: original,
            filename,
            mime_type: mime_type.into(),
            size,
            metadata: FileMetadata::default(),
            backend: "local".to_string(),
            url: None,
            created_at: Utc::now(),
        }
    }

    pub fn with_metadata(mut self, metadata: FileMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_backend(mut self, backend: impl Into<String>) -> Self {
        self.backend = backend.into();
        self
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Get file extension
    pub fn extension(&self) -> Option<&str> {
        Path::new(&self.filename)
            .extension()
            .and_then(|e| e.to_str())
    }

    /// Check if file is an image
    pub fn is_image(&self) -> bool {
        self.mime_type.starts_with("image/")
    }

    /// Check if file is a video
    pub fn is_video(&self) -> bool {
        self.mime_type.starts_with("video/")
    }

    /// Check if file is audio
    pub fn is_audio(&self) -> bool {
        self.mime_type.starts_with("audio/")
    }

    /// Check if file is a document
    pub fn is_document(&self) -> bool {
        matches!(
            self.mime_type.as_str(),
            "application/pdf"
                | "application/msword"
                | "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                | "application/vnd.ms-excel"
                | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                | "text/plain"
                | "text/markdown"
        )
    }

    /// Get human-readable size
    pub fn human_size(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if self.size >= GB {
            format!("{:.2} GB", self.size as f64 / GB as f64)
        } else if self.size >= MB {
            format!("{:.2} MB", self.size as f64 / MB as f64)
        } else if self.size >= KB {
            format!("{:.2} KB", self.size as f64 / KB as f64)
        } else {
            format!("{} bytes", self.size)
        }
    }
}

/// File metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileMetadata {
    /// Image width (if applicable)
    pub width: Option<u32>,
    /// Image height (if applicable)
    pub height: Option<u32>,
    /// Duration in seconds (for audio/video)
    pub duration: Option<u32>,
    /// Alt text for images
    pub alt_text: Option<String>,
    /// Title
    pub title: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Custom metadata
    pub custom: std::collections::HashMap<String, serde_json::Value>,
}

impl FileMetadata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn image(width: u32, height: u32) -> Self {
        Self {
            width: Some(width),
            height: Some(height),
            ..Default::default()
        }
    }

    pub fn video(width: u32, height: u32, duration: u32) -> Self {
        Self {
            width: Some(width),
            height: Some(height),
            duration: Some(duration),
            ..Default::default()
        }
    }

    pub fn audio(duration: u32) -> Self {
        Self {
            duration: Some(duration),
            ..Default::default()
        }
    }

    pub fn with_alt(mut self, alt: impl Into<String>) -> Self {
        self.alt_text = Some(alt.into());
        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_custom(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.custom.insert(key.into(), value);
        self
    }
}

/// Upload request
#[derive(Debug)]
pub struct UploadRequest {
    /// File content
    pub content: Bytes,
    /// Original filename
    pub filename: String,
    /// MIME type
    pub mime_type: String,
    /// Target directory/prefix
    pub directory: Option<String>,
    /// Metadata
    pub metadata: FileMetadata,
}

impl UploadRequest {
    pub fn new(content: Bytes, filename: impl Into<String>, mime_type: impl Into<String>) -> Self {
        Self {
            content,
            filename: filename.into(),
            mime_type: mime_type.into(),
            directory: None,
            metadata: FileMetadata::default(),
        }
    }

    pub fn with_directory(mut self, dir: impl Into<String>) -> Self {
        self.directory = Some(dir.into());
        self
    }

    pub fn with_metadata(mut self, metadata: FileMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn size(&self) -> u64 {
        self.content.len() as u64
    }
}

/// File path generator
pub struct PathGenerator;

impl PathGenerator {
    /// Generate a unique path for a file
    pub fn generate(filename: &str, directory: Option<&str>) -> String {
        let now = Utc::now();
        let uuid = Uuid::now_v7();

        // Extract extension
        let ext = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e))
            .unwrap_or_default();

        // Generate path: YYYY/MM/uuid.ext
        let date_prefix = now.format("%Y/%m").to_string();
        let new_filename = format!("{}{}", uuid, ext);

        match directory {
            Some(dir) => format!("{}/{}/{}", dir, date_prefix, new_filename),
            None => format!("{}/{}", date_prefix, new_filename),
        }
    }

    /// Generate path preserving original filename
    pub fn generate_with_name(filename: &str, directory: Option<&str>) -> String {
        let now = Utc::now();
        let date_prefix = now.format("%Y/%m").to_string();

        // Sanitize filename
        let safe_filename = Self::sanitize_filename(filename);

        match directory {
            Some(dir) => format!("{}/{}/{}", dir, date_prefix, safe_filename),
            None => format!("{}/{}", date_prefix, safe_filename),
        }
    }

    /// Sanitize a filename
    pub fn sanitize_filename(filename: &str) -> String {
        let stem = Path::new(filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");

        let ext = Path::new(filename)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let safe_stem: String = stem
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c.to_ascii_lowercase()
                } else if c.is_whitespace() {
                    '-'
                } else {
                    '_'
                }
            })
            .collect();

        if ext.is_empty() {
            safe_stem
        } else {
            format!("{}.{}", safe_stem, ext.to_lowercase())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stored_file() {
        let file = StoredFile::new("2024/01/abc.jpg", "photo.jpg", "image/jpeg", 1024);

        assert!(file.is_image());
        assert!(!file.is_video());
        assert_eq!(file.extension(), Some("jpg"));
        assert_eq!(file.human_size(), "1.00 KB");
    }

    #[test]
    fn test_path_generator() {
        let path = PathGenerator::generate("photo.jpg", Some("uploads"));
        assert!(path.starts_with("uploads/"));
        assert!(path.ends_with(".jpg"));

        let path = PathGenerator::generate("document.pdf", None);
        assert!(path.ends_with(".pdf"));
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(
            PathGenerator::sanitize_filename("My Photo (1).JPG"),
            "my-photo-_1_.jpg"
        );
        assert_eq!(
            PathGenerator::sanitize_filename("report.PDF"),
            "report.pdf"
        );
    }

    #[test]
    fn test_file_metadata() {
        let meta = FileMetadata::image(1920, 1080)
            .with_alt("A beautiful sunset")
            .with_title("Sunset Photo");

        assert_eq!(meta.width, Some(1920));
        assert_eq!(meta.height, Some(1080));
        assert_eq!(meta.alt_text, Some("A beautiful sunset".to_string()));
    }
}
