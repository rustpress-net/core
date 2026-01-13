//! Dropbox Client
//!
//! Dropbox storage provider implementation using Dropbox API v2.

use std::path::Path;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::models::storage::DropboxStorageConfig;
use super::storage::{StorageFile, StorageError};

/// Dropbox API base URLs
const API_BASE: &str = "https://api.dropboxapi.com/2";
const CONTENT_BASE: &str = "https://content.dropboxapi.com/2";

/// Maximum file size for simple upload (150 MB)
const MAX_SIMPLE_UPLOAD_SIZE: u64 = 150 * 1024 * 1024;

/// Chunk size for upload sessions (8 MB)
const UPLOAD_CHUNK_SIZE: usize = 8 * 1024 * 1024;

/// Dropbox client wrapper
pub struct DropboxClient {
    client: Client,
    config: DropboxStorageConfig,
}

/// Dropbox API error response
#[derive(Debug, Deserialize)]
struct DropboxError {
    error_summary: String,
}

/// Space usage response
#[derive(Debug, Deserialize)]
struct SpaceUsage {
    used: u64,
    allocation: SpaceAllocation,
}

#[derive(Debug, Deserialize)]
#[serde(tag = ".tag")]
enum SpaceAllocation {
    #[serde(rename = "individual")]
    Individual { allocated: u64 },
    #[serde(rename = "team")]
    Team { allocated: u64 },
}

/// File metadata response
#[derive(Debug, Deserialize)]
struct FileMetadata {
    name: String,
    path_display: Option<String>,
    size: Option<u64>,
    server_modified: Option<String>,
    #[serde(rename = ".tag")]
    tag: String,
}

/// List folder response
#[derive(Debug, Deserialize)]
struct ListFolderResponse {
    entries: Vec<FileMetadata>,
    cursor: String,
    has_more: bool,
}

/// Upload session start response
#[derive(Debug, Deserialize)]
struct UploadSessionStartResult {
    session_id: String,
}

/// Dropbox API arg for upload
#[derive(Debug, Serialize)]
struct UploadArg {
    path: String,
    mode: String,
    autorename: bool,
    mute: bool,
}

/// Dropbox API arg for download
#[derive(Debug, Serialize)]
struct DownloadArg {
    path: String,
}

/// Dropbox API arg for delete
#[derive(Debug, Serialize)]
struct DeleteArg {
    path: String,
}

/// Dropbox API arg for list folder
#[derive(Debug, Serialize)]
struct ListFolderArg {
    path: String,
    recursive: bool,
    include_deleted: bool,
    include_media_info: bool,
}

/// Dropbox API arg for list folder continue
#[derive(Debug, Serialize)]
struct ListFolderContinueArg {
    cursor: String,
}

/// Dropbox API arg for create folder
#[derive(Debug, Serialize)]
struct CreateFolderArg {
    path: String,
    autorename: bool,
}

/// Dropbox API arg for get metadata
#[derive(Debug, Serialize)]
struct GetMetadataArg {
    path: String,
    include_media_info: bool,
    include_deleted: bool,
}

/// Upload session cursor
#[derive(Debug, Serialize)]
struct UploadSessionCursor {
    session_id: String,
    offset: u64,
}

/// Upload session finish arg
#[derive(Debug, Serialize)]
struct UploadSessionFinishArg {
    cursor: UploadSessionCursor,
    commit: UploadArg,
}

