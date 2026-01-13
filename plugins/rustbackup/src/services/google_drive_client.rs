//! Google Drive Client
//!
//! Google Drive storage provider implementation using Google Drive API v3.

use std::path::Path;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::models::storage::GoogleDriveStorageConfig;
use super::storage::{StorageFile, StorageError};

/// Google Drive API base URL
const API_BASE: &str = "https://www.googleapis.com/drive/v3";
const UPLOAD_BASE: &str = "https://www.googleapis.com/upload/drive/v3";

/// Maximum file size for simple upload (5 MB)
const MAX_SIMPLE_UPLOAD_SIZE: u64 = 5 * 1024 * 1024;

/// Chunk size for resumable uploads (5 MB)
const UPLOAD_CHUNK_SIZE: usize = 5 * 1024 * 1024;

/// Google Drive client wrapper
pub struct GoogleDriveClient {
    client: Client,
    config: GoogleDriveStorageConfig,
}

/// Google Drive API error response
#[derive(Debug, Deserialize)]
struct DriveError {
    error: DriveErrorDetails,
}

#[derive(Debug, Deserialize)]
struct DriveErrorDetails {
    message: String,
    code: Option<i32>,
}

/// About response (for quota info)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AboutResponse {
    storage_quota: Option<StorageQuota>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StorageQuota {
    limit: Option<String>,
    usage: Option<String>,
}

/// File metadata response
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DriveFile {
    id: String,
    name: String,
    mime_type: Option<String>,
    size: Option<String>,
    modified_time: Option<String>,
    parents: Option<Vec<String>>,
}

/// List files response
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListFilesResponse {
    files: Vec<DriveFile>,
    next_page_token: Option<String>,
}

/// File metadata for upload
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FileMetadata {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parents: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mime_type: Option<String>,
}

