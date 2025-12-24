//! Job definitions and traits.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rustpress_core::error::Result;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::any::Any;
use uuid::Uuid;

/// Job status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Reserved,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

impl Default for JobStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Job payload trait for type-safe job data
pub trait JobPayload: Serialize + DeserializeOwned + Send + Sync + 'static {
    /// Unique job type identifier
    fn job_type() -> &'static str;

    /// Queue name for this job type
    fn queue() -> &'static str {
        "default"
    }

    /// Maximum number of attempts
    fn max_attempts() -> u32 {
        3
    }

    /// Timeout in seconds
    fn timeout_secs() -> u64 {
        300
    }

    /// Delay before processing (in seconds)
    fn delay_secs() -> u64 {
        0
    }
}

/// Job handler trait
#[async_trait]
pub trait JobHandler: Send + Sync {
    /// The payload type this handler processes
    type Payload: JobPayload;

    /// Handle the job
    async fn handle(&self, payload: Self::Payload) -> Result<()>;

    /// Called when job fails
    async fn failed(&self, _payload: Self::Payload, _error: &str) -> Result<()> {
        Ok(())
    }

    /// Called after job completes successfully
    async fn completed(&self, _payload: Self::Payload) -> Result<()> {
        Ok(())
    }
}

/// Job entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub queue: String,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub status: JobStatus,
    pub priority: i32,
    pub attempts: u32,
    pub max_attempts: u32,
    pub timeout_secs: u64,
    pub last_error: Option<String>,
    pub available_at: DateTime<Utc>,
    pub reserved_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl Job {
    /// Create a new job from a payload
    pub fn new<P: JobPayload>(payload: P) -> Self {
        Self {
            id: Uuid::now_v7(),
            tenant_id: None,
            queue: P::queue().to_string(),
            job_type: P::job_type().to_string(),
            payload: serde_json::to_value(payload).unwrap_or_default(),
            status: JobStatus::Pending,
            priority: 0,
            attempts: 0,
            max_attempts: P::max_attempts(),
            timeout_secs: P::timeout_secs(),
            last_error: None,
            available_at: Utc::now() + chrono::Duration::seconds(P::delay_secs() as i64),
            reserved_at: None,
            completed_at: None,
            created_at: Utc::now(),
        }
    }

    /// Create a new job with custom settings
    pub fn with_settings<P: JobPayload>(
        payload: P,
        queue: impl Into<String>,
        priority: i32,
        delay_secs: u64,
    ) -> Self {
        let mut job = Self::new(payload);
        job.queue = queue.into();
        job.priority = priority;
        job.available_at = Utc::now() + chrono::Duration::seconds(delay_secs as i64);
        job
    }

    /// Set tenant ID
    pub fn with_tenant(mut self, tenant_id: Uuid) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    /// Set priority (higher = more important)
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Delay job execution
    pub fn delay(mut self, seconds: u64) -> Self {
        self.available_at = Utc::now() + chrono::Duration::seconds(seconds as i64);
        self
    }

    /// Schedule job for a specific time
    pub fn schedule_at(mut self, time: DateTime<Utc>) -> Self {
        self.available_at = time;
        self
    }

    /// Check if job can be retried
    pub fn can_retry(&self) -> bool {
        self.attempts < self.max_attempts
    }

    /// Check if job has timed out
    pub fn has_timed_out(&self) -> bool {
        if let Some(reserved_at) = self.reserved_at {
            let elapsed = Utc::now().signed_duration_since(reserved_at);
            elapsed.num_seconds() > self.timeout_secs as i64
        } else {
            false
        }
    }

    /// Deserialize payload to typed struct
    pub fn payload<P: JobPayload>(&self) -> Result<P> {
        serde_json::from_value(self.payload.clone()).map_err(|e| {
            rustpress_core::error::Error::Job {
                job_id: self.id.to_string(),
                message: format!("Failed to deserialize payload: {}", e),
            }
        })
    }

    /// Mark job as reserved
    pub fn reserve(&mut self) {
        self.status = JobStatus::Reserved;
        self.reserved_at = Some(Utc::now());
        self.attempts += 1;
    }

    /// Mark job as processing
    pub fn start_processing(&mut self) {
        self.status = JobStatus::Processing;
    }

    /// Mark job as completed
    pub fn complete(&mut self) {
        self.status = JobStatus::Completed;
        self.completed_at = Some(Utc::now());
    }

    /// Mark job as failed
    pub fn fail(&mut self, error: impl Into<String>) {
        self.status = JobStatus::Failed;
        self.last_error = Some(error.into());
        self.reserved_at = None;
    }

    /// Release job back to queue for retry
    pub fn release(&mut self, delay_secs: u64) {
        self.status = JobStatus::Pending;
        self.reserved_at = None;
        self.available_at = Utc::now() + chrono::Duration::seconds(delay_secs as i64);
    }
}

