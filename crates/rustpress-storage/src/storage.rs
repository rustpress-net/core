//! High-level storage API.

use crate::backend::StorageBackend;
use crate::file::{FileMetadata, StoredFile, UploadRequest};
use bytes::Bytes;
use rustpress_core::error::{Error, Result};
use std::sync::Arc;

/// Storage configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Maximum upload size in bytes
    pub max_upload_size: u64,
    /// Allowed MIME types (empty = all allowed)
    pub allowed_types: Vec<String>,
    /// Denied MIME types
    pub denied_types: Vec<String>,
    /// Default directory for uploads
    pub default_directory: Option<String>,
    /// CDN URL for serving files
    pub cdn_url: Option<String>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            max_upload_size: 50 * 1024 * 1024, // 50MB
            allowed_types: vec![
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "image/gif".to_string(),
                "image/webp".to_string(),
                "image/svg+xml".to_string(),
                "application/pdf".to_string(),
                "video/mp4".to_string(),
                "video/webm".to_string(),
                "audio/mpeg".to_string(),
                "audio/ogg".to_string(),
                "text/plain".to_string(),
                "application/zip".to_string(),
            ],
            denied_types: vec![
                "application/x-executable".to_string(),
                "application/x-msdownload".to_string(),
            ],
            default_directory: None,
            cdn_url: None,
        }
    }
}

/// High-level storage interface
pub struct Storage {
    backend: Arc<dyn StorageBackend>,
    config: StorageConfig,
}

impl Storage {
    /// Create new storage with backend
    pub fn new(backend: Arc<dyn StorageBackend>) -> Self {
        Self {
            backend,
            config: StorageConfig::default(),
        }
    }

    /// Create storage with custom configuration
    pub fn with_config(backend: Arc<dyn StorageBackend>, config: StorageConfig) -> Self {
        Self { backend, config }
    }

    /// Upload a file
    pub async fn upload(
        &self,
        content: Bytes,
        filename: &str,
        mime_type: &str,
    ) -> Result<StoredFile> {
        self.validate_upload(&content, mime_type)?;

        let mut request = UploadRequest::new(content, filename, mime_type);
        if let Some(dir) = &self.config.default_directory {
            request = request.with_directory(dir);
        }

        self.backend.store(request).await
    }

    /// Upload a file with metadata
    pub async fn upload_with_metadata(
        &self,
        content: Bytes,
        filename: &str,
        mime_type: &str,
        metadata: FileMetadata,
    ) -> Result<StoredFile> {
        self.validate_upload(&content, mime_type)?;

        let mut request = UploadRequest::new(content, filename, mime_type).with_metadata(metadata);
        if let Some(dir) = &self.config.default_directory {
            request = request.with_directory(dir);
        }

        self.backend.store(request).await
    }

    /// Upload to a specific directory
    pub async fn upload_to(
        &self,
        content: Bytes,
        filename: &str,
        mime_type: &str,
        directory: &str,
    ) -> Result<StoredFile> {
        self.validate_upload(&content, mime_type)?;

        let request = UploadRequest::new(content, filename, mime_type).with_directory(directory);
        self.backend.store(request).await
    }

    /// Get file contents
    pub async fn get(&self, path: &str) -> Result<Bytes> {
        self.backend.get(path).await
    }

    /// Delete a file
    pub async fn delete(&self, path: &str) -> Result<bool> {
        self.backend.delete(path).await
    }

    /// Check if file exists
    pub async fn exists(&self, path: &str) -> Result<bool> {
        self.backend.exists(path).await
    }

    /// Get file size
    pub async fn size(&self, path: &str) -> Result<u64> {
        self.backend.size(path).await
    }

    /// Copy a file
    pub async fn copy(&self, from: &str, to: &str) -> Result<StoredFile> {
        self.backend.copy(from, to).await
    }

    /// Move a file
    pub async fn move_file(&self, from: &str, to: &str) -> Result<StoredFile> {
        self.backend.move_file(from, to).await
    }

    /// Get public URL
    pub fn url(&self, path: &str) -> Option<String> {
        // Use CDN URL if configured, otherwise use backend URL
        if let Some(cdn) = &self.config.cdn_url {
            Some(format!("{}/{}", cdn.trim_end_matches('/'), path))
        } else {
            self.backend.url(path)
        }
    }

    /// Get temporary/signed URL
    pub async fn temporary_url(&self, path: &str, expires_in_secs: u64) -> Result<String> {
        self.backend.temporary_url(path, expires_in_secs).await
    }

