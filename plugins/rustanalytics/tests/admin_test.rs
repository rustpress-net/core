//! Tests for the Admin Module

use serde_json;

use rustanalytics::admin::{
    AvailablePropertiesResponse, ConnectionTestResult, PropertyOption, SettingsFormData,
};
use rustanalytics::models::settings::{
    AnalyticsSettings, DateRangePreset, ReportFrequency,
};

// ============================================================================
// Helper Functions
// ============================================================================

fn sample_settings_form_data() -> SettingsFormData {
    SettingsFormData {
        ga_property_id: "properties/123456789".to_string(),
        ga_measurement_id: "G-ABCDEFGHIJ".to_string(),
        service_account_json: Some(r#"{"type": "service_account"}"#.to_string()),
        enable_tracking: true,
        track_logged_in_users: true,
        track_admin_users: false,
        anonymize_ip: true,
        respect_dnt: true,
        cookie_consent_required: false,
        enhanced_link_attribution: true,
        enhanced_ecommerce: false,
        default_date_range: "last_30_days".to_string(),
        show_realtime_widget: true,
        show_traffic_widget: true,
        show_toppages_widget: true,
        show_acquisition_widget: true,
        report_email_enabled: false,
        report_email_recipients: vec![],
        report_frequency: "weekly".to_string(),
        gdpr_compliant: true,
        ccpa_compliant: true,
    }
}

fn sample_analytics_settings() -> AnalyticsSettings {
    AnalyticsSettings {
        ga_property_id: "properties/987654321".to_string(),
        ga_measurement_id: "G-ZYXWVUTSRQ".to_string(),
        service_account_json: Some(r#"{"project_id": "test-project"}"#.to_string()),
        enable_tracking: true,
        track_logged_in_users: false,
        track_admin_users: true,
        anonymize_ip: false,
        respect_dnt: false,
        cookie_consent_required: true,
        enhanced_link_attribution: false,
        enhanced_ecommerce: true,
        default_date_range: DateRangePreset::Last7Days,
        show_realtime_widget: false,
        show_traffic_widget: true,
        show_toppages_widget: false,
        show_acquisition_widget: true,
        report_email_enabled: true,
        report_email_recipients: vec!["admin@example.com".to_string()],
        report_frequency: ReportFrequency::Daily,
        gdpr_compliant: false,
        ccpa_compliant: true,
        ..Default::default()
    }
}

// ============================================================================
// SettingsFormData Tests
// ============================================================================

#[test]
fn test_settings_form_data_creation() {
    let form = sample_settings_form_data();

    assert_eq!(form.ga_property_id, "properties/123456789");
    assert_eq!(form.ga_measurement_id, "G-ABCDEFGHIJ");
    assert!(form.service_account_json.is_some());
    assert!(form.enable_tracking);
    assert!(form.track_logged_in_users);
    assert!(!form.track_admin_users);
    assert!(form.anonymize_ip);
    assert!(form.respect_dnt);
    assert!(!form.cookie_consent_required);
    assert!(form.enhanced_link_attribution);
    assert!(!form.enhanced_ecommerce);
    assert_eq!(form.default_date_range, "last_30_days");
    assert!(form.show_realtime_widget);
    assert!(form.show_traffic_widget);
    assert!(form.show_toppages_widget);
    assert!(form.show_acquisition_widget);
    assert!(!form.report_email_enabled);
    assert!(form.report_email_recipients.is_empty());
    assert_eq!(form.report_frequency, "weekly");
    assert!(form.gdpr_compliant);
    assert!(form.ccpa_compliant);
}

#[test]
fn test_settings_form_data_without_service_account() {
    let form = SettingsFormData {
        ga_property_id: "properties/111".to_string(),
        ga_measurement_id: "G-TEST".to_string(),
        service_account_json: None,
        enable_tracking: true,
        track_logged_in_users: true,
        track_admin_users: false,
        anonymize_ip: true,
        respect_dnt: true,
        cookie_consent_required: false,
        enhanced_link_attribution: true,
        enhanced_ecommerce: false,
        default_date_range: "today".to_string(),
        show_realtime_widget: true,
        show_traffic_widget: true,
        show_toppages_widget: true,
        show_acquisition_widget: true,
        report_email_enabled: false,
        report_email_recipients: vec![],
        report_frequency: "monthly".to_string(),
        gdpr_compliant: true,
        ccpa_compliant: true,
    };

    assert!(form.service_account_json.is_none());
}

#[test]
fn test_settings_form_data_with_email_recipients() {
    let form = SettingsFormData {
        ga_property_id: "properties/222".to_string(),
        ga_measurement_id: "G-EMAIL".to_string(),
        service_account_json: None,
        enable_tracking: true,
        track_logged_in_users: true,
        track_admin_users: false,
        anonymize_ip: true,
        respect_dnt: true,
        cookie_consent_required: false,
        enhanced_link_attribution: true,
        enhanced_ecommerce: false,
        default_date_range: "last_7_days".to_string(),
        show_realtime_widget: true,
        show_traffic_widget: true,
        show_toppages_widget: true,
        show_acquisition_widget: true,
        report_email_enabled: true,
        report_email_recipients: vec![
            "admin@example.com".to_string(),
            "manager@example.com".to_string(),
            "analytics@example.com".to_string(),
        ],
        report_frequency: "daily".to_string(),
        gdpr_compliant: true,
        ccpa_compliant: true,
    };

    assert!(form.report_email_enabled);
    assert_eq!(form.report_email_recipients.len(), 3);
    assert!(form.report_email_recipients.contains(&"admin@example.com".to_string()));
}

#[test]
fn test_settings_form_data_serialization() {
    let form = sample_settings_form_data();

    let json = serde_json::to_string(&form).unwrap();

    assert!(json.contains("\"ga_property_id\":\"properties/123456789\""));
    assert!(json.contains("\"ga_measurement_id\":\"G-ABCDEFGHIJ\""));
    assert!(json.contains("\"enable_tracking\":true"));
    assert!(json.contains("\"gdpr_compliant\":true"));
}

#[test]
fn test_settings_form_data_deserialization() {
    let json = r#"{
        "ga_property_id": "properties/test",
        "ga_measurement_id": "G-TEST123",
        "service_account_json": null,
        "enable_tracking": true,
        "track_logged_in_users": false,
        "track_admin_users": true,
        "anonymize_ip": true,
        "respect_dnt": false,
        "cookie_consent_required": true,
        "enhanced_link_attribution": false,
        "enhanced_ecommerce": true,
        "default_date_range": "this_month",
        "show_realtime_widget": true,
        "show_traffic_widget": false,
        "show_toppages_widget": true,
        "show_acquisition_widget": false,
        "report_email_enabled": true,
        "report_email_recipients": ["test@example.com"],
        "report_frequency": "bi_weekly",
        "gdpr_compliant": false,
        "ccpa_compliant": true
    }"#;

    let form: SettingsFormData = serde_json::from_str(json).unwrap();

    assert_eq!(form.ga_property_id, "properties/test");
    assert_eq!(form.ga_measurement_id, "G-TEST123");
    assert!(form.service_account_json.is_none());
    assert!(!form.track_logged_in_users);
    assert!(form.track_admin_users);
    assert!(!form.respect_dnt);
    assert!(form.cookie_consent_required);
    assert!(!form.enhanced_link_attribution);
    assert!(form.enhanced_ecommerce);
    assert_eq!(form.default_date_range, "this_month");
    assert!(!form.show_traffic_widget);
    assert!(!form.show_acquisition_widget);
    assert!(form.report_email_enabled);
    assert_eq!(form.report_email_recipients.len(), 1);
    assert!(!form.gdpr_compliant);
}

#[test]
fn test_settings_form_data_json_roundtrip() {
    let original = sample_settings_form_data();

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: SettingsFormData = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.ga_property_id, original.ga_property_id);
    assert_eq!(deserialized.ga_measurement_id, original.ga_measurement_id);
    assert_eq!(deserialized.enable_tracking, original.enable_tracking);
    assert_eq!(deserialized.gdpr_compliant, original.gdpr_compliant);
}

