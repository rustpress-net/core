//! Tests for the Handlers Module

use serde_json;

use rustanalytics::handlers::{
    AnalyticsQuery, ApiResponse, ResponseMeta,
    // Overview handlers
    overview_handler, realtime_handler, realtime_active_users_handler,
    // Audience handlers
    audience_overview_handler, demographics_handler, geo_handler,
    technology_handler, mobile_handler, audience_behavior_handler,
    // Acquisition handlers
    acquisition_overview_handler, channels_handler, source_medium_handler,
    referrals_handler, campaigns_handler, social_handler, search_console_handler,
    // Behavior handlers
    behavior_overview_handler, site_content_handler, landing_pages_handler,
    exit_pages_handler, site_speed_handler, site_search_handler, events_handler,
    // Conversions handlers
    goals_handler, funnel_handler, multi_channel_handler, attribution_handler,
    // E-commerce handlers
    ecommerce_overview_handler, products_handler, sales_handler,
    transactions_handler, shopping_behavior_handler, checkout_behavior_handler,
    // Reports handlers
    list_reports_handler, create_report_handler, get_report_handler,
    update_report_handler, delete_report_handler, run_report_handler, export_report_handler,
    // Settings handlers
    get_settings_handler, update_settings_handler, test_connection_handler, authenticate_handler,
    // Data management handlers
    export_data_handler, sync_data_handler, clear_cache_handler,
};

// ============================================================================
// AnalyticsQuery Tests
// ============================================================================

#[test]
fn test_analytics_query_full_deserialization() {
    let json = r#"{
        "start_date": "2024-01-01",
        "end_date": "2024-01-31",
        "date_range": "last_30_days",
        "compare": true,
        "limit": 100,
        "offset": 0
    }"#;

    let query: AnalyticsQuery = serde_json::from_str(json).unwrap();

    assert_eq!(query.start_date, Some("2024-01-01".to_string()));
    assert_eq!(query.end_date, Some("2024-01-31".to_string()));
    assert_eq!(query.date_range, Some("last_30_days".to_string()));
    assert_eq!(query.compare, Some(true));
    assert_eq!(query.limit, Some(100));
    assert_eq!(query.offset, Some(0));
}

#[test]
fn test_analytics_query_partial_deserialization() {
    let json = r#"{
        "start_date": "2024-01-01",
        "end_date": "2024-01-31"
    }"#;

    let query: AnalyticsQuery = serde_json::from_str(json).unwrap();

    assert_eq!(query.start_date, Some("2024-01-01".to_string()));
    assert_eq!(query.end_date, Some("2024-01-31".to_string()));
    assert!(query.date_range.is_none());
    assert!(query.compare.is_none());
    assert!(query.limit.is_none());
    assert!(query.offset.is_none());
}

#[test]
fn test_analytics_query_empty_deserialization() {
    let json = r#"{}"#;

    let query: AnalyticsQuery = serde_json::from_str(json).unwrap();

    assert!(query.start_date.is_none());
    assert!(query.end_date.is_none());
    assert!(query.date_range.is_none());
    assert!(query.compare.is_none());
    assert!(query.limit.is_none());
    assert!(query.offset.is_none());
}

#[test]
fn test_analytics_query_only_date_range() {
    let json = r#"{
        "date_range": "last_7_days"
    }"#;

    let query: AnalyticsQuery = serde_json::from_str(json).unwrap();

    assert!(query.start_date.is_none());
    assert!(query.end_date.is_none());
    assert_eq!(query.date_range, Some("last_7_days".to_string()));
}

#[test]
fn test_analytics_query_with_pagination() {
    let json = r#"{
        "limit": 50,
        "offset": 100
    }"#;

    let query: AnalyticsQuery = serde_json::from_str(json).unwrap();

    assert_eq!(query.limit, Some(50));
    assert_eq!(query.offset, Some(100));
}

#[test]
fn test_analytics_query_compare_false() {
    let json = r#"{
        "compare": false
    }"#;

    let query: AnalyticsQuery = serde_json::from_str(json).unwrap();

    assert_eq!(query.compare, Some(false));
}

#[test]
fn test_analytics_query_debug_impl() {
    let json = r#"{
        "start_date": "2024-01-01",
        "limit": 10
    }"#;

    let query: AnalyticsQuery = serde_json::from_str(json).unwrap();
    let debug_str = format!("{:?}", query);

    assert!(debug_str.contains("AnalyticsQuery"));
    assert!(debug_str.contains("start_date"));
    assert!(debug_str.contains("2024-01-01"));
}

