//! Workers and KV management service

use crate::client::CloudflareClient;
use crate::error::{CloudflareError, CloudflareResult};
use crate::models::*;
use sqlx::PgPool;
use std::sync::Arc;

pub struct WorkersService {
    client: Option<Arc<CloudflareClient>>,
    #[allow(dead_code)]
    db: PgPool,
}

impl WorkersService {
    pub fn new(client: Arc<CloudflareClient>, db: PgPool) -> Self {
        Self { client: Some(client), db }
    }

    /// Create without a configured client (for initial setup)
    pub fn new_unconfigured(db: PgPool) -> Self {
        Self { client: None, db }
    }

    /// Get the client or return an error if not configured
    fn get_client(&self) -> CloudflareResult<&CloudflareClient> {
        self.client.as_ref()
            .map(|c| c.as_ref())
            .ok_or_else(|| CloudflareError::ConfigError("Cloudflare not configured. Please connect your account.".to_string()))
    }

    pub async fn list_workers(&self) -> CloudflareResult<Vec<Worker>> {
        let client = self.get_client()?;
        client.list_workers().await
    }

    pub async fn get_worker(&self, name: &str) -> CloudflareResult<Worker> {
        let client = self.get_client()?;
        client.get_worker(name).await
    }

    pub async fn deploy(&self, name: &str, script: &str) -> CloudflareResult<Worker> {
        let client = self.get_client()?;
        client.deploy_worker(name, script).await
    }

    pub async fn delete(&self, name: &str) -> CloudflareResult<()> {
        let client = self.get_client()?;
        client.delete_worker(name).await?;
        Ok(())
    }

    pub async fn list_routes(&self) -> CloudflareResult<Vec<WorkerRoute>> {
        let client = self.get_client()?;
        client.list_worker_routes().await
    }

    pub async fn create_route(&self, pattern: &str, script: &str) -> CloudflareResult<WorkerRoute> {
        let client = self.get_client()?;
        client.create_worker_route(pattern, script).await
    }

    // KV Operations
    pub async fn list_kv_namespaces(&self) -> CloudflareResult<Vec<KvNamespace>> {
        let client = self.get_client()?;
        client.list_kv_namespaces().await
    }

    pub async fn create_kv_namespace(&self, title: &str) -> CloudflareResult<KvNamespace> {
        let client = self.get_client()?;
        client.create_kv_namespace(title).await
    }

    pub async fn list_kv_keys(&self, namespace_id: &str) -> CloudflareResult<Vec<KvKey>> {
        let client = self.get_client()?;
        client.list_kv_keys(namespace_id).await
    }

    pub async fn get_kv(&self, namespace_id: &str, key: &str) -> CloudflareResult<String> {
        let client = self.get_client()?;
        client.get_kv_value(namespace_id, key).await
    }

    pub async fn set_kv(&self, namespace_id: &str, key: &str, value: &str) -> CloudflareResult<()> {
        let client = self.get_client()?;
        client.set_kv_value(namespace_id, key, value).await
    }

    pub async fn delete_kv(&self, namespace_id: &str, key: &str) -> CloudflareResult<()> {
        let client = self.get_client()?;
        client.delete_kv_value(namespace_id, key).await
    }
}
