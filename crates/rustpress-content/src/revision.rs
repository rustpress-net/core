//! # Revision System
//!
//! Content revision tracking, comparison, and restoration.
//!
//! Features:
//! - Revision history storage
//! - Side-by-side diff comparison
//! - Inline diff visualization
//! - Revision restoration
//! - Autosave integration
//! - Conflict detection

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff, Algorithm};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

/// Revision system errors
#[derive(Debug, Error)]
pub enum RevisionError {
    #[error("Revision not found: {0}")]
    NotFound(String),

    #[error("Invalid revision: {0}")]
    InvalidRevision(String),

    #[error("Conflict detected: {0}")]
    ConflictDetected(String),

    #[error("Max revisions exceeded")]
    MaxRevisionsExceeded,

    #[error("Storage error: {0}")]
    StorageError(String),
}

/// A content revision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revision {
    /// Unique revision ID
    pub id: Uuid,

    /// Parent post ID
    pub post_id: i64,

    /// Revision number (sequential)
    pub revision_number: u32,

    /// Revision type
    pub revision_type: RevisionType,

    /// Post title at this revision
    pub title: String,

    /// Post content at this revision
    pub content: String,

    /// Post excerpt at this revision
    pub excerpt: String,

    /// Serialized post meta at this revision
    pub meta: HashMap<String, serde_json::Value>,

    /// Author who created this revision
    pub author_id: i64,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Whether this is an autosave
    pub is_autosave: bool,

    /// Preview token (for preview links)
    pub preview_token: Option<String>,

    /// Content hash for conflict detection
    pub content_hash: String,

    /// Parent revision ID (if any)
    pub parent_revision_id: Option<Uuid>,
}

impl Revision {
    pub fn new(post_id: i64, author_id: i64) -> Self {
        Self {
            id: Uuid::new_v4(),
            post_id,
            revision_number: 0,
            revision_type: RevisionType::Manual,
            title: String::new(),
            content: String::new(),
            excerpt: String::new(),
            meta: HashMap::new(),
            author_id,
            created_at: Utc::now(),
            is_autosave: false,
            preview_token: None,
            content_hash: String::new(),
            parent_revision_id: None,
        }
    }

    /// Create from post data
    pub fn from_post(
        post_id: i64,
        author_id: i64,
        title: &str,
        content: &str,
        excerpt: &str,
    ) -> Self {
        let mut revision = Self::new(post_id, author_id);
        revision.title = title.to_string();
        revision.content = content.to_string();
        revision.excerpt = excerpt.to_string();
        revision.content_hash = Self::compute_hash(content);
        revision
    }

    /// Mark as autosave
    pub fn as_autosave(mut self) -> Self {
        self.is_autosave = true;
        self.revision_type = RevisionType::Autosave;
        self
    }

    /// Set revision type
    pub fn with_type(mut self, revision_type: RevisionType) -> Self {
        self.revision_type = revision_type;
        self
    }

    /// Generate preview token
    pub fn with_preview(mut self) -> Self {
        self.preview_token = Some(Uuid::new_v4().to_string());
        self
    }

    /// Compute content hash
    fn compute_hash(content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Check if content matches this revision
    pub fn matches_content(&self, content: &str) -> bool {
        self.content_hash == Self::compute_hash(content)
    }
}

/// Types of revisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RevisionType {
    /// Manual save by user
    Manual,

    /// Automatic save (periodic)
    Autosave,

    /// Published version
    Publish,

    /// Scheduled version
    Scheduled,

    /// Initial creation
    Initial,

    /// Restored from previous revision
    Restored,

    /// Imported content
    Import,
}

impl RevisionType {
    pub fn label(&self) -> &str {
        match self {
            Self::Manual => "Manual Save",
            Self::Autosave => "Autosave",
            Self::Publish => "Published",
            Self::Scheduled => "Scheduled",
            Self::Initial => "Initial Version",
            Self::Restored => "Restored",
            Self::Import => "Imported",
        }
    }
}

/// Diff output formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffFormat {
    /// Side-by-side comparison
    SideBySide,

    /// Inline with changes marked
    Inline,

    /// Unified diff format
    Unified,

    /// HTML with highlighting
    Html,

    /// Plain text
    Plain,
}