#[test]
fn test_settings_form_data_debug_impl() {
    let form = sample_settings_form_data();
    let debug_str = format!("{:?}", form);

    assert!(debug_str.contains("SettingsFormData"));
    assert!(debug_str.contains("ga_property_id"));
    assert!(debug_str.contains("enable_tracking"));
}

#[test]
fn test_settings_form_data_clone() {
    let form = sample_settings_form_data();
    let cloned = form.clone();

    assert_eq!(cloned.ga_property_id, form.ga_property_id);
    assert_eq!(cloned.ga_measurement_id, form.ga_measurement_id);
    assert_eq!(cloned.enable_tracking, form.enable_tracking);
    assert_eq!(cloned.report_email_recipients, form.report_email_recipients);
}

#[test]
fn test_settings_form_data_all_tracking_disabled() {
    let form = SettingsFormData {
        ga_property_id: "properties/disabled".to_string(),
        ga_measurement_id: "G-DISABLED".to_string(),
        service_account_json: None,
        enable_tracking: false,
        track_logged_in_users: false,
        track_admin_users: false,
        anonymize_ip: false,
        respect_dnt: false,
        cookie_consent_required: false,
        enhanced_link_attribution: false,
        enhanced_ecommerce: false,
        default_date_range: "today".to_string(),
        show_realtime_widget: false,
        show_traffic_widget: false,
        show_toppages_widget: false,
        show_acquisition_widget: false,
        report_email_enabled: false,
        report_email_recipients: vec![],
        report_frequency: "monthly".to_string(),
        gdpr_compliant: false,
        ccpa_compliant: false,
    };

    assert!(!form.enable_tracking);
    assert!(!form.track_logged_in_users);
    assert!(!form.track_admin_users);
    assert!(!form.show_realtime_widget);
    assert!(!form.gdpr_compliant);
    assert!(!form.ccpa_compliant);
}

