//! # OAuth/SSO Integration
//!
//! OAuth 2.0 and OpenID Connect integration for social login.
//!
//! Features:
//! - OAuth 2.0 authorization code flow
//! - OpenID Connect support
//! - Multiple provider support (Google, GitHub, Facebook, etc.)
//! - Account linking
//! - Automatic user creation

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// OAuth provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OAuthProvider {
    Google,
    GitHub,
    Facebook,
    Twitter,
    Microsoft,
    Apple,
    LinkedIn,
    Discord,
    GitLab,
    Custom,
}

impl OAuthProvider {
    pub fn name(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "Google",
            OAuthProvider::GitHub => "GitHub",
            OAuthProvider::Facebook => "Facebook",
            OAuthProvider::Twitter => "Twitter",
            OAuthProvider::Microsoft => "Microsoft",
            OAuthProvider::Apple => "Apple",
            OAuthProvider::LinkedIn => "LinkedIn",
            OAuthProvider::Discord => "Discord",
            OAuthProvider::GitLab => "GitLab",
            OAuthProvider::Custom => "Custom",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "google",
            OAuthProvider::GitHub => "github",
            OAuthProvider::Facebook => "facebook",
            OAuthProvider::Twitter => "twitter",
            OAuthProvider::Microsoft => "microsoft",
            OAuthProvider::Apple => "apple",
            OAuthProvider::LinkedIn => "linkedin",
            OAuthProvider::Discord => "discord",
            OAuthProvider::GitLab => "gitlab",
            OAuthProvider::Custom => "key",
        }
    }
}

/// OAuth provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthProviderConfig {
    /// Provider type
    pub provider: OAuthProvider,

    /// Display name
    pub name: String,

    /// Whether provider is enabled
    pub enabled: bool,

    /// Client ID
    pub client_id: String,

    /// Client secret
    pub client_secret: String,

    /// Authorization URL
    pub authorization_url: String,

    /// Token URL
    pub token_url: String,

    /// User info URL
    pub userinfo_url: Option<String>,

    /// Scopes to request
    pub scopes: Vec<String>,

    /// Redirect URI
    pub redirect_uri: String,

    /// PKCE enabled
    pub pkce_enabled: bool,

    /// OpenID Connect discovery URL
    pub oidc_discovery_url: Option<String>,

    /// Additional parameters
    pub extra_params: HashMap<String, String>,

    /// Button color
    pub button_color: Option<String>,

    /// Sort order
    pub sort_order: i32,
}

impl OAuthProviderConfig {
    /// Create Google OAuth config
    pub fn google(client_id: &str, client_secret: &str, redirect_uri: &str) -> Self {
        Self {
            provider: OAuthProvider::Google,
            name: "Google".to_string(),
            enabled: true,
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            authorization_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_url: "https://oauth2.googleapis.com/token".to_string(),
            userinfo_url: Some("https://www.googleapis.com/oauth2/v3/userinfo".to_string()),
            scopes: vec!["openid".to_string(), "email".to_string(), "profile".to_string()],
            redirect_uri: redirect_uri.to_string(),
            pkce_enabled: true,
            oidc_discovery_url: Some("https://accounts.google.com/.well-known/openid-configuration".to_string()),
            extra_params: HashMap::new(),
            button_color: Some("#4285F4".to_string()),
            sort_order: 1,
        }
    }

    /// Create GitHub OAuth config
    pub fn github(client_id: &str, client_secret: &str, redirect_uri: &str) -> Self {
        Self {
            provider: OAuthProvider::GitHub,
            name: "GitHub".to_string(),
            enabled: true,
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            authorization_url: "https://github.com/login/oauth/authorize".to_string(),
            token_url: "https://github.com/login/oauth/access_token".to_string(),
            userinfo_url: Some("https://api.github.com/user".to_string()),
            scopes: vec!["read:user".to_string(), "user:email".to_string()],
            redirect_uri: redirect_uri.to_string(),
            pkce_enabled: false,
            oidc_discovery_url: None,
            extra_params: HashMap::new(),
            button_color: Some("#333333".to_string()),
            sort_order: 2,
        }
    }

