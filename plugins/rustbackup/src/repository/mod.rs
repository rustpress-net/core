//! RustBackup Repository
//!
//! Database persistence layer for backup operations.

pub mod backup_repository;
pub mod schedule_repository;
pub mod storage_repository;
pub mod log_repository;

pub use backup_repository::BackupRepository;
pub use schedule_repository::ScheduleRepository;
pub use storage_repository::StorageRepository;
pub use log_repository::LogRepository;

use sqlx::PgPool;

/// Database connection pool wrapper
#[derive(Clone)]
pub struct DbPool {
    pool: PgPool,
}

impl DbPool {
    /// Create new database pool from connection string
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    /// Get the underlying pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}