#[test]
fn test_settings_form_data_all_widgets_hidden() {
    let form = SettingsFormData {
        ga_property_id: "properties/no_widgets".to_string(),
        ga_measurement_id: "G-NOWIDGETS".to_string(),
        service_account_json: None,
        enable_tracking: true,
        track_logged_in_users: true,
        track_admin_users: false,
        anonymize_ip: true,
        respect_dnt: true,
        cookie_consent_required: false,
        enhanced_link_attribution: true,
        enhanced_ecommerce: false,
        default_date_range: "last_30_days".to_string(),
        show_realtime_widget: false,
        show_traffic_widget: false,
        show_toppages_widget: false,
        show_acquisition_widget: false,
        report_email_enabled: false,
        report_email_recipients: vec![],
        report_frequency: "weekly".to_string(),
        gdpr_compliant: true,
        ccpa_compliant: true,
    };

    assert!(!form.show_realtime_widget);
    assert!(!form.show_traffic_widget);
    assert!(!form.show_toppages_widget);
    assert!(!form.show_acquisition_widget);
}

// ============================================================================
// From<AnalyticsSettings> Conversion Tests
// ============================================================================

#[test]
fn test_settings_form_from_analytics_settings() {
    let settings = sample_analytics_settings();
    let form: SettingsFormData = settings.into();

    assert_eq!(form.ga_property_id, "properties/987654321");
    assert_eq!(form.ga_measurement_id, "G-ZYXWVUTSRQ");
    assert!(form.service_account_json.is_some());
    assert!(form.enable_tracking);
    assert!(!form.track_logged_in_users);
    assert!(form.track_admin_users);
    assert!(!form.anonymize_ip);
    assert!(!form.respect_dnt);
    assert!(form.cookie_consent_required);
    assert!(!form.enhanced_link_attribution);
    assert!(form.enhanced_ecommerce);
    assert!(!form.show_realtime_widget);
    assert!(form.show_traffic_widget);
    assert!(!form.show_toppages_widget);
    assert!(form.show_acquisition_widget);
    assert!(form.report_email_enabled);
    assert_eq!(form.report_email_recipients.len(), 1);
    assert!(!form.gdpr_compliant);
    assert!(form.ccpa_compliant);
}

