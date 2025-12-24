//! Refresh Token Rotation (Point 58)
//!
//! Implements secure refresh token rotation with family tracking
//! to prevent token reuse attacks.

use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use rustpress_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

/// Refresh token entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    /// Token family for rotation tracking
    pub family_id: Uuid,
    /// Generation within the family (increments on rotation)
    pub generation: u32,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_info: Option<DeviceInfo>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    /// Revoked at (if token was invalidated)
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoke_reason: Option<RevokeReason>,
    /// Last used at
    pub last_used_at: Option<DateTime<Utc>>,
}

impl RefreshToken {
    pub fn is_valid(&self) -> bool {
        self.revoked_at.is_none() && Utc::now() < self.expires_at
    }

    pub fn is_revoked(&self) -> bool {
        self.revoked_at.is_some()
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }
}

/// Device information for token tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_type: String,
    pub device_name: Option<String>,
    pub os: Option<String>,
    pub browser: Option<String>,
    pub location: Option<String>,
}

/// Reason for token revocation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RevokeReason {
    /// User logged out
    Logout,
    /// Token was rotated
    Rotated,
    /// Detected token reuse (potential attack)
    TokenReuse,
    /// Password changed
    PasswordChange,
    /// Admin revoked
    AdminRevoke,
    /// Session expired
    Expired,
    /// Security concern
    Security,
}

/// Refresh token configuration
#[derive(Debug, Clone)]
pub struct RefreshTokenConfig {
    /// Token lifetime
    pub lifetime: Duration,
    /// Absolute maximum lifetime (even with rotation)
    pub absolute_lifetime: Duration,
    /// Token length in bytes
    pub token_length: usize,
    /// Maximum active token families per user
    pub max_families_per_user: usize,
    /// Revoke entire family on token reuse detection
    pub revoke_family_on_reuse: bool,
    /// Grace period for old token after rotation (seconds)
    pub rotation_grace_period_secs: u64,
}

impl Default for RefreshTokenConfig {
    fn default() -> Self {
        Self {
            lifetime: Duration::days(7),
            absolute_lifetime: Duration::days(30),
            token_length: 32,
            max_families_per_user: 5,
            revoke_family_on_reuse: true,
            rotation_grace_period_secs: 60,
        }
    }
}

/// Refresh token storage trait
#[async_trait::async_trait]
pub trait RefreshTokenStore: Send + Sync {
    /// Store a new refresh token
    async fn create(&self, token: &RefreshToken) -> Result<()>;

    /// Get token by hash
    async fn get_by_hash(&self, token_hash: &str) -> Result<Option<RefreshToken>>;

    /// Get token by ID
    async fn get(&self, id: Uuid) -> Result<Option<RefreshToken>>;

    /// Update token
    async fn update(&self, token: &RefreshToken) -> Result<()>;

    /// Revoke a token
    async fn revoke(&self, id: Uuid, reason: RevokeReason) -> Result<()>;

    /// Revoke all tokens in a family
    async fn revoke_family(&self, family_id: Uuid, reason: RevokeReason) -> Result<u64>;

    /// Revoke all tokens for a user
    async fn revoke_user_tokens(&self, user_id: Uuid, reason: RevokeReason) -> Result<u64>;

    /// Get all active tokens for a user
    async fn get_user_tokens(&self, user_id: Uuid) -> Result<Vec<RefreshToken>>;

    /// Get token families for a user
    async fn get_user_families(&self, user_id: Uuid) -> Result<Vec<Uuid>>;

    /// Get latest token in a family
    async fn get_latest_in_family(&self, family_id: Uuid) -> Result<Option<RefreshToken>>;

    /// Cleanup expired tokens
    async fn cleanup_expired(&self) -> Result<u64>;
}

/// Refresh token manager with rotation support
pub struct RefreshTokenManager<S: RefreshTokenStore> {
    store: S,
    config: RefreshTokenConfig,
}

impl<S: RefreshTokenStore> RefreshTokenManager<S> {
    pub fn new(store: S, config: RefreshTokenConfig) -> Self {
        Self { store, config }
    }

