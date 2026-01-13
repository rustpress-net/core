//! Analytics Service
//!
//! Core analytics service for fetching and processing Google Analytics data.

use std::sync::Arc;

use chrono::NaiveDate;
use tracing::debug;

use crate::models::*;
use crate::services::cache::CacheService;
use crate::services::client::{ClientError, GoogleAnalyticsClient};

/// Analytics Service for fetching and processing GA data
pub struct AnalyticsService {
    /// GA API client
    client: Arc<GoogleAnalyticsClient>,
    /// Cache service
    cache: Arc<CacheService>,
}

impl AnalyticsService {
    /// Create a new analytics service
    pub fn new(client: Arc<GoogleAnalyticsClient>, cache: Arc<CacheService>) -> Self {
        Self { client, cache }
    }

    /// Get analytics overview
    pub async fn get_overview(
        &self,
        date_range: DateRange,
        compare: bool,
    ) -> Result<AnalyticsOverview, ClientError> {
        let cache_key = format!(
            "overview:{}:{}:{}",
            date_range.start_date, date_range.end_date, compare
        );

        // Check cache
        if let Some(cached) = self.cache.get::<AnalyticsOverview>(&cache_key).await {
            debug!("Returning cached overview data");
            return Ok(cached);
        }

        // Build the report request
        let mut date_ranges = vec![GoogleAnalyticsClient::build_date_range(&date_range)];

        // Add comparison date range if requested
        if compare {
            let days = (date_range.end_date - date_range.start_date).num_days();
            let compare_end = date_range.start_date - chrono::Duration::days(1);
            let compare_start = compare_end - chrono::Duration::days(days);
            date_ranges.push(ApiDateRange {
                start_date: compare_start.format("%Y-%m-%d").to_string(),
                end_date: compare_end.format("%Y-%m-%d").to_string(),
                name: Some("comparison".to_string()),
            });
        }

        let request = RunReportRequest {
            property: format!("properties/{}", self.client.property_id()),
            date_ranges: date_ranges.clone(),
            dimensions: Some(vec![GoogleAnalyticsClient::dimension("date")]),
            metrics: vec![
                GoogleAnalyticsClient::metric("sessions"),
                GoogleAnalyticsClient::metric("totalUsers"),
                GoogleAnalyticsClient::metric("newUsers"),
                GoogleAnalyticsClient::metric("screenPageViews"),
                GoogleAnalyticsClient::metric("screenPageViewsPerSession"),
                GoogleAnalyticsClient::metric("averageSessionDuration"),
                GoogleAnalyticsClient::metric("bounceRate"),
                GoogleAnalyticsClient::metric("conversions"),
                GoogleAnalyticsClient::metric("totalRevenue"),
                GoogleAnalyticsClient::metric("ecommercePurchases"),
            ],
            dimension_filter: None,
            metric_filter: None,
            order_bys: Some(vec![OrderBy {
                desc: Some(false),
                dimension: Some(DimensionOrderBy {
                    dimension_name: "date".to_string(),
                    order_type: Some(DimensionOrderType::Alphanumeric),
                }),
                metric: None,
                pivot: None,
            }]),
            offset: None,
            limit: None,
            metric_aggregations: Some(vec!["TOTAL".to_string()]),
            keep_empty_rows: Some(true),
            return_property_quota: None,
        };

        let response = self.client.run_report(request).await?;

        // Process the response
        let overview = self.process_overview_response(response, date_range, compare)?;

        // Cache the result
        self.cache.set(&cache_key, &overview).await;

        Ok(overview)
    }

