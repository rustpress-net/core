//! Integration tests for Analytics Service
//!
//! These tests verify the analytics service's functionality including:
//! - Response processing and data transformation
//! - Metric parsing and calculations
//! - Date range handling
//! - Data aggregation and percentage calculations
//! - Edge cases and boundary conditions
//! - Unicode and special character handling

use chrono::{Datelike, NaiveDate};
use rustanalytics::models::api::*;
use rustanalytics::models::analytics::{
    DimensionValue as AnalyticsDimensionValue,
    Segment as AnalyticsSegment,
    SegmentType, SamplingInfo, SamplingLevel,
    SiteSearchData,
};
use rustanalytics::models::{
    AnalyticsOverview, ChannelData, DateRange, DailyMetrics,
    OverviewMetrics, PageData, ReferrerData, TrafficSource,
    MetricsComparison, CampaignData, KeywordData, EventData,
    SiteSpeedData, PageTimingData,
};
use serde_json::json;

// ============================================================================
// Test Fixtures
// ============================================================================

/// Create a sample overview report response
fn sample_overview_response() -> RunReportResponse {
    serde_json::from_value(json!({
        "dimensionHeaders": [
            { "name": "date" }
        ],
        "metricHeaders": [
            { "name": "sessions", "type": "TYPE_INTEGER" },
            { "name": "totalUsers", "type": "TYPE_INTEGER" },
            { "name": "newUsers", "type": "TYPE_INTEGER" },
            { "name": "screenPageViews", "type": "TYPE_INTEGER" },
            { "name": "screenPageViewsPerSession", "type": "TYPE_FLOAT" },
            { "name": "averageSessionDuration", "type": "TYPE_FLOAT" },
            { "name": "bounceRate", "type": "TYPE_FLOAT" },
            { "name": "conversions", "type": "TYPE_INTEGER" },
            { "name": "totalRevenue", "type": "TYPE_FLOAT" },
            { "name": "ecommercePurchases", "type": "TYPE_INTEGER" }
        ],
        "rows": [
            {
                "dimensionValues": [{ "value": "20231201" }],
                "metricValues": [
                    { "value": "100" },
                    { "value": "80" },
                    { "value": "50" },
                    { "value": "300" },
                    { "value": "3.0" },
                    { "value": "120.5" },
                    { "value": "0.45" },
                    { "value": "10" },
                    { "value": "500.00" },
                    { "value": "5" }
                ]
            },
            {
                "dimensionValues": [{ "value": "20231202" }],
                "metricValues": [
                    { "value": "150" },
                    { "value": "120" },
                    { "value": "75" },
                    { "value": "450" },
                    { "value": "3.0" },
                    { "value": "135.2" },
                    { "value": "0.40" },
                    { "value": "15" },
                    { "value": "750.00" },
                    { "value": "8" }
                ]
            },
            {
                "dimensionValues": [{ "value": "20231203" }],
                "metricValues": [
                    { "value": "200" },
                    { "value": "160" },
                    { "value": "100" },
                    { "value": "600" },
                    { "value": "3.0" },
                    { "value": "145.8" },
                    { "value": "0.35" },
                    { "value": "20" },
                    { "value": "1000.00" },
                    { "value": "10" }
                ]
            }
        ],
        "totals": [
            {
                "dimensionValues": [],
                "metricValues": [
                    { "value": "450" },
                    { "value": "360" },
                    { "value": "225" },
                    { "value": "1350" },
                    { "value": "3.0" },
                    { "value": "133.83" },
                    { "value": "0.40" },
                    { "value": "45" },
                    { "value": "2250.00" },
                    { "value": "23" }
                ]
            }
        ],
        "rowCount": 3
    })).unwrap()
}

/// Create a sample traffic sources response
fn sample_traffic_sources_response() -> RunReportResponse {
    serde_json::from_value(json!({
        "dimensionHeaders": [
            { "name": "sessionSource" },
            { "name": "sessionMedium" }
        ],
        "metricHeaders": [
            { "name": "sessions", "type": "TYPE_INTEGER" },
            { "name": "totalUsers", "type": "TYPE_INTEGER" },
            { "name": "newUsers", "type": "TYPE_INTEGER" },
            { "name": "bounceRate", "type": "TYPE_FLOAT" },
            { "name": "screenPageViewsPerSession", "type": "TYPE_FLOAT" },
            { "name": "averageSessionDuration", "type": "TYPE_FLOAT" },
            { "name": "conversions", "type": "TYPE_INTEGER" },
            { "name": "totalRevenue", "type": "TYPE_FLOAT" }
        ],
        "rows": [
            {
                "dimensionValues": [
                    { "value": "google" },
                    { "value": "organic" }
                ],
                "metricValues": [
                    { "value": "1000" },
                    { "value": "800" },
                    { "value": "500" },
                    { "value": "0.35" },
                    { "value": "3.5" },
                    { "value": "180.0" },
                    { "value": "50" },
                    { "value": "2500.00" }
                ]
            },
            {
                "dimensionValues": [
                    { "value": "(direct)" },
                    { "value": "(none)" }
                ],
                "metricValues": [
                    { "value": "500" },
                    { "value": "400" },
                    { "value": "200" },
                    { "value": "0.40" },
                    { "value": "2.8" },
                    { "value": "120.0" },
                    { "value": "25" },
                    { "value": "1000.00" }
                ]
            },
            {
                "dimensionValues": [
                    { "value": "facebook" },
                    { "value": "social" }
                ],
                "metricValues": [
                    { "value": "300" },
                    { "value": "250" },
                    { "value": "200" },
                    { "value": "0.50" },
                    { "value": "2.0" },
                    { "value": "90.0" },
                    { "value": "10" },
                    { "value": "500.00" }
                ]
            }
        ],
        "rowCount": 3
    })).unwrap()
}

/// Create a sample channels response
fn sample_channels_response() -> RunReportResponse {
    serde_json::from_value(json!({
        "dimensionHeaders": [
            { "name": "sessionDefaultChannelGroup" }
        ],
        "metricHeaders": [
            { "name": "sessions", "type": "TYPE_INTEGER" },
            { "name": "totalUsers", "type": "TYPE_INTEGER" },
            { "name": "newUsers", "type": "TYPE_INTEGER" },
            { "name": "bounceRate", "type": "TYPE_FLOAT" },
            { "name": "screenPageViewsPerSession", "type": "TYPE_FLOAT" },
            { "name": "averageSessionDuration", "type": "TYPE_FLOAT" },
            { "name": "conversions", "type": "TYPE_INTEGER" },
            { "name": "totalRevenue", "type": "TYPE_FLOAT" }
        ],
        "rows": [
            {
                "dimensionValues": [{ "value": "Organic Search" }],
                "metricValues": [
                    { "value": "5000" },
                    { "value": "4000" },
                    { "value": "2500" },
                    { "value": "0.35" },
                    { "value": "3.5" },
                    { "value": "180.0" },
                    { "value": "250" },
                    { "value": "12500.00" }
                ]
            },
            {
                "dimensionValues": [{ "value": "Direct" }],
                "metricValues": [
                    { "value": "3000" },
                    { "value": "2500" },
                    { "value": "1000" },
                    { "value": "0.40" },
                    { "value": "3.0" },
                    { "value": "150.0" },
                    { "value": "150" },
                    { "value": "7500.00" }
                ]
            },
            {
                "dimensionValues": [{ "value": "Social" }],
                "metricValues": [
                    { "value": "2000" },
                    { "value": "1800" },
                    { "value": "1500" },
                    { "value": "0.50" },
                    { "value": "2.0" },
                    { "value": "90.0" },
                    { "value": "50" },
                    { "value": "2500.00" }
                ]
            }
        ],
        "totals": [
            {
                "dimensionValues": [],
                "metricValues": [
                    { "value": "10000" },
                    { "value": "8300" },
                    { "value": "5000" },
                    { "value": "0.40" },
                    { "value": "3.0" },
                    { "value": "150.0" },
                    { "value": "450" },
                    { "value": "22500.00" }
                ]
            }
        ],
        "rowCount": 3
    })).unwrap()
}

/// Create a sample pages response
fn sample_pages_response() -> RunReportResponse {
    serde_json::from_value(json!({
        "dimensionHeaders": [
            { "name": "pagePath" },
            { "name": "pageTitle" }
        ],
        "metricHeaders": [
            { "name": "screenPageViews", "type": "TYPE_INTEGER" },
            { "name": "sessions", "type": "TYPE_INTEGER" },
            { "name": "averageSessionDuration", "type": "TYPE_FLOAT" },
            { "name": "bounceRate", "type": "TYPE_FLOAT" },
            { "name": "totalRevenue", "type": "TYPE_FLOAT" }
        ],
        "rows": [
            {
                "dimensionValues": [
                    { "value": "/" },
                    { "value": "Home Page" }
                ],
                "metricValues": [
                    { "value": "10000" },
                    { "value": "8000" },
                    { "value": "120.5" },
                    { "value": "0.30" },
                    { "value": "5000.00" }
                ]
            },
            {
                "dimensionValues": [
                    { "value": "/products" },
                    { "value": "Products" }
                ],
                "metricValues": [
                    { "value": "5000" },
                    { "value": "4000" },
                    { "value": "180.0" },
                    { "value": "0.25" },
                    { "value": "15000.00" }
                ]
            },
            {
                "dimensionValues": [
                    { "value": "/blog" },
                    { "value": "Blog" }
                ],
                "metricValues": [
                    { "value": "3000" },
                    { "value": "2500" },
                    { "value": "240.0" },
                    { "value": "0.35" },
                    { "value": "1000.00" }
                ]
            },
            {
                "dimensionValues": [
                    { "value": "/contact" },
                    { "value": "Contact Us" }
                ],
                "metricValues": [
                    { "value": "1500" },
                    { "value": "1200" },
                    { "value": "60.0" },
                    { "value": "0.45" },
                    { "value": "0.00" }
                ]
            }
        ],
        "rowCount": 4
    })).unwrap()
}

