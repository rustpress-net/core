//! Route-Level Auth Middleware (Point 63)
//!
//! Authentication and authorization middleware for protecting routes.

use crate::jwt::{Claims, JwtManager, TokenType};
use crate::permission::{Permission, PermissionChecker};
use crate::session::{Session, SessionManager, SessionStore};
use rustpress_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

/// Authenticated user context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub session_id: Option<Uuid>,
    pub roles: Vec<String>,
    pub permissions: HashSet<String>,
    pub auth_method: AuthMethod,
    pub tenant_id: Option<Uuid>,
    pub is_impersonating: bool,
    pub original_user_id: Option<Uuid>,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl AuthContext {
    pub fn new(user_id: Uuid, auth_method: AuthMethod) -> Self {
        Self {
            user_id,
            session_id: None,
            roles: Vec::new(),
            permissions: HashSet::new(),
            auth_method,
            tenant_id: None,
            is_impersonating: false,
            original_user_id: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_session(mut self, session_id: Uuid) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = roles;
        self
    }

    pub fn with_permissions(mut self, permissions: HashSet<String>) -> Self {
        self.permissions = permissions;
        self
    }

    pub fn with_tenant(mut self, tenant_id: Uuid) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    pub fn with_impersonation(mut self, original_user_id: Uuid) -> Self {
        self.is_impersonating = true;
        self.original_user_id = Some(original_user_id);
        self
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(permission) || self.permissions.contains("*:*")
    }

    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.iter().any(|r| self.has_role(r))
    }

    pub fn has_all_roles(&self, roles: &[&str]) -> bool {
        roles.iter().all(|r| self.has_role(r))
    }
}

/// Authentication method used
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthMethod {
    Session,
    JwtBearer,
    ApiKey,
    BasicAuth,
    OAuth2,
}

/// Authentication requirement for a route
#[derive(Debug, Clone)]
pub enum AuthRequirement {
    /// No authentication required
    None,
    /// Any authenticated user
    Authenticated,
    /// Specific role required
    Role(String),
    /// Any of these roles
    AnyRole(Vec<String>),
    /// All of these roles
    AllRoles(Vec<String>),
    /// Specific permission required
    Permission(Permission),
    /// Any of these permissions
    AnyPermission(Vec<Permission>),
    /// All of these permissions
    AllPermissions(Vec<Permission>),
    /// Custom check function
    Custom(String),
}

impl Default for AuthRequirement {
    fn default() -> Self {
        Self::None
    }
}

/// Auth middleware configuration
#[derive(Debug, Clone)]
pub struct AuthMiddlewareConfig {
    /// Extract user info for every request
    pub always_extract_user: bool,
    /// Required auth methods (empty = any)
    pub allowed_methods: Vec<AuthMethod>,
    /// Fail on invalid token (vs treating as unauthenticated)
    pub strict_validation: bool,
}

impl Default for AuthMiddlewareConfig {
    fn default() -> Self {
        Self {
            always_extract_user: true,
            allowed_methods: vec![
                AuthMethod::Session,
                AuthMethod::JwtBearer,
                AuthMethod::ApiKey,
            ],
            strict_validation: false,
        }
    }
}

/// Request context for authentication
pub struct AuthRequest {
    pub authorization_header: Option<String>,
    pub session_cookie: Option<String>,
    pub api_key_header: Option<String>,
    pub bearer_token: Option<String>,
    pub ip_address: String,
    pub user_agent: Option<String>,
}

impl AuthRequest {
    pub fn new(ip_address: impl Into<String>) -> Self {
        Self {
            authorization_header: None,
            session_cookie: None,
            api_key_header: None,
            bearer_token: None,
            ip_address: ip_address.into(),
            user_agent: None,
        }
    }

    pub fn with_authorization(mut self, header: impl Into<String>) -> Self {
        self.authorization_header = Some(header.into());
        self
    }

    pub fn with_session_cookie(mut self, cookie: impl Into<String>) -> Self {
        self.session_cookie = Some(cookie.into());
        self
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key_header = Some(key.into());
        self
    }

    pub fn with_bearer_token(mut self, token: impl Into<String>) -> Self {
        self.bearer_token = Some(token.into());
        self
    }

    pub fn with_user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    /// Extract bearer token from Authorization header
    pub fn extract_bearer_token(&self) -> Option<String> {
        if let Some(token) = &self.bearer_token {
            return Some(token.clone());
        }

        self.authorization_header.as_ref().and_then(|header| {
            if header.starts_with("Bearer ") {
                Some(header[7..].to_string())
            } else {
                None
            }
        })
    }

    /// Extract basic auth credentials
    pub fn extract_basic_auth(&self) -> Option<(String, String)> {
        self.authorization_header.as_ref().and_then(|header| {
            if !header.starts_with("Basic ") {
                return None;
            }

            let encoded = &header[6..];
            let decoded = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded).ok()?;
            let decoded_str = String::from_utf8(decoded).ok()?;
            let parts: Vec<&str> = decoded_str.splitn(2, ':').collect();

            if parts.len() == 2 {
                Some((parts[0].to_string(), parts[1].to_string()))
            } else {
                None
            }
        })
    }
}

