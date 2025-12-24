//! User service for handling user-related business logic.

use chrono::{DateTime, Utc};
use rustpress_core::error::{Error, Result};
use rustpress_core::service::SortOrder;
use rustpress_database::repository::users::{UserRepository, UserRow};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// User status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    Pending,
    Active,
    Suspended,
    Deleted,
}

impl Default for UserStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl std::fmt::Display for UserStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Active => write!(f, "active"),
            Self::Suspended => write!(f, "suspended"),
            Self::Deleted => write!(f, "deleted"),
        }
    }
}

impl std::str::FromStr for UserStatus {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(Self::Pending),
            "active" => Ok(Self::Active),
            "suspended" => Ok(Self::Suspended),
            "deleted" => Ok(Self::Deleted),
            _ => Err(format!("Invalid user status: {}", s)),
        }
    }
}

/// User role enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Subscriber,
    Contributor,
    Author,
    Editor,
    Administrator,
}

impl Default for UserRole {
    fn default() -> Self {
        Self::Subscriber
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Subscriber => write!(f, "subscriber"),
            Self::Contributor => write!(f, "contributor"),
            Self::Author => write!(f, "author"),
            Self::Editor => write!(f, "editor"),
            Self::Administrator => write!(f, "administrator"),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "subscriber" => Ok(Self::Subscriber),
            "contributor" => Ok(Self::Contributor),
            "author" => Ok(Self::Author),
            "editor" => Ok(Self::Editor),
            "administrator" | "admin" => Ok(Self::Administrator),
            _ => Err(format!("Invalid user role: {}", s)),
        }
    }
}

/// User response for API (excluding sensitive data)
#[derive(Debug, Clone, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub display_name: Option<String>,
    pub status: String,
    pub role: String,
    pub avatar_url: Option<String>,
    pub locale: Option<String>,
    pub timezone: Option<String>,
    pub email_verified: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Paginated users response
#[derive(Debug, Clone, Serialize)]
pub struct UsersListResponse {
    pub users: Vec<UserResponse>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

/// Create user request
#[derive(Debug, Clone, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub username: String,
    pub password: String,
    pub display_name: Option<String>,
    pub role: Option<String>,
    pub locale: Option<String>,
    pub timezone: Option<String>,
}

/// Update user request
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub status: Option<String>,
    pub role: Option<String>,
    pub avatar_url: Option<String>,
    pub locale: Option<String>,
    pub timezone: Option<String>,
}

/// Update password request
#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePasswordRequest {
    pub current_password: Option<String>,
    pub new_password: String,
}

/// User list query parameters
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UserListParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub status: Option<String>,
    pub role: Option<String>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

impl From<UserRow> for UserResponse {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id,
            email: row.email,
            username: row.username,
            display_name: row.display_name,
            status: row.status,
            role: row.role,
            avatar_url: row.avatar_url,
            locale: row.locale,
            timezone: row.timezone,
            email_verified: row.email_verified_at.is_some(),
            last_login_at: row.last_login_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// User service for handling user operations
#[derive(Clone)]
pub struct UserService {
    pool: PgPool,
}

impl UserService {
    /// Create a new user service
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get repository instance
    fn repo(&self) -> UserRepository {
        UserRepository::new(self.pool.clone())
    }

    /// Create a new user
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<UserResponse> {
        // Validate email
        if !self.is_valid_email(&request.email) {
            return Err(Error::validation("Invalid email address"));
        }

        // Validate username
        if request.username.len() < 3 || request.username.len() > 50 {
            return Err(Error::validation("Username must be between 3 and 50 characters"));
        }

        if !request.username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(Error::validation("Username can only contain letters, numbers, underscores, and hyphens"));
        }

        // Validate password
        if request.password.len() < 8 {
            return Err(Error::validation("Password must be at least 8 characters"));
        }

        // Check if email is already taken
        if self.repo().find_by_email(&request.email).await?.is_some() {
            return Err(Error::validation("Email is already registered"));
        }

        // Check if username is already taken
        if self.repo().find_by_username(&request.username).await?.is_some() {
            return Err(Error::validation("Username is already taken"));
        }

        // Hash password
        let password_hash = self.hash_password(&request.password)?;

        let now = Utc::now();
        let user = UserRow {
            id: Uuid::now_v7(),
            email: request.email.to_lowercase(),
            username: request.username,
            password_hash,
            display_name: request.display_name,
            status: "pending".to_string(),
            role: request.role.unwrap_or_else(|| "subscriber".to_string()),
            avatar_url: None,
            locale: request.locale,
            timezone: request.timezone,
            email_verified_at: None,
            last_login_at: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created = self.repo().create(&user).await?;
        Ok(UserResponse::from(created))
    }

