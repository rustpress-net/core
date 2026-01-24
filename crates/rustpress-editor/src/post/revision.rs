//! Post Revision System
//!
//! Version history and diff comparison for posts.

use crate::post::PostContent;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Post revision/version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostRevision {
    /// Revision ID
    pub id: i64,

    /// Version number
    pub version: u32,

    /// Author ID who made this revision
    pub author_id: i64,

    /// Title at this revision
    pub title: String,

    /// Content at this revision
    pub content: PostContent,

    /// Excerpt at this revision
    pub excerpt: Option<String>,

    /// Revision message/description
    pub message: Option<String>,

    /// Timestamp
    pub created_at: DateTime<Utc>,

    /// Revision type
    pub revision_type: RevisionType,

    /// Changes summary
    pub changes: Option<RevisionChanges>,
}

impl PostRevision {
    pub fn new(
        version: u32,
        author_id: i64,
        title: String,
        content: PostContent,
        excerpt: Option<String>,
        message: Option<String>,
    ) -> Self {
        Self {
            id: 0,
            version,
            author_id,
            title,
            content,
            excerpt,
            message,
            created_at: Utc::now(),
            revision_type: RevisionType::Manual,
            changes: None,
        }
    }

    /// Create an autosave revision
    pub fn autosave(
        version: u32,
        author_id: i64,
        title: String,
        content: PostContent,
        excerpt: Option<String>,
    ) -> Self {
        let mut rev = Self::new(version, author_id, title, content, excerpt, None);
        rev.revision_type = RevisionType::Autosave;
        rev
    }
}

/// Type of revision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevisionType {
    /// Manual save by user
    Manual,
    /// Automatic periodic save
    Autosave,
    /// Published version
    Publish,
    /// Pre-publish draft
    PrePublish,
    /// Imported content
    Import,
    /// Restored from revision
    Restore,
}

/// Summary of changes between revisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisionChanges {
    /// Title changed
    pub title_changed: bool,

    /// Content changed
    pub content_changed: bool,

    /// Excerpt changed
    pub excerpt_changed: bool,

    /// Number of blocks added
    pub blocks_added: u32,

    /// Number of blocks removed
    pub blocks_removed: u32,

    /// Number of blocks modified
    pub blocks_modified: u32,

    /// Word count change
    pub word_count_diff: i32,

    /// Character count change
    pub char_count_diff: i32,
}

impl RevisionChanges {
    /// Calculate changes between two revisions
    pub fn calculate(old: &PostRevision, new: &PostRevision) -> Self {
        let title_changed = old.title != new.title;
        let excerpt_changed = old.excerpt != new.excerpt;

        let old_block_ids: std::collections::HashSet<_> =
            old.content.get_all_block_ids().into_iter().collect();
        let new_block_ids: std::collections::HashSet<_> =
            new.content.get_all_block_ids().into_iter().collect();

        let blocks_added = new_block_ids.difference(&old_block_ids).count() as u32;
        let blocks_removed = old_block_ids.difference(&new_block_ids).count() as u32;

        let old_text = old.content.get_plain_text();
        let new_text = new.content.get_plain_text();

        let old_word_count = old_text.split_whitespace().count() as i32;
        let new_word_count = new_text.split_whitespace().count() as i32;

        Self {
            title_changed,
            content_changed: blocks_added > 0 || blocks_removed > 0 || old_text != new_text,
            excerpt_changed,
            blocks_added,
            blocks_removed,
            blocks_modified: 0, // Would need deep comparison
            word_count_diff: new_word_count - old_word_count,
            char_count_diff: (new_text.len() as i32) - (old_text.len() as i32),
        }
    }
}

