//! CSRF Protection (Point 75)
//!
//! Cross-Site Request Forgery protection using double-submit cookie
//! pattern and synchronizer tokens.

use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use rustpress_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

/// CSRF token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsrfToken {
    pub id: Uuid,
    pub token: String,
    pub token_hash: String,
    pub session_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub used: bool,
}

impl CsrfToken {
    pub fn is_valid(&self) -> bool {
        !self.used && Utc::now() < self.expires_at
    }
}

/// CSRF protection configuration
#[derive(Debug, Clone)]
pub struct CsrfConfig {
    /// Token lifetime
    pub token_lifetime: Duration,
    /// Token length in bytes
    pub token_length: usize,
    /// Cookie name for double-submit pattern
    pub cookie_name: String,
    /// Header name for token submission
    pub header_name: String,
    /// Form field name
    pub field_name: String,
    /// Safe methods that don't require CSRF protection
    pub safe_methods: Vec<String>,
    /// Paths to exclude from CSRF protection
    pub excluded_paths: Vec<String>,
    /// Use single-use tokens
    pub single_use: bool,
    /// Same site cookie policy
    pub same_site: SameSite,
    /// Secure cookie
    pub secure: bool,
}

impl Default for CsrfConfig {
    fn default() -> Self {
        Self {
            token_lifetime: Duration::hours(2),
            token_length: 32,
            cookie_name: "csrf_token".to_string(),
            header_name: "X-CSRF-Token".to_string(),
            field_name: "_csrf".to_string(),
            safe_methods: vec!["GET".to_string(), "HEAD".to_string(), "OPTIONS".to_string()],
            excluded_paths: vec![],
            single_use: false,
            same_site: SameSite::Strict,
            secure: true,
        }
    }
}

/// Same site policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

impl std::fmt::Display for SameSite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Strict => write!(f, "Strict"),
            Self::Lax => write!(f, "Lax"),
            Self::None => write!(f, "None"),
        }
    }
}

/// CSRF validation result
#[derive(Debug, Clone)]
pub struct CsrfValidation {
    pub valid: bool,
    pub reason: Option<String>,
}

impl CsrfValidation {
    pub fn valid() -> Self {
        Self {
            valid: true,
            reason: None,
        }
    }

    pub fn invalid(reason: impl Into<String>) -> Self {
        Self {
            valid: false,
            reason: Some(reason.into()),
        }
    }
}

/// CSRF token storage trait
#[async_trait::async_trait]
pub trait CsrfStore: Send + Sync {
    async fn store_token(&self, token: &CsrfToken) -> Result<()>;
    async fn get_token(&self, token_hash: &str) -> Result<Option<CsrfToken>>;
    async fn mark_used(&self, id: Uuid) -> Result<()>;
    async fn delete_token(&self, id: Uuid) -> Result<()>;
    async fn cleanup_expired(&self) -> Result<u64>;
}

/// CSRF protection manager
pub struct CsrfProtection<S: CsrfStore> {
    store: S,
    config: CsrfConfig,
}

impl<S: CsrfStore> CsrfProtection<S> {
    pub fn new(store: S, config: CsrfConfig) -> Self {
        Self { store, config }
    }

