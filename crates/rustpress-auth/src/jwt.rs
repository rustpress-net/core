//! JWT token management.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rustpress_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Issued at
    pub iat: i64,
    /// Expiration time
    pub exp: i64,
    /// Not before
    pub nbf: i64,
    /// JWT ID
    pub jti: String,
    /// Issuer
    pub iss: String,
    /// Token type
    pub typ: TokenType,
    /// User role
    pub role: Option<String>,
    /// Tenant ID (for multi-tenancy)
    pub tenant_id: Option<String>,
    /// Custom claims
    #[serde(flatten)]
    pub custom: std::collections::HashMap<String, serde_json::Value>,
}

impl Claims {
    pub fn new(user_id: impl Into<String>, issuer: impl Into<String>, typ: TokenType) -> Self {
        let now = Utc::now();
        Self {
            sub: user_id.into(),
            iat: now.timestamp(),
            exp: now.timestamp(), // Will be set by JwtManager
            nbf: now.timestamp(),
            jti: Uuid::now_v7().to_string(),
            iss: issuer.into(),
            typ,
            role: None,
            tenant_id: None,
            custom: std::collections::HashMap::new(),
        }
    }

    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.role = Some(role.into());
        self
    }

    pub fn with_tenant(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    pub fn with_custom(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.custom.insert(key.into(), value);
        self
    }

    pub fn user_id(&self) -> &str {
        &self.sub
    }

    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }
}

/// Token type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

/// Token pair (access + refresh)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// JWT configuration
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// Secret key for signing
    pub secret: String,
    /// Issuer
    pub issuer: String,
    /// Access token expiry (seconds)
    pub access_expiry_secs: i64,
    /// Refresh token expiry (seconds)
    pub refresh_expiry_secs: i64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "change-me-in-production".to_string(),
            issuer: "rustpress".to_string(),
            access_expiry_secs: 900, // 15 minutes
            refresh_expiry_secs: 604800, // 7 days
        }
    }
}

/// JWT Manager
pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    config: JwtConfig,
    validation: Validation,
}

impl JwtManager {
    pub fn new(config: JwtConfig) -> Self {
        let encoding_key = EncodingKey::from_secret(config.secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.secret.as_bytes());

        let mut validation = Validation::default();
        validation.set_issuer(&[&config.issuer]);
        validation.validate_exp = true;
        validation.validate_nbf = true;

        Self {
            encoding_key,
            decoding_key,
            config,
            validation,
        }
    }

