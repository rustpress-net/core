//! Delegated Authentication / Impersonation (Point 79)
//!
//! Allows administrators to impersonate users for support and debugging
//! with full audit trail.

use chrono::{DateTime, Duration, Utc};
use rustpress_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

/// Impersonation session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpersonationSession {
    pub id: Uuid,
    /// The admin/support user doing the impersonation
    pub impersonator_id: Uuid,
    /// The user being impersonated
    pub target_user_id: Uuid,
    /// Reason for impersonation
    pub reason: String,
    /// Original session ID (to restore)
    pub original_session_id: Option<Uuid>,
    /// Impersonation session token
    pub token_hash: String,
    /// IP address of impersonator
    pub ip_address: String,
    /// User agent
    pub user_agent: Option<String>,
    /// Restrictions on impersonation
    pub restrictions: ImpersonationRestrictions,
    /// When impersonation started
    pub started_at: DateTime<Utc>,
    /// When impersonation should end
    pub expires_at: DateTime<Utc>,
    /// When impersonation actually ended
    pub ended_at: Option<DateTime<Utc>>,
    /// How it ended
    pub end_reason: Option<ImpersonationEndReason>,
}

impl ImpersonationSession {
    pub fn is_active(&self) -> bool {
        self.ended_at.is_none() && Utc::now() < self.expires_at
    }
}

/// Restrictions during impersonation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImpersonationRestrictions {
    /// Cannot change password
    pub block_password_change: bool,
    /// Cannot change email
    pub block_email_change: bool,
    /// Cannot make purchases/payments
    pub block_payments: bool,
    /// Cannot delete account
    pub block_account_deletion: bool,
    /// Cannot view sensitive data (SSN, full credit card, etc)
    pub block_sensitive_data: bool,
    /// Read-only mode
    pub read_only: bool,
    /// Allowed paths (if empty, all paths allowed minus blocks)
    pub allowed_paths: Vec<String>,
    /// Blocked paths
    pub blocked_paths: Vec<String>,
}

impl ImpersonationRestrictions {
    pub fn standard() -> Self {
        Self {
            block_password_change: true,
            block_email_change: true,
            block_payments: true,
            block_account_deletion: true,
            block_sensitive_data: true,
            read_only: false,
            allowed_paths: Vec::new(),
            blocked_paths: vec![
                "/api/account/delete".to_string(),
                "/api/account/password".to_string(),
                "/api/payments/*".to_string(),
            ],
        }
    }

    pub fn read_only() -> Self {
        Self {
            read_only: true,
            ..Self::standard()
        }
    }

    pub fn is_path_allowed(&self, path: &str) -> bool {
        // Check blocked paths first
        for blocked in &self.blocked_paths {
            if blocked.ends_with('*') {
                if path.starts_with(&blocked[..blocked.len() - 1]) {
                    return false;
                }
            } else if path == blocked {
                return false;
            }
        }

        // If allowed paths specified, check them
        if !self.allowed_paths.is_empty() {
            for allowed in &self.allowed_paths {
                if allowed.ends_with('*') {
                    if path.starts_with(&allowed[..allowed.len() - 1]) {
                        return true;
                    }
                } else if path == allowed {
                    return true;
                }
            }
            return false;
        }

        true
    }

    pub fn is_method_allowed(&self, method: &str) -> bool {
        if self.read_only {
            matches!(method.to_uppercase().as_str(), "GET" | "HEAD" | "OPTIONS")
        } else {
            true
        }
    }
}

/// How impersonation ended
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImpersonationEndReason {
    /// Admin ended impersonation manually
    ManualEnd,
    /// Session timed out
    Timeout,
    /// Admin session ended
    AdminSessionEnded,
    /// Security concern forced end
    SecurityForced,
    /// Target user changed password
    PasswordChanged,
}

/// Impersonation configuration
#[derive(Debug, Clone)]
pub struct ImpersonationConfig {
    /// Maximum duration for impersonation
    pub max_duration: Duration,
    /// Default duration
    pub default_duration: Duration,
    /// Require reason
    pub require_reason: bool,
    /// Minimum reason length
    pub min_reason_length: usize,
    /// Roles that can impersonate
    pub allowed_impersonator_roles: Vec<String>,
    /// Roles that cannot be impersonated
    pub protected_roles: Vec<String>,
    /// Show impersonation banner to target user
    pub show_banner: bool,
    /// Notify target user
    pub notify_target: bool,
}

impl Default for ImpersonationConfig {
    fn default() -> Self {
        Self {
            max_duration: Duration::hours(2),
            default_duration: Duration::minutes(30),
            require_reason: true,
            min_reason_length: 10,
            allowed_impersonator_roles: vec!["administrator".to_string(), "support".to_string()],
            protected_roles: vec!["super_admin".to_string()],
            show_banner: true,
            notify_target: true,
        }
    }
}