/// Common job types
pub mod jobs {
    use super::*;

    /// Send email job
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SendEmailJob {
        pub to: String,
        pub subject: String,
        pub body: String,
        pub html: bool,
    }

    impl JobPayload for SendEmailJob {
        fn job_type() -> &'static str {
            "send_email"
        }

        fn queue() -> &'static str {
            "emails"
        }
    }

    /// Process webhook job
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ProcessWebhookJob {
        pub webhook_id: Uuid,
        pub event_type: String,
        pub payload: serde_json::Value,
    }

    impl JobPayload for ProcessWebhookJob {
        fn job_type() -> &'static str {
            "process_webhook"
        }

        fn queue() -> &'static str {
            "webhooks"
        }

        fn max_attempts() -> u32 {
            5
        }
    }

    /// Cleanup job
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CleanupJob {
        pub cleanup_type: String,
        pub older_than_days: u32,
    }

    impl JobPayload for CleanupJob {
        fn job_type() -> &'static str {
            "cleanup"
        }

        fn queue() -> &'static str {
            "maintenance"
        }

        fn timeout_secs() -> u64 {
            3600 // 1 hour
        }
    }

    /// Generate thumbnail job
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GenerateThumbnailJob {
        pub media_id: Uuid,
        pub sizes: Vec<(u32, u32)>,
    }

    impl JobPayload for GenerateThumbnailJob {
        fn job_type() -> &'static str {
            "generate_thumbnail"
        }

        fn queue() -> &'static str {
            "media"
        }
    }

    /// Sync external data job
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SyncExternalDataJob {
        pub source: String,
        pub entity_type: String,
        pub external_id: String,
    }

    impl JobPayload for SyncExternalDataJob {
        fn job_type() -> &'static str {
            "sync_external"
        }

        fn queue() -> &'static str {
            "sync"
        }

        fn max_attempts() -> u32 {
            3
        }

        fn timeout_secs() -> u64 {
            600
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jobs::*;

    #[test]
    fn test_job_creation() {
        let payload = SendEmailJob {
            to: "test@example.com".to_string(),
            subject: "Test".to_string(),
            body: "Hello".to_string(),
            html: false,
        };

        let job = Job::new(payload);
        assert_eq!(job.job_type, "send_email");
        assert_eq!(job.queue, "emails");
        assert_eq!(job.status, JobStatus::Pending);
        assert_eq!(job.attempts, 0);
    }

    #[test]
    fn test_job_reservation() {
        let payload = SendEmailJob {
            to: "test@example.com".to_string(),
            subject: "Test".to_string(),
            body: "Hello".to_string(),
            html: false,
        };

        let mut job = Job::new(payload);
        job.reserve();

        assert_eq!(job.status, JobStatus::Reserved);
        assert_eq!(job.attempts, 1);
        assert!(job.reserved_at.is_some());
    }

    #[test]
    fn test_job_retry() {
        let payload = SendEmailJob {
            to: "test@example.com".to_string(),
            subject: "Test".to_string(),
            body: "Hello".to_string(),
            html: false,
        };

        let mut job = Job::new(payload);

        // Reserve and fail multiple times
        for i in 1..=3 {
            job.reserve();
            assert_eq!(job.attempts, i);
            job.fail("Error");
        }

        assert!(!job.can_retry());
    }

    #[test]
    fn test_job_delay() {
        let payload = SendEmailJob {
            to: "test@example.com".to_string(),
            subject: "Test".to_string(),
            body: "Hello".to_string(),
            html: false,
        };

        let job = Job::new(payload).delay(60);
        assert!(job.available_at > Utc::now());
    }
}
