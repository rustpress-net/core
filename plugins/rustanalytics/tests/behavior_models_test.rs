//! Tests for behavior analytics models
//!
//! This module contains comprehensive tests for all behavior model types
//! including serialization, deserialization, and edge cases.

use chrono::NaiveDate;
use rustanalytics::models::behavior::*;
use rustanalytics::models::DateRange;

// ============================================================================
// Helper Functions
// ============================================================================

fn sample_date_range() -> DateRange {
    DateRange {
        start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
    }
}

fn sample_pageviews_trend() -> PageviewsTrend {
    PageviewsTrend {
        date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        pageviews: 5000,
        unique_pageviews: 4200,
        avg_time_on_page: 125.5,
    }
}

fn sample_behavior_comparison() -> BehaviorComparison {
    BehaviorComparison {
        pageviews_change: 15.5,
        unique_pageviews_change: 12.0,
        avg_time_on_page_change: 8.5,
        bounce_rate_change: -3.2,
        exit_rate_change: -1.5,
    }
}

fn sample_behavior_overview() -> BehaviorOverview {
    BehaviorOverview {
        date_range: sample_date_range(),
        pageviews: 150000,
        unique_pageviews: 120000,
        avg_time_on_page: 145.5,
        bounce_rate: 45.5,
        exit_rate: 35.0,
        page_value: 2.50,
        pageviews_trend: vec![sample_pageviews_trend()],
        comparison: Some(sample_behavior_comparison()),
    }
}

fn sample_page_content_data() -> PageContentData {
    PageContentData {
        page_path: "/blog/rust-tutorial".to_string(),
        page_title: "Rust Programming Tutorial".to_string(),
        pageviews: 5000,
        unique_pageviews: 4200,
        avg_time_on_page: 180.0,
        entrances: 3000,
        bounce_rate: 35.0,
        exit_rate: 25.0,
        page_value: 3.50,
        percentage: 3.5,
    }
}

fn sample_content_drilldown_data() -> ContentDrilldownData {
    ContentDrilldownData {
        page_path_level: "/blog".to_string(),
        level: 1,
        pageviews: 50000,
        unique_pageviews: 42000,
        avg_time_on_page: 150.0,
        bounce_rate: 40.0,
        exit_rate: 30.0,
        page_value: 2.75,
        children: vec![],
    }
}

fn sample_landing_page_data() -> LandingPageData {
    LandingPageData {
        landing_page: "/".to_string(),
        sessions: 25000,
        new_users_percentage: 65.0,
        new_sessions: 16250,
        bounce_rate: 45.0,
        pages_per_session: 3.5,
        avg_session_duration: 185.0,
        goal_conversion_rate: 3.5,
        goal_completions: 875,
        goal_value: 4375.0,
        transactions: 250,
        revenue: 12500.0,
        percentage: 25.0,
    }
}

fn sample_exit_page_data() -> ExitPageData {
    ExitPageData {
        exit_page: "/checkout/complete".to_string(),
        exits: 5000,
        pageviews: 6000,
        exit_rate: 83.3,
        percentage: 5.0,
    }
}

fn sample_flow_step_connection() -> FlowStepConnection {
    FlowStepConnection {
        to_node: "node_2".to_string(),
        sessions: 800,
        percentage: 80.0,
    }
}

fn sample_flow_step_node() -> FlowStepNode {
    FlowStepNode {
        node_id: "node_1".to_string(),
        name: "/homepage".to_string(),
        sessions: 1000,
        percentage: 100.0,
        connections: vec![sample_flow_step_connection()],
    }
}

fn sample_flow_step() -> FlowStep {
    FlowStep {
        step_number: 1,
        nodes: vec![sample_flow_step_node()],
        drop_off_count: 200,
        drop_off_rate: 20.0,
    }
}

fn sample_behavior_flow_data() -> BehaviorFlowData {
    BehaviorFlowData {
        date_range: sample_date_range(),
        starting_page: Some("/".to_string()),
        steps: vec![sample_flow_step()],
        total_sessions: 10000,
    }
}

fn sample_speed_trend_data() -> SpeedTrendData {
    SpeedTrendData {
        date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        avg_page_load_time: 2.5,
        avg_server_response_time: 0.3,
        page_load_sample: 5000,
    }
}

fn sample_browser_speed_data() -> BrowserSpeedData {
    BrowserSpeedData {
        browser: "Chrome".to_string(),
        avg_page_load_time: 2.2,
        avg_server_response_time: 0.25,
        page_load_sample: 30000,
        percentage: 65.0,
    }
}

fn sample_country_speed_data() -> CountrySpeedData {
    CountrySpeedData {
        country: "United States".to_string(),
        avg_page_load_time: 2.0,
        avg_server_response_time: 0.2,
        page_load_sample: 25000,
    }
}

fn sample_site_speed_overview() -> SiteSpeedOverview {
    SiteSpeedOverview {
        date_range: sample_date_range(),
        avg_page_load_time: 2.5,
        avg_redirection_time: 0.1,
        avg_domain_lookup_time: 0.05,
        avg_server_connection_time: 0.08,
        avg_server_response_time: 0.3,
        avg_page_download_time: 0.5,
        avg_document_interactive_time: 1.2,
        avg_document_content_loaded_time: 1.5,
        page_load_sample: 50000,
        speed_trend: vec![sample_speed_trend_data()],
        browser_breakdown: vec![sample_browser_speed_data()],
        country_breakdown: vec![sample_country_speed_data()],
    }
}

fn sample_page_timing_detail() -> PageTimingDetail {
    PageTimingDetail {
        page_path: "/products".to_string(),
        pageviews: 10000,
        avg_page_load_time: 2.8,
        avg_redirection_time: 0.1,
        avg_domain_lookup_time: 0.05,
        avg_server_connection_time: 0.1,
        avg_server_response_time: 0.35,
        avg_page_download_time: 0.6,
        page_load_sample: 8000,
    }
}

fn sample_dom_timing_data() -> DomTimingData {
    DomTimingData {
        page_path: "/products".to_string(),
        avg_document_interactive_time: 1.3,
        avg_document_content_loaded_time: 1.6,
        dom_interactive_sample: 7500,
    }
}

fn sample_speed_suggestion() -> SpeedSuggestion {
    SpeedSuggestion {
        category: SpeedSuggestionCategory::Images,
        title: "Optimize images".to_string(),
        description: "Compress and resize images to reduce page load time".to_string(),
        priority: SpeedSuggestionPriority::High,
        estimated_savings_ms: Some(500.0),
    }
}

fn sample_page_speed_suggestion() -> PageSpeedSuggestion {
    PageSpeedSuggestion {
        page_path: "/products".to_string(),
        pageviews: 10000,
        avg_page_load_time: 3.5,
        page_speed_score: Some(75),
        suggestions: vec![sample_speed_suggestion()],
    }
}

