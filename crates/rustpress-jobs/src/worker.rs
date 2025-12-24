//! Job worker implementation.

use crate::job::{Job, JobHandler, JobPayload, JobStatus};
use crate::queue::{JobQueue, Queue, QueueConfig};
use async_trait::async_trait;
use dashmap::DashMap;
use parking_lot::RwLock;
use rustpress_core::error::{Error, Result};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use uuid::Uuid;

/// Worker for processing jobs from a queue
pub struct Worker {
    queue: Arc<JobQueue>,
    handlers: Arc<DashMap<String, Arc<dyn JobHandlerDyn>>>,
    config: WorkerConfig,
    running: Arc<AtomicBool>,
}

/// Worker configuration
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// Queues to process
    pub queues: Vec<String>,
    /// Number of concurrent jobs
    pub concurrency: usize,
    /// Sleep duration when no jobs available
    pub sleep_on_empty: Duration,
    /// Maximum jobs to process before stopping (None = unlimited)
    pub max_jobs: Option<u64>,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            queues: vec!["default".to_string()],
            concurrency: 4,
            sleep_on_empty: Duration::from_secs(1),
            max_jobs: None,
        }
    }
}

impl Worker {
    pub fn new(queue: Arc<JobQueue>) -> Self {
        Self {
            queue,
            handlers: Arc::new(DashMap::new()),
            config: WorkerConfig::default(),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn with_config(queue: Arc<JobQueue>, config: WorkerConfig) -> Self {
        Self {
            queue,
            handlers: Arc::new(DashMap::new()),
            config,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Register a job handler
    pub fn register<H, P>(&self, handler: H)
    where
        H: JobHandler<Payload = P> + 'static,
        P: JobPayload,
    {
        let job_type = P::job_type().to_string();
        self.handlers
            .insert(job_type.clone(), Arc::new(TypedHandler { handler }));
        tracing::info!(job_type = %job_type, "Registered job handler");
    }

    /// Start the worker
    pub async fn run(&self) -> Result<()> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Err(Error::internal("Worker already running"));
        }

        let semaphore = Arc::new(Semaphore::new(self.config.concurrency));
        let mut jobs_processed = 0u64;

        tracing::info!(
            queues = ?self.config.queues,
            concurrency = self.config.concurrency,
            "Worker started"
        );

        while self.running.load(Ordering::SeqCst) {
            // Check max jobs limit
            if let Some(max) = self.config.max_jobs {
                if jobs_processed >= max {
                    tracing::info!(count = jobs_processed, "Reached max jobs limit");
                    break;
                }
            }

            let mut found_job = false;

            for queue_name in &self.config.queues {
                // Acquire permit before fetching job
                let permit = semaphore.clone().acquire_owned().await.unwrap();

                if let Some(job) = self.queue.pop(queue_name).await? {
                    found_job = true;
                    jobs_processed += 1;

                    let handlers = self.handlers.clone();
                    let queue = self.queue.clone();

                    // Process job in background
                    tokio::spawn(async move {
                        let _permit = permit; // Hold permit until done
                        let job_id = job.id;
                        let job_type = job.job_type.clone();

                        match Self::process_job(&handlers, &queue, job).await {
                            Ok(()) => {
                                tracing::debug!(job_id = %job_id, job_type = %job_type, "Job processed successfully");
                            }
                            Err(e) => {
                                tracing::error!(job_id = %job_id, job_type = %job_type, error = %e, "Job processing failed");
                            }
                        }
                    });
                } else {
                    drop(permit);
                }
            }

            if !found_job {
                tokio::time::sleep(self.config.sleep_on_empty).await;
            }
        }

        self.running.store(false, Ordering::SeqCst);
        tracing::info!(jobs_processed = jobs_processed, "Worker stopped");

        Ok(())
    }

    /// Stop the worker
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        tracing::info!("Worker stop requested");
    }

    /// Check if worker is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    async fn process_job(
        handlers: &DashMap<String, Arc<dyn JobHandlerDyn>>,
        queue: &JobQueue,
        job: Job,
    ) -> Result<()> {
        let job_id = job.id;
        let job_type = job.job_type.clone();

        // Find handler
        let handler = handlers.get(&job_type).map(|h| h.clone());

        match handler {
            Some(handler) => {
                // Process with timeout
                let timeout = Duration::from_secs(job.timeout_secs);
                let result = tokio::time::timeout(timeout, handler.handle_job(&job)).await;

                match result {
                    Ok(Ok(())) => {
                        queue.complete(job_id).await?;
                    }
                    Ok(Err(e)) => {
                        let error = e.to_string();
                        if job.can_retry() {
                            // Retry with exponential backoff
                            let delay = 60 * (2_u64.pow(job.attempts - 1));
                            queue.release(job_id, delay).await?;
                        } else {
                            queue.fail(job_id, &error).await?;
                        }
                    }
                    Err(_) => {
                        let error = "Job timed out";
                        if job.can_retry() {
                            queue.release(job_id, 60).await?;
                        } else {
                            queue.fail(job_id, error).await?;
                        }
                    }
                }
            }
            None => {
                let error = format!("No handler registered for job type: {}", job_type);
                queue.fail(job_id, &error).await?;
            }
        }

        Ok(())
    }
}

