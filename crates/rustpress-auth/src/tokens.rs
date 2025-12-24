//! Token management for password reset and email verification.
//! Points 64 (Password Reset) and 65 (Email Verification)

use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use rustpress_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

/// Token types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenType {
    PasswordReset,
    EmailVerification,
    AccountActivation,
    EmailChange,
    PhoneVerification,
}

/// Base token structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub token_type: TokenType,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// Password reset token (Point 64)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub email: String,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
}

impl PasswordResetToken {
    pub fn is_valid(&self) -> bool {
        self.used_at.is_none() && Utc::now() < self.expires_at
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }
}

/// Email verification token (Point 65)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub email: String,
    pub token_hash: String,
    pub token_type: TokenType,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
}

impl VerificationToken {
    pub fn is_valid(&self) -> bool {
        self.verified_at.is_none() && Utc::now() < self.expires_at
    }
}

/// Token storage trait
#[async_trait::async_trait]
pub trait TokenStore: Send + Sync {
    async fn store_token(&self, token: &SecureToken) -> Result<()>;
    async fn get_token(&self, token_hash: &str, token_type: TokenType) -> Result<Option<SecureToken>>;
    async fn mark_used(&self, id: Uuid) -> Result<()>;
    async fn invalidate_user_tokens(&self, user_id: Uuid, token_type: TokenType) -> Result<u64>;
    async fn cleanup_expired(&self) -> Result<u64>;
}

/// Token manager configuration
#[derive(Debug, Clone)]
pub struct TokenConfig {
    /// Password reset token validity duration
    pub password_reset_duration: Duration,
    /// Email verification token validity duration
    pub email_verification_duration: Duration,
    /// Token length in bytes
    pub token_length: usize,
    /// Maximum active tokens per user per type
    pub max_active_tokens: usize,
    /// Invalidate previous tokens on new request
    pub invalidate_previous: bool,
}

impl Default for TokenConfig {
    fn default() -> Self {
        Self {
            password_reset_duration: Duration::hours(1),
            email_verification_duration: Duration::hours(24),
            token_length: 32,
            max_active_tokens: 3,
            invalidate_previous: true,
        }
    }
}

/// Token manager for password reset and email verification
pub struct TokenManager<S: TokenStore> {
    store: S,
    config: TokenConfig,
}

impl<S: TokenStore> TokenManager<S> {
    pub fn new(store: S, config: TokenConfig) -> Self {
        Self { store, config }
    }

    /// Generate a secure random token
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

    /// Create a password reset token (Point 64)
    pub async fn create_password_reset(
        &self,
        user_id: Uuid,
        email: &str,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<(String, PasswordResetToken)> {
        // Optionally invalidate previous tokens
        if self.config.invalidate_previous {
            self.store.invalidate_user_tokens(user_id, TokenType::PasswordReset).await?;
        }

        let token = self.generate_token();
        let token_hash = Self::hash_token(&token);
        let now = Utc::now();

        let reset_token = PasswordResetToken {
            id: Uuid::now_v7(),
            user_id,
            email: email.to_string(),
            token_hash: token_hash.clone(),
            expires_at: now + self.config.password_reset_duration,
            ip_address: ip_address.map(String::from),
            user_agent: user_agent.map(String::from),
            created_at: now,
            used_at: None,
        };

        // Store as SecureToken
        let secure_token = SecureToken {
            id: reset_token.id,
            user_id,
            token_hash,
            token_type: TokenType::PasswordReset,
            expires_at: reset_token.expires_at,
            used_at: None,
            created_at: now,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("email".to_string(), email.to_string());
                if let Some(ip) = ip_address {
                    meta.insert("ip_address".to_string(), ip.to_string());
                }
                meta
            },
        };

        self.store.store_token(&secure_token).await?;

        Ok((token, reset_token))
    }

    /// Verify a password reset token
    pub async fn verify_password_reset(&self, token: &str) -> Result<SecureToken> {
        let token_hash = Self::hash_token(token);

        let stored = self.store
            .get_token(&token_hash, TokenType::PasswordReset)
            .await?
            .ok_or_else(|| Error::InvalidToken {
                reason: "Invalid or expired password reset token".to_string(),
            })?;

        if stored.used_at.is_some() {
            return Err(Error::InvalidToken {
                reason: "Token has already been used".to_string(),
            });
        }

        if Utc::now() >= stored.expires_at {
            return Err(Error::TokenExpired);
        }

        Ok(stored)
    }

    /// Consume a password reset token (mark as used)
    pub async fn consume_password_reset(&self, token: &str) -> Result<SecureToken> {
        let stored = self.verify_password_reset(token).await?;
        self.store.mark_used(stored.id).await?;
        Ok(stored)
    }

