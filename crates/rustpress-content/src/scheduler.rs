//! Scheduled publishing system
//!
//! Provides functionality for scheduling content publication,
//! managing publish queues, and handling timezone considerations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{ContentError, ContentResult, ContentStatus};

/// Scheduled publish job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledPublish {
    /// Job ID
    pub id: Uuid,

    /// Content ID to publish
    pub content_id: Uuid,

    /// Scheduled publication time
    pub scheduled_at: DateTime<Utc>,

    /// Job status
    pub status: ScheduleStatus,

    /// Number of publish attempts
    pub attempts: i32,

    /// Last attempt time
    pub last_attempt: Option<DateTime<Utc>>,

    /// Error message if failed
    pub error: Option<String>,

    /// When job was created
    pub created_at: DateTime<Utc>,
}

/// Schedule job status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

/// Publish scheduler service
pub struct PublishScheduler {
    pool: sqlx::PgPool,
    running: Arc<RwLock<bool>>,
}

impl PublishScheduler {
    /// Create new scheduler
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            pool,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Schedule content for publication
    pub async fn schedule(
        &self,
        content_id: Uuid,
        publish_at: DateTime<Utc>,
    ) -> ContentResult<ScheduledPublish> {
        // Validate publish time is in the future
        if publish_at <= Utc::now() {
            return Err(ContentError::Scheduler(
                "Scheduled time must be in the future".to_string(),
            ));
        }

        // Check for existing schedule
        let existing = sqlx::query_as::<_, ScheduledPublishRow>(
            "SELECT * FROM scheduled_publishes WHERE content_id = $1 AND status = 'pending'",
        )
        .bind(content_id)
        .fetch_optional(&self.pool)
        .await?;

        if existing.is_some() {
            return Err(ContentError::Scheduler(
                "Content already has a pending schedule".to_string(),
            ));
        }

        let job = ScheduledPublish {
            id: Uuid::new_v4(),
            content_id,
            scheduled_at: publish_at,
            status: ScheduleStatus::Pending,
            attempts: 0,
            last_attempt: None,
            error: None,
            created_at: Utc::now(),
        };

        // Insert job
        sqlx::query(
            r#"
            INSERT INTO scheduled_publishes (
                id, content_id, scheduled_at, status, attempts,
                last_attempt, error, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(job.id)
        .bind(job.content_id)
        .bind(job.scheduled_at)
        .bind(serde_json::to_string(&job.status).unwrap())
        .bind(job.attempts)
        .bind(job.last_attempt)
        .bind(&job.error)
        .bind(job.created_at)
        .execute(&self.pool)
        .await?;

        // Update content status to scheduled
        sqlx::query(
            "UPDATE contents SET status = $2, scheduled_at = $3 WHERE id = $1",
        )
        .bind(content_id)
        .bind(serde_json::to_string(&ContentStatus::Scheduled).unwrap())
        .bind(publish_at)
        .execute(&self.pool)
        .await?;

        Ok(job)
    }

    /// Reschedule a pending job
    pub async fn reschedule(
        &self,
        job_id: Uuid,
        new_time: DateTime<Utc>,
    ) -> ContentResult<ScheduledPublish> {
        if new_time <= Utc::now() {
            return Err(ContentError::Scheduler(
                "Scheduled time must be in the future".to_string(),
            ));
        }

        // Update job
        sqlx::query(
            r#"
            UPDATE scheduled_publishes
            SET scheduled_at = $2
            WHERE id = $1 AND status = 'pending'
            "#,
        )
        .bind(job_id)
        .bind(new_time)
        .execute(&self.pool)
        .await?;

        self.get_job(job_id).await
    }

    /// Cancel a scheduled publish
    pub async fn cancel(&self, job_id: Uuid) -> ContentResult<()> {
        let job = self.get_job(job_id).await?;

        if job.status != ScheduleStatus::Pending {
            return Err(ContentError::Scheduler(
                "Can only cancel pending schedules".to_string(),
            ));
        }

        // Update job status
        sqlx::query(
            "UPDATE scheduled_publishes SET status = $2 WHERE id = $1",
        )
        .bind(job_id)
        .bind(serde_json::to_string(&ScheduleStatus::Cancelled).unwrap())
        .execute(&self.pool)
        .await?;

        // Reset content status
        sqlx::query(
            "UPDATE contents SET status = $2, scheduled_at = NULL WHERE id = $1",
        )
        .bind(job.content_id)
        .bind(serde_json::to_string(&ContentStatus::Draft).unwrap())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get a scheduled job
    pub async fn get_job(&self, job_id: Uuid) -> ContentResult<ScheduledPublish> {
        let row = sqlx::query_as::<_, ScheduledPublishRow>(
            "SELECT * FROM scheduled_publishes WHERE id = $1",
        )
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ContentError::NotFound(job_id.to_string()))?;

        Ok(row.into())
    }

    /// Get schedule for content
    pub async fn get_content_schedule(&self, content_id: Uuid) -> ContentResult<Option<ScheduledPublish>> {
        let row = sqlx::query_as::<_, ScheduledPublishRow>(
            "SELECT * FROM scheduled_publishes WHERE content_id = $1 AND status = 'pending'",
        )
        .bind(content_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Into::into))
    }

    /// Get all pending jobs
    pub async fn get_pending_jobs(&self) -> ContentResult<Vec<ScheduledPublish>> {
        let rows = sqlx::query_as::<_, ScheduledPublishRow>(
            "SELECT * FROM scheduled_publishes WHERE status = 'pending' ORDER BY scheduled_at ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Get jobs due for publication
    pub async fn get_due_jobs(&self) -> ContentResult<Vec<ScheduledPublish>> {
        let rows = sqlx::query_as::<_, ScheduledPublishRow>(
            r#"
            SELECT * FROM scheduled_publishes
            WHERE status = 'pending' AND scheduled_at <= $1
            ORDER BY scheduled_at ASC
            "#,
        )
        .bind(Utc::now())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Process due jobs (publish content)
    pub async fn process_due_jobs(&self) -> ContentResult<Vec<PublishResult>> {
        let jobs = self.get_due_jobs().await?;
        let mut results = Vec::new();

        for job in jobs {
            let result = self.process_job(&job).await;
            results.push(result);
        }

        Ok(results)
    }

    /// Process a single job
    async fn process_job(&self, job: &ScheduledPublish) -> PublishResult {
        // Mark as processing
        let _ = sqlx::query(
            "UPDATE scheduled_publishes SET status = $2, attempts = attempts + 1, last_attempt = $3 WHERE id = $1",
        )
        .bind(job.id)
        .bind(serde_json::to_string(&ScheduleStatus::Processing).unwrap())
        .bind(Utc::now())
        .execute(&self.pool)
        .await;

        // Attempt to publish
        let result = self.publish_content(job.content_id).await;

        match result {
            Ok(_) => {
                // Mark as completed
                let _ = sqlx::query(
                    "UPDATE scheduled_publishes SET status = $2 WHERE id = $1",
                )
                .bind(job.id)
                .bind(serde_json::to_string(&ScheduleStatus::Completed).unwrap())
                .execute(&self.pool)
                .await;

                PublishResult {
                    job_id: job.id,
                    content_id: job.content_id,
                    success: true,
                    error: None,
                    published_at: Some(Utc::now()),
                }
            }
            Err(e) => {
                // Check retry limit
                let max_attempts = 3;
                let new_status = if job.attempts + 1 >= max_attempts {
                    ScheduleStatus::Failed
                } else {
                    ScheduleStatus::Pending
                };

                let error_msg = e.to_string();

                let _ = sqlx::query(
                    "UPDATE scheduled_publishes SET status = $2, error = $3 WHERE id = $1",
                )
                .bind(job.id)
                .bind(serde_json::to_string(&new_status).unwrap())
                .bind(&error_msg)
                .execute(&self.pool)
                .await;

                PublishResult {
                    job_id: job.id,
                    content_id: job.content_id,
                    success: false,
                    error: Some(error_msg),
                    published_at: None,
                }
            }
        }
    }

    /// Publish content
    async fn publish_content(&self, content_id: Uuid) -> ContentResult<()> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE contents
            SET status = $2, published_at = $3, scheduled_at = NULL, updated_at = $3
            WHERE id = $1
            "#,
        )
        .bind(content_id)
        .bind(serde_json::to_string(&ContentStatus::Published).unwrap())
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Start background scheduler
    pub async fn start_background_processor(self: Arc<Self>, interval_seconds: u64) {
        let scheduler = self.clone();

        tokio::spawn(async move {
            loop {
                // Check if still running
                {
                    let running = scheduler.running.read().await;
                    if !*running {
                        break;
                    }
                }

                // Process due jobs
                if let Err(e) = scheduler.process_due_jobs().await {
                    tracing::error!("Error processing scheduled jobs: {}", e);
                }

                // Wait before next check
                tokio::time::sleep(tokio::time::Duration::from_secs(interval_seconds)).await;
            }
        });

        // Mark as running
        let mut running = self.running.write().await;
        *running = true;
    }

    /// Stop background processor
    pub async fn stop_background_processor(&self) {
        let mut running = self.running.write().await;
        *running = false;
    }

    /// Get scheduler statistics
    pub async fn get_stats(&self) -> ContentResult<SchedulerStats> {
        let pending: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM scheduled_publishes WHERE status = 'pending'",
        )
        .fetch_one(&self.pool)
        .await?;

        let completed: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM scheduled_publishes WHERE status = 'completed'",
        )
        .fetch_one(&self.pool)
        .await?;

        let failed: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM scheduled_publishes WHERE status = 'failed'",
        )
        .fetch_one(&self.pool)
        .await?;

        let next_job = sqlx::query_as::<_, ScheduledPublishRow>(
            "SELECT * FROM scheduled_publishes WHERE status = 'pending' ORDER BY scheduled_at ASC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(SchedulerStats {
            pending_jobs: pending.0 as usize,
            completed_jobs: completed.0 as usize,
            failed_jobs: failed.0 as usize,
            next_scheduled: next_job.map(|j| j.scheduled_at),
        })
    }
}

