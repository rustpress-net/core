//! # Bulk Editing Operations
//!
//! Batch operations for content management.
//!
//! Features:
//! - Bulk status changes
//! - Bulk category/tag assignment
//! - Bulk author changes
//! - Bulk delete/trash
//! - Bulk custom field updates
//! - Quick inline editing

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Bulk operation errors
#[derive(Debug, Error)]
pub enum BulkError {
    #[error("Operation failed for item {0}: {1}")]
    ItemFailed(i64, String),

    #[error("No items provided")]
    NoItems,

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Operation error: {0}")]
    OperationError(String),
}

/// Bulk operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BulkOperationType {
    /// Change post status
    ChangeStatus,

    /// Move to trash
    Trash,

    /// Restore from trash
    Restore,

    /// Permanent delete
    Delete,

    /// Change author
    ChangeAuthor,

    /// Add categories
    AddCategories,

    /// Remove categories
    RemoveCategories,

    /// Set categories (replace all)
    SetCategories,

    /// Add tags
    AddTags,

    /// Remove tags
    RemoveTags,

    /// Set tags (replace all)
    SetTags,

    /// Enable/disable comments
    SetCommentStatus,

    /// Set sticky status
    SetSticky,

    /// Update custom field
    SetMeta,

    /// Remove custom field
    RemoveMeta,

    /// Change post format
    SetFormat,

    /// Change template
    SetTemplate,
}

/// Bulk operation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperation {
    /// Type of operation
    pub operation: BulkOperationType,

    /// IDs of items to operate on
    pub item_ids: Vec<i64>,

    /// Operation parameters
    pub params: BulkParams,

    /// User performing the operation
    pub user_id: i64,

    /// When the operation was requested
    pub requested_at: DateTime<Utc>,
}

/// Parameters for bulk operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BulkParams {
    /// New status for ChangeStatus
    pub status: Option<String>,

    /// Author ID for ChangeAuthor
    pub author_id: Option<i64>,

    /// Category IDs for category operations
    pub category_ids: Option<Vec<i64>>,

    /// Tag IDs for tag operations
    pub tag_ids: Option<Vec<i64>>,

    /// Comment status (open/closed)
    pub comment_status: Option<String>,

    /// Ping status (open/closed)
    pub ping_status: Option<String>,

    /// Sticky status
    pub sticky: Option<bool>,

    /// Meta key for meta operations
    pub meta_key: Option<String>,

    /// Meta value for meta operations
    pub meta_value: Option<String>,

    /// Post format
    pub format: Option<String>,

    /// Page template
    pub template: Option<String>,

    /// Additional custom parameters
    pub custom: HashMap<String, String>,
}

/// Result of a bulk operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkResult {
    /// Operation type
    pub operation: BulkOperationType,

    /// Successfully processed items
    pub succeeded: Vec<i64>,

    /// Failed items with reasons
    pub failed: Vec<BulkFailure>,

    /// Total items attempted
    pub total: usize,

    /// When the operation completed
    pub completed_at: DateTime<Utc>,

    /// Duration in milliseconds
    pub duration_ms: u64,
}

impl BulkResult {
    pub fn new(operation: BulkOperationType, total: usize) -> Self {
        Self {
            operation,
            succeeded: Vec::new(),
            failed: Vec::new(),
            total,
            completed_at: Utc::now(),
            duration_ms: 0,
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.succeeded.len() as f64 / self.total as f64 * 100.0
        }
    }

    pub fn is_complete_success(&self) -> bool {
        self.failed.is_empty()
    }

    pub fn is_complete_failure(&self) -> bool {
        self.succeeded.is_empty() && !self.failed.is_empty()
    }

    pub fn summary(&self) -> String {
        format!(
            "{} of {} items processed successfully ({:.1}%)",
            self.succeeded.len(),
            self.total,
            self.success_rate()
        )
    }
}

/// Details of a bulk operation failure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkFailure {
    pub item_id: i64,
    pub reason: String,
    pub error_code: Option<String>,
}

/// Bulk operation executor
pub struct BulkExecutor {
    /// Maximum items per batch
    max_batch_size: usize,

    /// Enable parallel processing
    parallel: bool,

    /// Skip items on error (continue with next)
    skip_on_error: bool,
}

impl Default for BulkExecutor {
    fn default() -> Self {
        Self {
            max_batch_size: 100,
            parallel: false,
            skip_on_error: true,
        }
    }
}

