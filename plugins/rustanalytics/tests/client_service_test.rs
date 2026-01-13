//! Comprehensive tests for the Google Analytics Client Service
//!
//! Tests cover:
//! - ClientError variants and error messages
//! - Helper methods (dimension, metric, order_by, filters)
//! - Date range building and formatting
//! - API model structures and serialization
//! - Filter expressions construction

use chrono::NaiveDate;
use serde_json;

use rustanalytics::models::api::*;
use rustanalytics::models::DateRange;
use rustanalytics::services::client::{ClientError, GoogleAnalyticsClient};

// ============================================================================
// ClientError Tests
// ============================================================================

mod client_error_tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_authentication_failed_error() {
        let error = ClientError::AuthenticationFailed("invalid token".to_string());
        assert_eq!(error.to_string(), "Authentication failed: invalid token");
    }

    #[test]
    fn test_request_failed_error() {
        let error = ClientError::RequestFailed("connection refused".to_string());
        assert_eq!(error.to_string(), "API request failed: connection refused");
    }

    #[test]
    fn test_invalid_response_error() {
        let error = ClientError::InvalidResponse("malformed JSON".to_string());
        assert_eq!(error.to_string(), "Invalid response: malformed JSON");
    }

    #[test]
    fn test_rate_limited_error() {
        let error = ClientError::RateLimited(60);
        assert_eq!(error.to_string(), "Rate limited, retry after 60 seconds");
    }

    #[test]
    fn test_rate_limited_error_various_durations() {
        let error1 = ClientError::RateLimited(0);
        assert_eq!(error1.to_string(), "Rate limited, retry after 0 seconds");

        let error2 = ClientError::RateLimited(1);
        assert_eq!(error2.to_string(), "Rate limited, retry after 1 seconds");

        let error3 = ClientError::RateLimited(3600);
        assert_eq!(error3.to_string(), "Rate limited, retry after 3600 seconds");
    }

    #[test]
    fn test_quota_exceeded_error() {
        let error = ClientError::QuotaExceeded("daily limit reached".to_string());
        assert_eq!(error.to_string(), "Quota exceeded: daily limit reached");
    }

    #[test]
    fn test_property_not_found_error() {
        let error = ClientError::PropertyNotFound("123456789".to_string());
        assert_eq!(error.to_string(), "Property not found: 123456789");
    }

    #[test]
    fn test_invalid_credentials_error() {
        let error = ClientError::InvalidCredentials("missing private key".to_string());
        assert_eq!(error.to_string(), "Invalid credentials: missing private key");
    }

    #[test]
    fn test_rsa_error() {
        let error = ClientError::RsaError("key parsing failed".to_string());
        assert_eq!(error.to_string(), "RSA error: key parsing failed");
    }

    #[test]
    fn test_client_error_debug_format() {
        let error = ClientError::AuthenticationFailed("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("AuthenticationFailed"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_client_error_is_error_trait() {
        let error = ClientError::RequestFailed("test".to_string());
        let _: &dyn Error = &error;
    }

    #[test]
    fn test_json_error_from() {
        let json_str = "{ invalid json }";
        let json_err = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
        let client_error: ClientError = json_err.into();
        assert!(client_error.to_string().contains("JSON error"));
    }

    #[test]
    fn test_client_error_empty_messages() {
        let error1 = ClientError::AuthenticationFailed(String::new());
        assert_eq!(error1.to_string(), "Authentication failed: ");

        let error2 = ClientError::QuotaExceeded(String::new());
        assert_eq!(error2.to_string(), "Quota exceeded: ");
    }

    #[test]
    fn test_client_error_unicode_messages() {
        let error = ClientError::RequestFailed("错误: 连接失败 🔥".to_string());
        assert!(error.to_string().contains("错误"));
        assert!(error.to_string().contains("🔥"));
    }

    #[test]
    fn test_client_error_long_messages() {
        let long_msg = "x".repeat(10000);
        let error = ClientError::InvalidResponse(long_msg.clone());
        assert!(error.to_string().contains(&long_msg));
    }
}

// ============================================================================
// Dimension Builder Tests
// ============================================================================

mod dimension_builder_tests {
    use super::*;

    #[test]
    fn test_dimension_basic() {
        let dim = GoogleAnalyticsClient::dimension("country");
        assert_eq!(dim.name, "country");
        assert!(dim.dimension_expression.is_none());
    }

    #[test]
    fn test_dimension_various_names() {
        let test_cases = vec![
            "city",
            "pageTitle",
            "sessionSource",
            "deviceCategory",
            "browser",
            "operatingSystem",
            "platform",
            "continent",
            "region",
            "language",
        ];

        for name in test_cases {
            let dim = GoogleAnalyticsClient::dimension(name);
            assert_eq!(dim.name, name);
        }
    }

    #[test]
    fn test_dimension_custom_dimensions() {
        let dim = GoogleAnalyticsClient::dimension("customUser:membership_level");
        assert_eq!(dim.name, "customUser:membership_level");
    }

    #[test]
    fn test_dimension_event_parameters() {
        let dim = GoogleAnalyticsClient::dimension("eventName");
        assert_eq!(dim.name, "eventName");
    }

    #[test]
    fn test_dimension_empty_name() {
        let dim = GoogleAnalyticsClient::dimension("");
        assert_eq!(dim.name, "");
    }

    #[test]
    fn test_dimension_unicode_name() {
        let dim = GoogleAnalyticsClient::dimension("国家");
        assert_eq!(dim.name, "国家");
    }

    #[test]
    fn test_dimension_serialization() {
        let dim = GoogleAnalyticsClient::dimension("country");
        let json = serde_json::to_string(&dim).unwrap();
        assert!(json.contains("\"name\":\"country\""));
    }

    #[test]
    fn test_dimension_deserialization() {
        let json = r#"{"name":"city"}"#;
        let dim: Dimension = serde_json::from_str(json).unwrap();
        assert_eq!(dim.name, "city");
    }

    #[test]
    fn test_dimension_clone() {
        let dim1 = GoogleAnalyticsClient::dimension("browser");
        let dim2 = dim1.clone();
        assert_eq!(dim1.name, dim2.name);
    }
}

// ============================================================================
// Metric Builder Tests
// ============================================================================

mod metric_builder_tests {
    use super::*;

    #[test]
    fn test_metric_basic() {
        let metric = GoogleAnalyticsClient::metric("sessions");
        assert_eq!(metric.name, "sessions");
        assert!(metric.expression.is_none());
        assert!(metric.invisible.is_none());
    }

    #[test]
    fn test_metric_various_names() {
        let test_cases = vec![
            "sessions",
            "activeUsers",
            "screenPageViews",
            "bounceRate",
            "averageSessionDuration",
            "newUsers",
            "totalUsers",
            "engagedSessions",
            "eventCount",
            "conversions",
        ];

        for name in test_cases {
            let metric = GoogleAnalyticsClient::metric(name);
            assert_eq!(metric.name, name);
        }
    }

    #[test]
    fn test_metric_custom_metrics() {
        let metric = GoogleAnalyticsClient::metric("customEvent:purchase_value");
        assert_eq!(metric.name, "customEvent:purchase_value");
    }

    #[test]
    fn test_metric_empty_name() {
        let metric = GoogleAnalyticsClient::metric("");
        assert_eq!(metric.name, "");
    }

    #[test]
    fn test_metric_serialization() {
        let metric = GoogleAnalyticsClient::metric("sessions");
        let json = serde_json::to_string(&metric).unwrap();
        assert!(json.contains("\"name\":\"sessions\""));
    }

    #[test]
    fn test_metric_deserialization() {
        let json = r#"{"name":"activeUsers"}"#;
        let metric: Metric = serde_json::from_str(json).unwrap();
        assert_eq!(metric.name, "activeUsers");
    }

    #[test]
    fn test_metric_with_expression() {
        let json = r#"{"name":"customMetric","expression":"sessions/users"}"#;
        let metric: Metric = serde_json::from_str(json).unwrap();
        assert_eq!(metric.expression, Some("sessions/users".to_string()));
    }

    #[test]
    fn test_metric_with_invisible() {
        let json = r#"{"name":"hiddenMetric","invisible":true}"#;
        let metric: Metric = serde_json::from_str(json).unwrap();
        assert_eq!(metric.invisible, Some(true));
    }

    #[test]
    fn test_metric_clone() {
        let metric1 = GoogleAnalyticsClient::metric("pageViews");
        let metric2 = metric1.clone();
        assert_eq!(metric1.name, metric2.name);
    }
}

// ============================================================================
// Order By Metric Descending Tests
// ============================================================================

mod order_by_tests {
    use super::*;

    #[test]
    fn test_order_by_metric_desc_basic() {
        let order_by = GoogleAnalyticsClient::order_by_metric_desc("sessions");
        assert_eq!(order_by.desc, Some(true));
        assert!(order_by.metric.is_some());
        assert_eq!(order_by.metric.as_ref().unwrap().metric_name, "sessions");
        assert!(order_by.dimension.is_none());
        assert!(order_by.pivot.is_none());
    }

    #[test]
    fn test_order_by_metric_desc_various_metrics() {
        let metrics = vec![
            "activeUsers",
            "screenPageViews",
            "bounceRate",
            "averageSessionDuration",
            "conversions",
        ];

        for metric_name in metrics {
            let order_by = GoogleAnalyticsClient::order_by_metric_desc(metric_name);
            assert_eq!(order_by.desc, Some(true));
            assert_eq!(order_by.metric.as_ref().unwrap().metric_name, metric_name);
        }
    }

    #[test]
    fn test_order_by_serialization() {
        let order_by = GoogleAnalyticsClient::order_by_metric_desc("sessions");
        let json = serde_json::to_string(&order_by).unwrap();
        assert!(json.contains("\"desc\":true"));
        assert!(json.contains("\"metricName\":\"sessions\""));
    }

