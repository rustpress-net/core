//! OAuth2 Provider (Point 59)
//!
//! Implements OAuth2 authorization server functionality for
//! third-party application authentication.

use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use rustpress_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;
use uuid::Uuid;

/// OAuth2 client application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Client {
    pub id: Uuid,
    pub client_id: String,
    pub client_secret_hash: String,
    pub name: String,
    pub description: Option<String>,
    pub redirect_uris: Vec<String>,
    pub allowed_scopes: HashSet<String>,
    pub grant_types: HashSet<GrantType>,
    pub is_confidential: bool,
    pub is_active: bool,
    pub owner_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl OAuth2Client {
    pub fn is_redirect_uri_valid(&self, uri: &str) -> bool {
        self.redirect_uris.iter().any(|allowed| {
            if allowed.ends_with('*') {
                uri.starts_with(&allowed[..allowed.len() - 1])
            } else {
                allowed == uri
            }
        })
    }

    pub fn supports_grant_type(&self, grant_type: &GrantType) -> bool {
        self.grant_types.contains(grant_type)
    }

    pub fn has_scope(&self, scope: &str) -> bool {
        self.allowed_scopes.contains(scope) || self.allowed_scopes.contains("*")
    }
}

/// OAuth2 grant types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrantType {
    AuthorizationCode,
    RefreshToken,
    ClientCredentials,
    Password,
    Implicit,
}

impl std::fmt::Display for GrantType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AuthorizationCode => write!(f, "authorization_code"),
            Self::RefreshToken => write!(f, "refresh_token"),
            Self::ClientCredentials => write!(f, "client_credentials"),
            Self::Password => write!(f, "password"),
            Self::Implicit => write!(f, "implicit"),
        }
    }
}

/// Authorization code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationCode {
    pub id: Uuid,
    pub code_hash: String,
    pub client_id: String,
    pub user_id: Uuid,
    pub redirect_uri: String,
    pub scopes: HashSet<String>,
    pub state: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<CodeChallengeMethod>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
}

impl AuthorizationCode {
    pub fn is_valid(&self) -> bool {
        self.used_at.is_none() && Utc::now() < self.expires_at
    }
}

/// PKCE code challenge method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CodeChallengeMethod {
    Plain,
    S256,
}

/// OAuth2 access token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2AccessToken {
    pub id: Uuid,
    pub token_hash: String,
    pub client_id: String,
    pub user_id: Option<Uuid>,
    pub scopes: HashSet<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl OAuth2AccessToken {
    pub fn is_valid(&self) -> bool {
        self.revoked_at.is_none() && Utc::now() < self.expires_at
    }

    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.contains(scope)
    }
}

/// OAuth2 refresh token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2RefreshToken {
    pub id: Uuid,
    pub token_hash: String,
    pub client_id: String,
    pub user_id: Option<Uuid>,
    pub scopes: HashSet<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl OAuth2RefreshToken {
    pub fn is_valid(&self) -> bool {
        self.revoked_at.is_none() && Utc::now() < self.expires_at
    }
}

/// Token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

/// OAuth2 provider configuration
#[derive(Debug, Clone)]
pub struct OAuth2ProviderConfig {
    pub issuer: String,
    pub authorization_code_lifetime: Duration,
    pub access_token_lifetime: Duration,
    pub refresh_token_lifetime: Duration,
    pub allow_public_clients: bool,
    pub require_pkce: bool,
}

impl Default for OAuth2ProviderConfig {
    fn default() -> Self {
        Self {
            issuer: "rustpress".to_string(),
            authorization_code_lifetime: Duration::minutes(10),
            access_token_lifetime: Duration::hours(1),
            refresh_token_lifetime: Duration::days(30),
            allow_public_clients: true,
            require_pkce: true,
        }
    }
}

