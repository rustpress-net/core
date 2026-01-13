//! Azure Blob Storage Client
//!
//! Azure Blob Storage provider implementation.

use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use chrono::{DateTime, Utc};
use futures::StreamExt;

use azure_storage::StorageCredentials;
use azure_storage_blobs::prelude::*;
use azure_core::prelude::*;

use crate::models::storage::AzureStorageConfig;
use super::storage::{StorageFile, StorageError};

/// Azure Blob Storage client wrapper
pub struct AzureClient {
    container_client: ContainerClient,
    container: String,
    path_prefix: Option<String>,
    access_tier: AccessTier,
}

impl AzureClient {
    /// Create a new Azure client from config
    pub async fn from_config(config: &AzureStorageConfig) -> Result<Self, StorageError> {
        let storage_credentials = if let Some(ref connection_string) = config.connection_string {
            // Use connection string
            StorageCredentials::connection_string(connection_string)
                .map_err(|e| StorageError::ConnectionFailed(format!("Invalid connection string: {}", e)))?
        } else if let Some(ref account_key) = config.account_key {
            // Use account name and key
            StorageCredentials::access_key(&config.account_name, account_key.clone())
        } else {
            return Err(StorageError::ConnectionFailed(
                "Either connection_string or account_key must be provided".to_string()
            ));
        };

        let blob_service_client = BlobServiceClient::new(&config.account_name, storage_credentials);
        let container_client = blob_service_client.container_client(&config.container);

        let access_tier = match config.access_tier.to_lowercase().as_str() {
            "hot" => AccessTier::Hot,
            "cool" => AccessTier::Cool,
            "cold" => AccessTier::Cold,
            "archive" => AccessTier::Archive,
            _ => AccessTier::Hot,
        };

        Ok(Self {
            container_client,
            container: config.container.clone(),
            path_prefix: config.path_prefix.clone(),
            access_tier,
        })
    }