    #[test]
    fn test_order_by_empty_metric_name() {
        let order_by = GoogleAnalyticsClient::order_by_metric_desc("");
        assert_eq!(order_by.metric.as_ref().unwrap().metric_name, "");
    }

    #[test]
    fn test_order_by_clone() {
        let order1 = GoogleAnalyticsClient::order_by_metric_desc("revenue");
        let order2 = order1.clone();
        assert_eq!(order1.desc, order2.desc);
        assert_eq!(
            order1.metric.as_ref().unwrap().metric_name,
            order2.metric.as_ref().unwrap().metric_name
        );
    }
}

// ============================================================================
// String Filter Tests
// ============================================================================

mod string_filter_tests {
    use super::*;

    #[test]
    fn test_string_filter_exact() {
        let filter = GoogleAnalyticsClient::string_filter(
            "country",
            StringFilterMatchType::Exact,
            "United States",
        );
        assert!(filter.filter.is_some());
        let inner = filter.filter.as_ref().unwrap();
        assert_eq!(inner.field_name, "country");
        assert!(inner.string_filter.is_some());
        let sf = inner.string_filter.as_ref().unwrap();
        assert_eq!(sf.match_type, StringFilterMatchType::Exact);
        assert_eq!(sf.value, "United States");
        assert_eq!(sf.case_sensitive, Some(false));
    }

    #[test]
    fn test_string_filter_begins_with() {
        let filter = GoogleAnalyticsClient::string_filter(
            "pageTitle",
            StringFilterMatchType::BeginsWith,
            "Blog:",
        );
        let sf = filter.filter.as_ref().unwrap().string_filter.as_ref().unwrap();
        assert_eq!(sf.match_type, StringFilterMatchType::BeginsWith);
        assert_eq!(sf.value, "Blog:");
    }

    #[test]
    fn test_string_filter_ends_with() {
        let filter = GoogleAnalyticsClient::string_filter(
            "pagePath",
            StringFilterMatchType::EndsWith,
            ".html",
        );
        let sf = filter.filter.as_ref().unwrap().string_filter.as_ref().unwrap();
        assert_eq!(sf.match_type, StringFilterMatchType::EndsWith);
        assert_eq!(sf.value, ".html");
    }

    #[test]
    fn test_string_filter_contains() {
        let filter = GoogleAnalyticsClient::string_filter(
            "eventName",
            StringFilterMatchType::Contains,
            "purchase",
        );
        let sf = filter.filter.as_ref().unwrap().string_filter.as_ref().unwrap();
        assert_eq!(sf.match_type, StringFilterMatchType::Contains);
        assert_eq!(sf.value, "purchase");
    }

    #[test]
    fn test_string_filter_full_regexp() {
        let filter = GoogleAnalyticsClient::string_filter(
            "pagePath",
            StringFilterMatchType::FullRegexp,
            "^/blog/[0-9]+$",
        );
        let sf = filter.filter.as_ref().unwrap().string_filter.as_ref().unwrap();
        assert_eq!(sf.match_type, StringFilterMatchType::FullRegexp);
        assert_eq!(sf.value, "^/blog/[0-9]+$");
    }

    #[test]
    fn test_string_filter_partial_regexp() {
        let filter = GoogleAnalyticsClient::string_filter(
            "source",
            StringFilterMatchType::PartialRegexp,
            "google|bing",
        );
        let sf = filter.filter.as_ref().unwrap().string_filter.as_ref().unwrap();
        assert_eq!(sf.match_type, StringFilterMatchType::PartialRegexp);
        assert_eq!(sf.value, "google|bing");
    }

    #[test]
    fn test_string_filter_no_groups() {
        let filter = GoogleAnalyticsClient::string_filter(
            "country",
            StringFilterMatchType::Exact,
            "Canada",
        );
        assert!(filter.and_group.is_none());
        assert!(filter.or_group.is_none());
        assert!(filter.not_expression.is_none());
    }

    #[test]
    fn test_string_filter_no_other_filters() {
        let filter = GoogleAnalyticsClient::string_filter(
            "city",
            StringFilterMatchType::Contains,
            "York",
        );
        let inner = filter.filter.as_ref().unwrap();
        assert!(inner.in_list_filter.is_none());
        assert!(inner.numeric_filter.is_none());
        assert!(inner.between_filter.is_none());
    }

    #[test]
    fn test_string_filter_serialization() {
        let filter = GoogleAnalyticsClient::string_filter(
            "country",
            StringFilterMatchType::Exact,
            "France",
        );
        let json = serde_json::to_string(&filter).unwrap();
        assert!(json.contains("\"fieldName\":\"country\""));
        assert!(json.contains("\"value\":\"France\""));
        assert!(json.contains("\"caseSensitive\":false"));
    }

    #[test]
    fn test_string_filter_empty_value() {
        let filter = GoogleAnalyticsClient::string_filter(
            "pageTitle",
            StringFilterMatchType::Exact,
            "",
        );
        let sf = filter.filter.as_ref().unwrap().string_filter.as_ref().unwrap();
        assert_eq!(sf.value, "");
    }

    #[test]
    fn test_string_filter_unicode_value() {
        let filter = GoogleAnalyticsClient::string_filter(
            "country",
            StringFilterMatchType::Exact,
            "日本",
        );
        let sf = filter.filter.as_ref().unwrap().string_filter.as_ref().unwrap();
        assert_eq!(sf.value, "日本");
    }

    #[test]
    fn test_string_filter_special_chars() {
        let filter = GoogleAnalyticsClient::string_filter(
            "pagePath",
            StringFilterMatchType::Contains,
            "/path?query=value&other=123",
        );
        let sf = filter.filter.as_ref().unwrap().string_filter.as_ref().unwrap();
        assert_eq!(sf.value, "/path?query=value&other=123");
    }
}

// ============================================================================
// Numeric Filter Tests
// ============================================================================

mod numeric_filter_tests {
    use super::*;

    #[test]
    fn test_numeric_filter_equal() {
        let filter = GoogleAnalyticsClient::numeric_filter(
            "sessions",
            NumericFilterOperation::Equal,
            100.0,
        );
        assert!(filter.filter.is_some());
        let inner = filter.filter.as_ref().unwrap();
        assert_eq!(inner.field_name, "sessions");
        assert!(inner.numeric_filter.is_some());
        let nf = inner.numeric_filter.as_ref().unwrap();
        assert_eq!(nf.operation, NumericFilterOperation::Equal);
        assert_eq!(nf.value.double_value, Some(100.0));
        assert!(nf.value.int64_value.is_none());
    }

    #[test]
    fn test_numeric_filter_less_than() {
        let filter = GoogleAnalyticsClient::numeric_filter(
            "bounceRate",
            NumericFilterOperation::LessThan,
            50.0,
        );
        let nf = filter.filter.as_ref().unwrap().numeric_filter.as_ref().unwrap();
        assert_eq!(nf.operation, NumericFilterOperation::LessThan);
        assert_eq!(nf.value.double_value, Some(50.0));
    }

    #[test]
    fn test_numeric_filter_less_than_or_equal() {
        let filter = GoogleAnalyticsClient::numeric_filter(
            "pageViews",
            NumericFilterOperation::LessThanOrEqual,
            1000.0,
        );
        let nf = filter.filter.as_ref().unwrap().numeric_filter.as_ref().unwrap();
        assert_eq!(nf.operation, NumericFilterOperation::LessThanOrEqual);
        assert_eq!(nf.value.double_value, Some(1000.0));
    }

    #[test]
    fn test_numeric_filter_greater_than() {
        let filter = GoogleAnalyticsClient::numeric_filter(
            "activeUsers",
            NumericFilterOperation::GreaterThan,
            10.0,
        );
        let nf = filter.filter.as_ref().unwrap().numeric_filter.as_ref().unwrap();
        assert_eq!(nf.operation, NumericFilterOperation::GreaterThan);
        assert_eq!(nf.value.double_value, Some(10.0));
    }

    #[test]
    fn test_numeric_filter_greater_than_or_equal() {
        let filter = GoogleAnalyticsClient::numeric_filter(
            "revenue",
            NumericFilterOperation::GreaterThanOrEqual,
            100.0,
        );
        let nf = filter.filter.as_ref().unwrap().numeric_filter.as_ref().unwrap();
        assert_eq!(nf.operation, NumericFilterOperation::GreaterThanOrEqual);
        assert_eq!(nf.value.double_value, Some(100.0));
    }

    #[test]
    fn test_numeric_filter_zero() {
        let filter = GoogleAnalyticsClient::numeric_filter(
            "sessions",
            NumericFilterOperation::GreaterThan,
            0.0,
        );
        let nf = filter.filter.as_ref().unwrap().numeric_filter.as_ref().unwrap();
        assert_eq!(nf.value.double_value, Some(0.0));
    }

    #[test]
    fn test_numeric_filter_negative() {
        let filter = GoogleAnalyticsClient::numeric_filter(
            "customMetric",
            NumericFilterOperation::GreaterThan,
            -100.0,
        );
        let nf = filter.filter.as_ref().unwrap().numeric_filter.as_ref().unwrap();
        assert_eq!(nf.value.double_value, Some(-100.0));
    }

    #[test]
    fn test_numeric_filter_decimal() {
        let filter = GoogleAnalyticsClient::numeric_filter(
            "avgSessionDuration",
            NumericFilterOperation::GreaterThan,
            120.5,
        );
        let nf = filter.filter.as_ref().unwrap().numeric_filter.as_ref().unwrap();
        assert_eq!(nf.value.double_value, Some(120.5));
    }

