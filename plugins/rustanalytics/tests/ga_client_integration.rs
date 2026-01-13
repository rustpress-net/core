//! Integration tests for Google Analytics Client
//!
//! These tests verify the GA client's functionality including:
//! - Request building and serialization
//! - Response parsing and deserialization
//! - Error handling for various HTTP status codes
//! - Filter and query builders
//! - Date range handling

use rustanalytics::models::api::*;
use rustanalytics::models::{DateRange, DateRangePreset};
use rustanalytics::services::client::{ClientError, GoogleAnalyticsClient};
use serde_json::json;

// ============================================================================
// Test Fixtures and Helpers
// ============================================================================

/// Sample GA4 report response for testing
fn sample_report_response() -> serde_json::Value {
    json!({
        "dimensionHeaders": [
            { "name": "date" },
            { "name": "country" }
        ],
        "metricHeaders": [
            { "name": "sessions", "type": "TYPE_INTEGER" },
            { "name": "totalUsers", "type": "TYPE_INTEGER" },
            { "name": "bounceRate", "type": "TYPE_FLOAT" }
        ],
        "rows": [
            {
                "dimensionValues": [
                    { "value": "20231201" },
                    { "value": "United States" }
                ],
                "metricValues": [
                    { "value": "1500" },
                    { "value": "1200" },
                    { "value": "0.45" }
                ]
            },
            {
                "dimensionValues": [
                    { "value": "20231201" },
                    { "value": "United Kingdom" }
                ],
                "metricValues": [
                    { "value": "800" },
                    { "value": "650" },
                    { "value": "0.38" }
                ]
            }
        ],
        "totals": [
            {
                "dimensionValues": [],
                "metricValues": [
                    { "value": "2300" },
                    { "value": "1850" },
                    { "value": "0.42" }
                ]
            }
        ],
        "rowCount": 2,
        "metadata": {
            "currencyCode": "USD",
            "timeZone": "America/Los_Angeles"
        },
        "propertyQuota": {
            "tokensPerDay": {
                "consumed": 10,
                "remaining": 24990
            },
            "tokensPerHour": {
                "consumed": 5,
                "remaining": 4995
            },
            "concurrentRequests": {
                "consumed": 1,
                "remaining": 9
            },
            "serverErrorsPerProjectPerHour": {
                "consumed": 0,
                "remaining": 10
            },
            "potentiallyThresholdedRequestsPerHour": {
                "consumed": 0,
                "remaining": 120
            }
        }
    })
}

/// Sample realtime report response
fn sample_realtime_response() -> serde_json::Value {
    json!({
        "dimensionHeaders": [
            { "name": "country" }
        ],
        "metricHeaders": [
            { "name": "activeUsers", "type": "TYPE_INTEGER" }
        ],
        "rows": [
            {
                "dimensionValues": [
                    { "value": "United States" }
                ],
                "metricValues": [
                    { "value": "42" }
                ]
            },
            {
                "dimensionValues": [
                    { "value": "Germany" }
                ],
                "metricValues": [
                    { "value": "18" }
                ]
            }
        ],
        "rowCount": 2
    })
}

/// Sample metadata response
fn sample_metadata_response() -> serde_json::Value {
    json!({
        "name": "properties/123456789/metadata",
        "dimensions": [
            {
                "apiName": "country",
                "uiName": "Country",
                "description": "The country from which user activity originated.",
                "category": "Geography"
            },
            {
                "apiName": "city",
                "uiName": "City",
                "description": "The city from which user activity originated.",
                "category": "Geography"
            },
            {
                "apiName": "date",
                "uiName": "Date",
                "description": "The date of the event.",
                "category": "Time"
            }
        ],
        "metrics": [
            {
                "apiName": "sessions",
                "uiName": "Sessions",
                "description": "The number of sessions that began on your site or app.",
                "category": "Session",
                "type": "TYPE_INTEGER"
            },
            {
                "apiName": "totalUsers",
                "uiName": "Total Users",
                "description": "The total number of users.",
                "category": "User",
                "type": "TYPE_INTEGER"
            },
            {
                "apiName": "bounceRate",
                "uiName": "Bounce Rate",
                "description": "The percentage of sessions that were not engaged.",
                "category": "Session",
                "type": "TYPE_FLOAT"
            }
        ]
    })
}

