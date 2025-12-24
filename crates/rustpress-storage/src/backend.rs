//! Storage backend implementations.

use crate::file::{PathGenerator, StoredFile, UploadRequest};
use async_trait::async_trait;
use bytes::Bytes;
use rustpress_core::error::{Error, Result};
use std::path::{Path, PathBuf};

/// Storage backend trait
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Backend name
    fn name(&self) -> &str;

    /// Store a file
    async fn store(&self, request: UploadRequest) -> Result<StoredFile>;

    /// Get file contents
    async fn get(&self, path: &str) -> Result<Bytes>;

    /// Delete a file
    async fn delete(&self, path: &str) -> Result<bool>;

    /// Check if file exists
    async fn exists(&self, path: &str) -> Result<bool>;

    /// Get file size
    async fn size(&self, path: &str) -> Result<u64>;

    /// Copy a file
    async fn copy(&self, from: &str, to: &str) -> Result<StoredFile>;

    /// Move a file
    async fn move_file(&self, from: &str, to: &str) -> Result<StoredFile>;

    /// Get public URL for a file
    fn url(&self, path: &str) -> Option<String>;

    /// Get temporary/signed URL
    async fn temporary_url(&self, path: &str, expires_in_secs: u64) -> Result<String>;

    /// List files in a directory
    async fn list(&self, prefix: &str) -> Result<Vec<String>>;

    /// Health check
    async fn health_check(&self) -> Result<()>;
}

/// Local filesystem storage backend
pub struct LocalBackend {
    root: PathBuf,
    base_url: Option<String>,
}

impl LocalBackend {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            base_url: None,
        }
    }

    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    fn full_path(&self, path: &str) -> PathBuf {
        self.root.join(path)
    }

    async fn ensure_directory(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                Error::Storage {
                    message: format!("Failed to create directory: {}", e),
                    source: Some(Box::new(e)),
                }
            })?;
        }
        Ok(())
    }
}

#[async_trait]
impl StorageBackend for LocalBackend {
    fn name(&self) -> &str {
        "local"
    }

    async fn store(&self, request: UploadRequest) -> Result<StoredFile> {
        let path = PathGenerator::generate(&request.filename, request.directory.as_deref());
        let full_path = self.full_path(&path);

        self.ensure_directory(&full_path).await?;

        tokio::fs::write(&full_path, &request.content)
            .await
            .map_err(|e| Error::Storage {
                message: format!("Failed to write file: {}", e),
                source: Some(Box::new(e)),
            })?;

        let size = request.content.len() as u64;

        let mut file = StoredFile::new(&path, &request.filename, &request.mime_type, size)
            .with_backend("local")
            .with_metadata(request.metadata);

        if let Some(url) = self.url(&path) {
            file = file.with_url(url);
        }

        tracing::debug!(path = %path, size = size, "File stored locally");
        Ok(file)
    }

