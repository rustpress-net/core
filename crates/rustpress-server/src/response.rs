//! Response types and helpers.

use axum::{
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use rustpress_core::api::{ApiResponse, PaginationMeta};

/// Success response wrapper
#[derive(Debug, Serialize)]
pub struct SuccessResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ResponseMeta>,
}

impl<T: Serialize> SuccessResponse<T> {
    pub fn new(data: T) -> Self {
        Self {
            success: true,
            data,
            meta: None,
        }
    }

    pub fn with_meta(mut self, meta: ResponseMeta) -> Self {
        self.meta = Some(meta);
        self
    }

    pub fn with_pagination(mut self, pagination: PaginationMeta) -> Self {
        self.meta = Some(ResponseMeta {
            pagination: Some(pagination),
            ..Default::default()
        });
        self
    }
}

impl<T: Serialize> IntoResponse for SuccessResponse<T> {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

/// Response metadata
#[derive(Debug, Default, Serialize)]
pub struct ResponseMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// Paginated list response
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub success: bool,
    pub data: Vec<T>,
    pub meta: PaginationMeta,
}

impl<T: Serialize> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: u64, page: u32, per_page: u32) -> Self {
        Self {
            success: true,
            data,
            meta: PaginationMeta::new(page, per_page, total),
        }
    }
}

impl<T: Serialize> IntoResponse for PaginatedResponse<T> {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

/// Created response (201)
pub struct Created<T: Serialize>(pub T);

impl<T: Serialize> IntoResponse for Created<T> {
    fn into_response(self) -> Response {
        (StatusCode::CREATED, Json(SuccessResponse::new(self.0))).into_response()
    }
}

/// Created response with location header
pub struct CreatedAt<T: Serialize> {
    pub data: T,
    pub location: String,
}

impl<T: Serialize> CreatedAt<T> {
    pub fn new(data: T, location: impl Into<String>) -> Self {
        Self {
            data,
            location: location.into(),
        }
    }
}

impl<T: Serialize> IntoResponse for CreatedAt<T> {
    fn into_response(self) -> Response {
        let mut response = (StatusCode::CREATED, Json(SuccessResponse::new(self.data))).into_response();
        if let Ok(location) = HeaderValue::from_str(&self.location) {
            response.headers_mut().insert(header::LOCATION, location);
        }
        response
    }
}

/// No content response (204)
pub struct NoContent;

impl IntoResponse for NoContent {
    fn into_response(self) -> Response {
        StatusCode::NO_CONTENT.into_response()
    }
}

/// Accepted response (202) - for async operations
#[derive(Debug, Serialize)]
pub struct Accepted<T: Serialize> {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_url: Option<String>,
}

impl<T: Serialize> Accepted<T> {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: None,
            status_url: None,
        }
    }

    pub fn with_data(mut self, data: T) -> Self {
        self.data = Some(data);
        self
    }

    pub fn with_status_url(mut self, url: impl Into<String>) -> Self {
        self.status_url = Some(url.into());
        self
    }
}

impl<T: Serialize> IntoResponse for Accepted<T> {
    fn into_response(self) -> Response {
        (StatusCode::ACCEPTED, Json(self)).into_response()
    }
}

/// File download response
pub struct FileDownload {
    pub data: Vec<u8>,
    pub filename: String,
    pub content_type: String,
}

impl FileDownload {
    pub fn new(data: Vec<u8>, filename: impl Into<String>, content_type: impl Into<String>) -> Self {
        Self {
            data,
            filename: filename.into(),
            content_type: content_type.into(),
        }
    }
}

impl IntoResponse for FileDownload {
    fn into_response(self) -> Response {
        let mut response = Response::new(axum::body::Body::from(self.data));

        let headers = response.headers_mut();

        if let Ok(content_type) = HeaderValue::from_str(&self.content_type) {
            headers.insert(header::CONTENT_TYPE, content_type);
        }

        let disposition = format!("attachment; filename=\"{}\"", self.filename);
        if let Ok(disposition) = HeaderValue::from_str(&disposition) {
            headers.insert(header::CONTENT_DISPOSITION, disposition);
        }

        response
    }
}

/// Streaming response for large data
pub struct StreamResponse {
    pub stream: tokio_stream::wrappers::ReceiverStream<Result<bytes::Bytes, std::io::Error>>,
    pub content_type: String,
}

/// Redirect response
pub struct Redirect {
    pub location: String,
    pub permanent: bool,
}

impl Redirect {
    pub fn temporary(location: impl Into<String>) -> Self {
        Self {
            location: location.into(),
            permanent: false,
        }
    }

    pub fn permanent(location: impl Into<String>) -> Self {
        Self {
            location: location.into(),
            permanent: true,
        }
    }
}

impl IntoResponse for Redirect {
    fn into_response(self) -> Response {
        let status = if self.permanent {
            StatusCode::MOVED_PERMANENTLY
        } else {
            StatusCode::FOUND
        };

        let mut response = status.into_response();
        if let Ok(location) = HeaderValue::from_str(&self.location) {
            response.headers_mut().insert(header::LOCATION, location);
        }
        response
    }
}

/// JSON response helper
pub fn json<T: Serialize>(data: T) -> impl IntoResponse {
    SuccessResponse::new(data)
}

/// Paginated JSON response helper
pub fn paginated<T: Serialize>(
    data: Vec<T>,
    total: u64,
    page: u32,
    per_page: u32,
) -> impl IntoResponse {
    PaginatedResponse::new(data, total, page, per_page)
}

/// Created response helper
pub fn created<T: Serialize>(data: T) -> impl IntoResponse {
    Created(data)
}

/// No content response helper
pub fn no_content() -> impl IntoResponse {
    NoContent
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_response() {
        let response = SuccessResponse::new("test data");
        assert!(response.success);
    }

    #[test]
    fn test_paginated_response() {
        let response = PaginatedResponse::new(vec![1, 2, 3], 100, 1, 10);
        assert!(response.success);
        assert_eq!(response.data.len(), 3);
        assert_eq!(response.meta.total, 100);
        assert_eq!(response.meta.total_pages, 10);
    }

    #[test]
    fn test_pagination_meta() {
        let meta = PaginationMeta::new(100, 1, 10);
        assert_eq!(meta.total, 100);
        assert_eq!(meta.page, 1);
        assert_eq!(meta.per_page, 10);
        assert_eq!(meta.total_pages, 10);
    }
}