#[test]
fn test_analytics_query_clone() {
    let json = r#"{
        "start_date": "2024-01-01",
        "end_date": "2024-01-31",
        "limit": 25
    }"#;

    let query: AnalyticsQuery = serde_json::from_str(json).unwrap();
    let cloned = query.clone();

    assert_eq!(cloned.start_date, query.start_date);
    assert_eq!(cloned.end_date, query.end_date);
    assert_eq!(cloned.limit, query.limit);
}

#[test]
fn test_analytics_query_large_limit() {
    let json = r#"{
        "limit": 10000,
        "offset": 50000
    }"#;

    let query: AnalyticsQuery = serde_json::from_str(json).unwrap();

    assert_eq!(query.limit, Some(10000));
    assert_eq!(query.offset, Some(50000));
}

#[test]
fn test_analytics_query_zero_values() {
    let json = r#"{
        "limit": 0,
        "offset": 0
    }"#;

    let query: AnalyticsQuery = serde_json::from_str(json).unwrap();

    assert_eq!(query.limit, Some(0));
    assert_eq!(query.offset, Some(0));
}

#[test]
fn test_analytics_query_negative_offset() {
    let json = r#"{
        "offset": -10
    }"#;

    let query: AnalyticsQuery = serde_json::from_str(json).unwrap();

    assert_eq!(query.offset, Some(-10));
}

