//! API Key Authentication (Point 67)
//!
//! Provides API key management for external service integrations.

use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use rustpress_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;
use uuid::Uuid;

/// API Key scope/permission
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApiKeyScope {
    pub resource: String,
    pub action: String,
}

impl ApiKeyScope {
    pub fn new(resource: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            action: action.into(),
        }
    }

    /// Full access scope
    pub fn full_access() -> Self {
        Self::new("*", "*")
    }

    /// Read-only scope
    pub fn read_only() -> Self {
        Self::new("*", "read")
    }

    /// Check if this scope covers another
    pub fn covers(&self, other: &ApiKeyScope) -> bool {
        let resource_match = self.resource == "*" || self.resource == other.resource;
        let action_match = self.action == "*" || self.action == other.action;
        resource_match && action_match
    }
}

impl std::fmt::Display for ApiKeyScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.resource, self.action)
    }
}

/// API Key entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub site_id: Option<Uuid>,
    pub user_id: Uuid,

    /// Key name/description
    pub name: String,

    /// Key prefix (first 8 chars, for display)
    pub prefix: String,

    /// Hashed key
    pub key_hash: String,

    /// Scopes/permissions
    pub scopes: HashSet<ApiKeyScope>,

    /// Rate limit (requests per minute)
    pub rate_limit: Option<u32>,

    /// IP allowlist
    pub allowed_ips: Option<Vec<String>>,

    /// Expiration
    pub expires_at: Option<DateTime<Utc>>,

    /// Last used
    pub last_used_at: Option<DateTime<Utc>>,
    pub last_used_ip: Option<String>,

    /// Usage stats
    pub request_count: u64,

    /// Status
    pub is_active: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoke_reason: Option<String>,

    /// Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ApiKey {
    /// Check if the key is valid
    pub fn is_valid(&self) -> bool {
        if !self.is_active || self.revoked_at.is_some() {
            return false;
        }

        if let Some(expires) = self.expires_at {
            if Utc::now() >= expires {
                return false;
            }
        }

        true
    }

    /// Check if IP is allowed
    pub fn is_ip_allowed(&self, ip: &str) -> bool {
        match &self.allowed_ips {
            Some(ips) if !ips.is_empty() => {
                ips.iter().any(|allowed| {
                    if allowed.contains('/') {
                        // CIDR notation - simplified check
                        ip.starts_with(allowed.split('/').next().unwrap_or(""))
                    } else {
                        allowed == ip
                    }
                })
            }
            _ => true,
        }
    }

    /// Check if key has scope
    pub fn has_scope(&self, scope: &ApiKeyScope) -> bool {
        self.scopes.iter().any(|s| s.covers(scope))
    }
}

/// API Key manager configuration
#[derive(Debug, Clone)]
pub struct ApiKeyConfig {
    /// Key length in bytes
    pub key_length: usize,
    /// Key prefix (e.g., "rp_live_" or "rp_test_")
    pub key_prefix: String,
    /// Default expiration
    pub default_expiry: Option<Duration>,
    /// Maximum keys per user
    pub max_keys_per_user: usize,
}

impl Default for ApiKeyConfig {
    fn default() -> Self {
        Self {
            key_length: 32,
            key_prefix: "rp_".to_string(),
            default_expiry: None,
            max_keys_per_user: 10,
        }
    }
}

/// API Key storage trait
#[async_trait::async_trait]
pub trait ApiKeyStore: Send + Sync {
    async fn create(&self, key: &ApiKey) -> Result<()>;
    async fn get_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>>;
    async fn get_by_id(&self, id: Uuid) -> Result<Option<ApiKey>>;
    async fn get_user_keys(&self, user_id: Uuid) -> Result<Vec<ApiKey>>;
    async fn update(&self, key: &ApiKey) -> Result<()>;
    async fn revoke(&self, id: Uuid, reason: Option<String>) -> Result<()>;
    async fn delete(&self, id: Uuid) -> Result<()>;
    async fn count_user_keys(&self, user_id: Uuid) -> Result<usize>;
}