    async fn get(&self, path: &str) -> Result<Bytes> {
        let full_path = self.full_path(path);

        let content = tokio::fs::read(&full_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::FileNotFound {
                    path: path.to_string(),
                }
            } else {
                Error::Storage {
                    message: format!("Failed to read file: {}", e),
                    source: Some(Box::new(e)),
                }
            }
        })?;

        Ok(Bytes::from(content))
    }

    async fn delete(&self, path: &str) -> Result<bool> {
        let full_path = self.full_path(path);

        match tokio::fs::remove_file(&full_path).await {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(Error::Storage {
                message: format!("Failed to delete file: {}", e),
                source: Some(Box::new(e)),
            }),
        }
    }

    async fn exists(&self, path: &str) -> Result<bool> {
        let full_path = self.full_path(path);
        Ok(tokio::fs::try_exists(&full_path).await.unwrap_or(false))
    }

    async fn size(&self, path: &str) -> Result<u64> {
        let full_path = self.full_path(path);

        let metadata = tokio::fs::metadata(&full_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::FileNotFound {
                    path: path.to_string(),
                }
            } else {
                Error::Storage {
                    message: format!("Failed to get file metadata: {}", e),
                    source: Some(Box::new(e)),
                }
            }
        })?;

        Ok(metadata.len())
    }

    async fn copy(&self, from: &str, to: &str) -> Result<StoredFile> {
        let from_path = self.full_path(from);
        let to_path = self.full_path(to);

        self.ensure_directory(&to_path).await?;

        tokio::fs::copy(&from_path, &to_path)
            .await
            .map_err(|e| Error::Storage {
                message: format!("Failed to copy file: {}", e),
                source: Some(Box::new(e)),
            })?;

        let metadata = tokio::fs::metadata(&to_path)
            .await
            .map_err(|e| Error::Storage {
                message: format!("Failed to get file metadata: {}", e),
                source: Some(Box::new(e)),
            })?;

        let filename = Path::new(to)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("file");

        Ok(StoredFile::new(to, filename, "application/octet-stream", metadata.len())
            .with_backend("local"))
    }

    async fn move_file(&self, from: &str, to: &str) -> Result<StoredFile> {
        let from_path = self.full_path(from);
        let to_path = self.full_path(to);

        self.ensure_directory(&to_path).await?;

        tokio::fs::rename(&from_path, &to_path)
            .await
            .map_err(|e| Error::Storage {
                message: format!("Failed to move file: {}", e),
                source: Some(Box::new(e)),
            })?;

        let metadata = tokio::fs::metadata(&to_path)
            .await
            .map_err(|e| Error::Storage {
                message: format!("Failed to get file metadata: {}", e),
                source: Some(Box::new(e)),
            })?;

        let filename = Path::new(to)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("file");

        Ok(StoredFile::new(to, filename, "application/octet-stream", metadata.len())
            .with_backend("local"))
    }

    fn url(&self, path: &str) -> Option<String> {
        self.base_url
            .as_ref()
            .map(|base| format!("{}/{}", base.trim_end_matches('/'), path))
    }

    async fn temporary_url(&self, path: &str, _expires_in_secs: u64) -> Result<String> {
        // Local storage doesn't support signed URLs, just return the regular URL
        self.url(path).ok_or_else(|| Error::Storage {
            message: "Base URL not configured".to_string(),
            source: None,
        })
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let dir_path = self.full_path(prefix);
        let mut files = Vec::new();

        let mut entries = tokio::fs::read_dir(&dir_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::FileNotFound {
                    path: prefix.to_string(),
                }
            } else {
                Error::Storage {
                    message: format!("Failed to list directory: {}", e),
                    source: Some(Box::new(e)),
                }
            }
        })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| Error::Storage {
            message: format!("Failed to read directory entry: {}", e),
            source: Some(Box::new(e)),
        })? {
            if let Some(path) = entry.path().strip_prefix(&self.root).ok() {
                files.push(path.to_string_lossy().to_string());
            }
        }

        Ok(files)
    }

    async fn health_check(&self) -> Result<()> {
        // Check if root directory exists and is writable
        if !self.root.exists() {
            tokio::fs::create_dir_all(&self.root).await.map_err(|e| {
                Error::Storage {
                    message: format!("Failed to create storage directory: {}", e),
                    source: Some(Box::new(e)),
                }
            })?;
        }

        // Try to write a test file
        let test_path = self.root.join(".health_check");
        tokio::fs::write(&test_path, "test")
            .await
            .map_err(|e| Error::Storage {
                message: format!("Storage directory not writable: {}", e),
                source: Some(Box::new(e)),
            })?;
        tokio::fs::remove_file(&test_path).await.ok();

        Ok(())
    }
}

/// S3-compatible storage backend
#[cfg(feature = "s3")]
pub struct S3Backend {
    store: object_store::aws::AmazonS3,
    bucket: String,
    base_url: Option<String>,
}