impl DropboxClient {
    /// Create a new Dropbox client from config
    pub fn new(config: &DropboxStorageConfig) -> Result<Self, StorageError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .map_err(|e| StorageError::ConnectionFailed(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            config: config.clone(),
        })
    }

    /// Get the full Dropbox path for a given filename
    fn get_dropbox_path(&self, name: &str) -> String {
        let prefix = self.config.path_prefix.as_deref().unwrap_or("");
        let prefix = prefix.trim_matches('/');

        if prefix.is_empty() {
            format!("/{}", name)
        } else {
            format!("/{}/{}", prefix, name)
        }
    }

    /// Get the base folder path
    fn get_base_path(&self) -> String {
        let prefix = self.config.path_prefix.as_deref().unwrap_or("");
        let prefix = prefix.trim_matches('/');

        if prefix.is_empty() {
            "".to_string() // Root folder in Dropbox API is empty string
        } else {
            format!("/{}", prefix)
        }
    }

    /// Make an API request with error handling
    async fn api_request<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> Result<R, StorageError> {
        let url = format!("{}{}", API_BASE, endpoint);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.access_token))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| StorageError::ConnectionFailed(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            if let Ok(error) = serde_json::from_str::<DropboxError>(&error_text) {
                return Err(StorageError::ConnectionFailed(format!(
                    "Dropbox API error ({}): {}",
                    status, error.error_summary
                )));
            }

            return Err(StorageError::ConnectionFailed(format!(
                "Dropbox API error ({}): {}",
                status, error_text
            )));
        }

        response.json::<R>().await
            .map_err(|e| StorageError::ConnectionFailed(format!("Failed to parse response: {}", e)))
    }

    /// Test Dropbox connection
    pub async fn test_connection(&self) -> Result<Option<i64>, StorageError> {
        let url = format!("{}/users/get_space_usage", API_BASE);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.access_token))
            .header("Content-Type", "application/json")
            .body("null")
            .send()
            .await
            .map_err(|e| StorageError::ConnectionFailed(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(StorageError::ConnectionFailed(format!(
                "Dropbox connection test failed ({}): {}",
                status, error_text
            )));
        }

        let usage: SpaceUsage = response.json().await
            .map_err(|e| StorageError::ConnectionFailed(format!("Failed to parse response: {}", e)))?;

        let allocated = match usage.allocation {
            SpaceAllocation::Individual { allocated } => allocated,
            SpaceAllocation::Team { allocated } => allocated,
        };

        let available = allocated.saturating_sub(usage.used) as i64;
        Ok(Some(available))
    }

    /// Ensure the base folder exists
    async fn ensure_folder_exists(&self, path: &str) -> Result<(), StorageError> {
        if path.is_empty() || path == "/" {
            return Ok(());
        }

        let arg = CreateFolderArg {
            path: path.to_string(),
            autorename: false,
        };

        let url = format!("{}/files/create_folder_v2", API_BASE);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.access_token))
            .header("Content-Type", "application/json")
            .json(&arg)
            .send()
            .await
            .map_err(|e| StorageError::ConnectionFailed(format!("Request failed: {}", e)))?;

        // Ignore "path/conflict/folder" error (folder already exists)
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            if !error_text.contains("path/conflict") {
                // Log but don't fail - the upload might still work
                tracing::warn!("Failed to create folder {}: {}", path, error_text);
            }
        }

        Ok(())
    }

    /// Upload a file to Dropbox (simple upload for files <= 150MB)
    async fn simple_upload(&self, data: Vec<u8>, remote_path: &str) -> Result<String, StorageError> {
        let arg = UploadArg {
            path: remote_path.to_string(),
            mode: "overwrite".to_string(),
            autorename: false,
            mute: true,
        };

        let url = format!("{}/files/upload", CONTENT_BASE);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.access_token))
            .header("Content-Type", "application/octet-stream")
            .header("Dropbox-API-Arg", serde_json::to_string(&arg).unwrap())
            .body(data)
            .send()
            .await
            .map_err(|e| StorageError::UploadFailed(format!("Upload request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(StorageError::UploadFailed(format!(
                "Dropbox upload failed ({}): {}",
                status, error_text
            )));
        }

        let metadata: FileMetadata = response.json().await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to parse upload response: {}", e)))?;

        Ok(metadata.path_display.unwrap_or_else(|| remote_path.to_string()))
    }

    /// Upload a file using upload session (for files > 150MB)
    async fn session_upload(&self, data: Vec<u8>, remote_path: &str) -> Result<String, StorageError> {
        // Start upload session
        let start_url = format!("{}/files/upload_session/start", CONTENT_BASE);

        let first_chunk_size = std::cmp::min(UPLOAD_CHUNK_SIZE, data.len());
        let first_chunk = &data[..first_chunk_size];

        let response = self.client
            .post(&start_url)
            .header("Authorization", format!("Bearer {}", self.config.access_token))
            .header("Content-Type", "application/octet-stream")
            .header("Dropbox-API-Arg", r#"{"close": false}"#)
            .body(first_chunk.to_vec())
            .send()
            .await
            .map_err(|e| StorageError::UploadFailed(format!("Session start failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(StorageError::UploadFailed(format!(
                "Session start failed: {}",
                error_text
            )));
        }

        let session: UploadSessionStartResult = response.json().await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to parse session response: {}", e)))?;

        let mut offset = first_chunk_size;

        // Append remaining chunks
        let append_url = format!("{}/files/upload_session/append_v2", CONTENT_BASE);

        while offset < data.len() {
            let chunk_end = std::cmp::min(offset + UPLOAD_CHUNK_SIZE, data.len());
            let chunk = &data[offset..chunk_end];

            let cursor = UploadSessionCursor {
                session_id: session.session_id.clone(),
                offset: offset as u64,
            };

            let arg = serde_json::json!({
                "cursor": cursor,
                "close": false
            });

            let response = self.client
                .post(&append_url)
                .header("Authorization", format!("Bearer {}", self.config.access_token))
                .header("Content-Type", "application/octet-stream")
                .header("Dropbox-API-Arg", serde_json::to_string(&arg).unwrap())
                .body(chunk.to_vec())
                .send()
                .await
                .map_err(|e| StorageError::UploadFailed(format!("Session append failed: {}", e)))?;

            if !response.status().is_success() {
                let error_text = response.text().await.unwrap_or_default();
                return Err(StorageError::UploadFailed(format!(
                    "Session append failed: {}",
                    error_text
                )));
            }

            offset = chunk_end;
        }

        // Finish upload session
        let finish_url = format!("{}/files/upload_session/finish", CONTENT_BASE);

        let finish_arg = UploadSessionFinishArg {
            cursor: UploadSessionCursor {
                session_id: session.session_id,
                offset: data.len() as u64,
            },
            commit: UploadArg {
                path: remote_path.to_string(),
                mode: "overwrite".to_string(),
                autorename: false,
                mute: true,
            },
        };

        let response = self.client
            .post(&finish_url)
            .header("Authorization", format!("Bearer {}", self.config.access_token))
            .header("Content-Type", "application/octet-stream")
            .header("Dropbox-API-Arg", serde_json::to_string(&finish_arg).unwrap())
            .send()
            .await
            .map_err(|e| StorageError::UploadFailed(format!("Session finish failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(StorageError::UploadFailed(format!(
                "Session finish failed: {}",
                error_text
            )));
        }

        let metadata: FileMetadata = response.json().await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to parse finish response: {}", e)))?;

        Ok(metadata.path_display.unwrap_or_else(|| remote_path.to_string()))
    }

    /// Upload a file to Dropbox
    pub async fn upload(&self, local_path: &Path, remote_name: &str) -> Result<String, StorageError> {
        // Ensure base folder exists
        let base_path = self.get_base_path();
        self.ensure_folder_exists(&base_path).await?;

        // Read file
        let mut file = File::open(local_path).await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to open file: {}", e)))?;

        let metadata = file.metadata().await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to get file metadata: {}", e)))?;

        let file_size = metadata.len();
        let mut buffer = Vec::with_capacity(file_size as usize);
        file.read_to_end(&mut buffer).await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to read file: {}", e)))?;

        let remote_path = self.get_dropbox_path(remote_name);

        // Use simple upload for small files, session upload for large files
        if file_size <= MAX_SIMPLE_UPLOAD_SIZE {
            self.simple_upload(buffer, &remote_path).await
        } else {
            self.session_upload(buffer, &remote_path).await
        }
    }

    /// Download a file from Dropbox
    pub async fn download(&self, remote_path: &str, local_path: &Path) -> Result<(), StorageError> {
        let full_remote_path = self.get_dropbox_path(remote_path);

        let arg = DownloadArg {
            path: full_remote_path.clone(),
        };

        let url = format!("{}/files/download", CONTENT_BASE);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.access_token))
            .header("Dropbox-API-Arg", serde_json::to_string(&arg).unwrap())
            .send()
            .await
            .map_err(|e| StorageError::DownloadFailed(format!("Download request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(StorageError::DownloadFailed(format!(
                "Dropbox download failed ({}): {}",
                status, error_text
            )));
        }

        let bytes = response.bytes().await
            .map_err(|e| StorageError::DownloadFailed(format!("Failed to read response: {}", e)))?;

        // Ensure parent directory exists
        if let Some(parent) = local_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| StorageError::DownloadFailed(format!("Failed to create directory: {}", e)))?;
        }

        tokio::fs::write(local_path, bytes).await
            .map_err(|e| StorageError::DownloadFailed(format!("Failed to write file: {}", e)))?;

        Ok(())
    }

    /// Delete a file from Dropbox
    pub async fn delete(&self, remote_path: &str) -> Result<(), StorageError> {
        let full_remote_path = self.get_dropbox_path(remote_path);

        let arg = DeleteArg {
            path: full_remote_path,
        };

        let url = format!("{}/files/delete_v2", API_BASE);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.access_token))
            .header("Content-Type", "application/json")
            .json(&arg)
            .send()
            .await
            .map_err(|e| StorageError::DeleteFailed(format!("Delete request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(StorageError::DeleteFailed(format!(
                "Dropbox delete failed ({}): {}",
                status, error_text
            )));
        }

        Ok(())
    }

    /// List files in Dropbox folder
    pub async fn list_files(&self, prefix: Option<&str>) -> Result<Vec<StorageFile>, StorageError> {
        let search_path = match prefix {
            Some(p) => self.get_dropbox_path(p),
            None => self.get_base_path(),
        };

        let arg = ListFolderArg {
            path: search_path.clone(),
            recursive: false,
            include_deleted: false,
            include_media_info: false,
        };

        let url = format!("{}/files/list_folder", API_BASE);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.access_token))
            .header("Content-Type", "application/json")
            .json(&arg)
            .send()
            .await
            .map_err(|e| StorageError::ListFailed(format!("List request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            // Handle "path/not_found" as empty folder
            if error_text.contains("path/not_found") {
                return Ok(Vec::new());
            }

            return Err(StorageError::ListFailed(format!(
                "Dropbox list failed ({}): {}",
                status, error_text
            )));
        }

        let mut result: ListFolderResponse = response.json().await
            .map_err(|e| StorageError::ListFailed(format!("Failed to parse list response: {}", e)))?;

        let mut files = Vec::new();

        // Process entries
        for entry in &result.entries {
            if entry.tag == "file" {
                files.push(self.metadata_to_storage_file(entry));
            }
        }

        // Handle pagination
        while result.has_more {
            let continue_arg = ListFolderContinueArg {
                cursor: result.cursor.clone(),
            };

            let continue_url = format!("{}/files/list_folder/continue", API_BASE);

            let response = self.client
                .post(&continue_url)
                .header("Authorization", format!("Bearer {}", self.config.access_token))
                .header("Content-Type", "application/json")
                .json(&continue_arg)
                .send()
                .await
                .map_err(|e| StorageError::ListFailed(format!("List continue failed: {}", e)))?;

            if !response.status().is_success() {
                break;
            }

            result = response.json().await
                .map_err(|e| StorageError::ListFailed(format!("Failed to parse continue response: {}", e)))?;

            for entry in &result.entries {
                if entry.tag == "file" {
                    files.push(self.metadata_to_storage_file(entry));
                }
            }
        }

        Ok(files)
    }

    /// Convert Dropbox metadata to StorageFile
    fn metadata_to_storage_file(&self, metadata: &FileMetadata) -> StorageFile {
        let modified = metadata.server_modified.as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        StorageFile {
            name: metadata.name.clone(),
            path: metadata.path_display.clone().unwrap_or_else(|| metadata.name.clone()),
            size: metadata.size.unwrap_or(0) as i64,
            modified,
        }
    }

    /// Check if a file exists on Dropbox
    pub async fn exists(&self, remote_path: &str) -> Result<bool, StorageError> {
        let full_remote_path = self.get_dropbox_path(remote_path);

        let arg = GetMetadataArg {
            path: full_remote_path,
            include_media_info: false,
            include_deleted: false,
        };

        let url = format!("{}/files/get_metadata", API_BASE);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.access_token))
            .header("Content-Type", "application/json")
            .json(&arg)
            .send()
            .await
            .map_err(|e| StorageError::ConnectionFailed(format!("Metadata request failed: {}", e)))?;

        Ok(response.status().is_success())
    }

    /// Get file metadata from Dropbox
    pub async fn get_metadata(&self, remote_path: &str) -> Result<StorageFile, StorageError> {
        let full_remote_path = self.get_dropbox_path(remote_path);

        let arg = GetMetadataArg {
            path: full_remote_path.clone(),
            include_media_info: false,
            include_deleted: false,
        };

        let url = format!("{}/files/get_metadata", API_BASE);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.access_token))
            .header("Content-Type", "application/json")
            .json(&arg)
            .send()
            .await
            .map_err(|e| StorageError::ListFailed(format!("Metadata request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(StorageError::ListFailed(format!(
                "Dropbox get metadata failed ({}): {}",
                status, error_text
            )));
        }

        let metadata: FileMetadata = response.json().await
            .map_err(|e| StorageError::ListFailed(format!("Failed to parse metadata response: {}", e)))?;

        Ok(self.metadata_to_storage_file(&metadata))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dropbox_client_creation() {
        let config = DropboxStorageConfig {
            access_token: "test_token".to_string(),
            refresh_token: None,
            path_prefix: Some("/backups".to_string()),
        };

        let client = DropboxClient::new(&config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_get_dropbox_path() {
        let config = DropboxStorageConfig {
            access_token: "test_token".to_string(),
            refresh_token: None,
            path_prefix: Some("/backups".to_string()),
        };

        let client = DropboxClient::new(&config).unwrap();
        assert_eq!(client.get_dropbox_path("test.tar.gz"), "/backups/test.tar.gz");
    }

    #[test]
    fn test_get_dropbox_path_no_prefix() {
        let config = DropboxStorageConfig {
            access_token: "test_token".to_string(),
            refresh_token: None,
            path_prefix: None,
        };

        let client = DropboxClient::new(&config).unwrap();
        assert_eq!(client.get_dropbox_path("test.tar.gz"), "/test.tar.gz");
    }

    #[test]
    fn test_get_base_path() {
        let config = DropboxStorageConfig {
            access_token: "test_token".to_string(),
            refresh_token: None,
            path_prefix: Some("/backups".to_string()),
        };

        let client = DropboxClient::new(&config).unwrap();
        assert_eq!(client.get_base_path(), "/backups");
    }

    #[test]
    fn test_get_base_path_empty() {
        let config = DropboxStorageConfig {
            access_token: "test_token".to_string(),
            refresh_token: None,
            path_prefix: None,
        };

        let client = DropboxClient::new(&config).unwrap();
        assert_eq!(client.get_base_path(), "");
    }
}
