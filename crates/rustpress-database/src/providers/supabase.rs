//! Supabase Database Provider
//!
//! Integration with Supabase managed PostgreSQL database.
//! Supports connection pooling, real-time subscriptions, and Row Level Security.
//!
//! # Configuration
//!
//! ```yaml
//! database:
//!   provider: supabase
//!   project_url: https://your-project.supabase.co
//!   anon_key: your-anon-key
//!   service_role_key: your-service-role-key  # For admin operations
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use sqlx::PgPool;
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::{DatabaseError, DatabaseProvider, Result};

/// Supabase configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SupabaseConfig {
    /// Supabase project URL (e.g., https://xxx.supabase.co)
    pub project_url: String,

    /// Supabase anon/public key
    pub anon_key: String,

    /// Supabase service role key (for admin operations)
    #[serde(default)]
    pub service_role_key: Option<String>,

    /// Database password (from Supabase dashboard)
    pub database_password: String,

    /// Use connection pooler (recommended for serverless)
    #[serde(default = "default_use_pooler")]
    pub use_pooler: bool,

    /// Pooler mode: transaction or session
    #[serde(default = "default_pooler_mode")]
    pub pooler_mode: String,

    /// Connection pool settings
    #[serde(default)]
    pub pool: PoolConfig,
}

fn default_use_pooler() -> bool {
    true
}

fn default_pooler_mode() -> String {
    "transaction".to_string()
}

/// Connection pool configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Minimum connections
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,

    /// Maximum connections
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Connection timeout in seconds
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout: u64,

    /// Idle timeout in seconds
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: u64,
}

fn default_min_connections() -> u32 {
    1
}
fn default_max_connections() -> u32 {
    10
}
fn default_connect_timeout() -> u64 {
    30
}
fn default_idle_timeout() -> u64 {
    600
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_connections: default_min_connections(),
            max_connections: default_max_connections(),
            connect_timeout: default_connect_timeout(),
            idle_timeout: default_idle_timeout(),
        }
    }
}

/// Supabase database provider
pub struct SupabaseProvider {
    config: SupabaseConfig,
    pool: Option<PgPool>,
}

impl SupabaseProvider {
    /// Create a new Supabase provider
    pub fn new(config: SupabaseConfig) -> Self {
        Self { config, pool: None }
    }

    /// Parse project reference from URL
    fn get_project_ref(&self) -> Result<String> {
        let url = &self.config.project_url;

        // Extract project ref from URL like https://xxx.supabase.co
        if let Some(host) = url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
        {
            if let Some(project_ref) = host.split('.').next() {
                return Ok(project_ref.to_string());
            }
        }

        Err(DatabaseError::Configuration(
            "Invalid Supabase project URL".to_string(),
        ))
    }

    /// Build database connection string
    fn build_connection_string(&self) -> Result<String> {
        let project_ref = self.get_project_ref()?;

        let host = if self.config.use_pooler {
            // Supavisor connection pooler
            format!(
                "aws-0-us-east-1.pooler.supabase.com"
            )
        } else {
            // Direct connection
            format!("db.{}.supabase.co", project_ref)
        };

        let port = if self.config.use_pooler {
            match self.config.pooler_mode.as_str() {
                "transaction" => 6543,
                "session" => 5432,
                _ => 6543,
            }
        } else {
            5432
        };

        let user = if self.config.use_pooler {
            format!("postgres.{}", project_ref)
        } else {
            "postgres".to_string()
        };

        Ok(format!(
            "postgresql://{}:{}@{}:{}/postgres",
            user, self.config.database_password, host, port
        ))
    }

    /// Build connection options
    fn build_connect_options(&self) -> Result<PgConnectOptions> {
        let project_ref = self.get_project_ref()?;

        let host = if self.config.use_pooler {
            "aws-0-us-east-1.pooler.supabase.com".to_string()
        } else {
            format!("db.{}.supabase.co", project_ref)
        };

        let port = if self.config.use_pooler {
            match self.config.pooler_mode.as_str() {
                "transaction" => 6543,
                "session" => 5432,
                _ => 6543,
            }
        } else {
            5432
        };

        let user = if self.config.use_pooler {
            format!("postgres.{}", project_ref)
        } else {
            "postgres".to_string()
        };

        let options = PgConnectOptions::new()
            .host(&host)
            .port(port)
            .username(&user)
            .password(&self.config.database_password)
            .database("postgres")
            .ssl_mode(PgSslMode::Require)
            .application_name("rustpress");

        Ok(options)
    }

    /// Get Supabase REST API client
    pub fn rest_client(&self) -> SupabaseRestClient {
        SupabaseRestClient {
            base_url: format!("{}/rest/v1", self.config.project_url),
            anon_key: self.config.anon_key.clone(),
            service_role_key: self.config.service_role_key.clone(),
        }
    }

    /// Get Supabase Auth client
    pub fn auth_client(&self) -> SupabaseAuthClient {
        SupabaseAuthClient {
            base_url: format!("{}/auth/v1", self.config.project_url),
            anon_key: self.config.anon_key.clone(),
            service_role_key: self.config.service_role_key.clone(),
        }
    }

    /// Get Supabase Storage client
    pub fn storage_client(&self) -> SupabaseStorageClient {
        SupabaseStorageClient {
            base_url: format!("{}/storage/v1", self.config.project_url),
            anon_key: self.config.anon_key.clone(),
            service_role_key: self.config.service_role_key.clone(),
        }
    }
}