    /// Create Facebook OAuth config
    pub fn facebook(client_id: &str, client_secret: &str, redirect_uri: &str) -> Self {
        Self {
            provider: OAuthProvider::Facebook,
            name: "Facebook".to_string(),
            enabled: true,
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            authorization_url: "https://www.facebook.com/v18.0/dialog/oauth".to_string(),
            token_url: "https://graph.facebook.com/v18.0/oauth/access_token".to_string(),
            userinfo_url: Some("https://graph.facebook.com/me?fields=id,name,email,picture".to_string()),
            scopes: vec!["email".to_string(), "public_profile".to_string()],
            redirect_uri: redirect_uri.to_string(),
            pkce_enabled: false,
            oidc_discovery_url: None,
            extra_params: HashMap::new(),
            button_color: Some("#1877F2".to_string()),
            sort_order: 3,
        }
    }

    /// Create Microsoft OAuth config
    pub fn microsoft(client_id: &str, client_secret: &str, redirect_uri: &str, tenant: &str) -> Self {
        Self {
            provider: OAuthProvider::Microsoft,
            name: "Microsoft".to_string(),
            enabled: true,
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            authorization_url: format!("https://login.microsoftonline.com/{}/oauth2/v2.0/authorize", tenant),
            token_url: format!("https://login.microsoftonline.com/{}/oauth2/v2.0/token", tenant),
            userinfo_url: Some("https://graph.microsoft.com/v1.0/me".to_string()),
            scopes: vec!["openid".to_string(), "email".to_string(), "profile".to_string()],
            redirect_uri: redirect_uri.to_string(),
            pkce_enabled: true,
            oidc_discovery_url: Some(format!("https://login.microsoftonline.com/{}/.well-known/openid-configuration", tenant)),
            extra_params: HashMap::new(),
            button_color: Some("#2F2F2F".to_string()),
            sort_order: 4,
        }
    }

    /// Create Discord OAuth config
    pub fn discord(client_id: &str, client_secret: &str, redirect_uri: &str) -> Self {
        Self {
            provider: OAuthProvider::Discord,
            name: "Discord".to_string(),
            enabled: true,
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            authorization_url: "https://discord.com/api/oauth2/authorize".to_string(),
            token_url: "https://discord.com/api/oauth2/token".to_string(),
            userinfo_url: Some("https://discord.com/api/users/@me".to_string()),
            scopes: vec!["identify".to_string(), "email".to_string()],
            redirect_uri: redirect_uri.to_string(),
            pkce_enabled: false,
            oidc_discovery_url: None,
            extra_params: HashMap::new(),
            button_color: Some("#5865F2".to_string()),
            sort_order: 5,
        }
    }
}

/// OAuth authorization state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthState {
    /// State token
    pub state: String,

    /// Provider
    pub provider: OAuthProvider,

    /// PKCE code verifier
    pub code_verifier: Option<String>,

    /// Nonce for OpenID Connect
    pub nonce: Option<String>,

    /// Redirect URL after authentication
    pub redirect_to: Option<String>,

    /// Whether this is a link operation (linking to existing account)
    pub is_link_operation: bool,

    /// User ID if linking
    pub link_user_id: Option<i64>,

    /// Created at
    pub created_at: DateTime<Utc>,

    /// Expires at
    pub expires_at: DateTime<Utc>,
}

impl OAuthState {
    pub fn new(provider: OAuthProvider) -> Self {
        let now = Utc::now();
        Self {
            state: Uuid::new_v4().to_string(),
            provider,
            code_verifier: None,
            nonce: None,
            redirect_to: None,
            is_link_operation: false,
            link_user_id: None,
            created_at: now,
            expires_at: now + Duration::minutes(10),
        }
    }