#[test]
fn test_analytics_query_various_date_ranges() {
    let ranges = vec![
        "today",
        "yesterday",
        "last_7_days",
        "last_30_days",
        "last_90_days",
        "this_month",
        "last_month",
        "this_year",
    ];

    for range in ranges {
        let json = format!(r#"{{"date_range": "{}"}}"#, range);
        let query: AnalyticsQuery = serde_json::from_str(&json).unwrap();
        assert_eq!(query.date_range, Some(range.to_string()));
    }
}

// ============================================================================
// ApiResponse Tests
// ============================================================================

#[test]
fn test_api_response_success_with_string() {
    let response: ApiResponse<String> = ApiResponse::success("Hello World".to_string());

    assert!(response.success);
    assert_eq!(response.data, Some("Hello World".to_string()));
    assert!(response.error.is_none());
    assert!(response.meta.is_some());
}

#[test]
fn test_api_response_success_with_number() {
    let response: ApiResponse<i32> = ApiResponse::success(42);

    assert!(response.success);
    assert_eq!(response.data, Some(42));
    assert!(response.error.is_none());
}

#[test]
fn test_api_response_success_with_vec() {
    let data = vec![1, 2, 3, 4, 5];
    let response: ApiResponse<Vec<i32>> = ApiResponse::success(data.clone());

    assert!(response.success);
    assert_eq!(response.data, Some(data));
    assert!(response.error.is_none());
}

#[test]
fn test_api_response_success_with_struct() {
    #[derive(Debug, Clone, PartialEq, serde::Serialize)]
    struct TestData {
        name: String,
        value: i32,
    }

    let data = TestData {
        name: "test".to_string(),
        value: 100,
    };
    let response: ApiResponse<TestData> = ApiResponse::success(data.clone());

    assert!(response.success);
    assert_eq!(response.data, Some(data));
}

#[test]
fn test_api_response_error() {
    let response: ApiResponse<String> = ApiResponse::error("Something went wrong");

    assert!(!response.success);
    assert!(response.data.is_none());
    assert_eq!(response.error, Some("Something went wrong".to_string()));
    assert!(response.meta.is_some());
}

#[test]
fn test_api_response_error_with_detailed_message() {
    let message = "Invalid request: missing required field 'property_id'";
    let response: ApiResponse<()> = ApiResponse::error(message);

    assert!(!response.success);
    assert_eq!(response.error, Some(message.to_string()));
}

#[test]
fn test_api_response_error_empty_message() {
    let response: ApiResponse<String> = ApiResponse::error("");

    assert!(!response.success);
    assert_eq!(response.error, Some("".to_string()));
}

#[test]
fn test_api_response_success_serialization() {
    let response: ApiResponse<String> = ApiResponse::success("test data".to_string());

    let json = serde_json::to_string(&response).unwrap();

    assert!(json.contains("\"success\":true"));
    assert!(json.contains("\"data\":\"test data\""));
    assert!(json.contains("\"error\":null"));
    assert!(json.contains("\"meta\""));
}

#[test]
fn test_api_response_error_serialization() {
    let response: ApiResponse<String> = ApiResponse::error("error message");

    let json = serde_json::to_string(&response).unwrap();

    assert!(json.contains("\"success\":false"));
    assert!(json.contains("\"data\":null"));
    assert!(json.contains("\"error\":\"error message\""));
}

#[test]
fn test_api_response_meta_has_request_id() {
    let response: ApiResponse<String> = ApiResponse::success("test".to_string());

    let meta = response.meta.unwrap();
    assert!(!meta.request_id.is_empty());
    // UUID v4 format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
    assert_eq!(meta.request_id.len(), 36);
    assert!(meta.request_id.contains('-'));
}

#[test]
fn test_api_response_meta_has_timestamp() {
    let response: ApiResponse<String> = ApiResponse::success("test".to_string());

    let meta = response.meta.unwrap();
    // Timestamp should be recent (within last minute)
    let now = chrono::Utc::now();
    let diff = now - meta.timestamp;
    assert!(diff.num_seconds() < 60);
}

#[test]
fn test_api_response_meta_cached_is_false() {
    let response: ApiResponse<String> = ApiResponse::success("test".to_string());

    let meta = response.meta.unwrap();
    assert!(!meta.cached);
}

#[test]
fn test_api_response_unique_request_ids() {
    let response1: ApiResponse<String> = ApiResponse::success("test1".to_string());
    let response2: ApiResponse<String> = ApiResponse::success("test2".to_string());

    let id1 = response1.meta.unwrap().request_id;
    let id2 = response2.meta.unwrap().request_id;

    assert_ne!(id1, id2);
}

#[test]
fn test_api_response_debug_impl() {
    let response: ApiResponse<String> = ApiResponse::success("test".to_string());
    let debug_str = format!("{:?}", response);

    assert!(debug_str.contains("ApiResponse"));
    assert!(debug_str.contains("success"));
    assert!(debug_str.contains("data"));
}

#[test]
fn test_api_response_clone() {
    let response: ApiResponse<String> = ApiResponse::success("test".to_string());
    let cloned = response.clone();

    assert_eq!(cloned.success, response.success);
    assert_eq!(cloned.data, response.data);
    assert_eq!(cloned.error, response.error);
}

#[test]
fn test_api_response_with_complex_data() {
    #[derive(Debug, Clone, PartialEq, serde::Serialize)]
    struct AnalyticsData {
        sessions: u64,
        users: u64,
        pageviews: u64,
        bounce_rate: f64,
    }

    let data = AnalyticsData {
        sessions: 10000,
        users: 5000,
        pageviews: 25000,
        bounce_rate: 45.5,
    };

    let response: ApiResponse<AnalyticsData> = ApiResponse::success(data.clone());

    assert!(response.success);
    assert_eq!(response.data.as_ref().unwrap().sessions, 10000);
    assert_eq!(response.data.as_ref().unwrap().bounce_rate, 45.5);
}

#[test]
fn test_api_response_with_option_data() {
    let response: ApiResponse<Option<String>> = ApiResponse::success(Some("value".to_string()));

    assert!(response.success);
    assert_eq!(response.data, Some(Some("value".to_string())));
}

#[test]
fn test_api_response_with_none_option() {
    let response: ApiResponse<Option<String>> = ApiResponse::success(None);

    assert!(response.success);
    assert_eq!(response.data, Some(None));
}

// ============================================================================
// ResponseMeta Tests
// ============================================================================

#[test]
fn test_response_meta_creation() {
    let meta = ResponseMeta {
        cached: true,
        request_id: "abc-123".to_string(),
        timestamp: chrono::Utc::now(),
    };

    assert!(meta.cached);
    assert_eq!(meta.request_id, "abc-123");
}

#[test]
fn test_response_meta_serialization() {
    let meta = ResponseMeta {
        cached: false,
        request_id: "test-id-456".to_string(),
        timestamp: chrono::Utc::now(),
    };

    let json = serde_json::to_string(&meta).unwrap();

    assert!(json.contains("\"cached\":false"));
    assert!(json.contains("\"request_id\":\"test-id-456\""));
    assert!(json.contains("\"timestamp\""));
}

#[test]
fn test_response_meta_debug_impl() {
    let meta = ResponseMeta {
        cached: true,
        request_id: "debug-test".to_string(),
        timestamp: chrono::Utc::now(),
    };

    let debug_str = format!("{:?}", meta);

    assert!(debug_str.contains("ResponseMeta"));
    assert!(debug_str.contains("cached"));
    assert!(debug_str.contains("request_id"));
}

#[test]
fn test_response_meta_clone() {
    let meta = ResponseMeta {
        cached: true,
        request_id: "clone-test".to_string(),
        timestamp: chrono::Utc::now(),
    };

    let cloned = meta.clone();

    assert_eq!(cloned.cached, meta.cached);
    assert_eq!(cloned.request_id, meta.request_id);
    assert_eq!(cloned.timestamp, meta.timestamp);
}

// ============================================================================
// Overview Handlers Tests
// ============================================================================

#[tokio::test]
async fn test_overview_handler_exists() {
    // Handler should be callable without panic
    overview_handler().await;
}

#[tokio::test]
async fn test_realtime_handler_exists() {
    realtime_handler().await;
}

#[tokio::test]
async fn test_realtime_active_users_handler_exists() {
    realtime_active_users_handler().await;
}

// ============================================================================
// Audience Handlers Tests
// ============================================================================

#[tokio::test]
async fn test_audience_overview_handler_exists() {
    audience_overview_handler().await;
}

#[tokio::test]
async fn test_demographics_handler_exists() {
    demographics_handler().await;
}

#[tokio::test]
async fn test_geo_handler_exists() {
    geo_handler().await;
}

#[tokio::test]
async fn test_technology_handler_exists() {
    technology_handler().await;
}

#[tokio::test]
async fn test_mobile_handler_exists() {
    mobile_handler().await;
}

#[tokio::test]
async fn test_audience_behavior_handler_exists() {
    audience_behavior_handler().await;
}

// ============================================================================
// Acquisition Handlers Tests
// ============================================================================

#[tokio::test]
async fn test_acquisition_overview_handler_exists() {
    acquisition_overview_handler().await;
}

#[tokio::test]
async fn test_channels_handler_exists() {
    channels_handler().await;
}

#[tokio::test]
async fn test_source_medium_handler_exists() {
    source_medium_handler().await;
}

#[tokio::test]
async fn test_referrals_handler_exists() {
    referrals_handler().await;
}

#[tokio::test]
async fn test_campaigns_handler_exists() {
    campaigns_handler().await;
}

#[tokio::test]
async fn test_social_handler_exists() {
    social_handler().await;
}

#[tokio::test]
async fn test_search_console_handler_exists() {
    search_console_handler().await;
}

// ============================================================================
// Behavior Handlers Tests
// ============================================================================

#[tokio::test]
async fn test_behavior_overview_handler_exists() {
    behavior_overview_handler().await;
}

#[tokio::test]
async fn test_site_content_handler_exists() {
    site_content_handler().await;
}

#[tokio::test]
async fn test_landing_pages_handler_exists() {
    landing_pages_handler().await;
}

#[tokio::test]
async fn test_exit_pages_handler_exists() {
    exit_pages_handler().await;
}

#[tokio::test]
async fn test_site_speed_handler_exists() {
    site_speed_handler().await;
}

#[tokio::test]
async fn test_site_search_handler_exists() {
    site_search_handler().await;
}

#[tokio::test]
async fn test_events_handler_exists() {
    events_handler().await;
}

// ============================================================================
// Conversions Handlers Tests
// ============================================================================

#[tokio::test]
async fn test_goals_handler_exists() {
    goals_handler().await;
}

#[tokio::test]
async fn test_funnel_handler_exists() {
    funnel_handler().await;
}

#[tokio::test]
async fn test_multi_channel_handler_exists() {
    multi_channel_handler().await;
}

#[tokio::test]
async fn test_attribution_handler_exists() {
    attribution_handler().await;
}

// ============================================================================
// E-commerce Handlers Tests
// ============================================================================

#[tokio::test]
async fn test_ecommerce_overview_handler_exists() {
    ecommerce_overview_handler().await;
}

#[tokio::test]
async fn test_products_handler_exists() {
    products_handler().await;
}

#[tokio::test]
async fn test_sales_handler_exists() {
    sales_handler().await;
}

#[tokio::test]
async fn test_transactions_handler_exists() {
    transactions_handler().await;
}

#[tokio::test]
async fn test_shopping_behavior_handler_exists() {
    shopping_behavior_handler().await;
}

#[tokio::test]
async fn test_checkout_behavior_handler_exists() {
    checkout_behavior_handler().await;
}

// ============================================================================
// Reports Handlers Tests
// ============================================================================

#[tokio::test]
async fn test_list_reports_handler_exists() {
    list_reports_handler().await;
}

#[tokio::test]
async fn test_create_report_handler_exists() {
    create_report_handler().await;
}

#[tokio::test]
async fn test_get_report_handler_exists() {
    get_report_handler().await;
}

#[tokio::test]
async fn test_update_report_handler_exists() {
    update_report_handler().await;
}

#[tokio::test]
async fn test_delete_report_handler_exists() {
    delete_report_handler().await;
}

#[tokio::test]
async fn test_run_report_handler_exists() {
    run_report_handler().await;
}

#[tokio::test]
async fn test_export_report_handler_exists() {
    export_report_handler().await;
}

// ============================================================================
// Settings Handlers Tests
// ============================================================================

#[tokio::test]
async fn test_get_settings_handler_exists() {
    get_settings_handler().await;
}

#[tokio::test]
async fn test_update_settings_handler_exists() {
    update_settings_handler().await;
}

#[tokio::test]
async fn test_test_connection_handler_exists() {
    test_connection_handler().await;
}

#[tokio::test]
async fn test_authenticate_handler_exists() {
    authenticate_handler().await;
}

// ============================================================================
// Data Management Handlers Tests
// ============================================================================

#[tokio::test]
async fn test_export_data_handler_exists() {
    export_data_handler().await;
}

#[tokio::test]
async fn test_sync_data_handler_exists() {
    sync_data_handler().await;
}

#[tokio::test]
async fn test_clear_cache_handler_exists() {
    clear_cache_handler().await;
}

// ============================================================================
// Edge Cases and Special Scenarios
// ============================================================================

#[test]
fn test_analytics_query_with_unicode_date_range() {
    let json = r#"{
        "date_range": "最後の7日間"
    }"#;

    let query: AnalyticsQuery = serde_json::from_str(json).unwrap();
    assert_eq!(query.date_range, Some("最後の7日間".to_string()));
}