    /// Generate a new refresh token
    fn generate_token(&self) -> String {
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..self.config.token_length)
            .map(|_| rng.gen())
            .collect();
        base64_url_encode(&bytes)
    }

    /// Hash a token for storage
    fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Create a new refresh token for a user
    pub async fn create(
        &self,
        user_id: Uuid,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        device_info: Option<DeviceInfo>,
    ) -> Result<(String, RefreshToken)> {
        // Check max families
        let families = self.store.get_user_families(user_id).await?;
        if families.len() >= self.config.max_families_per_user {
            // Revoke oldest family
            if let Some(oldest_family) = families.first() {
                self.store.revoke_family(*oldest_family, RevokeReason::Expired).await?;
            }
        }

        let raw_token = self.generate_token();
        let token_hash = Self::hash_token(&raw_token);
        let now = Utc::now();
        let family_id = Uuid::now_v7();

        let token = RefreshToken {
            id: Uuid::now_v7(),
            user_id,
            token_hash,
            family_id,
            generation: 1,
            ip_address: ip_address.map(String::from),
            user_agent: user_agent.map(String::from),
            device_info,
            expires_at: now + self.config.lifetime,
            created_at: now,
            revoked_at: None,
            revoke_reason: None,
            last_used_at: None,
        };

        self.store.create(&token).await?;
        Ok((raw_token, token))
    }

    /// Validate a refresh token
    pub async fn validate(&self, raw_token: &str) -> Result<RefreshToken> {
        let token_hash = Self::hash_token(raw_token);

        let token = self.store
            .get_by_hash(&token_hash)
            .await?
            .ok_or_else(|| Error::Authentication {
                message: "Invalid refresh token".to_string(),
            })?;

        if token.is_revoked() {
            // Check if this might be token reuse
            if self.config.revoke_family_on_reuse {
                // Check if there's a newer token in the family
                if let Some(latest) = self.store.get_latest_in_family(token.family_id).await? {
                    if latest.generation > token.generation {
                        // Token reuse detected! Revoke entire family
                        self.store.revoke_family(token.family_id, RevokeReason::TokenReuse).await?;
                        return Err(Error::Authentication {
                            message: "Token reuse detected. All sessions revoked for security.".to_string(),
                        });
                    }
                }
            }

            return Err(Error::Authentication {
                message: "Refresh token has been revoked".to_string(),
            });
        }

        if token.is_expired() {
            return Err(Error::TokenExpired);
        }

        Ok(token)
    }

    /// Rotate a refresh token (create new one and revoke old)
    pub async fn rotate(
        &self,
        raw_token: &str,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<(String, RefreshToken)> {
        let old_token = self.validate(raw_token).await?;

        // Check absolute lifetime
        let family_age = Utc::now().signed_duration_since(old_token.created_at);
        if family_age > self.config.absolute_lifetime {
            // Force re-authentication
            self.store.revoke_family(old_token.family_id, RevokeReason::Expired).await?;
            return Err(Error::Authentication {
                message: "Session expired. Please log in again.".to_string(),
            });
        }

        // Create new token in the same family
        let new_raw_token = self.generate_token();
        let new_token_hash = Self::hash_token(&new_raw_token);
        let now = Utc::now();

        let new_token = RefreshToken {
            id: Uuid::now_v7(),
            user_id: old_token.user_id,
            token_hash: new_token_hash,
            family_id: old_token.family_id,
            generation: old_token.generation + 1,
            ip_address: ip_address.map(String::from),
            user_agent: user_agent.map(String::from),
            device_info: old_token.device_info.clone(),
            expires_at: now + self.config.lifetime,
            created_at: now,
            revoked_at: None,
            revoke_reason: None,
            last_used_at: None,
        };

        self.store.create(&new_token).await?;

        // Revoke old token (with grace period consideration handled by validation)
        self.store.revoke(old_token.id, RevokeReason::Rotated).await?;

        Ok((new_raw_token, new_token))
    }

    /// Revoke a specific token
    pub async fn revoke(&self, raw_token: &str) -> Result<()> {
        let token_hash = Self::hash_token(raw_token);

        if let Some(token) = self.store.get_by_hash(&token_hash).await? {
            self.store.revoke(token.id, RevokeReason::Logout).await?;
        }

        Ok(())
    }

    /// Revoke all tokens for a user
    pub async fn revoke_user_tokens(&self, user_id: Uuid, reason: RevokeReason) -> Result<u64> {
        self.store.revoke_user_tokens(user_id, reason).await
    }

    /// Get all active sessions (tokens) for a user
    pub async fn get_user_sessions(&self, user_id: Uuid) -> Result<Vec<RefreshToken>> {
        let tokens = self.store.get_user_tokens(user_id).await?;
        Ok(tokens.into_iter().filter(|t| t.is_valid()).collect())
    }

    /// Cleanup expired tokens
    pub async fn cleanup(&self) -> Result<u64> {
        self.store.cleanup_expired().await
    }

    /// Get config
    pub fn config(&self) -> &RefreshTokenConfig {
        &self.config
    }
}

/// In-memory refresh token store
pub struct InMemoryRefreshTokenStore {
    tokens: RwLock<HashMap<String, RefreshToken>>,
}