fn sample_search_trend_data() -> SearchTrendData {
    SearchTrendData {
        date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        total_unique_searches: 500,
        sessions_with_search: 450,
    }
}

fn sample_site_search_usage() -> SiteSearchUsage {
    SiteSearchUsage {
        sessions_with_search: 5000,
        total_unique_searches: 8000,
        results_pageviews: 15000,
        search_exits: 1000,
        search_refinements: 2000,
        time_after_search: 120.0,
        search_depth: 2.5,
        percentage_search_sessions: 10.0,
        search_trend: vec![sample_search_trend_data()],
    }
}

fn sample_search_term_data() -> SearchTermData {
    SearchTermData {
        search_term: "rust programming".to_string(),
        total_unique_searches: 500,
        results_pageviews: 1200,
        search_exits: 50,
        search_exit_rate: 10.0,
        search_refinements: 100,
        time_after_search: 150.0,
        search_depth: 3.0,
        percentage: 6.25,
    }
}

fn sample_search_page_data() -> SearchPageData {
    SearchPageData {
        page_path: "/search".to_string(),
        search_starts: 3000,
        search_exits: 300,
        search_refinements: 600,
        time_after_search: 130.0,
        search_depth: 2.8,
    }
}

fn sample_event_trend_data() -> EventTrendData {
    EventTrendData {
        date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        total_events: 10000,
        unique_events: 8000,
        event_value: 5000.0,
    }
}

fn sample_top_event_data() -> TopEventData {
    TopEventData {
        event_category: "Video".to_string(),
        total_events: 5000,
        unique_events: 4000,
        event_value: 2500.0,
        avg_value: 0.5,
        percentage: 25.0,
    }
}

fn sample_events_overview() -> EventsOverview {
    EventsOverview {
        date_range: sample_date_range(),
        total_events: 50000,
        unique_events: 40000,
        event_value: 25000.0,
        avg_value: 0.5,
        sessions_with_event: 30000,
        events_per_session: 1.67,
        event_trend: vec![sample_event_trend_data()],
        top_events: vec![sample_top_event_data()],
    }
}

fn sample_event_action_data() -> EventActionData {
    EventActionData {
        event_action: "play".to_string(),
        event_category: Some("Video".to_string()),
        total_events: 3000,
        unique_events: 2500,
        event_value: 1500.0,
        avg_value: 0.5,
        sessions_with_event: 2000,
    }
}

fn sample_event_category_data() -> EventCategoryData {
    EventCategoryData {
        event_category: "Video".to_string(),
        total_events: 5000,
        unique_events: 4000,
        event_value: 2500.0,
        avg_value: 0.5,
        sessions_with_event: 3000,
        actions: vec![sample_event_action_data()],
    }
}

fn sample_event_label_data() -> EventLabelData {
    EventLabelData {
        event_label: "intro-video".to_string(),
        event_category: "Video".to_string(),
        event_action: "play".to_string(),
        total_events: 1000,
        unique_events: 800,
        event_value: 500.0,
        avg_value: 0.5,
    }
}

fn sample_event_flow_data() -> EventFlowData {
    EventFlowData {
        event_sequence: vec!["Video:play".to_string(), "Video:pause".to_string(), "Video:complete".to_string()],
        occurrences: 500,
        conversion_rate: 25.0,
    }
}

fn sample_page_event_data() -> PageEventData {
    PageEventData {
        page_path: "/video/tutorial".to_string(),
        total_events: 2000,
        unique_events: 1600,
        event_value: 1000.0,
        top_events: vec![sample_top_event_data()],
    }
}

fn sample_ads_trend_data() -> AdsTrendData {
    AdsTrendData {
        date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        impressions: 100000,
        clicks: 500,
        revenue: 250.0,
    }
}

fn sample_publisher_page_data() -> PublisherPageData {
    PublisherPageData {
        page_path: "/blog/popular-post".to_string(),
        impressions: 50000,
        clicks: 250,
        ctr: 0.5,
        revenue: 125.0,
        revenue_per_thousand: 2.50,
    }
}

fn sample_publisher_overview() -> PublisherOverview {
    PublisherOverview {
        date_range: sample_date_range(),
        publisher_impressions: 1000000,
        publisher_clicks: 5000,
        publisher_ctr: 0.5,
        publisher_revenue: 2500.0,
        publisher_revenue_per_session: 0.05,
        publisher_impressions_per_session: 20.0,
        ads_trend: vec![sample_ads_trend_data()],
        top_pages: vec![sample_publisher_page_data()],
    }
}

fn sample_experiment_variation() -> ExperimentVariation {
    ExperimentVariation {
        variation_id: "var_001".to_string(),
        variation_name: "Control".to_string(),
        sessions: 5000,
        conversions: 250,
        conversion_rate: 5.0,
        improvement: 0.0,
        probability_to_beat_baseline: 50.0,
    }
}

fn sample_experiment_data() -> ExperimentData {
    ExperimentData {
        experiment_id: "exp_001".to_string(),
        experiment_name: "Homepage CTA Test".to_string(),
        status: ExperimentStatus::Running,
        start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        end_date: None,
        objective: "Increase signups".to_string(),
        variations: vec![sample_experiment_variation()],
        winner: None,
        statistical_significance: 85.0,
    }
}

fn sample_user_timing_category() -> UserTimingCategory {
    UserTimingCategory {
        timing_category: "API".to_string(),
        avg_user_timing: 250.0,
        user_timing_sample: 10000,
        percentage: 40.0,
    }
}

fn sample_user_timing_variable() -> UserTimingVariable {
    UserTimingVariable {
        timing_variable: "fetch".to_string(),
        timing_category: "API".to_string(),
        avg_user_timing: 200.0,
        user_timing_sample: 8000,
    }
}

fn sample_user_timing_label() -> UserTimingLabel {
    UserTimingLabel {
        timing_label: "user-data".to_string(),
        timing_variable: "fetch".to_string(),
        timing_category: "API".to_string(),
        avg_user_timing: 180.0,
        user_timing_sample: 5000,
    }
}

fn sample_exception_detail() -> ExceptionDetail {
    ExceptionDetail {
        exception_description: "TypeError: Cannot read property 'x' of undefined".to_string(),
        exceptions: 500,
        fatal_exceptions: 50,
        fatal_exceptions_percentage: 10.0,
    }
}

fn sample_exception_trend_data() -> ExceptionTrendData {
    ExceptionTrendData {
        date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        exceptions: 100,
        fatal_exceptions: 10,
    }
}

// ============================================================================
// BehaviorOverview Tests
// ============================================================================

