//! OAuth2 Client for Social Logins (Point 60)
//!
//! Implements OAuth2 client functionality for social login providers
//! like Google, GitHub, Facebook, etc.

use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use rustpress_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use uuid::Uuid;

/// OAuth2 provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2ClientProvider {
    pub name: String,
    pub client_id: String,
    pub client_secret: String,
    pub authorization_url: String,
    pub token_url: String,
    pub userinfo_url: Option<String>,
    pub scopes: Vec<String>,
    pub redirect_uri: String,
    pub enabled: bool,
    /// Additional parameters for authorization request
    pub auth_params: HashMap<String, String>,
    /// Field mapping for user info
    pub user_mapping: UserFieldMapping,
}

impl OAuth2ClientProvider {
    /// Create a Google OAuth2 provider
    pub fn google(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            name: "google".to_string(),
            client_id,
            client_secret,
            authorization_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_url: "https://oauth2.googleapis.com/token".to_string(),
            userinfo_url: Some("https://www.googleapis.com/oauth2/v3/userinfo".to_string()),
            scopes: vec!["openid".to_string(), "email".to_string(), "profile".to_string()],
            redirect_uri,
            enabled: true,
            auth_params: HashMap::new(),
            user_mapping: UserFieldMapping::google(),
        }
    }

    /// Create a GitHub OAuth2 provider
    pub fn github(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            name: "github".to_string(),
            client_id,
            client_secret,
            authorization_url: "https://github.com/login/oauth/authorize".to_string(),
            token_url: "https://github.com/login/oauth/access_token".to_string(),
            userinfo_url: Some("https://api.github.com/user".to_string()),
            scopes: vec!["user:email".to_string()],
            redirect_uri,
            enabled: true,
            auth_params: HashMap::new(),
            user_mapping: UserFieldMapping::github(),
        }
    }

    /// Create a Facebook OAuth2 provider
    pub fn facebook(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            name: "facebook".to_string(),
            client_id,
            client_secret,
            authorization_url: "https://www.facebook.com/v18.0/dialog/oauth".to_string(),
            token_url: "https://graph.facebook.com/v18.0/oauth/access_token".to_string(),
            userinfo_url: Some("https://graph.facebook.com/v18.0/me?fields=id,name,email,picture".to_string()),
            scopes: vec!["email".to_string(), "public_profile".to_string()],
            redirect_uri,
            enabled: true,
            auth_params: HashMap::new(),
            user_mapping: UserFieldMapping::facebook(),
        }
    }

    /// Create a Discord OAuth2 provider
    pub fn discord(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            name: "discord".to_string(),
            client_id,
            client_secret,
            authorization_url: "https://discord.com/api/oauth2/authorize".to_string(),
            token_url: "https://discord.com/api/oauth2/token".to_string(),
            userinfo_url: Some("https://discord.com/api/users/@me".to_string()),
            scopes: vec!["identify".to_string(), "email".to_string()],
            redirect_uri,
            enabled: true,
            auth_params: HashMap::new(),
            user_mapping: UserFieldMapping::discord(),
        }
    }

    /// Create a Microsoft/Azure AD OAuth2 provider
    pub fn microsoft(client_id: String, client_secret: String, redirect_uri: String, tenant: Option<String>) -> Self {
        let tenant = tenant.unwrap_or_else(|| "common".to_string());
        Self {
            name: "microsoft".to_string(),
            client_id,
            client_secret,
            authorization_url: format!("https://login.microsoftonline.com/{}/oauth2/v2.0/authorize", tenant),
            token_url: format!("https://login.microsoftonline.com/{}/oauth2/v2.0/token", tenant),
            userinfo_url: Some("https://graph.microsoft.com/v1.0/me".to_string()),
            scopes: vec!["openid".to_string(), "email".to_string(), "profile".to_string()],
            redirect_uri,
            enabled: true,
            auth_params: HashMap::new(),
            user_mapping: UserFieldMapping::microsoft(),
        }
    }

    /// Create a custom OAuth2 provider
    pub fn custom(
        name: String,
        client_id: String,
        client_secret: String,
        authorization_url: String,
        token_url: String,
        userinfo_url: Option<String>,
        scopes: Vec<String>,
        redirect_uri: String,
    ) -> Self {
        Self {
            name,
            client_id,
            client_secret,
            authorization_url,
            token_url,
            userinfo_url,
            scopes,
            redirect_uri,
            enabled: true,
            auth_params: HashMap::new(),
            user_mapping: UserFieldMapping::default(),
        }
    }
}