/// Create a sample referrers response
fn sample_referrers_response() -> RunReportResponse {
    serde_json::from_value(json!({
        "dimensionHeaders": [
            { "name": "sessionSource" }
        ],
        "metricHeaders": [
            { "name": "sessions", "type": "TYPE_INTEGER" },
            { "name": "totalUsers", "type": "TYPE_INTEGER" },
            { "name": "newUsers", "type": "TYPE_INTEGER" },
            { "name": "bounceRate", "type": "TYPE_FLOAT" },
            { "name": "screenPageViewsPerSession", "type": "TYPE_FLOAT" },
            { "name": "averageSessionDuration", "type": "TYPE_FLOAT" }
        ],
        "rows": [
            {
                "dimensionValues": [{ "value": "example.com" }],
                "metricValues": [
                    { "value": "500" },
                    { "value": "400" },
                    { "value": "350" },
                    { "value": "0.35" },
                    { "value": "3.0" },
                    { "value": "150.0" }
                ]
            },
            {
                "dimensionValues": [{ "value": "blog.example.org" }],
                "metricValues": [
                    { "value": "300" },
                    { "value": "250" },
                    { "value": "200" },
                    { "value": "0.40" },
                    { "value": "2.5" },
                    { "value": "120.0" }
                ]
            },
            {
                "dimensionValues": [{ "value": "news.site.com" }],
                "metricValues": [
                    { "value": "200" },
                    { "value": "180" },
                    { "value": "150" },
                    { "value": "0.50" },
                    { "value": "2.0" },
                    { "value": "90.0" }
                ]
            }
        ],
        "totals": [
            {
                "dimensionValues": [],
                "metricValues": [
                    { "value": "1000" },
                    { "value": "830" },
                    { "value": "700" },
                    { "value": "0.40" },
                    { "value": "2.5" },
                    { "value": "120.0" }
                ]
            }
        ],
        "rowCount": 3
    })).unwrap()
}

// ============================================================================
// Helper Functions for Response Processing
// ============================================================================

/// Parse a metric value as u64
fn parse_metric_value(value: &Option<&MetricValue>) -> u64 {
    value
        .and_then(|v| v.value.as_ref())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

/// Parse a metric value as f64
fn parse_metric_float(value: &Option<&MetricValue>) -> f64 {
    value
        .and_then(|v| v.value.as_ref())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0)
}

/// Process overview response into AnalyticsOverview
fn process_overview_response(
    response: RunReportResponse,
    date_range: DateRange,
) -> AnalyticsOverview {
    let mut chart_data = Vec::new();
    let mut metrics = OverviewMetrics::default();

    // Process totals
    if let Some(totals) = response.totals.as_ref().and_then(|t| t.first()) {
        if let Some(values) = &totals.metric_values {
            metrics = OverviewMetrics {
                sessions: parse_metric_value(&values.get(0)),
                users: parse_metric_value(&values.get(1)),
                new_users: parse_metric_value(&values.get(2)),
                pageviews: parse_metric_value(&values.get(3)),
                pages_per_session: parse_metric_float(&values.get(4)),
                avg_session_duration: parse_metric_float(&values.get(5)),
                bounce_rate: parse_metric_float(&values.get(6)) * 100.0,
                goal_conversion_rate: 0.0,
                goal_completions: parse_metric_value(&values.get(7)),
                goal_value: parse_metric_float(&values.get(8)),
                transactions: parse_metric_value(&values.get(9)),
                revenue: parse_metric_float(&values.get(8)),
                ecommerce_conversion_rate: 0.0,
            };

            // Calculate conversion rates
            if metrics.sessions > 0 {
                metrics.goal_conversion_rate =
                    (metrics.goal_completions as f64 / metrics.sessions as f64) * 100.0;
                metrics.ecommerce_conversion_rate =
                    (metrics.transactions as f64 / metrics.sessions as f64) * 100.0;
            }
        }
    }

    // Process daily data for chart
    if let Some(rows) = response.rows {
        for row in rows {
            if let (Some(dims), Some(vals)) = (&row.dimension_values, &row.metric_values) {
                if let Some(date_str) = dims.get(0).and_then(|d| d.value.as_ref()) {
                    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y%m%d") {
                        chart_data.push(DailyMetrics {
                            date,
                            sessions: parse_metric_value(&vals.get(0)),
                            users: parse_metric_value(&vals.get(1)),
                            new_users: parse_metric_value(&vals.get(2)),
                            pageviews: parse_metric_value(&vals.get(3)),
                            bounce_rate: parse_metric_float(&vals.get(6)) * 100.0,
                            avg_session_duration: parse_metric_float(&vals.get(5)),
                            transactions: parse_metric_value(&vals.get(9)),
                            revenue: parse_metric_float(&vals.get(8)),
                        });
                    }
                }
            }
        }
    }

    AnalyticsOverview {
        date_range,
        metrics,
        comparison: None,
        chart_data,
    }
}

/// Process traffic sources response
fn process_traffic_sources_response(response: RunReportResponse) -> Vec<TrafficSource> {
    let mut sources = Vec::new();

    if let Some(rows) = response.rows {
        for row in rows {
            if let (Some(dims), Some(vals)) = (&row.dimension_values, &row.metric_values) {
                let source = dims.get(0).and_then(|d| d.value.clone()).unwrap_or_default();
                let medium = dims.get(1).and_then(|d| d.value.clone()).unwrap_or_default();
                let sessions = parse_metric_value(&vals.get(0));

                sources.push(TrafficSource {
                    source,
                    medium,
                    sessions,
                    users: parse_metric_value(&vals.get(1)),
                    new_users: parse_metric_value(&vals.get(2)),
                    bounce_rate: parse_metric_float(&vals.get(3)) * 100.0,
                    pages_per_session: parse_metric_float(&vals.get(4)),
                    avg_session_duration: parse_metric_float(&vals.get(5)),
                    goal_conversion_rate: if sessions > 0 {
                        (parse_metric_value(&vals.get(6)) as f64 / sessions as f64) * 100.0
                    } else {
                        0.0
                    },
                    goal_completions: parse_metric_value(&vals.get(6)),
                    revenue: parse_metric_float(&vals.get(7)),
                });
            }
        }
    }

    sources
}

/// Process channels response
fn process_channels_response(response: RunReportResponse) -> Vec<ChannelData> {
    let mut channels = Vec::new();
    let mut total_sessions: u64 = 0;

    // First pass: collect all data and total sessions
    let mut raw_data = Vec::new();

    if let Some(rows) = response.rows {
        for row in rows {
            if let (Some(dims), Some(vals)) = (&row.dimension_values, &row.metric_values) {
                let channel = dims.get(0).and_then(|d| d.value.clone()).unwrap_or_default();
                let sessions = parse_metric_value(&vals.get(0));
                total_sessions += sessions;

                raw_data.push((channel, vals.clone()));
            }
        }
    }

    // Second pass: calculate percentages
    for (channel, vals) in raw_data {
        let sessions = parse_metric_value(&vals.get(0));

        channels.push(ChannelData {
            channel,
            sessions,
            users: parse_metric_value(&vals.get(1)),
            new_users: parse_metric_value(&vals.get(2)),
            bounce_rate: parse_metric_float(&vals.get(3)) * 100.0,
            pages_per_session: parse_metric_float(&vals.get(4)),
            avg_session_duration: parse_metric_float(&vals.get(5)),
            conversions: parse_metric_value(&vals.get(6)),
            conversion_rate: if sessions > 0 {
                (parse_metric_value(&vals.get(6)) as f64 / sessions as f64) * 100.0
            } else {
                0.0
            },
            revenue: parse_metric_float(&vals.get(7)),
            percentage: if total_sessions > 0 {
                (sessions as f64 / total_sessions as f64) * 100.0
            } else {
                0.0
            },
        });
    }

    channels
}

/// Process pages response
fn process_pages_response(response: RunReportResponse) -> Vec<PageData> {
    let mut pages = Vec::new();

    if let Some(rows) = response.rows {
        for row in rows {
            if let (Some(dims), Some(vals)) = (&row.dimension_values, &row.metric_values) {
                let page_path = dims.get(0).and_then(|d| d.value.clone()).unwrap_or_default();
                let page_title = dims.get(1).and_then(|d| d.value.clone()).unwrap_or_default();
                let pageviews = parse_metric_value(&vals.get(0));
                let sessions = parse_metric_value(&vals.get(1));

                pages.push(PageData {
                    page_path,
                    page_title,
                    pageviews,
                    unique_pageviews: sessions,
                    avg_time_on_page: parse_metric_float(&vals.get(2)),
                    entrances: sessions,
                    bounce_rate: parse_metric_float(&vals.get(3)) * 100.0,
                    exit_rate: 0.0,
                    page_value: parse_metric_float(&vals.get(4)),
                });
            }
        }
    }

    pages
}

/// Process referrers response
fn process_referrers_response(response: RunReportResponse) -> Vec<ReferrerData> {
    let mut referrers = Vec::new();
    let mut total_sessions: u64 = 0;

    // First pass: get total
    if let Some(totals) = response.totals.as_ref().and_then(|t| t.first()) {
        if let Some(values) = &totals.metric_values {
            total_sessions = parse_metric_value(&values.get(0));
        }
    }

    // Second pass: process rows
    if let Some(rows) = response.rows {
        for row in rows {
            if let (Some(dims), Some(vals)) = (&row.dimension_values, &row.metric_values) {
                let referrer = dims.get(0).and_then(|d| d.value.clone()).unwrap_or_default();
                let sessions = parse_metric_value(&vals.get(0));

                referrers.push(ReferrerData {
                    referrer,
                    sessions,
                    users: parse_metric_value(&vals.get(1)),
                    new_users: parse_metric_value(&vals.get(2)),
                    bounce_rate: parse_metric_float(&vals.get(3)) * 100.0,
                    pages_per_session: parse_metric_float(&vals.get(4)),
                    avg_session_duration: parse_metric_float(&vals.get(5)),
                    percentage: if total_sessions > 0 {
                        (sessions as f64 / total_sessions as f64) * 100.0
                    } else {
                        0.0
                    },
                });
            }
        }
    }

    referrers
}

