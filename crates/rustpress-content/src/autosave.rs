//! Draft autosave functionality
//!
//! Provides automatic saving of content drafts to prevent data loss
//! during editing sessions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Block, ContentError, ContentResult};

/// Autosave entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Autosave {
    /// Autosave ID
    pub id: Uuid,

    /// Content ID (None for new unsaved content)
    pub content_id: Option<Uuid>,

    /// User who created the autosave
    pub user_id: Uuid,

    /// Title at time of autosave
    pub title: String,

    /// Content at time of autosave
    pub content: String,

    /// Blocks at time of autosave
    pub blocks: Vec<Block>,

    /// When autosave was created
    pub created_at: DateTime<Utc>,
}

/// Autosave service
pub struct AutosaveService {
    pool: sqlx::PgPool,
}

impl AutosaveService {
    /// Create new autosave service
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// Save autosave for content
    pub async fn save(
        &self,
        content_id: Option<Uuid>,
        user_id: Uuid,
        title: &str,
        content: &str,
        blocks: &[Block],
    ) -> ContentResult<Autosave> {
        let autosave = Autosave {
            id: Uuid::new_v4(),
            content_id,
            user_id,
            title: title.to_string(),
            content: content.to_string(),
            blocks: blocks.to_vec(),
            created_at: Utc::now(),
        };

        // Upsert - replace existing autosave for same content/user
        sqlx::query(
            r#"
            INSERT INTO content_autosaves (id, content_id, user_id, title, content, blocks, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (content_id, user_id)
            DO UPDATE SET
                id = $1,
                title = $4,
                content = $5,
                blocks = $6,
                created_at = $7
            "#,
        )
        .bind(autosave.id)
        .bind(autosave.content_id)
        .bind(autosave.user_id)
        .bind(&autosave.title)
        .bind(&autosave.content)
        .bind(serde_json::to_value(&autosave.blocks)?)
        .bind(autosave.created_at)
        .execute(&self.pool)
        .await?;

        Ok(autosave)
    }

    /// Get autosave for content and user
    pub async fn get(&self, content_id: Uuid, user_id: Uuid) -> ContentResult<Option<Autosave>> {
        let row = sqlx::query_as::<_, AutosaveRow>(
            "SELECT * FROM content_autosaves WHERE content_id = $1 AND user_id = $2",
        )
        .bind(content_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    /// Get all autosaves for a user (including new unsaved content)
    pub async fn get_user_autosaves(&self, user_id: Uuid) -> ContentResult<Vec<Autosave>> {
        let rows = sqlx::query_as::<_, AutosaveRow>(
            "SELECT * FROM content_autosaves WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|r| r.try_into())
            .collect()
    }

    /// Check if autosave exists and is newer than content
    pub async fn has_newer_autosave(
        &self,
        content_id: Uuid,
        user_id: Uuid,
        content_updated_at: DateTime<Utc>,
    ) -> ContentResult<bool> {
        let result: Option<(DateTime<Utc>,)> = sqlx::query_as(
            "SELECT created_at FROM content_autosaves WHERE content_id = $1 AND user_id = $2",
        )
        .bind(content_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result
            .map(|(autosave_time,)| autosave_time > content_updated_at)
            .unwrap_or(false))
    }

    /// Delete autosave
    pub async fn delete(&self, content_id: Uuid, user_id: Uuid) -> ContentResult<()> {
        sqlx::query("DELETE FROM content_autosaves WHERE content_id = $1 AND user_id = $2")
            .bind(content_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Delete all autosaves for content (after publish/save)
    pub async fn delete_for_content(&self, content_id: Uuid) -> ContentResult<()> {
        sqlx::query("DELETE FROM content_autosaves WHERE content_id = $1")
            .bind(content_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Clean up old autosaves (older than specified hours)
    pub async fn cleanup_old(&self, hours: i64) -> ContentResult<u64> {
        let cutoff = Utc::now() - chrono::Duration::hours(hours);

        let result = sqlx::query("DELETE FROM content_autosaves WHERE created_at < $1")
            .bind(cutoff)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    /// Get autosave statistics
    pub async fn get_stats(&self) -> ContentResult<AutosaveStats> {
        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM content_autosaves")
            .fetch_one(&self.pool)
            .await?;

        let unique_users: (i64,) =
            sqlx::query_as("SELECT COUNT(DISTINCT user_id) FROM content_autosaves")
                .fetch_one(&self.pool)
                .await?;

        let oldest: Option<(DateTime<Utc>,)> =
            sqlx::query_as("SELECT MIN(created_at) FROM content_autosaves")
                .fetch_optional(&self.pool)
                .await?;

        let newest: Option<(DateTime<Utc>,)> =
            sqlx::query_as("SELECT MAX(created_at) FROM content_autosaves")
                .fetch_optional(&self.pool)
                .await?;

        Ok(AutosaveStats {
            total_autosaves: total.0 as usize,
            unique_users: unique_users.0 as usize,
            oldest_autosave: oldest.map(|(t,)| t),
            newest_autosave: newest.map(|(t,)| t),
        })
    }
}

/// Autosave statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutosaveStats {
    pub total_autosaves: usize,
    pub unique_users: usize,
    pub oldest_autosave: Option<DateTime<Utc>>,
    pub newest_autosave: Option<DateTime<Utc>>,
}

/// Autosave request for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutosaveRequest {
    pub content_id: Option<Uuid>,
    pub title: String,
    pub content: String,
    pub blocks: Vec<Block>,
}

/// Autosave response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutosaveResponse {
    pub id: Uuid,
    pub saved_at: DateTime<Utc>,
    pub has_changes: bool,
}

/// Database row
#[derive(Debug, sqlx::FromRow)]
struct AutosaveRow {
    id: Uuid,
    content_id: Option<Uuid>,
    user_id: Uuid,
    title: String,
    content: String,
    blocks: serde_json::Value,
    created_at: DateTime<Utc>,
}

impl TryFrom<AutosaveRow> for Autosave {
    type Error = ContentError;

    fn try_from(row: AutosaveRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            content_id: row.content_id,
            user_id: row.user_id,
            title: row.title,
            content: row.content,
            blocks: serde_json::from_value(row.blocks)?,
            created_at: row.created_at,
        })
    }
}

/// Autosave manager for client-side coordination
#[derive(Debug, Clone)]
pub struct AutosaveConfig {
    /// Autosave interval in seconds
    pub interval_seconds: u64,

    /// Minimum characters to trigger autosave
    pub min_content_length: usize,

    /// Enable autosave
    pub enabled: bool,

    /// Show autosave notifications
    pub show_notifications: bool,
}

impl Default for AutosaveConfig {
    fn default() -> Self {
        Self {
            interval_seconds: 60,
            min_content_length: 10,
            enabled: true,
            show_notifications: true,
        }
    }
}

/// Generate client-side autosave JavaScript
pub fn generate_autosave_script(config: &AutosaveConfig) -> String {
    format!(
        r#"
(function() {{
    const AUTOSAVE_INTERVAL = {};
    const MIN_CONTENT_LENGTH = {};
    const ENABLED = {};
    const SHOW_NOTIFICATIONS = {};

    if (!ENABLED) return;

    let lastSavedContent = '';
    let autosaveTimer = null;

    function getEditorContent() {{
        // Get content from editor (customize based on editor type)
        const titleEl = document.querySelector('#editor-title');
        const contentEl = document.querySelector('#editor-content');

        return {{
            title: titleEl ? titleEl.value : '',
            content: contentEl ? contentEl.value : ''
        }};
    }}

    function hasChanges() {{
        const current = JSON.stringify(getEditorContent());
        return current !== lastSavedContent && current.length >= MIN_CONTENT_LENGTH;
    }}

    async function saveAutosave() {{
        if (!hasChanges()) return;

        const {{ title, content }} = getEditorContent();
        const contentId = document.querySelector('#content-id')?.value;

        try {{
            const response = await fetch('/api/autosave', {{
                method: 'POST',
                headers: {{ 'Content-Type': 'application/json' }},
                body: JSON.stringify({{
                    content_id: contentId || null,
                    title,
                    content,
                    blocks: []
                }})
            }});

            if (response.ok) {{
                lastSavedContent = JSON.stringify({{ title, content }});
                if (SHOW_NOTIFICATIONS) {{
                    showNotification('Draft saved');
                }}
            }}
        }} catch (err) {{
            console.error('Autosave failed:', err);
        }}
    }}

    function showNotification(message) {{
        const notification = document.createElement('div');
        notification.className = 'autosave-notification';
        notification.textContent = message;
        document.body.appendChild(notification);
        setTimeout(() => notification.remove(), 2000);
    }}

    function startAutosave() {{
        if (autosaveTimer) clearInterval(autosaveTimer);
        autosaveTimer = setInterval(saveAutosave, AUTOSAVE_INTERVAL * 1000);
    }}

    function stopAutosave() {{
        if (autosaveTimer) {{
            clearInterval(autosaveTimer);
            autosaveTimer = null;
        }}
    }}

    // Start autosave
    document.addEventListener('DOMContentLoaded', startAutosave);

    // Save before leaving page
    window.addEventListener('beforeunload', (e) => {{
        if (hasChanges()) {{
            saveAutosave();
            e.preventDefault();
            e.returnValue = '';
        }}
    }});

    // Expose functions globally
    window.RustPressAutosave = {{
        save: saveAutosave,
        start: startAutosave,
        stop: stopAutosave,
        hasChanges
    }};
}})();
"#,
        config.interval_seconds,
        config.min_content_length,
        config.enabled,
        config.show_notifications
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autosave_config_default() {
        let config = AutosaveConfig::default();
        assert_eq!(config.interval_seconds, 60);
        assert!(config.enabled);
    }

    #[test]
    fn test_generate_script() {
        let config = AutosaveConfig::default();
        let script = generate_autosave_script(&config);

        assert!(script.contains("AUTOSAVE_INTERVAL"));
        assert!(script.contains("saveAutosave"));
    }
}