/// Result of a publish attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishResult {
    pub job_id: Uuid,
    pub content_id: Uuid,
    pub success: bool,
    pub error: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
}

/// Scheduler statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStats {
    pub pending_jobs: usize,
    pub completed_jobs: usize,
    pub failed_jobs: usize,
    pub next_scheduled: Option<DateTime<Utc>>,
}

/// Database row representation
#[derive(Debug, sqlx::FromRow)]
struct ScheduledPublishRow {
    id: Uuid,
    content_id: Uuid,
    scheduled_at: DateTime<Utc>,
    status: String,
    attempts: i32,
    last_attempt: Option<DateTime<Utc>>,
    error: Option<String>,
    created_at: DateTime<Utc>,
}

impl From<ScheduledPublishRow> for ScheduledPublish {
    fn from(row: ScheduledPublishRow) -> Self {
        Self {
            id: row.id,
            content_id: row.content_id,
            scheduled_at: row.scheduled_at,
            status: serde_json::from_str(&row.status).unwrap_or(ScheduleStatus::Pending),
            attempts: row.attempts,
            last_attempt: row.last_attempt,
            error: row.error,
            created_at: row.created_at,
        }
    }
}

/// Cron expression parser for recurring schedules
pub struct CronScheduler;

impl CronScheduler {
    /// Parse cron expression and get next occurrence
    pub fn next_occurrence(cron_expr: &str) -> ContentResult<DateTime<Utc>> {
        let schedule = cron::Schedule::from_str(cron_expr)
            .map_err(|e| ContentError::Scheduler(format!("Invalid cron expression: {}", e)))?;

        schedule
            .upcoming(Utc)
            .next()
            .ok_or_else(|| ContentError::Scheduler("No upcoming occurrence".to_string()))
    }

