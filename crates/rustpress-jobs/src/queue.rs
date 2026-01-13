//! Job queue implementation.

use crate::job::{Job, JobPayload, JobStatus};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rustpress_core::error::{Error, Result};
use sqlx::PgPool;
#[allow(unused_imports)]
use std::sync::Arc;
use uuid::Uuid;

/// Queue configuration
#[derive(Debug, Clone)]
pub struct QueueConfig {
    /// Default timeout in seconds
    pub default_timeout: u64,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Retry delay in seconds
    pub retry_delay: u64,
    /// Number of jobs to fetch at once
    pub batch_size: u32,
    /// Sleep duration when queue is empty (milliseconds)
    pub sleep_on_empty_ms: u64,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            default_timeout: 300,
            max_retries: 3,
            retry_delay: 60,
            batch_size: 10,
            sleep_on_empty_ms: 1000,
        }
    }
}

/// Job queue trait
#[async_trait]
pub trait Queue: Send + Sync {
    /// Push a job to the queue
    async fn push(&self, job: Job) -> Result<Uuid>;

    /// Push a job with delay
    async fn push_delayed(&self, job: Job, delay_secs: u64) -> Result<Uuid>;

    /// Get next available job from queue
    async fn pop(&self, queue: &str) -> Result<Option<Job>>;

    /// Get multiple jobs from queue
    async fn pop_batch(&self, queue: &str, count: u32) -> Result<Vec<Job>>;

    /// Mark job as completed
    async fn complete(&self, job_id: Uuid) -> Result<()>;

    /// Mark job as failed
    async fn fail(&self, job_id: Uuid, error: &str) -> Result<()>;

    /// Release job back to queue
    async fn release(&self, job_id: Uuid, delay_secs: u64) -> Result<()>;

    /// Delete a job
    async fn delete(&self, job_id: Uuid) -> Result<()>;

    /// Get job by ID
    async fn get(&self, job_id: Uuid) -> Result<Option<Job>>;

    /// Get queue size
    async fn size(&self, queue: &str) -> Result<u64>;

    /// Clear all jobs from a queue
    async fn clear(&self, queue: &str) -> Result<u64>;

    /// Retry failed jobs
    async fn retry_failed(&self, queue: &str) -> Result<u64>;

    /// Release stale reserved jobs
    async fn release_stale(&self, older_than_secs: u64) -> Result<u64>;
}

/// Database-backed job queue
pub struct JobQueue {
    pool: PgPool,
    #[allow(dead_code)]
    config: QueueConfig,
    tenant_id: Option<Uuid>,
}

impl JobQueue {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            config: QueueConfig::default(),
            tenant_id: None,
        }
    }

    pub fn with_config(pool: PgPool, config: QueueConfig) -> Self {
        Self {
            pool,
            config,
            tenant_id: None,
        }
    }

    pub fn with_tenant(mut self, tenant_id: Uuid) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    /// Dispatch a job
    pub async fn dispatch<P: JobPayload>(&self, payload: P) -> Result<Uuid> {
        let mut job = Job::new(payload);
        if let Some(tenant_id) = self.tenant_id {
            job = job.with_tenant(tenant_id);
        }
        self.push(job).await
    }

    /// Dispatch a job with delay
    pub async fn dispatch_delayed<P: JobPayload>(
        &self,
        payload: P,
        delay_secs: u64,
    ) -> Result<Uuid> {
        let mut job = Job::new(payload).delay(delay_secs);
        if let Some(tenant_id) = self.tenant_id {
            job = job.with_tenant(tenant_id);
        }
        self.push(job).await
    }

    /// Dispatch a job at a specific time
    pub async fn dispatch_at<P: JobPayload>(
        &self,
        payload: P,
        time: DateTime<Utc>,
    ) -> Result<Uuid> {
        let mut job = Job::new(payload).schedule_at(time);
        if let Some(tenant_id) = self.tenant_id {
            job = job.with_tenant(tenant_id);
        }
        self.push(job).await
    }
}