/// OAuth2 provider storage trait
#[async_trait::async_trait]
pub trait OAuth2ProviderStore: Send + Sync {
    // Client management
    async fn create_client(&self, client: &OAuth2Client) -> Result<()>;
    async fn get_client_by_id(&self, client_id: &str) -> Result<Option<OAuth2Client>>;
    async fn update_client(&self, client: &OAuth2Client) -> Result<()>;
    async fn delete_client(&self, client_id: &str) -> Result<()>;

    // Authorization codes
    async fn store_auth_code(&self, code: &AuthorizationCode) -> Result<()>;
    async fn get_auth_code(&self, code_hash: &str) -> Result<Option<AuthorizationCode>>;
    async fn mark_auth_code_used(&self, id: Uuid) -> Result<()>;

    // Access tokens
    async fn store_access_token(&self, token: &OAuth2AccessToken) -> Result<()>;
    async fn get_access_token(&self, token_hash: &str) -> Result<Option<OAuth2AccessToken>>;
    async fn revoke_access_token(&self, id: Uuid) -> Result<()>;
    async fn revoke_client_tokens(&self, client_id: &str) -> Result<u64>;

    // Refresh tokens
    async fn store_refresh_token(&self, token: &OAuth2RefreshToken) -> Result<()>;
    async fn get_refresh_token(&self, token_hash: &str) -> Result<Option<OAuth2RefreshToken>>;
    async fn revoke_refresh_token(&self, id: Uuid) -> Result<()>;

    // Cleanup
    async fn cleanup_expired(&self) -> Result<u64>;
}

/// OAuth2 Authorization Server
pub struct OAuth2Provider<S: OAuth2ProviderStore> {
    store: S,
    config: OAuth2ProviderConfig,
}

impl<S: OAuth2ProviderStore> OAuth2Provider<S> {
    pub fn new(store: S, config: OAuth2ProviderConfig) -> Self {
        Self { store, config }
    }

