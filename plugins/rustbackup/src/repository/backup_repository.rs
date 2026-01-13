//! Backup Repository
//!
//! Database operations for backup records.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::backup::{
    Backup, BackupType, BackupStatus, BackupIncludes, BackupFilters, BackupStats, CompressionType,
};

/// Backup repository for database operations
pub struct BackupRepository {
    pool: PgPool,
}

impl BackupRepository {
    /// Create new backup repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Insert a new backup record
    pub async fn insert(&self, backup: &Backup) -> Result<(), sqlx::Error> {
        let backup_type = backup.backup_type.to_string();
        let status = backup.status.to_string();
        let compression = backup.compression.to_string();
        let includes = serde_json::to_value(&backup.includes).unwrap_or_default();

        sqlx::query(
            r#"
            INSERT INTO backups (
                id, name, backup_type, status, file_path, file_size, checksum,
                compression, encryption_info, includes, triggered_by, schedule_id,
                storage_id, remote_path, file_count, table_count, database_version,
                rustpress_version, created_at, started_at, completed_at, error_message
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22
            )
            "#
        )
        .bind(backup.id)
        .bind(&backup.name)
        .bind(&backup_type)
        .bind(&status)
        .bind(&backup.file_path)
        .bind(backup.file_size)
        .bind(&backup.checksum)
        .bind(&compression)
        .bind(serde_json::to_value(&backup.encryption_info).ok())
        .bind(&includes)
        .bind(&backup.triggered_by)
        .bind(backup.schedule_id)
        .bind(backup.storage_id)
        .bind(&backup.remote_path)
        .bind(backup.file_count)
        .bind(backup.table_count)
        .bind(&backup.database_version)
        .bind(&backup.rustpress_version)
        .bind(backup.created_at)
        .bind(backup.started_at)
        .bind(backup.completed_at)
        .bind(&backup.error_message)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update a backup record
    pub async fn update(&self, backup: &Backup) -> Result<(), sqlx::Error> {
        let status = backup.status.to_string();
        let includes = serde_json::to_value(&backup.includes).unwrap_or_default();

        sqlx::query(
            r#"
            UPDATE backups SET
                name = $2,
                status = $3,
                file_path = $4,
                file_size = $5,
                checksum = $6,
                encryption_info = $7,
                includes = $8,
                remote_path = $9,
                file_count = $10,
                table_count = $11,
                database_version = $12,
                started_at = $13,
                completed_at = $14,
                error_message = $15
            WHERE id = $1
            "#
        )
        .bind(backup.id)
        .bind(&backup.name)
        .bind(&status)
        .bind(&backup.file_path)
        .bind(backup.file_size)
        .bind(&backup.checksum)
        .bind(serde_json::to_value(&backup.encryption_info).ok())
        .bind(&includes)
        .bind(&backup.remote_path)
        .bind(backup.file_count)
        .bind(backup.table_count)
        .bind(&backup.database_version)
        .bind(backup.started_at)
        .bind(backup.completed_at)
        .bind(&backup.error_message)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get backup by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Backup>, sqlx::Error> {
        let row = sqlx::query_as::<_, BackupRow>(
            r#"
            SELECT
                id, name, backup_type, status, file_path, file_size, checksum,
                compression, encryption_info, includes, triggered_by, schedule_id,
                storage_id, remote_path, file_count, table_count, database_version,
                rustpress_version, created_at, started_at, completed_at, error_message
            FROM backups
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| self.row_to_backup(r)))
    }

    /// List backups with filters
    pub async fn list(&self, filters: BackupFilters) -> Result<Vec<Backup>, sqlx::Error> {
        let mut query = String::from(
            r#"
            SELECT
                id, name, backup_type, status, file_path, file_size, checksum,
                compression, encryption_info, includes, triggered_by, schedule_id,
                storage_id, remote_path, file_count, table_count, database_version,
                rustpress_version, created_at, started_at, completed_at, error_message
            FROM backups
            WHERE 1=1
            "#
        );

        // Add filter conditions
        if let Some(ref backup_type) = filters.backup_type {
            query.push_str(&format!(" AND backup_type = '{}'", backup_type));
        }

        if let Some(ref status) = filters.status {
            query.push_str(&format!(" AND status = '{}'", status));
        }

        if let Some(ref search) = filters.search {
            query.push_str(&format!(" AND name ILIKE '%{}%'", search));
        }

        if let Some(ref from) = filters.from_date {
            query.push_str(&format!(" AND created_at >= '{}'", from.to_rfc3339()));
        }

        if let Some(ref to) = filters.to_date {
            query.push_str(&format!(" AND created_at <= '{}'", to.to_rfc3339()));
        }

        if let Some(schedule_id) = filters.schedule_id {
            query.push_str(&format!(" AND schedule_id = '{}'", schedule_id));
        }

        if let Some(storage_id) = filters.storage_id {
            query.push_str(&format!(" AND storage_id = '{}'", storage_id));
        }

        // Sort by created_at descending
        query.push_str(" ORDER BY created_at DESC");

        let rows = sqlx::query_as::<_, BackupRow>(&query)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(|r| self.row_to_backup(r)).collect())
    }

