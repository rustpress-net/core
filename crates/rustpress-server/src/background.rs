//! Background task initialization and scheduling.
//!
//! This module handles the initialization of background workers and schedulers
//! for periodic tasks like publishing scheduled posts and cleaning up expired data.

use std::sync::Arc;
use tracing::{error, info};

use rustpress_jobs::{
    JobQueue, Scheduler, Schedule, Worker,
    PublishScheduledPostsJob, PublishScheduledPostsHandler,
    CleanThemePreviewsJob, CleanThemePreviewsHandler,
};

/// Initialize and start the job scheduler with periodic tasks
pub fn init_scheduler(job_queue: Arc<JobQueue>) -> Arc<Scheduler> {
    let scheduler = Arc::new(Scheduler::new(job_queue.clone()));

    // Schedule: Publish scheduled posts every minute
    scheduler.schedule_job(
        "publish_scheduled_posts",
        Schedule::every_minute(),
        PublishScheduledPostsJob { site_id: None },
    );

    // Schedule: Clean expired theme previews every hour
    scheduler.schedule_job(
        "clean_theme_previews",
        Schedule::hourly(),
        CleanThemePreviewsJob { site_id: None },
    );

    info!("Job scheduler initialized with periodic tasks:");
    info!("  - publish_scheduled_posts: every minute");
    info!("  - clean_theme_previews: hourly");

    scheduler
}

/// Start the background worker for processing jobs
pub fn start_worker(job_queue: Arc<JobQueue>, pool: sqlx::PgPool) {
    let worker = Worker::new(job_queue);

    // Register job handlers
    worker.register(PublishScheduledPostsHandler::new(pool.clone()));
    worker.register(CleanThemePreviewsHandler::new(pool.clone()));

    // Spawn worker in background
    tokio::spawn(async move {
        info!("Background job worker started");
        if let Err(e) = worker.run().await {
            error!("Worker error: {}", e);
        }
    });
}

/// Start the scheduler loop in a background task
pub fn start_scheduler(scheduler: Arc<Scheduler>) {
    tokio::spawn(async move {
        info!("Scheduler loop started");
        if let Err(e) = scheduler.run().await {
            error!("Scheduler error: {}", e);
        }
    });
}

/// Initialize all background tasks (scheduler + worker)
pub async fn init_background_tasks(job_queue: JobQueue, pool: sqlx::PgPool) -> Arc<Scheduler> {
    let job_queue_arc = Arc::new(job_queue);

    // Initialize and start worker
    start_worker(job_queue_arc.clone(), pool);

    // Initialize scheduler
    let scheduler = init_scheduler(job_queue_arc);

    // Start scheduler loop
    start_scheduler(scheduler.clone());

    scheduler
}
