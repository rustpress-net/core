//! DNS management service

use crate::client::CloudflareClient;
use crate::error::{CloudflareError, CloudflareResult};
use crate::models::{CreateDnsRecord, DnsListParams, DnsRecord, UpdateDnsRecord, DeleteResponse};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::info;

/// DNS management service
pub struct DnsService {
    client: Option<Arc<CloudflareClient>>,
    db: PgPool,
}

impl DnsService {
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

    /// List all DNS records
    pub async fn list(&self, params: Option<DnsListParams>) -> CloudflareResult<Vec<DnsRecord>> {
        let client = self.get_client()?;
        client.list_dns_records(params).await
    }

    /// Get a DNS record by ID
    pub async fn get(&self, id: &str) -> CloudflareResult<DnsRecord> {
        let client = self.get_client()?;
        let records = client.list_dns_records(None).await?;
        records
            .into_iter()
            .find(|r| r.id == id)
            .ok_or(CloudflareError::NotFound(format!("DNS record {}", id)))
    }

    /// Create a new DNS record
    pub async fn create(&self, record: CreateDnsRecord) -> CloudflareResult<DnsRecord> {
        let client = self.get_client()?;
        info!("Creating DNS record: {} -> {}", record.name, record.content);
        let result = client.create_dns_record(record).await?;
        self.sync_to_local(&result).await?;
        Ok(result)
    }

    /// Update a DNS record
    pub async fn update(&self, id: &str, record: UpdateDnsRecord) -> CloudflareResult<DnsRecord> {
        let client = self.get_client()?;
        info!("Updating DNS record {}: {} -> {}", id, record.name, record.content);
        let result = client.update_dns_record(id, record).await?;
        self.sync_to_local(&result).await?;
        Ok(result)
    }

    /// Delete a DNS record
    pub async fn delete(&self, id: &str) -> CloudflareResult<DeleteResponse> {
        let client = self.get_client()?;
        info!("Deleting DNS record {}", id);
        let result = client.delete_dns_record(id).await?;
        self.delete_from_local(id).await?;
        Ok(result)
    }

    /// Sync DNS record to local database
    async fn sync_to_local(&self, record: &DnsRecord) -> CloudflareResult<()> {
        sqlx::query(
            r#"
            INSERT INTO cloudflare_dns_records (
                cloudflare_id, record_type, name, content, proxied, ttl, priority, synced_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
            ON CONFLICT (cloudflare_id) DO UPDATE SET
                record_type = EXCLUDED.record_type,
                name = EXCLUDED.name,
                content = EXCLUDED.content,
                proxied = EXCLUDED.proxied,
                ttl = EXCLUDED.ttl,
                priority = EXCLUDED.priority,
                synced_at = NOW()
            "#,
        )
        .bind(&record.id)
        .bind(&record.record_type)
        .bind(&record.name)
        .bind(&record.content)
        .bind(record.proxied)
        .bind(record.ttl)
        .bind(record.priority)
        .execute(&self.db)
        .await
        .map_err(|e| CloudflareError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Delete DNS record from local database
    async fn delete_from_local(&self, id: &str) -> CloudflareResult<()> {
        sqlx::query("DELETE FROM cloudflare_dns_records WHERE cloudflare_id = $1")
            .bind(id)
            .execute(&self.db)
            .await
            .map_err(|e| CloudflareError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Full sync from Cloudflare to local
    pub async fn full_sync(&self) -> CloudflareResult<SyncResult> {
        let client = self.get_client()?;
        info!("Starting full DNS sync from Cloudflare");

        let records = client.list_dns_records(None).await?;
        let mut synced = 0;
        let mut errors = 0;

        for record in &records {
            match self.sync_to_local(record).await {
                Ok(_) => synced += 1,
                Err(e) => {
                    tracing::warn!("Failed to sync record {}: {}", record.id, e);
                    errors += 1;
                }
            }
        }

        info!("DNS sync complete: {} synced, {} errors", synced, errors);

        Ok(SyncResult {
            total: records.len(),
            synced,
            errors,
        })
    }

    /// Export DNS records as zone file format
    pub async fn export_zone_file(&self) -> CloudflareResult<String> {
        let client = self.get_client()?;
        let records = client.list_dns_records(None).await?;
        let zone = client.get_zone().await?;

        let mut output = format!("; Zone file for {}\n", zone.name);
        output.push_str(&format!("; Exported at {}\n\n", chrono::Utc::now()));
        output.push_str(&format!("$ORIGIN {}.\n", zone.name));
        output.push_str("$TTL 3600\n\n");

        for record in records {
            let name = if record.name == zone.name {
                "@".to_string()
            } else {
                record.name.replace(&format!(".{}", zone.name), "")
            };

            let proxied = if record.proxied { " ; proxied" } else { "" };

            output.push_str(&format!(
                "{}\t{}\tIN\t{}\t{}{}\n",
                name, record.ttl, record.record_type, record.content, proxied
            ));
        }

        Ok(output)
    }
}

/// Sync result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncResult {
    pub total: usize,
    pub synced: usize,
    pub errors: usize,
}