    /// Generate a new CSRF token
    fn generate_token_string(&self) -> String {
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..self.config.token_length)
            .map(|_| rng.gen())
            .collect();
        base64_url_encode(&bytes)
    }

    /// Hash a token
    fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Generate a new CSRF token for a session/user
    pub async fn generate(
        &self,
        session_id: Option<Uuid>,
        user_id: Option<Uuid>,
    ) -> Result<CsrfToken> {
        let token = self.generate_token_string();
        let token_hash = Self::hash_token(&token);
        let now = Utc::now();

        let csrf_token = CsrfToken {
            id: Uuid::now_v7(),
            token: token.clone(),
            token_hash,
            session_id,
            user_id,
            expires_at: now + self.config.token_lifetime,
            created_at: now,
            used: false,
        };

        self.store.store_token(&csrf_token).await?;
        Ok(csrf_token)
    }

    /// Validate a CSRF token
    pub async fn validate(&self, token: &str, session_id: Option<Uuid>) -> Result<CsrfValidation> {
        if token.is_empty() {
            return Ok(CsrfValidation::invalid("Missing CSRF token"));
        }

        let token_hash = Self::hash_token(token);
        let stored = self.store.get_token(&token_hash).await?;

        match stored {
            None => Ok(CsrfValidation::invalid("Invalid CSRF token")),
            Some(csrf_token) => {
                if !csrf_token.is_valid() {
                    return Ok(CsrfValidation::invalid("CSRF token expired or already used"));
                }

                // Validate session binding if session is provided
                if let Some(expected_session) = session_id {
                    if let Some(token_session) = csrf_token.session_id {
                        if token_session != expected_session {
                            return Ok(CsrfValidation::invalid("CSRF token session mismatch"));
                        }
                    }
                }

                // Mark as used if single-use
                if self.config.single_use {
                    self.store.mark_used(csrf_token.id).await?;
                }

                Ok(CsrfValidation::valid())
            }
        }
    }

    /// Check if method requires CSRF protection
    pub fn requires_protection(&self, method: &str) -> bool {
        !self.config.safe_methods.iter().any(|m| m.eq_ignore_ascii_case(method))
    }

    /// Check if path is excluded from protection
    pub fn is_path_excluded(&self, path: &str) -> bool {
        self.config.excluded_paths.iter().any(|p| {
            if p.ends_with('*') {
                path.starts_with(&p[..p.len() - 1])
            } else {
                p == path
            }
        })
    }

    /// Validate request
    pub async fn validate_request(
        &self,
        method: &str,
        path: &str,
        token: Option<&str>,
        session_id: Option<Uuid>,
    ) -> Result<CsrfValidation> {
        // Skip safe methods
        if !self.requires_protection(method) {
            return Ok(CsrfValidation::valid());
        }

        // Skip excluded paths
        if self.is_path_excluded(path) {
            return Ok(CsrfValidation::valid());
        }

        // Validate token
        match token {
            Some(t) => self.validate(t, session_id).await,
            None => Ok(CsrfValidation::invalid("Missing CSRF token")),
        }
    }

    /// Generate HTML hidden input for forms
    pub async fn form_field(&self, session_id: Option<Uuid>) -> Result<String> {
        let token = self.generate(session_id, None).await?;
        Ok(format!(
            r#"<input type="hidden" name="{}" value="{}">"#,
            self.config.field_name, token.token
        ))
    }

    /// Generate meta tag for JavaScript
    pub async fn meta_tag(&self, session_id: Option<Uuid>) -> Result<String> {
        let token = self.generate(session_id, None).await?;
        Ok(format!(
            r#"<meta name="csrf-token" content="{}">"#,
            token.token
        ))
    }

    /// Generate cookie string
    pub fn cookie_string(&self, token: &str) -> String {
        let mut cookie = format!("{}={}; Path=/", self.config.cookie_name, token);

        if self.config.secure {
            cookie.push_str("; Secure");
        }

        cookie.push_str("; HttpOnly");
        cookie.push_str(&format!("; SameSite={}", self.config.same_site));

        cookie
    }

    /// Extract token from request
    pub fn extract_token(
        &self,
        headers: &HashMap<String, String>,
        form_data: Option<&HashMap<String, String>>,
        query_params: Option<&HashMap<String, String>>,
    ) -> Option<String> {
        // Check header first (preferred)
        if let Some(token) = headers.get(&self.config.header_name) {
            if !token.is_empty() {
                return Some(token.clone());
            }
        }

        // Check form data
        if let Some(form) = form_data {
            if let Some(token) = form.get(&self.config.field_name) {
                if !token.is_empty() {
                    return Some(token.clone());
                }
            }
        }

        // Check query params (least preferred)
        if let Some(params) = query_params {
            if let Some(token) = params.get(&self.config.field_name) {
                if !token.is_empty() {
                    return Some(token.clone());
                }
            }
        }

        None
    }

    /// Cleanup expired tokens
    pub async fn cleanup(&self) -> Result<u64> {
        self.store.cleanup_expired().await
    }

    /// Get config
    pub fn config(&self) -> &CsrfConfig {
        &self.config
    }
}

/// In-memory CSRF store
pub struct InMemoryCsrfStore {
    tokens: RwLock<HashMap<String, CsrfToken>>,
}