/// Sample account summaries response
fn sample_account_summaries_response() -> serde_json::Value {
    json!({
        "accountSummaries": [
            {
                "name": "accountSummaries/123456",
                "account": "accounts/123456",
                "displayName": "Test Account",
                "propertySummaries": [
                    {
                        "property": "properties/123456789",
                        "displayName": "Test Property",
                        "propertyType": "PROPERTY_TYPE_ORDINARY"
                    },
                    {
                        "property": "properties/987654321",
                        "displayName": "Another Property",
                        "propertyType": "PROPERTY_TYPE_ORDINARY"
                    }
                ]
            }
        ]
    })
}

/// Sample error response from Google Analytics API
fn sample_error_response(code: u32, message: &str, status: &str) -> serde_json::Value {
    json!({
        "error": {
            "code": code,
            "message": message,
            "status": status
        }
    })
}

/// Google API Error structure for parsing
#[derive(Debug, serde::Deserialize)]
struct GoogleApiError {
    error: GoogleApiErrorDetails,
}

#[derive(Debug, serde::Deserialize)]
struct GoogleApiErrorDetails {
    code: u32,
    message: String,
    status: String,
}

// ============================================================================
// Request Building Tests
// ============================================================================

#[test]
fn test_build_date_range_last_7_days() {
    let date_range = DateRange::last_n_days(7);
    let api_range = GoogleAnalyticsClient::build_date_range(&date_range);

    assert!(!api_range.start_date.is_empty());
    assert!(!api_range.end_date.is_empty());
    assert!(api_range.name.is_none());

    // Verify date format (YYYY-MM-DD)
    assert!(api_range.start_date.len() == 10);
    assert!(api_range.end_date.len() == 10);
    assert!(api_range.start_date.contains('-'));
}

#[test]
fn test_build_date_range_today() {
    let date_range = DateRange::today();
    let api_range = GoogleAnalyticsClient::build_date_range(&date_range);

    // Start and end should be the same for today
    assert_eq!(api_range.start_date, api_range.end_date);
}

#[test]
fn test_build_date_range_yesterday() {
    let date_range = DateRange::yesterday();
    let api_range = GoogleAnalyticsClient::build_date_range(&date_range);

    assert_eq!(api_range.start_date, api_range.end_date);
}

#[test]
fn test_build_date_range_this_month() {
    let date_range = DateRange::this_month();
    let api_range = GoogleAnalyticsClient::build_date_range(&date_range);

    // Start should be day 01 of the month
    assert!(api_range.start_date.ends_with("-01"));
}

#[test]
fn test_dimension_builder() {
    let dim = GoogleAnalyticsClient::dimension("country");
    assert_eq!(dim.name, "country");
    assert!(dim.dimension_expression.is_none());
}

#[test]
fn test_dimension_builder_various_dimensions() {
    let dimensions = vec![
        "country", "city", "date", "pagePath", "sessionSource",
        "sessionMedium", "deviceCategory", "browser", "operatingSystem"
    ];

    for dim_name in dimensions {
        let dim = GoogleAnalyticsClient::dimension(dim_name);
        assert_eq!(dim.name, dim_name);
    }
}

#[test]
fn test_metric_builder() {
    let metric = GoogleAnalyticsClient::metric("sessions");
    assert_eq!(metric.name, "sessions");
    assert!(metric.expression.is_none());
    assert!(metric.invisible.is_none());
}

#[test]
fn test_metric_builder_various_metrics() {
    let metrics = vec![
        "sessions", "totalUsers", "newUsers", "activeUsers",
        "bounceRate", "screenPageViews", "averageSessionDuration",
        "engagementRate", "conversions", "totalRevenue"
    ];

    for metric_name in metrics {
        let metric = GoogleAnalyticsClient::metric(metric_name);
        assert_eq!(metric.name, metric_name);
    }
}

#[test]
fn test_order_by_metric_desc() {
    let order = GoogleAnalyticsClient::order_by_metric_desc("sessions");

    assert_eq!(order.desc, Some(true));
    assert!(order.metric.is_some());
    assert!(order.dimension.is_none());
    assert!(order.pivot.is_none());

    let metric_order = order.metric.unwrap();
    assert_eq!(metric_order.metric_name, "sessions");
}

