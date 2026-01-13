//! Schedule Repository
//!
//! Database operations for backup schedules.

use chrono::{DateTime, Utc, Weekday};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::schedule::{Schedule, ScheduleFrequency, ScheduleFilters, RetentionPolicy};
use crate::models::backup::{BackupType, BackupIncludes, CompressionType};

/// Schedule repository for database operations
pub struct ScheduleRepository {
    pool: PgPool,
}

impl ScheduleRepository {
    /// Create new schedule repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Insert a new schedule
    pub async fn insert(&self, schedule: &Schedule) -> Result<(), sqlx::Error> {
        let frequency = schedule.frequency.to_string();
        let backup_type = schedule.backup_type.to_string();
        let compression = schedule.compression.to_string();
        let includes = serde_json::to_value(&schedule.includes).unwrap_or_default();
        let retention = serde_json::to_value(&schedule.retention).unwrap_or_default();
        let days_of_week: Vec<String> = schedule.days_of_week.iter().map(|d| d.to_string()).collect();

        sqlx::query(
            r#"
            INSERT INTO backup_schedules (
                id, name, frequency, cron_expression, backup_type, includes,
                compression, encrypt, storage_id, retention, enabled,
                run_hour, run_minute, days_of_week, day_of_month, next_run,
                last_run, last_success, last_failure, run_count, success_count,
                failure_count, notify_on_success, notify_on_failure, notify_email,
                created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15,
                $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27
            )
            "#
        )
        .bind(schedule.id)
        .bind(&schedule.name)
        .bind(&frequency)
        .bind(&schedule.cron_expression)
        .bind(&backup_type)
        .bind(&includes)
        .bind(&compression)
        .bind(schedule.encrypt)
        .bind(schedule.storage_id)
        .bind(&retention)
        .bind(schedule.enabled)
        .bind(schedule.run_hour as i32)
        .bind(schedule.run_minute as i32)
        .bind(&days_of_week)
        .bind(schedule.day_of_month.map(|d| d as i32))
        .bind(schedule.next_run)
        .bind(schedule.last_run)
        .bind(schedule.last_success)
        .bind(schedule.last_failure)
        .bind(schedule.run_count as i64)
        .bind(schedule.success_count as i64)
        .bind(schedule.failure_count as i64)
        .bind(schedule.notify_on_success)
        .bind(schedule.notify_on_failure)
        .bind(&schedule.notify_email)
        .bind(schedule.created_at)
        .bind(schedule.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update a schedule
    pub async fn update(&self, schedule: &Schedule) -> Result<(), sqlx::Error> {
        let frequency = schedule.frequency.to_string();
        let backup_type = schedule.backup_type.to_string();
        let compression = schedule.compression.to_string();
        let includes = serde_json::to_value(&schedule.includes).unwrap_or_default();
        let retention = serde_json::to_value(&schedule.retention).unwrap_or_default();
        let days_of_week: Vec<String> = schedule.days_of_week.iter().map(|d| d.to_string()).collect();

        sqlx::query(
            r#"
            UPDATE backup_schedules SET
                name = $2,
                frequency = $3,
                cron_expression = $4,
                backup_type = $5,
                includes = $6,
                compression = $7,
                encrypt = $8,
                storage_id = $9,
                retention = $10,
                enabled = $11,
                run_hour = $12,
                run_minute = $13,
                days_of_week = $14,
                day_of_month = $15,
                next_run = $16,
                last_run = $17,
                last_success = $18,
                last_failure = $19,
                run_count = $20,
                success_count = $21,
                failure_count = $22,
                notify_on_success = $23,
                notify_on_failure = $24,
                notify_email = $25,
                updated_at = $26
            WHERE id = $1
            "#
        )
        .bind(schedule.id)
        .bind(&schedule.name)
        .bind(&frequency)
        .bind(&schedule.cron_expression)
        .bind(&backup_type)
        .bind(&includes)
        .bind(&compression)
        .bind(schedule.encrypt)
        .bind(schedule.storage_id)
        .bind(&retention)
        .bind(schedule.enabled)
        .bind(schedule.run_hour as i32)
        .bind(schedule.run_minute as i32)
        .bind(&days_of_week)
        .bind(schedule.day_of_month.map(|d| d as i32))
        .bind(schedule.next_run)
        .bind(schedule.last_run)
        .bind(schedule.last_success)
        .bind(schedule.last_failure)
        .bind(schedule.run_count as i64)
        .bind(schedule.success_count as i64)
        .bind(schedule.failure_count as i64)
        .bind(schedule.notify_on_success)
        .bind(schedule.notify_on_failure)
        .bind(&schedule.notify_email)
        .bind(schedule.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get schedule by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Schedule>, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduleRow>(
            r#"
            SELECT
                id, name, frequency, cron_expression, backup_type, includes,
                compression, encrypt, storage_id, retention, enabled,
                run_hour, run_minute, days_of_week, day_of_month, next_run,
                last_run, last_success, last_failure, run_count, success_count,
                failure_count, notify_on_success, notify_on_failure, notify_email,
                created_at, updated_at
            FROM backup_schedules
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(row_to_schedule(r))),
            None => Ok(None),
        }
    }

    /// List all schedules
    pub async fn list(&self, filters: ScheduleFilters) -> Result<Vec<Schedule>, sqlx::Error> {
        let mut query = String::from(
            r#"
            SELECT
                id, name, frequency, cron_expression, backup_type, includes,
                compression, encrypt, storage_id, retention, enabled,
                run_hour, run_minute, days_of_week, day_of_month, next_run,
                last_run, last_success, last_failure, run_count, success_count,
                failure_count, notify_on_success, notify_on_failure, notify_email,
                created_at, updated_at
            FROM backup_schedules
            WHERE 1=1
            "#
        );

        if let Some(enabled) = filters.enabled {
            query.push_str(&format!(" AND enabled = {}", enabled));
        }

        if let Some(ref search) = filters.search {
            query.push_str(&format!(" AND name ILIKE '%{}%'", search));
        }

        query.push_str(" ORDER BY created_at DESC");

        let rows = sqlx::query_as::<_, ScheduleRow>(&query)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(row_to_schedule).collect())
    }

    /// Get enabled schedules that are due to run
    pub async fn get_due_schedules(&self) -> Result<Vec<Schedule>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduleRow>(
            r#"
            SELECT
                id, name, frequency, cron_expression, backup_type, includes,
                compression, encrypt, storage_id, retention, enabled,
                run_hour, run_minute, days_of_week, day_of_month, next_run,
                last_run, last_success, last_failure, run_count, success_count,
                failure_count, notify_on_success, notify_on_failure, notify_email,
                created_at, updated_at
            FROM backup_schedules
            WHERE enabled = true AND next_run <= NOW()
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(row_to_schedule).collect())
    }

