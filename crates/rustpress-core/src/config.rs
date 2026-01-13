//! Configuration system for RustPress using config-rs.
//!
//! Supports TOML, YAML, and environment variable configuration.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Server configuration
    pub server: ServerConfig,
    /// Database configuration
    pub database: DatabaseConfig,
    /// Cache configuration
    pub cache: CacheConfig,
    /// Authentication configuration
    pub auth: AuthConfig,
    /// Storage configuration
    pub storage: StorageConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Metrics configuration
    pub metrics: MetricsConfig,
    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,
    /// Multi-tenancy configuration
    pub multitenancy: MultitenancyConfig,
    /// Job queue configuration
    pub jobs: JobConfig,
    /// API configuration
    pub api: ApiConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            cache: CacheConfig::default(),
            auth: AuthConfig::default(),
            storage: StorageConfig::default(),
            logging: LoggingConfig::default(),
            metrics: MetricsConfig::default(),
            rate_limit: RateLimitConfig::default(),
            multitenancy: MultitenancyConfig::default(),
            jobs: JobConfig::default(),
            api: ApiConfig::default(),
        }
    }
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Host to bind to
    pub host: String,
    /// Port to listen on
    pub port: u16,
    /// Number of worker threads
    pub workers: usize,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
    /// Maximum body size in bytes
    pub max_body_size: usize,
    /// Enable TLS
    pub tls_enabled: bool,
    /// TLS certificate path
    pub tls_cert_path: Option<PathBuf>,
    /// TLS key path
    pub tls_key_path: Option<PathBuf>,
    /// Graceful shutdown timeout in seconds
    pub shutdown_timeout_secs: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            workers: num_cpus::get(),
            request_timeout_secs: 30,
            max_body_size: 10 * 1024 * 1024, // 10MB
            tls_enabled: false,
            tls_cert_path: None,
            tls_key_path: None,
            shutdown_timeout_secs: 30,
        }
    }
}

impl ServerConfig {
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.request_timeout_secs)
    }

    pub fn shutdown_timeout(&self) -> Duration {
        Duration::from_secs(self.shutdown_timeout_secs)
    }
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database URL
    pub url: String,
    /// Minimum connection pool size
    pub pool_min: u32,
    /// Maximum connection pool size
    pub pool_max: u32,
    /// Connection timeout in seconds
    pub connect_timeout_secs: u64,
    /// Idle timeout in seconds
    pub idle_timeout_secs: u64,
    /// Maximum lifetime of a connection in seconds
    pub max_lifetime_secs: u64,
    /// Enable statement caching
    pub statement_cache_size: usize,
    /// Run migrations on startup
    pub run_migrations: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://localhost/rustpress".to_string(),
            pool_min: 2,
            pool_max: 10,
            connect_timeout_secs: 10,
            idle_timeout_secs: 600,
            max_lifetime_secs: 1800,
            statement_cache_size: 100,
            run_migrations: true,
        }
    }
}

impl DatabaseConfig {
    pub fn connect_timeout(&self) -> Duration {
        Duration::from_secs(self.connect_timeout_secs)
    }

    pub fn idle_timeout(&self) -> Duration {
        Duration::from_secs(self.idle_timeout_secs)
    }

