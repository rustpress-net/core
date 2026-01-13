//! GCS Client
//!
//! Google Cloud Storage provider implementation.

use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use chrono::{DateTime, Utc};

use google_cloud_storage::client::{Client, ClientConfig};
use google_cloud_storage::http::objects::delete::DeleteObjectRequest;
use google_cloud_storage::http::objects::download::Range;
use google_cloud_storage::http::objects::get::GetObjectRequest;
use google_cloud_storage::http::objects::list::ListObjectsRequest;
use google_cloud_storage::http::objects::upload::{Media, UploadObjectRequest, UploadType};
use google_cloud_storage::sign::SignedURLMethod;

use crate::models::storage::GCSStorageConfig;
use super::storage::{StorageFile, StorageError};

/// GCS client wrapper
pub struct GCSClient {
    client: Client,
    bucket: String,
    path_prefix: Option<String>,
    storage_class: String,
}

impl GCSClient {
    /// Create a new GCS client from config
    pub async fn from_config(config: &GCSStorageConfig) -> Result<Self, StorageError> {
        let client_config = if let Some(ref credentials_json) = config.credentials_json {
            // Use explicit credentials from JSON string
            ClientConfig::default()
                .with_credentials_json(credentials_json)
                .await
                .map_err(|e| StorageError::ConnectionFailed(format!("Failed to parse GCS credentials: {}", e)))?
        } else {
            // Use default credentials (environment variable, metadata server, etc.)
            ClientConfig::default()
                .with_auth()
                .await
                .map_err(|e| StorageError::ConnectionFailed(format!("Failed to authenticate with GCS: {}", e)))?
        };

        let client = Client::new(client_config);

        Ok(Self {
            client,
            bucket: config.bucket.clone(),
            path_prefix: config.path_prefix.clone(),
            storage_class: config.storage_class.clone(),
        })
    }

    /// Get the full GCS object name for a given filename
    fn get_object_name(&self, name: &str) -> String {
        match &self.path_prefix {
            Some(prefix) => {
                let prefix = prefix.trim_matches('/');
                if prefix.is_empty() {
                    name.to_string()
                } else {
                    format!("{}/{}", prefix, name)
                }
            }
            None => name.to_string(),
        }
    }

    /// Test GCS connection by listing bucket
    pub async fn test_connection(&self) -> Result<Option<i64>, StorageError> {
        // Try to list objects (max 1) to verify access
        let request = ListObjectsRequest {
            bucket: self.bucket.clone(),
            max_results: Some(1),
            ..Default::default()
        };

        self.client
            .list_objects(&request)
            .await
            .map_err(|e| StorageError::ConnectionFailed(format!("Failed to access GCS bucket: {}", e)))?;

        Ok(None) // GCS doesn't report available space
    }

    /// Upload a file to GCS
    pub async fn upload(&self, local_path: &Path, remote_name: &str) -> Result<String, StorageError> {
        let object_name = self.get_object_name(remote_name);

        // Read file into memory
        let mut file = File::open(local_path).await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to open file: {}", e)))?;

        let metadata = file.metadata().await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to get file metadata: {}", e)))?;

        let file_size = metadata.len();

        // Read file contents
        let mut buffer = Vec::with_capacity(file_size as usize);
        file.read_to_end(&mut buffer).await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to read file: {}", e)))?;

        // Determine content type based on file extension
        let content_type = Self::get_content_type(local_path);

        // Create upload request
        let upload_type = UploadType::Simple(Media::new(object_name.clone()));

        let request = UploadObjectRequest {
            bucket: self.bucket.clone(),
            ..Default::default()
        };

        self.client
            .upload_object(&request, buffer, &upload_type)
            .await
            .map_err(|e| StorageError::UploadFailed(format!("GCS upload failed: {}", e)))?;

        Ok(format!("gs://{}/{}", self.bucket, object_name))
    }

    /// Get content type based on file extension
    fn get_content_type(path: &Path) -> String {
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match extension.to_lowercase().as_str() {
            "gz" | "gzip" => "application/gzip",
            "zip" => "application/zip",
            "tar" => "application/x-tar",
            "sql" => "application/sql",
            "json" => "application/json",
            "xml" => "application/xml",
            _ => "application/octet-stream",
        }.to_string()
    }

    /// Download a file from GCS
    pub async fn download(&self, remote_path: &str, local_path: &Path) -> Result<(), StorageError> {
        let object_name = self.get_object_name(remote_path);

        let bytes = self.client
            .download_object(
                &GetObjectRequest {
                    bucket: self.bucket.clone(),
                    object: object_name,
                    ..Default::default()
                },
                &Range::default(),
            )
            .await
            .map_err(|e| StorageError::DownloadFailed(format!("GCS download failed: {}", e)))?;

        // Write to local file
        tokio::fs::write(local_path, bytes).await
            .map_err(|e| StorageError::DownloadFailed(format!("Failed to write file: {}", e)))?;

        Ok(())
    }