// ============================================================================
// Overview Response Tests
// ============================================================================

#[test]
fn test_process_overview_response_totals() {
    let response = sample_overview_response();
    let date_range = DateRange::last_n_days(3);
    let overview = process_overview_response(response, date_range);

    // Verify total metrics
    assert_eq!(overview.metrics.sessions, 450);
    assert_eq!(overview.metrics.users, 360);
    assert_eq!(overview.metrics.new_users, 225);
    assert_eq!(overview.metrics.pageviews, 1350);
    assert!((overview.metrics.pages_per_session - 3.0).abs() < 0.01);
    assert!((overview.metrics.avg_session_duration - 133.83).abs() < 0.01);
    assert!((overview.metrics.bounce_rate - 40.0).abs() < 0.01); // 0.40 * 100
    assert_eq!(overview.metrics.goal_completions, 45);
    assert!((overview.metrics.goal_value - 2250.0).abs() < 0.01);
    assert_eq!(overview.metrics.transactions, 23);
}

#[test]
fn test_process_overview_response_conversion_rates() {
    let response = sample_overview_response();
    let date_range = DateRange::last_n_days(3);
    let overview = process_overview_response(response, date_range);

    // Goal conversion rate: 45 / 450 * 100 = 10%
    assert!((overview.metrics.goal_conversion_rate - 10.0).abs() < 0.01);

    // Ecommerce conversion rate: 23 / 450 * 100 = 5.11%
    assert!((overview.metrics.ecommerce_conversion_rate - 5.11).abs() < 0.1);
}

#[test]
fn test_process_overview_response_daily_data() {
    let response = sample_overview_response();
    let date_range = DateRange::last_n_days(3);
    let overview = process_overview_response(response, date_range);

    // Should have 3 days of data
    assert_eq!(overview.chart_data.len(), 3);

    // Verify first day
    let day1 = &overview.chart_data[0];
    assert_eq!(day1.date, NaiveDate::from_ymd_opt(2023, 12, 1).unwrap());
    assert_eq!(day1.sessions, 100);
    assert_eq!(day1.users, 80);
    assert_eq!(day1.pageviews, 300);

    // Verify second day
    let day2 = &overview.chart_data[1];
    assert_eq!(day2.date, NaiveDate::from_ymd_opt(2023, 12, 2).unwrap());
    assert_eq!(day2.sessions, 150);

    // Verify third day
    let day3 = &overview.chart_data[2];
    assert_eq!(day3.date, NaiveDate::from_ymd_opt(2023, 12, 3).unwrap());
    assert_eq!(day3.sessions, 200);
}

#[test]
fn test_process_overview_empty_response() {
    let response: RunReportResponse = serde_json::from_value(json!({
        "dimensionHeaders": [{ "name": "date" }],
        "metricHeaders": [],
        "rows": [],
        "totals": [],
        "rowCount": 0
    })).unwrap();

    let date_range = DateRange::last_n_days(7);
    let overview = process_overview_response(response, date_range);

    assert_eq!(overview.metrics.sessions, 0);
    assert_eq!(overview.metrics.users, 0);
    assert!(overview.chart_data.is_empty());
}

#[test]
fn test_process_overview_with_high_values() {
    let response: RunReportResponse = serde_json::from_value(json!({
        "dimensionHeaders": [{ "name": "date" }],
        "metricHeaders": [
            { "name": "sessions", "type": "TYPE_INTEGER" },
            { "name": "totalUsers", "type": "TYPE_INTEGER" },
            { "name": "newUsers", "type": "TYPE_INTEGER" },
            { "name": "screenPageViews", "type": "TYPE_INTEGER" },
            { "name": "screenPageViewsPerSession", "type": "TYPE_FLOAT" },
            { "name": "averageSessionDuration", "type": "TYPE_FLOAT" },
            { "name": "bounceRate", "type": "TYPE_FLOAT" },
            { "name": "conversions", "type": "TYPE_INTEGER" },
            { "name": "totalRevenue", "type": "TYPE_FLOAT" },
            { "name": "ecommercePurchases", "type": "TYPE_INTEGER" }
        ],
        "rows": [],
        "totals": [
            {
                "dimensionValues": [],
                "metricValues": [
                    { "value": "999999999" },
                    { "value": "888888888" },
                    { "value": "777777777" },
                    { "value": "666666666" },
                    { "value": "15.5" },
                    { "value": "3600.0" },
                    { "value": "0.15" },
                    { "value": "555555555" },
                    { "value": "9999999999.99" },
                    { "value": "444444444" }
                ]
            }
        ],
        "rowCount": 0
    })).unwrap();

    let date_range = DateRange::last_n_days(30);
    let overview = process_overview_response(response, date_range);

    assert_eq!(overview.metrics.sessions, 999999999);
    assert_eq!(overview.metrics.users, 888888888);
    assert!((overview.metrics.revenue - 9999999999.99).abs() < 0.01);
}

#[test]
fn test_process_overview_with_very_small_bounce_rate() {
    let response: RunReportResponse = serde_json::from_value(json!({
        "dimensionHeaders": [{ "name": "date" }],
        "metricHeaders": [
            { "name": "sessions", "type": "TYPE_INTEGER" },
            { "name": "totalUsers", "type": "TYPE_INTEGER" },
            { "name": "newUsers", "type": "TYPE_INTEGER" },
            { "name": "screenPageViews", "type": "TYPE_INTEGER" },
            { "name": "screenPageViewsPerSession", "type": "TYPE_FLOAT" },
            { "name": "averageSessionDuration", "type": "TYPE_FLOAT" },
            { "name": "bounceRate", "type": "TYPE_FLOAT" },
            { "name": "conversions", "type": "TYPE_INTEGER" },
            { "name": "totalRevenue", "type": "TYPE_FLOAT" },
            { "name": "ecommercePurchases", "type": "TYPE_INTEGER" }
        ],
        "rows": [],
        "totals": [
            {
                "dimensionValues": [],
                "metricValues": [
                    { "value": "1000" },
                    { "value": "800" },
                    { "value": "500" },
                    { "value": "3000" },
                    { "value": "3.0" },
                    { "value": "120.0" },
                    { "value": "0.001" },
                    { "value": "100" },
                    { "value": "5000.00" },
                    { "value": "50" }
                ]
            }
        ],
        "rowCount": 0
    })).unwrap();

    let date_range = DateRange::last_n_days(7);
    let overview = process_overview_response(response, date_range);

    // 0.001 * 100 = 0.1%
    assert!((overview.metrics.bounce_rate - 0.1).abs() < 0.01);
}

// ============================================================================
// Traffic Sources Tests
// ============================================================================

#[test]
fn test_process_traffic_sources_response() {
    let response = sample_traffic_sources_response();
    let sources = process_traffic_sources_response(response);

    assert_eq!(sources.len(), 3);

    // Verify first source (google / organic)
    let google = &sources[0];
    assert_eq!(google.source, "google");
    assert_eq!(google.medium, "organic");
    assert_eq!(google.sessions, 1000);
    assert_eq!(google.users, 800);
    assert_eq!(google.new_users, 500);
    assert!((google.bounce_rate - 35.0).abs() < 0.01);
    assert!((google.pages_per_session - 3.5).abs() < 0.01);
    assert!((google.avg_session_duration - 180.0).abs() < 0.01);
    assert_eq!(google.goal_completions, 50);
    assert!((google.revenue - 2500.0).abs() < 0.01);

    // Verify conversion rate: 50 / 1000 * 100 = 5%
    assert!((google.goal_conversion_rate - 5.0).abs() < 0.01);
}

#[test]
fn test_process_traffic_sources_direct() {
    let response = sample_traffic_sources_response();
    let sources = process_traffic_sources_response(response);

    // Verify direct traffic
    let direct = &sources[1];
    assert_eq!(direct.source, "(direct)");
    assert_eq!(direct.medium, "(none)");
    assert_eq!(direct.sessions, 500);
}

#[test]
fn test_process_traffic_sources_social() {
    let response = sample_traffic_sources_response();
    let sources = process_traffic_sources_response(response);

    // Verify social traffic
    let social = &sources[2];
    assert_eq!(social.source, "facebook");
    assert_eq!(social.medium, "social");
    assert_eq!(social.sessions, 300);
    assert!((social.bounce_rate - 50.0).abs() < 0.01);
}

#[test]
fn test_process_traffic_sources_empty() {
    let response: RunReportResponse = serde_json::from_value(json!({
        "dimensionHeaders": [
            { "name": "sessionSource" },
            { "name": "sessionMedium" }
        ],
        "metricHeaders": [],
        "rows": [],
        "rowCount": 0
    })).unwrap();

    let sources = process_traffic_sources_response(response);
    assert!(sources.is_empty());
}

