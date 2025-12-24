//! # User Listing & Bulk Actions
//!
//! User listing with search, filters, and bulk operations.
//!
//! Features:
//! - Paginated user listing
//! - Search and filtering
//! - Sorting options
//! - Bulk actions (delete, role change, etc.)
//! - Column customization

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// User list item for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserListItem {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub roles: Vec<String>,
    pub posts_count: u32,
    pub registered_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub status: UserStatus,
    pub avatar_url: Option<String>,
}

/// User status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    Active,
    Inactive,
    Pending,
    Suspended,
    Deleted,
}

impl UserStatus {
    pub fn label(&self) -> &str {
        match self {
            Self::Active => "Active",
            Self::Inactive => "Inactive",
            Self::Pending => "Pending",
            Self::Suspended => "Suspended",
            Self::Deleted => "Deleted",
        }
    }

    pub fn color(&self) -> &str {
        match self {
            Self::Active => "green",
            Self::Inactive => "gray",
            Self::Pending => "yellow",
            Self::Suspended => "red",
            Self::Deleted => "black",
        }
    }
}

/// User list query parameters
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserListQuery {
    /// Search term
    pub search: Option<String>,

    /// Filter by role
    pub role: Option<String>,

    /// Filter by status
    pub status: Option<UserStatus>,

    /// Sort field
    pub order_by: UserOrderBy,

    /// Sort direction
    pub order: SortOrder,

    /// Page number (1-indexed)
    pub page: u32,

    /// Items per page
    pub per_page: u32,

    /// Include specific user IDs
    pub include: Vec<i64>,

    /// Exclude specific user IDs
    pub exclude: Vec<i64>,

    /// Filter by registration date (after)
    pub registered_after: Option<DateTime<Utc>>,

    /// Filter by registration date (before)
    pub registered_before: Option<DateTime<Utc>>,

    /// Filter by last login (after)
    pub logged_in_after: Option<DateTime<Utc>>,

    /// Custom meta filters
    pub meta_query: Vec<MetaQuery>,
}

/// Fields to sort by
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum UserOrderBy {
    #[default]
    Username,
    Email,
    DisplayName,
    RegisteredDate,
    LastLogin,
    PostsCount,
    Id,
}

impl UserOrderBy {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Username => "username",
            Self::Email => "email",
            Self::DisplayName => "display_name",
            Self::RegisteredDate => "registered_at",
            Self::LastLogin => "last_login",
            Self::PostsCount => "posts_count",
            Self::Id => "id",
        }
    }
}

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SortOrder {
    #[default]
    Asc,
    Desc,
}

/// Meta query for filtering by user meta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaQuery {
    pub key: String,
    pub value: String,
    pub compare: MetaCompare,
}

/// Meta comparison operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MetaCompare {
    #[default]
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    Like,
    NotLike,
    In,
    NotIn,
    Exists,
    NotExists,
}

impl UserListQuery {
    pub fn new() -> Self {
        Self {
            page: 1,
            per_page: 20,
            ..Default::default()
        }
    }

    pub fn search(mut self, term: &str) -> Self {
        self.search = Some(term.to_string());
        self
    }

    pub fn role(mut self, role: &str) -> Self {
        self.role = Some(role.to_string());
        self
    }

    pub fn status(mut self, status: UserStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn order_by(mut self, field: UserOrderBy) -> Self {
        self.order_by = field;
        self
    }

    pub fn order(mut self, order: SortOrder) -> Self {
        self.order = order;
        self
    }

    pub fn page(mut self, page: u32) -> Self {
        self.page = page.max(1);
        self
    }

    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = per_page.clamp(1, 100);
        self
    }

    /// Get offset for pagination
    pub fn offset(&self) -> u32 {
        (self.page.saturating_sub(1)) * self.per_page
    }
}

/// Paginated user list result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserListResult {
    /// User items
    pub users: Vec<UserListItem>,

    /// Total user count (before pagination)
    pub total: u64,

    /// Current page
    pub page: u32,

    /// Items per page
    pub per_page: u32,

    /// Total pages
    pub total_pages: u32,

    /// Applied query
    pub query: UserListQuery,
}