    /// Process overview response into AnalyticsOverview
    fn process_overview_response(
        &self,
        response: RunReportResponse,
        date_range: DateRange,
        _compare: bool,
    ) -> Result<AnalyticsOverview, ClientError> {
        let mut chart_data = Vec::new();
        let mut metrics = OverviewMetrics::default();

        // Process totals
        if let Some(totals) = response.totals.as_ref().and_then(|t| t.first()) {
            if let Some(values) = &totals.metric_values {
                metrics = OverviewMetrics {
                    sessions: Self::parse_metric_value(&values.get(0)),
                    users: Self::parse_metric_value(&values.get(1)),
                    new_users: Self::parse_metric_value(&values.get(2)),
                    pageviews: Self::parse_metric_value(&values.get(3)),
                    pages_per_session: Self::parse_metric_float(&values.get(4)),
                    avg_session_duration: Self::parse_metric_float(&values.get(5)),
                    bounce_rate: Self::parse_metric_float(&values.get(6)) * 100.0,
                    goal_conversion_rate: 0.0,
                    goal_completions: Self::parse_metric_value(&values.get(7)),
                    goal_value: Self::parse_metric_float(&values.get(8)),
                    transactions: Self::parse_metric_value(&values.get(9)),
                    revenue: Self::parse_metric_float(&values.get(8)),
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
                                sessions: Self::parse_metric_value(&vals.get(0)),
                                users: Self::parse_metric_value(&vals.get(1)),
                                new_users: Self::parse_metric_value(&vals.get(2)),
                                pageviews: Self::parse_metric_value(&vals.get(3)),
                                bounce_rate: Self::parse_metric_float(&vals.get(6)) * 100.0,
                                avg_session_duration: Self::parse_metric_float(&vals.get(5)),
                                transactions: Self::parse_metric_value(&vals.get(9)),
                                revenue: Self::parse_metric_float(&vals.get(8)),
                            });
                        }
                    }
                }
            }
        }

        Ok(AnalyticsOverview {
            date_range,
            metrics,
            comparison: None, // TODO: Implement comparison processing
            chart_data,
        })
    }

    /// Get traffic sources data
    pub async fn get_traffic_sources(
        &self,
        date_range: DateRange,
        limit: Option<i64>,
    ) -> Result<Vec<TrafficSource>, ClientError> {
        let cache_key = format!(
            "traffic_sources:{}:{}:{}",
            date_range.start_date,
            date_range.end_date,
            limit.unwrap_or(10)
        );

        if let Some(cached) = self.cache.get::<Vec<TrafficSource>>(&cache_key).await {
            return Ok(cached);
        }

        let request = RunReportRequest {
            property: format!("properties/{}", self.client.property_id()),
            date_ranges: vec![GoogleAnalyticsClient::build_date_range(&date_range)],
            dimensions: Some(vec![
                GoogleAnalyticsClient::dimension("sessionSource"),
                GoogleAnalyticsClient::dimension("sessionMedium"),
            ]),
            metrics: vec![
                GoogleAnalyticsClient::metric("sessions"),
                GoogleAnalyticsClient::metric("totalUsers"),
                GoogleAnalyticsClient::metric("newUsers"),
                GoogleAnalyticsClient::metric("bounceRate"),
                GoogleAnalyticsClient::metric("screenPageViewsPerSession"),
                GoogleAnalyticsClient::metric("averageSessionDuration"),
                GoogleAnalyticsClient::metric("conversions"),
                GoogleAnalyticsClient::metric("totalRevenue"),
            ],
            dimension_filter: None,
            metric_filter: None,
            order_bys: Some(vec![GoogleAnalyticsClient::order_by_metric_desc("sessions")]),
            offset: None,
            limit,
            metric_aggregations: None,
            keep_empty_rows: None,
            return_property_quota: None,
        };

        let response = self.client.run_report(request).await?;
        let sources = self.process_traffic_sources_response(response)?;

        self.cache.set(&cache_key, &sources).await;
        Ok(sources)
    }

    fn process_traffic_sources_response(
        &self,
        response: RunReportResponse,
    ) -> Result<Vec<TrafficSource>, ClientError> {
        let mut sources = Vec::new();

        if let Some(rows) = response.rows {
            for row in rows {
                if let (Some(dims), Some(vals)) = (&row.dimension_values, &row.metric_values) {
                    let source = dims.get(0).and_then(|d| d.value.clone()).unwrap_or_default();
                    let medium = dims.get(1).and_then(|d| d.value.clone()).unwrap_or_default();

                    let sessions = Self::parse_metric_value(&vals.get(0));

                    sources.push(TrafficSource {
                        source,
                        medium,
                        sessions,
                        users: Self::parse_metric_value(&vals.get(1)),
                        new_users: Self::parse_metric_value(&vals.get(2)),
                        bounce_rate: Self::parse_metric_float(&vals.get(3)) * 100.0,
                        pages_per_session: Self::parse_metric_float(&vals.get(4)),
                        avg_session_duration: Self::parse_metric_float(&vals.get(5)),
                        goal_conversion_rate: if sessions > 0 {
                            (Self::parse_metric_value(&vals.get(6)) as f64 / sessions as f64) * 100.0
                        } else {
                            0.0
                        },
                        goal_completions: Self::parse_metric_value(&vals.get(6)),
                        revenue: Self::parse_metric_float(&vals.get(7)),
                    });
                }
            }
        }

        Ok(sources)
    }

    /// Get channel data
    pub async fn get_channels(
        &self,
        date_range: DateRange,
    ) -> Result<Vec<ChannelData>, ClientError> {
        let cache_key = format!("channels:{}:{}", date_range.start_date, date_range.end_date);

        if let Some(cached) = self.cache.get::<Vec<ChannelData>>(&cache_key).await {
            return Ok(cached);
        }

        let request = RunReportRequest {
            property: format!("properties/{}", self.client.property_id()),
            date_ranges: vec![GoogleAnalyticsClient::build_date_range(&date_range)],
            dimensions: Some(vec![GoogleAnalyticsClient::dimension("sessionDefaultChannelGroup")]),
            metrics: vec![
                GoogleAnalyticsClient::metric("sessions"),
                GoogleAnalyticsClient::metric("totalUsers"),
                GoogleAnalyticsClient::metric("newUsers"),
                GoogleAnalyticsClient::metric("bounceRate"),
                GoogleAnalyticsClient::metric("screenPageViewsPerSession"),
                GoogleAnalyticsClient::metric("averageSessionDuration"),
                GoogleAnalyticsClient::metric("conversions"),
                GoogleAnalyticsClient::metric("totalRevenue"),
            ],
            dimension_filter: None,
            metric_filter: None,
            order_bys: Some(vec![GoogleAnalyticsClient::order_by_metric_desc("sessions")]),
            offset: None,
            limit: None,
            metric_aggregations: Some(vec!["TOTAL".to_string()]),
            keep_empty_rows: None,
            return_property_quota: None,
        };

        let response = self.client.run_report(request).await?;
        let channels = self.process_channels_response(response)?;

        self.cache.set(&cache_key, &channels).await;
        Ok(channels)
    }

    fn process_channels_response(
        &self,
        response: RunReportResponse,
    ) -> Result<Vec<ChannelData>, ClientError> {
        let mut channels = Vec::new();
        let mut total_sessions: u64 = 0;

        // First pass: collect all data and total sessions
        let mut raw_data = Vec::new();

        if let Some(rows) = response.rows {
            for row in rows {
                if let (Some(dims), Some(vals)) = (&row.dimension_values, &row.metric_values) {
                    let channel = dims.get(0).and_then(|d| d.value.clone()).unwrap_or_default();
                    let sessions = Self::parse_metric_value(&vals.get(0));
                    total_sessions += sessions;

                    raw_data.push((channel, vals.clone()));
                }
            }
        }

        // Second pass: calculate percentages
        for (channel, vals) in raw_data {
            let sessions = Self::parse_metric_value(&vals.get(0));

            channels.push(ChannelData {
                channel,
                sessions,
                users: Self::parse_metric_value(&vals.get(1)),
                new_users: Self::parse_metric_value(&vals.get(2)),
                bounce_rate: Self::parse_metric_float(&vals.get(3)) * 100.0,
                pages_per_session: Self::parse_metric_float(&vals.get(4)),
                avg_session_duration: Self::parse_metric_float(&vals.get(5)),
                conversions: Self::parse_metric_value(&vals.get(6)),
                conversion_rate: if sessions > 0 {
                    (Self::parse_metric_value(&vals.get(6)) as f64 / sessions as f64) * 100.0
                } else {
                    0.0
                },
                revenue: Self::parse_metric_float(&vals.get(7)),
                percentage: if total_sessions > 0 {
                    (sessions as f64 / total_sessions as f64) * 100.0
                } else {
                    0.0
                },
            });
        }

        Ok(channels)
    }

    /// Get top pages
    pub async fn get_top_pages(
        &self,
        date_range: DateRange,
        limit: Option<i64>,
    ) -> Result<Vec<PageData>, ClientError> {
        let cache_key = format!(
            "top_pages:{}:{}:{}",
            date_range.start_date,
            date_range.end_date,
            limit.unwrap_or(10)
        );

        if let Some(cached) = self.cache.get::<Vec<PageData>>(&cache_key).await {
            return Ok(cached);
        }

        let request = RunReportRequest {
            property: format!("properties/{}", self.client.property_id()),
            date_ranges: vec![GoogleAnalyticsClient::build_date_range(&date_range)],
            dimensions: Some(vec![
                GoogleAnalyticsClient::dimension("pagePath"),
                GoogleAnalyticsClient::dimension("pageTitle"),
            ]),
            metrics: vec![
                GoogleAnalyticsClient::metric("screenPageViews"),
                GoogleAnalyticsClient::metric("sessions"),
                GoogleAnalyticsClient::metric("averageSessionDuration"),
                GoogleAnalyticsClient::metric("bounceRate"),
                GoogleAnalyticsClient::metric("totalRevenue"),
            ],
            dimension_filter: None,
            metric_filter: None,
            order_bys: Some(vec![GoogleAnalyticsClient::order_by_metric_desc("screenPageViews")]),
            offset: None,
            limit,
            metric_aggregations: None,
            keep_empty_rows: None,
            return_property_quota: None,
        };

        let response = self.client.run_report(request).await?;
        let pages = self.process_pages_response(response)?;

        self.cache.set(&cache_key, &pages).await;
        Ok(pages)
    }

    fn process_pages_response(
        &self,
        response: RunReportResponse,
    ) -> Result<Vec<PageData>, ClientError> {
        let mut pages = Vec::new();

        if let Some(rows) = response.rows {
            for row in rows {
                if let (Some(dims), Some(vals)) = (&row.dimension_values, &row.metric_values) {
                    let page_path = dims.get(0).and_then(|d| d.value.clone()).unwrap_or_default();
                    let page_title = dims.get(1).and_then(|d| d.value.clone()).unwrap_or_default();
                    let pageviews = Self::parse_metric_value(&vals.get(0));
                    let sessions = Self::parse_metric_value(&vals.get(1));

                    pages.push(PageData {
                        page_path,
                        page_title,
                        pageviews,
                        unique_pageviews: sessions,
                        avg_time_on_page: Self::parse_metric_float(&vals.get(2)),
                        entrances: sessions,
                        bounce_rate: Self::parse_metric_float(&vals.get(3)) * 100.0,
                        exit_rate: 0.0, // Would need additional query
                        page_value: Self::parse_metric_float(&vals.get(4)),
                    });
                }
            }
        }

        Ok(pages)
    }

    /// Get referrers data
    pub async fn get_referrers(
        &self,
        date_range: DateRange,
        limit: Option<i64>,
    ) -> Result<Vec<ReferrerData>, ClientError> {
        let cache_key = format!(
            "referrers:{}:{}:{}",
            date_range.start_date,
            date_range.end_date,
            limit.unwrap_or(10)
        );

        if let Some(cached) = self.cache.get::<Vec<ReferrerData>>(&cache_key).await {
            return Ok(cached);
        }

        let request = RunReportRequest {
            property: format!("properties/{}", self.client.property_id()),
            date_ranges: vec![GoogleAnalyticsClient::build_date_range(&date_range)],
            dimensions: Some(vec![GoogleAnalyticsClient::dimension("sessionSource")]),
            metrics: vec![
                GoogleAnalyticsClient::metric("sessions"),
                GoogleAnalyticsClient::metric("totalUsers"),
                GoogleAnalyticsClient::metric("newUsers"),
                GoogleAnalyticsClient::metric("bounceRate"),
                GoogleAnalyticsClient::metric("screenPageViewsPerSession"),
                GoogleAnalyticsClient::metric("averageSessionDuration"),
            ],
            dimension_filter: Some(GoogleAnalyticsClient::string_filter(
                "sessionMedium",
                StringFilterMatchType::Exact,
                "referral",
            )),
            metric_filter: None,
            order_bys: Some(vec![GoogleAnalyticsClient::order_by_metric_desc("sessions")]),
            offset: None,
            limit,
            metric_aggregations: Some(vec!["TOTAL".to_string()]),
            keep_empty_rows: None,
            return_property_quota: None,
        };

        let response = self.client.run_report(request).await?;
        let referrers = self.process_referrers_response(response)?;

        self.cache.set(&cache_key, &referrers).await;
        Ok(referrers)
    }

    fn process_referrers_response(
        &self,
        response: RunReportResponse,
    ) -> Result<Vec<ReferrerData>, ClientError> {
        let mut referrers = Vec::new();
        let mut total_sessions: u64 = 0;

        // First pass: get total
        if let Some(totals) = response.totals.as_ref().and_then(|t| t.first()) {
            if let Some(values) = &totals.metric_values {
                total_sessions = Self::parse_metric_value(&values.get(0));
            }
        }

        // Second pass: process rows
        if let Some(rows) = response.rows {
            for row in rows {
                if let (Some(dims), Some(vals)) = (&row.dimension_values, &row.metric_values) {
                    let referrer = dims.get(0).and_then(|d| d.value.clone()).unwrap_or_default();
                    let sessions = Self::parse_metric_value(&vals.get(0));

                    referrers.push(ReferrerData {
                        referrer,
                        sessions,
                        users: Self::parse_metric_value(&vals.get(1)),
                        new_users: Self::parse_metric_value(&vals.get(2)),
                        bounce_rate: Self::parse_metric_float(&vals.get(3)) * 100.0,
                        pages_per_session: Self::parse_metric_float(&vals.get(4)),
                        avg_session_duration: Self::parse_metric_float(&vals.get(5)),
                        percentage: if total_sessions > 0 {
                            (sessions as f64 / total_sessions as f64) * 100.0
                        } else {
                            0.0
                        },
                    });
                }
            }
        }

        Ok(referrers)
    }

    // Helper methods for parsing metric values

    fn parse_metric_value(value: &Option<&MetricValue>) -> u64 {
        value
            .and_then(|v| v.value.as_ref())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    fn parse_metric_float(value: &Option<&MetricValue>) -> f64 {
        value
            .and_then(|v| v.value.as_ref())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0)
    }
}

impl std::fmt::Debug for AnalyticsService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnalyticsService")
            .field("client", &self.client)
            .finish()
    }
}
