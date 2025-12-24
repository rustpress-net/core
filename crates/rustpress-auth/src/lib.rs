//! # RustPress Auth
//!
//! Comprehensive authentication and authorization system for RustPress.
//!
//! ## Features
//!
//! - **Session Management** (Point 56): Server-side sessions with secure cookies
//! - **JWT Authentication** (Point 57): JSON Web Token based authentication
//! - **Refresh Token Rotation** (Point 58): Secure token refresh with family tracking
//! - **OAuth2 Provider** (Point 59): Authorization server for third-party apps
//! - **OAuth2 Client** (Point 60): Social login integration (Google, GitHub, etc.)
//! - **RBAC** (Point 61): Role-Based Access Control with permissions
//! - **Capability Permissions** (Point 62): Fine-grained permission system
//! - **Auth Middleware** (Point 63): Route-level authentication/authorization
//! - **Password Reset** (Point 64): Secure password reset flow
//! - **Email Verification** (Point 65): Email verification tokens
//! - **Two-Factor Auth** (Point 66): TOTP-based 2FA with recovery codes
//! - **API Keys** (Point 67): API key authentication with scopes
//! - **Rate Limiting** (Point 68): Per-user/IP rate limiting
//! - **Brute Force Protection** (Point 69, 77): Login attempt limiting with lockout
//! - **Session Invalidation** (Point 70): Cross-device session management
//! - **Remember Me** (Point 71): Extended session support
//! - **IP Filtering** (Point 72): Allowlist/blocklist with CIDR support
//! - **Audit Logging** (Point 73): Comprehensive auth event logging
//! - **CORS** (Point 74): Cross-Origin Resource Sharing configuration
//! - **CSRF Protection** (Point 75): Cross-Site Request Forgery prevention
//! - **Password Policies** (Point 76): Configurable password requirements
//! - **Password Change** (Point 78): Secure password change workflow
//! - **Impersonation** (Point 79): Admin user impersonation with audit trail
//! - **WebAuthn** (Point 80): Passwordless authentication with passkeys

// Core authentication modules
pub mod jwt;
pub mod session;
pub mod password;
pub mod tokens;
pub mod refresh_token;

// OAuth2 modules
pub mod oauth2_provider;
pub mod oauth2_client;

// Authorization modules
pub mod permission;
pub mod middleware;

// Two-factor authentication
pub mod totp;
pub mod webauthn;

// API authentication
pub mod api_key;

// Security modules
pub mod rate_limit;
pub mod brute_force;
pub mod ip_filter;
pub mod csrf;

// Audit and monitoring
pub mod audit;

// Admin features
pub mod impersonation;

// Re-exports for convenience
pub use jwt::{JwtManager, JwtConfig, Claims, TokenPair, TokenType};
pub use session::{Session, SessionManager, SessionConfig, SessionStore, SameSite};
pub use password::{PasswordHasher, PasswordValidator, PasswordRules, PasswordStrength};
pub use tokens::{TokenManager, TokenStore, SecureToken, PasswordResetToken, VerificationToken, TokenType as SecureTokenType};
pub use refresh_token::{RefreshTokenManager, RefreshToken, RefreshTokenConfig, RefreshTokenStore, RevokeReason};
pub use permission::{Permission, Role, PermissionChecker};
pub use middleware::{AuthMiddleware, AuthContext, AuthRequest, AuthRequirement, RouteProtection, AuthMethod};
pub use oauth2_provider::{OAuth2Provider, OAuth2Client as OAuth2RegisteredClient, OAuth2ProviderConfig, GrantType};
pub use oauth2_client::{OAuth2Client, OAuth2ClientProvider, OAuth2UserInfo, SocialConnection};
pub use totp::{TotpManager, TotpConfig, TotpSecret};
pub use webauthn::{WebAuthnManager, WebAuthnConfig, WebAuthnCredential, CredentialType};
pub use api_key::{ApiKeyManager, ApiKey, ApiKeyScope, ApiKeyConfig};
pub use rate_limit::{RateLimiter, RateLimitConfig, RateLimitResult};
pub use brute_force::{BruteForceProtection, BruteForceConfig, LockoutStatus, LoginAttempt};
pub use ip_filter::{IpFilter, IpFilterConfig, IpPattern, IpRule, IpRuleType};
pub use csrf::{CsrfProtection, CsrfConfig, CsrfToken};
pub use audit::{AuditLogger, AuthAuditEvent, AuthEventType, AuthEventBuilder, EventSeverity};
pub use impersonation::{ImpersonationManager, ImpersonationSession, ImpersonationConfig, ImpersonationRestrictions};

/// Prelude module for common imports
pub mod prelude {
    pub use crate::jwt::{JwtManager, Claims};
    pub use crate::session::{Session, SessionManager};
    pub use crate::password::{PasswordHasher, PasswordValidator};
    pub use crate::permission::{Permission, Role, PermissionChecker};
    pub use crate::middleware::{AuthMiddleware, AuthContext, AuthRequirement};
    pub use crate::totp::TotpManager;
    pub use crate::api_key::ApiKeyManager;
    pub use crate::rate_limit::RateLimiter;
    pub use crate::brute_force::BruteForceProtection;
    pub use crate::csrf::CsrfProtection;
    pub use crate::audit::AuditLogger;
}
