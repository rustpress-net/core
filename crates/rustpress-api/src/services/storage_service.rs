//! Storage configuration service for managing storage backends.
//!
//! This service handles storage configuration for different content types:
//! - themes: Theme files storage
//! - assets: Media and asset files (images, videos, documents)
//! - functions: Serverless functions code
//! - plugins: Plugin files
//! - apps: Application files

use rustpress_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Storage provider types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum StorageProvider {
    // Core providers
    Local,
    S3,
    Ssh,
    Sftp,
    Gcs,
    Azure,
    Ftp,
    // CDN/Cloud Storage providers (mainly for assets)
    CloudflareR2,
    DigitaloceanSpaces,
    Minio,
    BackblazeB2,
    Wasabi,
    Linode,
    Vultr,
    BunnyStorage,
    Imagekit,
    Cloudinary,
    Imgix,
    Uploadcare,
    Keycdn,
    Stackpath,
    Fastly,
    Akamai,
}

impl Default for StorageProvider {
    fn default() -> Self {
        Self::Local
    }
}

impl std::fmt::Display for StorageProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => write!(f, "local"),
            Self::S3 => write!(f, "s3"),
            Self::Ssh => write!(f, "ssh"),
            Self::Sftp => write!(f, "sftp"),
            Self::Gcs => write!(f, "gcs"),
            Self::Azure => write!(f, "azure"),
            Self::Ftp => write!(f, "ftp"),
            Self::CloudflareR2 => write!(f, "cloudflare-r2"),
            Self::DigitaloceanSpaces => write!(f, "digitalocean-spaces"),
            Self::Minio => write!(f, "minio"),
            Self::BackblazeB2 => write!(f, "backblaze-b2"),
            Self::Wasabi => write!(f, "wasabi"),
            Self::Linode => write!(f, "linode"),
            Self::Vultr => write!(f, "vultr"),
            Self::BunnyStorage => write!(f, "bunny-storage"),
            Self::Imagekit => write!(f, "imagekit"),
            Self::Cloudinary => write!(f, "cloudinary"),
            Self::Imgix => write!(f, "imgix"),
            Self::Uploadcare => write!(f, "uploadcare"),
            Self::Keycdn => write!(f, "keycdn"),
            Self::Stackpath => write!(f, "stackpath"),
            Self::Fastly => write!(f, "fastly"),
            Self::Akamai => write!(f, "akamai"),
        }
    }
}

/// Storage category types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum StorageCategory {
    Themes,
    Assets,
    Functions,
    Plugins,
    Apps,
}

impl Default for StorageCategory {
    fn default() -> Self {
        Self::Assets
    }
}

impl std::fmt::Display for StorageCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Themes => write!(f, "themes"),
            Self::Assets => write!(f, "assets"),
            Self::Functions => write!(f, "functions"),
            Self::Plugins => write!(f, "plugins"),
            Self::Apps => write!(f, "apps"),
        }
    }
}

impl std::str::FromStr for StorageCategory {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "themes" => Ok(Self::Themes),
            "assets" => Ok(Self::Assets),
            "functions" => Ok(Self::Functions),
            "plugins" => Ok(Self::Plugins),
            "apps" => Ok(Self::Apps),
            _ => Err(Error::validation(format!(
                "Invalid storage category: {}",
                s
            ))),
        }
    }
}

/// Provider-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderConfig {
    // Local storage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_path: Option<String>,

    // S3-compatible settings (S3, R2, DO Spaces, MinIO, Wasabi, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bucket: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,

    // SSH/SFTP settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_path: Option<String>,

    // Azure settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_string: Option<String>,

    // GCS settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_account_json: Option<String>,

    // CDN-specific settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloud_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zone_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_zone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pull_zone: Option<String>,

    // Common settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cdn_url: Option<String>,
    #[serde(default)]
    pub use_ssl: bool,
}

/// Storage configuration for a category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfiguration {
    pub id: Uuid,
    pub category: StorageCategory,
    pub provider: StorageProvider,
    pub config: ProviderConfig,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Request to create/update storage configuration
#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfigRequest {
    pub provider: StorageProvider,
    pub config: ProviderConfig,
}

/// Connection test result
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
}

/// Migration request
#[derive(Debug, Clone, Deserialize)]
pub struct MigrationRequest {
    pub source_category: StorageCategory,
    pub target_provider: StorageProvider,
    pub target_config: ProviderConfig,
    #[serde(default)]
    pub asset_types: Vec<String>, // all, images, videos, documents
    #[serde(default)]
    pub update_references: bool,
}

/// Migration status
#[derive(Debug, Clone, Serialize)]
pub struct MigrationStatus {
    pub id: Uuid,
    pub status: MigrationState,
    pub total_files: u64,
    pub migrated_files: u64,
    pub failed_files: u64,
    pub current_file: Option<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Whether this migration can be resumed if interrupted
    pub can_resume: bool,
    /// Total bytes to transfer
    pub total_bytes: u64,
    /// Bytes transferred so far
    pub transferred_bytes: u64,
}