    /// List users with pagination and filtering
    pub async fn list_users(&self, params: UserListParams) -> Result<UsersListResponse> {
        let page = params.page.unwrap_or(1).max(1);
        let per_page = params.per_page.unwrap_or(20).min(100).max(1);

        let sort_order = match params.sort_order.as_deref() {
            Some("asc") => SortOrder::Asc,
            _ => SortOrder::Desc,
        };

        let mut conditions = vec!["deleted_at IS NULL".to_string()];

        if let Some(ref status) = params.status {
            conditions.push(format!("status = '{}'", status.replace('\'', "''")));
        }

        if let Some(ref role) = params.role {
            conditions.push(format!("role = '{}'", role.replace('\'', "''")));
        }

        if let Some(ref search) = params.search {
            let escaped = search.replace('\'', "''");
            conditions.push(format!(
                "(username ILIKE '%{}%' OR email ILIKE '%{}%' OR display_name ILIKE '%{}%')",
                escaped, escaped, escaped
            ));
        }

        let where_clause = conditions.join(" AND ");
        let order_by = params.sort_by.as_deref().unwrap_or("created_at");
        let order_dir = if sort_order == SortOrder::Desc { "DESC" } else { "ASC" };

        // Count query
        let count_query = format!(
            "SELECT COUNT(*) as count FROM users WHERE {}",
            where_clause
        );
        let total: (i64,) = sqlx::query_as(&count_query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to count users", e))?;

        // Data query
        let offset = (page - 1) * per_page;
        let data_query = format!(
            "SELECT * FROM users WHERE {} ORDER BY {} {} LIMIT {} OFFSET {}",
            where_clause, order_by, order_dir, per_page, offset
        );

        let rows: Vec<UserRow> = sqlx::query_as(&data_query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to list users", e))?;

        let users: Vec<UserResponse> = rows.into_iter().map(UserResponse::from).collect();
        let total_pages = ((total.0 as f64) / (per_page as f64)).ceil() as u64;

        Ok(UsersListResponse {
            users,
            total: total.0 as u64,
            page: page.into(),
            per_page: per_page.into(),
            total_pages,
        })
    }

    /// Get a user by ID
    pub async fn get_user(&self, id: Uuid) -> Result<Option<UserResponse>> {
        let user = self.find_by_id(id).await?;
        Ok(user.map(UserResponse::from))
    }

    /// Get a user by email
    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<UserResponse>> {
        let user = self.repo().find_by_email(email).await?;
        Ok(user.map(UserResponse::from))
    }

    /// Get a user by username
    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<UserResponse>> {
        let user = self.repo().find_by_username(username).await?;
        Ok(user.map(UserResponse::from))
    }

    /// Update a user
    pub async fn update_user(&self, id: Uuid, request: UpdateUserRequest) -> Result<UserResponse> {
        let existing = self.find_by_id(id).await?
            .ok_or_else(|| Error::not_found("User", id.to_string()))?;

        // Check email uniqueness if changed
        if let Some(ref new_email) = request.email {
            if new_email.to_lowercase() != existing.email.to_lowercase() {
                if !self.is_valid_email(new_email) {
                    return Err(Error::validation("Invalid email address"));
                }
                if self.repo().find_by_email(new_email).await?.is_some() {
                    return Err(Error::validation("Email is already registered"));
                }
            }
        }

        // Check username uniqueness if changed
        if let Some(ref new_username) = request.username {
            if new_username != &existing.username {
                if self.repo().find_by_username(new_username).await?.is_some() {
                    return Err(Error::validation("Username is already taken"));
                }
            }
        }

        // Build update query
        let email = request.email.map(|e| e.to_lowercase()).unwrap_or(existing.email);
        let username = request.username.unwrap_or(existing.username);

        let query = r#"
            UPDATE users SET
                email = $2,
                username = $3,
                display_name = $4,
                status = $5,
                role = $6,
                avatar_url = $7,
                locale = $8,
                timezone = $9,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
        "#;

        let updated: UserRow = sqlx::query_as(query)
            .bind(id)
            .bind(email)
            .bind(username)
            .bind(request.display_name.or(existing.display_name))
            .bind(request.status.unwrap_or(existing.status))
            .bind(request.role.unwrap_or(existing.role))
            .bind(request.avatar_url.or(existing.avatar_url))
            .bind(request.locale.or(existing.locale))
            .bind(request.timezone.or(existing.timezone))
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to update user", e))?;

        Ok(UserResponse::from(updated))
    }

    /// Update user password
    pub async fn update_password(
        &self,
        id: Uuid,
        request: UpdatePasswordRequest,
        require_current: bool,
    ) -> Result<()> {
        let existing = self.find_by_id(id).await?
            .ok_or_else(|| Error::not_found("User", id.to_string()))?;

        // Verify current password if required
        if require_current {
            let current = request.current_password
                .ok_or_else(|| Error::validation("Current password is required"))?;

            if !self.verify_password(&current, &existing.password_hash)? {
                return Err(Error::validation("Current password is incorrect"));
            }
        }

        // Validate new password
        if request.new_password.len() < 8 {
            return Err(Error::validation("Password must be at least 8 characters"));
        }

        // Hash new password
        let password_hash = self.hash_password(&request.new_password)?;

        sqlx::query("UPDATE users SET password_hash = $2, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(password_hash)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to update password", e))?;

        Ok(())
    }

    /// Delete a user (soft delete)
    pub async fn delete_user(&self, id: Uuid) -> Result<bool> {
        let _ = self.find_by_id(id).await?
            .ok_or_else(|| Error::not_found("User", id.to_string()))?;

        sqlx::query("UPDATE users SET deleted_at = NOW(), status = 'deleted', updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to delete user", e))?;

        Ok(true)
    }