impl GoogleDriveClient {
    /// Create a new Google Drive client from config
    pub fn new(config: &GoogleDriveStorageConfig) -> Result<Self, StorageError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .map_err(|e| StorageError::ConnectionFailed(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            config: config.clone(),
        })
    }

    /// Get the parent folder ID (uses configured folder_id or root)
    fn get_parent_folder(&self) -> String {
        self.config.folder_id.clone().unwrap_or_else(|| "root".to_string())
    }

    /// Test Google Drive connection
    pub async fn test_connection(&self) -> Result<Option<i64>, StorageError> {
        let url = format!("{}/about?fields=storageQuota", API_BASE);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.access_token))
            .send()
            .await
            .map_err(|e| StorageError::ConnectionFailed(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(StorageError::ConnectionFailed(format!(
                "Google Drive connection test failed ({}): {}",
                status, error_text
            )));
        }

        let about: AboutResponse = response.json().await
            .map_err(|e| StorageError::ConnectionFailed(format!("Failed to parse response: {}", e)))?;

        if let Some(quota) = about.storage_quota {
            if let (Some(limit), Some(usage)) = (quota.limit, quota.usage) {
                let limit: i64 = limit.parse().unwrap_or(0);
                let usage: i64 = usage.parse().unwrap_or(0);
                return Ok(Some(limit - usage));
            }
        }

        Ok(None)
    }

    /// Find a file by name in the parent folder
    async fn find_file_by_name(&self, name: &str, parent_id: &str) -> Result<Option<DriveFile>, StorageError> {
        let query = format!(
            "name = '{}' and '{}' in parents and trashed = false",
            name.replace('\'', "\\'"),
            parent_id
        );

        let url = format!(
            "{}/files?q={}&fields=files(id,name,mimeType,size,modifiedTime,parents)",
            API_BASE,
            urlencoding::encode(&query)
        );

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.access_token))
            .send()
            .await
            .map_err(|e| StorageError::ConnectionFailed(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(StorageError::ListFailed(format!(
                "Failed to search for file: {}",
                error_text
            )));
        }

        let result: ListFilesResponse = response.json().await
            .map_err(|e| StorageError::ListFailed(format!("Failed to parse response: {}", e)))?;

        Ok(result.files.into_iter().next())
    }

    /// Create or get folder by path (returns folder ID)
    async fn ensure_folder_exists(&self, folder_name: &str, parent_id: &str) -> Result<String, StorageError> {
        // Check if folder already exists
        let query = format!(
            "name = '{}' and '{}' in parents and mimeType = 'application/vnd.google-apps.folder' and trashed = false",
            folder_name.replace('\'', "\\'"),
            parent_id
        );

        let url = format!(
            "{}/files?q={}&fields=files(id,name)",
            API_BASE,
            urlencoding::encode(&query)
        );

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.access_token))
            .send()
            .await
            .map_err(|e| StorageError::ConnectionFailed(format!("Request failed: {}", e)))?;

        if response.status().is_success() {
            let result: ListFilesResponse = response.json().await
                .map_err(|e| StorageError::ConnectionFailed(format!("Failed to parse response: {}", e)))?;

            if let Some(folder) = result.files.into_iter().next() {
                return Ok(folder.id);
            }
        }

        // Create the folder
        let metadata = FileMetadata {
            name: folder_name.to_string(),
            parents: Some(vec![parent_id.to_string()]),
            mime_type: Some("application/vnd.google-apps.folder".to_string()),
        };

        let url = format!("{}/files", API_BASE);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.access_token))
            .header("Content-Type", "application/json")
            .json(&metadata)
            .send()
            .await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to create folder: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(StorageError::UploadFailed(format!(
                "Failed to create folder: {}",
                error_text
            )));
        }

        let folder: DriveFile = response.json().await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to parse response: {}", e)))?;

        Ok(folder.id)
    }

    /// Ensure nested folder path exists, returns the final folder ID
    async fn ensure_folder_path(&self, path: &str) -> Result<String, StorageError> {
        let mut current_parent = self.get_parent_folder();

        for part in path.split('/').filter(|p| !p.is_empty()) {
            current_parent = self.ensure_folder_exists(part, &current_parent).await?;
        }

        Ok(current_parent)
    }

    /// Simple upload for small files (< 5MB)
    async fn simple_upload(&self, data: Vec<u8>, name: &str, parent_id: &str) -> Result<DriveFile, StorageError> {
        // Use multipart upload to include metadata
        let metadata = FileMetadata {
            name: name.to_string(),
            parents: Some(vec![parent_id.to_string()]),
            mime_type: Some("application/octet-stream".to_string()),
        };

        let metadata_json = serde_json::to_string(&metadata)
            .map_err(|e| StorageError::UploadFailed(format!("Failed to serialize metadata: {}", e)))?;

        // Build multipart request manually
        let boundary = format!("----RustBackup{}", uuid::Uuid::new_v4());

        let mut body = Vec::new();

        // Metadata part
        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(b"Content-Type: application/json; charset=UTF-8\r\n\r\n");
        body.extend_from_slice(metadata_json.as_bytes());
        body.extend_from_slice(b"\r\n");

        // File content part
        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
        body.extend_from_slice(&data);
        body.extend_from_slice(b"\r\n");

        // End boundary
        body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());

        let url = format!(
            "{}/files?uploadType=multipart&fields=id,name,mimeType,size,modifiedTime,parents",
            UPLOAD_BASE
        );

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.access_token))
            .header("Content-Type", format!("multipart/related; boundary={}", boundary))
            .body(body)
            .send()
            .await
            .map_err(|e| StorageError::UploadFailed(format!("Upload request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(StorageError::UploadFailed(format!(
                "Google Drive upload failed ({}): {}",
                status, error_text
            )));
        }

        response.json().await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to parse upload response: {}", e)))
    }

    /// Resumable upload for large files (> 5MB)
    async fn resumable_upload(&self, data: Vec<u8>, name: &str, parent_id: &str) -> Result<DriveFile, StorageError> {
        let metadata = FileMetadata {
            name: name.to_string(),
            parents: Some(vec![parent_id.to_string()]),
            mime_type: Some("application/octet-stream".to_string()),
        };

        // Step 1: Initiate resumable upload
        let url = format!(
            "{}/files?uploadType=resumable",
            UPLOAD_BASE
        );

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.access_token))
            .header("Content-Type", "application/json; charset=UTF-8")
            .header("X-Upload-Content-Type", "application/octet-stream")
            .header("X-Upload-Content-Length", data.len().to_string())
            .json(&metadata)
            .send()
            .await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to initiate upload: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(StorageError::UploadFailed(format!(
                "Failed to initiate resumable upload: {}",
                error_text
            )));
        }

        let upload_url = response
            .headers()
            .get("location")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
            .ok_or_else(|| StorageError::UploadFailed("No upload URL returned".to_string()))?;

        // Step 2: Upload in chunks
        let total_size = data.len();
        let mut offset = 0;

        while offset < total_size {
            let chunk_end = std::cmp::min(offset + UPLOAD_CHUNK_SIZE, total_size);
            let chunk = &data[offset..chunk_end];
            let is_final = chunk_end == total_size;

            let content_range = format!(
                "bytes {}-{}/{}",
                offset,
                chunk_end - 1,
                total_size
            );

            let response = self.client
                .put(&upload_url)
                .header("Content-Length", chunk.len().to_string())
                .header("Content-Range", content_range)
                .body(chunk.to_vec())
                .send()
                .await
                .map_err(|e| StorageError::UploadFailed(format!("Chunk upload failed: {}", e)))?;

            if is_final {
                if !response.status().is_success() {
                    let error_text = response.text().await.unwrap_or_default();
                    return Err(StorageError::UploadFailed(format!(
                        "Final chunk upload failed: {}",
                        error_text
                    )));
                }

                return response.json().await
                    .map_err(|e| StorageError::UploadFailed(format!("Failed to parse response: {}", e)));
            } else {
                // For non-final chunks, expect 308 Resume Incomplete
                if response.status().as_u16() != 308 && !response.status().is_success() {
                    let error_text = response.text().await.unwrap_or_default();
                    return Err(StorageError::UploadFailed(format!(
                        "Chunk upload failed: {}",
                        error_text
                    )));
                }
            }

            offset = chunk_end;
        }

        Err(StorageError::UploadFailed("Upload completed but no response received".to_string()))
    }

    /// Upload a file to Google Drive
    pub async fn upload(&self, local_path: &Path, remote_name: &str) -> Result<String, StorageError> {
        // Read file
        let mut file = File::open(local_path).await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to open file: {}", e)))?;

        let metadata = file.metadata().await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to get file metadata: {}", e)))?;

        let file_size = metadata.len();
        let mut buffer = Vec::with_capacity(file_size as usize);
        file.read_to_end(&mut buffer).await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to read file: {}", e)))?;

        // Determine parent folder
        let (parent_id, file_name) = if remote_name.contains('/') {
            // Has path components - create folder structure
            let parts: Vec<&str> = remote_name.rsplitn(2, '/').collect();
            let file_name = parts[0];
            let folder_path = parts.get(1).unwrap_or(&"");
            let parent_id = self.ensure_folder_path(folder_path).await?;
            (parent_id, file_name.to_string())
        } else {
            (self.get_parent_folder(), remote_name.to_string())
        };

        // Check if file already exists and delete it
        if let Ok(Some(existing)) = self.find_file_by_name(&file_name, &parent_id).await {
            self.delete_by_id(&existing.id).await?;
        }

        // Upload based on file size
        let result = if file_size <= MAX_SIMPLE_UPLOAD_SIZE {
            self.simple_upload(buffer, &file_name, &parent_id).await?
        } else {
            self.resumable_upload(buffer, &file_name, &parent_id).await?
        };

        Ok(format!("gdrive://{}", result.id))
    }

    /// Delete a file by ID
    async fn delete_by_id(&self, file_id: &str) -> Result<(), StorageError> {
        let url = format!("{}/files/{}", API_BASE, file_id);

        let response = self.client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.config.access_token))
            .send()
            .await
            .map_err(|e| StorageError::DeleteFailed(format!("Delete request failed: {}", e)))?;

        if !response.status().is_success() && response.status().as_u16() != 404 {
            let error_text = response.text().await.unwrap_or_default();
            return Err(StorageError::DeleteFailed(format!(
                "Google Drive delete failed: {}",
                error_text
            )));
        }

        Ok(())
    }

    /// Download a file from Google Drive
    pub async fn download(&self, remote_path: &str, local_path: &Path) -> Result<(), StorageError> {
        // remote_path could be a file ID or a file name
        let file_id = if remote_path.starts_with("gdrive://") {
            remote_path.trim_start_matches("gdrive://").to_string()
        } else {
            // Try to find file by name in parent folder
            let parent_id = self.get_parent_folder();
            match self.find_file_by_name(remote_path, &parent_id).await? {
                Some(file) => file.id,
                None => return Err(StorageError::DownloadFailed(format!(
                    "File not found: {}",
                    remote_path
                ))),
            }
        };

        let url = format!("{}/files/{}?alt=media", API_BASE, file_id);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.access_token))
            .send()
            .await
            .map_err(|e| StorageError::DownloadFailed(format!("Download request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(StorageError::DownloadFailed(format!(
                "Google Drive download failed ({}): {}",
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

    /// Delete a file from Google Drive
    pub async fn delete(&self, remote_path: &str) -> Result<(), StorageError> {
        // remote_path could be a file ID or a file name
        let file_id = if remote_path.starts_with("gdrive://") {
            remote_path.trim_start_matches("gdrive://").to_string()
        } else {
            // Try to find file by name in parent folder
            let parent_id = self.get_parent_folder();
            match self.find_file_by_name(remote_path, &parent_id).await? {
                Some(file) => file.id,
                None => return Ok(()), // File doesn't exist, nothing to delete
            }
        };

        self.delete_by_id(&file_id).await
    }

    /// List files in Google Drive folder
    pub async fn list_files(&self, prefix: Option<&str>) -> Result<Vec<StorageFile>, StorageError> {
        let parent_id = match prefix {
            Some(folder_path) if !folder_path.is_empty() => {
                // Try to find the folder
                let mut current_parent = self.get_parent_folder();
                for part in folder_path.split('/').filter(|p| !p.is_empty()) {
                    let query = format!(
                        "name = '{}' and '{}' in parents and mimeType = 'application/vnd.google-apps.folder' and trashed = false",
                        part.replace('\'', "\\'"),
                        current_parent
                    );

                    let url = format!(
                        "{}/files?q={}&fields=files(id)",
                        API_BASE,
                        urlencoding::encode(&query)
                    );

                    let response = self.client
                        .get(&url)
                        .header("Authorization", format!("Bearer {}", self.config.access_token))
                        .send()
                        .await
                        .map_err(|e| StorageError::ListFailed(format!("Request failed: {}", e)))?;

                    if response.status().is_success() {
                        let result: ListFilesResponse = response.json().await
                            .map_err(|e| StorageError::ListFailed(format!("Failed to parse response: {}", e)))?;

                        if let Some(folder) = result.files.into_iter().next() {
                            current_parent = folder.id;
                        } else {
                            // Folder not found, return empty list
                            return Ok(Vec::new());
                        }
                    } else {
                        return Ok(Vec::new());
                    }
                }
                current_parent
            }
            _ => self.get_parent_folder(),
        };

        let mut all_files = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let query = format!(
                "'{}' in parents and mimeType != 'application/vnd.google-apps.folder' and trashed = false",
                parent_id
            );

            let mut url = format!(
                "{}/files?q={}&fields=files(id,name,mimeType,size,modifiedTime,parents),nextPageToken&pageSize=1000",
                API_BASE,
                urlencoding::encode(&query)
            );

            if let Some(token) = &page_token {
                url = format!("{}&pageToken={}", url, token);
            }

            let response = self.client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.config.access_token))
                .send()
                .await
                .map_err(|e| StorageError::ListFailed(format!("Request failed: {}", e)))?;

            if !response.status().is_success() {
                let error_text = response.text().await.unwrap_or_default();
                return Err(StorageError::ListFailed(format!(
                    "Google Drive list failed: {}",
                    error_text
                )));
            }

            let result: ListFilesResponse = response.json().await
                .map_err(|e| StorageError::ListFailed(format!("Failed to parse response: {}", e)))?;

            for file in result.files {
                all_files.push(self.drive_file_to_storage_file(&file));
            }

            match result.next_page_token {
                Some(token) => page_token = Some(token),
                None => break,
            }
        }

        Ok(all_files)
    }

    /// Convert DriveFile to StorageFile
    fn drive_file_to_storage_file(&self, file: &DriveFile) -> StorageFile {
        let modified = file.modified_time.as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let size = file.size.as_ref()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);

        StorageFile {
            name: file.name.clone(),
            path: format!("gdrive://{}", file.id),
            size,
            modified,
        }
    }

    /// Check if a file exists on Google Drive
    pub async fn exists(&self, remote_path: &str) -> Result<bool, StorageError> {
        if remote_path.starts_with("gdrive://") {
            let file_id = remote_path.trim_start_matches("gdrive://");
            let url = format!("{}/files/{}?fields=id", API_BASE, file_id);

            let response = self.client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.config.access_token))
                .send()
                .await
                .map_err(|e| StorageError::ConnectionFailed(format!("Request failed: {}", e)))?;

            return Ok(response.status().is_success());
        }

        // Try to find file by name
        let parent_id = self.get_parent_folder();
        match self.find_file_by_name(remote_path, &parent_id).await? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    /// Get file metadata from Google Drive
    pub async fn get_metadata(&self, remote_path: &str) -> Result<StorageFile, StorageError> {
        let file_id = if remote_path.starts_with("gdrive://") {
            remote_path.trim_start_matches("gdrive://").to_string()
        } else {
            let parent_id = self.get_parent_folder();
            match self.find_file_by_name(remote_path, &parent_id).await? {
                Some(file) => file.id,
                None => return Err(StorageError::ListFailed(format!(
                    "File not found: {}",
                    remote_path
                ))),
            }
        };

        let url = format!(
            "{}/files/{}?fields=id,name,mimeType,size,modifiedTime,parents",
            API_BASE,
            file_id
        );

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.access_token))
            .send()
            .await
            .map_err(|e| StorageError::ListFailed(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(StorageError::ListFailed(format!(
                "Google Drive get metadata failed: {}",
                error_text
            )));
        }

        let file: DriveFile = response.json().await
            .map_err(|e| StorageError::ListFailed(format!("Failed to parse response: {}", e)))?;

        Ok(self.drive_file_to_storage_file(&file))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_drive_client_creation() {
        let config = GoogleDriveStorageConfig {
            access_token: "test_token".to_string(),
            refresh_token: None,
            folder_id: Some("folder123".to_string()),
        };

        let client = GoogleDriveClient::new(&config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_get_parent_folder() {
        let config = GoogleDriveStorageConfig {
            access_token: "test_token".to_string(),
            refresh_token: None,
            folder_id: Some("folder123".to_string()),
        };

        let client = GoogleDriveClient::new(&config).unwrap();
        assert_eq!(client.get_parent_folder(), "folder123");
    }

    #[test]
    fn test_get_parent_folder_default() {
        let config = GoogleDriveStorageConfig {
            access_token: "test_token".to_string(),
            refresh_token: None,
            folder_id: None,
        };

        let client = GoogleDriveClient::new(&config).unwrap();
        assert_eq!(client.get_parent_folder(), "root");
    }
}