/// Dynamic job handler trait for type erasure
#[async_trait]
trait JobHandlerDyn: Send + Sync {
    async fn handle_job(&self, job: &Job) -> Result<()>;
}

/// Typed handler wrapper
struct TypedHandler<H, P>
where
    H: JobHandler<Payload = P>,
    P: JobPayload,
{
    handler: H,
}

#[async_trait]
impl<H, P> JobHandlerDyn for TypedHandler<H, P>
where
    H: JobHandler<Payload = P> + Send + Sync,
    P: JobPayload,
{
    async fn handle_job(&self, job: &Job) -> Result<()> {
        let payload: P = job.payload()?;
        self.handler.handle(payload).await
    }
}

/// Worker pool for managing multiple workers
pub struct WorkerPool {
    workers: Vec<Arc<Worker>>,
    handles: RwLock<Vec<tokio::task::JoinHandle<Result<()>>>>,
}

impl WorkerPool {
    pub fn new(queue: Arc<JobQueue>, num_workers: usize, config: WorkerConfig) -> Self {
        let workers: Vec<_> = (0..num_workers)
            .map(|_| Arc::new(Worker::with_config(queue.clone(), config.clone())))
            .collect();

        Self {
            workers,
            handles: RwLock::new(Vec::new()),
        }
    }

    /// Register a handler on all workers
    pub fn register<H, P>(&self, handler: H)
    where
        H: JobHandler<Payload = P> + Clone + 'static,
        P: JobPayload,
    {
        for worker in &self.workers {
            worker.register(handler.clone());
        }
    }

    /// Start all workers
    pub fn start(&self) {
        let mut handles = self.handles.write();

        for (i, worker) in self.workers.iter().enumerate() {
            let worker = worker.clone();
            let handle = tokio::spawn(async move {
                tracing::info!(worker_id = i, "Starting worker");
                worker.run().await
            });
            handles.push(handle);
        }
    }

    /// Stop all workers
    pub fn stop(&self) {
        for worker in &self.workers {
            worker.stop();
        }
    }

    /// Wait for all workers to complete
    pub async fn wait(&self) -> Vec<Result<()>> {
        let handles: Vec<_> = {
            let mut h = self.handles.write();
            std::mem::take(&mut *h)
        };

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(Err(Error::internal(format!("Worker panicked: {}", e)))),
            }
        }
        results
    }

    /// Get worker count
    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_config() {
        let config = WorkerConfig::default();
        assert_eq!(config.concurrency, 4);
        assert!(config.queues.contains(&"default".to_string()));
    }
}