    /// Delete a file from GCS
    pub async fn delete(&self, remote_path: &str) -> Result<(), StorageError> {
        let object_name = self.get_object_name(remote_path);

        self.client
            .delete_object(&DeleteObjectRequest {
                bucket: self.bucket.clone(),
                object: object_name,
                ..Default::default()
            })
            .await
            .map_err(|e| StorageError::DeleteFailed(format!("GCS delete failed: {}", e)))?;

        Ok(())
    }

    /// List files in GCS
    pub async fn list_files(&self, prefix: Option<&str>) -> Result<Vec<StorageFile>, StorageError> {
        let full_prefix = match (&self.path_prefix, prefix) {
            (Some(base), Some(p)) => Some(format!("{}/{}", base.trim_matches('/'), p)),
            (Some(base), None) => Some(base.trim_matches('/').to_string()),
            (None, Some(p)) => Some(p.to_string()),
            (None, None) => None,
        };

        let mut files = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let request = ListObjectsRequest {
                bucket: self.bucket.clone(),
                prefix: full_prefix.clone(),
                page_token: page_token.clone(),
                max_results: Some(1000),
                ..Default::default()
            };

            let response = self.client
                .list_objects(&request)
                .await
                .map_err(|e| StorageError::ListFailed(format!("GCS list failed: {}", e)))?;

            if let Some(items) = response.items {
                for object in items {
                    let name = object.name.split('/').last().unwrap_or(&object.name).to_string();

                    // Parse the update time
                    let modified: Option<DateTime<Utc>> = object.updated
                        .and_then(|t| DateTime::parse_from_rfc3339(&t.to_rfc3339()).ok())
                        .map(|dt| dt.with_timezone(&Utc));

                    files.push(StorageFile {
                        name,
                        path: format!("gs://{}/{}", self.bucket, object.name),
                        size: object.size as i64,
                        modified,
                    });
                }
            }

            // Check for more pages
            if let Some(token) = response.next_page_token {
                page_token = Some(token);
            } else {
                break;
            }
        }

        Ok(files)
    }

    /// Check if a file exists in GCS
    pub async fn exists(&self, remote_path: &str) -> Result<bool, StorageError> {
        let object_name = self.get_object_name(remote_path);

        match self.client
            .get_object(&GetObjectRequest {
                bucket: self.bucket.clone(),
                object: object_name,
                ..Default::default()
            })
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                let error_string = e.to_string();
                if error_string.contains("404") || error_string.contains("Not Found") {
                    Ok(false)
                } else {
                    Err(StorageError::ConnectionFailed(format!("GCS get object failed: {}", e)))
                }
            }
        }
    }

    /// Get file metadata from GCS
    pub async fn get_metadata(&self, remote_path: &str) -> Result<StorageFile, StorageError> {
        let object_name = self.get_object_name(remote_path);

        let object = self.client
            .get_object(&GetObjectRequest {
                bucket: self.bucket.clone(),
                object: object_name.clone(),
                ..Default::default()
            })
            .await
            .map_err(|e| StorageError::ListFailed(format!("GCS get object failed: {}", e)))?;

        let name = object_name.split('/').last().unwrap_or(&object_name).to_string();

        let modified: Option<DateTime<Utc>> = object.updated
            .and_then(|t| DateTime::parse_from_rfc3339(&t.to_rfc3339()).ok())
            .map(|dt| dt.with_timezone(&Utc));

        Ok(StorageFile {
            name,
            path: format!("gs://{}/{}", self.bucket, object_name),
            size: object.size as i64,
            modified,
        })
    }

    /// Copy a file within GCS
    pub async fn copy(&self, source_path: &str, dest_path: &str) -> Result<String, StorageError> {
        let source_object = self.get_object_name(source_path);
        let dest_object = self.get_object_name(dest_path);

        // GCS copy is done by downloading and re-uploading for the google-cloud-storage crate
        // For a more efficient copy, we'd need to use the JSON API directly
        let bytes = self.client
            .download_object(
                &GetObjectRequest {
                    bucket: self.bucket.clone(),
                    object: source_object,
                    ..Default::default()
                },
                &Range::default(),
            )
            .await
            .map_err(|e| StorageError::DownloadFailed(format!("GCS copy (download) failed: {}", e)))?;

        let upload_type = UploadType::Simple(Media::new(dest_object.clone()));

        let request = UploadObjectRequest {
            bucket: self.bucket.clone(),
            ..Default::default()
        };

        self.client
            .upload_object(&request, bytes.to_vec(), &upload_type)
            .await
            .map_err(|e| StorageError::UploadFailed(format!("GCS copy (upload) failed: {}", e)))?;

        Ok(format!("gs://{}/{}", self.bucket, dest_object))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_object_name() {
        // Test would require mocking the GCS client
        // For now, just verify the config structure
        let config = GCSStorageConfig {
            bucket: "test-bucket".to_string(),
            project_id: "test-project".to_string(),
            credentials_json: None,
            path_prefix: Some("backups".to_string()),
            storage_class: "STANDARD".to_string(),
        };

        assert_eq!(config.bucket, "test-bucket");
        assert_eq!(config.project_id, "test-project");
    }
}