/// A single change in a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffChange {
    /// Type of change
    pub change_type: ChangeType,

    /// The text content
    pub text: String,

    /// Line number in old version (if applicable)
    pub old_line: Option<usize>,

    /// Line number in new version (if applicable)
    pub new_line: Option<usize>,
}

/// Type of change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// No change (context)
    Equal,

    /// Added content
    Insert,

    /// Removed content
    Delete,

    /// Modified content
    Replace,
}

impl From<ChangeTag> for ChangeType {
    fn from(tag: ChangeTag) -> Self {
        match tag {
            ChangeTag::Equal => Self::Equal,
            ChangeTag::Insert => Self::Insert,
            ChangeTag::Delete => Self::Delete,
        }
    }
}

/// Result of comparing two revisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    /// Old revision ID
    pub old_revision_id: Uuid,

    /// New revision ID
    pub new_revision_id: Uuid,

    /// Title changes
    pub title_diff: Vec<DiffChange>,

    /// Content changes
    pub content_diff: Vec<DiffChange>,

    /// Excerpt changes
    pub excerpt_diff: Vec<DiffChange>,

    /// Statistics
    pub stats: DiffStats,

    /// HTML representation (if requested)
    pub html: Option<String>,
}

/// Diff statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffStats {
    /// Number of lines added
    pub lines_added: usize,

    /// Number of lines removed
    pub lines_removed: usize,

    /// Number of lines unchanged
    pub lines_unchanged: usize,

    /// Number of words added
    pub words_added: usize,

    /// Number of words removed
    pub words_removed: usize,

    /// Character count difference
    pub char_diff: i64,
}

/// Revision comparison engine
pub struct RevisionDiff {
    /// Diff algorithm to use
    algorithm: Algorithm,

    /// Context lines for unified diff
    context_lines: usize,
}

impl Default for RevisionDiff {
    fn default() -> Self {
        Self {
            algorithm: Algorithm::Myers,
            context_lines: 3,
        }
    }
}

impl RevisionDiff {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_algorithm(mut self, algorithm: Algorithm) -> Self {
        self.algorithm = algorithm;
        self
    }

    pub fn with_context(mut self, lines: usize) -> Self {
        self.context_lines = lines;
        self
    }

    /// Compare two revisions
    pub fn compare(&self, old: &Revision, new: &Revision) -> DiffResult {
        let title_diff = self.diff_text(&old.title, &new.title);
        let content_diff = self.diff_text(&old.content, &new.content);
        let excerpt_diff = self.diff_text(&old.excerpt, &new.excerpt);

        let mut stats = DiffStats::default();
        Self::calculate_stats(&content_diff, &mut stats);

        DiffResult {
            old_revision_id: old.id,
            new_revision_id: new.id,
            title_diff,
            content_diff,
            excerpt_diff,
            stats,
            html: None,
        }
    }

    /// Compare two text strings
    pub fn diff_text(&self, old: &str, new: &str) -> Vec<DiffChange> {
        let diff = TextDiff::configure()
            .algorithm(self.algorithm)
            .diff_lines(old, new);

        let mut changes = Vec::new();
        let mut old_line = 1usize;
        let mut new_line = 1usize;

        for change in diff.iter_all_changes() {
            let change_type = ChangeType::from(change.tag());

            changes.push(DiffChange {
                change_type,
                text: change.value().to_string(),
                old_line: if change.tag() != ChangeTag::Insert {
                    let line = old_line;
                    old_line += 1;
                    Some(line)
                } else {
                    None
                },
                new_line: if change.tag() != ChangeTag::Delete {
                    let line = new_line;
                    new_line += 1;
                    Some(line)
                } else {
                    None
                },
            });
        }

        changes
    }

    /// Calculate diff statistics
    fn calculate_stats(changes: &[DiffChange], stats: &mut DiffStats) {
        for change in changes {
            match change.change_type {
                ChangeType::Equal => {
                    stats.lines_unchanged += 1;
                }
                ChangeType::Insert => {
                    stats.lines_added += 1;
                    stats.words_added += change.text.split_whitespace().count();
                    stats.char_diff += change.text.len() as i64;
                }
                ChangeType::Delete => {
                    stats.lines_removed += 1;
                    stats.words_removed += change.text.split_whitespace().count();
                    stats.char_diff -= change.text.len() as i64;
                }
                ChangeType::Replace => {
                    stats.lines_added += 1;
                    stats.lines_removed += 1;
                }
            }
        }
    }