/// API Key manager
pub struct ApiKeyManager<S: ApiKeyStore> {
    store: S,
    config: ApiKeyConfig,
}

impl<S: ApiKeyStore> ApiKeyManager<S> {
    pub fn new(store: S, config: ApiKeyConfig) -> Self {
        Self { store, config }
    }

    /// Generate a new API key
    fn generate_key(&self) -> String {
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..self.config.key_length)
            .map(|_| rng.gen())
            .collect();

        format!("{}{}", self.config.key_prefix, hex::encode(bytes))
    }

    /// Hash an API key
    fn hash_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Create a new API key
    pub async fn create(
        &self,
        user_id: Uuid,
        name: String,
        scopes: HashSet<ApiKeyScope>,
        site_id: Option<Uuid>,
        expires_at: Option<DateTime<Utc>>,
        allowed_ips: Option<Vec<String>>,
        rate_limit: Option<u32>,
    ) -> Result<(String, ApiKey)> {
        // Check key limit
        let count = self.store.count_user_keys(user_id).await?;
        if count >= self.config.max_keys_per_user {
            return Err(Error::validation(
                format!("Maximum {} API keys per user", self.config.max_keys_per_user)
            ));
        }

        let raw_key = self.generate_key();
        let key_hash = Self::hash_key(&raw_key);
        let prefix = raw_key.chars().take(12).collect();
        let now = Utc::now();

        let expiry = expires_at.or_else(|| {
            self.config.default_expiry.map(|d| now + d)
        });

        let api_key = ApiKey {
            id: Uuid::now_v7(),
            site_id,
            user_id,
            name,
            prefix,
            key_hash,
            scopes,
            rate_limit,
            allowed_ips,
            expires_at: expiry,
            last_used_at: None,
            last_used_ip: None,
            request_count: 0,
            is_active: true,
            revoked_at: None,
            revoke_reason: None,
            created_at: now,
            updated_at: now,
        };

        self.store.create(&api_key).await?;

        Ok((raw_key, api_key))
    }

    /// Validate an API key
    pub async fn validate(&self, key: &str, ip: Option<&str>) -> Result<ApiKey> {
        let key_hash = Self::hash_key(key);

        let mut api_key = self.store
            .get_by_hash(&key_hash)
            .await?
            .ok_or_else(|| Error::Authentication {
                message: "Invalid API key".to_string(),
            })?;

        if !api_key.is_valid() {
            return Err(Error::Authentication {
                message: "API key is expired or revoked".to_string(),
            });
        }

        // Check IP allowlist
        if let Some(ip) = ip {
            if !api_key.is_ip_allowed(ip) {
                return Err(Error::Authorization {
                    action: "IP not allowed".to_string(),
                    required: "api_key:ip_allowed".to_string(),
                });
            }
        }

        // Update usage stats
        api_key.last_used_at = Some(Utc::now());
        api_key.last_used_ip = ip.map(String::from);
        api_key.request_count += 1;
        api_key.updated_at = Utc::now();

        self.store.update(&api_key).await?;

        Ok(api_key)
    }

    /// Validate key and check scope
    pub async fn validate_with_scope(
        &self,
        key: &str,
        required_scope: &ApiKeyScope,
        ip: Option<&str>,
    ) -> Result<ApiKey> {
        let api_key = self.validate(key, ip).await?;

        if !api_key.has_scope(required_scope) {
            return Err(Error::Authorization {
                action: format!("API key missing required scope: {}", required_scope),
                required: required_scope.to_string(),
            });
        }

        Ok(api_key)
    }

    /// Get user's API keys
    pub async fn get_user_keys(&self, user_id: Uuid) -> Result<Vec<ApiKey>> {
        self.store.get_user_keys(user_id).await
    }

    /// Revoke an API key
    pub async fn revoke(&self, id: Uuid, reason: Option<String>) -> Result<()> {
        self.store.revoke(id, reason).await
    }

    /// Delete an API key
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        self.store.delete(id).await
    }

    /// Rotate an API key (create new, revoke old)
    pub async fn rotate(&self, old_key_id: Uuid) -> Result<(String, ApiKey)> {
        let old_key = self.store
            .get_by_id(old_key_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity_type: "ApiKey".to_string(),
                id: old_key_id.to_string(),
            })?;

        // Create new key with same settings
        let (new_raw_key, new_key) = self.create(
            old_key.user_id,
            format!("{} (rotated)", old_key.name),
            old_key.scopes,
            old_key.site_id,
            old_key.expires_at,
            old_key.allowed_ips,
            old_key.rate_limit,
        ).await?;

        // Revoke old key
        self.revoke(old_key_id, Some("Rotated".to_string())).await?;

        Ok((new_raw_key, new_key))
    }
}