    pub fn with_pkce(mut self) -> Self {
        self.code_verifier = Some(generate_code_verifier());
        self
    }

    pub fn with_nonce(mut self) -> Self {
        self.nonce = Some(Uuid::new_v4().to_string());
        self
    }

    pub fn for_linking(mut self, user_id: i64) -> Self {
        self.is_link_operation = true;
        self.link_user_id = Some(user_id);
        self
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Generate PKCE code challenge
    pub fn code_challenge(&self) -> Option<String> {
        self.code_verifier.as_ref().map(|v| {
            use sha2::{Sha256, Digest};
            let hash = Sha256::digest(v.as_bytes());
            data_encoding::BASE64URL_NOPAD.encode(&hash)
        })
    }
}

/// OAuth tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    /// Access token
    pub access_token: String,

    /// Refresh token
    pub refresh_token: Option<String>,

    /// Token type (usually "Bearer")
    pub token_type: String,

    /// Expires in seconds
    pub expires_in: Option<i64>,

    /// Calculated expiry time
    pub expires_at: Option<DateTime<Utc>>,

    /// ID token (for OpenID Connect)
    pub id_token: Option<String>,

    /// Scopes granted
    pub scope: Option<String>,
}

/// OAuth user profile from provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthUserProfile {
    /// Provider
    pub provider: OAuthProvider,

    /// Provider user ID
    pub provider_user_id: String,

    /// Email
    pub email: Option<String>,

    /// Email verified
    pub email_verified: Option<bool>,

    /// Full name
    pub name: Option<String>,

    /// First name
    pub first_name: Option<String>,

    /// Last name
    pub last_name: Option<String>,

    /// Avatar URL
    pub avatar_url: Option<String>,

    /// Profile URL
    pub profile_url: Option<String>,

    /// Username (from provider)
    pub username: Option<String>,

    /// Locale
    pub locale: Option<String>,

    /// Raw profile data
    pub raw_data: serde_json::Value,
}

/// Linked OAuth account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedAccount {
    /// Link ID
    pub id: Uuid,

    /// User ID
    pub user_id: i64,

    /// Provider
    pub provider: OAuthProvider,

    /// Provider user ID
    pub provider_user_id: String,

    /// Provider username/email
    pub provider_username: Option<String>,

    /// Access token (encrypted)
    pub access_token: Option<String>,

    /// Refresh token (encrypted)
    pub refresh_token: Option<String>,

    /// Token expires at
    pub token_expires_at: Option<DateTime<Utc>>,

    /// Last login at
    pub last_login_at: Option<DateTime<Utc>>,

    /// Created at
    pub created_at: DateTime<Utc>,

    /// Updated at
    pub updated_at: DateTime<Utc>,
}

impl LinkedAccount {
    pub fn new(user_id: i64, profile: &OAuthUserProfile) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            provider: profile.provider,
            provider_user_id: profile.provider_user_id.clone(),
            provider_username: profile.email.clone().or(profile.username.clone()),
            access_token: None,
            refresh_token: None,
            token_expires_at: None,
            last_login_at: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// OAuth manager
pub struct OAuthManager {
    /// Provider configurations
    providers: HashMap<OAuthProvider, OAuthProviderConfig>,

    /// Active states (for CSRF protection)
    states: HashMap<String, OAuthState>,

    /// Linked accounts by user
    linked_accounts: HashMap<i64, Vec<LinkedAccount>>,

    /// Linked accounts by provider ID
    accounts_by_provider: HashMap<(OAuthProvider, String), i64>,

    /// Settings
    settings: OAuthSettings,
}

/// OAuth settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthSettings {
    /// Allow registration via OAuth
    pub allow_registration: bool,

    /// Default role for OAuth users
    pub default_role: String,

    /// Require email verification
    pub require_email_verification: bool,