    /// List files in directory
    pub async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        self.backend.list(prefix).await
    }

    /// Health check
    pub async fn health_check(&self) -> Result<()> {
        self.backend.health_check().await
    }

    /// Get backend name
    pub fn backend_name(&self) -> &str {
        self.backend.name()
    }

    /// Validate upload
    fn validate_upload(&self, content: &Bytes, mime_type: &str) -> Result<()> {
        // Check size
        if content.len() as u64 > self.config.max_upload_size {
            return Err(Error::InvalidInput {
                field: "file".to_string(),
                message: format!(
                    "File too large. Maximum size is {} bytes",
                    self.config.max_upload_size
                ),
            });
        }

        // Check denied types
        if self.config.denied_types.iter().any(|t| t == mime_type) {
            return Err(Error::InvalidInput {
                field: "file".to_string(),
                message: format!("File type '{}' is not allowed", mime_type),
            });
        }

        // Check allowed types (if list is not empty)
        if !self.config.allowed_types.is_empty()
            && !self.config.allowed_types.iter().any(|t| t == mime_type)
        {
            return Err(Error::InvalidInput {
                field: "file".to_string(),
                message: format!("File type '{}' is not in the allowed types list", mime_type),
            });
        }

        Ok(())
    }
}

/// MIME type detection utilities
pub struct MimeDetector;

impl MimeDetector {
    /// Detect MIME type from file extension
    pub fn from_extension(ext: &str) -> Option<&'static str> {
        match ext.to_lowercase().as_str() {
            // Images
            "jpg" | "jpeg" => Some("image/jpeg"),
            "png" => Some("image/png"),
            "gif" => Some("image/gif"),
            "webp" => Some("image/webp"),
            "svg" => Some("image/svg+xml"),
            "ico" => Some("image/x-icon"),
            "bmp" => Some("image/bmp"),

            // Videos
            "mp4" => Some("video/mp4"),
            "webm" => Some("video/webm"),
            "avi" => Some("video/x-msvideo"),
            "mov" => Some("video/quicktime"),
            "mkv" => Some("video/x-matroska"),

            // Audio
            "mp3" => Some("audio/mpeg"),
            "ogg" => Some("audio/ogg"),
            "wav" => Some("audio/wav"),
            "flac" => Some("audio/flac"),
            "aac" => Some("audio/aac"),

            // Documents
            "pdf" => Some("application/pdf"),
            "doc" => Some("application/msword"),
            "docx" => Some(
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            ),
            "xls" => Some("application/vnd.ms-excel"),
            "xlsx" => {
                Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
            }
            "ppt" => Some("application/vnd.ms-powerpoint"),
            "pptx" => Some(
                "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            ),

            // Text
            "txt" => Some("text/plain"),
            "html" | "htm" => Some("text/html"),
            "css" => Some("text/css"),
            "js" => Some("application/javascript"),
            "json" => Some("application/json"),
            "xml" => Some("application/xml"),
            "md" => Some("text/markdown"),
            "csv" => Some("text/csv"),

            // Archives
            "zip" => Some("application/zip"),
            "rar" => Some("application/vnd.rar"),
            "7z" => Some("application/x-7z-compressed"),
            "tar" => Some("application/x-tar"),
            "gz" => Some("application/gzip"),

            // Other
            "woff" => Some("font/woff"),
            "woff2" => Some("font/woff2"),
            "ttf" => Some("font/ttf"),
            "otf" => Some("font/otf"),

            _ => None,
        }
    }

    /// Detect MIME type from filename
    pub fn from_filename(filename: &str) -> Option<&'static str> {
        let ext = std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())?;
        Self::from_extension(ext)
    }

    /// Check if MIME type is an image
    pub fn is_image(mime: &str) -> bool {
        mime.starts_with("image/")
    }

    /// Check if MIME type is a video
    pub fn is_video(mime: &str) -> bool {
        mime.starts_with("video/")
    }

    /// Check if MIME type is audio
    pub fn is_audio(mime: &str) -> bool {
        mime.starts_with("audio/")
    }

    /// Check if MIME type is a document
    pub fn is_document(mime: &str) -> bool {
        matches!(
            mime,
            "application/pdf"
                | "application/msword"
                | "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                | "application/vnd.ms-excel"
                | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                | "text/plain"
                | "text/markdown"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::LocalBackend;
    use tempfile::TempDir;

    fn create_test_storage() -> (Storage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let backend = Arc::new(LocalBackend::new(temp_dir.path()));
        let storage = Storage::new(backend);
        (storage, temp_dir)
    }

    #[tokio::test]
    async fn test_storage_upload() {
        let (storage, _temp) = create_test_storage();

        let content = Bytes::from("Hello, Storage!");
        let file = storage.upload(content, "test.txt", "text/plain").await.unwrap();

        assert!(file.path.ends_with(".txt"));
        assert_eq!(file.size, 15);
    }

    #[tokio::test]
    async fn test_storage_validation() {
        let (storage, _temp) = create_test_storage();

        // File too large
        let large_content = Bytes::from(vec![0u8; 100 * 1024 * 1024]); // 100MB
        let result = storage.upload(large_content, "big.bin", "application/octet-stream").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_mime_detector() {
        assert_eq!(MimeDetector::from_extension("jpg"), Some("image/jpeg"));
        assert_eq!(MimeDetector::from_extension("mp4"), Some("video/mp4"));
        assert_eq!(MimeDetector::from_filename("document.pdf"), Some("application/pdf"));
        assert!(MimeDetector::is_image("image/png"));
        assert!(MimeDetector::is_video("video/mp4"));
    }
}