#[test]
fn test_behavior_overview_creation() {
    let overview = sample_behavior_overview();
    assert_eq!(overview.pageviews, 150000);
    assert_eq!(overview.unique_pageviews, 120000);
    assert!(overview.comparison.is_some());
}

#[test]
fn test_behavior_overview_without_comparison() {
    let overview = BehaviorOverview {
        date_range: sample_date_range(),
        pageviews: 100000,
        unique_pageviews: 80000,
        avg_time_on_page: 120.0,
        bounce_rate: 50.0,
        exit_rate: 40.0,
        page_value: 2.0,
        pageviews_trend: vec![],
        comparison: None,
    };
    assert!(overview.comparison.is_none());
}

#[test]
fn test_behavior_overview_serialization() {
    let overview = sample_behavior_overview();
    let json = serde_json::to_string(&overview).unwrap();
    assert!(json.contains("\"pageviews\":150000"));
    assert!(json.contains("\"bounce_rate\":45.5"));
}

#[test]
fn test_behavior_overview_roundtrip() {
    let overview = sample_behavior_overview();
    let json = serde_json::to_string(&overview).unwrap();
    let deserialized: BehaviorOverview = serde_json::from_str(&json).unwrap();
    assert_eq!(overview.pageviews, deserialized.pageviews);
}

// ============================================================================
// PageviewsTrend Tests
// ============================================================================

#[test]
fn test_pageviews_trend_creation() {
    let trend = sample_pageviews_trend();
    assert_eq!(trend.pageviews, 5000);
    assert_eq!(trend.unique_pageviews, 4200);
}

#[test]
fn test_pageviews_trend_serialization() {
    let trend = sample_pageviews_trend();
    let json = serde_json::to_string(&trend).unwrap();
    assert!(json.contains("\"date\":\"2024-01-15\""));
    assert!(json.contains("\"pageviews\":5000"));
}

// ============================================================================
// BehaviorComparison Tests
// ============================================================================

#[test]
fn test_behavior_comparison_creation() {
    let comparison = sample_behavior_comparison();
    assert!((comparison.pageviews_change - 15.5).abs() < f64::EPSILON);
    assert!((comparison.bounce_rate_change - (-3.2)).abs() < f64::EPSILON);
}

#[test]
fn test_behavior_comparison_negative_values() {
    let comparison = BehaviorComparison {
        pageviews_change: -20.0,
        unique_pageviews_change: -15.0,
        avg_time_on_page_change: -10.0,
        bounce_rate_change: 5.0,
        exit_rate_change: 3.0,
    };
    assert!(comparison.pageviews_change < 0.0);
    assert!(comparison.bounce_rate_change > 0.0);
}

#[test]
fn test_behavior_comparison_serialization() {
    let comparison = sample_behavior_comparison();
    let json = serde_json::to_string(&comparison).unwrap();
    assert!(json.contains("pageviews_change"));
}

// ============================================================================
// SiteContentData Tests
// ============================================================================

#[test]
fn test_site_content_data_creation() {
    let data = SiteContentData {
        date_range: sample_date_range(),
        all_pages: vec![sample_page_content_data()],
        content_drilldown: vec![sample_content_drilldown_data()],
        landing_pages: vec![sample_landing_page_data()],
        exit_pages: vec![sample_exit_page_data()],
    };
    assert!(!data.all_pages.is_empty());
    assert!(!data.landing_pages.is_empty());
}

#[test]
fn test_site_content_data_serialization() {
    let data = SiteContentData {
        date_range: sample_date_range(),
        all_pages: vec![],
        content_drilldown: vec![],
        landing_pages: vec![],
        exit_pages: vec![],
    };
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"all_pages\":[]"));
}

// ============================================================================
// PageContentData Tests
// ============================================================================

#[test]
fn test_page_content_data_creation() {
    let data = sample_page_content_data();
    assert_eq!(data.page_path, "/blog/rust-tutorial");
    assert_eq!(data.pageviews, 5000);
}

#[test]
fn test_page_content_data_serialization() {
    let data = sample_page_content_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"page_path\":\"/blog/rust-tutorial\""));
    assert!(json.contains("\"page_title\":\"Rust Programming Tutorial\""));
}

// ============================================================================
// ContentDrilldownData Tests
// ============================================================================

#[test]
fn test_content_drilldown_data_creation() {
    let data = sample_content_drilldown_data();
    assert_eq!(data.page_path_level, "/blog");
    assert_eq!(data.level, 1);
}

#[test]
fn test_content_drilldown_data_with_children() {
    let child = ContentDrilldownData {
        page_path_level: "/blog/tutorials".to_string(),
        level: 2,
        pageviews: 20000,
        unique_pageviews: 17000,
        avg_time_on_page: 180.0,
        bounce_rate: 35.0,
        exit_rate: 25.0,
        page_value: 3.0,
        children: vec![],
    };

    let parent = ContentDrilldownData {
        page_path_level: "/blog".to_string(),
        level: 1,
        pageviews: 50000,
        unique_pageviews: 42000,
        avg_time_on_page: 150.0,
        bounce_rate: 40.0,
        exit_rate: 30.0,
        page_value: 2.75,
        children: vec![child],
    };

    assert_eq!(parent.children.len(), 1);
    assert_eq!(parent.children[0].level, 2);
}

#[test]
fn test_content_drilldown_data_serialization() {
    let data = sample_content_drilldown_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"page_path_level\":\"/blog\""));
    assert!(json.contains("\"level\":1"));
}

// ============================================================================
// LandingPageData Tests
// ============================================================================

#[test]
fn test_landing_page_data_creation() {
    let data = sample_landing_page_data();
    assert_eq!(data.landing_page, "/");
    assert_eq!(data.sessions, 25000);
}

#[test]
fn test_landing_page_data_serialization() {
    let data = sample_landing_page_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"landing_page\":\"/\""));
    assert!(json.contains("\"sessions\":25000"));
}

// ============================================================================
// ExitPageData Tests
// ============================================================================

#[test]
fn test_exit_page_data_creation() {
    let data = sample_exit_page_data();
    assert_eq!(data.exit_page, "/checkout/complete");
    assert_eq!(data.exits, 5000);
}

#[test]
fn test_exit_page_data_serialization() {
    let data = sample_exit_page_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"exit_page\":\"/checkout/complete\""));
}

// ============================================================================
// BehaviorFlowData Tests
// ============================================================================

#[test]
fn test_behavior_flow_data_creation() {
    let data = sample_behavior_flow_data();
    assert!(data.starting_page.is_some());
    assert_eq!(data.total_sessions, 10000);
}

#[test]
fn test_behavior_flow_data_without_starting_page() {
    let data = BehaviorFlowData {
        date_range: sample_date_range(),
        starting_page: None,
        steps: vec![],
        total_sessions: 5000,
    };
    assert!(data.starting_page.is_none());
}