    /// Auto-link accounts with same email
    pub auto_link_by_email: bool,

    /// Allowed email domains (empty = all allowed)
    pub allowed_email_domains: Vec<String>,
}

impl Default for OAuthSettings {
    fn default() -> Self {
        Self {
            allow_registration: true,
            default_role: "subscriber".to_string(),
            require_email_verification: false,
            auto_link_by_email: true,
            allowed_email_domains: Vec::new(),
        }
    }
}

impl Default for OAuthManager {
    fn default() -> Self {
        Self {
            providers: HashMap::new(),
            states: HashMap::new(),
            linked_accounts: HashMap::new(),
            accounts_by_provider: HashMap::new(),
            settings: OAuthSettings::default(),
        }
    }
}

impl OAuthManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_settings(mut self, settings: OAuthSettings) -> Self {
        self.settings = settings;
        self
    }

    /// Register a provider
    pub fn register_provider(&mut self, config: OAuthProviderConfig) {
        self.providers.insert(config.provider, config);
    }

    /// Get provider config
    pub fn get_provider(&self, provider: OAuthProvider) -> Option<&OAuthProviderConfig> {
        self.providers.get(&provider)
    }

    /// Get enabled providers
    pub fn get_enabled_providers(&self) -> Vec<&OAuthProviderConfig> {
        let mut providers: Vec<&OAuthProviderConfig> = self.providers.values()
            .filter(|p| p.enabled)
            .collect();
        providers.sort_by_key(|p| p.sort_order);
        providers
    }

    /// Create authorization URL
    pub fn create_authorization_url(&mut self, provider: OAuthProvider, redirect_to: Option<&str>) -> Result<(String, OAuthState), String> {
        let config = self.providers.get(&provider)
            .ok_or_else(|| "Provider not configured".to_string())?;

        if !config.enabled {
            return Err("Provider is disabled".to_string());
        }

        let mut state = OAuthState::new(provider);

        if config.pkce_enabled {
            state = state.with_pkce();
        }

        if config.oidc_discovery_url.is_some() {
            state = state.with_nonce();
        }

        if let Some(redirect) = redirect_to {
            state.redirect_to = Some(redirect.to_string());
        }

        // Build URL
        let mut url = url::Url::parse(&config.authorization_url)
            .map_err(|e| format!("Invalid authorization URL: {}", e))?;

        {
            let mut params = url.query_pairs_mut();
            params.append_pair("client_id", &config.client_id);
            params.append_pair("redirect_uri", &config.redirect_uri);
            params.append_pair("response_type", "code");
            params.append_pair("state", &state.state);
            params.append_pair("scope", &config.scopes.join(" "));

            if let Some(challenge) = state.code_challenge() {
                params.append_pair("code_challenge", &challenge);
                params.append_pair("code_challenge_method", "S256");
            }

            if let Some(nonce) = &state.nonce {
                params.append_pair("nonce", nonce);
            }

            for (key, value) in &config.extra_params {
                params.append_pair(key, value);
            }
        }

        // Store state
        self.states.insert(state.state.clone(), state.clone());

        Ok((url.to_string(), state))
    }

    /// Validate state
    pub fn validate_state(&mut self, state_token: &str) -> Result<OAuthState, String> {
        let state = self.states.remove(state_token)
            .ok_or_else(|| "Invalid state".to_string())?;

        if state.is_expired() {
            return Err("State has expired".to_string());
        }

        Ok(state)
    }

    /// Link an account to a user
    pub fn link_account(&mut self, user_id: i64, profile: &OAuthUserProfile, tokens: Option<&OAuthTokens>) {
        let mut account = LinkedAccount::new(user_id, profile);

        if let Some(tokens) = tokens {
            account.access_token = Some(tokens.access_token.clone());
            account.refresh_token = tokens.refresh_token.clone();
            account.token_expires_at = tokens.expires_at;
        }

        account.last_login_at = Some(Utc::now());

        // Store by provider
        self.accounts_by_provider.insert(
            (profile.provider, profile.provider_user_id.clone()),
            user_id,
        );

        // Store by user
        self.linked_accounts.entry(user_id)
            .or_insert_with(Vec::new)
            .push(account);
    }

    /// Get linked accounts for user
    pub fn get_linked_accounts(&self, user_id: i64) -> Vec<&LinkedAccount> {
        self.linked_accounts.get(&user_id)
            .map(|accounts| accounts.iter().collect())
            .unwrap_or_default()
    }

    /// Find user by provider account
    pub fn find_user_by_provider(&self, provider: OAuthProvider, provider_user_id: &str) -> Option<i64> {
        self.accounts_by_provider.get(&(provider, provider_user_id.to_string())).copied()
    }

    /// Unlink an account
    pub fn unlink_account(&mut self, user_id: i64, provider: OAuthProvider) -> Result<(), String> {
        // Get linked accounts for user
        let accounts = self.linked_accounts.get_mut(&user_id)
            .ok_or_else(|| "No linked accounts found".to_string())?;

        // Find the account to remove
        let account = accounts.iter()
            .find(|a| a.provider == provider)
            .ok_or_else(|| "Account not linked".to_string())?;

        let provider_user_id = account.provider_user_id.clone();

        // Remove from provider index
        self.accounts_by_provider.remove(&(provider, provider_user_id));

        // Remove from user's accounts
        accounts.retain(|a| a.provider != provider);

        Ok(())
    }

    /// Check if email domain is allowed
    pub fn is_email_allowed(&self, email: &str) -> bool {
        if self.settings.allowed_email_domains.is_empty() {
            return true;
        }

        if let Some(domain) = email.split('@').nth(1) {
            self.settings.allowed_email_domains.iter()
                .any(|d| domain.to_lowercase() == d.to_lowercase())
        } else {
            false
        }
    }

    /// Get settings
    pub fn settings(&self) -> &OAuthSettings {
        &self.settings
    }

    /// Cleanup expired states
    pub fn cleanup_expired_states(&mut self) {
        self.states.retain(|_, s| !s.is_expired());
    }
}