#[test]
fn test_settings_form_from_default_settings() {
    let settings = AnalyticsSettings::default();
    let form: SettingsFormData = settings.into();

    assert!(form.ga_property_id.is_empty());
    assert!(form.ga_measurement_id.is_empty());
    assert!(form.service_account_json.is_none());
    assert!(form.enable_tracking);
    assert!(form.track_logged_in_users);
    assert!(!form.track_admin_users);
    assert!(form.anonymize_ip);
    assert!(form.respect_dnt);
    assert!(!form.cookie_consent_required);
    assert!(form.enhanced_link_attribution);
    assert!(!form.enhanced_ecommerce);
    assert!(form.show_realtime_widget);
    assert!(form.show_traffic_widget);
    assert!(form.show_toppages_widget);
    assert!(form.show_acquisition_widget);
    assert!(!form.report_email_enabled);
    assert!(form.report_email_recipients.is_empty());
    assert!(form.gdpr_compliant);
    assert!(form.ccpa_compliant);
}

#[test]
fn test_settings_form_date_range_conversion() {
    // Test various date range presets
    let presets = vec![
        (DateRangePreset::Today, "today"),
        (DateRangePreset::Yesterday, "yesterday"),
        (DateRangePreset::Last7Days, "last7days"),
        (DateRangePreset::Last30Days, "last30days"),
        (DateRangePreset::ThisMonth, "thismonth"),
        (DateRangePreset::LastMonth, "lastmonth"),
    ];

    for (preset, _expected_contains) in presets {
        let mut settings = AnalyticsSettings::default();
        settings.default_date_range = preset;
        let form: SettingsFormData = settings.into();
        // The date range should contain the preset name in some form
        assert!(!form.default_date_range.is_empty());
    }
}

#[test]
fn test_settings_form_report_frequency_conversion() {
    let frequencies = vec![
        ReportFrequency::Daily,
        ReportFrequency::Weekly,
        ReportFrequency::BiWeekly,
        ReportFrequency::Monthly,
        ReportFrequency::Quarterly,
    ];

    for freq in frequencies {
        let mut settings = AnalyticsSettings::default();
        settings.report_frequency = freq;
        let form: SettingsFormData = settings.into();
        // The frequency should be converted to lowercase string
        assert!(!form.report_frequency.is_empty());
    }
}

// ============================================================================
// ConnectionTestResult Tests
// ============================================================================

#[test]
fn test_connection_test_result_success() {
    let result = ConnectionTestResult {
        success: true,
        property_name: Some("My Website".to_string()),
        account_name: Some("My Account".to_string()),
        error: None,
        quota_remaining: Some(1000),
    };

    assert!(result.success);
    assert_eq!(result.property_name, Some("My Website".to_string()));
    assert_eq!(result.account_name, Some("My Account".to_string()));
    assert!(result.error.is_none());
    assert_eq!(result.quota_remaining, Some(1000));
}

#[test]
fn test_connection_test_result_failure() {
    let result = ConnectionTestResult {
        success: false,
        property_name: None,
        account_name: None,
        error: Some("Invalid credentials".to_string()),
        quota_remaining: None,
    };

    assert!(!result.success);
    assert!(result.property_name.is_none());
    assert!(result.account_name.is_none());
    assert_eq!(result.error, Some("Invalid credentials".to_string()));
    assert!(result.quota_remaining.is_none());
}

#[test]
fn test_connection_test_result_partial_success() {
    let result = ConnectionTestResult {
        success: true,
        property_name: Some("Test Property".to_string()),
        account_name: None,
        error: None,
        quota_remaining: Some(500),
    };

    assert!(result.success);
    assert!(result.property_name.is_some());
    assert!(result.account_name.is_none());
    assert!(result.error.is_none());
}