/// Impersonation storage trait
#[async_trait::async_trait]
pub trait ImpersonationStore: Send + Sync {
    /// Create impersonation session
    async fn create(&self, session: &ImpersonationSession) -> Result<()>;

    /// Get session by token
    async fn get_by_token(&self, token_hash: &str) -> Result<Option<ImpersonationSession>>;

    /// Get session by ID
    async fn get(&self, id: Uuid) -> Result<Option<ImpersonationSession>>;

    /// Get active session for impersonator
    async fn get_active_for_impersonator(&self, impersonator_id: Uuid) -> Result<Option<ImpersonationSession>>;

    /// Get active sessions for target user
    async fn get_active_for_target(&self, target_user_id: Uuid) -> Result<Vec<ImpersonationSession>>;

    /// End session
    async fn end(&self, id: Uuid, reason: ImpersonationEndReason) -> Result<()>;

    /// End all sessions for impersonator
    async fn end_all_for_impersonator(&self, impersonator_id: Uuid, reason: ImpersonationEndReason) -> Result<u64>;

    /// Get impersonation history for target
    async fn get_history(&self, target_user_id: Uuid, limit: usize) -> Result<Vec<ImpersonationSession>>;

    /// Cleanup expired sessions
    async fn cleanup_expired(&self) -> Result<u64>;
}

/// User role checker trait
#[async_trait::async_trait]
pub trait UserRoleChecker: Send + Sync {
    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<String>>;
}

/// Impersonation manager
pub struct ImpersonationManager<S: ImpersonationStore, R: UserRoleChecker> {
    store: S,
    role_checker: R,
    config: ImpersonationConfig,
}

impl<S: ImpersonationStore, R: UserRoleChecker> ImpersonationManager<S, R> {
    pub fn new(store: S, role_checker: R, config: ImpersonationConfig) -> Self {
        Self {
            store,
            role_checker,
            config,
        }
    }

    /// Generate impersonation token
    fn generate_token() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        base64_url_encode(&bytes)
    }

    /// Hash a token
    fn hash_token(token: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Start impersonating a user
    pub async fn start(
        &self,
        impersonator_id: Uuid,
        target_user_id: Uuid,
        reason: String,
        ip_address: String,
        user_agent: Option<String>,
        duration: Option<Duration>,
        restrictions: Option<ImpersonationRestrictions>,
    ) -> Result<(String, ImpersonationSession)> {
        // Validate reason
        if self.config.require_reason && reason.len() < self.config.min_reason_length {
            return Err(Error::InvalidInput {
                field: "reason".to_string(),
                message: format!("Reason must be at least {} characters", self.config.min_reason_length),
            });
        }

        // Check impersonator has permission
        let impersonator_roles = self.role_checker.get_user_roles(impersonator_id).await?;
        let can_impersonate = impersonator_roles.iter().any(|r| {
            self.config.allowed_impersonator_roles.contains(r)
        });
        if !can_impersonate {
            return Err(Error::Authorization {
                action: "You don't have permission to impersonate users".to_string(),
                required: "impersonate".to_string(),
            });
        }

        // Check target is not protected
        let target_roles = self.role_checker.get_user_roles(target_user_id).await?;
        let is_protected = target_roles.iter().any(|r| {
            self.config.protected_roles.contains(r)
        });
        if is_protected {
            return Err(Error::Authorization {
                action: "This user cannot be impersonated".to_string(),
                required: "impersonate:protected".to_string(),
            });
        }

        // Cannot impersonate self
        if impersonator_id == target_user_id {
            return Err(Error::InvalidInput {
                field: "target_user_id".to_string(),
                message: "Cannot impersonate yourself".to_string(),
            });
        }

        // Check for existing active session
        if let Some(_existing) = self.store.get_active_for_impersonator(impersonator_id).await? {
            return Err(Error::InvalidInput {
                field: "impersonator_id".to_string(),
                message: "You already have an active impersonation session".to_string(),
            });
        }

        let token = Self::generate_token();
        let token_hash = Self::hash_token(&token);
        let now = Utc::now();
        let duration = duration.unwrap_or(self.config.default_duration);
        let capped_duration = if duration > self.config.max_duration {
            self.config.max_duration
        } else {
            duration
        };

        let session = ImpersonationSession {
            id: Uuid::now_v7(),
            impersonator_id,
            target_user_id,
            reason,
            original_session_id: None,
            token_hash,
            ip_address,
            user_agent,
            restrictions: restrictions.unwrap_or_else(ImpersonationRestrictions::standard),
            started_at: now,
            expires_at: now + capped_duration,
            ended_at: None,
            end_reason: None,
        };

        self.store.create(&session).await?;

        Ok((token, session))
    }

    /// Validate impersonation token
    pub async fn validate(&self, token: &str) -> Result<ImpersonationSession> {
        let token_hash = Self::hash_token(token);
        let session = self.store
            .get_by_token(&token_hash)
            .await?
            .ok_or_else(|| Error::Authentication {
                message: "Invalid impersonation token".to_string(),
            })?;

        if !session.is_active() {
            return Err(Error::Authentication {
                message: "Impersonation session expired or ended".to_string(),
            });
        }

        Ok(session)
    }

    /// Check if request is allowed during impersonation
    pub async fn check_request(
        &self,
        token: &str,
        path: &str,
        method: &str,
    ) -> Result<ImpersonationSession> {
        let session = self.validate(token).await?;

        if !session.restrictions.is_path_allowed(path) {
            return Err(Error::Authorization {
                action: "This action is not allowed during impersonation".to_string(),
                required: "impersonation:path_allowed".to_string(),
            });
        }

        if !session.restrictions.is_method_allowed(method) {
            return Err(Error::Authorization {
                action: "Write operations are not allowed during impersonation".to_string(),
                required: "impersonation:write".to_string(),
            });
        }

        Ok(session)
    }

    /// End impersonation
    pub async fn end(&self, token: &str) -> Result<()> {
        let session = self.validate(token).await?;
        self.store.end(session.id, ImpersonationEndReason::ManualEnd).await
    }

    /// End impersonation by ID
    pub async fn end_by_id(&self, id: Uuid, reason: ImpersonationEndReason) -> Result<()> {
        self.store.end(id, reason).await
    }

    /// End all impersonation sessions for a user being impersonated
    pub async fn end_all_for_target(&self, target_user_id: Uuid, reason: ImpersonationEndReason) -> Result<u64> {
        let sessions = self.store.get_active_for_target(target_user_id).await?;
        let mut count = 0;
        for session in sessions {
            self.store.end(session.id, reason).await?;
            count += 1;
        }
        Ok(count)
    }

    /// Get impersonation history for a user
    pub async fn get_history(&self, target_user_id: Uuid, limit: usize) -> Result<Vec<ImpersonationSession>> {
        self.store.get_history(target_user_id, limit).await
    }

    /// Check if a user is currently being impersonated
    pub async fn is_being_impersonated(&self, user_id: Uuid) -> Result<bool> {
        let sessions = self.store.get_active_for_target(user_id).await?;
        Ok(!sessions.is_empty())
    }

    /// Get config
    pub fn config(&self) -> &ImpersonationConfig {
        &self.config
    }
}