impl UserListResult {
    pub fn new(users: Vec<UserListItem>, total: u64, query: UserListQuery) -> Self {
        let total_pages = ((total as f64) / (query.per_page as f64)).ceil() as u32;

        Self {
            users,
            total,
            page: query.page,
            per_page: query.per_page,
            total_pages,
            query,
        }
    }

    pub fn has_next_page(&self) -> bool {
        self.page < self.total_pages
    }

    pub fn has_prev_page(&self) -> bool {
        self.page > 1
    }
}

/// List table column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListColumn {
    pub id: String,
    pub label: String,
    pub sortable: bool,
    pub visible: bool,
    pub width: Option<String>,
    pub order: i32,
}

impl ListColumn {
    pub fn new(id: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            sortable: true,
            visible: true,
            width: None,
            order: 0,
        }
    }

    pub fn not_sortable(mut self) -> Self {
        self.sortable = false;
        self
    }

    pub fn hidden(mut self) -> Self {
        self.visible = false;
        self
    }

    pub fn width(mut self, width: &str) -> Self {
        self.width = Some(width.to_string());
        self
    }

    pub fn order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }
}

/// Default columns for user list
pub fn default_user_columns() -> Vec<ListColumn> {
    vec![
        ListColumn::new("cb", "").not_sortable().width("2.2em").order(0),
        ListColumn::new("username", "Username").order(1),
        ListColumn::new("name", "Name").order(2),
        ListColumn::new("email", "Email").order(3),
        ListColumn::new("role", "Role").not_sortable().order(4),
        ListColumn::new("posts", "Posts").order(5),
        ListColumn::new("registered", "Registered").order(6),
        ListColumn::new("last_login", "Last Login").hidden().order(7),
    ]
}

// ============================================================================
// Bulk Actions
// ============================================================================

/// Bulk action type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BulkAction {
    Delete,
    ChangeRole,
    SendPasswordReset,
    Activate,
    Deactivate,
    Suspend,
    ConfirmEmail,
    Export,
}

impl BulkAction {
    pub fn label(&self) -> &str {
        match self {
            Self::Delete => "Delete",
            Self::ChangeRole => "Change Role",
            Self::SendPasswordReset => "Send Password Reset",
            Self::Activate => "Activate",
            Self::Deactivate => "Deactivate",
            Self::Suspend => "Suspend",
            Self::ConfirmEmail => "Confirm Email",
            Self::Export => "Export",
        }
    }

    pub fn requires_confirmation(&self) -> bool {
        matches!(self, Self::Delete | Self::Suspend)
    }

    pub fn requires_additional_input(&self) -> bool {
        matches!(self, Self::ChangeRole)
    }
}

/// Bulk action request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkActionRequest {
    pub action: BulkAction,
    pub user_ids: Vec<i64>,
    pub params: HashMap<String, String>,
    pub requested_by: i64,
    pub requested_at: DateTime<Utc>,
}

impl BulkActionRequest {
    pub fn new(action: BulkAction, user_ids: Vec<i64>, requested_by: i64) -> Self {
        Self {
            action,
            user_ids,
            params: HashMap::new(),
            requested_by,
            requested_at: Utc::now(),
        }
    }

    pub fn with_param(mut self, key: &str, value: &str) -> Self {
        self.params.insert(key.to_string(), value.to_string());
        self
    }

    pub fn get_param(&self, key: &str) -> Option<&String> {
        self.params.get(key)
    }
}

/// Bulk action result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkActionResult {
    pub action: BulkAction,
    pub total_requested: usize,
    pub successful: Vec<i64>,
    pub failed: Vec<BulkActionError>,
    pub completed_at: DateTime<Utc>,
}

/// Bulk action error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkActionError {
    pub user_id: i64,
    pub error: String,
}

impl BulkActionResult {
    pub fn new(action: BulkAction, total: usize) -> Self {
        Self {
            action,
            total_requested: total,
            successful: Vec::new(),
            failed: Vec::new(),
            completed_at: Utc::now(),
        }
    }

    pub fn add_success(&mut self, user_id: i64) {
        self.successful.push(user_id);
    }