#[test]
fn test_string_filter_exact_match() {
    let filter = GoogleAnalyticsClient::string_filter(
        "country",
        StringFilterMatchType::Exact,
        "United States"
    );

    assert!(filter.filter.is_some());
    assert!(filter.and_group.is_none());
    assert!(filter.or_group.is_none());
    assert!(filter.not_expression.is_none());

    let inner_filter = filter.filter.unwrap();
    assert_eq!(inner_filter.field_name, "country");

    let string_filter = inner_filter.string_filter.unwrap();
    assert_eq!(string_filter.value, "United States");
    assert_eq!(string_filter.case_sensitive, Some(false));
}

#[test]
fn test_string_filter_contains() {
    let filter = GoogleAnalyticsClient::string_filter(
        "pagePath",
        StringFilterMatchType::Contains,
        "/blog/"
    );

    let inner_filter = filter.filter.unwrap();
    let string_filter = inner_filter.string_filter.unwrap();

    assert_eq!(string_filter.value, "/blog/");
    assert!(matches!(string_filter.match_type, StringFilterMatchType::Contains));
}

#[test]
fn test_string_filter_begins_with() {
    let filter = GoogleAnalyticsClient::string_filter(
        "pagePath",
        StringFilterMatchType::BeginsWith,
        "/products"
    );

    let inner_filter = filter.filter.unwrap();
    let string_filter = inner_filter.string_filter.unwrap();

    assert_eq!(string_filter.value, "/products");
    assert!(matches!(string_filter.match_type, StringFilterMatchType::BeginsWith));
}

#[test]
fn test_string_filter_regex() {
    let filter = GoogleAnalyticsClient::string_filter(
        "pagePath",
        StringFilterMatchType::FullRegexp,
        r"^/blog/\d{4}/.*"
    );

    let inner_filter = filter.filter.unwrap();
    let string_filter = inner_filter.string_filter.unwrap();

    assert_eq!(string_filter.value, r"^/blog/\d{4}/.*");
    assert!(matches!(string_filter.match_type, StringFilterMatchType::FullRegexp));
}

#[test]
fn test_numeric_filter_greater_than() {
    let filter = GoogleAnalyticsClient::numeric_filter(
        "sessions",
        NumericFilterOperation::GreaterThan,
        100.0
    );

    let inner_filter = filter.filter.unwrap();
    assert_eq!(inner_filter.field_name, "sessions");

    let numeric_filter = inner_filter.numeric_filter.unwrap();
    assert_eq!(numeric_filter.value.double_value, Some(100.0));
    assert!(matches!(numeric_filter.operation, NumericFilterOperation::GreaterThan));
}

#[test]
fn test_numeric_filter_less_than() {
    let filter = GoogleAnalyticsClient::numeric_filter(
        "bounceRate",
        NumericFilterOperation::LessThan,
        0.5
    );

    let inner_filter = filter.filter.unwrap();
    let numeric_filter = inner_filter.numeric_filter.unwrap();

    assert_eq!(numeric_filter.value.double_value, Some(0.5));
    assert!(matches!(numeric_filter.operation, NumericFilterOperation::LessThan));
}

#[test]
fn test_numeric_filter_equal() {
    let filter = GoogleAnalyticsClient::numeric_filter(
        "conversions",
        NumericFilterOperation::Equal,
        5.0
    );

    let inner_filter = filter.filter.unwrap();
    let numeric_filter = inner_filter.numeric_filter.unwrap();

    assert_eq!(numeric_filter.value.double_value, Some(5.0));
}

// ============================================================================
// Response Parsing Tests
// ============================================================================

#[test]
fn test_parse_report_response() {
    let response_json = sample_report_response();
    let response: RunReportResponse = serde_json::from_value(response_json).unwrap();

    // Verify dimension headers
    let dim_headers = response.dimension_headers.unwrap();
    assert_eq!(dim_headers.len(), 2);
    assert_eq!(dim_headers[0].name, "date");
    assert_eq!(dim_headers[1].name, "country");

    // Verify metric headers
    let metric_headers = response.metric_headers.unwrap();
    assert_eq!(metric_headers.len(), 3);
    assert_eq!(metric_headers[0].name, "sessions");

    // Verify rows
    let rows = response.rows.unwrap();
    assert_eq!(rows.len(), 2);

    let first_row = &rows[0];
    let dim_values = first_row.dimension_values.as_ref().unwrap();
    assert_eq!(dim_values[0].value, Some("20231201".to_string()));
    assert_eq!(dim_values[1].value, Some("United States".to_string()));

    let metric_values = first_row.metric_values.as_ref().unwrap();
    assert_eq!(metric_values[0].value, Some("1500".to_string()));

    // Verify row count
    assert_eq!(response.row_count, Some(2));

    // Verify quota
    let quota = response.property_quota.unwrap();
    assert!(quota.tokens_per_day.is_some());
}