/// In-memory impersonation store
pub struct InMemoryImpersonationStore {
    sessions: RwLock<HashMap<Uuid, ImpersonationSession>>,
}

impl InMemoryImpersonationStore {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryImpersonationStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ImpersonationStore for InMemoryImpersonationStore {
    async fn create(&self, session: &ImpersonationSession) -> Result<()> {
        let mut sessions = self.sessions.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        sessions.insert(session.id, session.clone());
        Ok(())
    }

    async fn get_by_token(&self, token_hash: &str) -> Result<Option<ImpersonationSession>> {
        let sessions = self.sessions.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(sessions.values().find(|s| s.token_hash == token_hash).cloned())
    }

    async fn get(&self, id: Uuid) -> Result<Option<ImpersonationSession>> {
        let sessions = self.sessions.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(sessions.get(&id).cloned())
    }

    async fn get_active_for_impersonator(&self, impersonator_id: Uuid) -> Result<Option<ImpersonationSession>> {
        let sessions = self.sessions.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(sessions
            .values()
            .find(|s| s.impersonator_id == impersonator_id && s.is_active())
            .cloned())
    }

    async fn get_active_for_target(&self, target_user_id: Uuid) -> Result<Vec<ImpersonationSession>> {
        let sessions = self.sessions.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(sessions
            .values()
            .filter(|s| s.target_user_id == target_user_id && s.is_active())
            .cloned()
            .collect())
    }

    async fn end(&self, id: Uuid, reason: ImpersonationEndReason) -> Result<()> {
        let mut sessions = self.sessions.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;

        if let Some(session) = sessions.get_mut(&id) {
            session.ended_at = Some(Utc::now());
            session.end_reason = Some(reason);
        }
        Ok(())
    }

    async fn end_all_for_impersonator(&self, impersonator_id: Uuid, reason: ImpersonationEndReason) -> Result<u64> {
        let mut sessions = self.sessions.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;

        let now = Utc::now();
        let mut count = 0;
        for session in sessions.values_mut() {
            if session.impersonator_id == impersonator_id && session.is_active() {
                session.ended_at = Some(now);
                session.end_reason = Some(reason);
                count += 1;
            }
        }
        Ok(count)
    }