    /// Generate HTML diff output
    pub fn to_html(&self, diff: &DiffResult, format: DiffFormat) -> String {
        match format {
            DiffFormat::SideBySide => self.to_side_by_side_html(&diff.content_diff),
            DiffFormat::Inline => self.to_inline_html(&diff.content_diff),
            DiffFormat::Html => self.to_inline_html(&diff.content_diff),
            _ => self.to_plain(&diff.content_diff),
        }
    }

    /// Generate side-by-side HTML
    fn to_side_by_side_html(&self, changes: &[DiffChange]) -> String {
        let mut html = String::from(r#"<div class="diff-side-by-side"><div class="diff-left">"#);
        let mut right = String::from(r#"</div><div class="diff-right">"#);

        for change in changes {
            let escaped = Self::escape_html(&change.text);

            match change.change_type {
                ChangeType::Equal => {
                    html.push_str(&format!(
                        r#"<div class="diff-line diff-equal"><span class="line-num">{}</span><span class="line-content">{}</span></div>"#,
                        change.old_line.unwrap_or(0),
                        escaped
                    ));
                    right.push_str(&format!(
                        r#"<div class="diff-line diff-equal"><span class="line-num">{}</span><span class="line-content">{}</span></div>"#,
                        change.new_line.unwrap_or(0),
                        escaped
                    ));
                }
                ChangeType::Delete => {
                    html.push_str(&format!(
                        r#"<div class="diff-line diff-delete"><span class="line-num">{}</span><span class="line-content">{}</span></div>"#,
                        change.old_line.unwrap_or(0),
                        escaped
                    ));
                    right.push_str(r#"<div class="diff-line diff-empty"></div>"#);
                }
                ChangeType::Insert => {
                    html.push_str(r#"<div class="diff-line diff-empty"></div>"#);
                    right.push_str(&format!(
                        r#"<div class="diff-line diff-insert"><span class="line-num">{}</span><span class="line-content">{}</span></div>"#,
                        change.new_line.unwrap_or(0),
                        escaped
                    ));
                }
                ChangeType::Replace => {
                    html.push_str(&format!(
                        r#"<div class="diff-line diff-delete"><span class="line-num">{}</span><span class="line-content">{}</span></div>"#,
                        change.old_line.unwrap_or(0),
                        escaped
                    ));
                    right.push_str(&format!(
                        r#"<div class="diff-line diff-insert"><span class="line-num">{}</span><span class="line-content">{}</span></div>"#,
                        change.new_line.unwrap_or(0),
                        escaped
                    ));
                }
            }
        }

        html.push_str(&right);
        html.push_str("</div></div>");
        html
    }

    /// Generate inline HTML diff
    fn to_inline_html(&self, changes: &[DiffChange]) -> String {
        let mut html = String::from(r#"<div class="diff-inline">"#);

        for change in changes {
            let escaped = Self::escape_html(&change.text);

            let (class, prefix) = match change.change_type {
                ChangeType::Equal => ("diff-equal", " "),
                ChangeType::Insert => ("diff-insert", "+"),
                ChangeType::Delete => ("diff-delete", "-"),
                ChangeType::Replace => ("diff-replace", "~"),
            };

            let line_nums = match (change.old_line, change.new_line) {
                (Some(old), Some(new)) => format!("{}/{}", old, new),
                (Some(old), None) => format!("{}", old),
                (None, Some(new)) => format!("{}", new),
                (None, None) => String::new(),
            };

            html.push_str(&format!(
                r#"<div class="diff-line {}"><span class="line-num">{}</span><span class="line-prefix">{}</span><span class="line-content">{}</span></div>"#,
                class, line_nums, prefix, escaped
            ));
        }

        html.push_str("</div>");
        html
    }

    /// Generate plain text diff
    fn to_plain(&self, changes: &[DiffChange]) -> String {
        let mut output = String::new();

        for change in changes {
            let prefix = match change.change_type {
                ChangeType::Equal => " ",
                ChangeType::Insert => "+",
                ChangeType::Delete => "-",
                ChangeType::Replace => "~",
            };
            output.push_str(&format!("{}{}", prefix, change.text));
        }

        output
    }

    /// Escape HTML special characters
    fn escape_html(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    }

    /// Word-level diff for fine-grained comparison
    pub fn diff_words(&self, old: &str, new: &str) -> Vec<DiffChange> {
        let diff = TextDiff::configure()
            .algorithm(self.algorithm)
            .diff_words(old, new);

        diff.iter_all_changes()
            .map(|change| DiffChange {
                change_type: ChangeType::from(change.tag()),
                text: change.value().to_string(),
                old_line: None,
                new_line: None,
            })
            .collect()
    }

    /// Character-level diff for detailed comparison
    pub fn diff_chars(&self, old: &str, new: &str) -> Vec<DiffChange> {
        let diff = TextDiff::configure()
            .algorithm(self.algorithm)
            .diff_chars(old, new);

        diff.iter_all_changes()
            .map(|change| DiffChange {
                change_type: ChangeType::from(change.tag()),
                text: change.value().to_string(),
                old_line: None,
                new_line: None,
            })
            .collect()
    }
}

/// Revision manager for storing and retrieving revisions
pub struct RevisionManager {
    /// Maximum revisions to keep per post
    max_revisions: usize,

    /// Whether to keep all publish revisions
    keep_all_publishes: bool,

    /// Diff engine
    diff: RevisionDiff,
}

impl Default for RevisionManager {
    fn default() -> Self {
        Self {
            max_revisions: 25,
            keep_all_publishes: true,
            diff: RevisionDiff::new(),
        }
    }
}

impl RevisionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_revisions(mut self, max: usize) -> Self {
        self.max_revisions = max;
        self
    }

    /// Create a new revision for a post
    pub fn create_revision(
        &self,
        post_id: i64,
        author_id: i64,
        title: &str,
        content: &str,
        excerpt: &str,
        revision_type: RevisionType,
    ) -> Revision {
        Revision::from_post(post_id, author_id, title, content, excerpt)
            .with_type(revision_type)
    }

    /// Compare two revisions
    pub fn compare(&self, old: &Revision, new: &Revision) -> DiffResult {
        self.diff.compare(old, new)
    }

    /// Get HTML diff between revisions
    pub fn compare_html(&self, old: &Revision, new: &Revision, format: DiffFormat) -> String {
        let diff = self.compare(old, new);
        self.diff.to_html(&diff, format)
    }

    /// Check if two revisions have conflicts
    pub fn has_conflict(&self, base: &Revision, current_hash: &str) -> bool {
        base.content_hash != current_hash
    }

    /// Merge changes (three-way merge)
    pub fn three_way_merge(
        &self,
        base: &str,
        ours: &str,
        theirs: &str,
    ) -> Result<String, RevisionError> {
        // Simple three-way merge using line-by-line comparison
        let base_lines: Vec<&str> = base.lines().collect();
        let our_lines: Vec<&str> = ours.lines().collect();
        let their_lines: Vec<&str> = theirs.lines().collect();

        let mut result = Vec::new();
        let max_len = base_lines.len().max(our_lines.len()).max(their_lines.len());

        for i in 0..max_len {
            let base_line = base_lines.get(i).copied().unwrap_or("");
            let our_line = our_lines.get(i).copied().unwrap_or("");
            let their_line = their_lines.get(i).copied().unwrap_or("");

            if our_line == their_line {
                // Both made same change or no change
                result.push(our_line.to_string());
            } else if our_line == base_line {
                // We didn't change, use theirs
                result.push(their_line.to_string());
            } else if their_line == base_line {
                // They didn't change, use ours
                result.push(our_line.to_string());
            } else {
                // Conflict - both changed differently
                return Err(RevisionError::ConflictDetected(format!(
                    "Conflict at line {}: '{}' vs '{}'",
                    i + 1, our_line, their_line
                )));
            }
        }

        Ok(result.join("\n"))
    }

    /// Clean up old revisions
    pub fn cleanup_revisions(&self, revisions: &mut Vec<Revision>) {
        if revisions.len() <= self.max_revisions {
            return;
        }

        // Sort by creation date (newest first)
        revisions.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // Keep publishes if configured
        let mut to_keep: Vec<&Revision> = if self.keep_all_publishes {
            revisions.iter()
                .filter(|r| r.revision_type == RevisionType::Publish)
                .collect()
        } else {
            Vec::new()
        };

        // Add remaining up to limit
        for revision in revisions.iter() {
            if to_keep.len() >= self.max_revisions {
                break;
            }
            if !to_keep.contains(&revision) {
                to_keep.push(revision);
            }
        }

        let keep_ids: Vec<Uuid> = to_keep.iter().map(|r| r.id).collect();
        revisions.retain(|r| keep_ids.contains(&r.id));
    }

    /// Get diff engine
    pub fn diff(&self) -> &RevisionDiff {
        &self.diff
    }
}

/// Autosave configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutosaveConfig {
    /// Enable autosave
    pub enabled: bool,

    /// Autosave interval in seconds
    pub interval_seconds: u32,

    /// Maximum autosaves to keep
    pub max_autosaves: usize,

    /// Minimum content change to trigger autosave (characters)
    pub min_change_threshold: usize,
}

impl Default for AutosaveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: 60,
            max_autosaves: 5,
            min_change_threshold: 10,
        }
    }
}