#[test]
fn test_behavior_flow_data_serialization() {
    let data = sample_behavior_flow_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"total_sessions\":10000"));
}

// ============================================================================
// FlowStep Tests
// ============================================================================

#[test]
fn test_flow_step_creation() {
    let step = sample_flow_step();
    assert_eq!(step.step_number, 1);
    assert!(!step.nodes.is_empty());
}

#[test]
fn test_flow_step_serialization() {
    let step = sample_flow_step();
    let json = serde_json::to_string(&step).unwrap();
    assert!(json.contains("\"step_number\":1"));
    assert!(json.contains("\"drop_off_rate\":20"));
}

// ============================================================================
// FlowStepNode Tests
// ============================================================================

#[test]
fn test_flow_step_node_creation() {
    let node = sample_flow_step_node();
    assert_eq!(node.node_id, "node_1");
    assert_eq!(node.sessions, 1000);
}

#[test]
fn test_flow_step_node_serialization() {
    let node = sample_flow_step_node();
    let json = serde_json::to_string(&node).unwrap();
    assert!(json.contains("\"node_id\":\"node_1\""));
}

// ============================================================================
// FlowStepConnection Tests
// ============================================================================

#[test]
fn test_flow_step_connection_creation() {
    let connection = sample_flow_step_connection();
    assert_eq!(connection.to_node, "node_2");
    assert_eq!(connection.sessions, 800);
}

#[test]
fn test_flow_step_connection_serialization() {
    let connection = sample_flow_step_connection();
    let json = serde_json::to_string(&connection).unwrap();
    assert!(json.contains("\"to_node\":\"node_2\""));
}

// ============================================================================
// SiteSpeedOverview Tests
// ============================================================================

#[test]
fn test_site_speed_overview_creation() {
    let overview = sample_site_speed_overview();
    assert!((overview.avg_page_load_time - 2.5).abs() < f64::EPSILON);
    assert!(!overview.browser_breakdown.is_empty());
}

#[test]
fn test_site_speed_overview_serialization() {
    let overview = sample_site_speed_overview();
    let json = serde_json::to_string(&overview).unwrap();
    assert!(json.contains("\"avg_page_load_time\":2.5"));
    assert!(json.contains("\"page_load_sample\":50000"));
}

// ============================================================================
// SpeedTrendData Tests
// ============================================================================

#[test]
fn test_speed_trend_data_creation() {
    let trend = sample_speed_trend_data();
    assert!((trend.avg_page_load_time - 2.5).abs() < f64::EPSILON);
}

#[test]
fn test_speed_trend_data_serialization() {
    let trend = sample_speed_trend_data();
    let json = serde_json::to_string(&trend).unwrap();
    assert!(json.contains("\"date\":\"2024-01-15\""));
}

// ============================================================================
// BrowserSpeedData Tests
// ============================================================================

#[test]
fn test_browser_speed_data_creation() {
    let data = sample_browser_speed_data();
    assert_eq!(data.browser, "Chrome");
}

#[test]
fn test_browser_speed_data_various_browsers() {
    let browsers = vec!["Chrome", "Firefox", "Safari", "Edge", "Opera"];
    for browser_name in browsers {
        let data = BrowserSpeedData {
            browser: browser_name.to_string(),
            avg_page_load_time: 2.5,
            avg_server_response_time: 0.3,
            page_load_sample: 1000,
            percentage: 20.0,
        };
        assert_eq!(data.browser, browser_name);
    }
}

#[test]
fn test_browser_speed_data_serialization() {
    let data = sample_browser_speed_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"browser\":\"Chrome\""));
}

// ============================================================================
// CountrySpeedData Tests
// ============================================================================

#[test]
fn test_country_speed_data_creation() {
    let data = sample_country_speed_data();
    assert_eq!(data.country, "United States");
}

#[test]
fn test_country_speed_data_serialization() {
    let data = sample_country_speed_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"country\":\"United States\""));
}

// ============================================================================
// PageTimingsData Tests
// ============================================================================

#[test]
fn test_page_timings_data_creation() {
    let data = PageTimingsData {
        date_range: sample_date_range(),
        pages: vec![sample_page_timing_detail()],
        dom_timings: vec![sample_dom_timing_data()],
    };
    assert!(!data.pages.is_empty());
    assert!(!data.dom_timings.is_empty());
}

#[test]
fn test_page_timings_data_serialization() {
    let data = PageTimingsData {
        date_range: sample_date_range(),
        pages: vec![],
        dom_timings: vec![],
    };
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"pages\":[]"));
}

// ============================================================================
// PageTimingDetail Tests
// ============================================================================

#[test]
fn test_page_timing_detail_creation() {
    let detail = sample_page_timing_detail();
    assert_eq!(detail.page_path, "/products");
    assert_eq!(detail.pageviews, 10000);
}

#[test]
fn test_page_timing_detail_serialization() {
    let detail = sample_page_timing_detail();
    let json = serde_json::to_string(&detail).unwrap();
    assert!(json.contains("\"page_path\":\"/products\""));
}

// ============================================================================
// DomTimingData Tests
// ============================================================================

#[test]
fn test_dom_timing_data_creation() {
    let data = sample_dom_timing_data();
    assert_eq!(data.page_path, "/products");
}

#[test]
fn test_dom_timing_data_serialization() {
    let data = sample_dom_timing_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"avg_document_interactive_time\":1.3"));
}

// ============================================================================
// SpeedSuggestionCategory Enum Tests
// ============================================================================

#[test]
fn test_speed_suggestion_category_all_variants() {
    let categories = vec![
        SpeedSuggestionCategory::Rendering,
        SpeedSuggestionCategory::Resources,
        SpeedSuggestionCategory::Network,
        SpeedSuggestionCategory::ServerResponse,
        SpeedSuggestionCategory::Caching,
        SpeedSuggestionCategory::Images,
        SpeedSuggestionCategory::JavaScript,
        SpeedSuggestionCategory::Css,
        SpeedSuggestionCategory::Other,
    ];
    assert_eq!(categories.len(), 9);
}

#[test]
fn test_speed_suggestion_category_serialization() {
    assert_eq!(serde_json::to_string(&SpeedSuggestionCategory::Images).unwrap(), "\"images\"");
    assert_eq!(serde_json::to_string(&SpeedSuggestionCategory::JavaScript).unwrap(), "\"java_script\"");
    assert_eq!(serde_json::to_string(&SpeedSuggestionCategory::ServerResponse).unwrap(), "\"server_response\"");
}