    /// Generate a token pair for a user
    pub fn generate_tokens(&self, user_id: &str, role: Option<&str>, tenant_id: Option<&str>) -> Result<TokenPair> {
        let access_token = self.generate_access_token(user_id, role, tenant_id)?;
        let refresh_token = self.generate_refresh_token(user_id)?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.access_expiry_secs,
        })
    }

    /// Generate an access token
    pub fn generate_access_token(
        &self,
        user_id: &str,
        role: Option<&str>,
        tenant_id: Option<&str>,
    ) -> Result<String> {
        let mut claims = Claims::new(user_id, &self.config.issuer, TokenType::Access);
        claims.exp = (Utc::now() + Duration::seconds(self.config.access_expiry_secs)).timestamp();

        if let Some(role) = role {
            claims = claims.with_role(role);
        }
        if let Some(tenant) = tenant_id {
            claims = claims.with_tenant(tenant);
        }

        encode(&Header::default(), &claims, &self.encoding_key).map_err(|e| {
            Error::Internal {
                message: format!("Failed to generate access token: {}", e),
                request_id: None,
            }
        })
    }

    /// Generate a refresh token
    pub fn generate_refresh_token(&self, user_id: &str) -> Result<String> {
        let mut claims = Claims::new(user_id, &self.config.issuer, TokenType::Refresh);
        claims.exp = (Utc::now() + Duration::seconds(self.config.refresh_expiry_secs)).timestamp();

        encode(&Header::default(), &claims, &self.encoding_key).map_err(|e| {
            Error::Internal {
                message: format!("Failed to generate refresh token: {}", e),
                request_id: None,
            }
        })
    }

    /// Validate and decode a token
    pub fn validate(&self, token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation).map_err(
            |e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => Error::TokenExpired,
                jsonwebtoken::errors::ErrorKind::InvalidToken => Error::InvalidToken {
                    reason: "Invalid token format".to_string(),
                },
                jsonwebtoken::errors::ErrorKind::InvalidSignature => Error::InvalidToken {
                    reason: "Invalid signature".to_string(),
                },
                _ => Error::InvalidToken {
                    reason: e.to_string(),
                },
            },
        )?;

        Ok(token_data.claims)
    }

    /// Validate an access token
    pub fn validate_access_token(&self, token: &str) -> Result<Claims> {
        let claims = self.validate(token)?;
        if claims.typ != TokenType::Access {
            return Err(Error::InvalidToken {
                reason: "Not an access token".to_string(),
            });
        }
        Ok(claims)
    }

    /// Validate a refresh token
    pub fn validate_refresh_token(&self, token: &str) -> Result<Claims> {
        let claims = self.validate(token)?;
        if claims.typ != TokenType::Refresh {
            return Err(Error::InvalidToken {
                reason: "Not a refresh token".to_string(),
            });
        }
        Ok(claims)
    }

    /// Refresh tokens using a refresh token
    pub fn refresh_tokens(
        &self,
        refresh_token: &str,
        role: Option<&str>,
        tenant_id: Option<&str>,
    ) -> Result<TokenPair> {
        let claims = self.validate_refresh_token(refresh_token)?;
        self.generate_tokens(&claims.sub, role, tenant_id)
    }

    /// Extract user ID from token without full validation
    pub fn extract_user_id(&self, token: &str) -> Result<String> {
        let mut validation = Validation::default();
        validation.insecure_disable_signature_validation();
        validation.validate_exp = false;

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation).map_err(
            |e| Error::InvalidToken {
                reason: e.to_string(),
            },
        )?;

        Ok(token_data.claims.sub)
    }

    /// Get config
    pub fn config(&self) -> &JwtConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_manager() -> JwtManager {
        JwtManager::new(JwtConfig::default())
    }

    #[test]
    fn test_generate_tokens() {
        let manager = create_manager();
        let tokens = manager
            .generate_tokens("user-123", Some("admin"), None)
            .unwrap();

        assert!(!tokens.access_token.is_empty());
        assert!(!tokens.refresh_token.is_empty());
        assert_eq!(tokens.token_type, "Bearer");
    }

    #[test]
    fn test_validate_access_token() {
        let manager = create_manager();
        let token = manager
            .generate_access_token("user-123", Some("admin"), None)
            .unwrap();

        let claims = manager.validate_access_token(&token).unwrap();
        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.role, Some("admin".to_string()));
        assert_eq!(claims.typ, TokenType::Access);
    }

    #[test]
    fn test_validate_refresh_token() {
        let manager = create_manager();
        let token = manager.generate_refresh_token("user-123").unwrap();

        let claims = manager.validate_refresh_token(&token).unwrap();
        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.typ, TokenType::Refresh);
    }

    #[test]
    fn test_refresh_tokens() {
        let manager = create_manager();
        let refresh_token = manager.generate_refresh_token("user-123").unwrap();

        let new_tokens = manager
            .refresh_tokens(&refresh_token, Some("user"), None)
            .unwrap();

        let claims = manager.validate_access_token(&new_tokens.access_token).unwrap();
        assert_eq!(claims.sub, "user-123");
    }

    #[test]
    fn test_invalid_token() {
        let manager = create_manager();
        let result = manager.validate("invalid-token");
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_token_type() {
        let manager = create_manager();
        let access_token = manager
            .generate_access_token("user-123", None, None)
            .unwrap();

        // Try to validate access token as refresh token
        let result = manager.validate_refresh_token(&access_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_claims_with_tenant() {
        let manager = create_manager();
        let token = manager
            .generate_access_token("user-123", Some("admin"), Some("tenant-456"))
            .unwrap();

        let claims = manager.validate_access_token(&token).unwrap();
        assert_eq!(claims.tenant_id, Some("tenant-456".to_string()));
    }
}