#[test]
fn test_parse_realtime_response() {
    let response_json = sample_realtime_response();
    let response: RunRealtimeReportResponse = serde_json::from_value(response_json).unwrap();

    let rows = response.rows.unwrap();
    assert_eq!(rows.len(), 2);

    let first_row = &rows[0];
    let dim_values = first_row.dimension_values.as_ref().unwrap();
    assert_eq!(dim_values[0].value, Some("United States".to_string()));

    let metric_values = first_row.metric_values.as_ref().unwrap();
    assert_eq!(metric_values[0].value, Some("42".to_string()));
}

#[test]
fn test_parse_metadata_response() {
    let response_json = sample_metadata_response();
    let response: Metadata = serde_json::from_value(response_json).unwrap();

    let dimensions = response.dimensions.unwrap();
    assert_eq!(dimensions.len(), 3);
    assert_eq!(dimensions[0].api_name, Some("country".to_string()));

    let metrics = response.metrics.unwrap();
    assert_eq!(metrics.len(), 3);
    assert_eq!(metrics[0].api_name, Some("sessions".to_string()));
}

#[test]
fn test_parse_account_summaries() {
    let response_json = sample_account_summaries_response();
    let response: ListAccountSummariesResponse = serde_json::from_value(response_json).unwrap();

    let summaries = response.account_summaries.unwrap();
    assert_eq!(summaries.len(), 1);

    let account = &summaries[0];
    assert_eq!(account.display_name, Some("Test Account".to_string()));

    let properties = account.property_summaries.as_ref().unwrap();
    assert_eq!(properties.len(), 2);
    assert_eq!(properties[0].display_name, Some("Test Property".to_string()));
}

#[test]
fn test_parse_empty_response() {
    let response_json = json!({
        "dimensionHeaders": [],
        "metricHeaders": [],
        "rows": [],
        "rowCount": 0
    });

    let response: RunReportResponse = serde_json::from_value(response_json).unwrap();

    let rows = response.rows.unwrap();
    assert!(rows.is_empty());
    assert_eq!(response.row_count, Some(0));
}

#[test]
fn test_parse_response_with_null_values() {
    let response_json = json!({
        "dimensionHeaders": [{ "name": "country" }],
        "metricHeaders": [{ "name": "sessions", "type": "TYPE_INTEGER" }],
        "rows": [
            {
                "dimensionValues": [{ "value": null }],
                "metricValues": [{ "value": "0" }]
            }
        ],
        "rowCount": 1
    });

    let response: RunReportResponse = serde_json::from_value(response_json).unwrap();

    let rows = response.rows.unwrap();
    let first_row = &rows[0];
    let dim_values = first_row.dimension_values.as_ref().unwrap();
    assert!(dim_values[0].value.is_none());
}

// ============================================================================
// Request Serialization Tests
// ============================================================================

#[test]
fn test_serialize_run_report_request() {
    let request = RunReportRequest {
        property: "properties/123456789".to_string(),
        date_ranges: vec![ApiDateRange {
            start_date: "2023-12-01".to_string(),
            end_date: "2023-12-31".to_string(),
            name: Some("December".to_string()),
        }],
        dimensions: Some(vec![
            Dimension { name: "country".to_string(), dimension_expression: None },
            Dimension { name: "city".to_string(), dimension_expression: None },
        ]),
        metrics: vec![
            Metric { name: "sessions".to_string(), expression: None, invisible: None },
            Metric { name: "totalUsers".to_string(), expression: None, invisible: None },
        ],
        dimension_filter: None,
        metric_filter: None,
        order_bys: Some(vec![
            OrderBy {
                desc: Some(true),
                metric: Some(MetricOrderBy { metric_name: "sessions".to_string() }),
                dimension: None,
                pivot: None,
            }
        ]),
        offset: Some(0),
        limit: Some(100),
        metric_aggregations: Some(vec!["TOTAL".to_string()]),
        keep_empty_rows: Some(false),
        return_property_quota: Some(true),
    };

    let json = serde_json::to_value(&request).unwrap();

    assert_eq!(json["property"], "properties/123456789");
    assert_eq!(json["dateRanges"][0]["startDate"], "2023-12-01");
    assert_eq!(json["dimensions"].as_array().unwrap().len(), 2);
    assert_eq!(json["metrics"].as_array().unwrap().len(), 2);
    assert_eq!(json["limit"], 100);
}