/// Authentication middleware
pub struct AuthMiddleware {
    jwt_manager: Option<JwtManager>,
    permission_checker: PermissionChecker,
    config: AuthMiddlewareConfig,
}

impl AuthMiddleware {
    pub fn new(config: AuthMiddlewareConfig) -> Self {
        Self {
            jwt_manager: None,
            permission_checker: PermissionChecker::with_default_roles(),
            config,
        }
    }

    pub fn with_jwt_manager(mut self, manager: JwtManager) -> Self {
        self.jwt_manager = Some(manager);
        self
    }

    pub fn with_permission_checker(mut self, checker: PermissionChecker) -> Self {
        self.permission_checker = checker;
        self
    }

    /// Authenticate a request
    pub async fn authenticate(&self, request: &AuthRequest) -> Result<Option<AuthContext>> {
        // Try JWT bearer token
        if let Some(token) = request.extract_bearer_token() {
            if let Some(ref jwt_manager) = self.jwt_manager {
                match jwt_manager.validate_access_token(&token) {
                    Ok(claims) => {
                        return Ok(Some(self.context_from_jwt_claims(&claims)?));
                    }
                    Err(e) if self.config.strict_validation => {
                        return Err(e);
                    }
                    Err(_) => {}
                }
            }
        }

        // Try API key
        if let Some(api_key) = &request.api_key_header {
            // API key validation would be handled by ApiKeyManager
            // For now, return None to indicate not authenticated via API key
            let _ = api_key;
        }

        // Try session cookie
        if let Some(session_cookie) = &request.session_cookie {
            // Session validation would be handled by SessionManager
            // For now, return None to indicate not authenticated via session
            let _ = session_cookie;
        }

        Ok(None)
    }

    /// Create auth context from JWT claims
    fn context_from_jwt_claims(&self, claims: &Claims) -> Result<AuthContext> {
        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| Error::InvalidToken {
            reason: "Invalid user ID in token".to_string(),
        })?;

        let mut context = AuthContext::new(user_id, AuthMethod::JwtBearer);

        if let Some(role) = &claims.role {
            context.roles.push(role.clone());

            // Get permissions from role
            let permissions = self.permission_checker.get_all_permissions(role);
            context.permissions = permissions.iter().map(|p| p.to_string()).collect();
        }

        if let Some(tenant_id) = &claims.tenant_id {
            if let Ok(tid) = Uuid::parse_str(tenant_id) {
                context.tenant_id = Some(tid);
            }
        }

        Ok(context)
    }

    /// Check if context meets requirement
    pub fn check_requirement(&self, context: &AuthContext, requirement: &AuthRequirement) -> Result<()> {
        match requirement {
            AuthRequirement::None => Ok(()),
            AuthRequirement::Authenticated => Ok(()),
            AuthRequirement::Role(role) => {
                if context.has_role(role) {
                    Ok(())
                } else {
                    Err(Error::Authorization {
                        action: format!("Role '{}' required", role),
                        required: role.clone(),
                    })
                }
            }
            AuthRequirement::AnyRole(roles) => {
                if context.has_any_role(&roles.iter().map(|s| s.as_str()).collect::<Vec<_>>()) {
                    Ok(())
                } else {
                    Err(Error::Authorization {
                        action: format!("One of roles {:?} required", roles),
                        required: roles.join(","),
                    })
                }
            }
            AuthRequirement::AllRoles(roles) => {
                if context.has_all_roles(&roles.iter().map(|s| s.as_str()).collect::<Vec<_>>()) {
                    Ok(())
                } else {
                    Err(Error::Authorization {
                        action: format!("All roles {:?} required", roles),
                        required: roles.join(","),
                    })
                }
            }
            AuthRequirement::Permission(permission) => {
                if context.has_permission(&permission.to_string()) {
                    Ok(())
                } else {
                    Err(Error::Authorization {
                        action: format!("Permission '{}' required", permission),
                        required: permission.to_string(),
                    })
                }
            }
            AuthRequirement::AnyPermission(permissions) => {
                for perm in permissions {
                    if context.has_permission(&perm.to_string()) {
                        return Ok(());
                    }
                }
                Err(Error::Authorization {
                    action: "Insufficient permissions".to_string(),
                    required: permissions.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(","),
                })
            }
            AuthRequirement::AllPermissions(permissions) => {
                for perm in permissions {
                    if !context.has_permission(&perm.to_string()) {
                        return Err(Error::Authorization {
                            action: format!("Permission '{}' required", perm),
                            required: perm.to_string(),
                        });
                    }
                }
                Ok(())
            }
            AuthRequirement::Custom(name) => {
                // Custom checks would be registered and looked up
                Err(Error::Internal {
                    message: format!("Custom check '{}' not implemented", name),
                    request_id: None,
                })
            }
        }
    }

    /// Full authentication and authorization check
    pub async fn authenticate_and_authorize(
        &self,
        request: &AuthRequest,
        requirement: &AuthRequirement,
    ) -> Result<AuthContext> {
        // If no auth required, allow with empty context
        if matches!(requirement, AuthRequirement::None) {
            return Ok(AuthContext::new(Uuid::nil(), AuthMethod::Session));
        }

        // Authenticate
        let context = self.authenticate(request).await?.ok_or_else(|| {
            Error::Authentication {
                message: "Authentication required".to_string(),
            }
        })?;

        // Check auth method is allowed
        if !self.config.allowed_methods.is_empty()
            && !self.config.allowed_methods.contains(&context.auth_method)
        {
            return Err(Error::Authentication {
                message: "Authentication method not allowed".to_string(),
            });
        }

        // Authorize
        self.check_requirement(&context, requirement)?;

        Ok(context)
    }

    /// Get config
    pub fn config(&self) -> &AuthMiddlewareConfig {
        &self.config
    }
}