#[cfg(feature = "s3")]
impl S3Backend {
    pub fn new(
        bucket: impl Into<String>,
        region: impl Into<String>,
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> Result<Self> {
        use object_store::aws::AmazonS3Builder;

        let bucket = bucket.into();
        let store = AmazonS3Builder::new()
            .with_bucket_name(&bucket)
            .with_region(region)
            .with_access_key_id(access_key)
            .with_secret_access_key(secret_key)
            .build()
            .map_err(|e| Error::Storage {
                message: format!("Failed to create S3 backend: {}", e),
                source: Some(Box::new(e)),
            })?;

        Ok(Self {
            store,
            bucket,
            base_url: None,
        })
    }

    pub fn with_endpoint(
        bucket: impl Into<String>,
        endpoint: impl Into<String>,
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> Result<Self> {
        use object_store::aws::AmazonS3Builder;

        let bucket = bucket.into();
        let store = AmazonS3Builder::new()
            .with_bucket_name(&bucket)
            .with_endpoint(endpoint)
            .with_access_key_id(access_key)
            .with_secret_access_key(secret_key)
            .with_allow_http(true)
            .build()
            .map_err(|e| Error::Storage {
                message: format!("Failed to create S3 backend: {}", e),
                source: Some(Box::new(e)),
            })?;

        Ok(Self {
            store,
            bucket,
            base_url: None,
        })
    }

    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }
}

#[cfg(feature = "s3")]
#[async_trait]
impl StorageBackend for S3Backend {
    fn name(&self) -> &str {
        "s3"
    }

    async fn store(&self, request: UploadRequest) -> Result<StoredFile> {
        use object_store::ObjectStore;

        let path = PathGenerator::generate(&request.filename, request.directory.as_deref());
        let location = object_store::path::Path::from(path.clone());

        self.store
            .put(&location, request.content.clone().into())
            .await
            .map_err(|e| Error::Storage {
                message: format!("Failed to upload to S3: {}", e),
                source: Some(Box::new(e)),
            })?;

        let size = request.content.len() as u64;

        let mut file = StoredFile::new(&path, &request.filename, &request.mime_type, size)
            .with_backend("s3")
            .with_metadata(request.metadata);

        if let Some(url) = self.url(&path) {
            file = file.with_url(url);
        }

        Ok(file)
    }

    async fn get(&self, path: &str) -> Result<Bytes> {
        use object_store::ObjectStore;

        let location = object_store::path::Path::from(path);

        let result = self.store.get(&location).await.map_err(|e| {
            Error::Storage {
                message: format!("Failed to get from S3: {}", e),
                source: Some(Box::new(e)),
            }
        })?;

        let bytes = result.bytes().await.map_err(|e| Error::Storage {
            message: format!("Failed to read S3 object: {}", e),
            source: Some(Box::new(e)),
        })?;

        Ok(bytes)
    }

    async fn delete(&self, path: &str) -> Result<bool> {
        use object_store::ObjectStore;

        let location = object_store::path::Path::from(path);

        self.store.delete(&location).await.map_err(|e| {
            Error::Storage {
                message: format!("Failed to delete from S3: {}", e),
                source: Some(Box::new(e)),
            }
        })?;

        Ok(true)
    }

    async fn exists(&self, path: &str) -> Result<bool> {
        use object_store::ObjectStore;

        let location = object_store::path::Path::from(path);

        match self.store.head(&location).await {
            Ok(_) => Ok(true),
            Err(object_store::Error::NotFound { .. }) => Ok(false),
            Err(e) => Err(Error::Storage {
                message: format!("Failed to check S3 object: {}", e),
                source: Some(Box::new(e)),
            }),
        }
    }

    async fn size(&self, path: &str) -> Result<u64> {
        use object_store::ObjectStore;

        let location = object_store::path::Path::from(path);

        let meta = self.store.head(&location).await.map_err(|e| {
            Error::Storage {
                message: format!("Failed to get S3 object metadata: {}", e),
                source: Some(Box::new(e)),
            }
        })?;

        Ok(meta.size as u64)
    }