    #[test]
    fn test_numeric_filter_large_value() {
        let filter = GoogleAnalyticsClient::numeric_filter(
            "totalUsers",
            NumericFilterOperation::LessThan,
            1_000_000_000.0,
        );
        let nf = filter.filter.as_ref().unwrap().numeric_filter.as_ref().unwrap();
        assert_eq!(nf.value.double_value, Some(1_000_000_000.0));
    }

    #[test]
    fn test_numeric_filter_no_groups() {
        let filter = GoogleAnalyticsClient::numeric_filter(
            "sessions",
            NumericFilterOperation::Equal,
            100.0,
        );
        assert!(filter.and_group.is_none());
        assert!(filter.or_group.is_none());
        assert!(filter.not_expression.is_none());
    }

    #[test]
    fn test_numeric_filter_no_other_filters() {
        let filter = GoogleAnalyticsClient::numeric_filter(
            "pageViews",
            NumericFilterOperation::GreaterThan,
            50.0,
        );
        let inner = filter.filter.as_ref().unwrap();
        assert!(inner.string_filter.is_none());
        assert!(inner.in_list_filter.is_none());
        assert!(inner.between_filter.is_none());
    }

    #[test]
    fn test_numeric_filter_serialization() {
        let filter = GoogleAnalyticsClient::numeric_filter(
            "sessions",
            NumericFilterOperation::GreaterThan,
            100.0,
        );
        let json = serde_json::to_string(&filter).unwrap();
        assert!(json.contains("\"fieldName\":\"sessions\""));
        assert!(json.contains("\"doubleValue\":100.0"));
    }
}

// ============================================================================
// Date Range Builder Tests
// ============================================================================

mod date_range_tests {
    use super::*;

    #[test]
    fn test_build_date_range_basic() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let date_range = DateRange { start_date: start, end_date: end };
        let api_range = GoogleAnalyticsClient::build_date_range(&date_range);
        assert_eq!(api_range.start_date, "2024-01-01");
        assert_eq!(api_range.end_date, "2024-01-31");
        assert!(api_range.name.is_none());
    }

    #[test]
    fn test_build_date_range_same_day() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let date_range = DateRange { start_date: date, end_date: date };
        let api_range = GoogleAnalyticsClient::build_date_range(&date_range);
        assert_eq!(api_range.start_date, "2024-06-15");
        assert_eq!(api_range.end_date, "2024-06-15");
    }

    #[test]
    fn test_build_date_range_year_boundary() {
        let start = NaiveDate::from_ymd_opt(2023, 12, 31).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let date_range = DateRange { start_date: start, end_date: end };
        let api_range = GoogleAnalyticsClient::build_date_range(&date_range);
        assert_eq!(api_range.start_date, "2023-12-31");
        assert_eq!(api_range.end_date, "2024-01-01");
    }

    #[test]
    fn test_build_date_range_leap_year() {
        let start = NaiveDate::from_ymd_opt(2024, 2, 28).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap();
        let date_range = DateRange { start_date: start, end_date: end };
        let api_range = GoogleAnalyticsClient::build_date_range(&date_range);
        assert_eq!(api_range.start_date, "2024-02-28");
        assert_eq!(api_range.end_date, "2024-02-29");
    }

    #[test]
    fn test_build_date_range_full_year() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let date_range = DateRange { start_date: start, end_date: end };
        let api_range = GoogleAnalyticsClient::build_date_range(&date_range);
        assert_eq!(api_range.start_date, "2024-01-01");
        assert_eq!(api_range.end_date, "2024-12-31");
    }

    #[test]
    fn test_build_date_range_past_dates() {
        let start = NaiveDate::from_ymd_opt(2020, 3, 15).unwrap();
        let end = NaiveDate::from_ymd_opt(2020, 3, 20).unwrap();
        let date_range = DateRange { start_date: start, end_date: end };
        let api_range = GoogleAnalyticsClient::build_date_range(&date_range);
        assert_eq!(api_range.start_date, "2020-03-15");
        assert_eq!(api_range.end_date, "2020-03-20");
    }

    #[test]
    fn test_build_date_range_format() {
        let start = NaiveDate::from_ymd_opt(2024, 7, 5).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 8, 9).unwrap();
        let date_range = DateRange { start_date: start, end_date: end };
        let api_range = GoogleAnalyticsClient::build_date_range(&date_range);
        // Ensure format is YYYY-MM-DD with leading zeros
        assert_eq!(api_range.start_date, "2024-07-05");
        assert_eq!(api_range.end_date, "2024-08-09");
    }

    #[test]
    fn test_build_date_range_last_n_days() {
        let date_range = DateRange::last_n_days(7);
        let api_range = GoogleAnalyticsClient::build_date_range(&date_range);
        assert!(!api_range.start_date.is_empty());
        assert!(!api_range.end_date.is_empty());
        // Verify format
        assert!(api_range.start_date.len() == 10);
        assert!(api_range.end_date.len() == 10);
    }

    #[test]
    fn test_build_date_range_serialization() {
        let start = NaiveDate::from_ymd_opt(2024, 5, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 5, 31).unwrap();
        let date_range = DateRange { start_date: start, end_date: end };
        let api_range = GoogleAnalyticsClient::build_date_range(&date_range);
        let json = serde_json::to_string(&api_range).unwrap();
        assert!(json.contains("\"startDate\":\"2024-05-01\""));
        assert!(json.contains("\"endDate\":\"2024-05-31\""));
    }
}

// ============================================================================
// API Date Range Model Tests
// ============================================================================

mod api_date_range_tests {
    use super::*;

    #[test]
    fn test_api_date_range_with_name() {
        let range = ApiDateRange {
            start_date: "2024-01-01".to_string(),
            end_date: "2024-01-31".to_string(),
            name: Some("January".to_string()),
        };
        assert_eq!(range.name, Some("January".to_string()));
    }

    #[test]
    fn test_api_date_range_today() {
        let range = ApiDateRange {
            start_date: "today".to_string(),
            end_date: "today".to_string(),
            name: None,
        };
        assert_eq!(range.start_date, "today");
        assert_eq!(range.end_date, "today");
    }

    #[test]
    fn test_api_date_range_yesterday() {
        let range = ApiDateRange {
            start_date: "yesterday".to_string(),
            end_date: "yesterday".to_string(),
            name: None,
        };
        assert_eq!(range.start_date, "yesterday");
    }

    #[test]
    fn test_api_date_range_relative() {
        let range = ApiDateRange {
            start_date: "7daysAgo".to_string(),
            end_date: "today".to_string(),
            name: None,
        };
        assert_eq!(range.start_date, "7daysAgo");
    }

    #[test]
    fn test_api_date_range_serialization_with_name() {
        let range = ApiDateRange {
            start_date: "2024-01-01".to_string(),
            end_date: "2024-01-31".to_string(),
            name: Some("Q1-2024".to_string()),
        };
        let json = serde_json::to_string(&range).unwrap();
        assert!(json.contains("\"name\":\"Q1-2024\""));
    }

    #[test]
    fn test_api_date_range_deserialization() {
        let json = r#"{"startDate":"2024-06-01","endDate":"2024-06-30"}"#;
        let range: ApiDateRange = serde_json::from_str(json).unwrap();
        assert_eq!(range.start_date, "2024-06-01");
        assert_eq!(range.end_date, "2024-06-30");
        assert!(range.name.is_none());
    }

    #[test]
    fn test_api_date_range_clone() {
        let range1 = ApiDateRange {
            start_date: "2024-01-01".to_string(),
            end_date: "2024-12-31".to_string(),
            name: Some("Full Year".to_string()),
        };
        let range2 = range1.clone();
        assert_eq!(range1.start_date, range2.start_date);
        assert_eq!(range1.end_date, range2.end_date);
        assert_eq!(range1.name, range2.name);
    }
}

// ============================================================================
// Run Report Request Tests
// ============================================================================

mod run_report_request_tests {
    use super::*;

    #[test]
    fn test_run_report_request_minimal() {
        let request = RunReportRequest {
            property: "properties/123456789".to_string(),
            date_ranges: vec![ApiDateRange {
                start_date: "2024-01-01".to_string(),
                end_date: "2024-01-31".to_string(),
                name: None,
            }],
            dimensions: None,
            metrics: vec![GoogleAnalyticsClient::metric("sessions")],
            dimension_filter: None,
            metric_filter: None,
            order_bys: None,
            offset: None,
            limit: None,
            metric_aggregations: None,
            keep_empty_rows: None,
            return_property_quota: None,
        };
        assert_eq!(request.property, "properties/123456789");
        assert_eq!(request.metrics.len(), 1);
    }