#[test]
fn test_api_response_error_with_unicode() {
    let response: ApiResponse<String> = ApiResponse::error("エラーが発生しました");

    assert!(!response.success);
    assert_eq!(response.error, Some("エラーが発生しました".to_string()));
}

#[test]
fn test_api_response_success_with_unicode_data() {
    let response: ApiResponse<String> = ApiResponse::success("日本語データ".to_string());

    assert!(response.success);
    assert_eq!(response.data, Some("日本語データ".to_string()));
}

#[test]
fn test_api_response_error_with_special_characters() {
    let message = "Error: <script>alert('xss')</script> & special \"chars\"";
    let response: ApiResponse<String> = ApiResponse::error(message);

    assert_eq!(response.error, Some(message.to_string()));

    // Should serialize to valid JSON (quotes are escaped)
    let json = serde_json::to_string(&response).unwrap();
    // Quotes should be escaped in JSON
    assert!(json.contains("\\\"chars\\\""));
    // The message should be present
    assert!(json.contains("Error:"));
}

#[test]
fn test_analytics_query_with_iso_dates() {
    let json = r#"{
        "start_date": "2024-01-15T00:00:00Z",
        "end_date": "2024-01-31T23:59:59Z"
    }"#;

    let query: AnalyticsQuery = serde_json::from_str(json).unwrap();
    assert!(query.start_date.unwrap().contains("2024-01-15"));
    assert!(query.end_date.unwrap().contains("2024-01-31"));
}

