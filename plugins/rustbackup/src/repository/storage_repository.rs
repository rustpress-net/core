//! Storage Repository
//!
//! Database operations for storage providers.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::storage::{StorageProvider, StorageType, StorageConfig, StorageFilters, LocalStorageConfig};

/// Storage repository for database operations
pub struct StorageRepository {
    pool: PgPool,
}

impl StorageRepository {
    /// Create new storage repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Insert a new storage provider
    pub async fn insert(&self, provider: &StorageProvider) -> Result<(), sqlx::Error> {
        let provider_type = provider.provider_type.to_string();
        let config = serde_json::to_value(&provider.config).unwrap_or_default();

        sqlx::query(
            r#"
            INSERT INTO storage_providers (
                id, name, provider_type, config, is_default, enabled,
                last_connected, backup_count, storage_used, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#
        )
        .bind(provider.id)
        .bind(&provider.name)
        .bind(&provider_type)
        .bind(&config)
        .bind(provider.is_default)
        .bind(provider.enabled)
        .bind(provider.last_connected)
        .bind(provider.backup_count)
        .bind(provider.storage_used)
        .bind(provider.created_at)
        .bind(provider.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update a storage provider
    pub async fn update(&self, provider: &StorageProvider) -> Result<(), sqlx::Error> {
        let config = serde_json::to_value(&provider.config).unwrap_or_default();

        sqlx::query(
            r#"
            UPDATE storage_providers SET
                name = $2,
                config = $3,
                is_default = $4,
                enabled = $5,
                last_connected = $6,
                backup_count = $7,
                storage_used = $8,
                updated_at = $9
            WHERE id = $1
            "#
        )
        .bind(provider.id)
        .bind(&provider.name)
        .bind(&config)
        .bind(provider.is_default)
        .bind(provider.enabled)
        .bind(provider.last_connected)
        .bind(provider.backup_count)
        .bind(provider.storage_used)
        .bind(provider.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get storage provider by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<StorageProvider>, sqlx::Error> {
        let row = sqlx::query_as::<_, StorageRow>(
            r#"
            SELECT id, name, provider_type, config, is_default, enabled,
                   last_connected, backup_count, storage_used, created_at, updated_at
            FROM storage_providers
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(row_to_provider(r))),
            None => Ok(None),
        }
    }

    /// Get default storage provider
    pub async fn find_default(&self) -> Result<Option<StorageProvider>, sqlx::Error> {
        let row = sqlx::query_as::<_, StorageRow>(
            r#"
            SELECT id, name, provider_type, config, is_default, enabled,
                   last_connected, backup_count, storage_used, created_at, updated_at
            FROM storage_providers
            WHERE is_default = true
            LIMIT 1
            "#
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(row_to_provider(r))),
            None => Ok(None),
        }
    }

    /// List all storage providers
    pub async fn list(&self, filters: StorageFilters) -> Result<Vec<StorageProvider>, sqlx::Error> {
        let mut query = String::from(
            r#"
            SELECT id, name, provider_type, config, is_default, enabled,
                   last_connected, backup_count, storage_used, created_at, updated_at
            FROM storage_providers
            WHERE 1=1
            "#
        );

        if let Some(ref provider_type) = filters.provider_type {
            query.push_str(&format!(" AND provider_type = '{}'", provider_type));
        }

        if let Some(enabled) = filters.enabled {
            query.push_str(&format!(" AND enabled = {}", enabled));
        }

        query.push_str(" ORDER BY is_default DESC, name ASC");

        let rows = sqlx::query_as::<_, StorageRow>(&query)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(row_to_provider).collect())
    }

    /// Set default provider (clears existing default)
    pub async fn set_default(&self, id: Uuid) -> Result<(), sqlx::Error> {
        // Clear existing default
        sqlx::query("UPDATE storage_providers SET is_default = false WHERE is_default = true")
            .execute(&self.pool)
            .await?;

        // Set new default
        sqlx::query("UPDATE storage_providers SET is_default = true WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Delete storage provider
    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM storage_providers WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Update storage usage stats
    pub async fn update_usage(&self, id: Uuid, backup_count: i64, storage_used: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE storage_providers SET
                backup_count = $2,
                storage_used = $3,
                updated_at = NOW()
            WHERE id = $1
            "#
        )
        .bind(id)
        .bind(backup_count)
        .bind(storage_used)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

/// Storage row struct
#[derive(sqlx::FromRow)]
struct StorageRow {
    id: Uuid,
    name: String,
    provider_type: String,
    config: serde_json::Value,
    is_default: bool,
    enabled: bool,
    last_connected: Option<DateTime<Utc>>,
    backup_count: i64,
    storage_used: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// Convert row to storage provider
fn row_to_provider(row: StorageRow) -> StorageProvider {
    let config = serde_json::from_value(row.config).unwrap_or_else(|_| {
        StorageConfig::Local(LocalStorageConfig {
            path: "data/backups".to_string(),
        })
    });

    StorageProvider {
        id: row.id,
        name: row.name,
        provider_type: StorageType::from_str(&row.provider_type),
        config,
        is_default: row.is_default,
        enabled: row.enabled,
        last_connected: row.last_connected,
        backup_count: row.backup_count,
        storage_used: row.storage_used,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}
