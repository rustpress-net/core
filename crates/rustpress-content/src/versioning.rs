//! Content versioning and revision history
//!
//! Provides revision tracking, diff generation, and content history management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use uuid::Uuid;

use crate::{Content, ContentError, ContentResult};

/// Content revision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revision {
    /// Revision ID
    pub id: Uuid,

    /// Parent content ID
    pub content_id: Uuid,

    /// Revision number
    pub revision: i32,

    /// Title at this revision
    pub title: String,

    /// Content at this revision
    pub content: String,

    /// Blocks at this revision
    pub blocks: serde_json::Value,

    /// Author who made this revision
    pub author_id: Uuid,

    /// When this revision was created
    pub created_at: DateTime<Utc>,

    /// Summary of changes
    pub change_summary: Option<String>,
}

impl Revision {
    /// Create a new revision with defaults
    pub fn new(content_id: Uuid, revision: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            content_id,
            revision,
            title: String::new(),
            content: String::new(),
            blocks: serde_json::json!([]),
            author_id: Uuid::nil(),
            created_at: Utc::now(),
            change_summary: None,
        }
    }

    /// Create a new revision from content
    pub fn from_content(content: &Content) -> Self {
        Self {
            id: Uuid::new_v4(),
            content_id: content.id,
            revision: content.revision,
            title: content.title.clone(),
            content: content.content.clone(),
            blocks: serde_json::to_value(&content.blocks).unwrap_or(serde_json::json!([])),
            author_id: content.author_id,
            created_at: Utc::now(),
            change_summary: None,
        }
    }

    /// Builder: set title
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    /// Builder: set content
    pub fn with_content(mut self, content: &str) -> Self {
        self.content = content.to_string();
        self
    }

    /// Builder: set author
    pub fn with_author(mut self, author_id: Uuid) -> Self {
        self.author_id = author_id;
        self
    }
}

/// Diff between two revisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisionDiff {
    /// From revision number
    pub from_revision: i32,

    /// To revision number
    pub to_revision: i32,

    /// Title changed
    pub title_changed: bool,

    /// Title diff (if changed)
    pub title_diff: Option<String>,

    /// Content changes
    pub content_changes: Vec<DiffChange>,

    /// Statistics
    pub stats: DiffStats,
}

/// Individual change in a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffChange {
    /// Change type
    pub change_type: ChangeType,

    /// Line number in old version (for delete/equal)
    pub old_line: Option<usize>,

    /// Line number in new version (for insert/equal)
    pub new_line: Option<usize>,

    /// The content of this change
    pub content: String,
}

/// Type of change
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Insert,
    Delete,
    Equal,
}

/// Diff statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffStats {
    /// Lines added
    pub additions: usize,

    /// Lines removed
    pub deletions: usize,

    /// Lines unchanged
    pub unchanged: usize,

    /// Total changes
    pub total_changes: usize,
}

/// Versioning service
pub struct VersioningService {
    pool: sqlx::PgPool,
}