    async fn get_history(&self, target_user_id: Uuid, limit: usize) -> Result<Vec<ImpersonationSession>> {
        let sessions = self.sessions.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;

        let mut history: Vec<_> = sessions
            .values()
            .filter(|s| s.target_user_id == target_user_id)
            .cloned()
            .collect();
        history.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        history.truncate(limit);
        Ok(history)
    }

    async fn cleanup_expired(&self) -> Result<u64> {
        let mut sessions = self.sessions.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;

        let now = Utc::now();
        let before = sessions.len();
        sessions.retain(|_, s| {
            // Keep active sessions and recently ended ones
            s.is_active() || s.ended_at.map_or(false, |e| now.signed_duration_since(e).num_days() < 30)
        });
        Ok((before - sessions.len()) as u64)
    }
}

/// Simple role checker for testing
pub struct SimpleRoleChecker {
    roles: RwLock<HashMap<Uuid, Vec<String>>>,
}

impl SimpleRoleChecker {
    pub fn new() -> Self {
        Self {
            roles: RwLock::new(HashMap::new()),
        }
    }

    pub fn set_roles(&self, user_id: Uuid, roles: Vec<String>) -> Result<()> {
        let mut r = self.roles.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        r.insert(user_id, roles);
        Ok(())
    }
}

impl Default for SimpleRoleChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl UserRoleChecker for SimpleRoleChecker {
    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<String>> {
        let roles = self.roles.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(roles.get(&user_id).cloned().unwrap_or_default())
    }
}

/// Base64 URL-safe encoding
fn base64_url_encode(bytes: &[u8]) -> String {
    base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_start_impersonation() {
        let store = InMemoryImpersonationStore::new();
        let role_checker = SimpleRoleChecker::new();

        let admin_id = Uuid::now_v7();
        let user_id = Uuid::now_v7();

        role_checker.set_roles(admin_id, vec!["administrator".to_string()]).unwrap();
        role_checker.set_roles(user_id, vec!["subscriber".to_string()]).unwrap();

        let manager = ImpersonationManager::new(store, role_checker, ImpersonationConfig::default());

        let (token, session) = manager
            .start(
                admin_id,
                user_id,
                "Testing user issue #12345".to_string(),
                "192.168.1.1".to_string(),
                None,
                None,
                None,
            )
            .await
            .unwrap();

        assert!(!token.is_empty());
        assert!(session.is_active());
        assert_eq!(session.impersonator_id, admin_id);
        assert_eq!(session.target_user_id, user_id);
    }

    #[tokio::test]
    async fn test_cannot_impersonate_protected() {
        let store = InMemoryImpersonationStore::new();
        let role_checker = SimpleRoleChecker::new();

        let admin_id = Uuid::now_v7();
        let super_admin_id = Uuid::now_v7();

        role_checker.set_roles(admin_id, vec!["administrator".to_string()]).unwrap();
        role_checker.set_roles(super_admin_id, vec!["super_admin".to_string()]).unwrap();

        let manager = ImpersonationManager::new(store, role_checker, ImpersonationConfig::default());

        let result = manager
            .start(
                admin_id,
                super_admin_id,
                "Testing super admin".to_string(),
                "192.168.1.1".to_string(),
                None,
                None,
                None,
            )
            .await;

        assert!(result.is_err());
    }

    #[test]
    fn test_restrictions() {
        let restrictions = ImpersonationRestrictions::standard();

        assert!(!restrictions.is_path_allowed("/api/account/delete"));
        assert!(!restrictions.is_path_allowed("/api/payments/charge"));
        assert!(restrictions.is_path_allowed("/api/posts"));

        let read_only = ImpersonationRestrictions::read_only();
        assert!(read_only.is_method_allowed("GET"));
        assert!(!read_only.is_method_allowed("POST"));
    }

    #[tokio::test]
    async fn test_end_impersonation() {
        let store = InMemoryImpersonationStore::new();
        let role_checker = SimpleRoleChecker::new();

        let admin_id = Uuid::now_v7();
        let user_id = Uuid::now_v7();

        role_checker.set_roles(admin_id, vec!["administrator".to_string()]).unwrap();

        let manager = ImpersonationManager::new(store, role_checker, ImpersonationConfig::default());

        let (token, _) = manager
            .start(
                admin_id,
                user_id,
                "Support request".to_string(),
                "192.168.1.1".to_string(),
                None,
                None,
                None,
            )
            .await
            .unwrap();

        // End impersonation
        manager.end(&token).await.unwrap();

        // Should no longer be valid
        let result = manager.validate(&token).await;
        assert!(result.is_err());
    }
}
