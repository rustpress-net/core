//! PlanetScale Database Provider
//!
//! Integration with PlanetScale serverless MySQL database.
//! Supports branching, non-blocking schema changes, and horizontal scaling.
//!
//! # Configuration
//!
//! ```yaml
//! database:
//!   provider: planetscale
//!   host: aws.connect.psdb.cloud
//!   username: your-username
//!   password: your-password
//!   database: your-database
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions, MySqlSslMode};
use sqlx::MySqlPool;
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::{DatabaseError, DatabaseProvider, Result};

/// PlanetScale configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlanetScaleConfig {
    /// Database host (e.g., aws.connect.psdb.cloud)
    pub host: String,

    /// Database username
    pub username: String,

    /// Database password
    pub password: String,

    /// Database name
    pub database: String,

    /// Use SSL (required for PlanetScale)
    #[serde(default = "default_use_ssl")]
    pub use_ssl: bool,

    /// Connection pool settings
    #[serde(default)]
    pub pool: PoolConfig,

    /// Branch name (optional, for branch-based development)
    #[serde(default)]
    pub branch: Option<String>,
}

fn default_use_ssl() -> bool {
    true
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

    /// Max lifetime in seconds
    #[serde(default = "default_max_lifetime")]
    pub max_lifetime: u64,
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
fn default_max_lifetime() -> u64 {
    1800
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_connections: default_min_connections(),
            max_connections: default_max_connections(),
            connect_timeout: default_connect_timeout(),
            idle_timeout: default_idle_timeout(),
            max_lifetime: default_max_lifetime(),
        }
    }
}

/// PlanetScale database provider
pub struct PlanetScaleProvider {
    config: PlanetScaleConfig,
    pool: Option<MySqlPool>,
}

impl PlanetScaleProvider {
    /// Create a new PlanetScale provider
    pub fn new(config: PlanetScaleConfig) -> Self {
        Self { config, pool: None }
    }

    /// Build connection options
    fn build_connect_options(&self) -> MySqlConnectOptions {
        let mut options = MySqlConnectOptions::new()
            .host(&self.config.host)
            .port(3306)
            .username(&self.config.username)
            .password(&self.config.password)
            .database(&self.config.database);

        if self.config.use_ssl {
            options = options.ssl_mode(MySqlSslMode::Required);
        }

        options
    }

    /// Build connection string for external tools
    pub fn connection_string(&self) -> String {
        format!(
            "mysql://{}:{}@{}:3306/{}?ssl-mode=REQUIRED",
            self.config.username,
            self.config.password,
            self.config.host,
            self.config.database
        )
    }

    /// Get PlanetScale API client for branch operations
    pub fn api_client(&self, api_token: &str) -> PlanetScaleApiClient {
        PlanetScaleApiClient {
            base_url: "https://api.planetscale.com/v1".to_string(),
            token: api_token.to_string(),
            organization: None,
            database: self.config.database.clone(),
        }
    }
}

#[async_trait]
impl DatabaseProvider for PlanetScaleProvider {
    async fn connect(&mut self) -> Result<()> {
        info!("Connecting to PlanetScale database...");

        let options = self.build_connect_options();

        let pool = MySqlPoolOptions::new()
            .min_connections(self.config.pool.min_connections)
            .max_connections(self.config.pool.max_connections)
            .acquire_timeout(Duration::from_secs(self.config.pool.connect_timeout))
            .idle_timeout(Duration::from_secs(self.config.pool.idle_timeout))
            .max_lifetime(Duration::from_secs(self.config.pool.max_lifetime))
            .connect_with(options)
            .await
            .map_err(|e| DatabaseError::Connection(e.to_string()))?;

        // Test connection
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .map_err(|e| DatabaseError::Connection(e.to_string()))?;

        self.pool = Some(pool);
        info!("Connected to PlanetScale successfully");

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        if let Some(pool) = self.pool.take() {
            pool.close().await;
            info!("Disconnected from PlanetScale");
        }
        Ok(())
    }

    fn pool(&self) -> Option<&MySqlPool> {
        self.pool.as_ref()
    }

    async fn health_check(&self) -> Result<bool> {
        if let Some(pool) = &self.pool {
            match sqlx::query("SELECT 1").execute(pool).await {
                Ok(_) => Ok(true),
                Err(e) => {
                    warn!("PlanetScale health check failed: {}", e);
                    Ok(false)
                }
            }
        } else {
            Ok(false)
        }
    }

    fn provider_name(&self) -> &str {
        "planetscale"
    }
}

/// PlanetScale API client for database management
pub struct PlanetScaleApiClient {
    base_url: String,
    token: String,
    organization: Option<String>,
    database: String,
}

impl PlanetScaleApiClient {
    /// Set organization
    pub fn with_organization(mut self, org: &str) -> Self {
        self.organization = Some(org.to_string());
        self
    }

    /// List branches
    pub async fn list_branches(&self) -> Result<Vec<Branch>> {
        let org = self.organization.as_ref().ok_or_else(|| {
            DatabaseError::Configuration("Organization not set".to_string())
        })?;

        let client = reqwest::Client::new();
        let response = client
            .get(&format!(
                "{}/organizations/{}/databases/{}/branches",
                self.base_url, org, self.database
            ))
            .header("Authorization", &self.token)
            .send()
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        let branches: BranchListResponse = response
            .json()
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(branches.data)
    }