    /// Generate a random token
    fn generate_token(length: usize) -> String {
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..length).map(|_| rng.gen()).collect();
        base64_url_encode(&bytes)
    }

    /// Hash a token
    fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Generate client credentials
    pub fn generate_client_credentials() -> (String, String) {
        let client_id = format!("rp_{}", Self::generate_token(16));
        let client_secret = Self::generate_token(32);
        (client_id, client_secret)
    }

    /// Register a new OAuth2 client
    pub async fn register_client(
        &self,
        name: String,
        redirect_uris: Vec<String>,
        allowed_scopes: HashSet<String>,
        grant_types: HashSet<GrantType>,
        is_confidential: bool,
        owner_id: Option<Uuid>,
    ) -> Result<(OAuth2Client, String)> {
        let (client_id, client_secret) = Self::generate_client_credentials();
        let client_secret_hash = Self::hash_token(&client_secret);
        let now = Utc::now();

        let client = OAuth2Client {
            id: Uuid::now_v7(),
            client_id: client_id.clone(),
            client_secret_hash,
            name,
            description: None,
            redirect_uris,
            allowed_scopes,
            grant_types,
            is_confidential,
            is_active: true,
            owner_id,
            created_at: now,
            updated_at: now,
        };

        self.store.create_client(&client).await?;
        Ok((client, client_secret))
    }

    /// Authenticate a client
    pub async fn authenticate_client(
        &self,
        client_id: &str,
        client_secret: Option<&str>,
    ) -> Result<OAuth2Client> {
        let client = self.store
            .get_client_by_id(client_id)
            .await?
            .ok_or_else(|| Error::Authentication {
                message: "Invalid client".to_string(),
            })?;

        if !client.is_active {
            return Err(Error::Authentication {
                message: "Client is disabled".to_string(),
            });
        }

        if client.is_confidential {
            let secret = client_secret.ok_or_else(|| Error::Authentication {
                message: "Client secret required".to_string(),
            })?;

            let secret_hash = Self::hash_token(secret);
            if secret_hash != client.client_secret_hash {
                return Err(Error::Authentication {
                    message: "Invalid client secret".to_string(),
                });
            }
        }

        Ok(client)
    }

    /// Create authorization code
    pub async fn create_authorization_code(
        &self,
        client_id: &str,
        user_id: Uuid,
        redirect_uri: &str,
        scopes: HashSet<String>,
        state: Option<String>,
        code_challenge: Option<String>,
        code_challenge_method: Option<CodeChallengeMethod>,
    ) -> Result<String> {
        let client = self.authenticate_client(client_id, None).await?;

        // Validate redirect URI
        if !client.is_redirect_uri_valid(redirect_uri) {
            return Err(Error::InvalidInput {
                field: "redirect_uri".to_string(),
                message: "Invalid redirect URI".to_string(),
            });
        }

        // Validate scopes
        for scope in &scopes {
            if !client.has_scope(scope) {
                return Err(Error::InvalidInput {
                    field: "scope".to_string(),
                    message: format!("Invalid scope: {}", scope),
                });
            }
        }

        // Require PKCE if configured
        if self.config.require_pkce && code_challenge.is_none() {
            return Err(Error::InvalidInput {
                field: "code_challenge".to_string(),
                message: "PKCE is required".to_string(),
            });
        }

        let code = Self::generate_token(32);
        let code_hash = Self::hash_token(&code);
        let now = Utc::now();

        let auth_code = AuthorizationCode {
            id: Uuid::now_v7(),
            code_hash,
            client_id: client_id.to_string(),
            user_id,
            redirect_uri: redirect_uri.to_string(),
            scopes,
            state,
            code_challenge,
            code_challenge_method,
            expires_at: now + self.config.authorization_code_lifetime,
            created_at: now,
            used_at: None,
        };

        self.store.store_auth_code(&auth_code).await?;
        Ok(code)
    }

    /// Exchange authorization code for tokens
    pub async fn exchange_authorization_code(
        &self,
        code: &str,
        client_id: &str,
        client_secret: Option<&str>,
        redirect_uri: &str,
        code_verifier: Option<&str>,
    ) -> Result<TokenResponse> {
        let client = self.authenticate_client(client_id, client_secret).await?;

        if !client.supports_grant_type(&GrantType::AuthorizationCode) {
            return Err(Error::InvalidInput {
                field: "grant_type".to_string(),
                message: "Grant type not supported".to_string(),
            });
        }

        let code_hash = Self::hash_token(code);
        let auth_code = self.store
            .get_auth_code(&code_hash)
            .await?
            .ok_or_else(|| Error::Authentication {
                message: "Invalid authorization code".to_string(),
            })?;

        if !auth_code.is_valid() {
            return Err(Error::Authentication {
                message: "Authorization code expired or already used".to_string(),
            });
        }

        if auth_code.client_id != client_id {
            return Err(Error::Authentication {
                message: "Client mismatch".to_string(),
            });
        }

        if auth_code.redirect_uri != redirect_uri {
            return Err(Error::Authentication {
                message: "Redirect URI mismatch".to_string(),
            });
        }

        // Verify PKCE
        if let Some(challenge) = &auth_code.code_challenge {
            let verifier = code_verifier.ok_or_else(|| Error::InvalidInput {
                field: "code_verifier".to_string(),
                message: "Code verifier required".to_string(),
            })?;

            let valid = match auth_code.code_challenge_method {
                Some(CodeChallengeMethod::S256) => {
                    let mut hasher = Sha256::new();
                    hasher.update(verifier.as_bytes());
                    let computed = base64_url_encode(&hasher.finalize());
                    &computed == challenge
                }
                Some(CodeChallengeMethod::Plain) | None => verifier == challenge,
            };

            if !valid {
                return Err(Error::Authentication {
                    message: "Invalid code verifier".to_string(),
                });
            }
        }

        // Mark code as used
        self.store.mark_auth_code_used(auth_code.id).await?;

        // Generate tokens
        self.generate_tokens(&client, Some(auth_code.user_id), &auth_code.scopes).await
    }

    /// Generate access and refresh tokens
    async fn generate_tokens(
        &self,
        client: &OAuth2Client,
        user_id: Option<Uuid>,
        scopes: &HashSet<String>,
    ) -> Result<TokenResponse> {
        let now = Utc::now();

        // Generate access token
        let access_token_raw = Self::generate_token(32);
        let access_token = OAuth2AccessToken {
            id: Uuid::now_v7(),
            token_hash: Self::hash_token(&access_token_raw),
            client_id: client.client_id.clone(),
            user_id,
            scopes: scopes.clone(),
            expires_at: now + self.config.access_token_lifetime,
            created_at: now,
            revoked_at: None,
        };
        self.store.store_access_token(&access_token).await?;

        // Generate refresh token if grant type is supported
        let refresh_token = if client.supports_grant_type(&GrantType::RefreshToken) {
            let refresh_token_raw = Self::generate_token(32);
            let refresh_token = OAuth2RefreshToken {
                id: Uuid::now_v7(),
                token_hash: Self::hash_token(&refresh_token_raw),
                client_id: client.client_id.clone(),
                user_id,
                scopes: scopes.clone(),
                expires_at: now + self.config.refresh_token_lifetime,
                created_at: now,
                revoked_at: None,
            };
            self.store.store_refresh_token(&refresh_token).await?;
            Some(refresh_token_raw)
        } else {
            None
        };

        Ok(TokenResponse {
            access_token: access_token_raw,
            token_type: "Bearer".to_string(),
            expires_in: self.config.access_token_lifetime.num_seconds(),
            refresh_token,
            scope: Some(scopes.iter().cloned().collect::<Vec<_>>().join(" ")),
        })
    }

    /// Refresh tokens
    pub async fn refresh_tokens(
        &self,
        refresh_token: &str,
        client_id: &str,
        client_secret: Option<&str>,
    ) -> Result<TokenResponse> {
        let client = self.authenticate_client(client_id, client_secret).await?;

        if !client.supports_grant_type(&GrantType::RefreshToken) {
            return Err(Error::InvalidInput {
                field: "grant_type".to_string(),
                message: "Grant type not supported".to_string(),
            });
        }

        let token_hash = Self::hash_token(refresh_token);
        let stored_token = self.store
            .get_refresh_token(&token_hash)
            .await?
            .ok_or_else(|| Error::Authentication {
                message: "Invalid refresh token".to_string(),
            })?;

        if !stored_token.is_valid() {
            return Err(Error::TokenExpired);
        }

        if stored_token.client_id != client_id {
            return Err(Error::Authentication {
                message: "Client mismatch".to_string(),
            });
        }

        // Revoke old refresh token
        self.store.revoke_refresh_token(stored_token.id).await?;

        // Generate new tokens
        self.generate_tokens(&client, stored_token.user_id, &stored_token.scopes).await
    }

    /// Validate access token
    pub async fn validate_access_token(&self, token: &str) -> Result<OAuth2AccessToken> {
        let token_hash = Self::hash_token(token);
        let stored_token = self.store
            .get_access_token(&token_hash)
            .await?
            .ok_or_else(|| Error::Authentication {
                message: "Invalid access token".to_string(),
            })?;

        if !stored_token.is_valid() {
            return Err(Error::TokenExpired);
        }

        Ok(stored_token)
    }

    /// Revoke token
    pub async fn revoke_token(&self, token: &str) -> Result<()> {
        let token_hash = Self::hash_token(token);

        // Try access token first
        if let Some(access_token) = self.store.get_access_token(&token_hash).await? {
            self.store.revoke_access_token(access_token.id).await?;
            return Ok(());
        }

        // Try refresh token
        if let Some(refresh_token) = self.store.get_refresh_token(&token_hash).await? {
            self.store.revoke_refresh_token(refresh_token.id).await?;
            return Ok(());
        }

        Ok(())
    }

    /// Client credentials grant
    pub async fn client_credentials_grant(
        &self,
        client_id: &str,
        client_secret: &str,
        scopes: HashSet<String>,
    ) -> Result<TokenResponse> {
        let client = self.authenticate_client(client_id, Some(client_secret)).await?;

        if !client.supports_grant_type(&GrantType::ClientCredentials) {
            return Err(Error::InvalidInput {
                field: "grant_type".to_string(),
                message: "Grant type not supported".to_string(),
            });
        }

        // Validate scopes
        for scope in &scopes {
            if !client.has_scope(scope) {
                return Err(Error::InvalidInput {
                    field: "scope".to_string(),
                    message: format!("Invalid scope: {}", scope),
                });
            }
        }

        self.generate_tokens(&client, None, &scopes).await
    }

    /// Get config
    pub fn config(&self) -> &OAuth2ProviderConfig {
        &self.config
    }
}

