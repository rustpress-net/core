//! API types and versioning support.
//!
//! Provides infrastructure for API versioning and content negotiation.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// API version
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ApiVersion {
    pub major: u16,
    pub minor: u16,
}

impl ApiVersion {
    pub const V1: ApiVersion = ApiVersion { major: 1, minor: 0 };
    pub const V2: ApiVersion = ApiVersion { major: 2, minor: 0 };

    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    pub fn is_compatible_with(&self, other: &ApiVersion) -> bool {
        self.major == other.major && self.minor >= other.minor
    }
}

impl Default for ApiVersion {
    fn default() -> Self {
        Self::V1
    }
}

impl std::fmt::Display for ApiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}.{}", self.major, self.minor)
    }
}

impl FromStr for ApiVersion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim_start_matches('v').trim_start_matches('V');
        let parts: Vec<&str> = s.split('.').collect();

        match parts.as_slice() {
            [major] => Ok(ApiVersion {
                major: major.parse().map_err(|_| "Invalid major version")?,
                minor: 0,
            }),
            [major, minor] => Ok(ApiVersion {
                major: major.parse().map_err(|_| "Invalid major version")?,
                minor: minor.parse().map_err(|_| "Invalid minor version")?,
            }),
            _ => Err("Invalid version format".to_string()),
        }
    }
}

impl Serialize for ApiVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ApiVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ApiVersion::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// Content type for response negotiation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContentType {
    Json,
    Xml,
    Html,
    Yaml,
    MessagePack,
    Csv,
    Text,
}

impl ContentType {
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Json => "application/json",
            Self::Xml => "application/xml",
            Self::Html => "text/html",
            Self::Yaml => "application/yaml",
            Self::MessagePack => "application/msgpack",
            Self::Csv => "text/csv",
            Self::Text => "text/plain",
        }
    }

    pub fn from_mime(mime: &str) -> Option<Self> {
        match mime {
            "application/json" | "text/json" => Some(Self::Json),
            "application/xml" | "text/xml" => Some(Self::Xml),
            "text/html" => Some(Self::Html),
            "application/yaml" | "text/yaml" => Some(Self::Yaml),
            "application/msgpack" | "application/x-msgpack" => Some(Self::MessagePack),
            "text/csv" => Some(Self::Csv),
            "text/plain" => Some(Self::Text),
            _ => None,
        }
    }

    pub fn from_accept_header(accept: &str) -> Vec<Self> {
        accept
            .split(',')
            .filter_map(|part| {
                let mime = part.split(';').next()?.trim();
                Self::from_mime(mime)
            })
            .collect()
    }
}

impl Default for ContentType {
    fn default() -> Self {
        Self::Json
    }
}

/// Standard API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ResponseMeta>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            meta: None,
        }
    }

    pub fn error(error: ApiError) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            meta: None,
        }
    }

    pub fn with_meta(mut self, meta: ResponseMeta) -> Self {
        self.meta = Some(meta);
        self
    }
}

impl<T: Default> Default for ApiResponse<T> {
    fn default() -> Self {
        Self::success(T::default())
    }
}

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl ApiError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
            request_id: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_request_id(mut self, id: impl Into<String>) -> Self {
        self.request_id = Some(id.into());
        self
    }

    pub fn not_found(resource: &str) -> Self {
        Self::new("NOT_FOUND", format!("{} not found", resource))
    }

    pub fn unauthorized() -> Self {
        Self::new("UNAUTHORIZED", "Authentication required")
    }

    pub fn forbidden() -> Self {
        Self::new("FORBIDDEN", "Insufficient permissions")
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new("BAD_REQUEST", message)
    }

    pub fn internal() -> Self {
        Self::new("INTERNAL_ERROR", "An internal error occurred")
    }

    pub fn rate_limited(retry_after: u64) -> Self {
        Self::new("RATE_LIMITED", format!("Rate limit exceeded. Retry after {} seconds", retry_after))
    }
}

impl From<crate::error::Error> for ApiError {
    fn from(err: crate::error::Error) -> Self {
        Self::new(err.error_code(), err.to_string())
    }
}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_time_ms: Option<u64>,
}

impl ResponseMeta {
    pub fn new() -> Self {
        Self {
            pagination: None,
            request_id: None,
            version: None,
            processing_time_ms: None,
        }
    }

