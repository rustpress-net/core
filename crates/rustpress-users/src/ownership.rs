//! # Content Ownership & Multi-Author
//!
//! Content ownership transfer and multi-author support.
//!
//! Features:
//! - Content ownership transfer
//! - Multi-author post support
//! - Author ordering
//! - Contribution tracking

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Content ownership record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentOwnership {
    pub content_id: i64,
    pub content_type: String,
    pub owner_id: i64,
    pub created_at: DateTime<Utc>,
}

/// Ownership transfer request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipTransfer {
    pub id: Uuid,
    pub content_id: i64,
    pub content_type: String,
    pub from_user_id: i64,
    pub to_user_id: i64,
    pub status: TransferStatus,
    pub reason: Option<String>,
    pub requested_at: DateTime<Utc>,
    pub requested_by: i64,
    pub processed_at: Option<DateTime<Utc>>,
    pub processed_by: Option<i64>,
}

/// Transfer status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferStatus {
    Pending,
    Approved,
    Rejected,
    Cancelled,
}

impl OwnershipTransfer {
    pub fn new(content_id: i64, content_type: &str, from_user: i64, to_user: i64, requested_by: i64) -> Self {
        Self {
            id: Uuid::new_v4(),
            content_id,
            content_type: content_type.to_string(),
            from_user_id: from_user,
            to_user_id: to_user,
            status: TransferStatus::Pending,
            reason: None,
            requested_at: Utc::now(),
            requested_by,
            processed_at: None,
            processed_by: None,
        }
    }

    pub fn with_reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.to_string());
        self
    }

    pub fn approve(&mut self, by_user: i64) {
        self.status = TransferStatus::Approved;
        self.processed_at = Some(Utc::now());
        self.processed_by = Some(by_user);
    }

    pub fn reject(&mut self, by_user: i64) {
        self.status = TransferStatus::Rejected;
        self.processed_at = Some(Utc::now());
        self.processed_by = Some(by_user);
    }
}

/// Bulk transfer request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkTransfer {
    pub id: Uuid,
    pub from_user_id: i64,
    pub to_user_id: i64,
    pub content_types: Vec<String>,
    pub transfer_all: bool,
    pub specific_ids: Vec<i64>,
    pub status: TransferStatus,
    pub items_transferred: u32,
    pub items_failed: u32,
    pub requested_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl BulkTransfer {
    pub fn all_content(from_user: i64, to_user: i64) -> Self {
        Self {
            id: Uuid::new_v4(),
            from_user_id: from_user,
            to_user_id: to_user,
            content_types: vec!["post".to_string(), "page".to_string()],
            transfer_all: true,
            specific_ids: Vec::new(),
            status: TransferStatus::Pending,
            items_transferred: 0,
            items_failed: 0,
            requested_at: Utc::now(),
            completed_at: None,
        }
    }

    pub fn specific_items(from_user: i64, to_user: i64, ids: Vec<i64>) -> Self {
        Self {
            id: Uuid::new_v4(),
            from_user_id: from_user,
            to_user_id: to_user,
            content_types: Vec::new(),
            transfer_all: false,
            specific_ids: ids,
            status: TransferStatus::Pending,
            items_transferred: 0,
            items_failed: 0,
            requested_at: Utc::now(),
            completed_at: None,
        }
    }
}

// ============================================================================
// Multi-Author Support
// ============================================================================

/// Post author entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostAuthor {
    pub post_id: i64,
    pub user_id: i64,
    pub role: AuthorRole,
    pub order: i32,
    pub added_at: DateTime<Utc>,
    pub added_by: i64,
    pub contribution: Option<String>,
}

/// Author role in multi-author posts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorRole {
    Primary,
    CoAuthor,
    Contributor,
    Editor,
    Reviewer,
}

impl AuthorRole {
    pub fn label(&self) -> &str {
        match self {
            Self::Primary => "Primary Author",
            Self::CoAuthor => "Co-Author",
            Self::Contributor => "Contributor",
            Self::Editor => "Editor",
            Self::Reviewer => "Reviewer",
        }
    }