impl BulkExecutor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.max_batch_size = size;
        self
    }

    pub fn parallel(mut self, enabled: bool) -> Self {
        self.parallel = enabled;
        self
    }

    pub fn stop_on_error(mut self) -> Self {
        self.skip_on_error = false;
        self
    }

    /// Execute a bulk operation
    pub fn execute<F>(&self, operation: &BulkOperation, handler: F) -> BulkResult
    where
        F: Fn(i64, &BulkParams) -> Result<(), String>,
    {
        let start = std::time::Instant::now();
        let mut result = BulkResult::new(operation.operation, operation.item_ids.len());

        for &item_id in &operation.item_ids {
            match handler(item_id, &operation.params) {
                Ok(()) => result.succeeded.push(item_id),
                Err(reason) => {
                    result.failed.push(BulkFailure {
                        item_id,
                        reason,
                        error_code: None,
                    });

                    if !self.skip_on_error {
                        break;
                    }
                }
            }
        }

        result.completed_at = Utc::now();
        result.duration_ms = start.elapsed().as_millis() as u64;

        result
    }

    /// Create a change status operation
    pub fn change_status(item_ids: Vec<i64>, status: &str, user_id: i64) -> BulkOperation {
        BulkOperation {
            operation: BulkOperationType::ChangeStatus,
            item_ids,
            params: BulkParams {
                status: Some(status.to_string()),
                ..Default::default()
            },
            user_id,
            requested_at: Utc::now(),
        }
    }

    /// Create a trash operation
    pub fn trash(item_ids: Vec<i64>, user_id: i64) -> BulkOperation {
        BulkOperation {
            operation: BulkOperationType::Trash,
            item_ids,
            params: BulkParams::default(),
            user_id,
            requested_at: Utc::now(),
        }
    }

    /// Create a restore operation
    pub fn restore(item_ids: Vec<i64>, user_id: i64) -> BulkOperation {
        BulkOperation {
            operation: BulkOperationType::Restore,
            item_ids,
            params: BulkParams::default(),
            user_id,
            requested_at: Utc::now(),
        }
    }

    /// Create a delete operation
    pub fn delete(item_ids: Vec<i64>, user_id: i64) -> BulkOperation {
        BulkOperation {
            operation: BulkOperationType::Delete,
            item_ids,
            params: BulkParams::default(),
            user_id,
            requested_at: Utc::now(),
        }
    }

    /// Create an add categories operation
    pub fn add_categories(item_ids: Vec<i64>, category_ids: Vec<i64>, user_id: i64) -> BulkOperation {
        BulkOperation {
            operation: BulkOperationType::AddCategories,
            item_ids,
            params: BulkParams {
                category_ids: Some(category_ids),
                ..Default::default()
            },
            user_id,
            requested_at: Utc::now(),
        }
    }

    /// Create an add tags operation
    pub fn add_tags(item_ids: Vec<i64>, tag_ids: Vec<i64>, user_id: i64) -> BulkOperation {
        BulkOperation {
            operation: BulkOperationType::AddTags,
            item_ids,
            params: BulkParams {
                tag_ids: Some(tag_ids),
                ..Default::default()
            },
            user_id,
            requested_at: Utc::now(),
        }
    }

    /// Create a change author operation
    pub fn change_author(item_ids: Vec<i64>, author_id: i64, user_id: i64) -> BulkOperation {
        BulkOperation {
            operation: BulkOperationType::ChangeAuthor,
            item_ids,
            params: BulkParams {
                author_id: Some(author_id),
                ..Default::default()
            },
            user_id,
            requested_at: Utc::now(),
        }
    }

    /// Create a set meta operation
    pub fn set_meta(item_ids: Vec<i64>, key: &str, value: &str, user_id: i64) -> BulkOperation {
        BulkOperation {
            operation: BulkOperationType::SetMeta,
            item_ids,
            params: BulkParams {
                meta_key: Some(key.to_string()),
                meta_value: Some(value.to_string()),
                ..Default::default()
            },
            user_id,
            requested_at: Utc::now(),
        }
    }
}

// ============================================================================
// Quick Edit Functionality
// ============================================================================