    #[test]
    fn test_run_report_request_with_dimensions() {
        let request = RunReportRequest {
            property: "properties/123456789".to_string(),
            date_ranges: vec![ApiDateRange {
                start_date: "2024-01-01".to_string(),
                end_date: "2024-01-31".to_string(),
                name: None,
            }],
            dimensions: Some(vec![
                GoogleAnalyticsClient::dimension("country"),
                GoogleAnalyticsClient::dimension("city"),
            ]),
            metrics: vec![GoogleAnalyticsClient::metric("sessions")],
            dimension_filter: None,
            metric_filter: None,
            order_bys: None,
            offset: None,
            limit: None,
            metric_aggregations: None,
            keep_empty_rows: None,
            return_property_quota: None,
        };
        assert_eq!(request.dimensions.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_run_report_request_with_filters() {
        let request = RunReportRequest {
            property: "properties/123456789".to_string(),
            date_ranges: vec![ApiDateRange {
                start_date: "today".to_string(),
                end_date: "today".to_string(),
                name: None,
            }],
            dimensions: Some(vec![GoogleAnalyticsClient::dimension("country")]),
            metrics: vec![GoogleAnalyticsClient::metric("sessions")],
            dimension_filter: Some(GoogleAnalyticsClient::string_filter(
                "country",
                StringFilterMatchType::Exact,
                "United States",
            )),
            metric_filter: Some(GoogleAnalyticsClient::numeric_filter(
                "sessions",
                NumericFilterOperation::GreaterThan,
                10.0,
            )),
            order_bys: None,
            offset: None,
            limit: None,
            metric_aggregations: None,
            keep_empty_rows: None,
            return_property_quota: None,
        };
        assert!(request.dimension_filter.is_some());
        assert!(request.metric_filter.is_some());
    }

    #[test]
    fn test_run_report_request_with_order_by() {
        let request = RunReportRequest {
            property: "properties/123456789".to_string(),
            date_ranges: vec![ApiDateRange {
                start_date: "7daysAgo".to_string(),
                end_date: "today".to_string(),
                name: None,
            }],
            dimensions: Some(vec![GoogleAnalyticsClient::dimension("pagePath")]),
            metrics: vec![GoogleAnalyticsClient::metric("screenPageViews")],
            dimension_filter: None,
            metric_filter: None,
            order_bys: Some(vec![
                GoogleAnalyticsClient::order_by_metric_desc("screenPageViews"),
            ]),
            offset: None,
            limit: Some(10),
            metric_aggregations: None,
            keep_empty_rows: None,
            return_property_quota: None,
        };
        assert!(request.order_bys.is_some());
        assert_eq!(request.limit, Some(10));
    }

    #[test]
    fn test_run_report_request_with_pagination() {
        let request = RunReportRequest {
            property: "properties/123456789".to_string(),
            date_ranges: vec![ApiDateRange {
                start_date: "2024-01-01".to_string(),
                end_date: "2024-01-31".to_string(),
                name: None,
            }],
            dimensions: Some(vec![GoogleAnalyticsClient::dimension("eventName")]),
            metrics: vec![GoogleAnalyticsClient::metric("eventCount")],
            dimension_filter: None,
            metric_filter: None,
            order_bys: None,
            offset: Some(100),
            limit: Some(50),
            metric_aggregations: None,
            keep_empty_rows: None,
            return_property_quota: None,
        };
        assert_eq!(request.offset, Some(100));
        assert_eq!(request.limit, Some(50));
    }

    #[test]
    fn test_run_report_request_with_quota() {
        let request = RunReportRequest {
            property: "properties/123456789".to_string(),
            date_ranges: vec![ApiDateRange {
                start_date: "today".to_string(),
                end_date: "today".to_string(),
                name: None,
            }],
            dimensions: None,
            metrics: vec![GoogleAnalyticsClient::metric("sessions")],
            dimension_filter: None,
            metric_filter: None,
            order_bys: None,
            offset: None,
            limit: Some(1),
            metric_aggregations: None,
            keep_empty_rows: None,
            return_property_quota: Some(true),
        };
        assert_eq!(request.return_property_quota, Some(true));
    }

    #[test]
    fn test_run_report_request_serialization() {
        let request = RunReportRequest {
            property: "properties/123456789".to_string(),
            date_ranges: vec![ApiDateRange {
                start_date: "2024-01-01".to_string(),
                end_date: "2024-01-31".to_string(),
                name: None,
            }],
            dimensions: Some(vec![GoogleAnalyticsClient::dimension("country")]),
            metrics: vec![GoogleAnalyticsClient::metric("sessions")],
            dimension_filter: None,
            metric_filter: None,
            order_bys: None,
            offset: None,
            limit: None,
            metric_aggregations: None,
            keep_empty_rows: None,
            return_property_quota: None,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"property\":\"properties/123456789\""));
        assert!(json.contains("\"startDate\":\"2024-01-01\""));
    }
}

// ============================================================================
// Run Report Response Tests
// ============================================================================

mod run_report_response_tests {
    use super::*;

    #[test]
    fn test_run_report_response_empty() {
        let response = RunReportResponse {
            dimension_headers: None,
            metric_headers: None,
            rows: None,
            totals: None,
            maximums: None,
            minimums: None,
            row_count: Some(0),
            metadata: None,
            property_quota: None,
            kind: None,
        };
        assert_eq!(response.row_count, Some(0));
    }

    #[test]
    fn test_run_report_response_with_data() {
        let response = RunReportResponse {
            dimension_headers: Some(vec![DimensionHeader {
                name: "country".to_string(),
            }]),
            metric_headers: Some(vec![MetricHeader {
                name: "sessions".to_string(),
                metric_type: Some(MetricType::TypeInteger),
            }]),
            rows: Some(vec![Row {
                dimension_values: Some(vec![DimensionValue {
                    value: Some("United States".to_string()),
                    one_value: None,
                }]),
                metric_values: Some(vec![MetricValue {
                    value: Some("1000".to_string()),
                    one_value: None,
                }]),
            }]),
            totals: None,
            maximums: None,
            minimums: None,
            row_count: Some(1),
            metadata: None,
            property_quota: None,
            kind: Some("analyticsData#runReport".to_string()),
        };
        assert_eq!(response.rows.as_ref().unwrap().len(), 1);
        assert_eq!(response.dimension_headers.as_ref().unwrap()[0].name, "country");
    }

    #[test]
    fn test_run_report_response_with_totals() {
        let response = RunReportResponse {
            dimension_headers: None,
            metric_headers: Some(vec![MetricHeader {
                name: "sessions".to_string(),
                metric_type: None,
            }]),
            rows: None,
            totals: Some(vec![Row {
                dimension_values: None,
                metric_values: Some(vec![MetricValue {
                    value: Some("5000".to_string()),
                    one_value: None,
                }]),
            }]),
            maximums: Some(vec![Row {
                dimension_values: None,
                metric_values: Some(vec![MetricValue {
                    value: Some("500".to_string()),
                    one_value: None,
                }]),
            }]),
            minimums: Some(vec![Row {
                dimension_values: None,
                metric_values: Some(vec![MetricValue {
                    value: Some("10".to_string()),
                    one_value: None,
                }]),
            }]),
            row_count: Some(100),
            metadata: None,
            property_quota: None,
            kind: None,
        };
        assert!(response.totals.is_some());
        assert!(response.maximums.is_some());
        assert!(response.minimums.is_some());
    }

    #[test]
    fn test_run_report_response_deserialization() {
        let json = r#"{
            "dimensionHeaders": [{"name": "city"}],
            "metricHeaders": [{"name": "activeUsers", "type": "TYPE_INTEGER"}],
            "rows": [
                {
                    "dimensionValues": [{"value": "New York"}],
                    "metricValues": [{"value": "500"}]
                }
            ],
            "rowCount": 1
        }"#;
        let response: RunReportResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.row_count, Some(1));
        assert_eq!(response.dimension_headers.as_ref().unwrap()[0].name, "city");
    }

    #[test]
    fn test_run_report_response_with_metadata() {
        let response = RunReportResponse {
            dimension_headers: None,
            metric_headers: None,
            rows: None,
            totals: None,
            maximums: None,
            minimums: None,
            row_count: None,
            metadata: Some(ResponseMetaData {
                data_loss_from_other_row: Some(true),
                schema_restriction_response: None,
                currency_code: Some("USD".to_string()),
                time_zone: Some("America/Los_Angeles".to_string()),
            }),
            property_quota: None,
            kind: None,
        };
        let meta = response.metadata.as_ref().unwrap();
        assert_eq!(meta.currency_code, Some("USD".to_string()));
        assert_eq!(meta.time_zone, Some("America/Los_Angeles".to_string()));
    }

    #[test]
    fn test_run_report_response_with_quota() {
        let response = RunReportResponse {
            dimension_headers: None,
            metric_headers: None,
            rows: None,
            totals: None,
            maximums: None,
            minimums: None,
            row_count: None,
            metadata: None,
            property_quota: Some(PropertyQuota {
                tokens_per_day: Some(QuotaStatus {
                    consumed: Some(10),
                    remaining: Some(990),
                }),
                tokens_per_hour: Some(QuotaStatus {
                    consumed: Some(5),
                    remaining: Some(95),
                }),
                concurrent_requests: Some(QuotaStatus {
                    consumed: Some(1),
                    remaining: Some(9),
                }),
                server_errors_per_project_per_hour: None,
                potentially_thresholded_requests_per_hour: None,
                tokens_per_project_per_hour: None,
            }),
            kind: None,
        };
        let quota = response.property_quota.as_ref().unwrap();
        assert_eq!(quota.tokens_per_day.as_ref().unwrap().consumed, Some(10));
    }
}

// ============================================================================
// Realtime Report Request Tests
// ============================================================================

mod realtime_report_request_tests {
    use super::*;

    #[test]
    fn test_realtime_report_request_basic() {
        let request = RunRealtimeReportRequest {
            property: "properties/123456789".to_string(),
            dimensions: Some(vec![GoogleAnalyticsClient::dimension("country")]),
            metrics: vec![GoogleAnalyticsClient::metric("activeUsers")],
            dimension_filter: None,
            metric_filter: None,
            limit: Some(10),
            metric_aggregations: None,
            order_bys: None,
            return_property_quota: None,
            minute_ranges: None,
        };
        assert_eq!(request.property, "properties/123456789");
        assert_eq!(request.limit, Some(10));
    }

