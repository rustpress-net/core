//! Settings models for RustAnalytics plugin

use serde::{Deserialize, Serialize};
use validator::Validate;

/// Main plugin settings
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AnalyticsSettings {
    // Google Analytics Configuration
    #[validate(length(min = 1, message = "Property ID is required"))]
    pub ga_property_id: String,
    pub ga_measurement_id: String,
    pub ga_api_secret: String,
    pub service_account_json: Option<String>,

    // Tracking Options
    pub enable_tracking: bool,
    pub track_logged_in_users: bool,
    pub track_admin_users: bool,
    pub anonymize_ip: bool,
    pub respect_dnt: bool,
    pub cookie_consent_required: bool,
    pub enhanced_link_attribution: bool,
    pub enhanced_ecommerce: bool,

    // Data Retention
    #[validate(range(min = 1, max = 50))]
    pub data_retention_period: u32,
    #[validate(range(min = 1, max = 60))]
    pub cache_duration_minutes: u32,
    #[validate(range(min = 1, max = 24))]
    pub sync_frequency_hours: u32,

    // Dashboard Preferences
    pub default_date_range: DateRangePreset,
    pub show_realtime_widget: bool,
    pub show_traffic_widget: bool,
    pub show_toppages_widget: bool,
    pub show_acquisition_widget: bool,
    pub comparison_enabled: bool,

    // Advanced Options
    pub custom_dimensions: Vec<CustomDimension>,
    pub custom_metrics: Vec<CustomMetric>,
    pub excluded_ips: Vec<String>,
    pub excluded_user_roles: Vec<String>,
    pub cross_domain_tracking: Vec<String>,
    pub content_grouping: Vec<ContentGroup>,

    // Report Settings
    pub report_email_enabled: bool,
    pub report_email_recipients: Vec<String>,
    pub report_frequency: ReportFrequency,
    pub report_format: ReportFormat,

    // Privacy & Compliance
    pub gdpr_compliant: bool,
    pub ccpa_compliant: bool,
    pub data_processing_location: DataProcessingLocation,
}

impl Default for AnalyticsSettings {
    fn default() -> Self {
        Self {
            ga_property_id: String::new(),
            ga_measurement_id: String::new(),
            ga_api_secret: String::new(),
            service_account_json: None,
            enable_tracking: true,
            track_logged_in_users: true,
            track_admin_users: false,
            anonymize_ip: true,
            respect_dnt: true,
            cookie_consent_required: false,
            enhanced_link_attribution: true,
            enhanced_ecommerce: false,
            data_retention_period: 26,
            cache_duration_minutes: 15,
            sync_frequency_hours: 1,
            default_date_range: DateRangePreset::Last30Days,
            show_realtime_widget: true,
            show_traffic_widget: true,
            show_toppages_widget: true,
            show_acquisition_widget: true,
            comparison_enabled: true,
            custom_dimensions: Vec::new(),
            custom_metrics: Vec::new(),
            excluded_ips: Vec::new(),
            excluded_user_roles: vec!["administrator".to_string()],
            cross_domain_tracking: Vec::new(),
            content_grouping: Vec::new(),
            report_email_enabled: false,
            report_email_recipients: Vec::new(),
            report_frequency: ReportFrequency::Weekly,
            report_format: ReportFormat::Pdf,
            gdpr_compliant: true,
            ccpa_compliant: true,
            data_processing_location: DataProcessingLocation::Auto,
        }
    }
}

/// Date range presets for analytics queries
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DateRangePreset {
    Today,
    Yesterday,
    Last7Days,
    Last14Days,
    Last28Days,
    Last30Days,
    Last90Days,
    Last365Days,
    ThisMonth,
    LastMonth,
    ThisQuarter,
    LastQuarter,
    ThisYear,
    LastYear,
    Custom,
}

/// Custom dimension configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomDimension {
    pub index: u32,
    pub name: String,
    pub scope: DimensionScope,
    pub active: bool,
}

/// Scope for custom dimensions
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DimensionScope {
    Hit,
    Session,
    User,
    Product,
}

/// Custom metric configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMetric {
    pub index: u32,
    pub name: String,
    pub scope: MetricScope,
    pub formatting_type: MetricFormattingType,
    pub active: bool,
}

/// Scope for custom metrics
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricScope {
    Hit,
    Product,
}

/// Formatting type for custom metrics
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricFormattingType {
    Integer,
    Currency,
    Time,
    Float,
    Percent,
}

/// Content grouping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentGroup {
    pub index: u32,
    pub name: String,
    pub rules: Vec<ContentGroupRule>,
}

/// Rule for content grouping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentGroupRule {
    pub pattern: String,
    pub pattern_type: PatternType,
    pub group_name: String,
}

/// Pattern type for content grouping
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternType {
    Contains,
    StartsWith,
    EndsWith,
    Regex,
    Exact,
}

/// Report frequency options
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ReportFrequency {
    Daily,
    Weekly,
    BiWeekly,
    Monthly,
    Quarterly,
}

/// Report format options
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ReportFormat {
    Pdf,
    Csv,
    Excel,
    Html,
    Json,
}

/// Data processing location options
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DataProcessingLocation {
    Auto,
    Us,
    Eu,
    Asia,
}

/// Service account credentials for Google Analytics API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAccountCredentials {
    pub r#type: String,
    pub project_id: String,
    pub private_key_id: String,
    pub private_key: String,
    pub client_email: String,
    pub client_id: String,
    pub auth_uri: String,
    pub token_uri: String,
    pub auth_provider_x509_cert_url: String,
    pub client_x509_cert_url: String,
}

/// Connection status for Google Analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub connected: bool,
    pub property_name: Option<String>,
    pub property_id: Option<String>,
    pub account_name: Option<String>,
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
    pub error: Option<String>,
}

/// Available GA4 properties for selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableProperty {
    pub property_id: String,
    pub display_name: String,
    pub account_name: String,
    pub account_id: String,
    pub time_zone: String,
    pub currency_code: String,
    pub industry_category: Option<String>,
}