    pub fn add_failure(&mut self, user_id: i64, error: &str) {
        self.failed.push(BulkActionError {
            user_id,
            error: error.to_string(),
        });
    }

    pub fn success_count(&self) -> usize {
        self.successful.len()
    }

    pub fn failure_count(&self) -> usize {
        self.failed.len()
    }

    pub fn is_complete_success(&self) -> bool {
        self.failed.is_empty() && self.successful.len() == self.total_requested
    }
}

/// Bulk action executor
pub struct BulkExecutor {
    /// Users that cannot be deleted
    protected_users: Vec<i64>,
    /// Maximum users per bulk action
    max_batch_size: usize,
}

impl Default for BulkExecutor {
    fn default() -> Self {
        Self {
            protected_users: vec![1], // Default: protect user ID 1 (admin)
            max_batch_size: 100,
        }
    }
}

impl BulkExecutor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn protect_user(&mut self, user_id: i64) {
        if !self.protected_users.contains(&user_id) {
            self.protected_users.push(user_id);
        }
    }

    pub fn set_max_batch_size(&mut self, size: usize) {
        self.max_batch_size = size;
    }

    /// Validate bulk action request
    pub fn validate(&self, request: &BulkActionRequest) -> Result<(), String> {
        if request.user_ids.is_empty() {
            return Err("No users selected".to_string());
        }

        if request.user_ids.len() > self.max_batch_size {
            return Err(format!(
                "Too many users selected. Maximum is {}",
                self.max_batch_size
            ));
        }

        // Check for protected users in delete action
        if request.action == BulkAction::Delete {
            for id in &request.user_ids {
                if self.protected_users.contains(id) {
                    return Err(format!("User {} is protected and cannot be deleted", id));
                }
            }
        }

        // Validate additional params
        if request.action == BulkAction::ChangeRole && request.get_param("role").is_none() {
            return Err("Role parameter required for change role action".to_string());
        }

        // Prevent self-actions
        if request.user_ids.contains(&request.requested_by) {
            if matches!(request.action, BulkAction::Delete | BulkAction::Suspend | BulkAction::Deactivate) {
                return Err("Cannot perform this action on yourself".to_string());
            }
        }

        Ok(())
    }

    /// Get available bulk actions for user
    pub fn get_available_actions(&self, user_capabilities: &[&str]) -> Vec<BulkAction> {
        let mut actions = Vec::new();

        if user_capabilities.contains(&"delete_users") {
            actions.push(BulkAction::Delete);
        }

        if user_capabilities.contains(&"promote_users") {
            actions.push(BulkAction::ChangeRole);
        }

        if user_capabilities.contains(&"edit_users") {
            actions.push(BulkAction::SendPasswordReset);
            actions.push(BulkAction::Activate);
            actions.push(BulkAction::Deactivate);
            actions.push(BulkAction::Suspend);
            actions.push(BulkAction::ConfirmEmail);
        }

        if user_capabilities.contains(&"export") {
            actions.push(BulkAction::Export);
        }

        actions
    }
}

/// User search helper
pub struct UserSearch {
    /// Searchable fields
    fields: Vec<String>,
    /// Minimum search length
    min_length: usize,
}

impl Default for UserSearch {
    fn default() -> Self {
        Self {
            fields: vec![
                "username".to_string(),
                "email".to_string(),
                "display_name".to_string(),
                "first_name".to_string(),
                "last_name".to_string(),
            ],
            min_length: 2,
        }
    }
}

impl UserSearch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_field(&mut self, field: &str) {
        if !self.fields.contains(&field.to_string()) {
            self.fields.push(field.to_string());
        }
    }

    pub fn set_min_length(&mut self, length: usize) {
        self.min_length = length;
    }

    /// Validate search term
    pub fn validate(&self, term: &str) -> Result<(), String> {
        let trimmed = term.trim();

        if trimmed.len() < self.min_length {
            return Err(format!(
                "Search term must be at least {} characters",
                self.min_length
            ));
        }

        Ok(())
    }

    /// Build search pattern
    pub fn build_pattern(&self, term: &str) -> SearchPattern {
        let trimmed = term.trim();

        // Check if it's an exact email search
        if trimmed.contains('@') {
            return SearchPattern::Exact(trimmed.to_lowercase());
        }

        // Check if it's an ID search
        if let Ok(id) = trimmed.parse::<i64>() {
            return SearchPattern::Id(id);
        }

        // Default to wildcard search
        SearchPattern::Wildcard(trimmed.to_lowercase())
    }
}