/// User field mapping for different providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFieldMapping {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar: Option<String>,
    pub locale: Option<String>,
}

impl Default for UserFieldMapping {
    fn default() -> Self {
        Self {
            id: "id".to_string(),
            email: "email".to_string(),
            name: Some("name".to_string()),
            first_name: None,
            last_name: None,
            avatar: None,
            locale: None,
        }
    }
}

impl UserFieldMapping {
    pub fn google() -> Self {
        Self {
            id: "sub".to_string(),
            email: "email".to_string(),
            name: Some("name".to_string()),
            first_name: Some("given_name".to_string()),
            last_name: Some("family_name".to_string()),
            avatar: Some("picture".to_string()),
            locale: Some("locale".to_string()),
        }
    }

    pub fn github() -> Self {
        Self {
            id: "id".to_string(),
            email: "email".to_string(),
            name: Some("name".to_string()),
            first_name: None,
            last_name: None,
            avatar: Some("avatar_url".to_string()),
            locale: None,
        }
    }

    pub fn facebook() -> Self {
        Self {
            id: "id".to_string(),
            email: "email".to_string(),
            name: Some("name".to_string()),
            first_name: Some("first_name".to_string()),
            last_name: Some("last_name".to_string()),
            avatar: Some("picture.data.url".to_string()),
            locale: None,
        }
    }

    pub fn discord() -> Self {
        Self {
            id: "id".to_string(),
            email: "email".to_string(),
            name: Some("username".to_string()),
            first_name: None,
            last_name: None,
            avatar: None, // Discord avatar requires special handling
            locale: Some("locale".to_string()),
        }
    }

    pub fn microsoft() -> Self {
        Self {
            id: "id".to_string(),
            email: "mail".to_string(),
            name: Some("displayName".to_string()),
            first_name: Some("givenName".to_string()),
            last_name: Some("surname".to_string()),
            avatar: None,
            locale: None,
        }
    }
}

/// OAuth2 authorization state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2State {
    pub id: Uuid,
    pub provider: String,
    pub state: String,
    pub code_verifier: Option<String>,
    pub redirect_after: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl OAuth2State {
    pub fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at
    }
}

/// OAuth2 token response from provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<i64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    pub id_token: Option<String>,
}

/// User info from OAuth2 provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2UserInfo {
    pub provider: String,
    pub provider_user_id: String,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar_url: Option<String>,
    pub locale: Option<String>,
    pub raw_data: serde_json::Value,
}

/// Connected social account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialConnection {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub provider_user_id: String,
    pub email: Option<String>,
    pub access_token_hash: String,
    pub refresh_token_hash: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub connected_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

/// OAuth2 client configuration
#[derive(Debug, Clone)]
pub struct OAuth2ClientConfig {
    pub state_lifetime: Duration,
    pub use_pkce: bool,
}

impl Default for OAuth2ClientConfig {
    fn default() -> Self {
        Self {
            state_lifetime: Duration::minutes(10),
            use_pkce: true,
        }
    }
}

/// OAuth2 client state store trait
#[async_trait::async_trait]
pub trait OAuth2StateStore: Send + Sync {
    async fn store_state(&self, state: &OAuth2State) -> Result<()>;
    async fn get_state(&self, state: &str) -> Result<Option<OAuth2State>>;
    async fn delete_state(&self, state: &str) -> Result<()>;
    async fn cleanup_expired(&self) -> Result<u64>;
}

/// Social connections store trait
#[async_trait::async_trait]
pub trait SocialConnectionStore: Send + Sync {
    async fn create(&self, connection: &SocialConnection) -> Result<()>;
    async fn get_by_provider(&self, provider: &str, provider_user_id: &str) -> Result<Option<SocialConnection>>;
    async fn get_user_connections(&self, user_id: Uuid) -> Result<Vec<SocialConnection>>;
    async fn update(&self, connection: &SocialConnection) -> Result<()>;
    async fn delete(&self, id: Uuid) -> Result<()>;
    async fn delete_user_connection(&self, user_id: Uuid, provider: &str) -> Result<()>;
}

