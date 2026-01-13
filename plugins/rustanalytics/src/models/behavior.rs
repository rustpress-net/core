//! Behavior analytics models

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::DateRange;

/// Behavior overview data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorOverview {
    pub date_range: DateRange,
    pub pageviews: u64,
    pub unique_pageviews: u64,
    pub avg_time_on_page: f64,
    pub bounce_rate: f64,
    pub exit_rate: f64,
    pub page_value: f64,
    pub pageviews_trend: Vec<PageviewsTrend>,
    pub comparison: Option<BehaviorComparison>,
}

/// Pageviews trend data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageviewsTrend {
    pub date: NaiveDate,
    pub pageviews: u64,
    pub unique_pageviews: u64,
    pub avg_time_on_page: f64,
}

/// Behavior comparison data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorComparison {
    pub pageviews_change: f64,
    pub unique_pageviews_change: f64,
    pub avg_time_on_page_change: f64,
    pub bounce_rate_change: f64,
    pub exit_rate_change: f64,
}

/// Site content data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteContentData {
    pub date_range: DateRange,
    pub all_pages: Vec<PageContentData>,
    pub content_drilldown: Vec<ContentDrilldownData>,
    pub landing_pages: Vec<LandingPageData>,
    pub exit_pages: Vec<ExitPageData>,
}

/// Page content data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageContentData {
    pub page_path: String,
    pub page_title: String,
    pub pageviews: u64,
    pub unique_pageviews: u64,
    pub avg_time_on_page: f64,
    pub entrances: u64,
    pub bounce_rate: f64,
    pub exit_rate: f64,
    pub page_value: f64,
    pub percentage: f64,
}

/// Content drilldown data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentDrilldownData {
    pub page_path_level: String,
    pub level: u32,
    pub pageviews: u64,
    pub unique_pageviews: u64,
    pub avg_time_on_page: f64,
    pub bounce_rate: f64,
    pub exit_rate: f64,
    pub page_value: f64,
    pub children: Vec<ContentDrilldownData>,
}

/// Landing page data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandingPageData {
    pub landing_page: String,
    pub sessions: u64,
    pub new_users_percentage: f64,
    pub new_sessions: u64,
    pub bounce_rate: f64,
    pub pages_per_session: f64,
    pub avg_session_duration: f64,
    pub goal_conversion_rate: f64,
    pub goal_completions: u64,
    pub goal_value: f64,
    pub transactions: u64,
    pub revenue: f64,
    pub percentage: f64,
}

/// Exit page data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitPageData {
    pub exit_page: String,
    pub exits: u64,
    pub pageviews: u64,
    pub exit_rate: f64,
    pub percentage: f64,
}

/// Behavior flow data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorFlowData {
    pub date_range: DateRange,
    pub starting_page: Option<String>,
    pub steps: Vec<FlowStep>,
    pub total_sessions: u64,
}

/// Flow step in behavior flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowStep {
    pub step_number: u32,
    pub nodes: Vec<FlowStepNode>,
    pub drop_off_count: u64,
    pub drop_off_rate: f64,
}

/// Node within a flow step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowStepNode {
    pub node_id: String,
    pub name: String,
    pub sessions: u64,
    pub percentage: f64,
    pub connections: Vec<FlowStepConnection>,
}

/// Connection between flow nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowStepConnection {
    pub to_node: String,
    pub sessions: u64,
    pub percentage: f64,
}

/// Site speed data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteSpeedOverview {
    pub date_range: DateRange,
    pub avg_page_load_time: f64,
    pub avg_redirection_time: f64,
    pub avg_domain_lookup_time: f64,
    pub avg_server_connection_time: f64,
    pub avg_server_response_time: f64,
    pub avg_page_download_time: f64,
    pub avg_document_interactive_time: f64,
    pub avg_document_content_loaded_time: f64,
    pub page_load_sample: u64,
    pub speed_trend: Vec<SpeedTrendData>,
    pub browser_breakdown: Vec<BrowserSpeedData>,
    pub country_breakdown: Vec<CountrySpeedData>,
}

/// Speed trend data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedTrendData {
    pub date: NaiveDate,
    pub avg_page_load_time: f64,
    pub avg_server_response_time: f64,
    pub page_load_sample: u64,
}

