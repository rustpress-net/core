//! Graceful shutdown handling.

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tokio::sync::broadcast;
use tracing::{info, warn};

/// Shutdown signal receiver
pub type ShutdownReceiver = broadcast::Receiver<()>;

/// Shutdown signal sender
pub type ShutdownSender = broadcast::Sender<()>;

/// Graceful shutdown coordinator
#[derive(Clone)]
pub struct ShutdownController {
    /// Broadcast sender for shutdown signal
    sender: ShutdownSender,
    /// Flag indicating if shutdown has been initiated
    is_shutting_down: Arc<AtomicBool>,
    /// Timeout for graceful shutdown
    timeout: Duration,
}

impl ShutdownController {
    /// Create a new shutdown controller
    pub fn new(timeout: Duration) -> Self {
        let (sender, _) = broadcast::channel(1);
        Self {
            sender,
            is_shutting_down: Arc::new(AtomicBool::new(false)),
            timeout,
        }
    }

    /// Create with default 30 second timeout
    pub fn with_default_timeout() -> Self {
        Self::new(Duration::from_secs(30))
    }

    /// Subscribe to shutdown signal
    pub fn subscribe(&self) -> ShutdownReceiver {
        self.sender.subscribe()
    }

    /// Initiate shutdown
    pub fn shutdown(&self) {
        if self
            .is_shutting_down
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            info!("Initiating graceful shutdown");
            let _ = self.sender.send(());
        }
    }

    /// Check if shutdown has been initiated
    pub fn is_shutting_down(&self) -> bool {
        self.is_shutting_down.load(Ordering::SeqCst)
    }

    /// Get the shutdown timeout
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Wait for shutdown signal with timeout
    pub async fn wait_for_shutdown(&self) {
        let mut receiver = self.subscribe();

        tokio::select! {
            _ = receiver.recv() => {
                info!("Received shutdown signal");
            }
            _ = tokio::time::sleep(self.timeout) => {
                warn!("Shutdown timeout reached");
            }
        }
    }
}

impl Default for ShutdownController {
    fn default() -> Self {
        Self::with_default_timeout()
    }
}

/// Listen for OS shutdown signals (SIGINT, SIGTERM)
pub async fn listen_for_shutdown_signals(controller: ShutdownController) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C signal");
        }
        _ = terminate => {
            info!("Received terminate signal");
        }
    }

    controller.shutdown();
}

/// Graceful shutdown handler for the HTTP server
pub async fn graceful_shutdown(controller: ShutdownController) {
    let mut receiver = controller.subscribe();

    // Wait for shutdown signal
    let _ = receiver.recv().await;

    info!("Starting graceful shutdown sequence");
}

/// Shutdown handle for tracking active tasks
#[derive(Clone)]
pub struct ShutdownHandle {
    controller: ShutdownController,
    active_tasks: Arc<std::sync::atomic::AtomicUsize>,
}