    /// Suspend a user
    pub async fn suspend_user(&self, id: Uuid) -> Result<UserResponse> {
        let _ = self.find_by_id(id).await?
            .ok_or_else(|| Error::not_found("User", id.to_string()))?;

        let updated: UserRow = sqlx::query_as(
            "UPDATE users SET status = 'suspended', updated_at = NOW() WHERE id = $1 RETURNING *"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to suspend user", e))?;

        Ok(UserResponse::from(updated))
    }

    /// Activate a user
    pub async fn activate_user(&self, id: Uuid) -> Result<UserResponse> {
        let _ = self.find_by_id(id).await?
            .ok_or_else(|| Error::not_found("User", id.to_string()))?;

        let updated: UserRow = sqlx::query_as(
            "UPDATE users SET status = 'active', updated_at = NOW() WHERE id = $1 RETURNING *"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to activate user", e))?;

        Ok(UserResponse::from(updated))
    }

    /// Verify email
    pub async fn verify_email(&self, id: Uuid) -> Result<()> {
        self.repo().verify_email(id).await
    }

    /// Get counts by role
    pub async fn get_counts_by_role(&self) -> Result<std::collections::HashMap<String, i64>> {
        let query = r#"
            SELECT role, COUNT(*) as count
            FROM users
            WHERE deleted_at IS NULL
            GROUP BY role
        "#;

        let rows: Vec<(String, i64)> = sqlx::query_as(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to count users by role", e))?;

        Ok(rows.into_iter().collect())
    }

    // =====================
    // Helper methods
    // =====================

    async fn find_by_id(&self, id: Uuid) -> Result<Option<UserRow>> {
        let query = "SELECT * FROM users WHERE id = $1 AND deleted_at IS NULL";

        sqlx::query_as::<_, UserRow>(query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to find user", e))
    }

    fn is_valid_email(&self, email: &str) -> bool {
        // Basic email validation
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return false;
        }
        let local = parts[0];
        let domain = parts[1];

        if local.is_empty() || domain.is_empty() {
            return false;
        }

        if !domain.contains('.') {
            return false;
        }

        true
    }

    fn hash_password(&self, password: &str) -> Result<String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Note: In production, use bcrypt or argon2
        // This is a placeholder for demonstration
        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);
        let salt = Uuid::new_v4().to_string();
        salt.hash(&mut hasher);

        Ok(format!("hash:{:x}", hasher.finish()))
    }

    fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        // Note: In production, use proper password verification
        // This is a placeholder for demonstration
        if hash.starts_with("hash:") {
            // For demo purposes, we'll just return true
            // Real implementation would use bcrypt or argon2
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl Default for UserService {
    fn default() -> Self {
        panic!("UserService requires a database pool")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_status_display() {
        assert_eq!(UserStatus::Active.to_string(), "active");
        assert_eq!(UserStatus::Suspended.to_string(), "suspended");
    }

    #[test]
    fn test_user_role_from_str() {
        assert_eq!("admin".parse::<UserRole>().unwrap(), UserRole::Administrator);
        assert_eq!("editor".parse::<UserRole>().unwrap(), UserRole::Editor);
        assert!("invalid".parse::<UserRole>().is_err());
    }

    #[test]
    fn test_email_validation() {
        let service = UserService {
            pool: unsafe { std::mem::zeroed() },
        };

        assert!(service.is_valid_email("test@example.com"));
        assert!(service.is_valid_email("user.name@domain.org"));
        assert!(!service.is_valid_email("invalid"));
        assert!(!service.is_valid_email("@domain.com"));
        assert!(!service.is_valid_email("user@"));
    }
}
