//! Google Analytics API Client
//!
//! This module provides a client for interacting with the Google Analytics Data API (GA4).

use std::time::{Duration, Instant};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::Utc;
use reqwest::{Client, StatusCode};
use rsa::pkcs8::DecodePrivateKey;
use rsa::signature::{SignatureEncoding, Signer};
use rsa::RsaPrivateKey;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use thiserror::Error;
use tracing::warn;

use crate::models::api::*;
use crate::models::{ServiceAccountCredentials, DateRange, AvailableProperty};

/// Google Analytics API base URLs
const GA_DATA_API_BASE: &str = "https://analyticsdata.googleapis.com/v1beta";
const GA_ADMIN_API_BASE: &str = "https://analyticsadmin.googleapis.com/v1beta";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

/// Scopes required for Google Analytics API
const GA_SCOPES: &[&str] = &[
    "https://www.googleapis.com/auth/analytics.readonly",
    "https://www.googleapis.com/auth/analytics",
];

/// Client errors
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("API request failed: {0}")]
    RequestFailed(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Rate limited, retry after {0} seconds")]
    RateLimited(u64),

    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),

    #[error("Property not found: {0}")]
    PropertyNotFound(String),

    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("RSA error: {0}")]
    RsaError(String),
}

/// JWT claims for service account authentication
#[derive(Debug, Serialize)]
struct JwtClaims {
    iss: String,
    scope: String,
    aud: String,
    exp: i64,
    iat: i64,
}

/// Access token with expiry
#[derive(Debug, Clone)]
struct AccessToken {
    token: String,
    expires_at: Instant,
}

/// Google Analytics API Client
pub struct GoogleAnalyticsClient {
    /// HTTP client
    http_client: Client,
    /// GA4 Property ID
    property_id: String,
    /// Service account credentials
    credentials: Option<ServiceAccountCredentials>,
    /// Current access token
    access_token: tokio::sync::RwLock<Option<AccessToken>>,
}

impl GoogleAnalyticsClient {
    /// Create a new Google Analytics client
    pub async fn new(
        property_id: String,
        service_account_json: Option<String>,
    ) -> Result<Self, ClientError> {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()?;

        let credentials = if let Some(json) = service_account_json {
            Some(serde_json::from_str::<ServiceAccountCredentials>(&json)
                .map_err(|e| ClientError::InvalidCredentials(e.to_string()))?)
        } else {
            None
        };

        let client = Self {
            http_client,
            property_id,
            credentials,
            access_token: tokio::sync::RwLock::new(None),
        };

        // Validate connection
        if client.credentials.is_some() {
            client.get_access_token().await?;
        }

        Ok(client)
    }

    /// Get the property ID
    pub fn property_id(&self) -> &str {
        &self.property_id
    }

    /// Get or refresh access token
    async fn get_access_token(&self) -> Result<String, ClientError> {
        // Check if we have a valid token
        {
            let token = self.access_token.read().await;
            if let Some(ref t) = *token {
                if t.expires_at > Instant::now() {
                    return Ok(t.token.clone());
                }
            }
        }

        // Need to refresh token
        let new_token = self.refresh_access_token().await?;

        let mut token = self.access_token.write().await;
        *token = Some(AccessToken {
            token: new_token.clone(),
            expires_at: Instant::now() + Duration::from_secs(3500), // Token valid for ~1 hour
        });

        Ok(new_token)
    }