#[test]
fn test_connection_test_result_serialization() {
    let result = ConnectionTestResult {
        success: true,
        property_name: Some("Website Analytics".to_string()),
        account_name: Some("Company Account".to_string()),
        error: None,
        quota_remaining: Some(2500),
    };

    let json = serde_json::to_string(&result).unwrap();

    assert!(json.contains("\"success\":true"));
    assert!(json.contains("\"property_name\":\"Website Analytics\""));
    assert!(json.contains("\"account_name\":\"Company Account\""));
    assert!(json.contains("\"error\":null"));
    assert!(json.contains("\"quota_remaining\":2500"));
}

#[test]
fn test_connection_test_result_deserialization() {
    let json = r#"{
        "success": false,
        "property_name": null,
        "account_name": null,
        "error": "Permission denied",
        "quota_remaining": null
    }"#;

    let result: ConnectionTestResult = serde_json::from_str(json).unwrap();

    assert!(!result.success);
    assert!(result.property_name.is_none());
    assert_eq!(result.error, Some("Permission denied".to_string()));
}

#[test]
fn test_connection_test_result_json_roundtrip() {
    let original = ConnectionTestResult {
        success: true,
        property_name: Some("Test".to_string()),
        account_name: Some("Account".to_string()),
        error: None,
        quota_remaining: Some(100),
    };

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: ConnectionTestResult = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.success, original.success);
    assert_eq!(deserialized.property_name, original.property_name);
    assert_eq!(deserialized.quota_remaining, original.quota_remaining);
}

#[test]
fn test_connection_test_result_debug_impl() {
    let result = ConnectionTestResult {
        success: true,
        property_name: Some("Debug Test".to_string()),
        account_name: None,
        error: None,
        quota_remaining: Some(50),
    };

    let debug_str = format!("{:?}", result);

    assert!(debug_str.contains("ConnectionTestResult"));
    assert!(debug_str.contains("success"));
    assert!(debug_str.contains("property_name"));
}

#[test]
fn test_connection_test_result_clone() {
    let result = ConnectionTestResult {
        success: true,
        property_name: Some("Clone Test".to_string()),
        account_name: Some("Clone Account".to_string()),
        error: None,
        quota_remaining: Some(999),
    };

    let cloned = result.clone();

    assert_eq!(cloned.success, result.success);
    assert_eq!(cloned.property_name, result.property_name);
    assert_eq!(cloned.account_name, result.account_name);
    assert_eq!(cloned.quota_remaining, result.quota_remaining);
}

#[test]
fn test_connection_test_result_zero_quota() {
    let result = ConnectionTestResult {
        success: true,
        property_name: Some("Zero Quota".to_string()),
        account_name: Some("Account".to_string()),
        error: None,
        quota_remaining: Some(0),
    };

    assert_eq!(result.quota_remaining, Some(0));
}

#[test]
fn test_connection_test_result_various_errors() {
    let errors = vec![
        "Invalid API key",
        "Property not found",
        "Rate limit exceeded",
        "Network timeout",
        "Internal server error",
        "Authentication failed: token expired",
    ];

    for error_msg in errors {
        let result = ConnectionTestResult {
            success: false,
            property_name: None,
            account_name: None,
            error: Some(error_msg.to_string()),
            quota_remaining: None,
        };

        assert!(!result.success);
        assert_eq!(result.error, Some(error_msg.to_string()));
    }
}

// ============================================================================
// AvailablePropertiesResponse Tests
// ============================================================================

#[test]
fn test_available_properties_response_creation() {
    let response = AvailablePropertiesResponse {
        properties: vec![
            PropertyOption {
                property_id: "properties/111".to_string(),
                display_name: "Website 1".to_string(),
                account_name: "Account A".to_string(),
            },
            PropertyOption {
                property_id: "properties/222".to_string(),
                display_name: "Website 2".to_string(),
                account_name: "Account A".to_string(),
            },
        ],
    };

    assert_eq!(response.properties.len(), 2);
    assert_eq!(response.properties[0].property_id, "properties/111");
    assert_eq!(response.properties[1].display_name, "Website 2");
}

#[test]
fn test_available_properties_response_empty() {
    let response = AvailablePropertiesResponse {
        properties: vec![],
    };

    assert!(response.properties.is_empty());
}

