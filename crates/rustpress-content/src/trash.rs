//! # Trash and Restore System
//!
//! Soft delete functionality with automatic expiration.
//!
//! Features:
//! - Soft delete to trash
//! - Restore from trash
//! - Automatic expiration
//! - Bulk trash operations
//! - Trash history

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Trash system errors
#[derive(Debug, Error)]
pub enum TrashError {
    #[error("Item not found: {0}")]
    NotFound(i64),

    #[error("Item not in trash: {0}")]
    NotInTrash(i64),

    #[error("Cannot restore: {0}")]
    CannotRestore(String),

    #[error("Expiration error: {0}")]
    ExpirationError(String),
}

/// Trashed item metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrashedItem {
    /// Item ID
    pub id: i64,

    /// Item type (post, page, attachment, etc.)
    pub item_type: String,

    /// Original title
    pub title: String,

    /// Previous status before trashing
    pub previous_status: String,

    /// Who trashed it
    pub trashed_by: i64,

    /// When it was trashed
    pub trashed_at: DateTime<Utc>,

    /// When it will expire (permanent delete)
    pub expires_at: DateTime<Utc>,

    /// Additional metadata
    pub meta: HashMap<String, String>,
}

impl TrashedItem {
    pub fn new(id: i64, item_type: &str, title: &str, previous_status: &str, trashed_by: i64, retention_days: i64) -> Self {
        let now = Utc::now();
        Self {
            id,
            item_type: item_type.to_string(),
            title: title.to_string(),
            previous_status: previous_status.to_string(),
            trashed_by,
            trashed_at: now,
            expires_at: now + Duration::days(retention_days),
            meta: HashMap::new(),
        }
    }

    /// Check if item has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Time remaining until expiration
    pub fn time_remaining(&self) -> Duration {
        let remaining = self.expires_at.signed_duration_since(Utc::now());
        if remaining.num_seconds() < 0 {
            Duration::zero()
        } else {
            remaining
        }
    }

    /// Format time remaining for display
    pub fn time_remaining_display(&self) -> String {
        let remaining = self.time_remaining();
        let days = remaining.num_days();
        let hours = remaining.num_hours() % 24;

        if days > 0 {
            format!("{} days", days)
        } else if hours > 0 {
            format!("{} hours", hours)
        } else {
            let minutes = remaining.num_minutes();
            format!("{} minutes", minutes)
        }
    }
}

/// Trash manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrashConfig {
    /// Days to retain items in trash before permanent deletion
    pub retention_days: i64,

    /// Enable automatic expiration
    pub auto_expire: bool,

    /// Maximum items in trash
    pub max_items: usize,

    /// Whether to log trash operations
    pub log_operations: bool,
}

impl Default for TrashConfig {
    fn default() -> Self {
        Self {
            retention_days: 30,
            auto_expire: true,
            max_items: 10000,
            log_operations: true,
        }
    }
}

/// Trash manager
pub struct TrashManager {
    config: TrashConfig,
    items: HashMap<i64, TrashedItem>,
}

impl Default for TrashManager {
    fn default() -> Self {
        Self::new(TrashConfig::default())
    }
}

impl TrashManager {
    pub fn new(config: TrashConfig) -> Self {
        Self {
            config,
            items: HashMap::new(),
        }
    }

    /// Trash an item
    pub fn trash(
        &mut self,
        id: i64,
        item_type: &str,
        title: &str,
        previous_status: &str,
        user_id: i64,
    ) -> Result<TrashedItem, TrashError> {
        // Check max items
        if self.items.len() >= self.config.max_items {
            // Remove oldest expired items
            self.cleanup_expired();

            if self.items.len() >= self.config.max_items {
                // Remove oldest item
                if let Some(&oldest_id) = self.items.values()
                    .min_by_key(|item| item.trashed_at)
                    .map(|item| &item.id)
                {
                    self.items.remove(&oldest_id);
                }
            }
        }

        let item = TrashedItem::new(id, item_type, title, previous_status, user_id, self.config.retention_days);
        self.items.insert(id, item.clone());

        Ok(item)
    }

    /// Restore an item from trash
    pub fn restore(&mut self, id: i64) -> Result<TrashedItem, TrashError> {
        self.items.remove(&id).ok_or(TrashError::NotInTrash(id))
    }

    /// Get trashed item
    pub fn get(&self, id: i64) -> Option<&TrashedItem> {
        self.items.get(&id)
    }

    /// Check if item is in trash
    pub fn is_trashed(&self, id: i64) -> bool {
        self.items.contains_key(&id)
    }

    /// Get all trashed items
    pub fn get_all(&self) -> Vec<&TrashedItem> {
        self.items.values().collect()
    }

    /// Get trashed items by type
    pub fn get_by_type(&self, item_type: &str) -> Vec<&TrashedItem> {
        self.items.values()
            .filter(|item| item.item_type == item_type)
            .collect()
    }

    /// Get expired items
    pub fn get_expired(&self) -> Vec<&TrashedItem> {
        self.items.values()
            .filter(|item| item.is_expired())
            .collect()
    }