#[test]
fn test_speed_suggestion_category_deserialization() {
    assert_eq!(serde_json::from_str::<SpeedSuggestionCategory>("\"images\"").unwrap(), SpeedSuggestionCategory::Images);
    assert_eq!(serde_json::from_str::<SpeedSuggestionCategory>("\"caching\"").unwrap(), SpeedSuggestionCategory::Caching);
}

#[test]
fn test_speed_suggestion_category_equality() {
    assert_eq!(SpeedSuggestionCategory::Images, SpeedSuggestionCategory::Images);
    assert_ne!(SpeedSuggestionCategory::Images, SpeedSuggestionCategory::Css);
}

// ============================================================================
// SpeedSuggestionPriority Enum Tests
// ============================================================================

#[test]
fn test_speed_suggestion_priority_all_variants() {
    let priorities = vec![
        SpeedSuggestionPriority::High,
        SpeedSuggestionPriority::Medium,
        SpeedSuggestionPriority::Low,
    ];
    assert_eq!(priorities.len(), 3);
}

#[test]
fn test_speed_suggestion_priority_serialization() {
    assert_eq!(serde_json::to_string(&SpeedSuggestionPriority::High).unwrap(), "\"high\"");
    assert_eq!(serde_json::to_string(&SpeedSuggestionPriority::Medium).unwrap(), "\"medium\"");
    assert_eq!(serde_json::to_string(&SpeedSuggestionPriority::Low).unwrap(), "\"low\"");
}

#[test]
fn test_speed_suggestion_priority_deserialization() {
    assert_eq!(serde_json::from_str::<SpeedSuggestionPriority>("\"high\"").unwrap(), SpeedSuggestionPriority::High);
}

// ============================================================================
// SpeedSuggestion Tests
// ============================================================================

#[test]
fn test_speed_suggestion_creation() {
    let suggestion = sample_speed_suggestion();
    assert_eq!(suggestion.category, SpeedSuggestionCategory::Images);
    assert_eq!(suggestion.priority, SpeedSuggestionPriority::High);
}

#[test]
fn test_speed_suggestion_without_savings() {
    let suggestion = SpeedSuggestion {
        category: SpeedSuggestionCategory::Other,
        title: "General optimization".to_string(),
        description: "Review code for optimization opportunities".to_string(),
        priority: SpeedSuggestionPriority::Low,
        estimated_savings_ms: None,
    };
    assert!(suggestion.estimated_savings_ms.is_none());
}

#[test]
fn test_speed_suggestion_serialization() {
    let suggestion = sample_speed_suggestion();
    let json = serde_json::to_string(&suggestion).unwrap();
    assert!(json.contains("\"category\":\"images\""));
    assert!(json.contains("\"priority\":\"high\""));
}

// ============================================================================
// PageSpeedSuggestion Tests
// ============================================================================

#[test]
fn test_page_speed_suggestion_creation() {
    let suggestion = sample_page_speed_suggestion();
    assert_eq!(suggestion.page_path, "/products");
    assert!(suggestion.page_speed_score.is_some());
}

#[test]
fn test_page_speed_suggestion_without_score() {
    let suggestion = PageSpeedSuggestion {
        page_path: "/new-page".to_string(),
        pageviews: 100,
        avg_page_load_time: 5.0,
        page_speed_score: None,
        suggestions: vec![],
    };
    assert!(suggestion.page_speed_score.is_none());
}

#[test]
fn test_page_speed_suggestion_serialization() {
    let suggestion = sample_page_speed_suggestion();
    let json = serde_json::to_string(&suggestion).unwrap();
    assert!(json.contains("\"page_speed_score\":75"));
}

// ============================================================================
// SpeedSuggestions Tests
// ============================================================================

#[test]
fn test_speed_suggestions_creation() {
    let suggestions = SpeedSuggestions {
        date_range: sample_date_range(),
        pages: vec![sample_page_speed_suggestion()],
    };
    assert!(!suggestions.pages.is_empty());
}

#[test]
fn test_speed_suggestions_serialization() {
    let suggestions = SpeedSuggestions {
        date_range: sample_date_range(),
        pages: vec![],
    };
    let json = serde_json::to_string(&suggestions).unwrap();
    assert!(json.contains("\"pages\":[]"));
}

// ============================================================================
// SiteSearchData Tests
// ============================================================================

#[test]
fn test_site_search_data_creation() {
    let data = SiteSearchData {
        date_range: sample_date_range(),
        usage: sample_site_search_usage(),
        terms: vec![sample_search_term_data()],
        pages: vec![sample_search_page_data()],
    };
    assert!(!data.terms.is_empty());
}

#[test]
fn test_site_search_data_serialization() {
    let data = SiteSearchData {
        date_range: sample_date_range(),
        usage: sample_site_search_usage(),
        terms: vec![],
        pages: vec![],
    };
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"usage\""));
}

// ============================================================================
// SiteSearchUsage Tests
// ============================================================================

#[test]
fn test_site_search_usage_creation() {
    let usage = sample_site_search_usage();
    assert_eq!(usage.sessions_with_search, 5000);
    assert_eq!(usage.total_unique_searches, 8000);
}

#[test]
fn test_site_search_usage_serialization() {
    let usage = sample_site_search_usage();
    let json = serde_json::to_string(&usage).unwrap();
    assert!(json.contains("\"sessions_with_search\":5000"));
}

// ============================================================================
// SearchTrendData Tests
// ============================================================================

#[test]
fn test_search_trend_data_creation() {
    let trend = sample_search_trend_data();
    assert_eq!(trend.total_unique_searches, 500);
}

#[test]
fn test_search_trend_data_serialization() {
    let trend = sample_search_trend_data();
    let json = serde_json::to_string(&trend).unwrap();
    assert!(json.contains("\"date\":\"2024-01-15\""));
}

// ============================================================================
// SearchTermData Tests
// ============================================================================

#[test]
fn test_search_term_data_creation() {
    let data = sample_search_term_data();
    assert_eq!(data.search_term, "rust programming");
    assert_eq!(data.total_unique_searches, 500);
}

#[test]
fn test_search_term_data_serialization() {
    let data = sample_search_term_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"search_term\":\"rust programming\""));
}

// ============================================================================
// SearchPageData Tests
// ============================================================================

#[test]
fn test_search_page_data_creation() {
    let data = sample_search_page_data();
    assert_eq!(data.page_path, "/search");
}

#[test]
fn test_search_page_data_serialization() {
    let data = sample_search_page_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"page_path\":\"/search\""));
}

// ============================================================================
// EventsOverview Tests
// ============================================================================

#[test]
fn test_events_overview_creation() {
    let overview = sample_events_overview();
    assert_eq!(overview.total_events, 50000);
    assert!(!overview.top_events.is_empty());
}