    /// Delete schedule by ID
    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM backup_schedules WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

/// Schedule row struct
#[derive(sqlx::FromRow)]
struct ScheduleRow {
    id: Uuid,
    name: String,
    frequency: String,
    cron_expression: Option<String>,
    backup_type: String,
    includes: serde_json::Value,
    compression: String,
    encrypt: bool,
    storage_id: Option<Uuid>,
    retention: serde_json::Value,
    enabled: bool,
    run_hour: i32,
    run_minute: i32,
    days_of_week: Vec<String>,
    day_of_month: Option<i32>,
    next_run: Option<DateTime<Utc>>,
    last_run: Option<DateTime<Utc>>,
    last_success: Option<DateTime<Utc>>,
    last_failure: Option<DateTime<Utc>>,
    run_count: i64,
    success_count: i64,
    failure_count: i64,
    notify_on_success: bool,
    notify_on_failure: bool,
    notify_email: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// Convert row to schedule
fn row_to_schedule(row: ScheduleRow) -> Schedule {
    Schedule {
        id: row.id,
        name: row.name,
        frequency: ScheduleFrequency::from_str(&row.frequency),
        cron_expression: row.cron_expression,
        backup_type: BackupType::from_str(&row.backup_type),
        includes: serde_json::from_value(row.includes).unwrap_or_default(),
        compression: CompressionType::from_str(&row.compression),
        encrypt: row.encrypt,
        storage_id: row.storage_id,
        retention: serde_json::from_value(row.retention).unwrap_or_default(),
        enabled: row.enabled,
        run_hour: row.run_hour as i8,
        run_minute: row.run_minute as i8,
        days_of_week: row.days_of_week.iter()
            .filter_map(|d| parse_weekday(d))
            .collect(),
        day_of_month: row.day_of_month.map(|d| d as i8),
        next_run: row.next_run,
        last_run: row.last_run,
        last_success: row.last_success,
        last_failure: row.last_failure,
        run_count: row.run_count as i32,
        success_count: row.success_count as i32,
        failure_count: row.failure_count as i32,
        notify_on_success: row.notify_on_success,
        notify_on_failure: row.notify_on_failure,
        notify_email: row.notify_email,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

/// Parse weekday from string
fn parse_weekday(s: &str) -> Option<Weekday> {
    match s.to_lowercase().as_str() {
        "mon" | "monday" => Some(Weekday::Mon),
        "tue" | "tuesday" => Some(Weekday::Tue),
        "wed" | "wednesday" => Some(Weekday::Wed),
        "thu" | "thursday" => Some(Weekday::Thu),
        "fri" | "friday" => Some(Weekday::Fri),
        "sat" | "saturday" => Some(Weekday::Sat),
        "sun" | "sunday" => Some(Weekday::Sun),
        _ => None,
    }
}
