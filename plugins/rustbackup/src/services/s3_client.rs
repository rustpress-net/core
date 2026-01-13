//! S3 Client
//!
//! AWS S3 storage provider implementation.

#[cfg(feature = "s3")]
use aws_config::BehaviorVersion;
#[cfg(feature = "s3")]
use aws_sdk_s3::{
    Client,
    config::{Credentials, Region},
    primitives::ByteStream,
    types::StorageClass,
};
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use chrono::{DateTime, Utc};

use crate::models::storage::{S3StorageConfig, S3StorageClass};
use super::storage::{StorageFile, StorageError};

/// S3 client wrapper
#[cfg(feature = "s3")]
pub struct S3Client {
    client: Client,
    bucket: String,
    path_prefix: Option<String>,
    storage_class: StorageClass,
}

#[cfg(feature = "s3")]
impl S3Client {
    /// Create a new S3 client from config
    pub async fn from_config(config: &S3StorageConfig) -> Result<Self, StorageError> {
        let region = Region::new(config.region.clone());

        // Build AWS config
        let aws_config = if let (Some(access_key), Some(secret_key)) = (&config.access_key, &config.secret_key) {
            // Use explicit credentials
            let credentials = Credentials::new(
                access_key.clone(),
                secret_key.clone(),
                None, // session token
                None, // expiration
                "rustbackup",
            );

            let mut builder = aws_config::defaults(BehaviorVersion::latest())
                .region(region.clone())
                .credentials_provider(credentials);

            // Custom endpoint (for S3-compatible storage like MinIO, DigitalOcean Spaces, etc.)
            if let Some(endpoint) = &config.endpoint {
                builder = builder.endpoint_url(endpoint.clone());
            }

            builder.load().await
        } else {
            // Use default credential chain (IAM roles, env vars, etc.)
            aws_config::defaults(BehaviorVersion::latest())
                .region(region.clone())
                .load()
                .await
        };

        // Build S3 client config
        let mut s3_config_builder = aws_sdk_s3::config::Builder::from(&aws_config);

        // Force path style for custom endpoints (required for MinIO, etc.)
        if config.endpoint.is_some() {
            s3_config_builder = s3_config_builder.force_path_style(true);
        }

        let client = Client::from_conf(s3_config_builder.build());

        let storage_class = match config.storage_class {
            S3StorageClass::Standard => StorageClass::Standard,
            S3StorageClass::StandardIA => StorageClass::StandardIa,
            S3StorageClass::OneZoneIA => StorageClass::OnezoneIa,
            S3StorageClass::Glacier => StorageClass::Glacier,
            S3StorageClass::GlacierDeepArchive => StorageClass::DeepArchive,
            S3StorageClass::IntelligentTiering => StorageClass::IntelligentTiering,
        };

        Ok(Self {
            client,
            bucket: config.bucket.clone(),
            path_prefix: config.path_prefix.clone(),
            storage_class,
        })
    }