#[test]
fn test_serialize_realtime_request() {
    let request = RunRealtimeReportRequest {
        property: "properties/123456789".to_string(),
        dimensions: Some(vec![
            Dimension { name: "country".to_string(), dimension_expression: None },
        ]),
        metrics: vec![
            Metric { name: "activeUsers".to_string(), expression: None, invisible: None },
        ],
        dimension_filter: None,
        metric_filter: None,
        limit: Some(10),
        metric_aggregations: None,
        order_bys: None,
        return_property_quota: None,
        minute_ranges: Some(vec![
            MinuteRange {
                name: Some("last30".to_string()),
                start_minutes_ago: Some(29),
                end_minutes_ago: Some(0),
            }
        ]),
    };

    let json = serde_json::to_value(&request).unwrap();

    assert_eq!(json["property"], "properties/123456789");
    assert!(json["minuteRanges"].is_array());
}

#[test]
fn test_serialize_filter_expression() {
    let filter = FilterExpression {
        and_group: Some(FilterExpressionList {
            expressions: vec![
                GoogleAnalyticsClient::string_filter("country", StringFilterMatchType::Exact, "US"),
                GoogleAnalyticsClient::numeric_filter("sessions", NumericFilterOperation::GreaterThan, 10.0),
            ]
        }),
        or_group: None,
        not_expression: None,
        filter: None,
    };

    let json = serde_json::to_value(&filter).unwrap();

    assert!(json["andGroup"].is_object());
    assert_eq!(json["andGroup"]["expressions"].as_array().unwrap().len(), 2);
}

// ============================================================================
// Date Range Preset Tests
// ============================================================================