#[test]
fn test_events_overview_serialization() {
    let overview = sample_events_overview();
    let json = serde_json::to_string(&overview).unwrap();
    assert!(json.contains("\"total_events\":50000"));
}

// ============================================================================
// EventTrendData Tests
// ============================================================================

#[test]
fn test_event_trend_data_creation() {
    let trend = sample_event_trend_data();
    assert_eq!(trend.total_events, 10000);
}

#[test]
fn test_event_trend_data_serialization() {
    let trend = sample_event_trend_data();
    let json = serde_json::to_string(&trend).unwrap();
    assert!(json.contains("\"total_events\":10000"));
}

// ============================================================================
// TopEventData Tests
// ============================================================================

#[test]
fn test_top_event_data_creation() {
    let data = sample_top_event_data();
    assert_eq!(data.event_category, "Video");
}

#[test]
fn test_top_event_data_serialization() {
    let data = sample_top_event_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"event_category\":\"Video\""));
}

// ============================================================================
// EventDetailData Tests
// ============================================================================

#[test]
fn test_event_detail_data_creation() {
    let data = EventDetailData {
        date_range: sample_date_range(),
        events_by_category: vec![sample_event_category_data()],
        events_by_action: vec![sample_event_action_data()],
        events_by_label: vec![sample_event_label_data()],
        top_event_flows: vec![sample_event_flow_data()],
    };
    assert!(!data.events_by_category.is_empty());
}

#[test]
fn test_event_detail_data_serialization() {
    let data = EventDetailData {
        date_range: sample_date_range(),
        events_by_category: vec![],
        events_by_action: vec![],
        events_by_label: vec![],
        top_event_flows: vec![],
    };
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"events_by_category\":[]"));
}

// ============================================================================
// EventCategoryData Tests
// ============================================================================

#[test]
fn test_event_category_data_creation() {
    let data = sample_event_category_data();
    assert_eq!(data.event_category, "Video");
    assert!(!data.actions.is_empty());
}

#[test]
fn test_event_category_data_serialization() {
    let data = sample_event_category_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"event_category\":\"Video\""));
}

// ============================================================================
// EventActionData Tests
// ============================================================================

#[test]
fn test_event_action_data_creation() {
    let data = sample_event_action_data();
    assert_eq!(data.event_action, "play");
    assert!(data.event_category.is_some());
}

#[test]
fn test_event_action_data_without_category() {
    let data = EventActionData {
        event_action: "click".to_string(),
        event_category: None,
        total_events: 100,
        unique_events: 80,
        event_value: 50.0,
        avg_value: 0.5,
        sessions_with_event: 60,
    };
    assert!(data.event_category.is_none());
}

#[test]
fn test_event_action_data_serialization() {
    let data = sample_event_action_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"event_action\":\"play\""));
}

// ============================================================================
// EventLabelData Tests
// ============================================================================

#[test]
fn test_event_label_data_creation() {
    let data = sample_event_label_data();
    assert_eq!(data.event_label, "intro-video");
}

#[test]
fn test_event_label_data_serialization() {
    let data = sample_event_label_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"event_label\":\"intro-video\""));
}

// ============================================================================
// EventFlowData Tests
// ============================================================================

#[test]
fn test_event_flow_data_creation() {
    let data = sample_event_flow_data();
    assert_eq!(data.event_sequence.len(), 3);
    assert_eq!(data.occurrences, 500);
}

#[test]
fn test_event_flow_data_serialization() {
    let data = sample_event_flow_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"event_sequence\""));
    assert!(json.contains("Video:play"));
}

// ============================================================================
// EventsOnPagesData Tests
// ============================================================================

#[test]
fn test_events_on_pages_data_creation() {
    let data = EventsOnPagesData {
        date_range: sample_date_range(),
        pages: vec![sample_page_event_data()],
    };
    assert!(!data.pages.is_empty());
}

#[test]
fn test_events_on_pages_data_serialization() {
    let data = EventsOnPagesData {
        date_range: sample_date_range(),
        pages: vec![],
    };
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"pages\":[]"));
}

// ============================================================================
// PageEventData Tests
// ============================================================================

#[test]
fn test_page_event_data_creation() {
    let data = sample_page_event_data();
    assert_eq!(data.page_path, "/video/tutorial");
}

#[test]
fn test_page_event_data_serialization() {
    let data = sample_page_event_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"page_path\":\"/video/tutorial\""));
}

// ============================================================================
// PublisherOverview Tests
// ============================================================================

#[test]
fn test_publisher_overview_creation() {
    let overview = sample_publisher_overview();
    assert_eq!(overview.publisher_impressions, 1000000);
    assert!(!overview.top_pages.is_empty());
}

#[test]
fn test_publisher_overview_serialization() {
    let overview = sample_publisher_overview();
    let json = serde_json::to_string(&overview).unwrap();
    assert!(json.contains("\"publisher_impressions\":1000000"));
}

// ============================================================================
// AdsTrendData Tests
// ============================================================================

#[test]
fn test_ads_trend_data_creation() {
    let data = sample_ads_trend_data();
    assert_eq!(data.impressions, 100000);
}

#[test]
fn test_ads_trend_data_serialization() {
    let data = sample_ads_trend_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"impressions\":100000"));
}

// ============================================================================
// PublisherPageData Tests
// ============================================================================

#[test]
fn test_publisher_page_data_creation() {
    let data = sample_publisher_page_data();
    assert_eq!(data.page_path, "/blog/popular-post");
}

#[test]
fn test_publisher_page_data_serialization() {
    let data = sample_publisher_page_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"revenue_per_thousand\":2.5"));
}

// ============================================================================
// ExperimentStatus Enum Tests
// ============================================================================

#[test]
fn test_experiment_status_all_variants() {
    let statuses = vec![
        ExperimentStatus::Draft,
        ExperimentStatus::Running,
        ExperimentStatus::Paused,
        ExperimentStatus::Ended,
        ExperimentStatus::Archived,
    ];
    assert_eq!(statuses.len(), 5);
}

#[test]
fn test_experiment_status_serialization() {
    assert_eq!(serde_json::to_string(&ExperimentStatus::Draft).unwrap(), "\"draft\"");
    assert_eq!(serde_json::to_string(&ExperimentStatus::Running).unwrap(), "\"running\"");
    assert_eq!(serde_json::to_string(&ExperimentStatus::Paused).unwrap(), "\"paused\"");
    assert_eq!(serde_json::to_string(&ExperimentStatus::Ended).unwrap(), "\"ended\"");
    assert_eq!(serde_json::to_string(&ExperimentStatus::Archived).unwrap(), "\"archived\"");
}

#[test]
fn test_experiment_status_deserialization() {
    assert_eq!(serde_json::from_str::<ExperimentStatus>("\"running\"").unwrap(), ExperimentStatus::Running);
}