impl ShutdownHandle {
    /// Create a new shutdown handle
    pub fn new(controller: ShutdownController) -> Self {
        Self {
            controller,
            active_tasks: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    /// Register an active task
    pub fn register_task(&self) -> TaskGuard {
        self.active_tasks.fetch_add(1, Ordering::SeqCst);
        TaskGuard {
            active_tasks: Arc::clone(&self.active_tasks),
        }
    }

    /// Get the number of active tasks
    pub fn active_task_count(&self) -> usize {
        self.active_tasks.load(Ordering::SeqCst)
    }

    /// Wait for all tasks to complete
    pub async fn wait_for_tasks(&self, timeout: Duration) {
        let start = std::time::Instant::now();

        loop {
            let count = self.active_task_count();
            if count == 0 {
                info!("All active tasks completed");
                break;
            }

            if start.elapsed() > timeout {
                warn!(
                    "Shutdown timeout reached with {} tasks still active",
                    count
                );
                break;
            }

            info!("Waiting for {} active tasks to complete", count);
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Check if shutdown has been initiated
    pub fn is_shutting_down(&self) -> bool {
        self.controller.is_shutting_down()
    }

    /// Subscribe to shutdown signal
    pub fn subscribe(&self) -> ShutdownReceiver {
        self.controller.subscribe()
    }
}

/// Guard that decrements the active task count when dropped
pub struct TaskGuard {
    active_tasks: Arc<std::sync::atomic::AtomicUsize>,
}

impl Drop for TaskGuard {
    fn drop(&mut self) {
        self.active_tasks.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Shutdown-aware sleep that returns early if shutdown is initiated
pub async fn shutdown_aware_sleep(
    duration: Duration,
    controller: &ShutdownController,
) -> bool {
    let mut receiver = controller.subscribe();

    tokio::select! {
        _ = tokio::time::sleep(duration) => false,
        _ = receiver.recv() => {
            info!("Sleep interrupted by shutdown signal");
            true
        }
    }
}

/// Run a task with shutdown awareness
pub async fn run_with_shutdown<F, Fut>(
    controller: ShutdownController,
    task: F,
) where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()>,
{
    let mut receiver = controller.subscribe();

    tokio::select! {
        _ = task() => {
            info!("Task completed normally");
        }
        _ = receiver.recv() => {
            info!("Task interrupted by shutdown signal");
        }
    }
}

/// Shutdown phases for ordered cleanup
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ShutdownPhase {
    /// Stop accepting new connections
    StopAccepting,
    /// Drain existing connections
    DrainConnections,
    /// Stop background workers
    StopWorkers,
    /// Flush caches
    FlushCaches,
    /// Close database connections
    CloseDatabase,
    /// Final cleanup
    Cleanup,
}

impl ShutdownPhase {
    pub fn all() -> &'static [ShutdownPhase] {
        &[
            ShutdownPhase::StopAccepting,
            ShutdownPhase::DrainConnections,
            ShutdownPhase::StopWorkers,
            ShutdownPhase::FlushCaches,
            ShutdownPhase::CloseDatabase,
            ShutdownPhase::Cleanup,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            ShutdownPhase::StopAccepting => "Stop accepting connections",
            ShutdownPhase::DrainConnections => "Drain existing connections",
            ShutdownPhase::StopWorkers => "Stop background workers",
            ShutdownPhase::FlushCaches => "Flush caches",
            ShutdownPhase::CloseDatabase => "Close database connections",
            ShutdownPhase::Cleanup => "Final cleanup",
        }
    }
}

/// Ordered shutdown executor
pub struct ShutdownExecutor {
    controller: ShutdownController,
    handlers: Vec<(ShutdownPhase, Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>)>,
}

impl ShutdownExecutor {
    pub fn new(controller: ShutdownController) -> Self {
        Self {
            controller,
            handlers: Vec::new(),
        }
    }

    /// Register a shutdown handler for a specific phase
    pub fn register<F, Fut>(&mut self, phase: ShutdownPhase, handler: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.handlers.push((
            phase,
            Box::new(move || Box::pin(handler())),
        ));
    }

    /// Execute shutdown in order
    pub async fn execute(mut self) {
        // Sort handlers by phase
        self.handlers.sort_by_key(|(phase, _)| *phase);

        let timeout_per_phase = self.controller.timeout() / (ShutdownPhase::all().len() as u32);

        for phase in ShutdownPhase::all() {
            info!("Executing shutdown phase: {}", phase.name());

            let phase_handlers: Vec<_> = self
                .handlers
                .iter()
                .filter(|(p, _)| p == phase)
                .collect();

            if phase_handlers.is_empty() {
                continue;
            }

            // Execute all handlers for this phase with timeout
            let phase_future = async {
                for (_, handler) in phase_handlers {
                    handler().await;
                }
            };

            match tokio::time::timeout(timeout_per_phase, phase_future).await {
                Ok(_) => {
                    info!("Shutdown phase '{}' completed", phase.name());
                }
                Err(_) => {
                    warn!("Shutdown phase '{}' timed out", phase.name());
                }
            }
        }

        info!("Graceful shutdown complete");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_shutdown_controller() {
        let controller = ShutdownController::with_default_timeout();

        assert!(!controller.is_shutting_down());

        controller.shutdown();

        assert!(controller.is_shutting_down());
    }

    #[tokio::test]
    async fn test_shutdown_handle() {
        let controller = ShutdownController::with_default_timeout();
        let handle = ShutdownHandle::new(controller);

        assert_eq!(handle.active_task_count(), 0);

        {
            let _guard = handle.register_task();
            assert_eq!(handle.active_task_count(), 1);

            let _guard2 = handle.register_task();
            assert_eq!(handle.active_task_count(), 2);
        }

        // Guards dropped, count should be 0
        assert_eq!(handle.active_task_count(), 0);
    }

    #[tokio::test]
    async fn test_shutdown_phases() {
        let phases = ShutdownPhase::all();
        assert_eq!(phases.len(), 6);
        assert_eq!(phases[0], ShutdownPhase::StopAccepting);
        assert_eq!(phases[5], ShutdownPhase::Cleanup);
    }
}
