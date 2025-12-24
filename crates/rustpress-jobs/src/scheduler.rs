//! Job scheduler for recurring and scheduled tasks.

use crate::job::{Job, JobPayload};
use crate::queue::{JobQueue, Queue};
use chrono::{DateTime, Datelike, Duration, Utc};
use parking_lot::RwLock;
use rustpress_core::error::{Error, Result};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Scheduled task definition
pub struct ScheduledTask {
    pub name: String,
    pub schedule: Schedule,
    pub job_factory: Box<dyn Fn() -> Job + Send + Sync>,
    pub enabled: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
}

impl ScheduledTask {
    pub fn new<F>(name: impl Into<String>, schedule: Schedule, factory: F) -> Self
    where
        F: Fn() -> Job + Send + Sync + 'static,
    {
        let schedule_clone = schedule.clone();
        Self {
            name: name.into(),
            schedule,
            job_factory: Box::new(factory),
            enabled: true,
            last_run: None,
            next_run: Some(schedule_clone.next_run_time()),
        }
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    pub fn is_due(&self) -> bool {
        if !self.enabled {
            return false;
        }
        self.next_run
            .map(|t| Utc::now() >= t)
            .unwrap_or(false)
    }

    pub fn update_schedule(&mut self) {
        self.last_run = Some(Utc::now());
        self.next_run = Some(self.schedule.next_run_time());
    }
}

/// Schedule definition
#[derive(Debug, Clone)]
pub enum Schedule {
    /// Run every N seconds
    EverySeconds(u64),
    /// Run every N minutes
    EveryMinutes(u64),
    /// Run every N hours
    EveryHours(u64),
    /// Run daily at specified hour (0-23)
    DailyAt(u32),
    /// Run weekly on specified day (0=Sunday) at hour
    WeeklyAt(u32, u32),
    /// Run at specific times using cron-like expression
    Cron(CronSchedule),
}

impl Schedule {
    pub fn every_second() -> Self {
        Self::EverySeconds(1)
    }

    pub fn every_minute() -> Self {
        Self::EveryMinutes(1)
    }

    pub fn every_five_minutes() -> Self {
        Self::EveryMinutes(5)
    }

    pub fn every_ten_minutes() -> Self {
        Self::EveryMinutes(10)
    }

    pub fn every_fifteen_minutes() -> Self {
        Self::EveryMinutes(15)
    }

    pub fn every_thirty_minutes() -> Self {
        Self::EveryMinutes(30)
    }

    pub fn hourly() -> Self {
        Self::EveryHours(1)
    }

    pub fn daily() -> Self {
        Self::DailyAt(0)
    }

    pub fn daily_at(hour: u32) -> Self {
        Self::DailyAt(hour)
    }

    pub fn weekly() -> Self {
        Self::WeeklyAt(0, 0)
    }

    pub fn next_run_time(&self) -> DateTime<Utc> {
        let now = Utc::now();
        match self {
            Schedule::EverySeconds(secs) => now + Duration::seconds(*secs as i64),
            Schedule::EveryMinutes(mins) => now + Duration::minutes(*mins as i64),
            Schedule::EveryHours(hours) => now + Duration::hours(*hours as i64),
            Schedule::DailyAt(hour) => {
                let today = now.date_naive();
                let time = chrono::NaiveTime::from_hms_opt(*hour, 0, 0)
                    .unwrap_or(chrono::NaiveTime::MIN);
                let datetime = today.and_time(time);
                let scheduled = datetime.and_utc();
                if scheduled <= now {
                    scheduled + Duration::days(1)
                } else {
                    scheduled
                }
            }
            Schedule::WeeklyAt(day, hour) => {
                let today = now.date_naive();
                let time = chrono::NaiveTime::from_hms_opt(*hour, 0, 0)
                    .unwrap_or(chrono::NaiveTime::MIN);
                let current_weekday = today.weekday().num_days_from_sunday();
                let target_day = *day;
                let days_until = if target_day >= current_weekday {
                    target_day - current_weekday
                } else {
                    7 - (current_weekday - target_day)
                };
                let target_date = today + Duration::days(days_until as i64);
                target_date.and_time(time).and_utc()
            }
            Schedule::Cron(cron) => cron.next_run_time(),
        }
    }
}

/// Simple cron-like schedule
#[derive(Debug, Clone)]
pub struct CronSchedule {
    pub minute: Option<u32>,
    pub hour: Option<u32>,
    pub day_of_month: Option<u32>,
    pub month: Option<u32>,
    pub day_of_week: Option<u32>,
}

impl CronSchedule {
    pub fn new() -> Self {
        Self {
            minute: None,
            hour: None,
            day_of_month: None,
            month: None,
            day_of_week: None,
        }
    }