    /// Refresh the access token using service account credentials
    async fn refresh_access_token(&self) -> Result<String, ClientError> {
        let credentials = self.credentials.as_ref()
            .ok_or_else(|| ClientError::InvalidCredentials("No credentials configured".to_string()))?;

        let now = Utc::now().timestamp();
        let claims = JwtClaims {
            iss: credentials.client_email.clone(),
            scope: GA_SCOPES.join(" "),
            aud: GOOGLE_TOKEN_URL.to_string(),
            iat: now,
            exp: now + 3600,
        };

        // Create JWT manually using RSA
        let jwt = self.create_jwt(&claims, &credentials.private_key)?;

        let response = self.http_client
            .post(GOOGLE_TOKEN_URL)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &jwt),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ClientError::AuthenticationFailed(error_text));
        }

        let token_response: TokenResponse = response.json().await?;
        Ok(token_response.access_token)
    }

    /// Create a JWT token using RS256
    fn create_jwt(&self, claims: &JwtClaims, private_key_pem: &str) -> Result<String, ClientError> {
        // Parse the private key
        let private_key = RsaPrivateKey::from_pkcs8_pem(private_key_pem)
            .map_err(|e| ClientError::RsaError(format!("Failed to parse private key: {}", e)))?;

        // Create header
        let header = serde_json::json!({
            "alg": "RS256",
            "typ": "JWT"
        });

        // Encode header and claims
        let header_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_string(&header)
            .map_err(|e| ClientError::RsaError(format!("Failed to serialize header: {}", e)))?);
        let claims_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_string(&claims)
            .map_err(|e| ClientError::RsaError(format!("Failed to serialize claims: {}", e)))?);

        // Create signing input
        let signing_input = format!("{}.{}", header_b64, claims_b64);

        // Sign using PKCS#1 v1.5 with SHA-256
        use rsa::pkcs1v15::SigningKey;
        let signing_key = SigningKey::<Sha256>::new(private_key);
        let signature = signing_key.sign(signing_input.as_bytes());

        // Encode signature
        let signature_b64 = URL_SAFE_NO_PAD.encode(signature.to_bytes());

        Ok(format!("{}.{}", signing_input, signature_b64))
    }

    /// Make an authenticated API request
    async fn request<T: for<'de> Deserialize<'de>>(
        &self,
        method: reqwest::Method,
        url: &str,
        body: Option<&impl Serialize>,
    ) -> Result<T, ClientError> {
        let token = self.get_access_token().await?;

        let mut request = self.http_client
            .request(method, url)
            .bearer_auth(&token);

        if let Some(b) = body {
            request = request.json(b);
        }

        let response = request.send().await?;
        let status = response.status();

        match status {
            StatusCode::OK | StatusCode::CREATED => {
                let data = response.json().await?;
                Ok(data)
            }
            StatusCode::TOO_MANY_REQUESTS => {
                let retry_after = response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(60);
                Err(ClientError::RateLimited(retry_after))
            }
            StatusCode::FORBIDDEN => {
                let error_text = response.text().await.unwrap_or_default();
                if error_text.contains("quota") {
                    Err(ClientError::QuotaExceeded(error_text))
                } else {
                    Err(ClientError::RequestFailed(error_text))
                }
            }
            StatusCode::NOT_FOUND => {
                Err(ClientError::PropertyNotFound(self.property_id.clone()))
            }
            _ => {
                let error_text = response.text().await.unwrap_or_default();
                Err(ClientError::RequestFailed(format!(
                    "Status {}: {}",
                    status, error_text
                )))
            }
        }
    }

    /// Run a report
    pub async fn run_report(&self, request: RunReportRequest) -> Result<RunReportResponse, ClientError> {
        let url = format!(
            "{}/properties/{}:runReport",
            GA_DATA_API_BASE, self.property_id
        );
        self.request(reqwest::Method::POST, &url, Some(&request)).await
    }

    /// Run a real-time report
    pub async fn run_realtime_report(
        &self,
        request: RunRealtimeReportRequest,
    ) -> Result<RunRealtimeReportResponse, ClientError> {
        let url = format!(
            "{}/properties/{}:runRealtimeReport",
            GA_DATA_API_BASE, self.property_id
        );
        self.request(reqwest::Method::POST, &url, Some(&request)).await
    }

    /// Run batch reports
    pub async fn batch_run_reports(
        &self,
        request: BatchRunReportsRequest,
    ) -> Result<BatchRunReportsResponse, ClientError> {
        let url = format!(
            "{}/properties/{}:batchRunReports",
            GA_DATA_API_BASE, self.property_id
        );
        self.request(reqwest::Method::POST, &url, Some(&request)).await
    }

    /// Run a pivot report
    pub async fn run_pivot_report(
        &self,
        request: RunPivotReportRequest,
    ) -> Result<RunPivotReportResponse, ClientError> {
        let url = format!(
            "{}/properties/{}:runPivotReport",
            GA_DATA_API_BASE, self.property_id
        );
        self.request(reqwest::Method::POST, &url, Some(&request)).await
    }

    /// Run a funnel report
    pub async fn run_funnel_report(
        &self,
        request: RunFunnelReportRequest,
    ) -> Result<serde_json::Value, ClientError> {
        let url = format!(
            "{}/properties/{}:runFunnelReport",
            GA_DATA_API_BASE, self.property_id
        );
        self.request(reqwest::Method::POST, &url, Some(&request)).await
    }

    /// Get metadata for available dimensions and metrics
    pub async fn get_metadata(&self) -> Result<Metadata, ClientError> {
        let url = format!(
            "{}/properties/{}/metadata",
            GA_DATA_API_BASE, self.property_id
        );
        self.request::<Metadata>(reqwest::Method::GET, &url, None::<&()>).await
    }

    /// List available account summaries
    pub async fn list_account_summaries(&self) -> Result<ListAccountSummariesResponse, ClientError> {
        let url = format!("{}/accountSummaries", GA_ADMIN_API_BASE);
        self.request::<ListAccountSummariesResponse>(reqwest::Method::GET, &url, None::<&()>).await
    }

    /// Get available properties
    pub async fn get_available_properties(&self) -> Result<Vec<AvailableProperty>, ClientError> {
        let response = self.list_account_summaries().await?;

        let mut properties = Vec::new();

        if let Some(accounts) = response.account_summaries {
            for account in accounts {
                let account_name = account.display_name.unwrap_or_default();
                let account_id = account.account.unwrap_or_default();

                if let Some(props) = account.property_summaries {
                    for prop in props {
                        if let Some(property_id) = prop.property {
                            // Extract numeric ID from "properties/123456789"
                            let numeric_id = property_id
                                .strip_prefix("properties/")
                                .unwrap_or(&property_id)
                                .to_string();

                            properties.push(AvailableProperty {
                                property_id: numeric_id,
                                display_name: prop.display_name.unwrap_or_default(),
                                account_name: account_name.clone(),
                                account_id: account_id.clone(),
                                time_zone: String::new(), // Would need additional API call
                                currency_code: String::new(),
                                industry_category: None,
                            });
                        }
                    }
                }
            }
        }

        Ok(properties)
    }

    /// Check API quota
    pub async fn check_quota(&self) -> Result<PropertyQuota, ClientError> {
        // Run a minimal report to get quota info
        let request = RunReportRequest {
            property: format!("properties/{}", self.property_id),
            date_ranges: vec![ApiDateRange {
                start_date: "today".to_string(),
                end_date: "today".to_string(),
                name: None,
            }],
            dimensions: None,
            metrics: vec![Metric {
                name: "sessions".to_string(),
                expression: None,
                invisible: None,
            }],
            dimension_filter: None,
            metric_filter: None,
            order_bys: None,
            offset: None,
            limit: Some(1),
            metric_aggregations: None,
            keep_empty_rows: None,
            return_property_quota: Some(true),
        };

        let response = self.run_report(request).await?;

        response.property_quota
            .ok_or_else(|| ClientError::InvalidResponse("No quota info returned".to_string()))
    }

    /// Test the connection to Google Analytics
    pub async fn test_connection(&self) -> Result<bool, ClientError> {
        // Try to get metadata to verify connection
        match self.get_metadata().await {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!("Connection test failed: {}", e);
                Err(e)
            }
        }
    }

    // Helper methods for building common reports

    /// Build date range for API request
    pub fn build_date_range(date_range: &DateRange) -> ApiDateRange {
        ApiDateRange {
            start_date: date_range.start_date.format("%Y-%m-%d").to_string(),
            end_date: date_range.end_date.format("%Y-%m-%d").to_string(),
            name: None,
        }
    }

    /// Build dimension from name
    pub fn dimension(name: &str) -> Dimension {
        Dimension {
            name: name.to_string(),
            dimension_expression: None,
        }
    }

    /// Build metric from name
    pub fn metric(name: &str) -> Metric {
        Metric {
            name: name.to_string(),
            expression: None,
            invisible: None,
        }
    }

    /// Build order by for metric descending
    pub fn order_by_metric_desc(metric_name: &str) -> OrderBy {
        OrderBy {
            desc: Some(true),
            metric: Some(MetricOrderBy {
                metric_name: metric_name.to_string(),
            }),
            dimension: None,
            pivot: None,
        }
    }

    /// Build string filter
    pub fn string_filter(
        field_name: &str,
        match_type: StringFilterMatchType,
        value: &str,
    ) -> FilterExpression {
        FilterExpression {
            and_group: None,
            or_group: None,
            not_expression: None,
            filter: Some(Filter {
                field_name: field_name.to_string(),
                string_filter: Some(StringFilter {
                    match_type,
                    value: value.to_string(),
                    case_sensitive: Some(false),
                }),
                in_list_filter: None,
                numeric_filter: None,
                between_filter: None,
            }),
        }
    }

    /// Build numeric filter
    pub fn numeric_filter(
        field_name: &str,
        operation: NumericFilterOperation,
        value: f64,
    ) -> FilterExpression {
        FilterExpression {
            and_group: None,
            or_group: None,
            not_expression: None,
            filter: Some(Filter {
                field_name: field_name.to_string(),
                string_filter: None,
                in_list_filter: None,
                numeric_filter: Some(NumericFilter {
                    operation,
                    value: NumericValue {
                        int64_value: None,
                        double_value: Some(value),
                    },
                }),
                between_filter: None,
            }),
        }
    }
}

impl std::fmt::Debug for GoogleAnalyticsClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GoogleAnalyticsClient")
            .field("property_id", &self.property_id)
            .field("has_credentials", &self.credentials.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_date_range() {
        let date_range = DateRange::last_n_days(7);
        let api_range = GoogleAnalyticsClient::build_date_range(&date_range);
        assert!(!api_range.start_date.is_empty());
        assert!(!api_range.end_date.is_empty());
    }

    #[test]
    fn test_dimension_builder() {
        let dim = GoogleAnalyticsClient::dimension("country");
        assert_eq!(dim.name, "country");
    }

    #[test]
    fn test_metric_builder() {
        let metric = GoogleAnalyticsClient::metric("sessions");
        assert_eq!(metric.name, "sessions");
    }
}