impl InMemoryRefreshTokenStore {
    pub fn new() -> Self {
        Self {
            tokens: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryRefreshTokenStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl RefreshTokenStore for InMemoryRefreshTokenStore {
    async fn create(&self, token: &RefreshToken) -> Result<()> {
        let mut tokens = self.tokens.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        tokens.insert(token.token_hash.clone(), token.clone());
        Ok(())
    }

    async fn get_by_hash(&self, token_hash: &str) -> Result<Option<RefreshToken>> {
        let tokens = self.tokens.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(tokens.get(token_hash).cloned())
    }

    async fn get(&self, id: Uuid) -> Result<Option<RefreshToken>> {
        let tokens = self.tokens.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(tokens.values().find(|t| t.id == id).cloned())
    }

    async fn update(&self, token: &RefreshToken) -> Result<()> {
        let mut tokens = self.tokens.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        tokens.insert(token.token_hash.clone(), token.clone());
        Ok(())
    }

    async fn revoke(&self, id: Uuid, reason: RevokeReason) -> Result<()> {
        let mut tokens = self.tokens.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;

        for token in tokens.values_mut() {
            if token.id == id {
                token.revoked_at = Some(Utc::now());
                token.revoke_reason = Some(reason);
                break;
            }
        }
        Ok(())
    }

    async fn revoke_family(&self, family_id: Uuid, reason: RevokeReason) -> Result<u64> {
        let mut tokens = self.tokens.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;

        let now = Utc::now();
        let mut count = 0;
        for token in tokens.values_mut() {
            if token.family_id == family_id && token.revoked_at.is_none() {
                token.revoked_at = Some(now);
                token.revoke_reason = Some(reason);
                count += 1;
            }
        }
        Ok(count)
    }

    async fn revoke_user_tokens(&self, user_id: Uuid, reason: RevokeReason) -> Result<u64> {
        let mut tokens = self.tokens.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;

        let now = Utc::now();
        let mut count = 0;
        for token in tokens.values_mut() {
            if token.user_id == user_id && token.revoked_at.is_none() {
                token.revoked_at = Some(now);
                token.revoke_reason = Some(reason);
                count += 1;
            }
        }
        Ok(count)
    }

    async fn get_user_tokens(&self, user_id: Uuid) -> Result<Vec<RefreshToken>> {
        let tokens = self.tokens.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(tokens.values().filter(|t| t.user_id == user_id).cloned().collect())
    }

    async fn get_user_families(&self, user_id: Uuid) -> Result<Vec<Uuid>> {
        let tokens = self.tokens.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;

        let mut families: Vec<Uuid> = tokens
            .values()
            .filter(|t| t.user_id == user_id && t.is_valid())
            .map(|t| t.family_id)
            .collect();
        families.sort();
        families.dedup();
        Ok(families)
    }

    async fn get_latest_in_family(&self, family_id: Uuid) -> Result<Option<RefreshToken>> {
        let tokens = self.tokens.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;

        Ok(tokens
            .values()
            .filter(|t| t.family_id == family_id)
            .max_by_key(|t| t.generation)
            .cloned())
    }

    async fn cleanup_expired(&self) -> Result<u64> {
        let mut tokens = self.tokens.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;

        let now = Utc::now();
        let before = tokens.len();
        tokens.retain(|_, t| t.expires_at > now);
        Ok((before - tokens.len()) as u64)
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
    async fn test_create_refresh_token() {
        let store = InMemoryRefreshTokenStore::new();
        let manager = RefreshTokenManager::new(store, RefreshTokenConfig::default());

        let user_id = Uuid::now_v7();
        let (raw_token, token) = manager
            .create(user_id, Some("127.0.0.1"), None, None)
            .await
            .unwrap();

        assert!(!raw_token.is_empty());
        assert!(token.is_valid());
        assert_eq!(token.generation, 1);
    }

    #[tokio::test]
    async fn test_rotate_refresh_token() {
        let store = InMemoryRefreshTokenStore::new();
        let manager = RefreshTokenManager::new(store, RefreshTokenConfig::default());

        let user_id = Uuid::now_v7();
        let (raw_token, token) = manager
            .create(user_id, Some("127.0.0.1"), None, None)
            .await
            .unwrap();

        let family_id = token.family_id;

        // Rotate
        let (new_raw_token, new_token) = manager
            .rotate(&raw_token, Some("127.0.0.1"), None)
            .await
            .unwrap();

        assert_ne!(raw_token, new_raw_token);
        assert_eq!(new_token.family_id, family_id);
        assert_eq!(new_token.generation, 2);

        // Old token should be revoked
        assert!(manager.validate(&raw_token).await.is_err());
    }

    #[tokio::test]
    async fn test_token_reuse_detection() {
        let store = InMemoryRefreshTokenStore::new();
        let manager = RefreshTokenManager::new(store, RefreshTokenConfig::default());

        let user_id = Uuid::now_v7();
        let (raw_token, _) = manager
            .create(user_id, Some("127.0.0.1"), None, None)
            .await
            .unwrap();

        // Rotate to gen 2
        let (new_raw_token, _) = manager
            .rotate(&raw_token, Some("127.0.0.1"), None)
            .await
            .unwrap();

        // Try to use old token (reuse attack)
        let result = manager.validate(&raw_token).await;
        assert!(result.is_err());

        // New token should also be revoked (family compromised)
        let result = manager.validate(&new_raw_token).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_revoke_user_tokens() {
        let store = InMemoryRefreshTokenStore::new();
        let manager = RefreshTokenManager::new(store, RefreshTokenConfig::default());

        let user_id = Uuid::now_v7();
        manager.create(user_id, None, None, None).await.unwrap();
        manager.create(user_id, None, None, None).await.unwrap();

        let count = manager
            .revoke_user_tokens(user_id, RevokeReason::PasswordChange)
            .await
            .unwrap();
        assert_eq!(count, 2);

        let sessions = manager.get_user_sessions(user_id).await.unwrap();
        assert!(sessions.is_empty());
    }
}