#[test]
fn test_process_traffic_sources_with_unicode() {
    let response: RunReportResponse = serde_json::from_value(json!({
        "dimensionHeaders": [
            { "name": "sessionSource" },
            { "name": "sessionMedium" }
        ],
        "metricHeaders": [
            { "name": "sessions", "type": "TYPE_INTEGER" },
            { "name": "totalUsers", "type": "TYPE_INTEGER" },
            { "name": "newUsers", "type": "TYPE_INTEGER" },
            { "name": "bounceRate", "type": "TYPE_FLOAT" },
            { "name": "screenPageViewsPerSession", "type": "TYPE_FLOAT" },
            { "name": "averageSessionDuration", "type": "TYPE_FLOAT" },
            { "name": "conversions", "type": "TYPE_INTEGER" },
            { "name": "totalRevenue", "type": "TYPE_FLOAT" }
        ],
        "rows": [
            {
                "dimensionValues": [
                    { "value": "日本語サイト" },
                    { "value": "referral" }
                ],
                "metricValues": [
                    { "value": "100" },
                    { "value": "80" },
                    { "value": "60" },
                    { "value": "0.30" },
                    { "value": "4.0" },
                    { "value": "200.0" },
                    { "value": "10" },
                    { "value": "500.00" }
                ]
            },
            {
                "dimensionValues": [
                    { "value": "中文网站" },
                    { "value": "社交" }
                ],
                "metricValues": [
                    { "value": "50" },
                    { "value": "40" },
                    { "value": "30" },
                    { "value": "0.40" },
                    { "value": "3.0" },
                    { "value": "150.0" },
                    { "value": "5" },
                    { "value": "250.00" }
                ]
            }
        ],
        "rowCount": 2
    })).unwrap();

    let sources = process_traffic_sources_response(response);

    assert_eq!(sources.len(), 2);
    assert_eq!(sources[0].source, "日本語サイト");
    assert_eq!(sources[0].medium, "referral");
    assert_eq!(sources[1].source, "中文网站");
    assert_eq!(sources[1].medium, "社交");
}

#[test]
fn test_process_traffic_sources_with_special_characters() {
    let response: RunReportResponse = serde_json::from_value(json!({
        "dimensionHeaders": [
            { "name": "sessionSource" },
            { "name": "sessionMedium" }
        ],
        "metricHeaders": [
            { "name": "sessions", "type": "TYPE_INTEGER" },
            { "name": "totalUsers", "type": "TYPE_INTEGER" },
            { "name": "newUsers", "type": "TYPE_INTEGER" },
            { "name": "bounceRate", "type": "TYPE_FLOAT" },
            { "name": "screenPageViewsPerSession", "type": "TYPE_FLOAT" },
            { "name": "averageSessionDuration", "type": "TYPE_FLOAT" },
            { "name": "conversions", "type": "TYPE_INTEGER" },
            { "name": "totalRevenue", "type": "TYPE_FLOAT" }
        ],
        "rows": [
            {
                "dimensionValues": [
                    { "value": "example.com/path?query=1&foo=bar" },
                    { "value": "email/newsletter#campaign" }
                ],
                "metricValues": [
                    { "value": "100" },
                    { "value": "80" },
                    { "value": "60" },
                    { "value": "0.30" },
                    { "value": "4.0" },
                    { "value": "200.0" },
                    { "value": "10" },
                    { "value": "500.00" }
                ]
            }
        ],
        "rowCount": 1
    })).unwrap();

    let sources = process_traffic_sources_response(response);

    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].source, "example.com/path?query=1&foo=bar");
    assert_eq!(sources[0].medium, "email/newsletter#campaign");
}

// ============================================================================
// Channels Tests
// ============================================================================

#[test]
fn test_process_channels_response() {
    let response = sample_channels_response();
    let channels = process_channels_response(response);

    assert_eq!(channels.len(), 3);

    // Total sessions: 5000 + 3000 + 2000 = 10000
    let total_sessions: u64 = channels.iter().map(|c| c.sessions).sum();
    assert_eq!(total_sessions, 10000);
}

#[test]
fn test_process_channels_percentages() {
    let response = sample_channels_response();
    let channels = process_channels_response(response);

    // Organic Search: 5000 / 10000 * 100 = 50%
    let organic = &channels[0];
    assert_eq!(organic.channel, "Organic Search");
    assert!((organic.percentage - 50.0).abs() < 0.01);

    // Direct: 3000 / 10000 * 100 = 30%
    let direct = &channels[1];
    assert_eq!(direct.channel, "Direct");
    assert!((direct.percentage - 30.0).abs() < 0.01);

    // Social: 2000 / 10000 * 100 = 20%
    let social = &channels[2];
    assert_eq!(social.channel, "Social");
    assert!((social.percentage - 20.0).abs() < 0.01);
}

#[test]
fn test_process_channels_conversion_rates() {
    let response = sample_channels_response();
    let channels = process_channels_response(response);

    // Organic Search: 250 / 5000 * 100 = 5%
    let organic = &channels[0];
    assert!((organic.conversion_rate - 5.0).abs() < 0.01);

    // Direct: 150 / 3000 * 100 = 5%
    let direct = &channels[1];
    assert!((direct.conversion_rate - 5.0).abs() < 0.01);

    // Social: 50 / 2000 * 100 = 2.5%
    let social = &channels[2];
    assert!((social.conversion_rate - 2.5).abs() < 0.01);
}

#[test]
fn test_process_channels_metrics() {
    let response = sample_channels_response();
    let channels = process_channels_response(response);

    let organic = &channels[0];
    assert_eq!(organic.sessions, 5000);
    assert_eq!(organic.users, 4000);
    assert_eq!(organic.new_users, 2500);
    assert!((organic.bounce_rate - 35.0).abs() < 0.01);
    assert!((organic.pages_per_session - 3.5).abs() < 0.01);
    assert!((organic.avg_session_duration - 180.0).abs() < 0.01);
    assert_eq!(organic.conversions, 250);
    assert!((organic.revenue - 12500.0).abs() < 0.01);
}

#[test]
fn test_process_channels_many_channels() {
    let response: RunReportResponse = serde_json::from_value(json!({
        "dimensionHeaders": [{ "name": "sessionDefaultChannelGroup" }],
        "metricHeaders": [
            { "name": "sessions", "type": "TYPE_INTEGER" },
            { "name": "totalUsers", "type": "TYPE_INTEGER" },
            { "name": "newUsers", "type": "TYPE_INTEGER" },
            { "name": "bounceRate", "type": "TYPE_FLOAT" },
            { "name": "screenPageViewsPerSession", "type": "TYPE_FLOAT" },
            { "name": "averageSessionDuration", "type": "TYPE_FLOAT" },
            { "name": "conversions", "type": "TYPE_INTEGER" },
            { "name": "totalRevenue", "type": "TYPE_FLOAT" }
        ],
        "rows": [
            { "dimensionValues": [{ "value": "Organic Search" }], "metricValues": [{ "value": "1000" }, { "value": "800" }, { "value": "500" }, { "value": "0.30" }, { "value": "3.0" }, { "value": "120.0" }, { "value": "50" }, { "value": "2500.00" }] },
            { "dimensionValues": [{ "value": "Direct" }], "metricValues": [{ "value": "800" }, { "value": "600" }, { "value": "400" }, { "value": "0.35" }, { "value": "2.5" }, { "value": "100.0" }, { "value": "40" }, { "value": "2000.00" }] },
            { "dimensionValues": [{ "value": "Social" }], "metricValues": [{ "value": "600" }, { "value": "500" }, { "value": "400" }, { "value": "0.40" }, { "value": "2.0" }, { "value": "80.0" }, { "value": "20" }, { "value": "1000.00" }] },
            { "dimensionValues": [{ "value": "Email" }], "metricValues": [{ "value": "500" }, { "value": "400" }, { "value": "300" }, { "value": "0.25" }, { "value": "3.5" }, { "value": "150.0" }, { "value": "30" }, { "value": "1500.00" }] },
            { "dimensionValues": [{ "value": "Referral" }], "metricValues": [{ "value": "400" }, { "value": "350" }, { "value": "250" }, { "value": "0.35" }, { "value": "2.8" }, { "value": "110.0" }, { "value": "15" }, { "value": "750.00" }] },
            { "dimensionValues": [{ "value": "Paid Search" }], "metricValues": [{ "value": "350" }, { "value": "300" }, { "value": "200" }, { "value": "0.28" }, { "value": "3.2" }, { "value": "130.0" }, { "value": "25" }, { "value": "1250.00" }] },
            { "dimensionValues": [{ "value": "Display" }], "metricValues": [{ "value": "200" }, { "value": "180" }, { "value": "150" }, { "value": "0.45" }, { "value": "1.8" }, { "value": "60.0" }, { "value": "8" }, { "value": "400.00" }] },
            { "dimensionValues": [{ "value": "Affiliates" }], "metricValues": [{ "value": "100" }, { "value": "90" }, { "value": "80" }, { "value": "0.30" }, { "value": "2.5" }, { "value": "100.0" }, { "value": "10" }, { "value": "500.00" }] },
            { "dimensionValues": [{ "value": "Video" }], "metricValues": [{ "value": "50" }, { "value": "45" }, { "value": "40" }, { "value": "0.50" }, { "value": "1.5" }, { "value": "50.0" }, { "value": "2" }, { "value": "100.00" }] }
        ],
        "rowCount": 9
    })).unwrap();

    let channels = process_channels_response(response);

    assert_eq!(channels.len(), 9);

    // Total sessions: 1000 + 800 + 600 + 500 + 400 + 350 + 200 + 100 + 50 = 4000
    let total_sessions: u64 = channels.iter().map(|c| c.sessions).sum();
    assert_eq!(total_sessions, 4000);

    // Sum of all percentages should be 100%
    let total_percentage: f64 = channels.iter().map(|c| c.percentage).sum();
    assert!((total_percentage - 100.0).abs() < 0.01);
}

// ============================================================================
// Pages Tests
// ============================================================================