#[async_trait]
impl DatabaseProvider for SupabaseProvider {
    async fn connect(&mut self) -> Result<()> {
        info!("Connecting to Supabase database...");

        let options = self.build_connect_options()?;

        let pool = PgPoolOptions::new()
            .min_connections(self.config.pool.min_connections)
            .max_connections(self.config.pool.max_connections)
            .acquire_timeout(Duration::from_secs(self.config.pool.connect_timeout))
            .idle_timeout(Duration::from_secs(self.config.pool.idle_timeout))
            .connect_with(options)
            .await
            .map_err(|e| DatabaseError::Connection(e.to_string()))?;

        // Test connection
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .map_err(|e| DatabaseError::Connection(e.to_string()))?;

        self.pool = Some(pool);
        info!("Connected to Supabase successfully");

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        if let Some(pool) = self.pool.take() {
            pool.close().await;
            info!("Disconnected from Supabase");
        }
        Ok(())
    }

    fn pool(&self) -> Option<&PgPool> {
        self.pool.as_ref()
    }

    async fn health_check(&self) -> Result<bool> {
        if let Some(pool) = &self.pool {
            match sqlx::query("SELECT 1").execute(pool).await {
                Ok(_) => Ok(true),
                Err(e) => {
                    warn!("Supabase health check failed: {}", e);
                    Ok(false)
                }
            }
        } else {
            Ok(false)
        }
    }

    fn provider_name(&self) -> &str {
        "supabase"
    }
}

/// Supabase REST API client for direct table access
pub struct SupabaseRestClient {
    base_url: String,
    anon_key: String,
    service_role_key: Option<String>,
}

impl SupabaseRestClient {
    /// Make authenticated request
    pub async fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        use_service_role: bool,
    ) -> Result<reqwest::RequestBuilder> {
        let client = reqwest::Client::new();
        let url = format!("{}/{}", self.base_url, path);

        let key = if use_service_role {
            self.service_role_key
                .as_ref()
                .unwrap_or(&self.anon_key)
        } else {
            &self.anon_key
        };

        Ok(client
            .request(method, &url)
            .header("apikey", key)
            .header("Authorization", format!("Bearer {}", key))
            .header("Content-Type", "application/json"))
    }

    /// Select from table
    pub async fn select(&self, table: &str, query: &str) -> Result<serde_json::Value> {
        let response = self
            .request(reqwest::Method::GET, &format!("{}?{}", table, query), false)
            .await?
            .send()
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        response
            .json()
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))
    }

    /// Insert into table
    pub async fn insert(
        &self,
        table: &str,
        data: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let response = self
            .request(reqwest::Method::POST, table, true)
            .await?
            .header("Prefer", "return=representation")
            .json(&data)
            .send()
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        response
            .json()
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))
    }
}

/// Supabase Auth client
pub struct SupabaseAuthClient {
    base_url: String,
    anon_key: String,
    service_role_key: Option<String>,
}

impl SupabaseAuthClient {
    /// Sign up a new user
    pub async fn sign_up(
        &self,
        email: &str,
        password: &str,
    ) -> Result<serde_json::Value> {
        let client = reqwest::Client::new();

        let response = client
            .post(&format!("{}/signup", self.base_url))
            .header("apikey", &self.anon_key)
            .json(&serde_json::json!({
                "email": email,
                "password": password
            }))
            .send()
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        response
            .json()
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))
    }

    /// Sign in with email/password
    pub async fn sign_in(
        &self,
        email: &str,
        password: &str,
    ) -> Result<serde_json::Value> {
        let client = reqwest::Client::new();

        let response = client
            .post(&format!("{}/token?grant_type=password", self.base_url))
            .header("apikey", &self.anon_key)
            .json(&serde_json::json!({
                "email": email,
                "password": password
            }))
            .send()
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        response
            .json()
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))
    }
}

/// Supabase Storage client
pub struct SupabaseStorageClient {
    base_url: String,
    anon_key: String,
    service_role_key: Option<String>,
}

impl SupabaseStorageClient {
    /// Upload file to bucket
    pub async fn upload(
        &self,
        bucket: &str,
        path: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<String> {
        let client = reqwest::Client::new();
        let key = self.service_role_key.as_ref().unwrap_or(&self.anon_key);

        let response = client
            .post(&format!("{}/object/{}/{}", self.base_url, bucket, path))
            .header("apikey", key)
            .header("Authorization", format!("Bearer {}", key))
            .header("Content-Type", content_type)
            .body(data)
            .send()
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        if response.status().is_success() {
            Ok(format!("{}/object/public/{}/{}", self.base_url, bucket, path))
        } else {
            Err(DatabaseError::Query("Upload failed".to_string()))
        }
    }

    /// Get public URL for file
    pub fn get_public_url(&self, bucket: &str, path: &str) -> String {
        format!("{}/object/public/{}/{}", self.base_url, bucket, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supabase_config() {
        let config = SupabaseConfig {
            project_url: "https://test-project.supabase.co".to_string(),
            anon_key: "test-key".to_string(),
            service_role_key: None,
            database_password: "test-password".to_string(),
            use_pooler: true,
            pooler_mode: "transaction".to_string(),
            pool: PoolConfig::default(),
        };

        let provider = SupabaseProvider::new(config);
        assert_eq!(provider.provider_name(), "supabase");
    }

    #[test]
    fn test_get_project_ref() {
        let config = SupabaseConfig {
            project_url: "https://abcdefgh.supabase.co".to_string(),
            anon_key: "test".to_string(),
            service_role_key: None,
            database_password: "test".to_string(),
            use_pooler: true,
            pooler_mode: "transaction".to_string(),
            pool: PoolConfig::default(),
        };

        let provider = SupabaseProvider::new(config);
        assert_eq!(provider.get_project_ref().unwrap(), "abcdefgh");
    }
}