    /// Create an email verification token (Point 65)
    pub async fn create_email_verification(
        &self,
        user_id: Uuid,
        email: &str,
    ) -> Result<(String, VerificationToken)> {
        // Invalidate previous verification tokens for this user
        if self.config.invalidate_previous {
            self.store.invalidate_user_tokens(user_id, TokenType::EmailVerification).await?;
        }

        let token = self.generate_token();
        let token_hash = Self::hash_token(&token);
        let now = Utc::now();

        let verification_token = VerificationToken {
            id: Uuid::now_v7(),
            user_id,
            email: email.to_string(),
            token_hash: token_hash.clone(),
            token_type: TokenType::EmailVerification,
            expires_at: now + self.config.email_verification_duration,
            created_at: now,
            verified_at: None,
        };

        let secure_token = SecureToken {
            id: verification_token.id,
            user_id,
            token_hash,
            token_type: TokenType::EmailVerification,
            expires_at: verification_token.expires_at,
            used_at: None,
            created_at: now,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("email".to_string(), email.to_string());
                meta
            },
        };

        self.store.store_token(&secure_token).await?;

        Ok((token, verification_token))
    }

    /// Verify an email verification token
    pub async fn verify_email(&self, token: &str) -> Result<SecureToken> {
        let token_hash = Self::hash_token(token);

        let stored = self.store
            .get_token(&token_hash, TokenType::EmailVerification)
            .await?
            .ok_or_else(|| Error::InvalidToken {
                reason: "Invalid or expired verification token".to_string(),
            })?;

        if stored.used_at.is_some() {
            return Err(Error::InvalidToken {
                reason: "Email already verified".to_string(),
            });
        }

        if Utc::now() >= stored.expires_at {
            return Err(Error::TokenExpired);
        }

        // Mark as used
        self.store.mark_used(stored.id).await?;

        Ok(stored)
    }

    /// Create a generic secure token
    pub async fn create_token(
        &self,
        user_id: Uuid,
        token_type: TokenType,
        duration: Duration,
        metadata: HashMap<String, String>,
    ) -> Result<(String, SecureToken)> {
        let token = self.generate_token();
        let token_hash = Self::hash_token(&token);
        let now = Utc::now();

        let secure_token = SecureToken {
            id: Uuid::now_v7(),
            user_id,
            token_hash,
            token_type,
            expires_at: now + duration,
            used_at: None,
            created_at: now,
            metadata,
        };

        self.store.store_token(&secure_token).await?;

        Ok((token, secure_token))
    }

    /// Cleanup expired tokens
    pub async fn cleanup(&self) -> Result<u64> {
        self.store.cleanup_expired().await
    }
}

/// In-memory token store for testing
pub struct InMemoryTokenStore {
    tokens: RwLock<HashMap<String, SecureToken>>,
}

impl InMemoryTokenStore {
    pub fn new() -> Self {
        Self {
            tokens: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryTokenStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl TokenStore for InMemoryTokenStore {
    async fn store_token(&self, token: &SecureToken) -> Result<()> {
        let mut tokens = self.tokens.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        tokens.insert(token.token_hash.clone(), token.clone());
        Ok(())
    }

    async fn get_token(&self, token_hash: &str, token_type: TokenType) -> Result<Option<SecureToken>> {
        let tokens = self.tokens.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;

        Ok(tokens.get(token_hash)
            .filter(|t| t.token_type == token_type)
            .cloned())
    }

    async fn mark_used(&self, id: Uuid) -> Result<()> {
        let mut tokens = self.tokens.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;

        for token in tokens.values_mut() {
            if token.id == id {
                token.used_at = Some(Utc::now());
                break;
            }
        }
        Ok(())
    }

    async fn invalidate_user_tokens(&self, user_id: Uuid, token_type: TokenType) -> Result<u64> {
        let mut tokens = self.tokens.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;

        let to_remove: Vec<String> = tokens
            .iter()
            .filter(|(_, t)| t.user_id == user_id && t.token_type == token_type)
            .map(|(k, _)| k.clone())
            .collect();

        let count = to_remove.len() as u64;
        for key in to_remove {
            tokens.remove(&key);
        }

        Ok(count)
    }

    async fn cleanup_expired(&self) -> Result<u64> {
        let mut tokens = self.tokens.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;

        let now = Utc::now();
        let to_remove: Vec<String> = tokens
            .iter()
            .filter(|(_, t)| t.expires_at < now)
            .map(|(k, _)| k.clone())
            .collect();

        let count = to_remove.len() as u64;
        for key in to_remove {
            tokens.remove(&key);
        }

        Ok(count)
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
    async fn test_password_reset_token() {
        let store = InMemoryTokenStore::new();
        let manager = TokenManager::new(store, TokenConfig::default());

        let user_id = Uuid::now_v7();
        let (token, reset) = manager
            .create_password_reset(user_id, "test@example.com", None, None)
            .await
            .unwrap();

        assert!(!token.is_empty());
        assert!(reset.is_valid());

        // Verify the token
        let verified = manager.verify_password_reset(&token).await.unwrap();
        assert_eq!(verified.user_id, user_id);
    }

    #[tokio::test]
    async fn test_email_verification_token() {
        let store = InMemoryTokenStore::new();
        let manager = TokenManager::new(store, TokenConfig::default());

        let user_id = Uuid::now_v7();
        let (token, verification) = manager
            .create_email_verification(user_id, "test@example.com")
            .await
            .unwrap();

        assert!(!token.is_empty());
        assert!(verification.is_valid());

        // Verify the email
        let verified = manager.verify_email(&token).await.unwrap();
        assert_eq!(verified.user_id, user_id);

        // Token should be consumed
        assert!(manager.verify_email(&token).await.is_err());
    }
}