    pub fn with_pagination(mut self, pagination: PaginationMeta) -> Self {
        self.pagination = Some(pagination);
        self
    }

    pub fn with_request_id(mut self, id: impl Into<String>) -> Self {
        self.request_id = Some(id.into());
        self
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn with_processing_time(mut self, ms: u64) -> Self {
        self.processing_time_ms = Some(ms);
        self
    }
}

impl Default for ResponseMeta {
    fn default() -> Self {
        Self::new()
    }
}

/// Pagination metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
    pub total_pages: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_page: Option<u32>,
}

impl PaginationMeta {
    pub fn new(page: u32, per_page: u32, total: u64) -> Self {
        let total_pages = (total as f64 / per_page as f64).ceil() as u32;
        let next_page = if page < total_pages {
            Some(page + 1)
        } else {
            None
        };
        let prev_page = if page > 1 { Some(page - 1) } else { None };

        Self {
            page,
            per_page,
            total,
            total_pages,
            next_page,
            prev_page,
        }
    }
}

impl<T> From<crate::service::ListResult<T>> for PaginationMeta {
    fn from(result: crate::service::ListResult<T>) -> Self {
        Self::new(result.page, result.per_page, result.total)
    }
}

/// Paginated response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub pagination: PaginationMeta,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, pagination: PaginationMeta) -> Self {
        Self { items, pagination }
    }
}

impl<T> From<crate::service::ListResult<T>> for PaginatedResponse<T> {
    fn from(result: crate::service::ListResult<T>) -> Self {
        let pagination = PaginationMeta::new(result.page, result.per_page, result.total);
        Self {
            items: result.items,
            pagination,
        }
    }
}

/// Request query parameters for listing
#[derive(Debug, Clone, Deserialize)]
pub struct ListQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
    pub sort_by: Option<String>,
    #[serde(default)]
    pub sort_order: SortOrder,
    pub search: Option<String>,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    20
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    #[default]
    Desc,
}

impl From<ListQuery> for crate::service::ListParams {
    fn from(query: ListQuery) -> Self {
        let mut params = crate::service::ListParams::new()
            .page(query.page)
            .per_page(query.per_page.min(100)); // Cap at 100

        if let Some(sort_by) = query.sort_by {
            params = params.sort_by(sort_by);
        }

        params = params.sort_order(match query.sort_order {
            SortOrder::Asc => crate::service::SortOrder::Asc,
            SortOrder::Desc => crate::service::SortOrder::Desc,
        });

        if let Some(search) = query.search {
            params = params.search(search);
        }

        params
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_version_parsing() {
        assert_eq!("v1".parse::<ApiVersion>().unwrap(), ApiVersion::V1);
        assert_eq!("v1.0".parse::<ApiVersion>().unwrap(), ApiVersion::V1);
        assert_eq!(
            "v2.1".parse::<ApiVersion>().unwrap(),
            ApiVersion::new(2, 1)
        );
    }

    #[test]
    fn test_api_version_compatibility() {
        let v1_0 = ApiVersion::new(1, 0);
        let v1_1 = ApiVersion::new(1, 1);
        let v2_0 = ApiVersion::new(2, 0);

        assert!(v1_1.is_compatible_with(&v1_0));
        assert!(!v1_0.is_compatible_with(&v1_1));
        assert!(!v2_0.is_compatible_with(&v1_1));
    }

    #[test]
    fn test_content_type() {
        assert_eq!(
            ContentType::from_mime("application/json"),
            Some(ContentType::Json)
        );
        assert_eq!(
            ContentType::from_accept_header("application/json, text/html"),
            vec![ContentType::Json, ContentType::Html]
        );
    }

    #[test]
    fn test_api_response() {
        let response = ApiResponse::success("hello");
        assert!(response.success);
        assert_eq!(response.data, Some("hello"));

        let error = ApiResponse::<()>::error(ApiError::not_found("Post"));
        assert!(!error.success);
        assert!(error.error.is_some());
    }

    #[test]
    fn test_pagination_meta() {
        let meta = PaginationMeta::new(2, 10, 45);
        assert_eq!(meta.total_pages, 5);
        assert_eq!(meta.next_page, Some(3));
        assert_eq!(meta.prev_page, Some(1));
    }
}