    #[test]
    fn test_realtime_report_request_with_minute_ranges() {
        let request = RunRealtimeReportRequest {
            property: "properties/123456789".to_string(),
            dimensions: None,
            metrics: vec![GoogleAnalyticsClient::metric("activeUsers")],
            dimension_filter: None,
            metric_filter: None,
            limit: None,
            metric_aggregations: None,
            order_bys: None,
            return_property_quota: None,
            minute_ranges: Some(vec![
                MinuteRange {
                    name: Some("0-4 minutes ago".to_string()),
                    start_minutes_ago: Some(4),
                    end_minutes_ago: Some(0),
                },
                MinuteRange {
                    name: Some("25-29 minutes ago".to_string()),
                    start_minutes_ago: Some(29),
                    end_minutes_ago: Some(25),
                },
            ]),
        };
        assert_eq!(request.minute_ranges.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_realtime_report_request_serialization() {
        let request = RunRealtimeReportRequest {
            property: "properties/123456789".to_string(),
            dimensions: Some(vec![GoogleAnalyticsClient::dimension("city")]),
            metrics: vec![GoogleAnalyticsClient::metric("activeUsers")],
            dimension_filter: None,
            metric_filter: None,
            limit: Some(100),
            metric_aggregations: None,
            order_bys: None,
            return_property_quota: None,
            minute_ranges: None,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"property\":\"properties/123456789\""));
        assert!(json.contains("\"limit\":100"));
    }
}

// ============================================================================
// Realtime Report Response Tests
// ============================================================================

mod realtime_report_response_tests {
    use super::*;

    #[test]
    fn test_realtime_report_response_empty() {
        let response = RunRealtimeReportResponse {
            dimension_headers: None,
            metric_headers: None,
            rows: None,
            totals: None,
            maximums: None,
            minimums: None,
            row_count: Some(0),
            property_quota: None,
            kind: None,
        };
        assert_eq!(response.row_count, Some(0));
    }

    #[test]
    fn test_realtime_report_response_with_data() {
        let response = RunRealtimeReportResponse {
            dimension_headers: Some(vec![DimensionHeader {
                name: "city".to_string(),
            }]),
            metric_headers: Some(vec![MetricHeader {
                name: "activeUsers".to_string(),
                metric_type: Some(MetricType::TypeInteger),
            }]),
            rows: Some(vec![
                Row {
                    dimension_values: Some(vec![DimensionValue {
                        value: Some("Tokyo".to_string()),
                        one_value: None,
                    }]),
                    metric_values: Some(vec![MetricValue {
                        value: Some("50".to_string()),
                        one_value: None,
                    }]),
                },
                Row {
                    dimension_values: Some(vec![DimensionValue {
                        value: Some("London".to_string()),
                        one_value: None,
                    }]),
                    metric_values: Some(vec![MetricValue {
                        value: Some("30".to_string()),
                        one_value: None,
                    }]),
                },
            ]),
            totals: Some(vec![Row {
                dimension_values: None,
                metric_values: Some(vec![MetricValue {
                    value: Some("80".to_string()),
                    one_value: None,
                }]),
            }]),
            maximums: None,
            minimums: None,
            row_count: Some(2),
            property_quota: None,
            kind: Some("analyticsData#runRealtimeReport".to_string()),
        };
        assert_eq!(response.rows.as_ref().unwrap().len(), 2);
        assert_eq!(response.totals.as_ref().unwrap()[0].metric_values.as_ref().unwrap()[0].value, Some("80".to_string()));
    }

    #[test]
    fn test_realtime_report_response_deserialization() {
        let json = r#"{
            "dimensionHeaders": [{"name": "country"}],
            "metricHeaders": [{"name": "activeUsers"}],
            "rows": [
                {
                    "dimensionValues": [{"value": "Japan"}],
                    "metricValues": [{"value": "25"}]
                }
            ],
            "rowCount": 1,
            "kind": "analyticsData#runRealtimeReport"
        }"#;
        let response: RunRealtimeReportResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.row_count, Some(1));
        assert_eq!(response.kind, Some("analyticsData#runRealtimeReport".to_string()));
    }
}

// ============================================================================
// Batch Run Reports Tests
// ============================================================================

mod batch_reports_tests {
    use super::*;

    #[test]
    fn test_batch_run_reports_request() {
        let request = BatchRunReportsRequest {
            property: "properties/123456789".to_string(),
            requests: vec![
                RunReportRequest {
                    property: "properties/123456789".to_string(),
                    date_ranges: vec![ApiDateRange {
                        start_date: "7daysAgo".to_string(),
                        end_date: "today".to_string(),
                        name: None,
                    }],
                    dimensions: Some(vec![GoogleAnalyticsClient::dimension("country")]),
                    metrics: vec![GoogleAnalyticsClient::metric("sessions")],
                    dimension_filter: None,
                    metric_filter: None,
                    order_bys: None,
                    offset: None,
                    limit: None,
                    metric_aggregations: None,
                    keep_empty_rows: None,
                    return_property_quota: None,
                },
                RunReportRequest {
                    property: "properties/123456789".to_string(),
                    date_ranges: vec![ApiDateRange {
                        start_date: "7daysAgo".to_string(),
                        end_date: "today".to_string(),
                        name: None,
                    }],
                    dimensions: Some(vec![GoogleAnalyticsClient::dimension("deviceCategory")]),
                    metrics: vec![GoogleAnalyticsClient::metric("activeUsers")],
                    dimension_filter: None,
                    metric_filter: None,
                    order_bys: None,
                    offset: None,
                    limit: None,
                    metric_aggregations: None,
                    keep_empty_rows: None,
                    return_property_quota: None,
                },
            ],
        };
        assert_eq!(request.requests.len(), 2);
    }

    #[test]
    fn test_batch_run_reports_response() {
        let response = BatchRunReportsResponse {
            reports: Some(vec![
                RunReportResponse {
                    dimension_headers: Some(vec![DimensionHeader { name: "country".to_string() }]),
                    metric_headers: Some(vec![MetricHeader { name: "sessions".to_string(), metric_type: None }]),
                    rows: Some(vec![Row {
                        dimension_values: Some(vec![DimensionValue { value: Some("US".to_string()), one_value: None }]),
                        metric_values: Some(vec![MetricValue { value: Some("100".to_string()), one_value: None }]),
                    }]),
                    totals: None,
                    maximums: None,
                    minimums: None,
                    row_count: Some(1),
                    metadata: None,
                    property_quota: None,
                    kind: None,
                },
            ]),
            kind: Some("analyticsData#batchRunReports".to_string()),
        };
        assert_eq!(response.reports.as_ref().unwrap().len(), 1);
    }
}

// ============================================================================
// Pivot Report Tests
// ============================================================================

mod pivot_report_tests {
    use super::*;

    #[test]
    fn test_pivot_definition() {
        let pivot = Pivot {
            field_names: vec!["country".to_string(), "city".to_string()],
            order_bys: None,
            offset: Some(0),
            limit: Some(10),
            metric_aggregations: Some(vec!["TOTAL".to_string()]),
        };
        assert_eq!(pivot.field_names.len(), 2);
        assert_eq!(pivot.limit, Some(10));
    }

    #[test]
    fn test_run_pivot_report_request() {
        let request = RunPivotReportRequest {
            property: "properties/123456789".to_string(),
            date_ranges: vec![ApiDateRange {
                start_date: "30daysAgo".to_string(),
                end_date: "today".to_string(),
                name: None,
            }],
            pivots: vec![Pivot {
                field_names: vec!["browser".to_string()],
                order_bys: None,
                offset: None,
                limit: Some(5),
                metric_aggregations: None,
            }],
            dimensions: Some(vec![GoogleAnalyticsClient::dimension("country")]),
            metrics: vec![GoogleAnalyticsClient::metric("sessions")],
            dimension_filter: None,
            metric_filter: None,
            keep_empty_rows: None,
            return_property_quota: None,
        };
        assert_eq!(request.pivots.len(), 1);
        assert_eq!(request.pivots[0].field_names[0], "browser");
    }

    #[test]
    fn test_run_pivot_report_response() {
        let response = RunPivotReportResponse {
            pivot_headers: Some(vec![PivotHeader {
                pivot_dimension_headers: Some(vec![PivotDimensionHeader {
                    dimension_values: Some(vec![DimensionValue {
                        value: Some("Chrome".to_string()),
                        one_value: None,
                    }]),
                }]),
                row_count: Some(5),
            }]),
            dimension_headers: Some(vec![DimensionHeader { name: "country".to_string() }]),
            metric_headers: Some(vec![MetricHeader { name: "sessions".to_string(), metric_type: None }]),
            rows: None,
            aggregates: None,
            metadata: None,
            property_quota: None,
            kind: None,
        };
        assert!(response.pivot_headers.is_some());
        assert_eq!(response.pivot_headers.as_ref().unwrap()[0].row_count, Some(5));
    }
}

// ============================================================================
// Metadata Tests
// ============================================================================

mod metadata_tests {
    use super::*;

    #[test]
    fn test_metadata_response() {
        let metadata = Metadata {
            name: Some("properties/123456789/metadata".to_string()),
            dimensions: Some(vec![
                DimensionMetadata {
                    api_name: Some("country".to_string()),
                    ui_name: Some("Country".to_string()),
                    description: Some("The country of the user".to_string()),
                    deprecated_api_names: None,
                    custom_definition: Some(false),
                    category: Some("Geo".to_string()),
                },
            ]),
            metrics: Some(vec![
                MetricMetadata {
                    api_name: Some("sessions".to_string()),
                    ui_name: Some("Sessions".to_string()),
                    description: Some("The number of sessions".to_string()),
                    metric_type: Some(MetricType::TypeInteger),
                    expression: None,
                    deprecated_api_names: None,
                    custom_definition: Some(false),
                    blocked_reasons: None,
                    category: Some("Session".to_string()),
                },
            ]),
        };
        assert!(metadata.dimensions.is_some());
        assert!(metadata.metrics.is_some());
        assert_eq!(metadata.dimensions.as_ref().unwrap()[0].api_name, Some("country".to_string()));
    }