/// Browser speed data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserSpeedData {
    pub browser: String,
    pub avg_page_load_time: f64,
    pub avg_server_response_time: f64,
    pub page_load_sample: u64,
    pub percentage: f64,
}

/// Country speed data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountrySpeedData {
    pub country: String,
    pub avg_page_load_time: f64,
    pub avg_server_response_time: f64,
    pub page_load_sample: u64,
}

/// Page timings data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageTimingsData {
    pub date_range: DateRange,
    pub pages: Vec<PageTimingDetail>,
    pub dom_timings: Vec<DomTimingData>,
}

/// Page timing detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageTimingDetail {
    pub page_path: String,
    pub pageviews: u64,
    pub avg_page_load_time: f64,
    pub avg_redirection_time: f64,
    pub avg_domain_lookup_time: f64,
    pub avg_server_connection_time: f64,
    pub avg_server_response_time: f64,
    pub avg_page_download_time: f64,
    pub page_load_sample: u64,
}

/// DOM timing data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomTimingData {
    pub page_path: String,
    pub avg_document_interactive_time: f64,
    pub avg_document_content_loaded_time: f64,
    pub dom_interactive_sample: u64,
}

/// Speed suggestions data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedSuggestions {
    pub date_range: DateRange,
    pub pages: Vec<PageSpeedSuggestion>,
}

/// Page speed suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageSpeedSuggestion {
    pub page_path: String,
    pub pageviews: u64,
    pub avg_page_load_time: f64,
    pub page_speed_score: Option<u32>,
    pub suggestions: Vec<SpeedSuggestion>,
}

/// Individual speed suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedSuggestion {
    pub category: SpeedSuggestionCategory,
    pub title: String,
    pub description: String,
    pub priority: SpeedSuggestionPriority,
    pub estimated_savings_ms: Option<f64>,
}

/// Speed suggestion categories
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SpeedSuggestionCategory {
    Rendering,
    Resources,
    Network,
    ServerResponse,
    Caching,
    Images,
    JavaScript,
    Css,
    Other,
}

/// Speed suggestion priority
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SpeedSuggestionPriority {
    High,
    Medium,
    Low,
}

/// Site search data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteSearchData {
    pub date_range: DateRange,
    pub usage: SiteSearchUsage,
    pub terms: Vec<SearchTermData>,
    pub pages: Vec<SearchPageData>,
}

/// Site search usage overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteSearchUsage {
    pub sessions_with_search: u64,
    pub total_unique_searches: u64,
    pub results_pageviews: u64,
    pub search_exits: u64,
    pub search_refinements: u64,
    pub time_after_search: f64,
    pub search_depth: f64,
    pub percentage_search_sessions: f64,
    pub search_trend: Vec<SearchTrendData>,
}

/// Search trend data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchTrendData {
    pub date: NaiveDate,
    pub total_unique_searches: u64,
    pub sessions_with_search: u64,
}

/// Search term data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchTermData {
    pub search_term: String,
    pub total_unique_searches: u64,
    pub results_pageviews: u64,
    pub search_exits: u64,
    pub search_exit_rate: f64,
    pub search_refinements: u64,
    pub time_after_search: f64,
    pub search_depth: f64,
    pub percentage: f64,
}

/// Search page data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPageData {
    pub page_path: String,
    pub search_starts: u64,
    pub search_exits: u64,
    pub search_refinements: u64,
    pub time_after_search: f64,
    pub search_depth: f64,
}

/// Events overview data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventsOverview {
    pub date_range: DateRange,
    pub total_events: u64,
    pub unique_events: u64,
    pub event_value: f64,
    pub avg_value: f64,
    pub sessions_with_event: u64,
    pub events_per_session: f64,
    pub event_trend: Vec<EventTrendData>,
    pub top_events: Vec<TopEventData>,
}

/// Event trend data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTrendData {
    pub date: NaiveDate,
    pub total_events: u64,
    pub unique_events: u64,
    pub event_value: f64,
}

/// Top event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopEventData {
    pub event_category: String,
    pub total_events: u64,
    pub unique_events: u64,
    pub event_value: f64,
    pub avg_value: f64,
    pub percentage: f64,
}

/// Detailed event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDetailData {
    pub date_range: DateRange,
    pub events_by_category: Vec<EventCategoryData>,
    pub events_by_action: Vec<EventActionData>,
    pub events_by_label: Vec<EventLabelData>,
    pub top_event_flows: Vec<EventFlowData>,
}

