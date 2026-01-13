//! Core analytics models

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// Date range for analytics queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

impl DateRange {
    pub fn new(start: NaiveDate, end: NaiveDate) -> Self {
        Self {
            start_date: start,
            end_date: end,
        }
    }

    pub fn today() -> Self {
        let today = Utc::now().date_naive();
        Self::new(today, today)
    }

    pub fn yesterday() -> Self {
        let yesterday = Utc::now().date_naive() - chrono::Duration::days(1);
        Self::new(yesterday, yesterday)
    }

    pub fn last_n_days(n: i64) -> Self {
        let today = Utc::now().date_naive();
        let start = today - chrono::Duration::days(n - 1);
        Self::new(start, today)
    }

    pub fn this_month() -> Self {
        let today = Utc::now().date_naive();
        let start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
        Self::new(start, today)
    }

    pub fn last_month() -> Self {
        let today = Utc::now().date_naive();
        let first_of_month = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
        let end = first_of_month - chrono::Duration::days(1);
        let start = NaiveDate::from_ymd_opt(end.year(), end.month(), 1).unwrap();
        Self::new(start, end)
    }
}

/// Analytics overview data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsOverview {
    pub date_range: DateRange,
    pub metrics: OverviewMetrics,
    pub comparison: Option<MetricsComparison>,
    pub chart_data: Vec<DailyMetrics>,
}

/// Core overview metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverviewMetrics {
    pub sessions: u64,
    pub users: u64,
    pub new_users: u64,
    pub pageviews: u64,
    pub pages_per_session: f64,
    pub avg_session_duration: f64,
    pub bounce_rate: f64,
    pub goal_conversion_rate: f64,
    pub goal_completions: u64,
    pub goal_value: f64,
    pub transactions: u64,
    pub revenue: f64,
    pub ecommerce_conversion_rate: f64,
}

impl Default for OverviewMetrics {
    fn default() -> Self {
        Self {
            sessions: 0,
            users: 0,
            new_users: 0,
            pageviews: 0,
            pages_per_session: 0.0,
            avg_session_duration: 0.0,
            bounce_rate: 0.0,
            goal_conversion_rate: 0.0,
            goal_completions: 0,
            goal_value: 0.0,
            transactions: 0,
            revenue: 0.0,
            ecommerce_conversion_rate: 0.0,
        }
    }
}

/// Metrics comparison between two periods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsComparison {
    pub sessions_change: f64,
    pub users_change: f64,
    pub new_users_change: f64,
    pub pageviews_change: f64,
    pub pages_per_session_change: f64,
    pub avg_session_duration_change: f64,
    pub bounce_rate_change: f64,
    pub goal_conversion_rate_change: f64,
    pub revenue_change: f64,
}

/// Daily metrics for chart data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyMetrics {
    pub date: NaiveDate,
    pub sessions: u64,
    pub users: u64,
    pub new_users: u64,
    pub pageviews: u64,
    pub bounce_rate: f64,
    pub avg_session_duration: f64,
    pub transactions: u64,
    pub revenue: f64,
}

/// Traffic source data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficSource {
    pub source: String,
    pub medium: String,
    pub sessions: u64,
    pub users: u64,
    pub new_users: u64,
    pub bounce_rate: f64,
    pub pages_per_session: f64,
    pub avg_session_duration: f64,
    pub goal_conversion_rate: f64,
    pub goal_completions: u64,
    pub revenue: f64,
}

/// Channel data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelData {
    pub channel: String,
    pub sessions: u64,
    pub users: u64,
    pub new_users: u64,
    pub bounce_rate: f64,
    pub pages_per_session: f64,
    pub avg_session_duration: f64,
    pub conversions: u64,
    pub conversion_rate: f64,
    pub revenue: f64,
    pub percentage: f64,
}

/// Page data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageData {
    pub page_path: String,
    pub page_title: String,
    pub pageviews: u64,
    pub unique_pageviews: u64,
    pub avg_time_on_page: f64,
    pub entrances: u64,
    pub bounce_rate: f64,
    pub exit_rate: f64,
    pub page_value: f64,
}

/// Referrer data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferrerData {
    pub referrer: String,
    pub sessions: u64,
    pub users: u64,
    pub new_users: u64,
    pub bounce_rate: f64,
    pub pages_per_session: f64,
    pub avg_session_duration: f64,
    pub percentage: f64,
}

/// Campaign data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignData {
    pub campaign: String,
    pub source: String,
    pub medium: String,
    pub sessions: u64,
    pub users: u64,
    pub new_users: u64,
    pub bounce_rate: f64,
    pub pages_per_session: f64,
    pub avg_session_duration: f64,
    pub conversions: u64,
    pub conversion_rate: f64,
    pub revenue: f64,
    pub cost: f64,
    pub roi: f64,
}

/// Keyword data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordData {
    pub keyword: String,
    pub sessions: u64,
    pub users: u64,
    pub bounce_rate: f64,
    pub pages_per_session: f64,
    pub avg_session_duration: f64,
    pub conversions: u64,
    pub conversion_rate: f64,
}

/// Event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    pub event_category: String,
    pub event_action: String,
    pub event_label: Option<String>,
    pub total_events: u64,
    pub unique_events: u64,
    pub event_value: f64,
    pub avg_value: f64,
    pub sessions_with_event: u64,
}

/// Site speed data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteSpeedData {
    pub avg_page_load_time: f64,
    pub avg_domain_lookup_time: f64,
    pub avg_server_connection_time: f64,
    pub avg_server_response_time: f64,
    pub avg_page_download_time: f64,
    pub avg_redirection_time: f64,
    pub avg_document_interactive_time: f64,
    pub avg_document_content_loaded_time: f64,
    pub page_load_sample: u64,
}

/// Page timing data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageTimingData {
    pub page_path: String,
    pub pageviews: u64,
    pub avg_page_load_time: f64,
    pub avg_server_response_time: f64,
    pub avg_page_download_time: f64,
    pub page_load_sample: u64,
}

/// Site search data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteSearchData {
    pub search_term: String,
    pub total_unique_searches: u64,
    pub results_pageviews: u64,
    pub search_exits: u64,
    pub search_exit_rate: f64,
    pub search_refinements: u64,
    pub search_depth: f64,
    pub avg_search_duration: f64,
}

/// Cached analytics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedAnalyticsData {
    pub id: uuid::Uuid,
    pub cache_key: String,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Dimension value with metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionValue {
    pub value: String,
    pub sessions: u64,
    pub users: u64,
    pub pageviews: u64,
    pub bounce_rate: f64,
    pub avg_session_duration: f64,
    pub percentage: f64,
}

/// Segment definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub id: String,
    pub name: String,
    pub definition: String,
    pub segment_type: SegmentType,
}

/// Segment type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SegmentType {
    BuiltIn,
    Custom,
    System,
}

/// Sampling info from GA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingInfo {
    pub is_sampled: bool,
    pub samples_read_counts: Option<i64>,
    pub sampling_space_sizes: Option<i64>,
    pub sampling_level: Option<SamplingLevel>,
}

/// Sampling level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SamplingLevel {
    Default,
    Small,
    Large,
}