/// In-memory API key store for testing
pub struct InMemoryApiKeyStore {
    keys: RwLock<HashMap<String, ApiKey>>,
}

impl InMemoryApiKeyStore {
    pub fn new() -> Self {
        Self {
            keys: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryApiKeyStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ApiKeyStore for InMemoryApiKeyStore {
    async fn create(&self, key: &ApiKey) -> Result<()> {
        let mut keys = self.keys.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        keys.insert(key.key_hash.clone(), key.clone());
        Ok(())
    }

    async fn get_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>> {
        let keys = self.keys.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(keys.get(key_hash).cloned())
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<ApiKey>> {
        let keys = self.keys.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(keys.values().find(|k| k.id == id).cloned())
    }

    async fn get_user_keys(&self, user_id: Uuid) -> Result<Vec<ApiKey>> {
        let keys = self.keys.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(keys.values().filter(|k| k.user_id == user_id).cloned().collect())
    }

    async fn update(&self, key: &ApiKey) -> Result<()> {
        let mut keys = self.keys.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        keys.insert(key.key_hash.clone(), key.clone());
        Ok(())
    }

    async fn revoke(&self, id: Uuid, reason: Option<String>) -> Result<()> {
        let mut keys = self.keys.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;

        for key in keys.values_mut() {
            if key.id == id {
                key.is_active = false;
                key.revoked_at = Some(Utc::now());
                key.revoke_reason = reason;
                break;
            }
        }
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        let mut keys = self.keys.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        keys.retain(|_, k| k.id != id);
        Ok(())
    }

    async fn count_user_keys(&self, user_id: Uuid) -> Result<usize> {
        let keys = self.keys.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(keys.values().filter(|k| k.user_id == user_id && k.is_active).count())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_api_key() {
        let store = InMemoryApiKeyStore::new();
        let manager = ApiKeyManager::new(store, ApiKeyConfig::default());

        let user_id = Uuid::now_v7();
        let scopes = [ApiKeyScope::read_only()].into_iter().collect();

        let (raw_key, api_key) = manager
            .create(user_id, "Test Key".to_string(), scopes, None, None, None, None)
            .await
            .unwrap();

        assert!(raw_key.starts_with("rp_"));
        assert!(api_key.is_valid());
    }

    #[tokio::test]
    async fn test_validate_api_key() {
        let store = InMemoryApiKeyStore::new();
        let manager = ApiKeyManager::new(store, ApiKeyConfig::default());

        let user_id = Uuid::now_v7();
        let scopes = [ApiKeyScope::full_access()].into_iter().collect();

        let (raw_key, _) = manager
            .create(user_id, "Test Key".to_string(), scopes, None, None, None, None)
            .await
            .unwrap();

        let validated = manager.validate(&raw_key, None).await.unwrap();
        assert_eq!(validated.user_id, user_id);
    }

    #[tokio::test]
    async fn test_scope_checking() {
        let scope = ApiKeyScope::new("posts", "*");
        let read_scope = ApiKeyScope::new("posts", "read");

        assert!(scope.covers(&read_scope));
        assert!(!read_scope.covers(&scope));
    }
}