    pub fn can_edit(&self) -> bool {
        matches!(self, Self::Primary | Self::CoAuthor | Self::Editor)
    }

    pub fn can_publish(&self) -> bool {
        matches!(self, Self::Primary)
    }
}

impl PostAuthor {
    pub fn primary(post_id: i64, user_id: i64, added_by: i64) -> Self {
        Self {
            post_id,
            user_id,
            role: AuthorRole::Primary,
            order: 0,
            added_at: Utc::now(),
            added_by,
            contribution: None,
        }
    }

    pub fn co_author(post_id: i64, user_id: i64, added_by: i64) -> Self {
        Self {
            post_id,
            user_id,
            role: AuthorRole::CoAuthor,
            order: 1,
            added_at: Utc::now(),
            added_by,
            contribution: None,
        }
    }

    pub fn with_contribution(mut self, contribution: &str) -> Self {
        self.contribution = Some(contribution.to_string());
        self
    }

    pub fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }
}

/// Contribution tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contribution {
    pub id: Uuid,
    pub post_id: i64,
    pub user_id: i64,
    pub contribution_type: ContributionType,
    pub description: Option<String>,
    pub word_count: Option<u32>,
    pub sections: Vec<String>,
    pub recorded_at: DateTime<Utc>,
}

/// Types of contributions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContributionType {
    Writing,
    Editing,
    Research,
    Images,
    Review,
    Translation,
    Custom(String),
}

impl ContributionType {
    pub fn label(&self) -> &str {
        match self {
            Self::Writing => "Writing",
            Self::Editing => "Editing",
            Self::Research => "Research",
            Self::Images => "Images/Media",
            Self::Review => "Review",
            Self::Translation => "Translation",
            Self::Custom(s) => s,
        }
    }
}

/// Multi-author manager
pub struct MultiAuthorManager {
    authors: HashMap<i64, Vec<PostAuthor>>,
    contributions: Vec<Contribution>,
    ownership: HashMap<(i64, String), ContentOwnership>,
    transfers: Vec<OwnershipTransfer>,
}

impl Default for MultiAuthorManager {
    fn default() -> Self {
        Self {
            authors: HashMap::new(),
            contributions: Vec::new(),
            ownership: HashMap::new(),
            transfers: Vec::new(),
        }
    }
}

impl MultiAuthorManager {
    pub fn new() -> Self {
        Self::default()
    }

    // Ownership methods

    /// Set content owner
    pub fn set_owner(&mut self, content_id: i64, content_type: &str, owner_id: i64) {
        self.ownership.insert(
            (content_id, content_type.to_string()),
            ContentOwnership {
                content_id,
                content_type: content_type.to_string(),
                owner_id,
                created_at: Utc::now(),
            },
        );
    }

    /// Get content owner
    pub fn get_owner(&self, content_id: i64, content_type: &str) -> Option<i64> {
        self.ownership.get(&(content_id, content_type.to_string()))
            .map(|o| o.owner_id)
    }

    /// Request ownership transfer
    pub fn request_transfer(&mut self, transfer: OwnershipTransfer) -> Uuid {
        let id = transfer.id;
        self.transfers.push(transfer);
        id
    }

    /// Get pending transfers for user
    pub fn get_pending_transfers(&self, user_id: i64) -> Vec<&OwnershipTransfer> {
        self.transfers.iter()
            .filter(|t| {
                t.status == TransferStatus::Pending &&
                (t.from_user_id == user_id || t.to_user_id == user_id)
            })
            .collect()
    }

    /// Process transfer
    pub fn process_transfer(&mut self, transfer_id: Uuid, approved: bool, by_user: i64) -> Result<(), String> {
        // Find transfer and extract ownership info before mutation
        let ownership_info = {
            let transfer = self.transfers.iter_mut()
                .find(|t| t.id == transfer_id)
                .ok_or_else(|| "Transfer not found".to_string())?;

            if transfer.status != TransferStatus::Pending {
                return Err("Transfer already processed".to_string());
            }

            let info = if approved {
                transfer.approve(by_user);
                Some((transfer.content_id, transfer.content_type.clone(), transfer.to_user_id))
            } else {
                transfer.reject(by_user);
                None
            };
            info
        };

        // Update ownership after releasing the mutable borrow
        if let Some((content_id, content_type, to_user_id)) = ownership_info {
            self.set_owner(content_id, &content_type, to_user_id);
        }

        Ok(())
    }