    pub fn minute(mut self, min: u32) -> Self {
        self.minute = Some(min);
        self
    }

    pub fn hour(mut self, hour: u32) -> Self {
        self.hour = Some(hour);
        self
    }

    pub fn day_of_month(mut self, day: u32) -> Self {
        self.day_of_month = Some(day);
        self
    }

    pub fn month(mut self, month: u32) -> Self {
        self.month = Some(month);
        self
    }

    pub fn day_of_week(mut self, day: u32) -> Self {
        self.day_of_week = Some(day);
        self
    }

    pub fn next_run_time(&self) -> DateTime<Utc> {
        // Simplified implementation - just adds 1 day for now
        // A real implementation would properly calculate based on cron fields
        Utc::now() + Duration::days(1)
    }
}

impl Default for CronSchedule {
    fn default() -> Self {
        Self::new()
    }
}

/// Job scheduler
pub struct Scheduler {
    queue: Arc<JobQueue>,
    tasks: RwLock<HashMap<String, ScheduledTask>>,
    running: Arc<AtomicBool>,
    check_interval: std::time::Duration,
}

impl Scheduler {
    pub fn new(queue: Arc<JobQueue>) -> Self {
        Self {
            queue,
            tasks: RwLock::new(HashMap::new()),
            running: Arc::new(AtomicBool::new(false)),
            check_interval: std::time::Duration::from_secs(60),
        }
    }

    pub fn with_check_interval(mut self, interval: std::time::Duration) -> Self {
        self.check_interval = interval;
        self
    }

    /// Schedule a task
    pub fn schedule(&self, task: ScheduledTask) -> &Self {
        let name = task.name.clone();
        self.tasks.write().insert(name.clone(), task);
        tracing::info!(task = %name, "Scheduled task registered");
        self
    }

    /// Schedule a recurring job
    pub fn schedule_job<P: JobPayload + Clone + 'static>(
        &self,
        name: impl Into<String>,
        schedule: Schedule,
        payload: P,
    ) -> &Self {
        let task = ScheduledTask::new(name, schedule, move || Job::new(payload.clone()));
        self.schedule(task)
    }

    /// Remove a scheduled task
    pub fn unschedule(&self, name: &str) -> bool {
        self.tasks.write().remove(name).is_some()
    }

    /// Enable a task
    pub fn enable(&self, name: &str) -> bool {
        if let Some(task) = self.tasks.write().get_mut(name) {
            task.enabled = true;
            task.next_run = Some(task.schedule.next_run_time());
            true
        } else {
            false
        }
    }

    /// Disable a task
    pub fn disable(&self, name: &str) -> bool {
        if let Some(task) = self.tasks.write().get_mut(name) {
            task.enabled = false;
            true
        } else {
            false
        }
    }

    /// Get task status
    pub fn status(&self, name: &str) -> Option<TaskStatus> {
        self.tasks.read().get(name).map(|t| TaskStatus {
            name: t.name.clone(),
            enabled: t.enabled,
            last_run: t.last_run,
            next_run: t.next_run,
        })
    }

    /// List all tasks
    pub fn list_tasks(&self) -> Vec<TaskStatus> {
        self.tasks
            .read()
            .values()
            .map(|t| TaskStatus {
                name: t.name.clone(),
                enabled: t.enabled,
                last_run: t.last_run,
                next_run: t.next_run,
            })
            .collect()
    }