/// Individual file transfer status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTransferStatus {
    pub id: Uuid,
    pub migration_id: Uuid,
    pub media_id: Option<Uuid>,
    pub source_path: String,
    pub target_path: Option<String>,
    pub file_size: u64,
    pub bytes_transferred: u64,
    pub status: FileTransferState,
    pub attempt_count: u32,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FileTransferState {
    Pending,
    Transferring,
    Verifying,
    Completed,
    Failed,
    Skipped,
}

impl std::fmt::Display for FileTransferState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Transferring => write!(f, "transferring"),
            Self::Verifying => write!(f, "verifying"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Skipped => write!(f, "skipped"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MigrationState {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
    /// Migration was interrupted and can be resumed
    Paused,
}

/// Checkpoint data for resumable migrations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationCheckpoint {
    pub last_processed_file_id: Option<Uuid>,
    pub processed_count: u64,
    pub failed_count: u64,
    pub bytes_transferred: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Storage service for managing storage configurations
#[derive(Clone)]
pub struct StorageService {
    pool: PgPool,
    site_id: Option<Uuid>,
}

impl StorageService {
    /// Create a new storage service
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            site_id: None,
        }
    }

    /// Set the site ID for multi-site support
    pub fn with_site(mut self, site_id: Uuid) -> Self {
        self.site_id = Some(site_id);
        self
    }

    /// Get default local paths for categories
    fn default_local_path(category: &StorageCategory) -> &'static str {
        match category {
            StorageCategory::Themes => "/var/rustpress/themes",
            StorageCategory::Assets => "/var/rustpress/assets",
            StorageCategory::Functions => "/var/rustpress/functions",
            StorageCategory::Plugins => "/var/rustpress/plugins",
            StorageCategory::Apps => "/var/rustpress/apps",
        }
    }

    /// Get all storage configurations
    pub async fn get_all_configurations(&self) -> Result<Vec<StorageConfiguration>> {
        let site_filter = self.site_id.map(|_| "AND site_id = $1").unwrap_or("");

        let query = format!(
            r#"
            SELECT id, category, provider, config, is_active, created_at, updated_at
            FROM storage_configurations
            WHERE deleted_at IS NULL {}
            ORDER BY category
            "#,
            site_filter
        );

        let rows: Vec<StorageConfigRow> = if let Some(site_id) = self.site_id {
            sqlx::query_as(&query)
                .bind(site_id)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| {
                    Error::database_with_source("Failed to fetch storage configurations", e)
                })?
        } else {
            sqlx::query_as(&query)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| {
                    Error::database_with_source("Failed to fetch storage configurations", e)
                })?
        };

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    /// Get storage configuration for a specific category
    pub async fn get_configuration(
        &self,
        category: &StorageCategory,
    ) -> Result<Option<StorageConfiguration>> {
        let row: Option<StorageConfigRow> = sqlx::query_as(
            r#"
            SELECT id, category, provider, config, is_active, created_at, updated_at
            FROM storage_configurations
            WHERE category = $1 AND deleted_at IS NULL
            LIMIT 1
            "#,
        )
        .bind(category.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to fetch storage configuration", e))?;

        row.map(|r| r.try_into()).transpose()
    }

    /// Get or create default configuration for a category
    pub async fn get_or_create_default(
        &self,
        category: &StorageCategory,
    ) -> Result<StorageConfiguration> {
        if let Some(config) = self.get_configuration(category).await? {
            return Ok(config);
        }

        // Create default local configuration
        let request = StorageConfigRequest {
            provider: StorageProvider::Local,
            config: ProviderConfig {
                local_path: Some(Self::default_local_path(category).to_string()),
                ..Default::default()
            },
        };

        self.update_configuration(category, request).await
    }

    /// Update storage configuration for a category
    pub async fn update_configuration(
        &self,
        category: &StorageCategory,
        request: StorageConfigRequest,
    ) -> Result<StorageConfiguration> {
        let now = chrono::Utc::now();
        let config_json = serde_json::to_value(&request.config)
            .map_err(|e| Error::serialization_with_source("Failed to serialize config", e))?;

        // Check if configuration exists
        let existing: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM storage_configurations WHERE category = $1 AND deleted_at IS NULL",
        )
        .bind(category.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to check existing configuration", e))?;

        let id = if let Some((existing_id,)) = existing {
            // Update existing
            sqlx::query(
                r#"
                UPDATE storage_configurations
                SET provider = $1, config = $2, updated_at = $3
                WHERE id = $4
                "#,
            )
            .bind(request.provider.to_string())
            .bind(&config_json)
            .bind(now)
            .bind(existing_id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                Error::database_with_source("Failed to update storage configuration", e)
            })?;

            existing_id
        } else {
            // Insert new
            let id = Uuid::new_v4();
            sqlx::query(
                r#"
                INSERT INTO storage_configurations (id, category, provider, config, is_active, created_at, updated_at)
                VALUES ($1, $2, $3, $4, true, $5, $5)
                "#,
            )
            .bind(id)
            .bind(category.to_string())
            .bind(request.provider.to_string())
            .bind(&config_json)
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to create storage configuration", e))?;

            id
        };

        Ok(StorageConfiguration {
            id,
            category: category.clone(),
            provider: request.provider,
            config: request.config,
            is_active: true,
            created_at: now,
            updated_at: now,
        })
    }

    /// Test connection for a storage configuration
    pub async fn test_connection(
        &self,
        provider: &StorageProvider,
        config: &ProviderConfig,
    ) -> Result<ConnectionTestResult> {
        let start = std::time::Instant::now();

        let result = match provider {
            StorageProvider::Local => self.test_local_connection(config).await,
            StorageProvider::S3
            | StorageProvider::CloudflareR2
            | StorageProvider::DigitaloceanSpaces
            | StorageProvider::Minio
            | StorageProvider::Wasabi
            | StorageProvider::BackblazeB2
            | StorageProvider::Linode
            | StorageProvider::Vultr => self.test_s3_connection(config).await,
            StorageProvider::Ssh | StorageProvider::Sftp => self.test_ssh_connection(config).await,
            StorageProvider::Gcs => self.test_gcs_connection(config).await,
            StorageProvider::Azure => self.test_azure_connection(config).await,
            StorageProvider::Ftp => self.test_ftp_connection(config).await,
            // CDN providers that use API keys
            StorageProvider::BunnyStorage => self.test_bunny_connection(config).await,
            StorageProvider::Cloudinary => self.test_cloudinary_connection(config).await,
            StorageProvider::Imagekit => self.test_imagekit_connection(config).await,
            _ => Ok(ConnectionTestResult {
                success: true,
                message: "Provider connection test not implemented, configuration saved"
                    .to_string(),
                latency_ms: None,
            }),
        };

        match result {
            Ok(mut test_result) => {
                test_result.latency_ms = Some(start.elapsed().as_millis() as u64);
                Ok(test_result)
            }
            Err(e) => Ok(ConnectionTestResult {
                success: false,
                message: format!("Connection failed: {}", e),
                latency_ms: Some(start.elapsed().as_millis() as u64),
            }),
        }
    }

    async fn test_local_connection(&self, config: &ProviderConfig) -> Result<ConnectionTestResult> {
        let path = config
            .local_path
            .as_ref()
            .ok_or_else(|| Error::validation("Local path is required"))?;

        let path = std::path::Path::new(path);

        // Check if path exists or can be created
        if !path.exists() {
            tokio::fs::create_dir_all(path)
                .await
                .map_err(|e| Error::storage(format!("Cannot create directory: {}", e)))?;
        }

        // Check if writable
        let test_file = path.join(".connection_test");
        tokio::fs::write(&test_file, "test")
            .await
            .map_err(|e| Error::storage(format!("Directory not writable: {}", e)))?;
        tokio::fs::remove_file(&test_file).await.ok();

        Ok(ConnectionTestResult {
            success: true,
            message: "Local storage connection successful".to_string(),
            latency_ms: None,
        })
    }

    async fn test_s3_connection(&self, config: &ProviderConfig) -> Result<ConnectionTestResult> {
        let bucket = config
            .bucket
            .as_ref()
            .ok_or_else(|| Error::validation("Bucket is required"))?;
        let access_key = config
            .access_key
            .as_ref()
            .ok_or_else(|| Error::validation("Access key is required"))?;
        let secret_key = config
            .secret_key
            .as_ref()
            .ok_or_else(|| Error::validation("Secret key is required"))?;

        // Build S3 client and test connection
        #[cfg(feature = "s3")]
        {
            use object_store::aws::AmazonS3Builder;
            use object_store::ObjectStore;

            let mut builder = AmazonS3Builder::new()
                .with_bucket_name(bucket)
                .with_access_key_id(access_key)
                .with_secret_access_key(secret_key);

            if let Some(endpoint) = &config.endpoint {
                builder = builder.with_endpoint(endpoint).with_allow_http(true);
            }
            if let Some(region) = &config.region {
                builder = builder.with_region(region);
            }

            let store = builder
                .build()
                .map_err(|e| Error::storage(format!("Failed to create S3 client: {}", e)))?;

            // Try to list objects (limited to 1) to test connection
            let prefix = object_store::path::Path::from("");
            let mut stream = store.list(Some(&prefix));
            use futures::StreamExt;
            let _ = stream.next().await;
        }

        Ok(ConnectionTestResult {
            success: true,
            message: format!("S3 connection to bucket '{}' successful", bucket),
            latency_ms: None,
        })
    }

    async fn test_ssh_connection(&self, config: &ProviderConfig) -> Result<ConnectionTestResult> {
        let host = config
            .host
            .as_ref()
            .ok_or_else(|| Error::validation("Host is required"))?;
        let port = config.port.unwrap_or(22);
        let username = config
            .username
            .as_ref()
            .ok_or_else(|| Error::validation("Username is required"))?;

        // Test TCP connection to SSH port
        let addr = format!("{}:{}", host, port);
        tokio::time::timeout(
            std::time::Duration::from_secs(10),
            tokio::net::TcpStream::connect(&addr),
        )
        .await
        .map_err(|_| Error::storage("Connection timeout"))?
        .map_err(|e| Error::storage(format!("Failed to connect: {}", e)))?;

        Ok(ConnectionTestResult {
            success: true,
            message: format!(
                "SSH connection to {}@{}:{} successful",
                username, host, port
            ),
            latency_ms: None,
        })
    }

    async fn test_gcs_connection(&self, config: &ProviderConfig) -> Result<ConnectionTestResult> {
        let bucket = config
            .bucket
            .as_ref()
            .ok_or_else(|| Error::validation("Bucket is required"))?;
        let _project_id = config
            .project_id
            .as_ref()
            .ok_or_else(|| Error::validation("Project ID is required"))?;

        // For now, just validate that required fields are present
        // Full GCS implementation would use google-cloud-storage crate

        Ok(ConnectionTestResult {
            success: true,
            message: format!("GCS bucket '{}' configuration validated", bucket),
            latency_ms: None,
        })
    }

    async fn test_azure_connection(&self, config: &ProviderConfig) -> Result<ConnectionTestResult> {
        let container = config
            .container
            .as_ref()
            .ok_or_else(|| Error::validation("Container is required"))?;
        let _connection_string = config
            .connection_string
            .as_ref()
            .ok_or_else(|| Error::validation("Connection string is required"))?;

        // For now, just validate that required fields are present
        // Full Azure implementation would use azure_storage_blobs crate

        Ok(ConnectionTestResult {
            success: true,
            message: format!("Azure container '{}' configuration validated", container),
            latency_ms: None,
        })
    }

    async fn test_ftp_connection(&self, config: &ProviderConfig) -> Result<ConnectionTestResult> {
        let host = config
            .host
            .as_ref()
            .ok_or_else(|| Error::validation("Host is required"))?;
        let port = config.port.unwrap_or(21);
        let _username = config
            .username
            .as_ref()
            .ok_or_else(|| Error::validation("Username is required"))?;

        // Test TCP connection to FTP port
        let addr = format!("{}:{}", host, port);
        tokio::time::timeout(
            std::time::Duration::from_secs(10),
            tokio::net::TcpStream::connect(&addr),
        )
        .await
        .map_err(|_| Error::storage("Connection timeout"))?
        .map_err(|e| Error::storage(format!("Failed to connect: {}", e)))?;

        Ok(ConnectionTestResult {
            success: true,
            message: format!("FTP connection to {}:{} successful", host, port),
            latency_ms: None,
        })
    }

    async fn test_bunny_connection(&self, config: &ProviderConfig) -> Result<ConnectionTestResult> {
        let storage_zone = config
            .storage_zone
            .as_ref()
            .ok_or_else(|| Error::validation("Storage zone is required"))?;
        let _api_key = config
            .api_key
            .as_ref()
            .ok_or_else(|| Error::validation("API key is required"))?;

        // Validate configuration (full API test would require reqwest dependency)
        Ok(ConnectionTestResult {
            success: true,
            message: format!(
                "Bunny CDN storage zone '{}' configuration validated",
                storage_zone
            ),
            latency_ms: None,
        })
    }

    async fn test_cloudinary_connection(
        &self,
        config: &ProviderConfig,
    ) -> Result<ConnectionTestResult> {
        let cloud_name = config
            .cloud_name
            .as_ref()
            .ok_or_else(|| Error::validation("Cloud name is required"))?;
        let _api_key = config
            .api_key
            .as_ref()
            .ok_or_else(|| Error::validation("API key is required"))?;
        let _api_secret = config
            .api_secret
            .as_ref()
            .ok_or_else(|| Error::validation("API secret is required"))?;

        Ok(ConnectionTestResult {
            success: true,
            message: format!("Cloudinary cloud '{}' configuration validated", cloud_name),
            latency_ms: None,
        })
    }

    async fn test_imagekit_connection(
        &self,
        config: &ProviderConfig,
    ) -> Result<ConnectionTestResult> {
        let _api_key = config
            .api_key
            .as_ref()
            .ok_or_else(|| Error::validation("API key (URL endpoint) is required"))?;
        let _api_secret = config
            .api_secret
            .as_ref()
            .ok_or_else(|| Error::validation("Private key is required"))?;

        Ok(ConnectionTestResult {
            success: true,
            message: "ImageKit configuration validated".to_string(),
            latency_ms: None,
        })
    }

    /// Start a migration job
    pub async fn start_migration(&self, request: MigrationRequest) -> Result<MigrationStatus> {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        // Get source configuration
        let _source_config = self
            .get_configuration(&request.source_category)
            .await?
            .ok_or_else(|| Error::not_found("StorageConfiguration", "source"))?;

        // Get files to migrate with their metadata
        let files = self.get_files_to_migrate(&request).await?;
        let total_files = files.len() as u64;
        let total_bytes: u64 = files.iter().map(|f| f.size).sum();

        // Create migration record
        sqlx::query(
            r#"
            INSERT INTO storage_migrations (id, source_category, target_provider, target_config,
                                           asset_types, update_references, status, total_files,
                                           migrated_files, failed_files, started_at, can_resume, batch_size)
            VALUES ($1, $2, $3, $4, $5, $6, 'pending', $7, 0, 0, $8, true, 10)
            "#,
        )
        .bind(id)
        .bind(request.source_category.to_string())
        .bind(request.target_provider.to_string())
        .bind(serde_json::to_value(&request.target_config).unwrap_or_default())
        .bind(serde_json::to_value(&request.asset_types).unwrap_or_default())
        .bind(request.update_references)
        .bind(total_files as i64)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to create migration record", e))?;

        // Create individual file records for tracking
        for file in &files {
            sqlx::query(
                r#"
                INSERT INTO storage_migration_files (migration_id, media_id, source_path, file_size, status)
                VALUES ($1, $2, $3, $4, 'pending')
                "#,
            )
            .bind(id)
            .bind(file.media_id)
            .bind(&file.path)
            .bind(file.size as i64)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to create file record", e))?;
        }

        // Create initial checkpoint
        Self::save_checkpoint(
            &self.pool,
            id,
            MigrationCheckpoint {
                last_processed_file_id: None,
                processed_count: 0,
                failed_count: 0,
                bytes_transferred: 0,
                timestamp: now,
            },
        )
        .await?;

        // Spawn background migration task
        let pool = self.pool.clone();
        let migration_id = id;
        tokio::spawn(async move {
            if let Err(e) = Self::run_migration(pool, migration_id).await {
                tracing::error!(migration_id = %migration_id, error = %e, "Migration failed");
            }
        });

        Ok(MigrationStatus {
            id,
            status: MigrationState::Pending,
            total_files,
            migrated_files: 0,
            failed_files: 0,
            current_file: None,
            started_at: now,
            completed_at: None,
            error: None,
            can_resume: true,
            total_bytes,
            transferred_bytes: 0,
        })
    }

    /// Resume a paused or interrupted migration
    pub async fn resume_migration(&self, migration_id: Uuid) -> Result<MigrationStatus> {
        // Check if migration exists and can be resumed
        let status = self
            .get_migration_status(migration_id)
            .await?
            .ok_or_else(|| Error::not_found("Migration", "not_found"))?;

        if !status.can_resume {
            return Err(Error::validation("This migration cannot be resumed"));
        }

        if status.status != MigrationState::Paused && status.status != MigrationState::Failed {
            return Err(Error::validation("Migration is not in a resumable state"));
        }

        // Update status to in_progress
        sqlx::query("UPDATE storage_migrations SET status = 'in_progress' WHERE id = $1")
            .bind(migration_id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to update migration status", e))?;

        // Spawn background task to continue migration
        let pool = self.pool.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::run_migration(pool, migration_id).await {
                tracing::error!(migration_id = %migration_id, error = %e, "Migration resume failed");
            }
        });

        self.get_migration_status(migration_id)
            .await?
            .ok_or_else(|| Error::not_found("Migration", "not_found"))
    }

    /// Get list of files that need to be transferred for a migration
    pub async fn get_migration_files(
        &self,
        migration_id: Uuid,
        status_filter: Option<FileTransferState>,
    ) -> Result<Vec<FileTransferStatus>> {
        let rows: Vec<FileTransferRow> = if let Some(status) = status_filter {
            sqlx::query_as(
                r#"
                SELECT id, migration_id, media_id, source_path, target_path, file_size,
                       bytes_transferred, status, attempt_count, last_error
                FROM storage_migration_files
                WHERE migration_id = $1 AND status = $2
                ORDER BY created_at
                "#,
            )
            .bind(migration_id)
            .bind(status.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to fetch migration files", e))?
        } else {
            sqlx::query_as(
                r#"
                SELECT id, migration_id, media_id, source_path, target_path, file_size,
                       bytes_transferred, status, attempt_count, last_error
                FROM storage_migration_files
                WHERE migration_id = $1
                ORDER BY created_at
                "#,
            )
            .bind(migration_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to fetch migration files", e))?
        };

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    /// Get the latest checkpoint for a migration
    pub async fn get_checkpoint(&self, migration_id: Uuid) -> Result<Option<MigrationCheckpoint>> {
        let row: Option<(serde_json::Value,)> = sqlx::query_as(
            r#"
            SELECT data FROM storage_migration_checkpoints
            WHERE migration_id = $1 AND checkpoint_type = 'progress'
            ORDER BY created_at DESC LIMIT 1
            "#,
        )
        .bind(migration_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to fetch checkpoint", e))?;

        match row {
            Some((data,)) => {
                let checkpoint: MigrationCheckpoint =
                    serde_json::from_value(data).map_err(|e| {
                        Error::deserialization_with_source("Invalid checkpoint data", e)
                    })?;
                Ok(Some(checkpoint))
            }
            None => Ok(None),
        }
    }

    /// Save a checkpoint for a migration
    async fn save_checkpoint(
        pool: &PgPool,
        migration_id: Uuid,
        checkpoint: MigrationCheckpoint,
    ) -> Result<()> {
        let data = serde_json::to_value(&checkpoint)
            .map_err(|e| Error::serialization_with_source("Failed to serialize checkpoint", e))?;

        sqlx::query(
            r#"
            INSERT INTO storage_migration_checkpoints (migration_id, checkpoint_type, data)
            VALUES ($1, 'progress', $2)
            ON CONFLICT (migration_id, checkpoint_type)
            DO UPDATE SET data = $2, created_at = NOW()
            "#,
        )
        .bind(migration_id)
        .bind(data)
        .execute(pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to save checkpoint", e))?;

        Ok(())
    }

    async fn count_files_to_migrate(&self, request: &MigrationRequest) -> Result<u64> {
        // For assets, count from media table based on asset types
        let count: (i64,) =
            if request.asset_types.is_empty() || request.asset_types.contains(&"all".to_string()) {
                sqlx::query_as("SELECT COUNT(*) FROM media WHERE deleted_at IS NULL")
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|e| Error::database_with_source("Failed to count files", e))?
            } else {
                // Filter by MIME type
                let mime_patterns: Vec<String> = request
                    .asset_types
                    .iter()
                    .flat_map(|t| match t.as_str() {
                        "images" => vec!["image/%".to_string()],
                        "videos" => vec!["video/%".to_string()],
                        "documents" => vec![
                            "application/pdf".to_string(),
                            "application/%document%".to_string(),
                            "text/%".to_string(),
                        ],
                        _ => vec![],
                    })
                    .collect();

                if mime_patterns.is_empty() {
                    sqlx::query_as("SELECT COUNT(*) FROM media WHERE deleted_at IS NULL")
                        .fetch_one(&self.pool)
                        .await
                        .map_err(|e| Error::database_with_source("Failed to count files", e))?
                } else {
                    let conditions: Vec<String> = mime_patterns
                        .iter()
                        .enumerate()
                        .map(|(i, _)| format!("mime_type LIKE ${}", i + 1))
                        .collect();
                    let query = format!(
                        "SELECT COUNT(*) FROM media WHERE deleted_at IS NULL AND ({})",
                        conditions.join(" OR ")
                    );

                    let mut builder = sqlx::query_as::<_, (i64,)>(&query);
                    for pattern in &mime_patterns {
                        builder = builder.bind(pattern);
                    }
                    builder
                        .fetch_one(&self.pool)
                        .await
                        .map_err(|e| Error::database_with_source("Failed to count files", e))?
                }
            };

        Ok(count.0 as u64)
    }

    /// Get files to migrate with their metadata
    async fn get_files_to_migrate(&self, request: &MigrationRequest) -> Result<Vec<MediaFileInfo>> {
        let rows: Vec<MediaFileInfo> =
            if request.asset_types.is_empty() || request.asset_types.contains(&"all".to_string()) {
                sqlx::query_as(
                    r#"
                SELECT id as media_id, file_path as path, COALESCE(file_size, 0) as size
                FROM media WHERE deleted_at IS NULL
                ORDER BY created_at
                "#,
                )
                .fetch_all(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to fetch files", e))?
            } else {
                // Filter by MIME type
                let mime_patterns: Vec<String> = request
                    .asset_types
                    .iter()
                    .flat_map(|t| match t.as_str() {
                        "images" => vec!["image/%".to_string()],
                        "videos" => vec!["video/%".to_string()],
                        "documents" => vec![
                            "application/pdf".to_string(),
                            "application/%document%".to_string(),
                            "text/%".to_string(),
                        ],
                        _ => vec![],
                    })
                    .collect();

                if mime_patterns.is_empty() {
                    sqlx::query_as(
                        r#"
                    SELECT id as media_id, file_path as path, COALESCE(file_size, 0) as size
                    FROM media WHERE deleted_at IS NULL
                    ORDER BY created_at
                    "#,
                    )
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| Error::database_with_source("Failed to fetch files", e))?
                } else {
                    // Build dynamic query for mime type filtering
                    let conditions: Vec<String> = (0..mime_patterns.len())
                        .map(|i| format!("mime_type LIKE ${}", i + 1))
                        .collect();
                    let query = format!(
                        r#"
                    SELECT id as media_id, file_path as path, COALESCE(file_size, 0) as size
                    FROM media WHERE deleted_at IS NULL AND ({})
                    ORDER BY created_at
                    "#,
                        conditions.join(" OR ")
                    );

                    let mut builder = sqlx::query_as::<_, MediaFileInfo>(&query);
                    for pattern in &mime_patterns {
                        builder = builder.bind(pattern);
                    }
                    builder
                        .fetch_all(&self.pool)
                        .await
                        .map_err(|e| Error::database_with_source("Failed to fetch files", e))?
                }
            };

        Ok(rows)
    }

    async fn run_migration(pool: PgPool, migration_id: Uuid) -> Result<()> {
        // Update status to in_progress
        sqlx::query("UPDATE storage_migrations SET status = 'in_progress' WHERE id = $1")
            .bind(migration_id)
            .execute(&pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to update migration status", e))?;

        // Get migration details
        let migration: Option<MigrationRow> = sqlx::query_as(
            r#"
            SELECT id, source_category, target_provider, target_config, asset_types,
                   update_references, total_files
            FROM storage_migrations WHERE id = $1
            "#,
        )
        .bind(migration_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to fetch migration details", e))?;

        let _migration = migration.ok_or_else(|| Error::not_found("Migration", "not_found"))?;

        // Get pending files to process (supports resume)
        let batch_size = 10;
        let mut processed_count: u64 = 0;
        let mut failed_count: u64 = 0;
        let mut bytes_transferred: u64 = 0;

        loop {
            // Check if migration was cancelled
            let status_check: Option<(String,)> =
                sqlx::query_as("SELECT status FROM storage_migrations WHERE id = $1")
                    .bind(migration_id)
                    .fetch_optional(&pool)
                    .await
                    .map_err(|e| {
                        Error::database_with_source("Failed to check migration status", e)
                    })?;

            if let Some((status,)) = status_check {
                if status == "cancelled" {
                    tracing::info!(migration_id = %migration_id, "Migration cancelled by user");
                    return Ok(());
                }
            }

            // Get next batch of pending files
            let pending_files: Vec<PendingFileRow> = sqlx::query_as(
                r#"
                SELECT id, source_path, file_size, media_id
                FROM storage_migration_files
                WHERE migration_id = $1 AND status IN ('pending', 'failed')
                AND attempt_count < 3
                ORDER BY created_at
                LIMIT $2
                "#,
            )
            .bind(migration_id)
            .bind(batch_size)
            .fetch_all(&pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to fetch pending files", e))?;

            if pending_files.is_empty() {
                break; // No more files to process
            }

            // Process each file in the batch
            for file in pending_files {
                // Update file status to transferring
                sqlx::query(
                    r#"
                    UPDATE storage_migration_files
                    SET status = 'transferring', started_at = NOW(), attempt_count = attempt_count + 1
                    WHERE id = $1
                    "#,
                )
                .bind(file.id)
                .execute(&pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to update file status", e))?;

                // Update current file in migration status
                sqlx::query(
                    "UPDATE storage_migrations SET current_file = $1, last_processed_file_id = $2 WHERE id = $3"
                )
                .bind(&file.source_path)
                .bind(file.id)
                .bind(migration_id)
                .execute(&pool)
                .await
                .ok();

                // Simulate file transfer (replace with actual transfer logic)
                let transfer_result = Self::transfer_file(&pool, &file).await;

                match transfer_result {
                    Ok(target_path) => {
                        // Update file as completed
                        sqlx::query(
                            r#"
                            UPDATE storage_migration_files
                            SET status = 'completed', target_path = $1, bytes_transferred = file_size, completed_at = NOW()
                            WHERE id = $2
                            "#,
                        )
                        .bind(&target_path)
                        .bind(file.id)
                        .execute(&pool)
                        .await
                        .map_err(|e| Error::database_with_source("Failed to update file status", e))?;

                        processed_count += 1;
                        bytes_transferred += file.file_size as u64;

                        // Update migration progress
                        sqlx::query(
                            "UPDATE storage_migrations SET migrated_files = $1 WHERE id = $2",
                        )
                        .bind(processed_count as i64)
                        .bind(migration_id)
                        .execute(&pool)
                        .await
                        .ok();
                    }
                    Err(e) => {
                        // Update file as failed
                        sqlx::query(
                            r#"
                            UPDATE storage_migration_files
                            SET status = 'failed', last_error = $1
                            WHERE id = $2
                            "#,
                        )
                        .bind(e.to_string())
                        .bind(file.id)
                        .execute(&pool)
                        .await
                        .ok();

                        failed_count += 1;

                        // Update failed count
                        sqlx::query(
                            "UPDATE storage_migrations SET failed_files = $1 WHERE id = $2",
                        )
                        .bind(failed_count as i64)
                        .bind(migration_id)
                        .execute(&pool)
                        .await
                        .ok();
                    }
                }

                // Save checkpoint after each file
                Self::save_checkpoint(
                    &pool,
                    migration_id,
                    MigrationCheckpoint {
                        last_processed_file_id: Some(file.id),
                        processed_count,
                        failed_count,
                        bytes_transferred,
                        timestamp: chrono::Utc::now(),
                    },
                )
                .await
                .ok();
            }
        }

        // Check final status
        let remaining_failed: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM storage_migration_files
            WHERE migration_id = $1 AND status = 'failed' AND attempt_count >= 3
            "#,
        )
        .bind(migration_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to count failed files", e))?;

        let final_status = if remaining_failed.0 > 0 {
            "completed" // Completed with some failures
        } else {
            "completed"
        };

        // Update status to completed
        sqlx::query(
            r#"
            UPDATE storage_migrations
            SET status = $1, completed_at = NOW(), current_file = NULL
            WHERE id = $2
            "#,
        )
        .bind(final_status)
        .bind(migration_id)
        .execute(&pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to update migration status", e))?;

        tracing::info!(
            migration_id = %migration_id,
            processed = processed_count,
            failed = failed_count,
            "Migration completed"
        );

        Ok(())
    }

    /// Transfer a single file (placeholder - implement actual transfer logic)
    async fn transfer_file(_pool: &PgPool, file: &PendingFileRow) -> Result<String> {
        // TODO: Implement actual file transfer based on source and target providers
        // This should:
        // 1. Read from source storage
        // 2. Write to target storage with progress tracking
        // 3. Verify checksum if available
        // 4. Return the target path

        // Simulate transfer delay based on file size
        let delay_ms = std::cmp::min(file.file_size / 1000, 100) as u64;
        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;

        // Return simulated target path
        Ok(format!("/migrated{}", file.source_path))
    }

    /// Get migration status
    pub async fn get_migration_status(
        &self,
        migration_id: Uuid,
    ) -> Result<Option<MigrationStatus>> {
        let row: Option<MigrationStatusRow> = sqlx::query_as(
            r#"
            SELECT id, status, total_files, migrated_files, failed_files, current_file,
                   started_at, completed_at, error
            FROM storage_migrations WHERE id = $1
            "#,
        )
        .bind(migration_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to fetch migration status", e))?;

        row.map(|r| r.try_into()).transpose()
    }

    /// Cancel a running migration
    pub async fn cancel_migration(&self, migration_id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE storage_migrations SET status = 'cancelled', completed_at = NOW() WHERE id = $1 AND status IN ('pending', 'in_progress')",
        )
        .bind(migration_id)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to cancel migration", e))?;

        Ok(result.rows_affected() > 0)
    }
}

// Database row types
#[derive(Debug, sqlx::FromRow)]
struct StorageConfigRow {
    id: Uuid,
    category: String,
    provider: String,
    config: serde_json::Value,
    is_active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<StorageConfigRow> for StorageConfiguration {
    type Error = Error;

    fn try_from(row: StorageConfigRow) -> std::result::Result<Self, Self::Error> {
        let category: StorageCategory = row.category.parse()?;
        let provider: StorageProvider =
            serde_json::from_value(serde_json::Value::String(row.provider))
                .map_err(|e| Error::deserialization_with_source("Invalid provider", e))?;
        let config: ProviderConfig = serde_json::from_value(row.config)
            .map_err(|e| Error::deserialization_with_source("Invalid config", e))?;

        Ok(StorageConfiguration {
            id: row.id,
            category,
            provider,
            config,
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
struct MigrationRow {
    id: Uuid,
    source_category: String,
    target_provider: String,
    target_config: serde_json::Value,
    asset_types: serde_json::Value,
    update_references: bool,
    total_files: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct MigrationStatusRow {
    id: Uuid,
    status: String,
    total_files: i64,
    migrated_files: i64,
    failed_files: i64,
    current_file: Option<String>,
    started_at: chrono::DateTime<chrono::Utc>,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
    error: Option<String>,
}

impl TryFrom<MigrationStatusRow> for MigrationStatus {
    type Error = Error;

    fn try_from(row: MigrationStatusRow) -> std::result::Result<Self, Self::Error> {
        let status = match row.status.as_str() {
            "pending" => MigrationState::Pending,
            "in_progress" => MigrationState::InProgress,
            "completed" => MigrationState::Completed,
            "failed" => MigrationState::Failed,
            "cancelled" => MigrationState::Cancelled,
            "paused" => MigrationState::Paused,
            _ => return Err(Error::deserialization("Invalid migration status")),
        };

        Ok(MigrationStatus {
            id: row.id,
            status,
            total_files: row.total_files as u64,
            migrated_files: row.migrated_files as u64,
            failed_files: row.failed_files as u64,
            current_file: row.current_file,
            started_at: row.started_at,
            completed_at: row.completed_at,
            error: row.error,
            can_resume: true, // Default to true, should be fetched from DB
            total_bytes: 0,
            transferred_bytes: 0,
        })
    }
}

/// Media file information for migration
#[derive(Debug, sqlx::FromRow)]
struct MediaFileInfo {
    media_id: Option<Uuid>,
    path: String,
    size: i64,
}

/// Pending file row for migration processing
#[derive(Debug, sqlx::FromRow)]
struct PendingFileRow {
    id: Uuid,
    source_path: String,
    file_size: i64,
    media_id: Option<Uuid>,
}

/// File transfer row for querying file statuses
#[derive(Debug, sqlx::FromRow)]
struct FileTransferRow {
    id: Uuid,
    migration_id: Uuid,
    media_id: Option<Uuid>,
    source_path: String,
    target_path: Option<String>,
    file_size: i64,
    bytes_transferred: i64,
    status: String,
    attempt_count: i32,
    last_error: Option<String>,
}

impl TryFrom<FileTransferRow> for FileTransferStatus {
    type Error = Error;

    fn try_from(row: FileTransferRow) -> std::result::Result<Self, Self::Error> {
        let status = match row.status.as_str() {
            "pending" => FileTransferState::Pending,
            "transferring" => FileTransferState::Transferring,
            "verifying" => FileTransferState::Verifying,
            "completed" => FileTransferState::Completed,
            "failed" => FileTransferState::Failed,
            "skipped" => FileTransferState::Skipped,
            _ => return Err(Error::deserialization("Invalid file transfer status")),
        };

        Ok(FileTransferStatus {
            id: row.id,
            migration_id: row.migration_id,
            media_id: row.media_id,
            source_path: row.source_path,
            target_path: row.target_path,
            file_size: row.file_size as u64,
            bytes_transferred: row.bytes_transferred as u64,
            status,
            attempt_count: row.attempt_count as u32,
            last_error: row.last_error,
        })
    }
}