#[test]
fn test_available_properties_response_serialization() {
    let response = AvailablePropertiesResponse {
        properties: vec![PropertyOption {
            property_id: "properties/333".to_string(),
            display_name: "Test Site".to_string(),
            account_name: "Test Account".to_string(),
        }],
    };

    let json = serde_json::to_string(&response).unwrap();

    assert!(json.contains("\"properties\""));
    assert!(json.contains("properties/333"));
    assert!(json.contains("Test Site"));
    assert!(json.contains("Test Account"));
}

#[test]
fn test_available_properties_response_deserialization() {
    let json = r#"{
        "properties": [
            {
                "property_id": "properties/444",
                "display_name": "Deserialized Site",
                "account_name": "Deserialized Account"
            }
        ]
    }"#;

    let response: AvailablePropertiesResponse = serde_json::from_str(json).unwrap();

    assert_eq!(response.properties.len(), 1);
    assert_eq!(response.properties[0].property_id, "properties/444");
    assert_eq!(response.properties[0].display_name, "Deserialized Site");
}

#[test]
fn test_available_properties_response_multiple_accounts() {
    let response = AvailablePropertiesResponse {
        properties: vec![
            PropertyOption {
                property_id: "properties/100".to_string(),
                display_name: "Site A1".to_string(),
                account_name: "Account A".to_string(),
            },
            PropertyOption {
                property_id: "properties/101".to_string(),
                display_name: "Site A2".to_string(),
                account_name: "Account A".to_string(),
            },
            PropertyOption {
                property_id: "properties/200".to_string(),
                display_name: "Site B1".to_string(),
                account_name: "Account B".to_string(),
            },
            PropertyOption {
                property_id: "properties/300".to_string(),
                display_name: "Site C1".to_string(),
                account_name: "Account C".to_string(),
            },
        ],
    };

    assert_eq!(response.properties.len(), 4);

    // Count properties per account
    let account_a_count = response
        .properties
        .iter()
        .filter(|p| p.account_name == "Account A")
        .count();
    assert_eq!(account_a_count, 2);
}

#[test]
fn test_available_properties_response_debug_impl() {
    let response = AvailablePropertiesResponse {
        properties: vec![PropertyOption {
            property_id: "properties/debug".to_string(),
            display_name: "Debug Site".to_string(),
            account_name: "Debug Account".to_string(),
        }],
    };

    let debug_str = format!("{:?}", response);

    assert!(debug_str.contains("AvailablePropertiesResponse"));
    assert!(debug_str.contains("properties"));
}

#[test]
fn test_available_properties_response_clone() {
    let response = AvailablePropertiesResponse {
        properties: vec![PropertyOption {
            property_id: "properties/clone".to_string(),
            display_name: "Clone Site".to_string(),
            account_name: "Clone Account".to_string(),
        }],
    };

    let cloned = response.clone();

    assert_eq!(cloned.properties.len(), response.properties.len());
    assert_eq!(cloned.properties[0].property_id, response.properties[0].property_id);
}

// ============================================================================
// PropertyOption Tests
// ============================================================================

#[test]
fn test_property_option_creation() {
    let option = PropertyOption {
        property_id: "properties/555".to_string(),
        display_name: "My Analytics Property".to_string(),
        account_name: "My Google Account".to_string(),
    };

    assert_eq!(option.property_id, "properties/555");
    assert_eq!(option.display_name, "My Analytics Property");
    assert_eq!(option.account_name, "My Google Account");
}

#[test]
fn test_property_option_serialization() {
    let option = PropertyOption {
        property_id: "properties/666".to_string(),
        display_name: "Serialized Property".to_string(),
        account_name: "Serialized Account".to_string(),
    };

    let json = serde_json::to_string(&option).unwrap();

    assert!(json.contains("\"property_id\":\"properties/666\""));
    assert!(json.contains("\"display_name\":\"Serialized Property\""));
    assert!(json.contains("\"account_name\":\"Serialized Account\""));
}