/// Search pattern types
#[derive(Debug, Clone)]
pub enum SearchPattern {
    Exact(String),
    Wildcard(String),
    Id(i64),
}

impl SearchPattern {
    /// Check if a value matches this pattern
    pub fn matches(&self, value: &str) -> bool {
        match self {
            Self::Exact(pattern) => value.to_lowercase() == *pattern,
            Self::Wildcard(pattern) => value.to_lowercase().contains(pattern),
            Self::Id(_) => false, // ID matching handled separately
        }
    }
}

/// Filter preset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterPreset {
    pub id: String,
    pub name: String,
    pub query: UserListQuery,
    pub is_system: bool,
}

impl FilterPreset {
    pub fn new(id: &str, name: &str, query: UserListQuery) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            query,
            is_system: false,
        }
    }

    pub fn system(mut self) -> Self {
        self.is_system = true;
        self
    }
}

/// Default filter presets
pub fn default_filter_presets() -> Vec<FilterPreset> {
    vec![
        FilterPreset::new(
            "all",
            "All Users",
            UserListQuery::new()
        ).system(),
        FilterPreset::new(
            "administrators",
            "Administrators",
            UserListQuery::new().role("administrator")
        ).system(),
        FilterPreset::new(
            "pending",
            "Pending",
            UserListQuery::new().status(UserStatus::Pending)
        ).system(),
        FilterPreset::new(
            "suspended",
            "Suspended",
            UserListQuery::new().status(UserStatus::Suspended)
        ).system(),
        FilterPreset::new(
            "recent",
            "Recently Registered",
            UserListQuery::new().order_by(UserOrderBy::RegisteredDate).order(SortOrder::Desc)
        ).system(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_list_query() {
        let query = UserListQuery::new()
            .search("john")
            .role("editor")
            .page(2)
            .per_page(10);

        assert_eq!(query.search, Some("john".to_string()));
        assert_eq!(query.role, Some("editor".to_string()));
        assert_eq!(query.page, 2);
        assert_eq!(query.per_page, 10);
        assert_eq!(query.offset(), 10);
    }

    #[test]
    fn test_pagination() {
        let users = vec![];
        let result = UserListResult::new(
            users,
            45,
            UserListQuery::new().page(2).per_page(10)
        );

        assert_eq!(result.total_pages, 5);
        assert!(result.has_next_page());
        assert!(result.has_prev_page());
    }

    #[test]
    fn test_bulk_validation() {
        let executor = BulkExecutor::new();

        let request = BulkActionRequest::new(
            BulkAction::Delete,
            vec![2, 3, 4],
            5
        );

        assert!(executor.validate(&request).is_ok());

        // Trying to delete protected user
        let request = BulkActionRequest::new(
            BulkAction::Delete,
            vec![1], // Protected
            5
        );

        assert!(executor.validate(&request).is_err());
    }

    #[test]
    fn test_search_pattern() {
        let search = UserSearch::new();

        assert!(matches!(search.build_pattern("john"), SearchPattern::Wildcard(_)));
        assert!(matches!(search.build_pattern("john@example.com"), SearchPattern::Exact(_)));
        assert!(matches!(search.build_pattern("123"), SearchPattern::Id(123)));
    }

    #[test]
    fn test_pattern_matching() {
        let pattern = SearchPattern::Wildcard("john".to_string());
        assert!(pattern.matches("John Doe"));
        assert!(pattern.matches("johnny"));
        assert!(!pattern.matches("Jane Doe"));

        let exact = SearchPattern::Exact("john@example.com".to_string());
        assert!(exact.matches("John@Example.com"));
        assert!(!exact.matches("john@test.com"));
    }
}