    #[test]
    fn test_dimension_metadata() {
        let dim_meta = DimensionMetadata {
            api_name: Some("customUser:membership".to_string()),
            ui_name: Some("Membership Level".to_string()),
            description: Some("Custom dimension for membership".to_string()),
            deprecated_api_names: Some(vec!["old_membership".to_string()]),
            custom_definition: Some(true),
            category: Some("Custom".to_string()),
        };
        assert_eq!(dim_meta.custom_definition, Some(true));
        assert!(dim_meta.deprecated_api_names.is_some());
    }

    #[test]
    fn test_metric_metadata() {
        let metric_meta = MetricMetadata {
            api_name: Some("customEvent:revenue".to_string()),
            ui_name: Some("Revenue".to_string()),
            description: Some("Custom revenue metric".to_string()),
            metric_type: Some(MetricType::TypeCurrency),
            expression: Some("eventValue".to_string()),
            deprecated_api_names: None,
            custom_definition: Some(true),
            blocked_reasons: None,
            category: Some("Ecommerce".to_string()),
        };
        assert_eq!(metric_meta.metric_type, Some(MetricType::TypeCurrency));
    }

    #[test]
    fn test_metric_type_variants() {
        let types = vec![
            MetricType::MetricTypeUnspecified,
            MetricType::TypeInteger,
            MetricType::TypeFloat,
            MetricType::TypeSeconds,
            MetricType::TypeMilliseconds,
            MetricType::TypeMinutes,
            MetricType::TypeHours,
            MetricType::TypeStandard,
            MetricType::TypeCurrency,
            MetricType::TypeFeet,
            MetricType::TypeMiles,
            MetricType::TypeMeters,
            MetricType::TypeKilometers,
        ];
        assert_eq!(types.len(), 13);
    }
}

// ============================================================================
// Account Summary Tests
// ============================================================================

mod account_summary_tests {
    use super::*;

    #[test]
    fn test_account_summary() {
        let summary = AccountSummary {
            name: Some("accountSummaries/123".to_string()),
            account: Some("accounts/123".to_string()),
            display_name: Some("My Account".to_string()),
            property_summaries: Some(vec![
                PropertySummary {
                    property: Some("properties/456".to_string()),
                    display_name: Some("My Website".to_string()),
                    property_type: Some("PROPERTY_TYPE_ORDINARY".to_string()),
                },
            ]),
        };
        assert_eq!(summary.display_name, Some("My Account".to_string()));
        assert_eq!(summary.property_summaries.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_property_summary() {
        let prop = PropertySummary {
            property: Some("properties/789".to_string()),
            display_name: Some("Test Property".to_string()),
            property_type: Some("PROPERTY_TYPE_ORDINARY".to_string()),
        };
        assert_eq!(prop.property, Some("properties/789".to_string()));
    }

    #[test]
    fn test_list_account_summaries_response() {
        let response = ListAccountSummariesResponse {
            account_summaries: Some(vec![
                AccountSummary {
                    name: Some("accountSummaries/1".to_string()),
                    account: Some("accounts/1".to_string()),
                    display_name: Some("Account 1".to_string()),
                    property_summaries: Some(vec![]),
                },
                AccountSummary {
                    name: Some("accountSummaries/2".to_string()),
                    account: Some("accounts/2".to_string()),
                    display_name: Some("Account 2".to_string()),
                    property_summaries: Some(vec![]),
                },
            ]),
            next_page_token: Some("token123".to_string()),
        };
        assert_eq!(response.account_summaries.as_ref().unwrap().len(), 2);
        assert_eq!(response.next_page_token, Some("token123".to_string()));
    }
}

// ============================================================================
// Token Response Tests
// ============================================================================

mod token_response_tests {
    use super::*;

    #[test]
    fn test_token_response() {
        let response = TokenResponse {
            access_token: "ya29.abc123".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            scope: Some("https://www.googleapis.com/auth/analytics.readonly".to_string()),
        };
        assert_eq!(response.token_type, "Bearer");
        assert_eq!(response.expires_in, 3600);
    }

    #[test]
    fn test_token_response_deserialization() {
        let json = r#"{
            "access_token": "test_token",
            "token_type": "Bearer",
            "expires_in": 3599
        }"#;
        let response: TokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.access_token, "test_token");
        assert!(response.scope.is_none());
    }
}

// ============================================================================
// API Error Tests
// ============================================================================

mod api_error_tests {
    use super::*;

    #[test]
    fn test_api_error() {
        let error = ApiError {
            error: ApiErrorDetail {
                code: 403,
                message: "Quota exceeded".to_string(),
                status: Some("PERMISSION_DENIED".to_string()),
                details: None,
            },
        };
        assert_eq!(error.error.code, 403);
        assert_eq!(error.error.message, "Quota exceeded");
    }

    #[test]
    fn test_api_error_deserialization() {
        let json = r#"{
            "error": {
                "code": 429,
                "message": "Rate Limit Exceeded",
                "status": "RESOURCE_EXHAUSTED"
            }
        }"#;
        let error: ApiError = serde_json::from_str(json).unwrap();
        assert_eq!(error.error.code, 429);
    }

    #[test]
    fn test_api_error_with_details() {
        let json = r#"{
            "error": {
                "code": 400,
                "message": "Invalid request",
                "status": "INVALID_ARGUMENT",
                "details": [
                    {"@type": "type.googleapis.com/google.rpc.BadRequest", "fieldViolations": []}
                ]
            }
        }"#;
        let error: ApiError = serde_json::from_str(json).unwrap();
        assert!(error.error.details.is_some());
    }
}

// ============================================================================
// Filter Expression Tests
// ============================================================================

mod filter_expression_tests {
    use super::*;

    #[test]
    fn test_filter_expression_with_and_group() {
        let filter = FilterExpression {
            and_group: Some(FilterExpressionList {
                expressions: vec![
                    GoogleAnalyticsClient::string_filter("country", StringFilterMatchType::Exact, "US"),
                    GoogleAnalyticsClient::string_filter("city", StringFilterMatchType::Contains, "York"),
                ],
            }),
            or_group: None,
            not_expression: None,
            filter: None,
        };
        assert!(filter.and_group.is_some());
        assert_eq!(filter.and_group.as_ref().unwrap().expressions.len(), 2);
    }

    #[test]
    fn test_filter_expression_with_or_group() {
        let filter = FilterExpression {
            and_group: None,
            or_group: Some(FilterExpressionList {
                expressions: vec![
                    GoogleAnalyticsClient::string_filter("browser", StringFilterMatchType::Exact, "Chrome"),
                    GoogleAnalyticsClient::string_filter("browser", StringFilterMatchType::Exact, "Firefox"),
                ],
            }),
            not_expression: None,
            filter: None,
        };
        assert!(filter.or_group.is_some());
        assert_eq!(filter.or_group.as_ref().unwrap().expressions.len(), 2);
    }

    #[test]
    fn test_filter_expression_with_not() {
        let inner_filter = GoogleAnalyticsClient::string_filter(
            "country",
            StringFilterMatchType::Exact,
            "US",
        );
        let filter = FilterExpression {
            and_group: None,
            or_group: None,
            not_expression: Some(Box::new(inner_filter)),
            filter: None,
        };
        assert!(filter.not_expression.is_some());
    }

    #[test]
    fn test_in_list_filter() {
        let filter = InListFilter {
            values: vec!["Chrome".to_string(), "Firefox".to_string(), "Safari".to_string()],
            case_sensitive: Some(false),
        };
        assert_eq!(filter.values.len(), 3);
    }

    #[test]
    fn test_between_filter() {
        let filter = BetweenFilter {
            from_value: NumericValue {
                int64_value: None,
                double_value: Some(10.0),
            },
            to_value: NumericValue {
                int64_value: None,
                double_value: Some(100.0),
            },
        };
        assert_eq!(filter.from_value.double_value, Some(10.0));
        assert_eq!(filter.to_value.double_value, Some(100.0));
    }
}

// ============================================================================
// Funnel Report Tests
// ============================================================================

mod funnel_report_tests {
    use super::*;

    #[test]
    fn test_funnel_definition() {
        let funnel = Funnel {
            is_open_funnel: Some(false),
            steps: vec![
                FunnelStep {
                    name: Some("View Product".to_string()),
                    is_directly_followed_by: Some(false),
                    within_duration_from_prior_step: None,
                    filter_expression: None,
                },
                FunnelStep {
                    name: Some("Add to Cart".to_string()),
                    is_directly_followed_by: Some(true),
                    within_duration_from_prior_step: Some("3600s".to_string()),
                    filter_expression: None,
                },
                FunnelStep {
                    name: Some("Purchase".to_string()),
                    is_directly_followed_by: Some(false),
                    within_duration_from_prior_step: None,
                    filter_expression: None,
                },
            ],
        };
        assert_eq!(funnel.steps.len(), 3);
        assert_eq!(funnel.is_open_funnel, Some(false));
    }

    #[test]
    fn test_funnel_step_with_filter() {
        let step = FunnelStep {
            name: Some("Purchase Event".to_string()),
            is_directly_followed_by: None,
            within_duration_from_prior_step: None,
            filter_expression: Some(FunnelFilterExpression {
                and_group: None,
                or_group: None,
                not_expression: None,
                funnel_field_filter: Some(FunnelFieldFilter {
                    field_name: "eventName".to_string(),
                    string_filter: Some(StringFilter {
                        match_type: StringFilterMatchType::Exact,
                        value: "purchase".to_string(),
                        case_sensitive: Some(false),
                    }),
                    in_list_filter: None,
                    numeric_filter: None,
                    between_filter: None,
                }),
                funnel_event_filter: None,
            }),
        };
        assert!(step.filter_expression.is_some());
    }

