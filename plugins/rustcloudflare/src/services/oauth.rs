//! Cloudflare OAuth service for SSO authentication

use crate::error::{CloudflareError, CloudflareResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, error};

/// Cloudflare OAuth configuration
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

/// OAuth token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

/// Cloudflare user info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareUserInfo {
    pub id: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub two_factor_authentication_enabled: bool,
    pub suspended: bool,
    pub has_pro_zones: bool,
    pub has_business_zones: bool,
    pub has_enterprise_zones: bool,
    pub organizations: Option<Vec<CloudflareOrganization>>,
}

/// Cloudflare organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareOrganization {
    pub id: String,
    pub name: String,
    pub status: String,
    pub permissions: Vec<String>,
    pub roles: Vec<String>,
}

/// Cloudflare account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareAccount {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub account_type: Option<String>,
    pub settings: Option<AccountSettings>,
}

/// Account settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSettings {
    pub enforce_twofactor: Option<bool>,
    pub use_account_custom_ns_by_default: Option<bool>,
}

/// Cloudflare zone (simplified)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareZoneInfo {
    pub id: String,
    pub name: String,
    pub status: String,
    pub paused: bool,
    pub plan: Option<ZonePlan>,
}

/// Zone plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZonePlan {
    pub id: String,
    pub name: String,
}

/// OAuth service for Cloudflare authentication
pub struct OAuthService {
    client: Client,
    config: Option<OAuthConfig>,
}

impl OAuthService {
    /// Create a new OAuth service
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            config: None,
        }
    }

    /// Create with OAuth config
    pub fn with_config(config: OAuthConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            config: Some(config),
        }
    }

    /// Generate OAuth authorization URL
    pub fn get_auth_url(&self, state: &str) -> CloudflareResult<String> {
        let config = self.config.as_ref().ok_or_else(|| {
            CloudflareError::ConfigError("OAuth not configured".to_string())
        })?;

        let url = format!(
            "https://dash.cloudflare.com/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&state={}&scope=account:read%20zone:read%20user:read",
            urlencoding::encode(&config.client_id),
            urlencoding::encode(&config.redirect_uri),
            urlencoding::encode(state)
        );

        Ok(url)
    }

    /// Exchange authorization code for tokens
    pub async fn exchange_code(&self, code: &str) -> CloudflareResult<OAuthTokenResponse> {
        let config = self.config.as_ref().ok_or_else(|| {
            CloudflareError::ConfigError("OAuth not configured".to_string())
        })?;

        let response = self.client
            .post("https://api.cloudflare.com/client/v4/oauth/token")
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("client_id", &config.client_id),
                ("client_secret", &config.client_secret),
                ("redirect_uri", &config.redirect_uri),
            ])
            .send()
            .await
            .map_err(|e| CloudflareError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("OAuth token exchange failed: {}", error_text);
            return Err(CloudflareError::AuthenticationError(
                "Failed to exchange authorization code".to_string()
            ));
        }

        let token: OAuthTokenResponse = response.json().await
            .map_err(|e| CloudflareError::Internal(format!("Failed to parse token response: {}", e)))?;

        info!("Successfully exchanged OAuth code for tokens");
        Ok(token)
    }

    /// Get user info using access token
    pub async fn get_user_info(&self, access_token: &str) -> CloudflareResult<CloudflareUserInfo> {
        let response = self.client
            .get("https://api.cloudflare.com/client/v4/user")
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| CloudflareError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CloudflareError::AuthenticationError(
                "Failed to get user info".to_string()
            ));
        }

        #[derive(Deserialize)]
        struct UserResponse {
            result: CloudflareUserInfo,
        }

        let user_response: UserResponse = response.json().await
            .map_err(|e| CloudflareError::Internal(format!("Failed to parse user info: {}", e)))?;

        Ok(user_response.result)
    }

    /// List user's accounts
    pub async fn list_accounts(&self, access_token: &str) -> CloudflareResult<Vec<CloudflareAccount>> {
        let response = self.client
            .get("https://api.cloudflare.com/client/v4/accounts")
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| CloudflareError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CloudflareError::AuthenticationError(
                "Failed to list accounts".to_string()
            ));
        }

        #[derive(Deserialize)]
        struct AccountsResponse {
            result: Vec<CloudflareAccount>,
        }

        let accounts_response: AccountsResponse = response.json().await
            .map_err(|e| CloudflareError::Internal(format!("Failed to parse accounts: {}", e)))?;

        Ok(accounts_response.result)
    }

    /// List zones for an account
    pub async fn list_zones(
        &self,
        access_token: &str,
        account_id: Option<&str>
    ) -> CloudflareResult<Vec<CloudflareZoneInfo>> {
        let mut url = "https://api.cloudflare.com/client/v4/zones".to_string();

        if let Some(account) = account_id {
            url = format!("{}?account.id={}", url, account);
        }

        let response = self.client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| CloudflareError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CloudflareError::AuthenticationError(
                "Failed to list zones".to_string()
            ));
        }

        #[derive(Deserialize)]
        struct ZonesResponse {
            result: Vec<CloudflareZoneInfo>,
        }

        let zones_response: ZonesResponse = response.json().await
            .map_err(|e| CloudflareError::Internal(format!("Failed to parse zones: {}", e)))?;

        Ok(zones_response.result)
    }

    /// Verify an API token
    pub async fn verify_token(&self, api_token: &str) -> CloudflareResult<bool> {
        let response = self.client
            .get("https://api.cloudflare.com/client/v4/user/tokens/verify")
            .bearer_auth(api_token)
            .send()
            .await
            .map_err(|e| CloudflareError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Ok(false);
        }

        #[derive(Deserialize)]
        struct VerifyResponse {
            success: bool,
        }

        let verify: VerifyResponse = response.json().await
            .map_err(|e| CloudflareError::Internal(format!("Failed to parse verify response: {}", e)))?;

        Ok(verify.success)
    }

    /// Get accounts and zones using API token (for manual token entry)
    pub async fn get_token_resources(&self, api_token: &str) -> CloudflareResult<TokenResources> {
        // Get user info
        let user = self.get_user_info(api_token).await?;

        // Get accounts
        let accounts = self.list_accounts(api_token).await?;

        // Get zones
        let zones = self.list_zones(api_token, None).await?;

        Ok(TokenResources {
            user,
            accounts,
            zones,
        })
    }
}

/// Resources available to a token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResources {
    pub user: CloudflareUserInfo,
    pub accounts: Vec<CloudflareAccount>,
    pub zones: Vec<CloudflareZoneInfo>,
}

impl Default for OAuthService {
    fn default() -> Self {
        Self::new()
    }
}