    /// Get the full S3 key for a given filename
    fn get_key(&self, name: &str) -> String {
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

    /// Test S3 connection by listing bucket
    pub async fn test_connection(&self) -> Result<Option<i64>, StorageError> {
        // Try to head the bucket to verify access
        self.client
            .head_bucket()
            .bucket(&self.bucket)
            .send()
            .await
            .map_err(|e| StorageError::ConnectionFailed(format!("Failed to access bucket: {}", e)))?;

        Ok(None) // S3 doesn't report available space
    }

    /// Upload a file to S3
    pub async fn upload(&self, local_path: &Path, remote_name: &str) -> Result<String, StorageError> {
        let key = self.get_key(remote_name);

        // Read file into memory
        let mut file = File::open(local_path).await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to open file: {}", e)))?;

        let metadata = file.metadata().await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to get file metadata: {}", e)))?;

        let file_size = metadata.len();

        // For large files, use multipart upload
        if file_size > 100 * 1024 * 1024 {
            // 100MB threshold
            self.multipart_upload(local_path, &key, file_size).await
        } else {
            // Simple upload for smaller files
            let mut buffer = Vec::with_capacity(file_size as usize);
            file.read_to_end(&mut buffer).await
                .map_err(|e| StorageError::UploadFailed(format!("Failed to read file: {}", e)))?;

            let body = ByteStream::from(buffer);

            self.client
                .put_object()
                .bucket(&self.bucket)
                .key(&key)
                .storage_class(self.storage_class.clone())
                .body(body)
                .send()
                .await
                .map_err(|e| StorageError::UploadFailed(format!("S3 upload failed: {}", e)))?;

            Ok(format!("s3://{}/{}", self.bucket, key))
        }
    }

    /// Multipart upload for large files
    async fn multipart_upload(&self, local_path: &Path, key: &str, _file_size: u64) -> Result<String, StorageError> {
        const PART_SIZE: u64 = 10 * 1024 * 1024; // 10MB parts

        // Create multipart upload
        let create_response = self.client
            .create_multipart_upload()
            .bucket(&self.bucket)
            .key(key)
            .storage_class(self.storage_class.clone())
            .send()
            .await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to create multipart upload: {}", e)))?;

        let upload_id = create_response.upload_id()
            .ok_or_else(|| StorageError::UploadFailed("No upload ID returned".to_string()))?;

        let mut file = File::open(local_path).await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to open file: {}", e)))?;

        let mut parts = Vec::new();
        let mut part_number = 1i32;

        loop {
            let mut buffer = vec![0u8; PART_SIZE as usize];
            let bytes_read = file.read(&mut buffer).await
                .map_err(|e| StorageError::UploadFailed(format!("Failed to read file: {}", e)))?;

            if bytes_read == 0 {
                break;
            }

            buffer.truncate(bytes_read);
            let body = ByteStream::from(buffer);

            let upload_part_response = self.client
                .upload_part()
                .bucket(&self.bucket)
                .key(key)
                .upload_id(upload_id)
                .part_number(part_number)
                .body(body)
                .send()
                .await
                .map_err(|e| {
                    // Try to abort the multipart upload on failure
                    let _ = self.abort_multipart_upload(key, upload_id);
                    StorageError::UploadFailed(format!("Failed to upload part {}: {}", part_number, e))
                })?;

            let e_tag = upload_part_response.e_tag()
                .ok_or_else(|| StorageError::UploadFailed("No ETag returned for part".to_string()))?;

            parts.push(
                aws_sdk_s3::types::CompletedPart::builder()
                    .part_number(part_number)
                    .e_tag(e_tag)
                    .build()
            );

            part_number += 1;
        }

        // Complete multipart upload
        let completed_upload = aws_sdk_s3::types::CompletedMultipartUpload::builder()
            .set_parts(Some(parts))
            .build();

        self.client
            .complete_multipart_upload()
            .bucket(&self.bucket)
            .key(key)
            .upload_id(upload_id)
            .multipart_upload(completed_upload)
            .send()
            .await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to complete multipart upload: {}", e)))?;

        Ok(format!("s3://{}/{}", self.bucket, key))
    }

    /// Abort a multipart upload (fire and forget)
    fn abort_multipart_upload(&self, key: &str, upload_id: &str) {
        let client = self.client.clone();
        let bucket = self.bucket.clone();
        let key = key.to_string();
        let upload_id = upload_id.to_string();

        tokio::spawn(async move {
            let _ = client
                .abort_multipart_upload()
                .bucket(&bucket)
                .key(&key)
                .upload_id(&upload_id)
                .send()
                .await;
        });
    }

    /// Download a file from S3
    pub async fn download(&self, remote_path: &str, local_path: &Path) -> Result<(), StorageError> {
        let key = self.get_key(remote_path);

        let response = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| StorageError::DownloadFailed(format!("S3 download failed: {}", e)))?;

        // Collect body bytes
        let body = response.body.collect().await
            .map_err(|e| StorageError::DownloadFailed(format!("Failed to read S3 response: {}", e)))?;

        let bytes = body.into_bytes();

        // Write to local file
        tokio::fs::write(local_path, bytes).await
            .map_err(|e| StorageError::DownloadFailed(format!("Failed to write file: {}", e)))?;

        Ok(())
    }

    /// Delete a file from S3
    pub async fn delete(&self, remote_path: &str) -> Result<(), StorageError> {
        let key = self.get_key(remote_path);

        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| StorageError::DeleteFailed(format!("S3 delete failed: {}", e)))?;