impl VersioningService {
    /// Create new versioning service
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// Create a new revision for content
    pub async fn create_revision(&self, content: &Content) -> ContentResult<Revision> {
        let revision = Revision::from_content(content);

        sqlx::query(
            r#"
            INSERT INTO content_revisions (
                id, content_id, revision, title, content, blocks,
                author_id, created_at, change_summary
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(revision.id)
        .bind(revision.content_id)
        .bind(revision.revision)
        .bind(&revision.title)
        .bind(&revision.content)
        .bind(&revision.blocks)
        .bind(revision.author_id)
        .bind(revision.created_at)
        .bind(&revision.change_summary)
        .execute(&self.pool)
        .await?;

        Ok(revision)
    }

    /// Get all revisions for content
    pub async fn get_revisions(&self, content_id: Uuid) -> ContentResult<Vec<Revision>> {
        let rows = sqlx::query_as::<_, RevisionRow>(
            "SELECT * FROM content_revisions WHERE content_id = $1 ORDER BY revision DESC",
        )
        .bind(content_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// Get a specific revision
    pub async fn get_revision(&self, content_id: Uuid, revision: i32) -> ContentResult<Revision> {
        let row = sqlx::query_as::<_, RevisionRow>(
            "SELECT * FROM content_revisions WHERE content_id = $1 AND revision = $2",
        )
        .bind(content_id)
        .bind(revision)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ContentError::NotFound(format!("Revision {} not found", revision)))?;

        Ok(row.into())
    }

    /// Get the latest revision
    pub async fn get_latest_revision(&self, content_id: Uuid) -> ContentResult<Revision> {
        let row = sqlx::query_as::<_, RevisionRow>(
            "SELECT * FROM content_revisions WHERE content_id = $1 ORDER BY revision DESC LIMIT 1",
        )
        .bind(content_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ContentError::NotFound("No revisions found".to_string()))?;

        Ok(row.into())
    }

    /// Compare two revisions
    pub async fn compare_revisions(
        &self,
        content_id: Uuid,
        from_revision: i32,
        to_revision: i32,
    ) -> ContentResult<RevisionDiff> {
        let from = self.get_revision(content_id, from_revision).await?;
        let to = self.get_revision(content_id, to_revision).await?;

        Ok(Self::generate_diff(&from, &to))
    }

    /// Generate diff between two revisions
    pub fn generate_diff(from: &Revision, to: &Revision) -> RevisionDiff {
        let title_changed = from.title != to.title;
        let title_diff = if title_changed {
            Some(format!("'{}' -> '{}'", from.title, to.title))
        } else {
            None
        };

        let content_diff = TextDiff::from_lines(&from.content, &to.content);
        let mut content_changes = Vec::new();
        let mut old_line = 1usize;
        let mut new_line = 1usize;
        let mut additions = 0usize;
        let mut deletions = 0usize;
        let mut unchanged = 0usize;

        for change in content_diff.iter_all_changes() {
            let (change_type, old_ln, new_ln) = match change.tag() {
                ChangeTag::Delete => {
                    deletions += 1;
                    let ln = old_line;
                    old_line += 1;
                    (ChangeType::Delete, Some(ln), None)
                }
                ChangeTag::Insert => {
                    additions += 1;
                    let ln = new_line;
                    new_line += 1;
                    (ChangeType::Insert, None, Some(ln))
                }
                ChangeTag::Equal => {
                    unchanged += 1;
                    let oln = old_line;
                    let nln = new_line;
                    old_line += 1;
                    new_line += 1;
                    (ChangeType::Equal, Some(oln), Some(nln))
                }
            };

            content_changes.push(DiffChange {
                change_type,
                old_line: old_ln,
                new_line: new_ln,
                content: change.value().to_string(),
            });
        }

        RevisionDiff {
            from_revision: from.revision,
            to_revision: to.revision,
            title_changed,
            title_diff,
            content_changes,
            stats: DiffStats {
                additions,
                deletions,
                unchanged,
                total_changes: additions + deletions,
            },
        }
    }

    /// Restore content to a specific revision
    pub async fn restore_revision(
        &self,
        content_id: Uuid,
        revision: i32,
    ) -> ContentResult<Revision> {
        let revision_data = self.get_revision(content_id, revision).await?;

        // Get current content to determine new revision number
        let current = self.get_latest_revision(content_id).await?;
        let new_revision = current.revision + 1;

        // Create new revision with restored content
        let restored = Revision {
            id: Uuid::new_v4(),
            content_id,
            revision: new_revision,
            title: revision_data.title,
            content: revision_data.content,
            blocks: revision_data.blocks,
            author_id: revision_data.author_id,
            created_at: Utc::now(),
            change_summary: Some(format!("Restored from revision {}", revision)),
        };

        // Update content in main table
        sqlx::query(
            r#"
            UPDATE contents SET
                title = $2, content = $3, blocks = $4, revision = $5, updated_at = $6
            WHERE id = $1
            "#,
        )
        .bind(content_id)
        .bind(&restored.title)
        .bind(&restored.content)
        .bind(&restored.blocks)
        .bind(new_revision)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;

        // Save the restored revision
        sqlx::query(
            r#"
            INSERT INTO content_revisions (
                id, content_id, revision, title, content, blocks,
                author_id, created_at, change_summary
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(restored.id)
        .bind(restored.content_id)
        .bind(restored.revision)
        .bind(&restored.title)
        .bind(&restored.content)
        .bind(&restored.blocks)
        .bind(restored.author_id)
        .bind(restored.created_at)
        .bind(&restored.change_summary)
        .execute(&self.pool)
        .await?;

        Ok(restored)
    }

    /// Delete old revisions (keep most recent N)
    pub async fn cleanup_revisions(&self, content_id: Uuid, keep_count: i32) -> ContentResult<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM content_revisions
            WHERE content_id = $1
            AND revision NOT IN (
                SELECT revision FROM content_revisions
                WHERE content_id = $1
                ORDER BY revision DESC
                LIMIT $2
            )
            "#,
        )
        .bind(content_id)
        .bind(keep_count)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Get revision count for content
    pub async fn count_revisions(&self, content_id: Uuid) -> ContentResult<i64> {
        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM content_revisions WHERE content_id = $1")
                .bind(content_id)
                .fetch_one(&self.pool)
                .await?;

        Ok(count.0)
    }
}

/// Database row representation
#[derive(Debug, sqlx::FromRow)]
struct RevisionRow {
    id: Uuid,
    content_id: Uuid,
    revision: i32,
    title: String,
    content: String,
    blocks: serde_json::Value,
    author_id: Uuid,
    created_at: DateTime<Utc>,
    change_summary: Option<String>,
}

impl From<RevisionRow> for Revision {
    fn from(row: RevisionRow) -> Self {
        Self {
            id: row.id,
            content_id: row.content_id,
            revision: row.revision,
            title: row.title,
            content: row.content,
            blocks: row.blocks,
            author_id: row.author_id,
            created_at: row.created_at,
            change_summary: row.change_summary,
        }
    }
}

/// Generate HTML diff view
pub fn generate_html_diff(diff: &RevisionDiff) -> String {
    let mut html = String::from(r#"<div class="diff-view">"#);

    // Stats
    html.push_str(&format!(
        r#"<div class="diff-stats">
            <span class="additions">+{}</span>
            <span class="deletions">-{}</span>
        </div>"#,
        diff.stats.additions, diff.stats.deletions
    ));

    // Title change
    if let Some(ref title_diff) = diff.title_diff {
        html.push_str(&format!(
            r#"<div class="title-change">Title: {}</div>"#,
            title_diff
        ));
    }

    // Content diff
    html.push_str(r#"<table class="diff-table"><tbody>"#);

    for change in &diff.content_changes {
        let class = match change.change_type {
            ChangeType::Insert => "diff-add",
            ChangeType::Delete => "diff-del",
            ChangeType::Equal => "diff-equal",
        };

        let old_ln = change.old_line.map(|n| n.to_string()).unwrap_or_default();
        let new_ln = change.new_line.map(|n| n.to_string()).unwrap_or_default();

        html.push_str(&format!(
            r#"<tr class="{}">
                <td class="line-num">{}</td>
                <td class="line-num">{}</td>
                <td class="line-content"><pre>{}</pre></td>
            </tr>"#,
            class,
            old_ln,
            new_ln,
            ammonia::clean(&change.content)
        ));
    }

    html.push_str("</tbody></table></div>");
    html
}

/// Generate unified diff format
pub fn generate_unified_diff(diff: &RevisionDiff, from_label: &str, to_label: &str) -> String {
    let mut output = String::new();

    output.push_str(&format!("--- {}\n", from_label));
    output.push_str(&format!("+++ {}\n", to_label));

    // Group changes into hunks
    let mut in_hunk = false;
    let mut hunk_old_start = 0usize;
    let mut hunk_new_start = 0usize;
    let mut hunk_old_count = 0usize;
    let mut hunk_new_count = 0usize;
    let mut hunk_content = String::new();

    for change in &diff.content_changes {
        match change.change_type {
            ChangeType::Equal => {
                if in_hunk {
                    // Include some context
                    hunk_old_count += 1;
                    hunk_new_count += 1;
                    hunk_content.push_str(&format!(" {}", change.content));
                }
            }
            ChangeType::Delete => {
                if !in_hunk {
                    in_hunk = true;
                    hunk_old_start = change.old_line.unwrap_or(1);
                    hunk_new_start = change.new_line.unwrap_or(hunk_old_start);
                }
                hunk_old_count += 1;
                hunk_content.push_str(&format!("-{}", change.content));
            }
            ChangeType::Insert => {
                if !in_hunk {
                    in_hunk = true;
                    hunk_old_start = change.old_line.unwrap_or(1);
                    hunk_new_start = change.new_line.unwrap_or(1);
                }
                hunk_new_count += 1;
                hunk_content.push_str(&format!("+{}", change.content));
            }
        }
    }

    // Output final hunk if any
    if in_hunk {
        output.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            hunk_old_start, hunk_old_count, hunk_new_start, hunk_new_count
        ));
        output.push_str(&hunk_content);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_diff() {
        let from = Revision {
            id: Uuid::new_v4(),
            content_id: Uuid::new_v4(),
            revision: 1,
            title: "Title".to_string(),
            content: "Line 1\nLine 2\nLine 3".to_string(),
            blocks: serde_json::json!([]),
            author_id: Uuid::new_v4(),
            created_at: Utc::now(),
            change_summary: None,
        };

        let to = Revision {
            id: Uuid::new_v4(),
            content_id: from.content_id,
            revision: 2,
            title: "Title".to_string(),
            content: "Line 1\nLine 2 modified\nLine 3\nLine 4".to_string(),
            blocks: serde_json::json!([]),
            author_id: Uuid::new_v4(),
            created_at: Utc::now(),
            change_summary: None,
        };

        let diff = VersioningService::generate_diff(&from, &to);

        assert!(!diff.title_changed);
        assert!(diff.stats.additions > 0 || diff.stats.deletions > 0);
    }

    #[test]
    fn test_title_change_detection() {
        let from = Revision {
            id: Uuid::new_v4(),
            content_id: Uuid::new_v4(),
            revision: 1,
            title: "Old Title".to_string(),
            content: "Content".to_string(),
            blocks: serde_json::json!([]),
            author_id: Uuid::new_v4(),
            created_at: Utc::now(),
            change_summary: None,
        };

        let to = Revision {
            id: Uuid::new_v4(),
            content_id: from.content_id,
            revision: 2,
            title: "New Title".to_string(),
            content: "Content".to_string(),
            blocks: serde_json::json!([]),
            author_id: Uuid::new_v4(),
            created_at: Utc::now(),
            change_summary: None,
        };

        let diff = VersioningService::generate_diff(&from, &to);

        assert!(diff.title_changed);
        assert!(diff.title_diff.is_some());
    }
}