    #[test]
    fn test_run_funnel_report_request() {
        let request = RunFunnelReportRequest {
            property: "properties/123456789".to_string(),
            date_ranges: vec![ApiDateRange {
                start_date: "30daysAgo".to_string(),
                end_date: "today".to_string(),
                name: None,
            }],
            funnel: Funnel {
                is_open_funnel: Some(true),
                steps: vec![FunnelStep {
                    name: Some("Step 1".to_string()),
                    is_directly_followed_by: None,
                    within_duration_from_prior_step: None,
                    filter_expression: None,
                }],
            },
            funnel_breakdown: Some(FunnelBreakdown {
                breakdown_dimension: Some(GoogleAnalyticsClient::dimension("deviceCategory")),
                limit: Some(5),
            }),
            funnel_next_action: None,
            funnel_visualization_type: Some("STANDARD_FUNNEL".to_string()),
            segments: None,
            limit: Some(10),
            dimension_filter: None,
            return_property_quota: None,
        };
        assert!(request.funnel_breakdown.is_some());
        assert_eq!(request.funnel_visualization_type, Some("STANDARD_FUNNEL".to_string()));
    }
}

// ============================================================================
// Segment Tests
// ============================================================================

mod segment_tests {
    use super::*;

    #[test]
    fn test_segment_definition() {
        let segment = Segment {
            name: Some("Power Users".to_string()),
            user_segment: Some(UserSegment {
                user_inclusion_criteria: Some(UserSegmentCriteria {
                    and_condition_groups: Some(vec![UserSegmentConditionGroup {
                        condition_scoping: Some("USER_CRITERIA_WITHIN_SAME_SESSION".to_string()),
                        segment_filter_expression: None,
                    }]),
                    and_sequence_groups: None,
                }),
                exclusion: None,
            }),
            session_segment: None,
            event_segment: None,
        };
        assert_eq!(segment.name, Some("Power Users".to_string()));
        assert!(segment.user_segment.is_some());
    }

    #[test]
    fn test_session_segment() {
        let segment = Segment {
            name: Some("Mobile Sessions".to_string()),
            user_segment: None,
            session_segment: Some(SessionSegment {
                session_inclusion_criteria: Some(SessionSegmentCriteria {
                    and_condition_groups: Some(vec![SessionSegmentConditionGroup {
                        condition_scoping: Some("SESSION_CRITERIA_WITHIN_SAME_EVENT".to_string()),
                        segment_filter_expression: None,
                    }]),
                }),
                exclusion: None,
            }),
            event_segment: None,
        };
        assert!(segment.session_segment.is_some());
    }

    #[test]
    fn test_event_segment() {
        let segment = Segment {
            name: Some("Purchase Events".to_string()),
            user_segment: None,
            session_segment: None,
            event_segment: Some(EventSegment {
                event_inclusion_criteria: Some(EventSegmentCriteria {
                    and_condition_groups: Some(vec![EventSegmentConditionGroup {
                        condition_scoping: Some("EVENT_CRITERIA_WITHIN_SAME_EVENT".to_string()),
                        segment_filter_expression: None,
                    }]),
                }),
                exclusion: None,
            }),
        };
        assert!(segment.event_segment.is_some());
    }

    #[test]
    fn test_segment_filter_expression() {
        let expr = SegmentFilterExpression {
            and_group: None,
            or_group: None,
            not_expression: None,
            segment_filter: Some(SegmentFilter {
                field_name: "deviceCategory".to_string(),
                string_filter: Some(StringFilter {
                    match_type: StringFilterMatchType::Exact,
                    value: "mobile".to_string(),
                    case_sensitive: Some(false),
                }),
                in_list_filter: None,
                numeric_filter: None,
                between_filter: None,
                filter_scoping: Some(SegmentFilterScoping {
                    at_any_point_in_time: Some(true),
                    in_any_n_day_period: None,
                }),
            }),
            segment_event_filter: None,
        };
        assert!(expr.segment_filter.is_some());
    }
}

// ============================================================================
// Dimension Order By Tests
// ============================================================================

mod dimension_order_by_tests {
    use super::*;

    #[test]
    fn test_dimension_order_by() {
        let order = DimensionOrderBy {
            dimension_name: "country".to_string(),
            order_type: Some(DimensionOrderType::Alphanumeric),
        };
        assert_eq!(order.dimension_name, "country");
        assert_eq!(order.order_type, Some(DimensionOrderType::Alphanumeric));
    }

    #[test]
    fn test_dimension_order_type_variants() {
        let types = vec![
            DimensionOrderType::Alphanumeric,
            DimensionOrderType::CaseInsensitiveAlphanumeric,
            DimensionOrderType::Numeric,
        ];
        assert_eq!(types.len(), 3);
    }

    #[test]
    fn test_order_by_with_dimension() {
        let order_by = OrderBy {
            desc: Some(false),
            metric: None,
            dimension: Some(DimensionOrderBy {
                dimension_name: "date".to_string(),
                order_type: Some(DimensionOrderType::Numeric),
            }),
            pivot: None,
        };
        assert!(order_by.dimension.is_some());
        assert_eq!(order_by.desc, Some(false));
    }
}

// ============================================================================
// Pivot Order By Tests
// ============================================================================

mod pivot_order_by_tests {
    use super::*;

    #[test]
    fn test_pivot_order_by() {
        let order = PivotOrderBy {
            metric_name: "sessions".to_string(),
            pivot_selections: vec![
                PivotSelection {
                    dimension_name: "browser".to_string(),
                    dimension_value: "Chrome".to_string(),
                },
            ],
        };
        assert_eq!(order.metric_name, "sessions");
        assert_eq!(order.pivot_selections.len(), 1);
    }

    #[test]
    fn test_pivot_selection() {
        let selection = PivotSelection {
            dimension_name: "country".to_string(),
            dimension_value: "United States".to_string(),
        };
        assert_eq!(selection.dimension_name, "country");
        assert_eq!(selection.dimension_value, "United States");
    }

    #[test]
    fn test_order_by_with_pivot() {
        let order_by = OrderBy {
            desc: Some(true),
            metric: None,
            dimension: None,
            pivot: Some(PivotOrderBy {
                metric_name: "revenue".to_string(),
                pivot_selections: vec![
                    PivotSelection {
                        dimension_name: "browser".to_string(),
                        dimension_value: "Chrome".to_string(),
                    },
                    PivotSelection {
                        dimension_name: "deviceCategory".to_string(),
                        dimension_value: "desktop".to_string(),
                    },
                ],
            }),
        };
        assert!(order_by.pivot.is_some());
        assert_eq!(order_by.pivot.as_ref().unwrap().pivot_selections.len(), 2);
    }
}

// ============================================================================
// Dimension Expression Tests
// ============================================================================

mod dimension_expression_tests {
    use super::*;

    #[test]
    fn test_dimension_with_lower_case() {
        let dim = Dimension {
            name: "lowerCountry".to_string(),
            dimension_expression: Some(DimensionExpression {
                lower_case: Some(CaseExpression {
                    dimension_name: "country".to_string(),
                }),
                upper_case: None,
                concatenate: None,
            }),
        };
        assert!(dim.dimension_expression.is_some());
        assert!(dim.dimension_expression.as_ref().unwrap().lower_case.is_some());
    }

    #[test]
    fn test_dimension_with_upper_case() {
        let dim = Dimension {
            name: "upperBrowser".to_string(),
            dimension_expression: Some(DimensionExpression {
                lower_case: None,
                upper_case: Some(CaseExpression {
                    dimension_name: "browser".to_string(),
                }),
                concatenate: None,
            }),
        };
        assert!(dim.dimension_expression.as_ref().unwrap().upper_case.is_some());
    }

    #[test]
    fn test_dimension_with_concatenate() {
        let dim = Dimension {
            name: "fullLocation".to_string(),
            dimension_expression: Some(DimensionExpression {
                lower_case: None,
                upper_case: None,
                concatenate: Some(ConcatenateExpression {
                    dimension_names: vec!["country".to_string(), "city".to_string()],
                    delimiter: Some(" - ".to_string()),
                }),
            }),
        };
        let concat = dim.dimension_expression.as_ref().unwrap().concatenate.as_ref().unwrap();
        assert_eq!(concat.dimension_names.len(), 2);
        assert_eq!(concat.delimiter, Some(" - ".to_string()));
    }
}

// ============================================================================
// Property Quota Tests
// ============================================================================

mod property_quota_tests {
    use super::*;

    #[test]
    fn test_property_quota_full() {
        let quota = PropertyQuota {
            tokens_per_day: Some(QuotaStatus {
                consumed: Some(100),
                remaining: Some(24900),
            }),
            tokens_per_hour: Some(QuotaStatus {
                consumed: Some(50),
                remaining: Some(4950),
            }),
            concurrent_requests: Some(QuotaStatus {
                consumed: Some(2),
                remaining: Some(8),
            }),
            server_errors_per_project_per_hour: Some(QuotaStatus {
                consumed: Some(0),
                remaining: Some(10),
            }),
            potentially_thresholded_requests_per_hour: Some(QuotaStatus {
                consumed: Some(5),
                remaining: Some(115),
            }),
            tokens_per_project_per_hour: Some(QuotaStatus {
                consumed: Some(200),
                remaining: Some(9800),
            }),
        };
        assert_eq!(quota.tokens_per_day.as_ref().unwrap().consumed, Some(100));
        assert_eq!(quota.tokens_per_day.as_ref().unwrap().remaining, Some(24900));
    }

    #[test]
    fn test_quota_status_serialization() {
        let status = QuotaStatus {
            consumed: Some(10),
            remaining: Some(90),
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"consumed\":10"));
        assert!(json.contains("\"remaining\":90"));
    }

    #[test]
    fn test_quota_status_deserialization() {
        let json = r#"{"consumed": 25, "remaining": 75}"#;
        let status: QuotaStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status.consumed, Some(25));
        assert_eq!(status.remaining, Some(75));
    }
}