impl InMemoryCsrfStore {
    pub fn new() -> Self {
        Self {
            tokens: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryCsrfStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl CsrfStore for InMemoryCsrfStore {
    async fn store_token(&self, token: &CsrfToken) -> Result<()> {
        let mut tokens = self.tokens.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        tokens.insert(token.token_hash.clone(), token.clone());
        Ok(())
    }

    async fn get_token(&self, token_hash: &str) -> Result<Option<CsrfToken>> {
        let tokens = self.tokens.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(tokens.get(token_hash).cloned())
    }

    async fn mark_used(&self, id: Uuid) -> Result<()> {
        let mut tokens = self.tokens.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        for token in tokens.values_mut() {
            if token.id == id {
                token.used = true;
                break;
            }
        }
        Ok(())
    }

    async fn delete_token(&self, id: Uuid) -> Result<()> {
        let mut tokens = self.tokens.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        tokens.retain(|_, t| t.id != id);
        Ok(())
    }

    async fn cleanup_expired(&self) -> Result<u64> {
        let mut tokens = self.tokens.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        let now = Utc::now();
        let before = tokens.len();
        tokens.retain(|_, t| t.expires_at > now && !t.used);
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
    async fn test_generate_and_validate() {
        let store = InMemoryCsrfStore::new();
        let protection = CsrfProtection::new(store, CsrfConfig::default());

        let csrf_token = protection.generate(None, None).await.unwrap();

        let result = protection.validate(&csrf_token.token, None).await.unwrap();
        assert!(result.valid);
    }

    #[tokio::test]
    async fn test_invalid_token() {
        let store = InMemoryCsrfStore::new();
        let protection = CsrfProtection::new(store, CsrfConfig::default());

        let result = protection.validate("invalid_token", None).await.unwrap();
        assert!(!result.valid);
    }

    #[tokio::test]
    async fn test_single_use_token() {
        let store = InMemoryCsrfStore::new();
        let config = CsrfConfig {
            single_use: true,
            ..Default::default()
        };
        let protection = CsrfProtection::new(store, config);

        let csrf_token = protection.generate(None, None).await.unwrap();

        // First use should succeed
        let result = protection.validate(&csrf_token.token, None).await.unwrap();
        assert!(result.valid);

        // Second use should fail
        let result = protection.validate(&csrf_token.token, None).await.unwrap();
        assert!(!result.valid);
    }

    #[test]
    fn test_safe_methods() {
        let protection = CsrfProtection::new(InMemoryCsrfStore::new(), CsrfConfig::default());

        assert!(!protection.requires_protection("GET"));
        assert!(!protection.requires_protection("HEAD"));
        assert!(!protection.requires_protection("OPTIONS"));
        assert!(protection.requires_protection("POST"));
        assert!(protection.requires_protection("PUT"));
        assert!(protection.requires_protection("DELETE"));
    }

    #[test]
    fn test_excluded_paths() {
        let config = CsrfConfig {
            excluded_paths: vec![
                "/api/webhook".to_string(),
                "/api/public/*".to_string(),
            ],
            ..Default::default()
        };
        let protection = CsrfProtection::new(InMemoryCsrfStore::new(), config);

        assert!(protection.is_path_excluded("/api/webhook"));
        assert!(protection.is_path_excluded("/api/public/endpoint"));
        assert!(!protection.is_path_excluded("/api/private/endpoint"));
    }

    #[tokio::test]
    async fn test_form_field_generation() {
        let store = InMemoryCsrfStore::new();
        let protection = CsrfProtection::new(store, CsrfConfig::default());

        let html = protection.form_field(None).await.unwrap();
        assert!(html.contains(r#"type="hidden""#));
        assert!(html.contains(r#"name="_csrf""#));
    }

    #[test]
    fn test_token_extraction() {
        let protection = CsrfProtection::new(InMemoryCsrfStore::new(), CsrfConfig::default());

        let mut headers = HashMap::new();
        headers.insert("X-CSRF-Token".to_string(), "header_token".to_string());

        let token = protection.extract_token(&headers, None, None);
        assert_eq!(token, Some("header_token".to_string()));

        let mut form_data = HashMap::new();
        form_data.insert("_csrf".to_string(), "form_token".to_string());

        let token = protection.extract_token(&HashMap::new(), Some(&form_data), None);
        assert_eq!(token, Some("form_token".to_string()));
    }
}