/// In-memory OAuth2 provider store
pub struct InMemoryOAuth2ProviderStore {
    clients: RwLock<HashMap<String, OAuth2Client>>,
    auth_codes: RwLock<HashMap<String, AuthorizationCode>>,
    access_tokens: RwLock<HashMap<String, OAuth2AccessToken>>,
    refresh_tokens: RwLock<HashMap<String, OAuth2RefreshToken>>,
}

impl InMemoryOAuth2ProviderStore {
    pub fn new() -> Self {
        Self {
            clients: RwLock::new(HashMap::new()),
            auth_codes: RwLock::new(HashMap::new()),
            access_tokens: RwLock::new(HashMap::new()),
            refresh_tokens: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryOAuth2ProviderStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl OAuth2ProviderStore for InMemoryOAuth2ProviderStore {
    async fn create_client(&self, client: &OAuth2Client) -> Result<()> {
        let mut clients = self.clients.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        clients.insert(client.client_id.clone(), client.clone());
        Ok(())
    }

    async fn get_client_by_id(&self, client_id: &str) -> Result<Option<OAuth2Client>> {
        let clients = self.clients.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(clients.get(client_id).cloned())
    }

    async fn update_client(&self, client: &OAuth2Client) -> Result<()> {
        let mut clients = self.clients.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        clients.insert(client.client_id.clone(), client.clone());
        Ok(())
    }

    async fn delete_client(&self, client_id: &str) -> Result<()> {
        let mut clients = self.clients.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        clients.remove(client_id);
        Ok(())
    }

    async fn store_auth_code(&self, code: &AuthorizationCode) -> Result<()> {
        let mut codes = self.auth_codes.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        codes.insert(code.code_hash.clone(), code.clone());
        Ok(())
    }

    async fn get_auth_code(&self, code_hash: &str) -> Result<Option<AuthorizationCode>> {
        let codes = self.auth_codes.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(codes.get(code_hash).cloned())
    }

    async fn mark_auth_code_used(&self, id: Uuid) -> Result<()> {
        let mut codes = self.auth_codes.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        for code in codes.values_mut() {
            if code.id == id {
                code.used_at = Some(Utc::now());
                break;
            }
        }
        Ok(())
    }

    async fn store_access_token(&self, token: &OAuth2AccessToken) -> Result<()> {
        let mut tokens = self.access_tokens.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        tokens.insert(token.token_hash.clone(), token.clone());
        Ok(())
    }

    async fn get_access_token(&self, token_hash: &str) -> Result<Option<OAuth2AccessToken>> {
        let tokens = self.access_tokens.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(tokens.get(token_hash).cloned())
    }

    async fn revoke_access_token(&self, id: Uuid) -> Result<()> {
        let mut tokens = self.access_tokens.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        for token in tokens.values_mut() {
            if token.id == id {
                token.revoked_at = Some(Utc::now());
                break;
            }
        }
        Ok(())
    }

    async fn revoke_client_tokens(&self, client_id: &str) -> Result<u64> {
        let mut access = self.access_tokens.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        let mut refresh = self.refresh_tokens.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;

        let now = Utc::now();
        let mut count = 0;

        for token in access.values_mut() {
            if token.client_id == client_id && token.revoked_at.is_none() {
                token.revoked_at = Some(now);
                count += 1;
            }
        }

        for token in refresh.values_mut() {
            if token.client_id == client_id && token.revoked_at.is_none() {
                token.revoked_at = Some(now);
                count += 1;
            }
        }

        Ok(count)
    }

    async fn store_refresh_token(&self, token: &OAuth2RefreshToken) -> Result<()> {
        let mut tokens = self.refresh_tokens.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        tokens.insert(token.token_hash.clone(), token.clone());
        Ok(())
    }

    async fn get_refresh_token(&self, token_hash: &str) -> Result<Option<OAuth2RefreshToken>> {
        let tokens = self.refresh_tokens.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(tokens.get(token_hash).cloned())
    }

    async fn revoke_refresh_token(&self, id: Uuid) -> Result<()> {
        let mut tokens = self.refresh_tokens.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        for token in tokens.values_mut() {
            if token.id == id {
                token.revoked_at = Some(Utc::now());
                break;
            }
        }
        Ok(())
    }

    async fn cleanup_expired(&self) -> Result<u64> {
        let now = Utc::now();
        let mut count = 0;

        {
            let mut codes = self.auth_codes.write().map_err(|_| Error::Internal {
                message: "Lock poisoned".to_string(),
                request_id: None,
            })?;
            let before = codes.len();
            codes.retain(|_, c| c.expires_at > now);
            count += (before - codes.len()) as u64;
        }

        {
            let mut tokens = self.access_tokens.write().map_err(|_| Error::Internal {
                message: "Lock poisoned".to_string(),
                request_id: None,
            })?;
            let before = tokens.len();
            tokens.retain(|_, t| t.expires_at > now);
            count += (before - tokens.len()) as u64;
        }

        {
            let mut tokens = self.refresh_tokens.write().map_err(|_| Error::Internal {
                message: "Lock poisoned".to_string(),
                request_id: None,
            })?;
            let before = tokens.len();
            tokens.retain(|_, t| t.expires_at > now);
            count += (before - tokens.len()) as u64;
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
    async fn test_register_client() {
        let store = InMemoryOAuth2ProviderStore::new();
        let provider = OAuth2Provider::new(store, OAuth2ProviderConfig::default());

        let (client, secret) = provider
            .register_client(
                "Test App".to_string(),
                vec!["https://example.com/callback".to_string()],
                ["read".to_string()].into_iter().collect(),
                [GrantType::AuthorizationCode, GrantType::RefreshToken].into_iter().collect(),
                true,
                None,
            )
            .await
            .unwrap();

        assert!(client.client_id.starts_with("rp_"));
        assert!(!secret.is_empty());
    }

    #[tokio::test]
    async fn test_client_credentials_grant() {
        let store = InMemoryOAuth2ProviderStore::new();
        let provider = OAuth2Provider::new(store, OAuth2ProviderConfig::default());

        let (client, secret) = provider
            .register_client(
                "Test App".to_string(),
                vec![],
                ["read".to_string()].into_iter().collect(),
                [GrantType::ClientCredentials].into_iter().collect(),
                true,
                None,
            )
            .await
            .unwrap();

        let response = provider
            .client_credentials_grant(
                &client.client_id,
                &secret,
                ["read".to_string()].into_iter().collect(),
            )
            .await
            .unwrap();

        assert!(!response.access_token.is_empty());
        assert_eq!(response.token_type, "Bearer");
    }
}