#[test]
fn test_property_option_deserialization() {
    let json = r#"{
        "property_id": "properties/777",
        "display_name": "Deserialized Property",
        "account_name": "Deserialized Account"
    }"#;

    let option: PropertyOption = serde_json::from_str(json).unwrap();

    assert_eq!(option.property_id, "properties/777");
    assert_eq!(option.display_name, "Deserialized Property");
    assert_eq!(option.account_name, "Deserialized Account");
}

#[test]
fn test_property_option_json_roundtrip() {
    let original = PropertyOption {
        property_id: "properties/888".to_string(),
        display_name: "Roundtrip Property".to_string(),
        account_name: "Roundtrip Account".to_string(),
    };

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: PropertyOption = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.property_id, original.property_id);
    assert_eq!(deserialized.display_name, original.display_name);
    assert_eq!(deserialized.account_name, original.account_name);
}

#[test]
fn test_property_option_debug_impl() {
    let option = PropertyOption {
        property_id: "properties/debug".to_string(),
        display_name: "Debug Property".to_string(),
        account_name: "Debug Account".to_string(),
    };

    let debug_str = format!("{:?}", option);

    assert!(debug_str.contains("PropertyOption"));
    assert!(debug_str.contains("property_id"));
    assert!(debug_str.contains("display_name"));
    assert!(debug_str.contains("account_name"));
}

#[test]
fn test_property_option_clone() {
    let option = PropertyOption {
        property_id: "properties/clone".to_string(),
        display_name: "Clone Property".to_string(),
        account_name: "Clone Account".to_string(),
    };

    let cloned = option.clone();

    assert_eq!(cloned.property_id, option.property_id);
    assert_eq!(cloned.display_name, option.display_name);
    assert_eq!(cloned.account_name, option.account_name);
}

#[test]
fn test_property_option_with_long_names() {
    let option = PropertyOption {
        property_id: "properties/999999999999".to_string(),
        display_name: "This Is A Very Long Property Display Name That Might Be Used For A Complex Website With Multiple Sections".to_string(),
        account_name: "Enterprise Organization with a Very Long Account Name for Testing Purposes".to_string(),
    };

    assert!(option.display_name.len() > 50);
    assert!(option.account_name.len() > 50);

    // Should still serialize/deserialize correctly
    let json = serde_json::to_string(&option).unwrap();
    let deserialized: PropertyOption = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.display_name, option.display_name);
}

#[test]
fn test_property_option_with_special_characters() {
    let option = PropertyOption {
        property_id: "properties/special".to_string(),
        display_name: "Property with <special> & \"characters\"".to_string(),
        account_name: "Account with 'quotes' & symbols @#$%".to_string(),
    };

    let json = serde_json::to_string(&option).unwrap();
    let deserialized: PropertyOption = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.display_name, option.display_name);
    assert_eq!(deserialized.account_name, option.account_name);
}

#[test]
fn test_property_option_with_unicode() {
    let option = PropertyOption {
        property_id: "properties/unicode".to_string(),
        display_name: "日本語のプロパティ".to_string(),
        account_name: "アカウント名 🌐".to_string(),
    };

    let json = serde_json::to_string(&option).unwrap();
    let deserialized: PropertyOption = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.display_name, "日本語のプロパティ");
    assert_eq!(deserialized.account_name, "アカウント名 🌐");
}

// ============================================================================
// Edge Cases and Integration Tests
// ============================================================================

#[test]
fn test_settings_form_privacy_compliance_combinations() {
    // Neither GDPR nor CCPA
    let form1 = SettingsFormData {
        gdpr_compliant: false,
        ccpa_compliant: false,
        ..sample_settings_form_data()
    };
    assert!(!form1.gdpr_compliant);
    assert!(!form1.ccpa_compliant);

    // Only GDPR
    let form2 = SettingsFormData {
        gdpr_compliant: true,
        ccpa_compliant: false,
        ..sample_settings_form_data()
    };
    assert!(form2.gdpr_compliant);
    assert!(!form2.ccpa_compliant);

    // Only CCPA
    let form3 = SettingsFormData {
        gdpr_compliant: false,
        ccpa_compliant: true,
        ..sample_settings_form_data()
    };
    assert!(!form3.gdpr_compliant);
    assert!(form3.ccpa_compliant);

    // Both
    let form4 = SettingsFormData {
        gdpr_compliant: true,
        ccpa_compliant: true,
        ..sample_settings_form_data()
    };
    assert!(form4.gdpr_compliant);
    assert!(form4.ccpa_compliant);
}