/// Quick edit fields (inline editing)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuickEditData {
    /// Post ID
    pub id: i64,

    /// Title (optional update)
    pub title: Option<String>,

    /// Slug (optional update)
    pub slug: Option<String>,

    /// Date (optional update)
    pub date: Option<DateTime<Utc>>,

    /// Author ID (optional update)
    pub author_id: Option<i64>,

    /// Status (optional update)
    pub status: Option<String>,

    /// Password (optional update)
    pub password: Option<String>,

    /// Category IDs (optional update)
    pub categories: Option<Vec<i64>>,

    /// Tag IDs (optional update)
    pub tags: Option<Vec<i64>>,

    /// Comment status (optional update)
    pub comment_status: Option<String>,

    /// Ping status (optional update)
    pub ping_status: Option<String>,

    /// Sticky (optional update)
    pub sticky: Option<bool>,

    /// Post format (optional update)
    pub format: Option<String>,

    /// Template (optional update)
    pub template: Option<String>,

    /// Menu order (optional update)
    pub menu_order: Option<i32>,

    /// Parent ID (optional update)
    pub parent_id: Option<i64>,
}

impl QuickEditData {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn with_slug(mut self, slug: &str) -> Self {
        self.slug = Some(slug.to_string());
        self
    }

    pub fn with_status(mut self, status: &str) -> Self {
        self.status = Some(status.to_string());
        self
    }

    pub fn with_author(mut self, author_id: i64) -> Self {
        self.author_id = Some(author_id);
        self
    }

    pub fn with_categories(mut self, categories: Vec<i64>) -> Self {
        self.categories = Some(categories);
        self
    }

    pub fn with_tags(mut self, tags: Vec<i64>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Get list of fields being updated
    pub fn updated_fields(&self) -> Vec<&str> {
        let mut fields = Vec::new();

        if self.title.is_some() { fields.push("title"); }
        if self.slug.is_some() { fields.push("slug"); }
        if self.date.is_some() { fields.push("date"); }
        if self.author_id.is_some() { fields.push("author"); }
        if self.status.is_some() { fields.push("status"); }
        if self.password.is_some() { fields.push("password"); }
        if self.categories.is_some() { fields.push("categories"); }
        if self.tags.is_some() { fields.push("tags"); }
        if self.comment_status.is_some() { fields.push("comment_status"); }
        if self.ping_status.is_some() { fields.push("ping_status"); }
        if self.sticky.is_some() { fields.push("sticky"); }
        if self.format.is_some() { fields.push("format"); }
        if self.template.is_some() { fields.push("template"); }
        if self.menu_order.is_some() { fields.push("menu_order"); }
        if self.parent_id.is_some() { fields.push("parent"); }

        fields
    }

    /// Check if any fields are being updated
    pub fn has_updates(&self) -> bool {
        !self.updated_fields().is_empty()
    }
}

/// Quick edit result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickEditResult {
    pub id: i64,
    pub success: bool,
    pub updated_fields: Vec<String>,
    pub errors: Vec<String>,
}

/// Quick edit handler
pub struct QuickEditHandler;

impl QuickEditHandler {
    /// Validate quick edit data
    pub fn validate(data: &QuickEditData) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if let Some(ref title) = data.title {
            if title.trim().is_empty() {
                errors.push("Title cannot be empty".to_string());
            }
        }

        if let Some(ref slug) = data.slug {
            if slug.contains(' ') {
                errors.push("Slug cannot contain spaces".to_string());
            }
        }

        if let Some(ref status) = data.status {
            let valid_statuses = ["draft", "pending", "publish", "future", "private"];
            if !valid_statuses.contains(&status.as_str()) {
                errors.push(format!("Invalid status: {}", status));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bulk_operation() {
        let executor = BulkExecutor::new();
        let operation = BulkExecutor::change_status(vec![1, 2, 3], "draft", 1);

        let result = executor.execute(&operation, |_id, _params| {
            Ok(())
        });

        assert_eq!(result.succeeded.len(), 3);
        assert!(result.is_complete_success());
    }

    #[test]
    fn test_bulk_with_failures() {
        let executor = BulkExecutor::new();
        let operation = BulkExecutor::trash(vec![1, 2, 3], 1);

        let result = executor.execute(&operation, |id, _params| {
            if id == 2 {
                Err("Item locked".to_string())
            } else {
                Ok(())
            }
        });

        assert_eq!(result.succeeded.len(), 2);
        assert_eq!(result.failed.len(), 1);
        assert_eq!(result.failed[0].item_id, 2);
    }

    #[test]
    fn test_quick_edit_data() {
        let data = QuickEditData::new(1)
            .with_title("New Title")
            .with_status("draft");

        assert!(data.has_updates());
        assert!(data.updated_fields().contains(&"title"));
        assert!(data.updated_fields().contains(&"status"));
    }

    #[test]
    fn test_quick_edit_validation() {
        let mut data = QuickEditData::new(1);
        data.title = Some("".to_string());
        data.slug = Some("has spaces".to_string());

        let result = QuickEditHandler::validate(&data);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 2);
    }
}