    // Multi-author methods

    /// Add author to post
    pub fn add_author(&mut self, author: PostAuthor) -> Result<(), String> {
        let authors = self.authors.entry(author.post_id)
            .or_insert_with(Vec::new);

        // Check if already an author
        if authors.iter().any(|a| a.user_id == author.user_id) {
            return Err("User is already an author".to_string());
        }

        // Check for existing primary author
        if author.role == AuthorRole::Primary {
            if authors.iter().any(|a| a.role == AuthorRole::Primary) {
                return Err("Post already has a primary author".to_string());
            }
        }

        authors.push(author);
        authors.sort_by_key(|a| a.order);

        Ok(())
    }

    /// Remove author from post
    pub fn remove_author(&mut self, post_id: i64, user_id: i64) -> Result<(), String> {
        let authors = self.authors.get_mut(&post_id)
            .ok_or_else(|| "Post not found".to_string())?;

        // Cannot remove primary author if they're the only one
        let is_primary = authors.iter().find(|a| a.user_id == user_id)
            .map(|a| a.role == AuthorRole::Primary)
            .unwrap_or(false);

        if is_primary && authors.len() == 1 {
            return Err("Cannot remove the only primary author".to_string());
        }

        authors.retain(|a| a.user_id != user_id);

        Ok(())
    }

    /// Get authors for post
    pub fn get_authors(&self, post_id: i64) -> Vec<&PostAuthor> {
        self.authors.get(&post_id)
            .map(|authors| authors.iter().collect())
            .unwrap_or_default()
    }

    /// Get primary author
    pub fn get_primary_author(&self, post_id: i64) -> Option<&PostAuthor> {
        self.authors.get(&post_id)
            .and_then(|authors| authors.iter().find(|a| a.role == AuthorRole::Primary))
    }

    /// Change author role
    pub fn change_author_role(&mut self, post_id: i64, user_id: i64, new_role: AuthorRole) -> Result<(), String> {
        let authors = self.authors.get_mut(&post_id)
            .ok_or_else(|| "Post not found".to_string())?;

        // If changing to primary, demote current primary
        if new_role == AuthorRole::Primary {
            for author in authors.iter_mut() {
                if author.role == AuthorRole::Primary {
                    author.role = AuthorRole::CoAuthor;
                }
            }
        }

        let author = authors.iter_mut()
            .find(|a| a.user_id == user_id)
            .ok_or_else(|| "Author not found".to_string())?;

        author.role = new_role;

        Ok(())
    }

    /// Reorder authors
    pub fn reorder_authors(&mut self, post_id: i64, user_ids: &[i64]) -> Result<(), String> {
        let authors = self.authors.get_mut(&post_id)
            .ok_or_else(|| "Post not found".to_string())?;

        for (index, user_id) in user_ids.iter().enumerate() {
            if let Some(author) = authors.iter_mut().find(|a| a.user_id == *user_id) {
                author.order = index as i32;
            }
        }

        authors.sort_by_key(|a| a.order);

        Ok(())
    }

    /// Check if user is author
    pub fn is_author(&self, post_id: i64, user_id: i64) -> bool {
        self.authors.get(&post_id)
            .map(|authors| authors.iter().any(|a| a.user_id == user_id))
            .unwrap_or(false)
    }

    /// Check if user can edit
    pub fn can_edit(&self, post_id: i64, user_id: i64) -> bool {
        self.authors.get(&post_id)
            .and_then(|authors| authors.iter().find(|a| a.user_id == user_id))
            .map(|author| author.role.can_edit())
            .unwrap_or(false)
    }

    // Contribution methods

    /// Record contribution
    pub fn record_contribution(&mut self, contribution: Contribution) {
        self.contributions.push(contribution);
    }