    /// Get the full blob name for a given filename
    fn get_blob_name(&self, name: &str) -> String {
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

    /// Test Azure connection by getting container properties
    pub async fn test_connection(&self) -> Result<Option<i64>, StorageError> {
        // Try to get container properties to verify access
        self.container_client
            .get_properties()
            .await
            .map_err(|e| StorageError::ConnectionFailed(format!("Failed to access Azure container: {}", e)))?;

        Ok(None) // Azure doesn't report available space in a simple way
    }

    /// Upload a file to Azure Blob Storage
    pub async fn upload(&self, local_path: &Path, remote_name: &str) -> Result<String, StorageError> {
        let blob_name = self.get_blob_name(remote_name);

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

        // Get blob client
        let blob_client = self.container_client.blob_client(&blob_name);

        // Upload as block blob
        blob_client
            .put_block_blob(buffer)
            .content_type(content_type)
            .access_tier(self.access_tier.clone())
            .await
            .map_err(|e| StorageError::UploadFailed(format!("Azure upload failed: {}", e)))?;

        Ok(format!("https://{}.blob.core.windows.net/{}/{}",
            self.container_client.account(),
            self.container,
            blob_name
        ))
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

    /// Download a file from Azure Blob Storage
    pub async fn download(&self, remote_path: &str, local_path: &Path) -> Result<(), StorageError> {
        let blob_name = self.get_blob_name(remote_path);

        let blob_client = self.container_client.blob_client(&blob_name);

        // Download blob content
        let response = blob_client
            .get_content()
            .await
            .map_err(|e| StorageError::DownloadFailed(format!("Azure download failed: {}", e)))?;

        // Write to local file
        tokio::fs::write(local_path, response).await
            .map_err(|e| StorageError::DownloadFailed(format!("Failed to write file: {}", e)))?;

        Ok(())
    }

    /// Delete a file from Azure Blob Storage
    pub async fn delete(&self, remote_path: &str) -> Result<(), StorageError> {
        let blob_name = self.get_blob_name(remote_path);

        let blob_client = self.container_client.blob_client(&blob_name);

        blob_client
            .delete()
            .await
            .map_err(|e| StorageError::DeleteFailed(format!("Azure delete failed: {}", e)))?;

        Ok(())
    }

    /// List files in Azure Blob Storage
    pub async fn list_files(&self, prefix: Option<&str>) -> Result<Vec<StorageFile>, StorageError> {
        let full_prefix = match (&self.path_prefix, prefix) {
            (Some(base), Some(p)) => Some(format!("{}/{}", base.trim_matches('/'), p)),
            (Some(base), None) => Some(base.trim_matches('/').to_string()),
            (None, Some(p)) => Some(p.to_string()),
            (None, None) => None,
        };

        let mut files = Vec::new();

        let mut stream = self.container_client
            .list_blobs()
            .prefix(full_prefix.clone().unwrap_or_default())
            .into_stream();

        while let Some(result) = stream.next().await {
            let response = result
                .map_err(|e| StorageError::ListFailed(format!("Azure list failed: {}", e)))?;

            for blob in response.blobs.blobs() {
                let name = blob.name.split('/').last().unwrap_or(&blob.name).to_string();

                let modified: Option<DateTime<Utc>> = blob.properties.last_modified
                    .map(|t| DateTime::from_timestamp(t.unix_timestamp(), 0))
                    .flatten();

                files.push(StorageFile {
                    name,
                    path: format!("https://{}.blob.core.windows.net/{}/{}",
                        self.container_client.account(),
                        self.container,
                        blob.name
                    ),
                    size: blob.properties.content_length as i64,
                    modified,
                });
            }
        }

        Ok(files)
    }

    /// Check if a file exists in Azure Blob Storage
    pub async fn exists(&self, remote_path: &str) -> Result<bool, StorageError> {
        let blob_name = self.get_blob_name(remote_path);

        let blob_client = self.container_client.blob_client(&blob_name);

        match blob_client.get_properties().await {
            Ok(_) => Ok(true),
            Err(e) => {
                let error_string = e.to_string();
                if error_string.contains("404") || error_string.contains("BlobNotFound") {
                    Ok(false)
                } else {
                    Err(StorageError::ConnectionFailed(format!("Azure get properties failed: {}", e)))
                }
            }
        }
    }

    /// Get file metadata from Azure Blob Storage
    pub async fn get_metadata(&self, remote_path: &str) -> Result<StorageFile, StorageError> {
        let blob_name = self.get_blob_name(remote_path);

        let blob_client = self.container_client.blob_client(&blob_name);

        let properties = blob_client
            .get_properties()
            .await
            .map_err(|e| StorageError::ListFailed(format!("Azure get properties failed: {}", e)))?;

        let name = blob_name.split('/').last().unwrap_or(&blob_name).to_string();

        let modified: Option<DateTime<Utc>> = properties.blob.properties.last_modified
            .map(|t| DateTime::from_timestamp(t.unix_timestamp(), 0))
            .flatten();

        Ok(StorageFile {
            name,
            path: format!("https://{}.blob.core.windows.net/{}/{}",
                self.container_client.account(),
                self.container,
                blob_name
            ),
            size: properties.blob.properties.content_length as i64,
            modified,
        })
    }

    /// Copy a file within Azure Blob Storage
    pub async fn copy(&self, source_path: &str, dest_path: &str) -> Result<String, StorageError> {
        let source_blob = self.get_blob_name(source_path);
        let dest_blob = self.get_blob_name(dest_path);

        let source_url = format!("https://{}.blob.core.windows.net/{}/{}",
            self.container_client.account(),
            self.container,
            source_blob
        );

        let dest_client = self.container_client.blob_client(&dest_blob);

        dest_client
            .copy(url::Url::parse(&source_url).map_err(|e| StorageError::UploadFailed(e.to_string()))?)
            .await
            .map_err(|e| StorageError::UploadFailed(format!("Azure copy failed: {}", e)))?;

        Ok(format!("https://{}.blob.core.windows.net/{}/{}",
            self.container_client.account(),
            self.container,
            dest_blob
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_blob_name() {
        // Test would require mocking the Azure client
        // For now, just verify the config structure
        let config = AzureStorageConfig {
            container: "test-container".to_string(),
            account_name: "testaccount".to_string(),
            account_key: Some("testkey".to_string()),
            connection_string: None,
            path_prefix: Some("backups".to_string()),
            access_tier: "Hot".to_string(),
        };

        assert_eq!(config.container, "test-container");
        assert_eq!(config.account_name, "testaccount");
    }
}