#[test]
fn test_process_pages_response() {
    let response = sample_pages_response();
    let pages = process_pages_response(response);

    assert_eq!(pages.len(), 4);

    // Verify home page
    let home = &pages[0];
    assert_eq!(home.page_path, "/");
    assert_eq!(home.page_title, "Home Page");
    assert_eq!(home.pageviews, 10000);
    assert_eq!(home.unique_pageviews, 8000);
    assert!((home.avg_time_on_page - 120.5).abs() < 0.01);
    assert!((home.bounce_rate - 30.0).abs() < 0.01);
    assert!((home.page_value - 5000.0).abs() < 0.01);
}

#[test]
fn test_process_pages_order() {
    let response = sample_pages_response();
    let pages = process_pages_response(response);

    // Pages should be ordered by pageviews (descending in original response)
    assert_eq!(pages[0].page_path, "/");
    assert_eq!(pages[0].pageviews, 10000);

    assert_eq!(pages[1].page_path, "/products");
    assert_eq!(pages[1].pageviews, 5000);

    assert_eq!(pages[2].page_path, "/blog");
    assert_eq!(pages[2].pageviews, 3000);

    assert_eq!(pages[3].page_path, "/contact");
    assert_eq!(pages[3].pageviews, 1500);
}

#[test]
fn test_process_pages_with_revenue() {
    let response = sample_pages_response();
    let pages = process_pages_response(response);

    // Products page should have highest revenue
    let products = &pages[1];
    assert_eq!(products.page_path, "/products");
    assert!((products.page_value - 15000.0).abs() < 0.01);

    // Contact page should have zero revenue
    let contact = &pages[3];
    assert_eq!(contact.page_path, "/contact");
    assert!((contact.page_value - 0.0).abs() < 0.01);
}

#[test]
fn test_process_pages_with_unicode_paths() {
    let response: RunReportResponse = serde_json::from_value(json!({
        "dimensionHeaders": [
            { "name": "pagePath" },
            { "name": "pageTitle" }
        ],
        "metricHeaders": [
            { "name": "screenPageViews", "type": "TYPE_INTEGER" },
            { "name": "sessions", "type": "TYPE_INTEGER" },
            { "name": "averageSessionDuration", "type": "TYPE_FLOAT" },
            { "name": "bounceRate", "type": "TYPE_FLOAT" },
            { "name": "totalRevenue", "type": "TYPE_FLOAT" }
        ],
        "rows": [
            {
                "dimensionValues": [
                    { "value": "/продукты" },
                    { "value": "Продукты - Русский" }
                ],
                "metricValues": [
                    { "value": "500" },
                    { "value": "400" },
                    { "value": "120.0" },
                    { "value": "0.30" },
                    { "value": "1000.00" }
                ]
            },
            {
                "dimensionValues": [
                    { "value": "/商品" },
                    { "value": "商品 - 中文" }
                ],
                "metricValues": [
                    { "value": "300" },
                    { "value": "250" },
                    { "value": "90.0" },
                    { "value": "0.35" },
                    { "value": "500.00" }
                ]
            }
        ],
        "rowCount": 2
    })).unwrap();

    let pages = process_pages_response(response);

    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0].page_path, "/продукты");
    assert_eq!(pages[0].page_title, "Продукты - Русский");
    assert_eq!(pages[1].page_path, "/商品");
    assert_eq!(pages[1].page_title, "商品 - 中文");
}

#[test]
fn test_process_pages_with_query_strings() {
    let response: RunReportResponse = serde_json::from_value(json!({
        "dimensionHeaders": [
            { "name": "pagePath" },
            { "name": "pageTitle" }
        ],
        "metricHeaders": [
            { "name": "screenPageViews", "type": "TYPE_INTEGER" },
            { "name": "sessions", "type": "TYPE_INTEGER" },
            { "name": "averageSessionDuration", "type": "TYPE_FLOAT" },
            { "name": "bounceRate", "type": "TYPE_FLOAT" },
            { "name": "totalRevenue", "type": "TYPE_FLOAT" }
        ],
        "rows": [
            {
                "dimensionValues": [
                    { "value": "/search?q=test&category=products&sort=price" },
                    { "value": "Search Results" }
                ],
                "metricValues": [
                    { "value": "1000" },
                    { "value": "800" },
                    { "value": "60.0" },
                    { "value": "0.25" },
                    { "value": "2000.00" }
                ]
            },
            {
                "dimensionValues": [
                    { "value": "/product/123?ref=campaign&utm_source=email" },
                    { "value": "Product Details" }
                ],
                "metricValues": [
                    { "value": "500" },
                    { "value": "400" },
                    { "value": "180.0" },
                    { "value": "0.20" },
                    { "value": "5000.00" }
                ]
            }
        ],
        "rowCount": 2
    })).unwrap();

    let pages = process_pages_response(response);

    assert_eq!(pages.len(), 2);
    assert!(pages[0].page_path.contains("?q=test"));
    assert!(pages[1].page_path.contains("utm_source=email"));
}

// ============================================================================
// Referrers Tests
// ============================================================================

#[test]
fn test_process_referrers_response() {
    let response = sample_referrers_response();
    let referrers = process_referrers_response(response);

    assert_eq!(referrers.len(), 3);

    // Verify first referrer
    let first = &referrers[0];
    assert_eq!(first.referrer, "example.com");
    assert_eq!(first.sessions, 500);
    assert_eq!(first.users, 400);
    assert_eq!(first.new_users, 350);
    assert!((first.bounce_rate - 35.0).abs() < 0.01);
    assert!((first.pages_per_session - 3.0).abs() < 0.01);
    assert!((first.avg_session_duration - 150.0).abs() < 0.01);
}

#[test]
fn test_process_referrers_percentages() {
    let response = sample_referrers_response();
    let referrers = process_referrers_response(response);

    // Total from totals row is 1000
    // example.com: 500 / 1000 * 100 = 50%
    assert!((referrers[0].percentage - 50.0).abs() < 0.01);

    // blog.example.org: 300 / 1000 * 100 = 30%
    assert!((referrers[1].percentage - 30.0).abs() < 0.01);

    // news.site.com: 200 / 1000 * 100 = 20%
    assert!((referrers[2].percentage - 20.0).abs() < 0.01);
}

#[test]
fn test_process_referrers_without_totals() {
    let response: RunReportResponse = serde_json::from_value(json!({
        "dimensionHeaders": [{ "name": "sessionSource" }],
        "metricHeaders": [
            { "name": "sessions", "type": "TYPE_INTEGER" },
            { "name": "totalUsers", "type": "TYPE_INTEGER" },
            { "name": "newUsers", "type": "TYPE_INTEGER" },
            { "name": "bounceRate", "type": "TYPE_FLOAT" },
            { "name": "screenPageViewsPerSession", "type": "TYPE_FLOAT" },
            { "name": "averageSessionDuration", "type": "TYPE_FLOAT" }
        ],
        "rows": [
            {
                "dimensionValues": [{ "value": "test.com" }],
                "metricValues": [
                    { "value": "100" },
                    { "value": "80" },
                    { "value": "60" },
                    { "value": "0.30" },
                    { "value": "3.0" },
                    { "value": "120.0" }
                ]
            }
        ],
        "rowCount": 1
    })).unwrap();

    let referrers = process_referrers_response(response);

    assert_eq!(referrers.len(), 1);
    // Without totals, percentage should be 0
    assert!((referrers[0].percentage - 0.0).abs() < 0.01);
}

// ============================================================================
// Metric Parsing Tests
// ============================================================================

#[test]
fn test_parse_metric_value_valid_integer() {
    let metric = MetricValue { value: Some("12345".to_string()), one_value: None };
    let result = parse_metric_value(&Some(&metric));
    assert_eq!(result, 12345);
}

#[test]
fn test_parse_metric_value_zero() {
    let metric = MetricValue { value: Some("0".to_string()), one_value: None };
    let result = parse_metric_value(&Some(&metric));
    assert_eq!(result, 0);
}

#[test]
fn test_parse_metric_value_none() {
    let result = parse_metric_value(&None);
    assert_eq!(result, 0);
}

#[test]
fn test_parse_metric_value_null_value() {
    let metric = MetricValue { value: None, one_value: None };
    let result = parse_metric_value(&Some(&metric));
    assert_eq!(result, 0);
}

#[test]
fn test_parse_metric_value_invalid_string() {
    let metric = MetricValue { value: Some("not_a_number".to_string()), one_value: None };
    let result = parse_metric_value(&Some(&metric));
    assert_eq!(result, 0);
}

#[test]
fn test_parse_metric_value_large_number() {
    let metric = MetricValue { value: Some("9999999999".to_string()), one_value: None };
    let result = parse_metric_value(&Some(&metric));
    assert_eq!(result, 9999999999);
}

#[test]
fn test_parse_metric_value_with_leading_zeros() {
    let metric = MetricValue { value: Some("00012345".to_string()), one_value: None };
    let result = parse_metric_value(&Some(&metric));
    assert_eq!(result, 12345);
}

#[test]
fn test_parse_metric_value_with_whitespace() {
    let metric = MetricValue { value: Some("  12345  ".to_string()), one_value: None };
    let result = parse_metric_value(&Some(&metric));
    // parse() should fail with whitespace
    assert_eq!(result, 0);
}

#[test]
fn test_parse_metric_value_empty_string() {
    let metric = MetricValue { value: Some("".to_string()), one_value: None };
    let result = parse_metric_value(&Some(&metric));
    assert_eq!(result, 0);
}

#[test]
fn test_parse_metric_value_max_u64() {
    let metric = MetricValue { value: Some("18446744073709551615".to_string()), one_value: None };
    let result = parse_metric_value(&Some(&metric));
    assert_eq!(result, u64::MAX);
}

#[test]
fn test_parse_metric_float_valid() {
    let metric = MetricValue { value: Some("123.456".to_string()), one_value: None };
    let result = parse_metric_float(&Some(&metric));
    assert!((result - 123.456).abs() < 0.001);
}