#[test]
fn test_experiment_status_equality() {
    assert_eq!(ExperimentStatus::Running, ExperimentStatus::Running);
    assert_ne!(ExperimentStatus::Running, ExperimentStatus::Paused);
}

// ============================================================================
// ExperimentVariation Tests
// ============================================================================

#[test]
fn test_experiment_variation_creation() {
    let variation = sample_experiment_variation();
    assert_eq!(variation.variation_id, "var_001");
    assert_eq!(variation.conversion_rate, 5.0);
}

#[test]
fn test_experiment_variation_serialization() {
    let variation = sample_experiment_variation();
    let json = serde_json::to_string(&variation).unwrap();
    assert!(json.contains("\"variation_id\":\"var_001\""));
}

// ============================================================================
// ExperimentData Tests
// ============================================================================

#[test]
fn test_experiment_data_creation() {
    let experiment = sample_experiment_data();
    assert_eq!(experiment.experiment_id, "exp_001");
    assert_eq!(experiment.status, ExperimentStatus::Running);
}

#[test]
fn test_experiment_data_with_winner() {
    let experiment = ExperimentData {
        experiment_id: "exp_002".to_string(),
        experiment_name: "Button Color Test".to_string(),
        status: ExperimentStatus::Ended,
        start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        end_date: Some(NaiveDate::from_ymd_opt(2024, 1, 31).unwrap()),
        objective: "Increase clicks".to_string(),
        variations: vec![sample_experiment_variation()],
        winner: Some("var_002".to_string()),
        statistical_significance: 95.0,
    };
    assert!(experiment.winner.is_some());
    assert!(experiment.end_date.is_some());
}

#[test]
fn test_experiment_data_serialization() {
    let experiment = sample_experiment_data();
    let json = serde_json::to_string(&experiment).unwrap();
    assert!(json.contains("\"experiment_id\":\"exp_001\""));
    assert!(json.contains("\"status\":\"running\""));
}

// ============================================================================
// ExperimentsOverview Tests
// ============================================================================

#[test]
fn test_experiments_overview_creation() {
    let overview = ExperimentsOverview {
        date_range: sample_date_range(),
        experiments: vec![sample_experiment_data()],
    };
    assert!(!overview.experiments.is_empty());
}

#[test]
fn test_experiments_overview_serialization() {
    let overview = ExperimentsOverview {
        date_range: sample_date_range(),
        experiments: vec![],
    };
    let json = serde_json::to_string(&overview).unwrap();
    assert!(json.contains("\"experiments\":[]"));
}

// ============================================================================
// UserTimingsData Tests
// ============================================================================

#[test]
fn test_user_timings_data_creation() {
    let data = UserTimingsData {
        date_range: sample_date_range(),
        categories: vec![sample_user_timing_category()],
        variables: vec![sample_user_timing_variable()],
        labels: vec![sample_user_timing_label()],
    };
    assert!(!data.categories.is_empty());
}

#[test]
fn test_user_timings_data_serialization() {
    let data = UserTimingsData {
        date_range: sample_date_range(),
        categories: vec![],
        variables: vec![],
        labels: vec![],
    };
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"categories\":[]"));
}

// ============================================================================
// UserTimingCategory Tests
// ============================================================================

#[test]
fn test_user_timing_category_creation() {
    let category = sample_user_timing_category();
    assert_eq!(category.timing_category, "API");
}

#[test]
fn test_user_timing_category_serialization() {
    let category = sample_user_timing_category();
    let json = serde_json::to_string(&category).unwrap();
    assert!(json.contains("\"timing_category\":\"API\""));
}

// ============================================================================
// UserTimingVariable Tests
// ============================================================================

#[test]
fn test_user_timing_variable_creation() {
    let variable = sample_user_timing_variable();
    assert_eq!(variable.timing_variable, "fetch");
}

#[test]
fn test_user_timing_variable_serialization() {
    let variable = sample_user_timing_variable();
    let json = serde_json::to_string(&variable).unwrap();
    assert!(json.contains("\"timing_variable\":\"fetch\""));
}

// ============================================================================
// UserTimingLabel Tests
// ============================================================================

#[test]
fn test_user_timing_label_creation() {
    let label = sample_user_timing_label();
    assert_eq!(label.timing_label, "user-data");
}

#[test]
fn test_user_timing_label_serialization() {
    let label = sample_user_timing_label();
    let json = serde_json::to_string(&label).unwrap();
    assert!(json.contains("\"timing_label\":\"user-data\""));
}

// ============================================================================
// ExceptionsData Tests
// ============================================================================

#[test]
fn test_exceptions_data_creation() {
    let data = ExceptionsData {
        date_range: sample_date_range(),
        exceptions: vec![sample_exception_detail()],
        exceptions_trend: vec![sample_exception_trend_data()],
    };
    assert!(!data.exceptions.is_empty());
}

#[test]
fn test_exceptions_data_serialization() {
    let data = ExceptionsData {
        date_range: sample_date_range(),
        exceptions: vec![],
        exceptions_trend: vec![],
    };
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"exceptions\":[]"));
}

// ============================================================================
// ExceptionDetail Tests
// ============================================================================

#[test]
fn test_exception_detail_creation() {
    let detail = sample_exception_detail();
    assert_eq!(detail.exceptions, 500);
    assert_eq!(detail.fatal_exceptions, 50);
}

#[test]
fn test_exception_detail_serialization() {
    let detail = sample_exception_detail();
    let json = serde_json::to_string(&detail).unwrap();
    assert!(json.contains("\"exception_description\""));
}

// ============================================================================
// ExceptionTrendData Tests
// ============================================================================

#[test]
fn test_exception_trend_data_creation() {
    let trend = sample_exception_trend_data();
    assert_eq!(trend.exceptions, 100);
    assert_eq!(trend.fatal_exceptions, 10);
}

#[test]
fn test_exception_trend_data_serialization() {
    let trend = sample_exception_trend_data();
    let json = serde_json::to_string(&trend).unwrap();
    assert!(json.contains("\"date\":\"2024-01-15\""));
}

// ============================================================================
// Edge Cases and Complex Scenarios
// ============================================================================

#[test]
fn test_unicode_in_page_paths() {
    let data = PageContentData {
        page_path: "/ブログ/記事".to_string(),
        page_title: "日本語のブログ記事".to_string(),
        pageviews: 1000,
        unique_pageviews: 800,
        avg_time_on_page: 120.0,
        entrances: 500,
        bounce_rate: 40.0,
        exit_rate: 30.0,
        page_value: 2.0,
        percentage: 1.0,
    };
    let json = serde_json::to_string(&data).unwrap();
    let deserialized: PageContentData = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.page_path, "/ブログ/記事");
}