        Ok(())
    }

    /// List files in S3
    pub async fn list_files(&self, prefix: Option<&str>) -> Result<Vec<StorageFile>, StorageError> {
        let full_prefix = match (&self.path_prefix, prefix) {
            (Some(base), Some(p)) => Some(format!("{}/{}", base.trim_matches('/'), p)),
            (Some(base), None) => Some(base.trim_matches('/').to_string()),
            (None, Some(p)) => Some(p.to_string()),
            (None, None) => None,
        };

        let mut files = Vec::new();
        let mut continuation_token: Option<String> = None;

        loop {
            let mut request = self.client
                .list_objects_v2()
                .bucket(&self.bucket);

            if let Some(ref prefix) = full_prefix {
                request = request.prefix(prefix);
            }

            if let Some(ref token) = continuation_token {
                request = request.continuation_token(token);
            }

            let response = request.send().await
                .map_err(|e| StorageError::ListFailed(format!("S3 list failed: {}", e)))?;

            if let Some(contents) = response.contents {
                for object in contents {
                    let key = object.key().unwrap_or_default();
                    let name = key.split('/').last().unwrap_or(key).to_string();

                    let modified: Option<DateTime<Utc>> = object.last_modified()
                        .and_then(|dt| {
                            DateTime::from_timestamp(dt.secs(), dt.subsec_nanos())
                        });

                    files.push(StorageFile {
                        name,
                        path: format!("s3://{}/{}", self.bucket, key),
                        size: object.size().unwrap_or(0),
                        modified,
                    });
                }
            }

            if response.is_truncated() == Some(true) {
                continuation_token = response.next_continuation_token;
            } else {
                break;
            }
        }

        Ok(files)
    }

    /// Check if a file exists in S3
    pub async fn exists(&self, remote_path: &str) -> Result<bool, StorageError> {
        let key = self.get_key(remote_path);

        match self.client
            .head_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                let service_error = e.into_service_error();
                if service_error.is_not_found() {
                    Ok(false)
                } else {
                    Err(StorageError::ConnectionFailed(format!("S3 head object failed: {}", service_error)))
                }
            }
        }
    }

    /// Get file metadata from S3
    pub async fn get_metadata(&self, remote_path: &str) -> Result<StorageFile, StorageError> {
        let key = self.get_key(remote_path);

        let response = self.client
            .head_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| StorageError::ListFailed(format!("S3 head object failed: {}", e)))?;

        let name = key.split('/').last().unwrap_or(&key).to_string();

        let modified: Option<DateTime<Utc>> = response.last_modified()
            .and_then(|dt| {
                DateTime::from_timestamp(dt.secs(), dt.subsec_nanos())
            });

        Ok(StorageFile {
            name,
            path: format!("s3://{}/{}", self.bucket, key),
            size: response.content_length().unwrap_or(0),
            modified,
        })
    }

    /// Copy a file within S3
    pub async fn copy(&self, source_path: &str, dest_path: &str) -> Result<String, StorageError> {
        let source_key = self.get_key(source_path);
        let dest_key = self.get_key(dest_path);

        let copy_source = format!("{}/{}", self.bucket, source_key);

        self.client
            .copy_object()
            .bucket(&self.bucket)
            .copy_source(&copy_source)
            .key(&dest_key)
            .storage_class(self.storage_class.clone())
            .send()
            .await
            .map_err(|e| StorageError::UploadFailed(format!("S3 copy failed: {}", e)))?;

        Ok(format!("s3://{}/{}", self.bucket, dest_key))
    }
}

// Stub implementation when S3 feature is disabled
#[cfg(not(feature = "s3"))]
pub struct S3Client;

#[cfg(not(feature = "s3"))]
impl S3Client {
    pub async fn from_config(_config: &S3StorageConfig) -> Result<Self, StorageError> {
        Err(StorageError::NotImplemented("S3 support not compiled. Enable the 's3' feature.".to_string()))
    }

    pub async fn test_connection(&self) -> Result<Option<i64>, StorageError> {
        Err(StorageError::NotImplemented("S3 support not compiled".to_string()))
    }

    pub async fn upload(&self, _local_path: &Path, _remote_name: &str) -> Result<String, StorageError> {
        Err(StorageError::NotImplemented("S3 support not compiled".to_string()))
    }

    pub async fn download(&self, _remote_path: &str, _local_path: &Path) -> Result<(), StorageError> {
        Err(StorageError::NotImplemented("S3 support not compiled".to_string()))
    }

    pub async fn delete(&self, _remote_path: &str) -> Result<(), StorageError> {
        Err(StorageError::NotImplemented("S3 support not compiled".to_string()))
    }

    pub async fn list_files(&self, _prefix: Option<&str>) -> Result<Vec<StorageFile>, StorageError> {
        Err(StorageError::NotImplemented("S3 support not compiled".to_string()))
    }

    pub async fn exists(&self, _remote_path: &str) -> Result<bool, StorageError> {
        Err(StorageError::NotImplemented("S3 support not compiled".to_string()))
    }

    pub async fn get_metadata(&self, _remote_path: &str) -> Result<StorageFile, StorageError> {
        Err(StorageError::NotImplemented("S3 support not compiled".to_string()))
    }

    pub async fn copy(&self, _source_path: &str, _dest_path: &str) -> Result<String, StorageError> {
        Err(StorageError::NotImplemented("S3 support not compiled".to_string()))
    }
}

#[cfg(all(test, feature = "s3"))]
mod tests {
    use super::*;
    use crate::models::storage::S3StorageClass;

    #[tokio::test]
    async fn test_get_key() {
        let config = S3StorageConfig {
            bucket: "test-bucket".to_string(),
            region: "us-east-1".to_string(),
            access_key: Some("test".to_string()),
            secret_key: Some("test".to_string()),
            endpoint: None,
            path_prefix: Some("backups".to_string()),
            storage_class: S3StorageClass::Standard,
        };

        // This test would need mocking for the AWS client
        // For now, just test that config parsing works
        assert_eq!(config.bucket, "test-bucket");
    }
}