#[test]
fn test_parse_metric_float_zero() {
    let metric = MetricValue { value: Some("0.0".to_string()), one_value: None };
    let result = parse_metric_float(&Some(&metric));
    assert!((result - 0.0).abs() < 0.001);
}

#[test]
fn test_parse_metric_float_none() {
    let result = parse_metric_float(&None);
    assert!((result - 0.0).abs() < 0.001);
}

#[test]
fn test_parse_metric_float_integer_string() {
    let metric = MetricValue { value: Some("100".to_string()), one_value: None };
    let result = parse_metric_float(&Some(&metric));
    assert!((result - 100.0).abs() < 0.001);
}

#[test]
fn test_parse_metric_float_scientific_notation() {
    let metric = MetricValue { value: Some("1.5e3".to_string()), one_value: None };
    let result = parse_metric_float(&Some(&metric));
    assert!((result - 1500.0).abs() < 0.001);
}

#[test]
fn test_parse_metric_float_negative() {
    let metric = MetricValue { value: Some("-50.5".to_string()), one_value: None };
    let result = parse_metric_float(&Some(&metric));
    assert!((result - (-50.5)).abs() < 0.001);
}

#[test]
fn test_parse_metric_float_very_small() {
    let metric = MetricValue { value: Some("0.000001".to_string()), one_value: None };
    let result = parse_metric_float(&Some(&metric));
    assert!((result - 0.000001).abs() < 0.0000001);
}

#[test]
fn test_parse_metric_float_very_large() {
    let metric = MetricValue { value: Some("999999999999.99".to_string()), one_value: None };
    let result = parse_metric_float(&Some(&metric));
    assert!((result - 999999999999.99).abs() < 0.01);
}

#[test]
fn test_parse_metric_float_scientific_negative_exponent() {
    let metric = MetricValue { value: Some("1.5e-3".to_string()), one_value: None };
    let result = parse_metric_float(&Some(&metric));
    assert!((result - 0.0015).abs() < 0.0001);
}

#[test]
fn test_parse_metric_float_positive_infinity_string() {
    let metric = MetricValue { value: Some("inf".to_string()), one_value: None };
    let result = parse_metric_float(&Some(&metric));
    assert!(result.is_infinite() && result > 0.0);
}

#[test]
fn test_parse_metric_float_nan_string() {
    let metric = MetricValue { value: Some("NaN".to_string()), one_value: None };
    let result = parse_metric_float(&Some(&metric));
    // NaN parses as NaN in Rust
    assert!(result.is_nan() || result == 0.0);
}

// ============================================================================
// Date Range Tests
// ============================================================================

#[test]
fn test_date_range_in_overview() {
    let response = sample_overview_response();
    let date_range = DateRange::last_n_days(7);
    let overview = process_overview_response(response, date_range.clone());

    assert_eq!(overview.date_range.start_date, date_range.start_date);
    assert_eq!(overview.date_range.end_date, date_range.end_date);
}

#[test]
fn test_overview_with_different_date_ranges() {
    let response = sample_overview_response();

    // Test with today
    let today_range = DateRange::today();
    let overview_today = process_overview_response(response.clone(), today_range.clone());
    assert_eq!(overview_today.date_range.start_date, today_range.start_date);

    // Test with last 30 days
    let monthly_range = DateRange::last_n_days(30);
    let overview_monthly = process_overview_response(response, monthly_range.clone());
    assert_eq!(overview_monthly.date_range.start_date, monthly_range.start_date);
}

#[test]
fn test_date_range_today() {
    let range = DateRange::today();
    assert_eq!(range.start_date, range.end_date);
}

#[test]
fn test_date_range_yesterday() {
    let range = DateRange::yesterday();
    assert_eq!(range.start_date, range.end_date);
}

#[test]
fn test_date_range_last_7_days() {
    let range = DateRange::last_n_days(7);
    let diff = range.end_date - range.start_date;
    assert_eq!(diff.num_days(), 6); // 7 days inclusive means 6 day difference
}

#[test]
fn test_date_range_last_30_days() {
    let range = DateRange::last_n_days(30);
    let diff = range.end_date - range.start_date;
    assert_eq!(diff.num_days(), 29);
}

#[test]
fn test_date_range_this_month() {
    let range = DateRange::this_month();
    assert_eq!(range.start_date.day(), 1);
}

#[test]
fn test_date_range_last_month() {
    let range = DateRange::last_month();
    assert_eq!(range.start_date.day(), 1);
    // Last day should be the last day of that month
}

#[test]
fn test_date_range_custom() {
    let start = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2023, 12, 31).unwrap();
    let range = DateRange::new(start, end);

    assert_eq!(range.start_date, start);
    assert_eq!(range.end_date, end);
}

#[test]
fn test_date_range_serialization() {
    let range = DateRange::last_n_days(7);
    let json = serde_json::to_string(&range).unwrap();
    let deserialized: DateRange = serde_json::from_str(&json).unwrap();

    assert_eq!(range.start_date, deserialized.start_date);
    assert_eq!(range.end_date, deserialized.end_date);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_process_response_with_missing_dimensions() {
    let response: RunReportResponse = serde_json::from_value(json!({
        "dimensionHeaders": [{ "name": "date" }],
        "metricHeaders": [{ "name": "sessions", "type": "TYPE_INTEGER" }],
        "rows": [
            {
                "dimensionValues": [],
                "metricValues": [{ "value": "100" }]
            }
        ],
        "rowCount": 1
    })).unwrap();

    let date_range = DateRange::today();
    let overview = process_overview_response(response, date_range);

    // Should handle missing dimension values gracefully
    assert!(overview.chart_data.is_empty());
}

#[test]
fn test_process_response_with_null_metric_values() {
    let response: RunReportResponse = serde_json::from_value(json!({
        "dimensionHeaders": [{ "name": "sessionSource" }],
        "metricHeaders": [
            { "name": "sessions", "type": "TYPE_INTEGER" },
            { "name": "totalUsers", "type": "TYPE_INTEGER" }
        ],
        "rows": [
            {
                "dimensionValues": [{ "value": "google" }],
                "metricValues": [
                    { "value": null },
                    { "value": "100" }
                ]
            }
        ],
        "rowCount": 1
    })).unwrap();

    let sources = process_traffic_sources_response(response);

    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].sessions, 0); // null should parse as 0
}

#[test]
fn test_zero_sessions_no_division_by_zero() {
    let response: RunReportResponse = serde_json::from_value(json!({
        "dimensionHeaders": [
            { "name": "sessionSource" },
            { "name": "sessionMedium" }
        ],
        "metricHeaders": [
            { "name": "sessions", "type": "TYPE_INTEGER" },
            { "name": "totalUsers", "type": "TYPE_INTEGER" },
            { "name": "newUsers", "type": "TYPE_INTEGER" },
            { "name": "bounceRate", "type": "TYPE_FLOAT" },
            { "name": "screenPageViewsPerSession", "type": "TYPE_FLOAT" },
            { "name": "averageSessionDuration", "type": "TYPE_FLOAT" },
            { "name": "conversions", "type": "TYPE_INTEGER" },
            { "name": "totalRevenue", "type": "TYPE_FLOAT" }
        ],
        "rows": [
            {
                "dimensionValues": [
                    { "value": "test" },
                    { "value": "test" }
                ],
                "metricValues": [
                    { "value": "0" },
                    { "value": "0" },
                    { "value": "0" },
                    { "value": "0" },
                    { "value": "0" },
                    { "value": "0" },
                    { "value": "5" },
                    { "value": "100.00" }
                ]
            }
        ],
        "rowCount": 1
    })).unwrap();

    let sources = process_traffic_sources_response(response);

    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].sessions, 0);
    // Conversion rate should be 0, not NaN or Infinity
    assert!((sources[0].goal_conversion_rate - 0.0).abs() < 0.001);
    assert!(!sources[0].goal_conversion_rate.is_nan());
    assert!(!sources[0].goal_conversion_rate.is_infinite());
}

#[test]
fn test_channels_zero_total_sessions_no_division_by_zero() {
    let response: RunReportResponse = serde_json::from_value(json!({
        "dimensionHeaders": [{ "name": "sessionDefaultChannelGroup" }],
        "metricHeaders": [
            { "name": "sessions", "type": "TYPE_INTEGER" },
            { "name": "totalUsers", "type": "TYPE_INTEGER" },
            { "name": "newUsers", "type": "TYPE_INTEGER" },
            { "name": "bounceRate", "type": "TYPE_FLOAT" },
            { "name": "screenPageViewsPerSession", "type": "TYPE_FLOAT" },
            { "name": "averageSessionDuration", "type": "TYPE_FLOAT" },
            { "name": "conversions", "type": "TYPE_INTEGER" },
            { "name": "totalRevenue", "type": "TYPE_FLOAT" }
        ],
        "rows": [
            {
                "dimensionValues": [{ "value": "Test Channel" }],
                "metricValues": [
                    { "value": "0" },
                    { "value": "0" },
                    { "value": "0" },
                    { "value": "0" },
                    { "value": "0" },
                    { "value": "0" },
                    { "value": "0" },
                    { "value": "0" }
                ]
            }
        ],
        "rowCount": 1
    })).unwrap();

    let channels = process_channels_response(response);

    assert_eq!(channels.len(), 1);
    assert_eq!(channels[0].sessions, 0);
    assert!((channels[0].percentage - 0.0).abs() < 0.001);
    assert!(!channels[0].percentage.is_nan());
    assert!((channels[0].conversion_rate - 0.0).abs() < 0.001);
    assert!(!channels[0].conversion_rate.is_nan());
}