/// Route protection builder
pub struct RouteProtection {
    requirement: AuthRequirement,
    rate_limit: Option<String>,
    audit: bool,
}

impl RouteProtection {
    pub fn new() -> Self {
        Self {
            requirement: AuthRequirement::None,
            rate_limit: None,
            audit: false,
        }
    }

    pub fn authenticated() -> Self {
        Self {
            requirement: AuthRequirement::Authenticated,
            rate_limit: None,
            audit: false,
        }
    }

    pub fn role(role: impl Into<String>) -> Self {
        Self {
            requirement: AuthRequirement::Role(role.into()),
            rate_limit: None,
            audit: false,
        }
    }

    pub fn any_role(roles: Vec<String>) -> Self {
        Self {
            requirement: AuthRequirement::AnyRole(roles),
            rate_limit: None,
            audit: false,
        }
    }

    pub fn permission(permission: Permission) -> Self {
        Self {
            requirement: AuthRequirement::Permission(permission),
            rate_limit: None,
            audit: false,
        }
    }

    pub fn with_rate_limit(mut self, tier: impl Into<String>) -> Self {
        self.rate_limit = Some(tier.into());
        self
    }

    pub fn with_audit(mut self) -> Self {
        self.audit = true;
        self
    }

    pub fn requirement(&self) -> &AuthRequirement {
        &self.requirement
    }

    pub fn should_audit(&self) -> bool {
        self.audit
    }

    pub fn rate_limit_tier(&self) -> Option<&str> {
        self.rate_limit.as_deref()
    }
}

impl Default for RouteProtection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_context() {
        let context = AuthContext::new(Uuid::now_v7(), AuthMethod::JwtBearer)
            .with_roles(vec!["admin".to_string(), "editor".to_string()])
            .with_permissions(["posts:read".to_string(), "posts:write".to_string()].into_iter().collect());

        assert!(context.has_role("admin"));
        assert!(context.has_role("editor"));
        assert!(!context.has_role("subscriber"));

        assert!(context.has_permission("posts:read"));
        assert!(!context.has_permission("users:delete"));
    }

    #[test]
    fn test_auth_request_bearer_extraction() {
        let request = AuthRequest::new("127.0.0.1")
            .with_authorization("Bearer test_token_here");

        let token = request.extract_bearer_token();
        assert_eq!(token, Some("test_token_here".to_string()));
    }

    #[test]
    fn test_auth_request_basic_extraction() {
        // user:password base64 encoded
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, "testuser:testpass");
        let request = AuthRequest::new("127.0.0.1")
            .with_authorization(format!("Basic {}", encoded));

        let (user, pass) = request.extract_basic_auth().unwrap();
        assert_eq!(user, "testuser");
        assert_eq!(pass, "testpass");
    }

    #[test]
    fn test_requirement_check() {
        let middleware = AuthMiddleware::new(AuthMiddlewareConfig::default());

        let context = AuthContext::new(Uuid::now_v7(), AuthMethod::JwtBearer)
            .with_roles(vec!["admin".to_string()]);

        // Should pass - has admin role
        assert!(middleware
            .check_requirement(&context, &AuthRequirement::Role("admin".to_string()))
            .is_ok());

        // Should fail - doesn't have editor role
        assert!(middleware
            .check_requirement(&context, &AuthRequirement::Role("editor".to_string()))
            .is_err());
    }

    #[test]
    fn test_route_protection_builder() {
        let protection = RouteProtection::role("admin")
            .with_rate_limit("strict")
            .with_audit();

        assert!(matches!(protection.requirement(), AuthRequirement::Role(r) if r == "admin"));
        assert_eq!(protection.rate_limit_tier(), Some("strict"));
        assert!(protection.should_audit());
    }
}