    /// Delete backup by ID
    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM backups WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get backup statistics
    pub async fn get_stats(&self) -> Result<BackupStats, sqlx::Error> {
        let row = sqlx::query_as::<_, StatsRow>(
            r#"
            SELECT
                COUNT(*)::bigint as total_backups,
                COUNT(*) FILTER (WHERE status = 'completed')::bigint as successful_backups,
                COUNT(*) FILTER (WHERE status = 'failed')::bigint as failed_backups,
                COALESCE(SUM(file_size) FILTER (WHERE remote_path IS NULL), 0)::bigint as local_storage_used,
                COALESCE(SUM(file_size) FILTER (WHERE remote_path IS NOT NULL), 0)::bigint as remote_storage_used,
                MAX(created_at) as last_backup,
                MAX(created_at) FILTER (WHERE status = 'completed') as last_successful,
                COALESCE(AVG(file_size), 0)::bigint as average_size,
                COALESCE(AVG(EXTRACT(EPOCH FROM (completed_at - started_at))), 0)::bigint as average_duration,
                COUNT(*) FILTER (WHERE created_at >= NOW() - INTERVAL '7 days')::bigint as backups_this_week,
                COUNT(*) FILTER (WHERE created_at >= NOW() - INTERVAL '30 days')::bigint as backups_this_month
            FROM backups
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(BackupStats {
            total_backups: row.total_backups.unwrap_or(0),
            successful_backups: row.successful_backups.unwrap_or(0),
            failed_backups: row.failed_backups.unwrap_or(0),
            local_storage_used: row.local_storage_used.unwrap_or(0),
            remote_storage_used: row.remote_storage_used.unwrap_or(0),
            last_backup: row.last_backup,
            last_successful: row.last_successful,
            average_size: row.average_size.unwrap_or(0),
            average_duration: row.average_duration.unwrap_or(0) as i32,
            backups_this_week: row.backups_this_week.unwrap_or(0),
            backups_this_month: row.backups_this_month.unwrap_or(0),
        })
    }

    /// Count backups for schedule
    pub async fn count_by_schedule(&self, schedule_id: Uuid) -> Result<i64, sqlx::Error> {
        let row = sqlx::query_as::<_, CountRow>("SELECT COUNT(*)::bigint as count FROM backups WHERE schedule_id = $1")
            .bind(schedule_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(row.count.unwrap_or(0))
    }

    /// Get latest backup for schedule
    pub async fn get_latest_by_schedule(&self, schedule_id: Uuid) -> Result<Option<Backup>, sqlx::Error> {
        let row = sqlx::query_as::<_, BackupRow>(
            r#"
            SELECT
                id, name, backup_type, status, file_path, file_size, checksum,
                compression, encryption_info, includes, triggered_by, schedule_id,
                storage_id, remote_path, file_count, table_count, database_version,
                rustpress_version, created_at, started_at, completed_at, error_message
            FROM backups
            WHERE schedule_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#
        )
        .bind(schedule_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| self.row_to_backup(r)))
    }

    /// Internal helper to convert database row to Backup
    fn row_to_backup(&self, row: BackupRow) -> Backup {
        let duration_seconds = match (row.started_at, row.completed_at) {
            (started, Some(completed)) => Some((completed - started).num_seconds() as i32),
            _ => None,
        };

        Backup {
            id: row.id,
            name: row.name,
            backup_type: BackupType::from_str(&row.backup_type),
            status: BackupStatus::from_str(&row.status),
            file_path: row.file_path,
            file_size: row.file_size,
            checksum: row.checksum,
            compression: CompressionType::from_str(&row.compression),
            encryption_info: row.encryption_info
                .and_then(|v| serde_json::from_value(v).ok()),
            includes: row.includes
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default(),
            triggered_by: row.triggered_by,
            schedule_id: row.schedule_id,
            storage_id: row.storage_id,
            remote_path: row.remote_path,
            file_count: row.file_count,
            table_count: row.table_count,
            database_version: row.database_version,
            rustpress_version: row.rustpress_version,
            created_at: row.created_at,
            started_at: row.started_at,
            completed_at: row.completed_at,
            duration_seconds,
            error_message: row.error_message,
        }
    }
}

/// Backup row struct for dynamic queries
#[derive(sqlx::FromRow)]
struct BackupRow {
    id: Uuid,
    name: String,
    backup_type: String,
    status: String,
    file_path: Option<String>,
    file_size: Option<i64>,
    checksum: Option<String>,
    compression: String,
    encryption_info: Option<serde_json::Value>,
    includes: Option<serde_json::Value>,
    triggered_by: Option<String>,
    schedule_id: Option<Uuid>,
    storage_id: Option<Uuid>,
    remote_path: Option<String>,
    file_count: Option<i64>,
    table_count: Option<i32>,
    database_version: Option<String>,
    rustpress_version: String,
    created_at: DateTime<Utc>,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    error_message: Option<String>,
}

/// Stats row struct for aggregate queries
#[derive(sqlx::FromRow)]
struct StatsRow {
    total_backups: Option<i64>,
    successful_backups: Option<i64>,
    failed_backups: Option<i64>,
    local_storage_used: Option<i64>,
    remote_storage_used: Option<i64>,
    last_backup: Option<DateTime<Utc>>,
    last_successful: Option<DateTime<Utc>>,
    average_size: Option<i64>,
    average_duration: Option<i64>,
    backups_this_week: Option<i64>,
    backups_this_month: Option<i64>,
}

/// Count row struct for count queries
#[derive(sqlx::FromRow)]
struct CountRow {
    count: Option<i64>,
}