#[test]
fn test_empty_string_fields() {
    let data = PageContentData {
        page_path: "".to_string(),
        page_title: "".to_string(),
        pageviews: 0,
        unique_pageviews: 0,
        avg_time_on_page: 0.0,
        entrances: 0,
        bounce_rate: 0.0,
        exit_rate: 0.0,
        page_value: 0.0,
        percentage: 0.0,
    };
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"page_path\":\"\""));
}

#[test]
fn test_large_values() {
    let overview = BehaviorOverview {
        date_range: sample_date_range(),
        pageviews: u64::MAX / 2,
        unique_pageviews: u64::MAX / 4,
        avg_time_on_page: 1000000.0,
        bounce_rate: 99.99,
        exit_rate: 99.99,
        page_value: 1000000.0,
        pageviews_trend: vec![],
        comparison: None,
    };
    let json = serde_json::to_string(&overview).unwrap();
    let deserialized: BehaviorOverview = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.pageviews, u64::MAX / 2);
}

#[test]
fn test_floating_point_precision() {
    let comparison = BehaviorComparison {
        pageviews_change: 33.333333333333336,
        unique_pageviews_change: 16.666666666666668,
        avg_time_on_page_change: -0.00000001,
        bounce_rate_change: 0.00000001,
        exit_rate_change: 99.999999999999,
    };
    let json = serde_json::to_string(&comparison).unwrap();
    let deserialized: BehaviorComparison = serde_json::from_str(&json).unwrap();
    assert!((deserialized.pageviews_change - 33.333333333333336).abs() < 1e-10);
}

#[test]
fn test_deeply_nested_content_drilldown() {
    let level3 = ContentDrilldownData {
        page_path_level: "/blog/rust/advanced".to_string(),
        level: 3,
        pageviews: 5000,
        unique_pageviews: 4000,
        avg_time_on_page: 200.0,
        bounce_rate: 30.0,
        exit_rate: 20.0,
        page_value: 4.0,
        children: vec![],
    };

    let level2 = ContentDrilldownData {
        page_path_level: "/blog/rust".to_string(),
        level: 2,
        pageviews: 20000,
        unique_pageviews: 16000,
        avg_time_on_page: 180.0,
        bounce_rate: 35.0,
        exit_rate: 25.0,
        page_value: 3.5,
        children: vec![level3],
    };

    let level1 = ContentDrilldownData {
        page_path_level: "/blog".to_string(),
        level: 1,
        pageviews: 50000,
        unique_pageviews: 40000,
        avg_time_on_page: 150.0,
        bounce_rate: 40.0,
        exit_rate: 30.0,
        page_value: 3.0,
        children: vec![level2],
    };

    assert_eq!(level1.children[0].children[0].level, 3);
}

#[test]
fn test_complex_flow_structure() {
    let step1 = FlowStep {
        step_number: 1,
        nodes: vec![
            FlowStepNode {
                node_id: "home".to_string(),
                name: "/".to_string(),
                sessions: 10000,
                percentage: 100.0,
                connections: vec![
                    FlowStepConnection { to_node: "products".to_string(), sessions: 4000, percentage: 40.0 },
                    FlowStepConnection { to_node: "blog".to_string(), sessions: 3000, percentage: 30.0 },
                ],
            },
        ],
        drop_off_count: 3000,
        drop_off_rate: 30.0,
    };

    let step2 = FlowStep {
        step_number: 2,
        nodes: vec![
            FlowStepNode {
                node_id: "products".to_string(),
                name: "/products".to_string(),
                sessions: 4000,
                percentage: 57.0,
                connections: vec![],
            },
            FlowStepNode {
                node_id: "blog".to_string(),
                name: "/blog".to_string(),
                sessions: 3000,
                percentage: 43.0,
                connections: vec![],
            },
        ],
        drop_off_count: 2000,
        drop_off_rate: 29.0,
    };

    let flow = BehaviorFlowData {
        date_range: sample_date_range(),
        starting_page: Some("/".to_string()),
        steps: vec![step1, step2],
        total_sessions: 10000,
    };

    assert_eq!(flow.steps.len(), 2);
    assert_eq!(flow.steps[0].nodes[0].connections.len(), 2);
}

#[test]
fn test_special_characters_in_search_terms() {
    let data = SearchTermData {
        search_term: "C++ programming guide".to_string(),
        total_unique_searches: 100,
        results_pageviews: 200,
        search_exits: 10,
        search_exit_rate: 10.0,
        search_refinements: 20,
        time_after_search: 120.0,
        search_depth: 2.5,
        percentage: 1.0,
    };
    let json = serde_json::to_string(&data).unwrap();
    let deserialized: SearchTermData = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.search_term, "C++ programming guide");
}

#[test]
fn test_clone_all_major_types() {
    let overview = sample_behavior_overview();
    let cloned = overview.clone();
    assert_eq!(overview.pageviews, cloned.pageviews);

    let experiment = sample_experiment_data();
    let cloned_exp = experiment.clone();
    assert_eq!(experiment.experiment_id, cloned_exp.experiment_id);

    let flow = sample_behavior_flow_data();
    let cloned_flow = flow.clone();
    assert_eq!(flow.total_sessions, cloned_flow.total_sessions);
}

#[test]
fn test_debug_trait_implementation() {
    let overview = sample_behavior_overview();
    let debug_str = format!("{:?}", overview);
    assert!(debug_str.contains("BehaviorOverview"));

    let status = ExperimentStatus::Running;
    let debug_str = format!("{:?}", status);
    assert!(debug_str.contains("Running"));
}

#[test]
fn test_many_events() {
    let events: Vec<TopEventData> = (0..100).map(|i| {
        TopEventData {
            event_category: format!("Category_{}", i),
            total_events: (100 - i) as u64 * 100,
            unique_events: (100 - i) as u64 * 80,
            event_value: (100 - i) as f64 * 50.0,
            avg_value: 0.5,
            percentage: 1.0,
        }
    }).collect();

    let overview = EventsOverview {
        date_range: sample_date_range(),
        total_events: 500000,
        unique_events: 400000,
        event_value: 250000.0,
        avg_value: 0.5,
        sessions_with_event: 300000,
        events_per_session: 1.67,
        event_trend: vec![],
        top_events: events,
    };

    assert_eq!(overview.top_events.len(), 100);
}

#[test]
fn test_zero_values() {
    let comparison = BehaviorComparison {
        pageviews_change: 0.0,
        unique_pageviews_change: 0.0,
        avg_time_on_page_change: 0.0,
        bounce_rate_change: 0.0,
        exit_rate_change: 0.0,
    };

    let json = serde_json::to_string(&comparison).unwrap();
    assert!(json.contains(":0"));
}