#[test]
fn test_invalid_date_format_in_response() {
    let response: RunReportResponse = serde_json::from_value(json!({
        "dimensionHeaders": [{ "name": "date" }],
        "metricHeaders": [
            { "name": "sessions", "type": "TYPE_INTEGER" }
        ],
        "rows": [
            {
                "dimensionValues": [{ "value": "invalid-date" }],
                "metricValues": [{ "value": "100" }]
            },
            {
                "dimensionValues": [{ "value": "2023-12-01" }],
                "metricValues": [{ "value": "200" }]
            },
            {
                "dimensionValues": [{ "value": "20231203" }],
                "metricValues": [{ "value": "300" }]
            }
        ],
        "rowCount": 3
    })).unwrap();

    let date_range = DateRange::last_n_days(7);
    let overview = process_overview_response(response, date_range);

    // Only the valid date format (YYYYMMDD) should be parsed
    assert_eq!(overview.chart_data.len(), 1);
    assert_eq!(overview.chart_data[0].sessions, 300);
}

#[test]
fn test_empty_source_medium_values() {
    let response: RunReportResponse = serde_json::from_value(json!({
        "dimensionHeaders": [
            { "name": "sessionSource" },
            { "name": "sessionMedium" }
        ],
        "metricHeaders": [
            { "name": "sessions", "type": "TYPE_INTEGER" },
            { "name": "totalUsers", "type": "TYPE_INTEGER" },
            { "name": "newUsers", "type": "TYPE_INTEGER" },
            { "name": "bounceRate", "type": "TYPE_FLOAT" },
            { "name": "screenPageViewsPerSession", "type": "TYPE_FLOAT" },
            { "name": "averageSessionDuration", "type": "TYPE_FLOAT" },
            { "name": "conversions", "type": "TYPE_INTEGER" },
            { "name": "totalRevenue", "type": "TYPE_FLOAT" }
        ],
        "rows": [
            {
                "dimensionValues": [
                    { "value": "" },
                    { "value": "" }
                ],
                "metricValues": [
                    { "value": "100" },
                    { "value": "80" },
                    { "value": "50" },
                    { "value": "0.30" },
                    { "value": "3.0" },
                    { "value": "120.0" },
                    { "value": "10" },
                    { "value": "500.00" }
                ]
            }
        ],
        "rowCount": 1
    })).unwrap();

    let sources = process_traffic_sources_response(response);

    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].source, "");
    assert_eq!(sources[0].medium, "");
    assert_eq!(sources[0].sessions, 100);
}

// ============================================================================
// Model Serialization Tests
// ============================================================================

#[test]
fn test_analytics_overview_serialization() {
    let overview = AnalyticsOverview {
        date_range: DateRange::last_n_days(7),
        metrics: OverviewMetrics::default(),
        comparison: None,
        chart_data: vec![],
    };

    let json = serde_json::to_string(&overview).unwrap();
    let deserialized: AnalyticsOverview = serde_json::from_str(&json).unwrap();

    assert_eq!(overview.date_range.start_date, deserialized.date_range.start_date);
    assert_eq!(overview.metrics.sessions, deserialized.metrics.sessions);
}

#[test]
fn test_overview_metrics_default() {
    let metrics = OverviewMetrics::default();

    assert_eq!(metrics.sessions, 0);
    assert_eq!(metrics.users, 0);
    assert_eq!(metrics.new_users, 0);
    assert_eq!(metrics.pageviews, 0);
    assert!((metrics.pages_per_session - 0.0).abs() < 0.001);
    assert!((metrics.avg_session_duration - 0.0).abs() < 0.001);
    assert!((metrics.bounce_rate - 0.0).abs() < 0.001);
    assert!((metrics.goal_conversion_rate - 0.0).abs() < 0.001);
    assert_eq!(metrics.goal_completions, 0);
    assert!((metrics.goal_value - 0.0).abs() < 0.001);
    assert_eq!(metrics.transactions, 0);
    assert!((metrics.revenue - 0.0).abs() < 0.001);
    assert!((metrics.ecommerce_conversion_rate - 0.0).abs() < 0.001);
}

#[test]
fn test_traffic_source_serialization() {
    let source = TrafficSource {
        source: "google".to_string(),
        medium: "organic".to_string(),
        sessions: 1000,
        users: 800,
        new_users: 500,
        bounce_rate: 35.0,
        pages_per_session: 3.5,
        avg_session_duration: 180.0,
        goal_conversion_rate: 5.0,
        goal_completions: 50,
        revenue: 2500.0,
    };

    let json = serde_json::to_string(&source).unwrap();
    let deserialized: TrafficSource = serde_json::from_str(&json).unwrap();

    assert_eq!(source.source, deserialized.source);
    assert_eq!(source.medium, deserialized.medium);
    assert_eq!(source.sessions, deserialized.sessions);
}

#[test]
fn test_channel_data_serialization() {
    let channel = ChannelData {
        channel: "Organic Search".to_string(),
        sessions: 5000,
        users: 4000,
        new_users: 2500,
        bounce_rate: 35.0,
        pages_per_session: 3.5,
        avg_session_duration: 180.0,
        conversions: 250,
        conversion_rate: 5.0,
        revenue: 12500.0,
        percentage: 50.0,
    };

    let json = serde_json::to_string(&channel).unwrap();
    let deserialized: ChannelData = serde_json::from_str(&json).unwrap();

    assert_eq!(channel.channel, deserialized.channel);
    assert_eq!(channel.sessions, deserialized.sessions);
    assert!((channel.percentage - deserialized.percentage).abs() < 0.001);
}

#[test]
fn test_page_data_serialization() {
    let page = PageData {
        page_path: "/products".to_string(),
        page_title: "Products Page".to_string(),
        pageviews: 5000,
        unique_pageviews: 4000,
        avg_time_on_page: 120.0,
        entrances: 3500,
        bounce_rate: 30.0,
        exit_rate: 25.0,
        page_value: 15000.0,
    };

    let json = serde_json::to_string(&page).unwrap();
    let deserialized: PageData = serde_json::from_str(&json).unwrap();

    assert_eq!(page.page_path, deserialized.page_path);
    assert_eq!(page.pageviews, deserialized.pageviews);
}

#[test]
fn test_referrer_data_serialization() {
    let referrer = ReferrerData {
        referrer: "example.com".to_string(),
        sessions: 500,
        users: 400,
        new_users: 350,
        bounce_rate: 35.0,
        pages_per_session: 3.0,
        avg_session_duration: 150.0,
        percentage: 50.0,
    };

    let json = serde_json::to_string(&referrer).unwrap();
    let deserialized: ReferrerData = serde_json::from_str(&json).unwrap();

    assert_eq!(referrer.referrer, deserialized.referrer);
    assert_eq!(referrer.sessions, deserialized.sessions);
}

#[test]
fn test_metrics_comparison_serialization() {
    let comparison = MetricsComparison {
        sessions_change: 15.5,
        users_change: -5.2,
        new_users_change: 10.0,
        pageviews_change: 20.0,
        pages_per_session_change: 0.5,
        avg_session_duration_change: -10.0,
        bounce_rate_change: -2.0,
        goal_conversion_rate_change: 1.5,
        revenue_change: 25.0,
    };

    let json = serde_json::to_string(&comparison).unwrap();
    let deserialized: MetricsComparison = serde_json::from_str(&json).unwrap();

    assert!((comparison.sessions_change - deserialized.sessions_change).abs() < 0.001);
    assert!((comparison.users_change - deserialized.users_change).abs() < 0.001);
}

#[test]
fn test_daily_metrics_serialization() {
    let daily = DailyMetrics {
        date: NaiveDate::from_ymd_opt(2023, 12, 1).unwrap(),
        sessions: 100,
        users: 80,
        new_users: 50,
        pageviews: 300,
        bounce_rate: 40.0,
        avg_session_duration: 120.0,
        transactions: 5,
        revenue: 500.0,
    };

    let json = serde_json::to_string(&daily).unwrap();
    let deserialized: DailyMetrics = serde_json::from_str(&json).unwrap();

    assert_eq!(daily.date, deserialized.date);
    assert_eq!(daily.sessions, deserialized.sessions);
}

// ============================================================================
// Additional Model Tests
// ============================================================================

#[test]
fn test_campaign_data_creation() {
    let campaign = CampaignData {
        campaign: "summer_sale".to_string(),
        source: "google".to_string(),
        medium: "cpc".to_string(),
        sessions: 1000,
        users: 800,
        new_users: 600,
        bounce_rate: 35.0,
        pages_per_session: 3.0,
        avg_session_duration: 120.0,
        conversions: 50,
        conversion_rate: 5.0,
        revenue: 5000.0,
        cost: 1000.0,
        roi: 400.0,
    };

    assert_eq!(campaign.campaign, "summer_sale");
    assert_eq!(campaign.roi, 400.0);
}

#[test]
fn test_campaign_data_serialization() {
    let campaign = CampaignData {
        campaign: "test".to_string(),
        source: "google".to_string(),
        medium: "cpc".to_string(),
        sessions: 100,
        users: 80,
        new_users: 60,
        bounce_rate: 30.0,
        pages_per_session: 2.5,
        avg_session_duration: 100.0,
        conversions: 10,
        conversion_rate: 10.0,
        revenue: 1000.0,
        cost: 200.0,
        roi: 400.0,
    };

    let json = serde_json::to_string(&campaign).unwrap();
    let deserialized: CampaignData = serde_json::from_str(&json).unwrap();

    assert_eq!(campaign.campaign, deserialized.campaign);
    assert!((campaign.roi - deserialized.roi).abs() < 0.001);
}

#[test]
fn test_keyword_data_creation() {
    let keyword = KeywordData {
        keyword: "rust programming".to_string(),
        sessions: 500,
        users: 400,
        bounce_rate: 25.0,
        pages_per_session: 4.0,
        avg_session_duration: 180.0,
        conversions: 25,
        conversion_rate: 5.0,
    };

    assert_eq!(keyword.keyword, "rust programming");
    assert_eq!(keyword.conversions, 25);
}