#[test]
fn test_date_range_presets() {
    // Test all presets create valid date ranges
    let presets = vec![
        DateRangePreset::Today,
        DateRangePreset::Yesterday,
        DateRangePreset::Last7Days,
        DateRangePreset::Last14Days,
        DateRangePreset::Last28Days,
        DateRangePreset::Last30Days,
        DateRangePreset::Last90Days,
        DateRangePreset::ThisMonth,
        DateRangePreset::LastMonth,
    ];

    for preset in presets {
        let date_range = match preset {
            DateRangePreset::Today => DateRange::today(),
            DateRangePreset::Yesterday => DateRange::yesterday(),
            DateRangePreset::Last7Days => DateRange::last_n_days(7),
            DateRangePreset::Last14Days => DateRange::last_n_days(14),
            DateRangePreset::Last28Days => DateRange::last_n_days(28),
            DateRangePreset::Last30Days => DateRange::last_n_days(30),
            DateRangePreset::Last90Days => DateRange::last_n_days(90),
            DateRangePreset::ThisMonth => DateRange::this_month(),
            DateRangePreset::LastMonth => DateRange::last_month(),
            _ => DateRange::last_n_days(30),
        };

        let api_range = GoogleAnalyticsClient::build_date_range(&date_range);

        // All ranges should have valid dates
        assert!(!api_range.start_date.is_empty());
        assert!(!api_range.end_date.is_empty());

        // Start date should be <= end date
        assert!(api_range.start_date <= api_range.end_date);
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_client_error_display() {
    let errors = vec![
        (ClientError::AuthenticationFailed("Invalid token".to_string()), "Authentication failed"),
        (ClientError::RequestFailed("Server error".to_string()), "API request failed"),
        (ClientError::InvalidResponse("Bad JSON".to_string()), "Invalid response"),
        (ClientError::RateLimited(60), "Rate limited"),
        (ClientError::QuotaExceeded("Daily limit".to_string()), "Quota exceeded"),
        (ClientError::PropertyNotFound("12345".to_string()), "Property not found"),
        (ClientError::InvalidCredentials("Bad key".to_string()), "Invalid credentials"),
    ];

    for (error, expected_prefix) in errors {
        let error_str = error.to_string();
        assert!(error_str.contains(expected_prefix),
            "Error '{}' should contain '{}'", error_str, expected_prefix);
    }
}

#[test]
fn test_parse_api_error_response_invalid_argument() {
    let error_json = sample_error_response(400, "Invalid dimension: unknown_dimension", "INVALID_ARGUMENT");
    let parsed: GoogleApiError = serde_json::from_value(error_json).unwrap();

    assert_eq!(parsed.error.code, 400);
    assert!(parsed.error.message.contains("Invalid dimension"));
    assert_eq!(parsed.error.status, "INVALID_ARGUMENT");
}

#[test]
fn test_parse_api_error_response_permission_denied() {
    let error_json = sample_error_response(403, "User does not have sufficient permissions", "PERMISSION_DENIED");
    let parsed: GoogleApiError = serde_json::from_value(error_json).unwrap();

    assert_eq!(parsed.error.code, 403);
    assert!(parsed.error.message.contains("permissions"));
    assert_eq!(parsed.error.status, "PERMISSION_DENIED");
}

#[test]
fn test_parse_api_error_response_not_found() {
    let error_json = sample_error_response(404, "Property not found: properties/999999999", "NOT_FOUND");
    let parsed: GoogleApiError = serde_json::from_value(error_json).unwrap();

    assert_eq!(parsed.error.code, 404);
    assert!(parsed.error.message.contains("Property not found"));
    assert_eq!(parsed.error.status, "NOT_FOUND");
}

#[test]
fn test_parse_api_error_response_rate_limited() {
    let error_json = sample_error_response(429, "Quota exceeded for quota metric 'Requests per minute'", "RESOURCE_EXHAUSTED");
    let parsed: GoogleApiError = serde_json::from_value(error_json).unwrap();

    assert_eq!(parsed.error.code, 429);
    assert!(parsed.error.message.contains("Quota exceeded"));
    assert_eq!(parsed.error.status, "RESOURCE_EXHAUSTED");
}

#[test]
fn test_parse_api_error_response_internal_error() {
    let error_json = sample_error_response(500, "Internal server error", "INTERNAL");
    let parsed: GoogleApiError = serde_json::from_value(error_json).unwrap();

    assert_eq!(parsed.error.code, 500);
    assert_eq!(parsed.error.status, "INTERNAL");
}

// ============================================================================
// Complex Request Building Tests
// ============================================================================

#[test]
fn test_build_complex_report_request() {
    let date_range = DateRange::last_n_days(30);

    // Build a complex report request with multiple dimensions, metrics, and filters
    let request = RunReportRequest {
        property: "properties/123456789".to_string(),
        date_ranges: vec![GoogleAnalyticsClient::build_date_range(&date_range)],
        dimensions: Some(vec![
            GoogleAnalyticsClient::dimension("date"),
            GoogleAnalyticsClient::dimension("country"),
            GoogleAnalyticsClient::dimension("city"),
            GoogleAnalyticsClient::dimension("deviceCategory"),
        ]),
        metrics: vec![
            GoogleAnalyticsClient::metric("sessions"),
            GoogleAnalyticsClient::metric("totalUsers"),
            GoogleAnalyticsClient::metric("newUsers"),
            GoogleAnalyticsClient::metric("bounceRate"),
            GoogleAnalyticsClient::metric("averageSessionDuration"),
        ],
        dimension_filter: Some(FilterExpression {
            and_group: Some(FilterExpressionList {
                expressions: vec![
                    GoogleAnalyticsClient::string_filter("country", StringFilterMatchType::Exact, "United States"),
                    GoogleAnalyticsClient::string_filter("deviceCategory", StringFilterMatchType::Exact, "desktop"),
                ]
            }),
            or_group: None,
            not_expression: None,
            filter: None,
        }),
        metric_filter: Some(GoogleAnalyticsClient::numeric_filter(
            "sessions",
            NumericFilterOperation::GreaterThan,
            10.0
        )),
        order_bys: Some(vec![
            GoogleAnalyticsClient::order_by_metric_desc("sessions"),
        ]),
        offset: Some(0),
        limit: Some(1000),
        metric_aggregations: Some(vec!["TOTAL".to_string(), "MAXIMUM".to_string(), "MINIMUM".to_string()]),
        keep_empty_rows: Some(false),
        return_property_quota: Some(true),
    };

    // Serialize and verify structure
    let json = serde_json::to_value(&request).unwrap();

    assert_eq!(json["dimensions"].as_array().unwrap().len(), 4);
    assert_eq!(json["metrics"].as_array().unwrap().len(), 5);
    assert!(json["dimensionFilter"]["andGroup"]["expressions"].is_array());
    assert!(json["metricFilter"]["filter"]["numericFilter"].is_object());
    assert_eq!(json["metricAggregations"].as_array().unwrap().len(), 3);
}

#[test]
fn test_build_pivot_report_request() {
    let request = RunPivotReportRequest {
        property: "properties/123456789".to_string(),
        date_ranges: vec![ApiDateRange {
            start_date: "2023-12-01".to_string(),
            end_date: "2023-12-31".to_string(),
            name: None,
        }],
        dimensions: Some(vec![
            GoogleAnalyticsClient::dimension("country"),
            GoogleAnalyticsClient::dimension("deviceCategory"),
        ]),
        metrics: vec![
            GoogleAnalyticsClient::metric("sessions"),
        ],
        pivots: vec![
            Pivot {
                field_names: vec!["deviceCategory".to_string()],
                order_bys: None,
                offset: None,
                limit: Some(5),
                metric_aggregations: None,
            }
        ],
        dimension_filter: None,
        metric_filter: None,
        keep_empty_rows: None,
        return_property_quota: None,
    };

    let json = serde_json::to_value(&request).unwrap();

    assert!(json["pivots"].is_array());
    assert_eq!(json["pivots"][0]["fieldNames"][0], "deviceCategory");
}

// ============================================================================
// Async Integration Tests with Mock Server
// ============================================================================

#[tokio::test]
async fn test_parse_successful_report_response_async() {
    // This test verifies async parsing of report responses
    let response_json = sample_report_response();
    let response: RunReportResponse = serde_json::from_value(response_json).unwrap();

    // Async verification
    tokio::task::spawn_blocking(move || {
        assert!(response.rows.is_some());
        assert_eq!(response.row_count, Some(2));
    }).await.unwrap();
}

#[tokio::test]
async fn test_date_range_calculations_async() {
    // Test date range calculations in async context
    let ranges = tokio::task::spawn_blocking(|| {
        vec![
            DateRange::today(),
            DateRange::yesterday(),
            DateRange::last_n_days(7),
            DateRange::last_n_days(30),
            DateRange::this_month(),
            DateRange::last_month(),
        ]
    }).await.unwrap();

    for range in ranges {
        let api_range = GoogleAnalyticsClient::build_date_range(&range);
        assert!(api_range.start_date <= api_range.end_date);
    }
}

// ============================================================================
// Property Quota Tests
// ============================================================================

#[test]
fn test_parse_property_quota() {
    let quota_json = json!({
        "tokensPerDay": {
            "consumed": 100,
            "remaining": 24900
        },
        "tokensPerHour": {
            "consumed": 50,
            "remaining": 4950
        },
        "concurrentRequests": {
            "consumed": 2,
            "remaining": 8
        },
        "serverErrorsPerProjectPerHour": {
            "consumed": 0,
            "remaining": 10
        },
        "potentiallyThresholdedRequestsPerHour": {
            "consumed": 5,
            "remaining": 115
        }
    });

    let quota: PropertyQuota = serde_json::from_value(quota_json).unwrap();

    let tokens_per_day = quota.tokens_per_day.unwrap();
    assert_eq!(tokens_per_day.consumed, Some(100));
    assert_eq!(tokens_per_day.remaining, Some(24900));
}

// ============================================================================
// Batch Request Tests
// ============================================================================

#[test]
fn test_batch_run_reports_request() {
    let requests = vec![
        RunReportRequest {
            property: "properties/123456789".to_string(),
            date_ranges: vec![ApiDateRange {
                start_date: "2023-12-01".to_string(),
                end_date: "2023-12-31".to_string(),
                name: None,
            }],
            dimensions: Some(vec![GoogleAnalyticsClient::dimension("country")]),
            metrics: vec![GoogleAnalyticsClient::metric("sessions")],
            dimension_filter: None,
            metric_filter: None,
            order_bys: None,
            offset: None,
            limit: Some(10),
            metric_aggregations: None,
            keep_empty_rows: None,
            return_property_quota: None,
        },
        RunReportRequest {
            property: "properties/123456789".to_string(),
            date_ranges: vec![ApiDateRange {
                start_date: "2023-12-01".to_string(),
                end_date: "2023-12-31".to_string(),
                name: None,
            }],
            dimensions: Some(vec![GoogleAnalyticsClient::dimension("deviceCategory")]),
            metrics: vec![GoogleAnalyticsClient::metric("totalUsers")],
            dimension_filter: None,
            metric_filter: None,
            order_bys: None,
            offset: None,
            limit: Some(10),
            metric_aggregations: None,
            keep_empty_rows: None,
            return_property_quota: None,
        },
    ];

    let batch_request = BatchRunReportsRequest {
        property: "properties/123456789".to_string(),
        requests,
    };
    let json = serde_json::to_value(&batch_request).unwrap();

    assert_eq!(json["requests"].as_array().unwrap().len(), 2);
    assert_eq!(json["property"], "properties/123456789");
}

#[test]
fn test_parse_batch_response() {
    let batch_response_json = json!({
        "reports": [
            {
                "dimensionHeaders": [{ "name": "country" }],
                "metricHeaders": [{ "name": "sessions", "type": "TYPE_INTEGER" }],
                "rows": [
                    {
                        "dimensionValues": [{ "value": "US" }],
                        "metricValues": [{ "value": "100" }]
                    }
                ],
                "rowCount": 1
            },
            {
                "dimensionHeaders": [{ "name": "deviceCategory" }],
                "metricHeaders": [{ "name": "totalUsers", "type": "TYPE_INTEGER" }],
                "rows": [
                    {
                        "dimensionValues": [{ "value": "mobile" }],
                        "metricValues": [{ "value": "50" }]
                    }
                ],
                "rowCount": 1
            }
        ]
    });

    let response: BatchRunReportsResponse = serde_json::from_value(batch_response_json).unwrap();

    let reports = response.reports.unwrap();
    assert_eq!(reports.len(), 2);

    let first_report = &reports[0];
    assert_eq!(first_report.row_count, Some(1));
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_empty_dimensions_request() {
    let request = RunReportRequest {
        property: "properties/123456789".to_string(),
        date_ranges: vec![ApiDateRange {
            start_date: "2023-12-01".to_string(),
            end_date: "2023-12-31".to_string(),
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

    let json = serde_json::to_value(&request).unwrap();

    // dimensions should be null/absent
    assert!(json.get("dimensions").is_none() || json["dimensions"].is_null());
}

#[test]
fn test_large_limit_value() {
    let request = RunReportRequest {
        property: "properties/123456789".to_string(),
        date_ranges: vec![ApiDateRange {
            start_date: "2023-01-01".to_string(),
            end_date: "2023-12-31".to_string(),
            name: None,
        }],
        dimensions: Some(vec![GoogleAnalyticsClient::dimension("date")]),
        metrics: vec![GoogleAnalyticsClient::metric("sessions")],
        dimension_filter: None,
        metric_filter: None,
        order_bys: None,
        offset: None,
        limit: Some(100000),
        metric_aggregations: None,
        keep_empty_rows: None,
        return_property_quota: None,
    };

    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["limit"], 100000);
}

#[test]
fn test_special_characters_in_filter_value() {
    let filter = GoogleAnalyticsClient::string_filter(
        "pagePath",
        StringFilterMatchType::Contains,
        "/path/with spaces & special=chars?query=1"
    );

    let json = serde_json::to_value(&filter).unwrap();
    assert_eq!(
        json["filter"]["stringFilter"]["value"],
        "/path/with spaces & special=chars?query=1"
    );
}

#[test]
fn test_unicode_in_dimension_value() {
    let filter = GoogleAnalyticsClient::string_filter(
        "country",
        StringFilterMatchType::Exact,
        "日本"
    );

    let json = serde_json::to_value(&filter).unwrap();
    assert_eq!(json["filter"]["stringFilter"]["value"], "日本");
}