/// OAuth2 client for social logins
pub struct OAuth2Client<S: OAuth2StateStore, C: SocialConnectionStore> {
    providers: HashMap<String, OAuth2ClientProvider>,
    state_store: S,
    connection_store: C,
    config: OAuth2ClientConfig,
}

impl<S: OAuth2StateStore, C: SocialConnectionStore> OAuth2Client<S, C> {
    pub fn new(state_store: S, connection_store: C, config: OAuth2ClientConfig) -> Self {
        Self {
            providers: HashMap::new(),
            state_store,
            connection_store,
            config,
        }
    }

    /// Register a provider
    pub fn register_provider(&mut self, provider: OAuth2ClientProvider) {
        self.providers.insert(provider.name.clone(), provider);
    }

    /// Get authorization URL for a provider
    pub async fn get_authorization_url(
        &self,
        provider_name: &str,
        redirect_after: Option<String>,
    ) -> Result<(String, OAuth2State)> {
        let provider = self.providers.get(provider_name).ok_or_else(|| Error::InvalidInput {
            field: "provider".to_string(),
            message: format!("Unknown provider: {}", provider_name),
        })?;

        if !provider.enabled {
            return Err(Error::InvalidInput {
                field: "provider".to_string(),
                message: "Provider is disabled".to_string(),
            });
        }

        let state_value = generate_random_string(32);
        let code_verifier = if self.config.use_pkce {
            Some(generate_random_string(64))
        } else {
            None
        };

        let now = Utc::now();
        let oauth_state = OAuth2State {
            id: Uuid::now_v7(),
            provider: provider_name.to_string(),
            state: state_value.clone(),
            code_verifier: code_verifier.clone(),
            redirect_after,
            expires_at: now + self.config.state_lifetime,
            created_at: now,
        };

        self.state_store.store_state(&oauth_state).await?;

        let mut url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&state={}&scope={}",
            provider.authorization_url,
            urlencoding::encode(&provider.client_id),
            urlencoding::encode(&provider.redirect_uri),
            urlencoding::encode(&state_value),
            urlencoding::encode(&provider.scopes.join(" "))
        );

        // Add PKCE if enabled
        if let Some(ref verifier) = code_verifier {
            let challenge = generate_code_challenge(verifier);
            url.push_str(&format!("&code_challenge={}&code_challenge_method=S256", challenge));
        }

        // Add additional auth params
        for (key, value) in &provider.auth_params {
            url.push_str(&format!("&{}={}", urlencoding::encode(key), urlencoding::encode(value)));
        }

