//! Custom extractors for Axum.

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, Path, Query},
    http::{header, request::Parts, HeaderMap, StatusCode},
    Json,
};
use rustpress_auth::{Claims, JwtManager};
use rustpress_core::context::RequestContext;
use rustpress_core::types::Pagination;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::error::HttpError;
use crate::state::AppState;

/// Authenticated user extracted from JWT
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: Option<String>,
    pub roles: Vec<String>,
    pub claims: Claims,
}

impl AuthUser {
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    pub fn is_admin(&self) -> bool {
        self.has_role("administrator")
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = HttpError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        // Extract token from Authorization header
        let token = extract_bearer_token(&parts.headers)
            .ok_or_else(|| HttpError::unauthorized("Missing authorization header"))?;

        // Validate token
        let claims = app_state
            .jwt
            .validate_access_token(&token)
            .map_err(|_| HttpError::unauthorized("Invalid or expired token"))?;

        // Parse user ID from subject
        let id = Uuid::parse_str(&claims.sub)
            .map_err(|_| HttpError::unauthorized("Invalid user ID in token"))?;

        // Get email from custom claims if present
        let email = claims.custom.get("email")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Convert role to roles vector
        let roles: Vec<String> = claims.role.iter().cloned().collect();

        Ok(AuthUser {
            id,
            email,
            roles,
            claims,
        })
    }
}

/// Optional authenticated user (doesn't fail if no auth)
#[derive(Debug, Clone)]
pub struct MaybeAuthUser(pub Option<AuthUser>);

#[async_trait]
impl<S> FromRequestParts<S> for MaybeAuthUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = HttpError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match AuthUser::from_request_parts(parts, state).await {
            Ok(user) => Ok(MaybeAuthUser(Some(user))),
            Err(_) => Ok(MaybeAuthUser(None)),
        }
    }
}

/// Request context extractor
#[derive(Debug, Clone)]
pub struct ReqContext(pub RequestContext);

#[async_trait]
impl<S> FromRequestParts<S> for ReqContext
where
    S: Send + Sync,
{
    type Rejection = HttpError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Get path and method from request
        let path = parts.uri.path().to_string();
        let method = parts.method.to_string();

        let mut ctx = RequestContext::new(path, method);

        // Extract client IP
        if let Some(forwarded) = parts.headers.get("x-forwarded-for") {
            if let Ok(s) = forwarded.to_str() {
                if let Some(ip) = s.split(',').next() {
                    ctx = ctx.with_client_ip(ip.trim().to_string());
                }
            }
        }

        // Extract user agent
        if let Some(ua) = parts.headers.get(header::USER_AGENT) {
            if let Ok(s) = ua.to_str() {
                ctx = ctx.with_user_agent(s.to_string());
            }
        }

        Ok(ReqContext(ctx))
    }
}

/// Validated JSON body extractor
#[derive(Debug)]
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<S, T> axum::extract::FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = HttpError;

    async fn from_request(
        req: axum::http::Request<axum::body::Body>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| HttpError::bad_request(format!("Invalid JSON: {}", e)))?;

        value
            .validate()
            .map_err(|e| HttpError::unprocessable_entity(format!("Validation error: {}", e)))?;

        Ok(ValidatedJson(value))
    }
}

/// Pagination query parameters
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    20
}

impl PaginationParams {
    /// Convert to Pagination with a known total
    pub fn into_pagination(self, total: u64) -> Pagination {
        Pagination::new(self.page, self.per_page, total)
    }

    /// Calculate offset for database queries
    pub fn offset(&self) -> u64 {
        ((self.page.saturating_sub(1)) as u64) * (self.per_page as u64)
    }

    /// Get limit for database queries
    pub fn limit(&self) -> u64 {
        self.per_page as u64
    }
}

/// Pagination extractor
pub struct PaginatedQuery(pub PaginationParams);

#[async_trait]
impl<S> FromRequestParts<S> for PaginatedQuery
where
    S: Send + Sync,
{
    type Rejection = HttpError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(params): Query<PaginationParams> =
            Query::from_request_parts(parts, state)
                .await
                .map_err(|e| HttpError::bad_request(format!("Invalid pagination params: {}", e)))?;

        Ok(PaginatedQuery(params))
    }
}

/// UUID path parameter extractor
pub struct PathId(pub Uuid);

#[async_trait]
impl<S> FromRequestParts<S> for PathId
where
    S: Send + Sync,
{
    type Rejection = HttpError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(id): Path<Uuid> = Path::from_request_parts(parts, state)
            .await
            .map_err(|_| HttpError::bad_request("Invalid ID format"))?;

        Ok(PathId(id))
    }
}

/// Extract bearer token from Authorization header
fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

/// Require specific permission - helper function for permission checking
/// Instead of const generics (not supported for strings in stable Rust),
/// use this helper function in route handlers
pub async fn require_permission(
    user: &AuthUser,
    app_state: &AppState,
    resource: &str,
    action: &str,
) -> Result<(), HttpError> {
    if !app_state.permissions.can(&user.roles, resource, action) {
        return Err(HttpError::forbidden(format!(
            "Missing permission: {}:{}",
            resource, action
        )));
    }
    Ok(())
}

/// Content type extractor
#[derive(Debug, Clone)]
pub struct ContentType(pub mime::Mime);

#[async_trait]
impl<S> FromRequestParts<S> for ContentType
where
    S: Send + Sync,
{
    type Rejection = HttpError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let content_type = parts
            .headers
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(mime::APPLICATION_JSON);

        Ok(ContentType(content_type))
    }
}

/// Accept header extractor for content negotiation
#[derive(Debug, Clone)]
pub struct AcceptHeader(pub Vec<mime::Mime>);

#[async_trait]
impl<S> FromRequestParts<S> for AcceptHeader
where
    S: Send + Sync,
{
    type Rejection = HttpError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let accept = parts
            .headers
            .get(header::ACCEPT)
            .and_then(|v| v.to_str().ok())
            .map(|s| {
                s.split(',')
                    .filter_map(|part| {
                        let mime_str = part.split(';').next()?.trim();
                        mime_str.parse().ok()
                    })
                    .collect()
            })
            .unwrap_or_else(|| vec![mime::APPLICATION_JSON]);

        Ok(AcceptHeader(accept))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_defaults() {
        let params = PaginationParams {
            page: default_page(),
            per_page: default_per_page(),
        };
        assert_eq!(params.page, 1);
        assert_eq!(params.per_page, 20);
    }

    #[test]
    fn test_pagination_conversion() {
        let params = PaginationParams {
            page: 2,
            per_page: 50,
        };
        let pagination = params.into_pagination(100);
        assert_eq!(pagination.page, 2);
        assert_eq!(pagination.per_page, 50);
        assert_eq!(pagination.total, 100);
    }

    #[test]
    fn test_pagination_offset_limit() {
        let params = PaginationParams {
            page: 3,
            per_page: 25,
        };
        assert_eq!(params.offset(), 50);
        assert_eq!(params.limit(), 25);
    }
}