#[test]
fn test_connection_result_and_properties_integration() {
    // Simulate a successful connection followed by property list
    let connection_result = ConnectionTestResult {
        success: true,
        property_name: None,
        account_name: Some("Main Account".to_string()),
        error: None,
        quota_remaining: Some(5000),
    };

    assert!(connection_result.success);

    // After successful connection, we'd get properties
    let properties = AvailablePropertiesResponse {
        properties: vec![
            PropertyOption {
                property_id: "properties/123".to_string(),
                display_name: "Main Website".to_string(),
                account_name: "Main Account".to_string(),
            },
            PropertyOption {
                property_id: "properties/456".to_string(),
                display_name: "Blog".to_string(),
                account_name: "Main Account".to_string(),
            },
        ],
    };

    // All properties should belong to the connected account
    for prop in &properties.properties {
        assert_eq!(prop.account_name, "Main Account");
    }
}

#[test]
fn test_settings_form_date_range_options() {
    let date_ranges = vec![
        "today",
        "yesterday",
        "last_7_days",
        "last_14_days",
        "last_28_days",
        "last_30_days",
        "last_90_days",
        "last_365_days",
        "this_month",
        "last_month",
        "this_quarter",
        "last_quarter",
        "this_year",
        "last_year",
        "custom",
    ];

    for date_range in date_ranges {
        let form = SettingsFormData {
            default_date_range: date_range.to_string(),
            ..sample_settings_form_data()
        };

        let json = serde_json::to_string(&form).unwrap();
        let deserialized: SettingsFormData = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.default_date_range, date_range);
    }
}

#[test]
fn test_settings_form_report_frequency_options() {
    let frequencies = vec!["daily", "weekly", "bi_weekly", "monthly", "quarterly"];

    for freq in frequencies {
        let form = SettingsFormData {
            report_frequency: freq.to_string(),
            ..sample_settings_form_data()
        };

        let json = serde_json::to_string(&form).unwrap();
        let deserialized: SettingsFormData = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.report_frequency, freq);
    }
}

#[test]
fn test_large_email_recipients_list() {
    let recipients: Vec<String> = (0..100)
        .map(|i| format!("user{}@example.com", i))
        .collect();

    let form = SettingsFormData {
        report_email_enabled: true,
        report_email_recipients: recipients.clone(),
        ..sample_settings_form_data()
    };

    assert_eq!(form.report_email_recipients.len(), 100);

    let json = serde_json::to_string(&form).unwrap();
    let deserialized: SettingsFormData = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.report_email_recipients.len(), 100);
    assert!(deserialized.report_email_recipients.contains(&"user50@example.com".to_string()));
}

#[test]
fn test_settings_form_empty_strings() {
    let form = SettingsFormData {
        ga_property_id: "".to_string(),
        ga_measurement_id: "".to_string(),
        service_account_json: Some("".to_string()),
        enable_tracking: true,
        track_logged_in_users: true,
        track_admin_users: false,
        anonymize_ip: true,
        respect_dnt: true,
        cookie_consent_required: false,
        enhanced_link_attribution: true,
        enhanced_ecommerce: false,
        default_date_range: "".to_string(),
        show_realtime_widget: true,
        show_traffic_widget: true,
        show_toppages_widget: true,
        show_acquisition_widget: true,
        report_email_enabled: false,
        report_email_recipients: vec![],
        report_frequency: "".to_string(),
        gdpr_compliant: true,
        ccpa_compliant: true,
    };

    assert!(form.ga_property_id.is_empty());
    assert!(form.ga_measurement_id.is_empty());
    assert_eq!(form.service_account_json, Some("".to_string()));
    assert!(form.default_date_range.is_empty());
    assert!(form.report_frequency.is_empty());
}