        Ok((url, oauth_state))
    }

    /// Handle OAuth2 callback
    pub async fn handle_callback(
        &self,
        code: &str,
        state: &str,
    ) -> Result<(OAuth2TokenResponse, OAuth2UserInfo)> {
        // Validate state
        let oauth_state = self.state_store
            .get_state(state)
            .await?
            .ok_or_else(|| Error::Authentication {
                message: "Invalid OAuth2 state".to_string(),
            })?;

        if !oauth_state.is_valid() {
            return Err(Error::Authentication {
                message: "OAuth2 state expired".to_string(),
            });
        }

        // Delete state (single use)
        self.state_store.delete_state(state).await?;

        let provider = self.providers.get(&oauth_state.provider).ok_or_else(|| Error::Internal {
            message: "Provider not found".to_string(),
            request_id: None,
        })?;

        // Exchange code for token
        let token_response = self.exchange_code(provider, code, oauth_state.code_verifier.as_deref()).await?;

        // Get user info
        let user_info = if let Some(ref userinfo_url) = provider.userinfo_url {
            self.fetch_user_info(provider, &token_response.access_token, userinfo_url).await?
        } else {
            // Try to extract from ID token if available
            if let Some(ref id_token) = token_response.id_token {
                self.parse_id_token(provider, id_token)?
            } else {
                return Err(Error::Internal {
                    message: "No userinfo URL or ID token available".to_string(),
                    request_id: None,
                });
            }
        };

        Ok((token_response, user_info))
    }

    /// Exchange authorization code for tokens
    async fn exchange_code(
        &self,
        provider: &OAuth2ClientProvider,
        code: &str,
        code_verifier: Option<&str>,
    ) -> Result<OAuth2TokenResponse> {
        let mut params = HashMap::new();
        params.insert("grant_type", "authorization_code");
        params.insert("code", code);
        params.insert("redirect_uri", &provider.redirect_uri);
        params.insert("client_id", &provider.client_id);
        params.insert("client_secret", &provider.client_secret);

        let verifier_string;
        if let Some(verifier) = code_verifier {
            verifier_string = verifier.to_string();
            params.insert("code_verifier", &verifier_string);
        }

        // In a real implementation, this would use reqwest or similar
        // For now, we return a placeholder error
        Err(Error::Internal {
            message: "HTTP client not implemented - use reqwest in production".to_string(),
            request_id: None,
        })
    }

    /// Fetch user info from provider
    async fn fetch_user_info(
        &self,
        provider: &OAuth2ClientProvider,
        access_token: &str,
        userinfo_url: &str,
    ) -> Result<OAuth2UserInfo> {
        // In a real implementation, this would use reqwest or similar
        // For now, we return a placeholder error
        let _ = (access_token, userinfo_url);
        Err(Error::Internal {
            message: "HTTP client not implemented - use reqwest in production".to_string(),
            request_id: None,
        })
    }

    /// Parse ID token (JWT)
    fn parse_id_token(&self, provider: &OAuth2ClientProvider, id_token: &str) -> Result<OAuth2UserInfo> {
        // In a real implementation, this would decode the JWT
        let _ = id_token;
        Err(Error::Internal {
            message: "JWT decoding not implemented".to_string(),
            request_id: None,
        })
    }

    /// Link social account to user
    pub async fn link_account(
        &self,
        user_id: Uuid,
        provider_name: &str,
        user_info: &OAuth2UserInfo,
        token_response: &OAuth2TokenResponse,
    ) -> Result<SocialConnection> {
        // Check if already linked
        if let Some(existing) = self.connection_store
            .get_by_provider(provider_name, &user_info.provider_user_id)
            .await?
        {
            if existing.user_id != user_id {
                return Err(Error::InvalidInput {
                    field: "provider".to_string(),
                    message: "This social account is already linked to another user".to_string(),
                });
            }
            return Ok(existing);
        }

        let now = Utc::now();
        let connection = SocialConnection {
            id: Uuid::now_v7(),
            user_id,
            provider: provider_name.to_string(),
            provider_user_id: user_info.provider_user_id.clone(),
            email: user_info.email.clone(),
            access_token_hash: hash_token(&token_response.access_token),
            refresh_token_hash: token_response.refresh_token.as_ref().map(|t| hash_token(t)),
            token_expires_at: token_response.expires_in.map(|e| now + Duration::seconds(e)),
            metadata: user_info.raw_data.clone(),
            connected_at: now,
            last_used_at: Some(now),
        };

        self.connection_store.create(&connection).await?;
        Ok(connection)
    }

    /// Get user's social connections
    pub async fn get_user_connections(&self, user_id: Uuid) -> Result<Vec<SocialConnection>> {
        self.connection_store.get_user_connections(user_id).await
    }

    /// Unlink social account
    pub async fn unlink_account(&self, user_id: Uuid, provider_name: &str) -> Result<()> {
        self.connection_store.delete_user_connection(user_id, provider_name).await
    }

    /// Find user by social login
    pub async fn find_user_by_social(
        &self,
        provider_name: &str,
        provider_user_id: &str,
    ) -> Result<Option<SocialConnection>> {
        self.connection_store.get_by_provider(provider_name, provider_user_id).await
    }

    /// Get available providers
    pub fn get_providers(&self) -> Vec<&OAuth2ClientProvider> {
        self.providers.values().filter(|p| p.enabled).collect()
    }
}

/// Generate a random string
fn generate_random_string(length: usize) -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..length).map(|_| rng.gen()).collect();
    base64_url_encode(&bytes)
}

/// Generate PKCE code challenge
fn generate_code_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    base64_url_encode(&hasher.finalize())
}

/// Hash a token
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/// Base64 URL-safe encoding
fn base64_url_encode(bytes: &[u8]) -> String {
    base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, bytes)
}

/// In-memory state store
pub struct InMemoryOAuth2StateStore {
    states: std::sync::RwLock<HashMap<String, OAuth2State>>,
}