// ============================================================================
// Response Metadata Tests
// ============================================================================

mod response_metadata_tests {
    use super::*;

    #[test]
    fn test_response_metadata() {
        let metadata = ResponseMetaData {
            data_loss_from_other_row: Some(true),
            schema_restriction_response: Some(SchemaRestrictionResponse {
                active_metric_restrictions: Some(vec![ActiveMetricRestriction {
                    metric_name: Some("revenue".to_string()),
                    restricted_metric_types: Some(vec!["COST_DATA".to_string()]),
                }]),
            }),
            currency_code: Some("EUR".to_string()),
            time_zone: Some("Europe/Berlin".to_string()),
        };
        assert_eq!(metadata.data_loss_from_other_row, Some(true));
        assert_eq!(metadata.currency_code, Some("EUR".to_string()));
    }

    #[test]
    fn test_active_metric_restriction() {
        let restriction = ActiveMetricRestriction {
            metric_name: Some("purchaseRevenue".to_string()),
            restricted_metric_types: Some(vec![
                "COST_DATA".to_string(),
                "REVENUE_DATA".to_string(),
            ]),
        };
        assert_eq!(restriction.restricted_metric_types.as_ref().unwrap().len(), 2);
    }
}

// ============================================================================
// Minute Range Tests
// ============================================================================

mod minute_range_tests {
    use super::*;

    #[test]
    fn test_minute_range_basic() {
        let range = MinuteRange {
            name: Some("Last 5 minutes".to_string()),
            start_minutes_ago: Some(4),
            end_minutes_ago: Some(0),
        };
        assert_eq!(range.name, Some("Last 5 minutes".to_string()));
        assert_eq!(range.start_minutes_ago, Some(4));
        assert_eq!(range.end_minutes_ago, Some(0));
    }

    #[test]
    fn test_minute_range_full_period() {
        let range = MinuteRange {
            name: Some("Full 30 minutes".to_string()),
            start_minutes_ago: Some(29),
            end_minutes_ago: Some(0),
        };
        assert_eq!(range.start_minutes_ago, Some(29));
    }

    #[test]
    fn test_minute_range_no_name() {
        let range = MinuteRange {
            name: None,
            start_minutes_ago: Some(14),
            end_minutes_ago: Some(10),
        };
        assert!(range.name.is_none());
    }

    #[test]
    fn test_minute_range_serialization() {
        let range = MinuteRange {
            name: Some("Recent".to_string()),
            start_minutes_ago: Some(4),
            end_minutes_ago: Some(0),
        };
        let json = serde_json::to_string(&range).unwrap();
        assert!(json.contains("\"name\":\"Recent\""));
        assert!(json.contains("\"startMinutesAgo\":4"));
        assert!(json.contains("\"endMinutesAgo\":0"));
    }
}

// ============================================================================
// Row and Value Tests
// ============================================================================

mod row_value_tests {
    use super::*;

    #[test]
    fn test_row_with_dimension_and_metric() {
        let row = Row {
            dimension_values: Some(vec![
                DimensionValue {
                    value: Some("United States".to_string()),
                    one_value: None,
                },
                DimensionValue {
                    value: Some("New York".to_string()),
                    one_value: None,
                },
            ]),
            metric_values: Some(vec![
                MetricValue {
                    value: Some("1000".to_string()),
                    one_value: None,
                },
                MetricValue {
                    value: Some("500.5".to_string()),
                    one_value: None,
                },
            ]),
        };
        assert_eq!(row.dimension_values.as_ref().unwrap().len(), 2);
        assert_eq!(row.metric_values.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_row_empty() {
        let row = Row {
            dimension_values: None,
            metric_values: None,
        };
        assert!(row.dimension_values.is_none());
        assert!(row.metric_values.is_none());
    }

    #[test]
    fn test_dimension_value_with_one_value() {
        let value = DimensionValue {
            value: None,
            one_value: Some("Alternative Value".to_string()),
        };
        assert!(value.value.is_none());
        assert_eq!(value.one_value, Some("Alternative Value".to_string()));
    }

    #[test]
    fn test_metric_value_parsing() {
        let values = vec![
            ("123", 123i64),
            ("0", 0),
            ("999999999", 999999999),
        ];
        for (str_val, expected) in values {
            let mv = MetricValue {
                value: Some(str_val.to_string()),
                one_value: None,
            };
            let parsed: i64 = mv.value.as_ref().unwrap().parse().unwrap();
            assert_eq!(parsed, expected);
        }
    }

    #[test]
    fn test_metric_value_float_parsing() {
        let mv = MetricValue {
            value: Some("123.456".to_string()),
            one_value: None,
        };
        let parsed: f64 = mv.value.as_ref().unwrap().parse().unwrap();
        assert!((parsed - 123.456).abs() < 0.001);
    }
}

// ============================================================================
// String Filter Match Type Tests
// ============================================================================

mod string_filter_match_type_tests {
    use super::*;

    #[test]
    fn test_string_filter_match_type_serialization() {
        let types = vec![
            (StringFilterMatchType::Exact, "EXACT"),
            (StringFilterMatchType::BeginsWith, "BEGINS_WITH"),
            (StringFilterMatchType::EndsWith, "ENDS_WITH"),
            (StringFilterMatchType::Contains, "CONTAINS"),
            (StringFilterMatchType::FullRegexp, "FULL_REGEXP"),
            (StringFilterMatchType::PartialRegexp, "PARTIAL_REGEXP"),
        ];

        for (match_type, expected) in types {
            let json = serde_json::to_string(&match_type).unwrap();
            assert_eq!(json, format!("\"{}\"", expected));
        }
    }

    #[test]
    fn test_string_filter_match_type_deserialization() {
        let json_values = vec![
            ("\"EXACT\"", StringFilterMatchType::Exact),
            ("\"BEGINS_WITH\"", StringFilterMatchType::BeginsWith),
            ("\"ENDS_WITH\"", StringFilterMatchType::EndsWith),
            ("\"CONTAINS\"", StringFilterMatchType::Contains),
            ("\"FULL_REGEXP\"", StringFilterMatchType::FullRegexp),
            ("\"PARTIAL_REGEXP\"", StringFilterMatchType::PartialRegexp),
        ];

        for (json, expected) in json_values {
            let parsed: StringFilterMatchType = serde_json::from_str(json).unwrap();
            assert_eq!(parsed, expected);
        }
    }
}

// ============================================================================
// Numeric Filter Operation Tests
// ============================================================================

mod numeric_filter_operation_tests {
    use super::*;

    #[test]
    fn test_numeric_filter_operation_serialization() {
        let operations = vec![
            (NumericFilterOperation::Equal, "EQUAL"),
            (NumericFilterOperation::LessThan, "LESS_THAN"),
            (NumericFilterOperation::LessThanOrEqual, "LESS_THAN_OR_EQUAL"),
            (NumericFilterOperation::GreaterThan, "GREATER_THAN"),
            (NumericFilterOperation::GreaterThanOrEqual, "GREATER_THAN_OR_EQUAL"),
        ];

        for (operation, expected) in operations {
            let json = serde_json::to_string(&operation).unwrap();
            assert_eq!(json, format!("\"{}\"", expected));
        }
    }

    #[test]
    fn test_numeric_filter_operation_deserialization() {
        let json_values = vec![
            ("\"EQUAL\"", NumericFilterOperation::Equal),
            ("\"LESS_THAN\"", NumericFilterOperation::LessThan),
            ("\"LESS_THAN_OR_EQUAL\"", NumericFilterOperation::LessThanOrEqual),
            ("\"GREATER_THAN\"", NumericFilterOperation::GreaterThan),
            ("\"GREATER_THAN_OR_EQUAL\"", NumericFilterOperation::GreaterThanOrEqual),
        ];

        for (json, expected) in json_values {
            let parsed: NumericFilterOperation = serde_json::from_str(json).unwrap();
            assert_eq!(parsed, expected);
        }
    }
}

// ============================================================================
// Clone and Debug Tests
// ============================================================================

mod clone_debug_tests {
    use super::*;

    #[test]
    fn test_run_report_request_clone() {
        let request = RunReportRequest {
            property: "properties/123".to_string(),
            date_ranges: vec![ApiDateRange {
                start_date: "today".to_string(),
                end_date: "today".to_string(),
                name: None,
            }],
            dimensions: None,
            metrics: vec![GoogleAnalyticsClient::metric("sessions")],
            dimension_filter: None,
            metric_filter: None,
            order_bys: None,
            offset: None,
            limit: None,
            metric_aggregations: None,
            keep_empty_rows: None,
            return_property_quota: None,
        };
        let cloned = request.clone();
        assert_eq!(request.property, cloned.property);
    }

    #[test]
    fn test_filter_expression_debug() {
        let filter = GoogleAnalyticsClient::string_filter(
            "country",
            StringFilterMatchType::Exact,
            "US",
        );
        let debug_str = format!("{:?}", filter);
        assert!(debug_str.contains("FilterExpression"));
    }

    #[test]
    fn test_order_by_debug() {
        let order = GoogleAnalyticsClient::order_by_metric_desc("sessions");
        let debug_str = format!("{:?}", order);
        assert!(debug_str.contains("OrderBy"));
    }

    #[test]
    fn test_quota_status_clone() {
        let status = QuotaStatus {
            consumed: Some(50),
            remaining: Some(950),
        };
        let cloned = status.clone();
        assert_eq!(status.consumed, cloned.consumed);
        assert_eq!(status.remaining, cloned.remaining);
    }
}