    pub fn max_lifetime(&self) -> Duration {
        Duration::from_secs(self.max_lifetime_secs)
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Cache backend type
    pub backend: CacheBackend,
    /// Redis URL (if using Redis)
    pub redis_url: Option<String>,
    /// Default TTL in seconds
    pub default_ttl_secs: u64,
    /// Maximum memory for in-memory cache in MB
    pub max_memory_mb: usize,
    /// Enable cache metrics
    pub enable_metrics: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CacheBackend {
    Memory,
    Redis,
    Hybrid, // Memory + Redis
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            backend: CacheBackend::Memory,
            redis_url: None,
            default_ttl_secs: 3600,
            max_memory_mb: 256,
            enable_metrics: true,
        }
    }
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// JWT secret key
    pub jwt_secret: String,
    /// JWT access token expiry in seconds
    pub jwt_access_expiry_secs: u64,
    /// JWT refresh token expiry in seconds
    pub jwt_refresh_expiry_secs: u64,
    /// JWT issuer
    pub jwt_issuer: String,
    /// Password minimum length
    pub password_min_length: usize,
    /// Require uppercase in password
    pub password_require_uppercase: bool,
    /// Require lowercase in password
    pub password_require_lowercase: bool,
    /// Require digit in password
    pub password_require_digit: bool,
    /// Require special character in password
    pub password_require_special: bool,
    /// Maximum login attempts before lockout
    pub max_login_attempts: u32,
    /// Lockout duration in seconds
    pub lockout_duration_secs: u64,
    /// Session timeout in seconds
    pub session_timeout_secs: u64,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "change-me-in-production".to_string(),
            jwt_access_expiry_secs: 900, // 15 minutes
            jwt_refresh_expiry_secs: 604800, // 7 days
            jwt_issuer: "rustpress".to_string(),
            password_min_length: 8,
            password_require_uppercase: true,
            password_require_lowercase: true,
            password_require_digit: true,
            password_require_special: false,
            max_login_attempts: 5,
            lockout_duration_secs: 900, // 15 minutes
            session_timeout_secs: 86400, // 24 hours
        }
    }
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Storage backend type
    pub backend: StorageBackend,
    /// Local storage path
    pub local_path: PathBuf,
    /// S3 bucket name
    pub s3_bucket: Option<String>,
    /// S3 region
    pub s3_region: Option<String>,
    /// S3 endpoint (for MinIO, etc.)
    pub s3_endpoint: Option<String>,
    /// Maximum upload size in bytes
    pub max_upload_size: usize,
    /// Allowed file types
    pub allowed_types: Vec<String>,
    /// CDN URL prefix
    pub cdn_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    Local,
    S3,
    Azure,
    Gcs,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: StorageBackend::Local,
            local_path: PathBuf::from("./uploads"),
            s3_bucket: None,
            s3_region: None,
            s3_endpoint: None,
            max_upload_size: 50 * 1024 * 1024, // 50MB
            allowed_types: vec![
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "image/gif".to_string(),
                "image/webp".to_string(),
                "application/pdf".to_string(),
                "video/mp4".to_string(),
                "audio/mpeg".to_string(),
            ],
            cdn_url: None,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,
    /// Log format
    pub format: LogFormat,
    /// Output to stdout
    pub stdout: bool,
    /// Log file path
    pub file_path: Option<PathBuf>,
    /// Enable request logging
    pub log_requests: bool,
    /// Log request bodies
    pub log_request_body: bool,
    /// Log response bodies
    pub log_response_body: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Text,
    Json,
    Pretty,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Text,
            stdout: true,
            file_path: None,
            log_requests: true,
            log_request_body: false,
            log_response_body: false,
        }
    }
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,
    /// Metrics endpoint path
    pub endpoint: String,
    /// Include default process metrics
    pub include_process_metrics: bool,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "/metrics".to_string(),
            include_process_metrics: true,
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    pub enabled: bool,
    /// Requests per window
    pub requests_per_window: u32,
    /// Window size in seconds
    pub window_secs: u64,
    /// Rate limit by IP
    pub by_ip: bool,
    /// Rate limit by user
    pub by_user: bool,
    /// Rate limit by API key
    pub by_api_key: bool,
    /// Endpoints exempt from rate limiting
    pub exempt_paths: Vec<String>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            requests_per_window: 100,
            window_secs: 60,
            by_ip: true,
            by_user: true,
            by_api_key: true,
            exempt_paths: vec!["/health".to_string(), "/metrics".to_string(), "/api/v4".to_string(), "/admin".to_string()],
        }
    }
}

/// Multi-tenancy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultitenancyConfig {
    /// Enable multi-tenancy
    pub enabled: bool,
    /// Tenant isolation mode
    pub isolation: TenantIsolation,
    /// Tenant identification strategy
    pub identification: TenantIdentification,
    /// Default tenant ID
    pub default_tenant: Option<String>,
    /// Maximum tenants allowed (None = unlimited)
    pub max_tenants: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TenantIsolation {
    /// Shared database, discriminator column
    SharedDatabase,
    /// Separate schema per tenant
    SeparateSchema,
    /// Separate database per tenant
    SeparateDatabase,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TenantIdentification {
    /// Identify by subdomain
    Subdomain,
    /// Identify by HTTP header
    Header,
    /// Identify by path prefix
    Path,
    /// Identify by JWT claim
    Jwt,
}

impl Default for MultitenancyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            isolation: TenantIsolation::SharedDatabase,
            identification: TenantIdentification::Subdomain,
            default_tenant: Some("default".to_string()),
            max_tenants: None,
        }
    }
}

/// Job queue configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobConfig {
    /// Enable job processing
    pub enabled: bool,
    /// Number of worker threads
    pub workers: usize,
    /// Maximum retries for failed jobs
    pub max_retries: u32,
    /// Retry delay in seconds
    pub retry_delay_secs: u64,
    /// Job timeout in seconds
    pub timeout_secs: u64,
    /// Backend for job storage
    pub backend: JobBackend,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JobBackend {
    Database,
    Redis,
}

impl Default for JobConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            workers: 4,
            max_retries: 3,
            retry_delay_secs: 60,
            timeout_secs: 300,
            backend: JobBackend::Database,
        }
    }
}

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// API prefix
    pub prefix: String,
    /// Default API version
    pub default_version: String,
    /// Supported API versions
    pub supported_versions: Vec<String>,
    /// Enable CORS
    pub cors_enabled: bool,
    /// Allowed CORS origins
    pub cors_origins: Vec<String>,
    /// Enable API documentation
    pub docs_enabled: bool,
    /// Documentation path
    pub docs_path: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            prefix: "/api".to_string(),
            default_version: "v1".to_string(),
            supported_versions: vec!["v1".to_string()],
            cors_enabled: true,
            cors_origins: vec!["*".to_string()],
            docs_enabled: true,
            docs_path: "/docs".to_string(),
        }
    }
}

// Helper function to get number of CPUs
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.pool_max, 10);
        assert!(config.metrics.enabled);
    }

    #[test]
    fn test_server_address() {
        let config = ServerConfig::default();
        assert_eq!(config.address(), "127.0.0.1:8080");
    }

    #[test]
    fn test_serialization() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.server.port, deserialized.server.port);
    }
}