/// Diff between two text versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDiff {
    /// Diff hunks
    pub hunks: Vec<DiffHunk>,

    /// Statistics
    pub stats: DiffStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    /// Old content range start
    pub old_start: usize,
    /// Old content range length
    pub old_length: usize,
    /// New content range start
    pub new_start: usize,
    /// New content range length
    pub new_length: usize,
    /// Changes in this hunk
    pub changes: Vec<DiffChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffChange {
    /// Change type
    pub change_type: DiffChangeType,
    /// Content
    pub content: String,
    /// Line number in old version (if applicable)
    pub old_line: Option<usize>,
    /// Line number in new version (if applicable)
    pub new_line: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiffChangeType {
    /// Unchanged line
    Equal,
    /// Inserted line
    Insert,
    /// Deleted line
    Delete,
    /// Modified line (contains both old and new)
    Modify,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffStats {
    /// Lines added
    pub additions: u32,
    /// Lines removed
    pub deletions: u32,
    /// Lines unchanged
    pub unchanged: u32,
    /// Total changes
    pub total_changes: u32,
}

/// Compare two texts and generate diff
pub fn compute_diff(old_text: &str, new_text: &str) -> TextDiff {
    use similar::{ChangeTag, TextDiff as SimilarDiff};

    let diff = SimilarDiff::from_lines(old_text, new_text);
    let mut hunks = Vec::new();
    let mut current_hunk: Option<DiffHunk> = None;
    let mut stats = DiffStats::default();

    let mut old_line = 0usize;
    let mut new_line = 0usize;

    for change in diff.iter_all_changes() {
        let change_type = match change.tag() {
            ChangeTag::Equal => {
                stats.unchanged += 1;
                old_line += 1;
                new_line += 1;
                DiffChangeType::Equal
            }
            ChangeTag::Insert => {
                stats.additions += 1;
                stats.total_changes += 1;
                new_line += 1;
                DiffChangeType::Insert
            }
            ChangeTag::Delete => {
                stats.deletions += 1;
                stats.total_changes += 1;
                old_line += 1;
                DiffChangeType::Delete
            }
        };

        // Only include non-equal changes in hunks for efficiency
        if change_type != DiffChangeType::Equal {
            let diff_change = DiffChange {
                change_type,
                content: change.value().to_string(),
                old_line: if change_type == DiffChangeType::Delete {
                    Some(old_line)
                } else {
                    None
                },
                new_line: if change_type == DiffChangeType::Insert {
                    Some(new_line)
                } else {
                    None
                },
            };

            if let Some(ref mut hunk) = current_hunk {
                hunk.changes.push(diff_change);
            } else {
                current_hunk = Some(DiffHunk {
                    old_start: old_line.saturating_sub(1),
                    old_length: 0,
                    new_start: new_line.saturating_sub(1),
                    new_length: 0,
                    changes: vec![diff_change],
                });
            }
        } else if let Some(hunk) = current_hunk.take() {
            // End of changes, save hunk
            hunks.push(hunk);
        }
    }

    // Don't forget the last hunk
    if let Some(hunk) = current_hunk {
        hunks.push(hunk);
    }

    TextDiff { hunks, stats }
}

/// Revision comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisionComparison {
    /// Old revision
    pub old_revision: RevisionSummary,
    /// New revision
    pub new_revision: RevisionSummary,
    /// Title diff (if changed)
    pub title_diff: Option<TextDiff>,
    /// Content diff
    pub content_diff: TextDiff,
    /// Excerpt diff (if changed)
    pub excerpt_diff: Option<TextDiff>,
    /// Changes summary
    pub changes: RevisionChanges,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisionSummary {
    pub id: i64,
    pub version: u32,
    pub author_id: i64,
    pub created_at: DateTime<Utc>,
    pub word_count: u32,
}

/// Compare two revisions
pub fn compare_revisions(old: &PostRevision, new: &PostRevision) -> RevisionComparison {
    let old_text = old.content.get_plain_text();
    let new_text = new.content.get_plain_text();

    let title_diff = if old.title != new.title {
        Some(compute_diff(&old.title, &new.title))
    } else {
        None
    };

    let excerpt_diff = match (&old.excerpt, &new.excerpt) {
        (Some(old_e), Some(new_e)) if old_e != new_e => Some(compute_diff(old_e, new_e)),
        (None, Some(new_e)) => Some(compute_diff("", new_e)),
        (Some(old_e), None) => Some(compute_diff(old_e, "")),
        _ => None,
    };

    RevisionComparison {
        old_revision: RevisionSummary {
            id: old.id,
            version: old.version,
            author_id: old.author_id,
            created_at: old.created_at,
            word_count: old_text.split_whitespace().count() as u32,
        },
        new_revision: RevisionSummary {
            id: new.id,
            version: new.version,
            author_id: new.author_id,
            created_at: new.created_at,
            word_count: new_text.split_whitespace().count() as u32,
        },
        title_diff,
        content_diff: compute_diff(&old_text, &new_text),
        excerpt_diff,
        changes: RevisionChanges::calculate(old, new),
    }
}
