//! Job handlers for RustPress background tasks.
//!
//! This module contains handlers for scheduled tasks like publishing
//! scheduled posts and cleaning up expired theme previews.

use async_trait::async_trait;
use rustpress_core::error::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{error, info};
use uuid::Uuid;

use crate::job::{JobHandler, JobPayload};

/// Publish scheduled posts job - runs periodically to publish posts that are due
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishScheduledPostsJob {
    /// Optional site ID to limit scope (None = all sites)
    pub site_id: Option<Uuid>,
}

impl JobPayload for PublishScheduledPostsJob {
    fn job_type() -> &'static str {
        "publish_scheduled_posts"
    }

    fn queue() -> &'static str {
        "content"
    }

    fn max_attempts() -> u32 {
        3
    }

    fn timeout_secs() -> u64 {
        300 // 5 minutes
    }
}

/// Handler for publishing scheduled posts
pub struct PublishScheduledPostsHandler {
    pool: PgPool,
}

impl PublishScheduledPostsHandler {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JobHandler for PublishScheduledPostsHandler {
    type Payload = PublishScheduledPostsJob;

    async fn handle(&self, payload: Self::Payload) -> Result<()> {
        info!(site_id = ?payload.site_id, "Processing scheduled posts for publication");

        let now = chrono::Utc::now();

        // Find all posts that are scheduled and due for publication
        let query = if let Some(site_id) = payload.site_id {
            sqlx::query(
                r#"
                UPDATE posts
                SET status = 'published', published_at = $1, scheduled_at = NULL, updated_at = $1
                WHERE status = 'scheduled'
                  AND scheduled_at <= $1
                  AND site_id = $2
                RETURNING id
                "#,
            )
            .bind(now)
            .bind(site_id)
        } else {
            sqlx::query(
                r#"
                UPDATE posts
                SET status = 'published', published_at = $1, scheduled_at = NULL, updated_at = $1
                WHERE status = 'scheduled'
                  AND scheduled_at <= $1
                RETURNING id
                "#,
            )
            .bind(now)
        };

        let result = query.execute(&self.pool).await.map_err(|e| {
            rustpress_core::error::Error::database(format!("Failed to publish scheduled posts: {}", e))
        })?;

        let published_count = result.rows_affected();
        info!(published_count, "Published scheduled posts");

        Ok(())
    }

    async fn failed(&self, payload: Self::Payload, error: &str) -> Result<()> {
        error!(
            site_id = ?payload.site_id,
            error,
            "Failed to publish scheduled posts"
        );
        Ok(())
    }

    async fn completed(&self, payload: Self::Payload) -> Result<()> {
        info!(site_id = ?payload.site_id, "Completed scheduled posts publishing job");
        Ok(())
    }
}

/// Clean expired theme previews job - removes stale preview sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanThemePreviewsJob {
    /// Optional site ID to limit scope (None = all sites)
    pub site_id: Option<Uuid>,
}

impl JobPayload for CleanThemePreviewsJob {
    fn job_type() -> &'static str {
        "clean_theme_previews"
    }

    fn queue() -> &'static str {
        "maintenance"
    }

    fn max_attempts() -> u32 {
        3
    }

    fn timeout_secs() -> u64 {
        300 // 5 minutes
    }
}

/// Handler for cleaning expired theme previews
pub struct CleanThemePreviewsHandler {
    pool: PgPool,
}

impl CleanThemePreviewsHandler {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JobHandler for CleanThemePreviewsHandler {
    type Payload = CleanThemePreviewsJob;

    async fn handle(&self, payload: Self::Payload) -> Result<()> {
        info!(site_id = ?payload.site_id, "Cleaning expired theme previews");

        // Delete expired theme previews
        let query = if let Some(site_id) = payload.site_id {
            sqlx::query(
                r#"
                DELETE FROM theme_previews
                WHERE expires_at < NOW()
                  AND site_id = $1
                "#,
            )
            .bind(site_id)
        } else {
            sqlx::query(
                r#"
                DELETE FROM theme_previews
                WHERE expires_at < NOW()
                "#,
            )
        };

        let result = query.execute(&self.pool).await.map_err(|e| {
            rustpress_core::error::Error::database(format!("Failed to clean theme previews: {}", e))
        })?;

        let deleted_count = result.rows_affected();
        info!(deleted_count, "Cleaned expired theme previews");

        Ok(())
    }

    async fn failed(&self, payload: Self::Payload, error: &str) -> Result<()> {
        error!(
            site_id = ?payload.site_id,
            error,
            "Failed to clean theme previews"
        );
        Ok(())
    }

    async fn completed(&self, payload: Self::Payload) -> Result<()> {
        info!(site_id = ?payload.site_id, "Completed theme preview cleanup job");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_publish_scheduled_posts_job_type() {
        assert_eq!(PublishScheduledPostsJob::job_type(), "publish_scheduled_posts");
        assert_eq!(PublishScheduledPostsJob::queue(), "content");
    }

    #[test]
    fn test_clean_theme_previews_job_type() {
        assert_eq!(CleanThemePreviewsJob::job_type(), "clean_theme_previews");
        assert_eq!(CleanThemePreviewsJob::queue(), "maintenance");
    }
}