#[async_trait]
impl Queue for JobQueue {
    async fn push(&self, job: Job) -> Result<Uuid> {
        let id = job.id;

        sqlx::query(
            r#"
            INSERT INTO jobs (id, tenant_id, queue, job_type, payload, status, priority, attempts, max_attempts, available_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(job.id)
        .bind(job.tenant_id)
        .bind(&job.queue)
        .bind(&job.job_type)
        .bind(&job.payload)
        .bind(serde_json::to_string(&job.status).unwrap_or_default())
        .bind(job.priority)
        .bind(job.attempts as i32)
        .bind(job.max_attempts as i32)
        .bind(job.available_at)
        .bind(job.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to push job", e))?;

        tracing::debug!(job_id = %id, queue = %job.queue, "Job pushed to queue");
        Ok(id)
    }

    async fn push_delayed(&self, mut job: Job, delay_secs: u64) -> Result<Uuid> {
        job.available_at = Utc::now() + chrono::Duration::seconds(delay_secs as i64);
        self.push(job).await
    }

    async fn pop(&self, queue: &str) -> Result<Option<Job>> {
        let tenant_condition = match self.tenant_id {
            Some(id) => format!("AND tenant_id = '{}'", id),
            None => String::new(),
        };

        let query = format!(
            r#"
            UPDATE jobs
            SET status = 'reserved', reserved_at = NOW(), attempts = attempts + 1
            WHERE id = (
                SELECT id FROM jobs
                WHERE queue = $1
                AND status = 'pending'
                AND available_at <= NOW()
                {}
                ORDER BY priority DESC, available_at ASC
                FOR UPDATE SKIP LOCKED
                LIMIT 1
            )
            RETURNING id, tenant_id, queue, job_type, payload, status, priority, attempts, max_attempts, last_error, available_at, reserved_at, completed_at, created_at
            "#,
            tenant_condition
        );

        let job: Option<JobRow> = sqlx::query_as(&query)
            .bind(queue)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to pop job", e))?;

        Ok(job.map(|r| r.into()))
    }

    async fn pop_batch(&self, queue: &str, count: u32) -> Result<Vec<Job>> {
        let mut jobs = Vec::new();
        for _ in 0..count {
            match self.pop(queue).await? {
                Some(job) => jobs.push(job),
                None => break,
            }
        }
        Ok(jobs)
    }

    async fn complete(&self, job_id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE jobs SET status = 'completed', completed_at = NOW() WHERE id = $1",
        )
        .bind(job_id)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to complete job", e))?;

        tracing::debug!(job_id = %job_id, "Job completed");
        Ok(())
    }

    async fn fail(&self, job_id: Uuid, error: &str) -> Result<()> {
        sqlx::query(
            "UPDATE jobs SET status = 'failed', last_error = $2, reserved_at = NULL WHERE id = $1",
        )
        .bind(job_id)
        .bind(error)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to fail job", e))?;

        tracing::debug!(job_id = %job_id, error = %error, "Job failed");
        Ok(())
    }

    async fn release(&self, job_id: Uuid, delay_secs: u64) -> Result<()> {
        let available_at = Utc::now() + chrono::Duration::seconds(delay_secs as i64);

        sqlx::query(
            "UPDATE jobs SET status = 'pending', reserved_at = NULL, available_at = $2 WHERE id = $1",
        )
        .bind(job_id)
        .bind(available_at)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to release job", e))?;

        tracing::debug!(job_id = %job_id, delay_secs = delay_secs, "Job released");
        Ok(())
    }

    async fn delete(&self, job_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM jobs WHERE id = $1")
            .bind(job_id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to delete job", e))?;

        Ok(())
    }

    async fn get(&self, job_id: Uuid) -> Result<Option<Job>> {
        let job: Option<JobRow> =
            sqlx::query_as("SELECT * FROM jobs WHERE id = $1")
                .bind(job_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to get job", e))?;

        Ok(job.map(|r| r.into()))
    }

    async fn size(&self, queue: &str) -> Result<u64> {
        let (count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM jobs WHERE queue = $1 AND status = 'pending'",
        )
        .bind(queue)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to get queue size", e))?;

        Ok(count as u64)
    }

    async fn clear(&self, queue: &str) -> Result<u64> {
        let result = sqlx::query("DELETE FROM jobs WHERE queue = $1")
            .bind(queue)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to clear queue", e))?;

        Ok(result.rows_affected())
    }

    async fn retry_failed(&self, queue: &str) -> Result<u64> {
        let result = sqlx::query(
            r#"
            UPDATE jobs
            SET status = 'pending', reserved_at = NULL, available_at = NOW()
            WHERE queue = $1 AND status = 'failed' AND attempts < max_attempts
            "#,
        )
        .bind(queue)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to retry failed jobs", e))?;

        Ok(result.rows_affected())
    }

    async fn release_stale(&self, older_than_secs: u64) -> Result<u64> {
        let threshold = Utc::now() - chrono::Duration::seconds(older_than_secs as i64);

        let result = sqlx::query(
            r#"
            UPDATE jobs
            SET status = 'pending', reserved_at = NULL
            WHERE status = 'reserved' AND reserved_at < $1
            "#,
        )
        .bind(threshold)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to release stale jobs", e))?;

        if result.rows_affected() > 0 {
            tracing::info!(count = result.rows_affected(), "Released stale jobs");
        }

        Ok(result.rows_affected())
    }
}

/// Database row for jobs
#[derive(sqlx::FromRow)]
struct JobRow {
    id: Uuid,
    tenant_id: Option<Uuid>,
    queue: String,
    job_type: String,
    payload: serde_json::Value,
    status: String,
    priority: Option<i32>,
    attempts: Option<i32>,
    max_attempts: Option<i32>,
    last_error: Option<String>,
    available_at: DateTime<Utc>,
    reserved_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl From<JobRow> for Job {
    fn from(row: JobRow) -> Self {
        Job {
            id: row.id,
            tenant_id: row.tenant_id,
            queue: row.queue,
            job_type: row.job_type,
            payload: row.payload,
            status: serde_json::from_str(&row.status).unwrap_or(JobStatus::Pending),
            priority: row.priority.unwrap_or(0),
            attempts: row.attempts.unwrap_or(0) as u32,
            max_attempts: row.max_attempts.unwrap_or(3) as u32,
            timeout_secs: 300,
            last_error: row.last_error,
            available_at: row.available_at,
            reserved_at: row.reserved_at,
            completed_at: row.completed_at,
            created_at: row.created_at,
        }
    }
}