    /// Get contributions for post
    pub fn get_contributions(&self, post_id: i64) -> Vec<&Contribution> {
        self.contributions.iter()
            .filter(|c| c.post_id == post_id)
            .collect()
    }

    /// Get contributions by user
    pub fn get_user_contributions(&self, user_id: i64) -> Vec<&Contribution> {
        self.contributions.iter()
            .filter(|c| c.user_id == user_id)
            .collect()
    }

    /// Get posts by author
    pub fn get_posts_by_author(&self, user_id: i64) -> Vec<i64> {
        self.authors.iter()
            .filter(|(_, authors)| authors.iter().any(|a| a.user_id == user_id))
            .map(|(post_id, _)| *post_id)
            .collect()
    }

    /// Get co-authored posts
    pub fn get_coauthored_posts(&self, user_id: i64) -> Vec<i64> {
        self.authors.iter()
            .filter(|(_, authors)| {
                authors.len() > 1 && authors.iter().any(|a| a.user_id == user_id)
            })
            .map(|(post_id, _)| *post_id)
            .collect()
    }
}

/// Byline generator
pub struct BylineGenerator;

impl BylineGenerator {
    /// Generate byline string
    pub fn generate(authors: &[&PostAuthor], names: &HashMap<i64, String>) -> String {
        let author_names: Vec<_> = authors.iter()
            .filter_map(|a| names.get(&a.user_id))
            .collect();

        match author_names.len() {
            0 => String::new(),
            1 => author_names[0].clone(),
            2 => format!("{} and {}", author_names[0], author_names[1]),
            _ => {
                let last = author_names.last().unwrap();
                let rest: Vec<_> = author_names[..author_names.len()-1].iter()
                    .map(|s| s.as_str())
                    .collect();
                format!("{}, and {}", rest.join(", "), last)
            }
        }
    }

    /// Generate byline with roles
    pub fn generate_with_roles(authors: &[&PostAuthor], names: &HashMap<i64, String>) -> String {
        authors.iter()
            .filter_map(|a| {
                names.get(&a.user_id).map(|name| {
                    if a.role == AuthorRole::Primary {
                        name.clone()
                    } else {
                        format!("{} ({})", name, a.role.label())
                    }
                })
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ownership_transfer() {
        let mut manager = MultiAuthorManager::new();
        manager.set_owner(1, "post", 10);

        let transfer = OwnershipTransfer::new(1, "post", 10, 20, 10);
        let transfer_id = manager.request_transfer(transfer);

        manager.process_transfer(transfer_id, true, 1).unwrap();

        assert_eq!(manager.get_owner(1, "post"), Some(20));
    }

    #[test]
    fn test_multi_author() {
        let mut manager = MultiAuthorManager::new();

        manager.add_author(PostAuthor::primary(1, 10, 10)).unwrap();
        manager.add_author(PostAuthor::co_author(1, 20, 10)).unwrap();

        let authors = manager.get_authors(1);
        assert_eq!(authors.len(), 2);
        assert!(manager.is_author(1, 10));
        assert!(manager.is_author(1, 20));
    }

    #[test]
    fn test_author_roles() {
        let mut manager = MultiAuthorManager::new();

        manager.add_author(PostAuthor::primary(1, 10, 10)).unwrap();
        manager.add_author(PostAuthor::co_author(1, 20, 10)).unwrap();

        assert!(manager.can_edit(1, 10));
        assert!(manager.can_edit(1, 20));

        let primary = manager.get_primary_author(1).unwrap();
        assert_eq!(primary.user_id, 10);
    }

    #[test]
    fn test_byline() {
        let authors = vec![
            PostAuthor::primary(1, 10, 10),
            PostAuthor::co_author(1, 20, 10),
        ];
        let author_refs: Vec<_> = authors.iter().collect();

        let mut names = HashMap::new();
        names.insert(10, "Alice".to_string());
        names.insert(20, "Bob".to_string());

        let byline = BylineGenerator::generate(&author_refs, &names);
        assert_eq!(byline, "Alice and Bob");
    }
}