    /// Clean up expired items
    pub fn cleanup_expired(&mut self) -> Vec<i64> {
        let expired_ids: Vec<i64> = self.items.values()
            .filter(|item| item.is_expired())
            .map(|item| item.id)
            .collect();

        for id in &expired_ids {
            self.items.remove(id);
        }

        expired_ids
    }

    /// Permanently delete item from trash
    pub fn delete_permanently(&mut self, id: i64) -> Result<TrashedItem, TrashError> {
        self.items.remove(&id).ok_or(TrashError::NotInTrash(id))
    }

    /// Empty trash (delete all)
    pub fn empty(&mut self) -> Vec<TrashedItem> {
        let items: Vec<TrashedItem> = self.items.values().cloned().collect();
        self.items.clear();
        items
    }

    /// Empty trash for specific type
    pub fn empty_by_type(&mut self, item_type: &str) -> Vec<TrashedItem> {
        let to_remove: Vec<i64> = self.items.values()
            .filter(|item| item.item_type == item_type)
            .map(|item| item.id)
            .collect();

        let mut removed = Vec::new();
        for id in to_remove {
            if let Some(item) = self.items.remove(&id) {
                removed.push(item);
            }
        }

        removed
    }

    /// Get trash count
    pub fn count(&self) -> usize {
        self.items.len()
    }

    /// Get trash count by type
    pub fn count_by_type(&self, item_type: &str) -> usize {
        self.items.values()
            .filter(|item| item.item_type == item_type)
            .count()
    }

    /// Update configuration
    pub fn set_config(&mut self, config: TrashConfig) {
        self.config = config;
    }
}

/// Bulk trash operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkTrashResult {
    pub succeeded: Vec<i64>,
    pub failed: Vec<(i64, String)>,
    pub total_attempted: usize,
}

impl BulkTrashResult {
    pub fn new() -> Self {
        Self {
            succeeded: Vec::new(),
            failed: Vec::new(),
            total_attempted: 0,
        }
    }

    pub fn success_count(&self) -> usize {
        self.succeeded.len()
    }

    pub fn failure_count(&self) -> usize {
        self.failed.len()
    }
}

impl Default for BulkTrashResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Bulk trash operations
pub trait BulkTrashOperations {
    fn bulk_trash(&mut self, ids: &[i64], item_type: &str, user_id: i64) -> BulkTrashResult;
    fn bulk_restore(&mut self, ids: &[i64]) -> BulkTrashResult;
    fn bulk_delete(&mut self, ids: &[i64]) -> BulkTrashResult;
}

impl BulkTrashOperations for TrashManager {
    fn bulk_trash(&mut self, ids: &[i64], item_type: &str, user_id: i64) -> BulkTrashResult {
        let mut result = BulkTrashResult::new();
        result.total_attempted = ids.len();

        for &id in ids {
            match self.trash(id, item_type, "", "publish", user_id) {
                Ok(_) => result.succeeded.push(id),
                Err(e) => result.failed.push((id, e.to_string())),
            }
        }

        result
    }

    fn bulk_restore(&mut self, ids: &[i64]) -> BulkTrashResult {
        let mut result = BulkTrashResult::new();
        result.total_attempted = ids.len();

        for &id in ids {
            match self.restore(id) {
                Ok(_) => result.succeeded.push(id),
                Err(e) => result.failed.push((id, e.to_string())),
            }
        }

        result
    }

    fn bulk_delete(&mut self, ids: &[i64]) -> BulkTrashResult {
        let mut result = BulkTrashResult::new();
        result.total_attempted = ids.len();

        for &id in ids {
            match self.delete_permanently(id) {
                Ok(_) => result.succeeded.push(id),
                Err(e) => result.failed.push((id, e.to_string())),
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trash_item() {
        let mut manager = TrashManager::default();

        let item = manager.trash(1, "post", "Test Post", "publish", 1).unwrap();
        assert_eq!(item.id, 1);
        assert!(!item.is_expired());
        assert!(manager.is_trashed(1));
    }

    #[test]
    fn test_restore_item() {
        let mut manager = TrashManager::default();

        manager.trash(1, "post", "Test Post", "publish", 1).unwrap();
        assert!(manager.is_trashed(1));

        let restored = manager.restore(1).unwrap();
        assert_eq!(restored.previous_status, "publish");
        assert!(!manager.is_trashed(1));
    }

    #[test]
    fn test_expiration() {
        let item = TrashedItem::new(1, "post", "Test", "publish", 1, 0);
        // With 0 days retention, should expire immediately
        assert!(item.is_expired());
    }

    #[test]
    fn test_bulk_operations() {
        let mut manager = TrashManager::default();

        let result = manager.bulk_trash(&[1, 2, 3], "post", 1);
        assert_eq!(result.success_count(), 3);

        let result = manager.bulk_restore(&[1, 2]);
        assert_eq!(result.success_count(), 2);

        assert!(manager.is_trashed(3));
        assert!(!manager.is_trashed(1));
    }
}