    /// Create a new branch
    pub async fn create_branch(&self, name: &str, parent: &str) -> Result<Branch> {
        let org = self.organization.as_ref().ok_or_else(|| {
            DatabaseError::Configuration("Organization not set".to_string())
        })?;

        let client = reqwest::Client::new();
        let response = client
            .post(&format!(
                "{}/organizations/{}/databases/{}/branches",
                self.base_url, org, self.database
            ))
            .header("Authorization", &self.token)
            .json(&serde_json::json!({
                "name": name,
                "parent_branch": parent
            }))
            .send()
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        response
            .json()
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))
    }

    /// Create deploy request (for schema changes)
    pub async fn create_deploy_request(
        &self,
        branch: &str,
        into_branch: &str,
    ) -> Result<DeployRequest> {
        let org = self.organization.as_ref().ok_or_else(|| {
            DatabaseError::Configuration("Organization not set".to_string())
        })?;

        let client = reqwest::Client::new();
        let response = client
            .post(&format!(
                "{}/organizations/{}/databases/{}/deploy-requests",
                self.base_url, org, self.database
            ))
            .header("Authorization", &self.token)
            .json(&serde_json::json!({
                "branch": branch,
                "into_branch": into_branch
            }))
            .send()
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        response
            .json()
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))
    }

    /// Get connection string for branch
    pub async fn get_branch_connection(
        &self,
        branch: &str,
    ) -> Result<BranchConnection> {
        let org = self.organization.as_ref().ok_or_else(|| {
            DatabaseError::Configuration("Organization not set".to_string())
        })?;

        let client = reqwest::Client::new();
        let response = client
            .post(&format!(
                "{}/organizations/{}/databases/{}/branches/{}/passwords",
                self.base_url, org, self.database, branch
            ))
            .header("Authorization", &self.token)
            .json(&serde_json::json!({
                "role": "readwriter"
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

/// Branch information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub parent_branch: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub production: bool,
    pub ready: bool,
}

/// Branch list response
#[derive(Debug, Deserialize)]
struct BranchListResponse {
    data: Vec<Branch>,
}

/// Deploy request information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployRequest {
    pub id: String,
    pub branch: String,
    pub into_branch: String,
    pub state: String,
    pub created_at: String,
}

/// Branch connection credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchConnection {
    pub id: String,
    pub host: String,
    pub username: String,
    pub password: String,
    pub database: String,
}

impl BranchConnection {
    /// Build connection string
    pub fn connection_string(&self) -> String {
        format!(
            "mysql://{}:{}@{}:3306/{}?ssl-mode=REQUIRED",
            self.username, self.password, self.host, self.database
        )
    }

    /// Build PlanetScale config
    pub fn to_config(&self) -> PlanetScaleConfig {
        PlanetScaleConfig {
            host: self.host.clone(),
            username: self.username.clone(),
            password: self.password.clone(),
            database: self.database.clone(),
            use_ssl: true,
            pool: PoolConfig::default(),
            branch: None,
        }
    }
}

/// PlanetScale migration helper
pub struct PlanetScaleMigrator {
    api: PlanetScaleApiClient,
}

impl PlanetScaleMigrator {
    /// Create a new migrator
    pub fn new(api: PlanetScaleApiClient) -> Self {
        Self { api }
    }

    /// Run migrations using branch workflow
    ///
    /// 1. Create a development branch from main
    /// 2. Apply schema changes to dev branch
    /// 3. Create deploy request
    /// 4. Merge changes to production (non-blocking)
    pub async fn run_branch_migration(
        &self,
        migration_name: &str,
        schema_sql: &str,
    ) -> Result<DeployRequest> {
        info!("Starting branch migration: {}", migration_name);

        // Create development branch
        let branch_name = format!("migration-{}", migration_name);
        let branch = self
            .api
            .create_branch(&branch_name, "main")
            .await?;

        info!("Created branch: {}", branch.name);

        // Wait for branch to be ready
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Get connection for branch
        let conn = self.api.get_branch_connection(&branch_name).await?;

        // Apply schema changes
        let config = conn.to_config();
        let mut provider = PlanetScaleProvider::new(config);
        provider.connect().await?;

        if let Some(pool) = provider.pool() {
            sqlx::query(schema_sql)
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        provider.disconnect().await?;

        info!("Schema changes applied to branch");

        // Create deploy request
        let deploy_request = self
            .api
            .create_deploy_request(&branch_name, "main")
            .await?;

        info!("Deploy request created: {}", deploy_request.id);

        Ok(deploy_request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planetscale_config() {
        let config = PlanetScaleConfig {
            host: "aws.connect.psdb.cloud".to_string(),
            username: "test".to_string(),
            password: "test".to_string(),
            database: "rustpress".to_string(),
            use_ssl: true,
            pool: PoolConfig::default(),
            branch: None,
        };

        let provider = PlanetScaleProvider::new(config);
        assert_eq!(provider.provider_name(), "planetscale");
    }

    #[test]
    fn test_connection_string() {
        let config = PlanetScaleConfig {
            host: "aws.connect.psdb.cloud".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
            database: "mydb".to_string(),
            use_ssl: true,
            pool: PoolConfig::default(),
            branch: None,
        };

        let provider = PlanetScaleProvider::new(config);
        let conn_str = provider.connection_string();

        assert!(conn_str.contains("mysql://"));
        assert!(conn_str.contains("ssl-mode=REQUIRED"));
    }
}