/// Generate PKCE code verifier
fn generate_code_verifier() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    data_encoding::BASE64URL_NOPAD.encode(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_provider_config() {
        let config = OAuthProviderConfig::google("client_id", "client_secret", "https://example.com/callback");
        assert_eq!(config.provider, OAuthProvider::Google);
        assert!(config.enabled);
        assert!(config.pkce_enabled);
    }

    #[test]
    fn test_oauth_state() {
        let state = OAuthState::new(OAuthProvider::Google)
            .with_pkce()
            .with_nonce();

        assert!(!state.state.is_empty());
        assert!(state.code_verifier.is_some());
        assert!(state.nonce.is_some());
        assert!(!state.is_expired());
    }

    #[test]
    fn test_code_challenge() {
        let state = OAuthState::new(OAuthProvider::Google).with_pkce();
        let challenge = state.code_challenge();
        assert!(challenge.is_some());
    }

    #[test]
    fn test_oauth_manager() {
        let mut manager = OAuthManager::new();
        manager.register_provider(OAuthProviderConfig::google(
            "client_id",
            "client_secret",
            "https://example.com/callback",
        ));

        let enabled = manager.get_enabled_providers();
        assert_eq!(enabled.len(), 1);
    }

    #[test]
    fn test_authorization_url() {
        let mut manager = OAuthManager::new();
        manager.register_provider(OAuthProviderConfig::google(
            "client_id",
            "client_secret",
            "https://example.com/callback",
        ));

        let (url, state) = manager.create_authorization_url(OAuthProvider::Google, None).unwrap();
        assert!(url.contains("accounts.google.com"));
        assert!(url.contains("client_id=client_id"));
        assert!(url.contains(&format!("state={}", state.state)));
    }
}