    /// Get next N occurrences
    pub fn next_occurrences(cron_expr: &str, count: usize) -> ContentResult<Vec<DateTime<Utc>>> {
        let schedule = cron::Schedule::from_str(cron_expr)
            .map_err(|e| ContentError::Scheduler(format!("Invalid cron expression: {}", e)))?;

        Ok(schedule.upcoming(Utc).take(count).collect())
    }

    /// Validate cron expression
    pub fn validate(cron_expr: &str) -> bool {
        cron::Schedule::from_str(cron_expr).is_ok()
    }

    /// Human-readable cron expression
    pub fn describe(cron_expr: &str) -> String {
        // Simple descriptions for common patterns
        match cron_expr {
            "0 0 * * *" => "Daily at midnight".to_string(),
            "0 0 * * 0" => "Weekly on Sunday at midnight".to_string(),
            "0 0 1 * *" => "Monthly on the 1st at midnight".to_string(),
            "0 9 * * 1-5" => "Weekdays at 9:00 AM".to_string(),
            _ => format!("Custom schedule: {}", cron_expr),
        }
    }
}

use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cron_validation() {
        // The cron crate uses 6 or 7 fields: sec min hour day_of_month month day_of_week [year]
        assert!(CronScheduler::validate("0 0 0 * * *")); // Daily at midnight
        assert!(CronScheduler::validate("0 */5 * * * *")); // Every 5 minutes
        assert!(!CronScheduler::validate("invalid"));
    }

    #[test]
    fn test_cron_next_occurrence() {
        // The cron crate uses 6 or 7 fields: sec min hour day_of_month month day_of_week [year]
        let result = CronScheduler::next_occurrence("0 0 0 * * *");
        assert!(result.is_ok());
    }
}
