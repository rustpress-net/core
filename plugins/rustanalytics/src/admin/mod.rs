//! Admin Module for RustAnalytics
//!
//! This module contains admin-related functionality including settings management.

use serde::{Deserialize, Serialize};

use crate::models::AnalyticsSettings;

/// Admin settings form data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsFormData {
    // Google Analytics Configuration
    pub ga_property_id: String,
    pub ga_measurement_id: String,
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

    // Dashboard Preferences
    pub default_date_range: String,
    pub show_realtime_widget: bool,
    pub show_traffic_widget: bool,
    pub show_toppages_widget: bool,
    pub show_acquisition_widget: bool,

    // Report Settings
    pub report_email_enabled: bool,
    pub report_email_recipients: Vec<String>,
    pub report_frequency: String,

    // Privacy & Compliance
    pub gdpr_compliant: bool,
    pub ccpa_compliant: bool,
}

impl From<AnalyticsSettings> for SettingsFormData {
    fn from(settings: AnalyticsSettings) -> Self {
        Self {
            ga_property_id: settings.ga_property_id,
            ga_measurement_id: settings.ga_measurement_id,
            service_account_json: settings.service_account_json,
            enable_tracking: settings.enable_tracking,
            track_logged_in_users: settings.track_logged_in_users,
            track_admin_users: settings.track_admin_users,
            anonymize_ip: settings.anonymize_ip,
            respect_dnt: settings.respect_dnt,
            cookie_consent_required: settings.cookie_consent_required,
            enhanced_link_attribution: settings.enhanced_link_attribution,
            enhanced_ecommerce: settings.enhanced_ecommerce,
            default_date_range: format!("{:?}", settings.default_date_range).to_lowercase(),
            show_realtime_widget: settings.show_realtime_widget,
            show_traffic_widget: settings.show_traffic_widget,
            show_toppages_widget: settings.show_toppages_widget,
            show_acquisition_widget: settings.show_acquisition_widget,
            report_email_enabled: settings.report_email_enabled,
            report_email_recipients: settings.report_email_recipients,
            report_frequency: format!("{:?}", settings.report_frequency).to_lowercase(),
            gdpr_compliant: settings.gdpr_compliant,
            ccpa_compliant: settings.ccpa_compliant,
        }
    }
}

/// Connection test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub property_name: Option<String>,
    pub account_name: Option<String>,
    pub error: Option<String>,
    pub quota_remaining: Option<i32>,
}

/// Available properties response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailablePropertiesResponse {
    pub properties: Vec<PropertyOption>,
}

/// Property option for selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyOption {
    pub property_id: String,
    pub display_name: String,
    pub account_name: String,
}