/// Event category data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventCategoryData {
    pub event_category: String,
    pub total_events: u64,
    pub unique_events: u64,
    pub event_value: f64,
    pub avg_value: f64,
    pub sessions_with_event: u64,
    pub actions: Vec<EventActionData>,
}

/// Event action data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventActionData {
    pub event_action: String,
    pub event_category: Option<String>,
    pub total_events: u64,
    pub unique_events: u64,
    pub event_value: f64,
    pub avg_value: f64,
    pub sessions_with_event: u64,
}

/// Event label data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLabelData {
    pub event_label: String,
    pub event_category: String,
    pub event_action: String,
    pub total_events: u64,
    pub unique_events: u64,
    pub event_value: f64,
    pub avg_value: f64,
}

/// Event flow data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFlowData {
    pub event_sequence: Vec<String>,
    pub occurrences: u64,
    pub conversion_rate: f64,
}

/// Events on pages data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventsOnPagesData {
    pub date_range: DateRange,
    pub pages: Vec<PageEventData>,
}

/// Page event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageEventData {
    pub page_path: String,
    pub total_events: u64,
    pub unique_events: u64,
    pub event_value: f64,
    pub top_events: Vec<TopEventData>,
}

/// Publisher data (for AdSense/Ad Exchange)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublisherOverview {
    pub date_range: DateRange,
    pub publisher_impressions: u64,
    pub publisher_clicks: u64,
    pub publisher_ctr: f64,
    pub publisher_revenue: f64,
    pub publisher_revenue_per_session: f64,
    pub publisher_impressions_per_session: f64,
    pub ads_trend: Vec<AdsTrendData>,
    pub top_pages: Vec<PublisherPageData>,
}

/// Ads trend data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdsTrendData {
    pub date: NaiveDate,
    pub impressions: u64,
    pub clicks: u64,
    pub revenue: f64,
}

/// Publisher page data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublisherPageData {
    pub page_path: String,
    pub impressions: u64,
    pub clicks: u64,
    pub ctr: f64,
    pub revenue: f64,
    pub revenue_per_thousand: f64,
}

/// Content experiments data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentsOverview {
    pub date_range: DateRange,
    pub experiments: Vec<ExperimentData>,
}

/// Individual experiment data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentData {
    pub experiment_id: String,
    pub experiment_name: String,
    pub status: ExperimentStatus,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub objective: String,
    pub variations: Vec<ExperimentVariation>,
    pub winner: Option<String>,
    pub statistical_significance: f64,
}

/// Experiment status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExperimentStatus {
    Draft,
    Running,
    Paused,
    Ended,
    Archived,
}

/// Experiment variation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentVariation {
    pub variation_id: String,
    pub variation_name: String,
    pub sessions: u64,
    pub conversions: u64,
    pub conversion_rate: f64,
    pub improvement: f64,
    pub probability_to_beat_baseline: f64,
}

/// User timings data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTimingsData {
    pub date_range: DateRange,
    pub categories: Vec<UserTimingCategory>,
    pub variables: Vec<UserTimingVariable>,
    pub labels: Vec<UserTimingLabel>,
}

/// User timing category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTimingCategory {
    pub timing_category: String,
    pub avg_user_timing: f64,
    pub user_timing_sample: u64,
    pub percentage: f64,
}

/// User timing variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTimingVariable {
    pub timing_variable: String,
    pub timing_category: String,
    pub avg_user_timing: f64,
    pub user_timing_sample: u64,
}

/// User timing label
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTimingLabel {
    pub timing_label: String,
    pub timing_variable: String,
    pub timing_category: String,
    pub avg_user_timing: f64,
    pub user_timing_sample: u64,
}

/// Exceptions data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExceptionsData {
    pub date_range: DateRange,
    pub exceptions: Vec<ExceptionDetail>,
    pub exceptions_trend: Vec<ExceptionTrendData>,
}

/// Exception detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExceptionDetail {
    pub exception_description: String,
    pub exceptions: u64,
    pub fatal_exceptions: u64,
    pub fatal_exceptions_percentage: f64,
}

/// Exception trend data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExceptionTrendData {
    pub date: NaiveDate,
    pub exceptions: u64,
    pub fatal_exceptions: u64,
}