#[test]
fn test_api_response_with_empty_vec() {
    let response: ApiResponse<Vec<String>> = ApiResponse::success(vec![]);

    assert!(response.success);
    assert_eq!(response.data, Some(vec![]));
}

#[test]
fn test_api_response_with_nested_structure() {
    #[derive(Debug, Clone, PartialEq, serde::Serialize)]
    struct Inner {
        value: i32,
    }

    #[derive(Debug, Clone, PartialEq, serde::Serialize)]
    struct Outer {
        items: Vec<Inner>,
        total: i32,
    }

    let data = Outer {
        items: vec![Inner { value: 1 }, Inner { value: 2 }],
        total: 2,
    };

    let response: ApiResponse<Outer> = ApiResponse::success(data);

    assert!(response.success);
    assert_eq!(response.data.as_ref().unwrap().items.len(), 2);
    assert_eq!(response.data.as_ref().unwrap().total, 2);
}

#[test]
fn test_api_response_json_roundtrip() {
    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    struct TestData {
        id: i32,
        name: String,
    }

    let original = TestData {
        id: 42,
        name: "test".to_string(),
    };

    let response: ApiResponse<TestData> = ApiResponse::success(original);
    let json = serde_json::to_string(&response).unwrap();

    // Verify JSON structure
    assert!(json.contains("\"success\":true"));
    assert!(json.contains("\"id\":42"));
    assert!(json.contains("\"name\":\"test\""));
}

