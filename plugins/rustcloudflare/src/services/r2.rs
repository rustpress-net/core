//! R2 Storage service

use crate::client::CloudflareClient;
use crate::config::CloudflareConfig;
use crate::error::{CloudflareError, CloudflareResult};
use aws_sdk_s3::Client as S3Client;
use sqlx::PgPool;
use std::sync::Arc;

pub struct R2Service {
    #[allow(dead_code)]
    client: Option<Arc<CloudflareClient>>,
    #[allow(dead_code)]
    db: PgPool,
    s3_client: Option<S3Client>,
}

impl R2Service {
    pub fn new(client: Arc<CloudflareClient>, db: PgPool) -> Self {
        Self { client: Some(client), db, s3_client: None }
    }

    /// Create without a configured client (for initial setup)
    pub fn new_unconfigured(db: PgPool) -> Self {
        Self { client: None, db, s3_client: None }
    }

    /// Get the S3 client or return an error if not initialized
    fn get_s3_client(&self) -> CloudflareResult<&S3Client> {
        self.s3_client.as_ref()
            .ok_or_else(|| CloudflareError::R2Error("R2 not initialized. Please configure R2 credentials.".to_string()))
    }

    pub async fn init_s3_client(&mut self, config: &CloudflareConfig) -> CloudflareResult<()> {
        if !config.r2_enabled {
            return Ok(());
        }

        let access_key = config.r2_access_key_id.as_ref()
            .ok_or(CloudflareError::MissingConfig("r2_access_key_id".into()))?;
        let secret_key = config.r2_secret_access_key.as_ref()
            .ok_or(CloudflareError::MissingConfig("r2_secret_access_key".into()))?;

        let creds = aws_credential_types::Credentials::new(
            access_key, secret_key, None, None, "rustcloudflare"
        );

        let s3_config = aws_sdk_s3::Config::builder()
            .endpoint_url(config.r2_endpoint())
            .credentials_provider(creds)
            .region(aws_sdk_s3::config::Region::new("auto"))
            .force_path_style(true)
            .build();

        self.s3_client = Some(S3Client::from_conf(s3_config));
        Ok(())
    }

    pub async fn list_buckets(&self) -> CloudflareResult<Vec<String>> {
        let client = self.get_s3_client()?;

        let result = client.list_buckets().send().await
            .map_err(|e| CloudflareError::R2Error(e.to_string()))?;

        Ok(result.buckets().iter().filter_map(|b| b.name().map(|s| s.to_string())).collect())
    }

    pub async fn list_objects(&self, bucket: &str, prefix: Option<&str>) -> CloudflareResult<Vec<R2Object>> {
        let client = self.get_s3_client()?;

        let mut req = client.list_objects_v2().bucket(bucket);
        if let Some(p) = prefix {
            req = req.prefix(p);
        }

        let result = req.send().await
            .map_err(|e| CloudflareError::R2Error(e.to_string()))?;

        Ok(result.contents().iter().map(|obj| R2Object {
            key: obj.key().unwrap_or_default().to_string(),
            size: obj.size().unwrap_or(0),
            etag: obj.e_tag().map(|s| s.to_string()),
            last_modified: obj.last_modified().map(|t| t.to_string()),
        }).collect())
    }

    pub async fn upload(&self, bucket: &str, key: &str, body: Vec<u8>, content_type: Option<&str>) -> CloudflareResult<()> {
        let client = self.get_s3_client()?;

        let mut req = client.put_object()
            .bucket(bucket)
            .key(key)
            .body(body.into());

        if let Some(ct) = content_type {
            req = req.content_type(ct);
        }

        req.send().await
            .map_err(|e| CloudflareError::R2Error(e.to_string()))?;

        Ok(())
    }

    pub async fn delete(&self, bucket: &str, key: &str) -> CloudflareResult<()> {
        let client = self.get_s3_client()?;

        client.delete_object()
            .bucket(bucket)
            .key(key)
            .send().await
            .map_err(|e| CloudflareError::R2Error(e.to_string()))?;

        Ok(())
    }

    pub async fn get_presigned_url(&self, _bucket: &str, _key: &str, _expires_in: u64) -> CloudflareResult<String> {
        // R2 presigned URLs would be implemented here
        Err(CloudflareError::R2Error("Presigned URLs not implemented".into()))
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct R2Object {
    pub key: String,
    pub size: i64,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}