#[test]
fn test_event_data_creation() {
    let event = EventData {
        event_category: "Button".to_string(),
        event_action: "Click".to_string(),
        event_label: Some("Submit Form".to_string()),
        total_events: 1000,
        unique_events: 800,
        event_value: 5000.0,
        avg_value: 5.0,
        sessions_with_event: 600,
    };

    assert_eq!(event.event_category, "Button");
    assert_eq!(event.event_label, Some("Submit Form".to_string()));
}

#[test]
fn test_event_data_without_label() {
    let event = EventData {
        event_category: "Video".to_string(),
        event_action: "Play".to_string(),
        event_label: None,
        total_events: 500,
        unique_events: 400,
        event_value: 0.0,
        avg_value: 0.0,
        sessions_with_event: 300,
    };

    assert_eq!(event.event_label, None);
}

#[test]
fn test_site_speed_data_creation() {
    let speed = SiteSpeedData {
        avg_page_load_time: 2.5,
        avg_domain_lookup_time: 0.1,
        avg_server_connection_time: 0.2,
        avg_server_response_time: 0.5,
        avg_page_download_time: 0.3,
        avg_redirection_time: 0.1,
        avg_document_interactive_time: 1.5,
        avg_document_content_loaded_time: 2.0,
        page_load_sample: 10000,
    };

    assert!((speed.avg_page_load_time - 2.5).abs() < 0.001);
    assert_eq!(speed.page_load_sample, 10000);
}

#[test]
fn test_page_timing_data_creation() {
    let timing = PageTimingData {
        page_path: "/products".to_string(),
        pageviews: 5000,
        avg_page_load_time: 2.0,
        avg_server_response_time: 0.3,
        avg_page_download_time: 0.2,
        page_load_sample: 4500,
    };

    assert_eq!(timing.page_path, "/products");
    assert!((timing.avg_page_load_time - 2.0).abs() < 0.001);
}

#[test]
fn test_site_search_data_creation() {
    let search = SiteSearchData {
        search_term: "rust tutorial".to_string(),
        total_unique_searches: 100,
        results_pageviews: 250,
        search_exits: 20,
        search_exit_rate: 20.0,
        search_refinements: 15,
        search_depth: 2.5,
        avg_search_duration: 30.0,
    };

    assert_eq!(search.search_term, "rust tutorial");
    assert_eq!(search.total_unique_searches, 100);
}

#[test]
fn test_dimension_value_creation() {
    let dim = AnalyticsDimensionValue {
        value: "United States".to_string(),
        sessions: 10000,
        users: 8000,
        pageviews: 30000,
        bounce_rate: 35.0,
        avg_session_duration: 120.0,
        percentage: 50.0,
    };

    assert_eq!(dim.value, "United States");
    assert!((dim.percentage - 50.0).abs() < 0.001);
}

#[test]
fn test_segment_creation() {
    let segment = AnalyticsSegment {
        id: "gaid::-1".to_string(),
        name: "All Users".to_string(),
        definition: "users::condition::".to_string(),
        segment_type: SegmentType::BuiltIn,
    };

    assert_eq!(segment.id, "gaid::-1");
    assert_eq!(segment.segment_type, SegmentType::BuiltIn);
}

#[test]
fn test_segment_type_variants() {
    assert_eq!(SegmentType::BuiltIn, SegmentType::BuiltIn);
    assert_eq!(SegmentType::Custom, SegmentType::Custom);
    assert_eq!(SegmentType::System, SegmentType::System);
    assert_ne!(SegmentType::BuiltIn, SegmentType::Custom);
}

#[test]
fn test_sampling_info_creation() {
    let sampling = SamplingInfo {
        is_sampled: true,
        samples_read_counts: Some(100000),
        sampling_space_sizes: Some(1000000),
        sampling_level: Some(SamplingLevel::Default),
    };

    assert!(sampling.is_sampled);
    assert_eq!(sampling.samples_read_counts, Some(100000));
}

#[test]
fn test_sampling_level_variants() {
    assert_eq!(SamplingLevel::Default, SamplingLevel::Default);
    assert_eq!(SamplingLevel::Small, SamplingLevel::Small);
    assert_eq!(SamplingLevel::Large, SamplingLevel::Large);
}

#[test]
fn test_sampling_info_not_sampled() {
    let sampling = SamplingInfo {
        is_sampled: false,
        samples_read_counts: None,
        sampling_space_sizes: None,
        sampling_level: None,
    };

    assert!(!sampling.is_sampled);
    assert!(sampling.samples_read_counts.is_none());
}

// ============================================================================
// Clone and Debug Tests
// ============================================================================

#[test]
fn test_overview_metrics_clone() {
    let metrics = OverviewMetrics {
        sessions: 100,
        users: 80,
        new_users: 50,
        pageviews: 300,
        pages_per_session: 3.0,
        avg_session_duration: 120.0,
        bounce_rate: 40.0,
        goal_conversion_rate: 10.0,
        goal_completions: 10,
        goal_value: 500.0,
        transactions: 5,
        revenue: 500.0,
        ecommerce_conversion_rate: 5.0,
    };

    let cloned = metrics.clone();
    assert_eq!(metrics.sessions, cloned.sessions);
    assert_eq!(metrics.users, cloned.users);
}

#[test]
fn test_date_range_clone() {
    let range = DateRange::last_n_days(7);
    let cloned = range.clone();

    assert_eq!(range.start_date, cloned.start_date);
    assert_eq!(range.end_date, cloned.end_date);
}

#[test]
fn test_traffic_source_debug() {
    let source = TrafficSource {
        source: "google".to_string(),
        medium: "organic".to_string(),
        sessions: 1000,
        users: 800,
        new_users: 500,
        bounce_rate: 35.0,
        pages_per_session: 3.5,
        avg_session_duration: 180.0,
        goal_conversion_rate: 5.0,
        goal_completions: 50,
        revenue: 2500.0,
    };

    let debug_str = format!("{:?}", source);
    assert!(debug_str.contains("google"));
    assert!(debug_str.contains("organic"));
}

#[test]
fn test_channel_data_debug() {
    let channel = ChannelData {
        channel: "Organic Search".to_string(),
        sessions: 5000,
        users: 4000,
        new_users: 2500,
        bounce_rate: 35.0,
        pages_per_session: 3.5,
        avg_session_duration: 180.0,
        conversions: 250,
        conversion_rate: 5.0,
        revenue: 12500.0,
        percentage: 50.0,
    };

    let debug_str = format!("{:?}", channel);
    assert!(debug_str.contains("Organic Search"));
}

// ============================================================================
// Response Row Count Tests
// ============================================================================

#[test]
fn test_single_row_response() {
    let response: RunReportResponse = serde_json::from_value(json!({
        "dimensionHeaders": [{ "name": "sessionDefaultChannelGroup" }],
        "metricHeaders": [
            { "name": "sessions", "type": "TYPE_INTEGER" },
            { "name": "totalUsers", "type": "TYPE_INTEGER" },
            { "name": "newUsers", "type": "TYPE_INTEGER" },
            { "name": "bounceRate", "type": "TYPE_FLOAT" },
            { "name": "screenPageViewsPerSession", "type": "TYPE_FLOAT" },
            { "name": "averageSessionDuration", "type": "TYPE_FLOAT" },
            { "name": "conversions", "type": "TYPE_INTEGER" },
            { "name": "totalRevenue", "type": "TYPE_FLOAT" }
        ],
        "rows": [
            {
                "dimensionValues": [{ "value": "Organic Search" }],
                "metricValues": [
                    { "value": "1000" },
                    { "value": "800" },
                    { "value": "500" },
                    { "value": "0.30" },
                    { "value": "3.0" },
                    { "value": "120.0" },
                    { "value": "50" },
                    { "value": "2500.00" }
                ]
            }
        ],
        "rowCount": 1
    })).unwrap();

    let channels = process_channels_response(response);

    assert_eq!(channels.len(), 1);
    assert!((channels[0].percentage - 100.0).abs() < 0.01); // Single channel = 100%
}

#[test]
fn test_large_row_count_response() {
    let mut rows = Vec::new();
    for i in 0..100 {
        rows.push(json!({
            "dimensionValues": [{ "value": format!("Source {}", i) }],
            "metricValues": [
                { "value": "100" },
                { "value": "80" },
                { "value": "50" },
                { "value": "0.30" },
                { "value": "3.0" },
                { "value": "120.0" }
            ]
        }));
    }

    let response: RunReportResponse = serde_json::from_value(json!({
        "dimensionHeaders": [{ "name": "sessionSource" }],
        "metricHeaders": [
            { "name": "sessions", "type": "TYPE_INTEGER" },
            { "name": "totalUsers", "type": "TYPE_INTEGER" },
            { "name": "newUsers", "type": "TYPE_INTEGER" },
            { "name": "bounceRate", "type": "TYPE_FLOAT" },
            { "name": "screenPageViewsPerSession", "type": "TYPE_FLOAT" },
            { "name": "averageSessionDuration", "type": "TYPE_FLOAT" }
        ],
        "rows": rows,
        "totals": [
            {
                "dimensionValues": [],
                "metricValues": [
                    { "value": "10000" },
                    { "value": "8000" },
                    { "value": "5000" },
                    { "value": "0.30" },
                    { "value": "3.0" },
                    { "value": "120.0" }
                ]
            }
        ],
        "rowCount": 100
    })).unwrap();

    let referrers = process_referrers_response(response);

    assert_eq!(referrers.len(), 100);

    // Each referrer should have 1% (100/10000 * 100)
    for referrer in &referrers {
        assert!((referrer.percentage - 1.0).abs() < 0.01);
    }
}