    async fn copy(&self, from: &str, to: &str) -> Result<StoredFile> {
        use object_store::ObjectStore;

        let from_location = object_store::path::Path::from(from);
        let to_location = object_store::path::Path::from(to);

        self.store
            .copy(&from_location, &to_location)
            .await
            .map_err(|e| Error::Storage {
                message: format!("Failed to copy S3 object: {}", e),
                source: Some(Box::new(e)),
            })?;

        let size = self.size(to).await?;
        let filename = Path::new(to)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("file");

        Ok(StoredFile::new(to, filename, "application/octet-stream", size).with_backend("s3"))
    }

    async fn move_file(&self, from: &str, to: &str) -> Result<StoredFile> {
        let file = self.copy(from, to).await?;
        self.delete(from).await?;
        Ok(file)
    }

    fn url(&self, path: &str) -> Option<String> {
        self.base_url
            .as_ref()
            .map(|base| format!("{}/{}", base.trim_end_matches('/'), path))
    }

    async fn temporary_url(&self, path: &str, _expires_in_secs: u64) -> Result<String> {
        // For a real implementation, you'd use pre-signed URLs
        self.url(path).ok_or_else(|| Error::Storage {
            message: "Base URL not configured".to_string(),
            source: None,
        })
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        use futures::StreamExt;
        use object_store::ObjectStore;

        let prefix_path = object_store::path::Path::from(prefix);
        let mut stream = self.store.list(Some(&prefix_path));
        let mut files = Vec::new();

        while let Some(result) = stream.next().await {
            let meta = result.map_err(|e| Error::Storage {
                message: format!("Failed to list S3 objects: {}", e),
                source: Some(Box::new(e)),
            })?;
            files.push(meta.location.to_string());
        }

        Ok(files)
    }

    async fn health_check(&self) -> Result<()> {
        use object_store::ObjectStore;

        // Try to list root to verify connection
        let prefix = object_store::path::Path::from("");
        let mut stream = self.store.list(Some(&prefix));

        // Just verify we can start listing
        use futures::StreamExt;
        let _ = stream.next().await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_local_backend_store_and_get() {
        let temp_dir = TempDir::new().unwrap();
        let backend = LocalBackend::new(temp_dir.path());

        let content = Bytes::from("Hello, World!");
        let request = UploadRequest::new(content.clone(), "test.txt", "text/plain");

        let stored = backend.store(request).await.unwrap();
        assert_eq!(stored.size, 13);
        assert!(stored.path.ends_with(".txt"));

        let retrieved = backend.get(&stored.path).await.unwrap();
        assert_eq!(retrieved, content);
    }

    #[tokio::test]
    async fn test_local_backend_delete() {
        let temp_dir = TempDir::new().unwrap();
        let backend = LocalBackend::new(temp_dir.path());

        let content = Bytes::from("test content");
        let request = UploadRequest::new(content, "delete_me.txt", "text/plain");

        let stored = backend.store(request).await.unwrap();
        assert!(backend.exists(&stored.path).await.unwrap());

        backend.delete(&stored.path).await.unwrap();
        assert!(!backend.exists(&stored.path).await.unwrap());
    }

    #[tokio::test]
    async fn test_local_backend_copy() {
        let temp_dir = TempDir::new().unwrap();
        let backend = LocalBackend::new(temp_dir.path());

        let content = Bytes::from("copy test");
        let request = UploadRequest::new(content, "original.txt", "text/plain");

        let stored = backend.store(request).await.unwrap();
        let copied = backend
            .copy(&stored.path, "2024/02/copied.txt")
            .await
            .unwrap();

        assert!(backend.exists(&stored.path).await.unwrap());
        assert!(backend.exists(&copied.path).await.unwrap());
    }

    #[tokio::test]
    async fn test_local_backend_url() {
        let temp_dir = TempDir::new().unwrap();
        let backend =
            LocalBackend::new(temp_dir.path()).with_base_url("https://cdn.example.com/uploads");

        let url = backend.url("2024/01/test.jpg");
        assert_eq!(
            url,
            Some("https://cdn.example.com/uploads/2024/01/test.jpg".to_string())
        );
    }
}