/// Autosave manager
pub struct AutosaveManager {
    config: AutosaveConfig,
}

impl Default for AutosaveManager {
    fn default() -> Self {
        Self {
            config: AutosaveConfig::default(),
        }
    }
}

impl AutosaveManager {
    pub fn new(config: AutosaveConfig) -> Self {
        Self { config }
    }

    /// Check if autosave should be triggered
    pub fn should_autosave(&self, last_content: &str, current_content: &str, elapsed_seconds: u64) -> bool {
        if !self.config.enabled {
            return false;
        }

        if elapsed_seconds < self.config.interval_seconds as u64 {
            return false;
        }

        let change_size = if current_content.len() > last_content.len() {
            current_content.len() - last_content.len()
        } else {
            last_content.len() - current_content.len()
        };

        change_size >= self.config.min_change_threshold
    }

    /// Create autosave revision
    pub fn create_autosave(
        &self,
        post_id: i64,
        author_id: i64,
        title: &str,
        content: &str,
        excerpt: &str,
    ) -> Revision {
        Revision::from_post(post_id, author_id, title, content, excerpt)
            .as_autosave()
    }

    /// Clean up old autosaves
    pub fn cleanup_autosaves(&self, revisions: &mut Vec<Revision>) {
        // Get only autosaves
        let mut autosaves: Vec<&Revision> = revisions.iter()
            .filter(|r| r.is_autosave)
            .collect();

        if autosaves.len() <= self.config.max_autosaves {
            return;
        }

        // Sort by date (newest first)
        autosaves.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // Mark old ones for removal
        let to_remove: Vec<Uuid> = autosaves
            .iter()
            .skip(self.config.max_autosaves)
            .map(|r| r.id)
            .collect();

        revisions.retain(|r| !to_remove.contains(&r.id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_revision_creation() {
        let rev = Revision::from_post(1, 1, "Title", "Content", "Excerpt");
        assert_eq!(rev.post_id, 1);
        assert_eq!(rev.title, "Title");
        assert!(!rev.content_hash.is_empty());
    }

    #[test]
    fn test_diff_text() {
        let diff = RevisionDiff::new();
        let old = "Hello\nWorld";
        let new = "Hello\nRust";

        let changes = diff.diff_text(old, new);

        assert!(changes.iter().any(|c| c.change_type == ChangeType::Equal && c.text.contains("Hello")));
        assert!(changes.iter().any(|c| c.change_type == ChangeType::Delete && c.text.contains("World")));
        assert!(changes.iter().any(|c| c.change_type == ChangeType::Insert && c.text.contains("Rust")));
    }

    #[test]
    fn test_revision_compare() {
        let manager = RevisionManager::new();

        let old = Revision::from_post(1, 1, "Old Title", "Old content", "");
        let new = Revision::from_post(1, 1, "New Title", "New content", "");

        let diff = manager.compare(&old, &new);

        assert!(diff.stats.lines_removed > 0 || diff.stats.lines_added > 0);
    }

    #[test]
    fn test_content_hash() {
        let rev = Revision::from_post(1, 1, "Title", "Content", "");
        assert!(rev.matches_content("Content"));
        assert!(!rev.matches_content("Different"));
    }

    #[test]
    fn test_autosave_trigger() {
        let manager = AutosaveManager::new(AutosaveConfig {
            enabled: true,
            interval_seconds: 60,
            min_change_threshold: 10,
            ..Default::default()
        });

        // Should not trigger - not enough time
        assert!(!manager.should_autosave("old", "new content here", 30));

        // Should not trigger - not enough change
        assert!(!manager.should_autosave("content", "contentt", 120));

        // Should trigger
        assert!(manager.should_autosave("old", "new content with changes", 120));
    }
}
