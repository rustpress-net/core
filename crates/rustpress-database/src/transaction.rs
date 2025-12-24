//! Transaction support for database operations.

use rustpress_core::error::{Error, Result};
use sqlx::{PgPool, Postgres};
use std::future::Future;

/// Transaction wrapper for coordinated database operations
pub struct Transaction<'a> {
    tx: Option<sqlx::Transaction<'a, Postgres>>,
}

impl<'a> Transaction<'a> {
    /// Begin a new transaction
    pub async fn begin(pool: &'a PgPool) -> Result<Self> {
        let tx = pool
            .begin()
            .await
            .map_err(|e| Error::database_with_source("Failed to begin transaction", e))?;

        Ok(Self { tx: Some(tx) })
    }

    /// Get a reference to the inner transaction
    pub fn inner(&mut self) -> &mut sqlx::Transaction<'a, Postgres> {
        self.tx.as_mut().expect("Transaction already consumed")
    }

    /// Commit the transaction
    pub async fn commit(mut self) -> Result<()> {
        let tx = self.tx.take().expect("Transaction already consumed");
        tx.commit()
            .await
            .map_err(|e| Error::database_with_source("Failed to commit transaction", e))
    }

    /// Rollback the transaction
    pub async fn rollback(mut self) -> Result<()> {
        let tx = self.tx.take().expect("Transaction already consumed");
        tx.rollback()
            .await
            .map_err(|e| Error::database_with_source("Failed to rollback transaction", e))
    }
}

impl<'a> Drop for Transaction<'a> {
    fn drop(&mut self) {
        if self.tx.is_some() {
            tracing::warn!("Transaction dropped without commit or rollback");
        }
    }
}

/// Execute a closure within a transaction
pub async fn with_transaction<'a, F, Fut, T>(pool: &'a PgPool, f: F) -> Result<T>
where
    F: FnOnce(&mut sqlx::Transaction<'a, Postgres>) -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| Error::database_with_source("Failed to begin transaction", e))?;

    match f(&mut tx).await {
        Ok(result) => {
            tx.commit()
                .await
                .map_err(|e| Error::database_with_source("Failed to commit transaction", e))?;
            Ok(result)
        }
        Err(e) => {
            // Attempt to rollback, but don't mask the original error
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}

/// Savepoint for nested transactions
pub struct Savepoint<'a> {
    tx: &'a mut sqlx::Transaction<'a, Postgres>,
    name: String,
    released: bool,
}

impl<'a> Savepoint<'a> {
    /// Create a savepoint
    pub async fn create(tx: &'a mut sqlx::Transaction<'a, Postgres>, name: &str) -> Result<Self> {
        sqlx::query(&format!("SAVEPOINT {}", name))
            .execute(&mut **tx)
            .await
            .map_err(|e| Error::database_with_source("Failed to create savepoint", e))?;

        Ok(Self {
            tx,
            name: name.to_string(),
            released: false,
        })
    }

    /// Release the savepoint (commit nested work)
    pub async fn release(mut self) -> Result<()> {
        sqlx::query(&format!("RELEASE SAVEPOINT {}", self.name))
            .execute(&mut **self.tx)
            .await
            .map_err(|e| Error::database_with_source("Failed to release savepoint", e))?;

        self.released = true;
        Ok(())
    }

    /// Rollback to the savepoint
    pub async fn rollback(mut self) -> Result<()> {
        sqlx::query(&format!("ROLLBACK TO SAVEPOINT {}", self.name))
            .execute(&mut **self.tx)
            .await
            .map_err(|e| Error::database_with_source("Failed to rollback to savepoint", e))?;

        self.released = true;
        Ok(())
    }
}

impl<'a> Drop for Savepoint<'a> {
    fn drop(&mut self) {
        if !self.released {
            tracing::warn!(name = %self.name, "Savepoint dropped without release or rollback");
        }
    }
}

/// Unit of Work pattern implementation
pub struct UnitOfWork<'a> {
    pool: &'a PgPool,
    tx: Option<sqlx::Transaction<'a, Postgres>>,
}

impl<'a> UnitOfWork<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool, tx: None }
    }

    /// Begin the unit of work
    pub async fn begin(&mut self) -> Result<()> {
        if self.tx.is_some() {
            return Err(Error::database("Unit of work already started"));
        }

        self.tx = Some(
            self.pool
                .begin()
                .await
                .map_err(|e| Error::database_with_source("Failed to begin unit of work", e))?,
        );

        Ok(())
    }

    /// Get a reference to the transaction
    pub fn tx(&mut self) -> Result<&mut sqlx::Transaction<'a, Postgres>> {
        self.tx
            .as_mut()
            .ok_or_else(|| Error::database("Unit of work not started"))
    }

    /// Commit all changes
    pub async fn commit(mut self) -> Result<()> {
        let tx = self
            .tx
            .take()
            .ok_or_else(|| Error::database("Unit of work not started"))?;

        tx.commit()
            .await
            .map_err(|e| Error::database_with_source("Failed to commit unit of work", e))
    }

    /// Rollback all changes
    pub async fn rollback(mut self) -> Result<()> {
        let tx = self
            .tx
            .take()
            .ok_or_else(|| Error::database("Unit of work not started"))?;

        tx.rollback()
            .await
            .map_err(|e| Error::database_with_source("Failed to rollback unit of work", e))
    }

    /// Check if unit of work is active
    pub fn is_active(&self) -> bool {
        self.tx.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a running PostgreSQL instance
    // In a real project, you'd use testcontainers or a test database

    #[test]
    fn test_unit_of_work_state() {
        // This is a basic unit test that doesn't require a database
        // Real integration tests would test with an actual database
    }
}