impl InMemoryOAuth2StateStore {
    pub fn new() -> Self {
        Self {
            states: std::sync::RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryOAuth2StateStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl OAuth2StateStore for InMemoryOAuth2StateStore {
    async fn store_state(&self, state: &OAuth2State) -> Result<()> {
        let mut states = self.states.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        states.insert(state.state.clone(), state.clone());
        Ok(())
    }

    async fn get_state(&self, state: &str) -> Result<Option<OAuth2State>> {
        let states = self.states.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(states.get(state).cloned())
    }

    async fn delete_state(&self, state: &str) -> Result<()> {
        let mut states = self.states.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        states.remove(state);
        Ok(())
    }

    async fn cleanup_expired(&self) -> Result<u64> {
        let mut states = self.states.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        let now = Utc::now();
        let before = states.len();
        states.retain(|_, s| s.expires_at > now);
        Ok((before - states.len()) as u64)
    }
}

/// In-memory social connections store
pub struct InMemorySocialConnectionStore {
    connections: std::sync::RwLock<HashMap<Uuid, SocialConnection>>,
}

impl InMemorySocialConnectionStore {
    pub fn new() -> Self {
        Self {
            connections: std::sync::RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemorySocialConnectionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl SocialConnectionStore for InMemorySocialConnectionStore {
    async fn create(&self, connection: &SocialConnection) -> Result<()> {
        let mut connections = self.connections.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        connections.insert(connection.id, connection.clone());
        Ok(())
    }

    async fn get_by_provider(&self, provider: &str, provider_user_id: &str) -> Result<Option<SocialConnection>> {
        let connections = self.connections.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(connections
            .values()
            .find(|c| c.provider == provider && c.provider_user_id == provider_user_id)
            .cloned())
    }

    async fn get_user_connections(&self, user_id: Uuid) -> Result<Vec<SocialConnection>> {
        let connections = self.connections.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(connections.values().filter(|c| c.user_id == user_id).cloned().collect())
    }

    async fn update(&self, connection: &SocialConnection) -> Result<()> {
        let mut connections = self.connections.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        connections.insert(connection.id, connection.clone());
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        let mut connections = self.connections.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        connections.remove(&id);
        Ok(())
    }

    async fn delete_user_connection(&self, user_id: Uuid, provider: &str) -> Result<()> {
        let mut connections = self.connections.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        connections.retain(|_, c| !(c.user_id == user_id && c.provider == provider));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_provider() {
        let provider = OAuth2ClientProvider::google(
            "client_id".to_string(),
            "client_secret".to_string(),
            "https://example.com/callback".to_string(),
        );

        assert_eq!(provider.name, "google");
        assert!(provider.scopes.contains(&"openid".to_string()));
    }

    #[test]
    fn test_github_provider() {
        let provider = OAuth2ClientProvider::github(
            "client_id".to_string(),
            "client_secret".to_string(),
            "https://example.com/callback".to_string(),
        );

        assert_eq!(provider.name, "github");
        assert!(provider.scopes.contains(&"user:email".to_string()));
    }

    #[tokio::test]
    async fn test_authorization_url() {
        let state_store = InMemoryOAuth2StateStore::new();
        let connection_store = InMemorySocialConnectionStore::new();
        let mut client = OAuth2Client::new(state_store, connection_store, OAuth2ClientConfig::default());

        client.register_provider(OAuth2ClientProvider::google(
            "test_client_id".to_string(),
            "test_secret".to_string(),
            "https://example.com/callback".to_string(),
        ));

        let (url, state) = client
            .get_authorization_url("google", None)
            .await
            .unwrap();

        assert!(url.contains("accounts.google.com"));
        assert!(url.contains("client_id=test_client_id"));
        assert!(url.contains(&format!("state={}", state.state)));
    }

    #[test]
    fn test_code_challenge() {
        let verifier = "test_verifier_string_that_is_long_enough";
        let challenge = generate_code_challenge(verifier);

        // Verify it produces consistent output
        let challenge2 = generate_code_challenge(verifier);
        assert_eq!(challenge, challenge2);

        // Verify different verifiers produce different challenges
        let challenge3 = generate_code_challenge("different_verifier_string");
        assert_ne!(challenge, challenge3);
    }
}