    /// Run the scheduler
    pub async fn run(&self) -> Result<()> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Err(Error::internal("Scheduler already running"));
        }

        tracing::info!("Scheduler started");

        while self.running.load(Ordering::SeqCst) {
            self.process_due_tasks().await?;
            tokio::time::sleep(self.check_interval).await;
        }

        tracing::info!("Scheduler stopped");
        Ok(())
    }

    /// Stop the scheduler
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Check if scheduler is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Process all due tasks
    async fn process_due_tasks(&self) -> Result<()> {
        // Collect jobs to dispatch while holding the lock, then release it before async operations
        let jobs_to_dispatch: Vec<(String, Job)> = {
            let mut tasks = self.tasks.write();
            let due_task_names: Vec<String> = tasks
                .iter()
                .filter(|(_, t)| t.is_due())
                .map(|(name, _)| name.clone())
                .collect();

            let mut jobs = Vec::new();
            for task_name in due_task_names {
                if let Some(task) = tasks.get_mut(&task_name) {
                    let job = (task.job_factory)();
                    task.update_schedule();
                    jobs.push((task_name, job));
                }
            }
            jobs
        }; // Lock is released here

        // Now dispatch jobs without holding the lock
        for (task_name, job) in jobs_to_dispatch {
            match self.queue.push(job).await {
                Ok(job_id) => {
                    tracing::info!(
                        task = %task_name,
                        job_id = %job_id,
                        "Scheduled task dispatched"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        task = %task_name,
                        error = %e,
                        "Failed to dispatch scheduled task"
                    );
                }
            }
        }

        Ok(())
    }

    /// Run due tasks once (for testing)
    pub async fn tick(&self) -> Result<u32> {
        // Collect jobs to dispatch while holding the lock
        let jobs_to_dispatch: Vec<Job> = {
            let mut tasks = self.tasks.write();
            let due_task_names: Vec<String> = tasks
                .iter()
                .filter(|(_, t)| t.is_due())
                .map(|(name, _)| name.clone())
                .collect();

            let mut jobs = Vec::new();
            for task_name in due_task_names {
                if let Some(task) = tasks.get_mut(&task_name) {
                    let job = (task.job_factory)();
                    task.update_schedule();
                    jobs.push(job);
                }
            }
            jobs
        }; // Lock is released here

        // Dispatch jobs without holding the lock
        let count = jobs_to_dispatch.len() as u32;
        for job in jobs_to_dispatch {
            self.queue.push(job).await?;
        }

        Ok(count)
    }
}

/// Task status info
#[derive(Debug, Clone)]
pub struct TaskStatus {
    pub name: String,
    pub enabled: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_every_seconds() {
        let schedule = Schedule::EverySeconds(30);
        let next = schedule.next_run_time();
        let now = Utc::now();
        assert!(next > now);
        assert!(next <= now + Duration::seconds(31));
    }

    #[test]
    fn test_schedule_every_minutes() {
        let schedule = Schedule::EveryMinutes(5);
        let next = schedule.next_run_time();
        let now = Utc::now();
        assert!(next > now);
        assert!(next <= now + Duration::minutes(6));
    }

    #[test]
    fn test_scheduled_task_is_due() {
        let schedule = Schedule::EverySeconds(0); // Immediate
        let task = ScheduledTask::new("test", schedule, || {
            Job::new(crate::job::jobs::CleanupJob {
                cleanup_type: "test".to_string(),
                older_than_days: 30,
            })
        });

        // Task should be due since next_run is now
        assert!(task.is_due());
    }

    #[test]
    fn test_disabled_task_not_due() {
        let schedule = Schedule::EverySeconds(0);
        let task = ScheduledTask::new("test", schedule, || {
            Job::new(crate::job::jobs::CleanupJob {
                cleanup_type: "test".to_string(),
                older_than_days: 30,
            })
        })
        .disabled();

        assert!(!task.is_due());
    }
}