#[test]
fn test_multiple_api_responses_independent() {
    let response1: ApiResponse<i32> = ApiResponse::success(100);
    let response2: ApiResponse<i32> = ApiResponse::error("failed");
    let response3: ApiResponse<i32> = ApiResponse::success(200);

    assert!(response1.success);
    assert!(!response2.success);
    assert!(response3.success);

    assert_eq!(response1.data, Some(100));
    assert!(response2.data.is_none());
    assert_eq!(response3.data, Some(200));
}

#[test]
fn test_response_meta_timestamp_is_utc() {
    let response: ApiResponse<String> = ApiResponse::success("test".to_string());
    let meta = response.meta.unwrap();

    // Serialize and check for UTC indicator
    let json = serde_json::to_string(&meta).unwrap();
    // chrono UTC timestamps end with Z or +00:00
    assert!(json.contains("Z") || json.contains("+00:00"));
}

// ============================================================================
// Handler Concurrency Tests
// ============================================================================

#[tokio::test]
async fn test_handlers_can_run_concurrently() {
    let (r1, r2, r3) = tokio::join!(
        overview_handler(),
        realtime_handler(),
        audience_overview_handler()
    );

    // All should complete without panic
    // Results are unit type since handlers are stubs
    assert_eq!(r1, ());
    assert_eq!(r2, ());
    assert_eq!(r3, ());
}

#[tokio::test]
async fn test_all_overview_handlers_concurrent() {
    tokio::join!(
        overview_handler(),
        realtime_handler(),
        realtime_active_users_handler()
    );
}

#[tokio::test]
async fn test_all_audience_handlers_concurrent() {
    tokio::join!(
        audience_overview_handler(),
        demographics_handler(),
        geo_handler(),
        technology_handler(),
        mobile_handler(),
        audience_behavior_handler()
    );
}

#[tokio::test]
async fn test_all_acquisition_handlers_concurrent() {
    tokio::join!(
        acquisition_overview_handler(),
        channels_handler(),
        source_medium_handler(),
        referrals_handler(),
        campaigns_handler(),
        social_handler(),
        search_console_handler()
    );
}

#[tokio::test]
async fn test_all_behavior_handlers_concurrent() {
    tokio::join!(
        behavior_overview_handler(),
        site_content_handler(),
        landing_pages_handler(),
        exit_pages_handler(),
        site_speed_handler(),
        site_search_handler(),
        events_handler()
    );
}

#[tokio::test]
async fn test_all_conversion_handlers_concurrent() {
    tokio::join!(
        goals_handler(),
        funnel_handler(),
        multi_channel_handler(),
        attribution_handler()
    );
}

#[tokio::test]
async fn test_all_ecommerce_handlers_concurrent() {
    tokio::join!(
        ecommerce_overview_handler(),
        products_handler(),
        sales_handler(),
        transactions_handler(),
        shopping_behavior_handler(),
        checkout_behavior_handler()
    );
}

#[tokio::test]
async fn test_all_report_handlers_concurrent() {
    tokio::join!(
        list_reports_handler(),
        create_report_handler(),
        get_report_handler(),
        update_report_handler(),
        delete_report_handler(),
        run_report_handler(),
        export_report_handler()
    );
}

#[tokio::test]
async fn test_all_settings_handlers_concurrent() {
    tokio::join!(
        get_settings_handler(),
        update_settings_handler(),
        test_connection_handler(),
        authenticate_handler()
    );
}

#[tokio::test]
async fn test_all_data_management_handlers_concurrent() {
    tokio::join!(
        export_data_handler(),
        sync_data_handler(),
        clear_cache_handler()
    );
}
