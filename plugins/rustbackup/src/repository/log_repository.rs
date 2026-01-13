//! Log Repository
//!
//! Database operations for backup logs.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::log::{BackupLog, LogLevel, LogFilters, LogStats, OperationStats};

/// Log repository for database operations
pub struct LogRepository {
    pool: PgPool,
}

impl LogRepository {
    /// Create new log repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Insert a new log entry
    pub async fn insert(&self, log: &BackupLog) -> Result<(), sqlx::Error> {
        let level = log.level.to_string();

        sqlx::query(
            r#"
            INSERT INTO backup_logs (
                id, backup_id, schedule_id, level, operation, message,
                context, progress, bytes_processed, files_processed, duration_ms, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#
        )
        .bind(log.id)
        .bind(log.backup_id)
        .bind(log.schedule_id)
        .bind(&level)
        .bind(&log.operation)
        .bind(&log.message)
        .bind(&log.context)
        .bind(log.progress)
        .bind(log.bytes_processed)
        .bind(log.files_processed)
        .bind(log.duration_ms)
        .bind(log.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get log by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<BackupLog>, sqlx::Error> {
        let row = sqlx::query_as::<_, LogRow>(
            r#"
            SELECT id, backup_id, schedule_id, level, operation, message,
                   context, progress, bytes_processed, files_processed, duration_ms, created_at
            FROM backup_logs
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(row_to_log(r))),
            None => Ok(None),
        }
    }

    /// List logs with filters
    pub async fn list(&self, filters: LogFilters) -> Result<Vec<BackupLog>, sqlx::Error> {
        let mut query = String::from(
            r#"
            SELECT id, backup_id, schedule_id, level, operation, message,
                   context, progress, bytes_processed, files_processed, duration_ms, created_at
            FROM backup_logs
            WHERE 1=1
            "#
        );

        if let Some(backup_id) = filters.backup_id {
            query.push_str(&format!(" AND backup_id = '{}'", backup_id));
        }

        if let Some(schedule_id) = filters.schedule_id {
            query.push_str(&format!(" AND schedule_id = '{}'", schedule_id));
        }

        if let Some(ref level) = filters.level {
            query.push_str(&format!(" AND level = '{}'", level));
        }

        if let Some(ref operation) = filters.operation {
            query.push_str(&format!(" AND operation = '{}'", operation));
        }

        if let Some(ref from) = filters.date_from {
            query.push_str(&format!(" AND created_at >= '{}'", from.to_rfc3339()));
        }

        if let Some(ref to) = filters.date_to {
            query.push_str(&format!(" AND created_at <= '{}'", to.to_rfc3339()));
        }

        if let Some(ref search) = filters.search {
            query.push_str(&format!(" AND message ILIKE '%{}%'", search));
        }

        query.push_str(" ORDER BY created_at DESC LIMIT 1000");

        let rows = sqlx::query_as::<_, LogRow>(&query)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(row_to_log).collect())
    }

    /// Get logs for a backup
    pub async fn find_by_backup(&self, backup_id: Uuid) -> Result<Vec<BackupLog>, sqlx::Error> {
        let rows = sqlx::query_as::<_, LogRow>(
            r#"
            SELECT id, backup_id, schedule_id, level, operation, message,
                   context, progress, bytes_processed, files_processed, duration_ms, created_at
            FROM backup_logs
            WHERE backup_id = $1
            ORDER BY created_at ASC
            "#
        )
        .bind(backup_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(row_to_log).collect())
    }

    /// Get log statistics
    pub async fn get_stats(&self) -> Result<LogStats, sqlx::Error> {
        let counts = sqlx::query_as::<_, LogCountRow>(
            r#"
            SELECT
                COUNT(*)::bigint as total_entries,
                COUNT(*) FILTER (WHERE level = 'error')::bigint as error_count,
                COUNT(*) FILTER (WHERE level = 'warning')::bigint as warning_count,
                COUNT(*) FILTER (WHERE level = 'info')::bigint as info_count,
                COUNT(*) FILTER (WHERE level = 'debug')::bigint as debug_count
            FROM backup_logs
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        let operation_rows = sqlx::query_as::<_, OperationRow>(
            r#"
            SELECT
                operation,
                COUNT(*)::bigint as count,
                AVG(duration_ms)::bigint as avg_duration_ms,
                COALESCE(SUM(bytes_processed), 0)::bigint as total_bytes,
                COALESCE(SUM(files_processed), 0)::bigint as total_files
            FROM backup_logs
            GROUP BY operation
            ORDER BY count DESC
            LIMIT 10
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let operations: Vec<OperationStats> = operation_rows
            .into_iter()
            .map(|row| OperationStats {
                operation: row.operation,
                count: row.count.unwrap_or(0),
                avg_duration_ms: row.avg_duration_ms,
                total_bytes: row.total_bytes.unwrap_or(0),
                total_files: row.total_files.unwrap_or(0),
            })
            .collect();

        Ok(LogStats {
            total_entries: counts.total_entries.unwrap_or(0),
            debug_count: counts.debug_count.unwrap_or(0),
            info_count: counts.info_count.unwrap_or(0),
            warning_count: counts.warning_count.unwrap_or(0),
            error_count: counts.error_count.unwrap_or(0),
            operations,
        })
    }

    /// Delete logs older than given date
    pub async fn delete_older_than(&self, cutoff: DateTime<Utc>) -> Result<i64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM backup_logs WHERE created_at < $1")
            .bind(cutoff)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() as i64)
    }

    /// Delete logs for a backup
    pub async fn delete_by_backup(&self, backup_id: Uuid) -> Result<i64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM backup_logs WHERE backup_id = $1")
            .bind(backup_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() as i64)
    }
}

/// Log row struct
#[derive(sqlx::FromRow)]
struct LogRow {
    id: Uuid,
    backup_id: Option<Uuid>,
    schedule_id: Option<Uuid>,
    level: String,
    operation: String,
    message: String,
    context: Option<serde_json::Value>,
    progress: Option<i32>,
    bytes_processed: Option<i64>,
    files_processed: Option<i64>,
    duration_ms: Option<i64>,
    created_at: DateTime<Utc>,
}

/// Count row for stats
#[derive(sqlx::FromRow)]
struct LogCountRow {
    total_entries: Option<i64>,
    error_count: Option<i64>,
    warning_count: Option<i64>,
    info_count: Option<i64>,
    debug_count: Option<i64>,
}

/// Operation row for stats
#[derive(sqlx::FromRow)]
struct OperationRow {
    operation: String,
    count: Option<i64>,
    avg_duration_ms: Option<i64>,
    total_bytes: Option<i64>,
    total_files: Option<i64>,
}

/// Convert row to log entry
fn row_to_log(row: LogRow) -> BackupLog {
    BackupLog {
        id: row.id,
        backup_id: row.backup_id,
        schedule_id: row.schedule_id,
        level: LogLevel::from_str(&row.level),
        operation: row.operation,
        message: row.message,
        context: row.context,
        progress: row.progress,
        bytes_processed: row.bytes_processed,
        files_processed: row.files_processed,
        duration_ms: row.duration_ms,
        created_at: row.created_at,
    }
}
