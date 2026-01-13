//! Real-time Analytics Service
//!
//! Service for fetching real-time analytics data from Google Analytics.

use std::sync::Arc;

use chrono::Utc;

use crate::models::realtime::*;
use crate::models::api::*;
use crate::services::client::{ClientError, GoogleAnalyticsClient};

/// Real-time Analytics Service
pub struct RealtimeService {
    /// GA API client
    client: Arc<GoogleAnalyticsClient>,
}

impl RealtimeService {
    /// Create a new realtime service
    pub fn new(client: Arc<GoogleAnalyticsClient>) -> Self {
        Self { client }
    }

    /// Get real-time overview data
    pub async fn get_overview(&self) -> Result<RealtimeOverview, ClientError> {
        // Fetch multiple real-time reports in parallel
        let (
            active_users_result,
            pages_result,
            sources_result,
            locations_result,
            devices_result,
        ) = tokio::join!(
            self.get_active_users(),
            self.get_top_active_pages(10),
            self.get_top_traffic_sources(10),
            self.get_top_locations(10),
            self.get_device_breakdown(),
        );

        let active_users = active_users_result?;
        let top_active_pages = pages_result?;
        let top_traffic_sources = sources_result?;
        let top_locations = locations_result?;
        let device_breakdown = devices_result?;

        Ok(RealtimeOverview {
            active_users: active_users.0,
            active_users_1min: active_users.1,
            active_users_5min: active_users.2,
            active_users_10min: active_users.3,
            active_users_30min: active_users.4,
            pageviews_per_minute: Vec::new(), // Would need time-series data
            pageviews_per_second: Vec::new(),
            top_active_pages,
            top_referrers: Vec::new(),
            top_keywords: Vec::new(),
            top_locations,
            top_traffic_sources,
            top_social_sources: Vec::new(),
            device_breakdown,
            active_events: Vec::new(),
            active_conversions: Vec::new(),
            timestamp: Utc::now(),
        })
    }

    /// Get active users count at different time intervals
    pub async fn get_active_users(&self) -> Result<(u32, u32, u32, u32, u32), ClientError> {
        let request = RunRealtimeReportRequest {
            property: format!("properties/{}", self.client.property_id()),
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
                    name: Some("current".to_string()),
                    start_minutes_ago: Some(0),
                    end_minutes_ago: Some(0),
                },
                MinuteRange {
                    name: Some("1min".to_string()),
                    start_minutes_ago: Some(1),
                    end_minutes_ago: Some(0),
                },
                MinuteRange {
                    name: Some("5min".to_string()),
                    start_minutes_ago: Some(5),
                    end_minutes_ago: Some(0),
                },
                MinuteRange {
                    name: Some("10min".to_string()),
                    start_minutes_ago: Some(10),
                    end_minutes_ago: Some(0),
                },
                MinuteRange {
                    name: Some("30min".to_string()),
                    start_minutes_ago: Some(30),
                    end_minutes_ago: Some(0),
                },
            ]),
        };

        let response = self.client.run_realtime_report(request).await?;

        // Parse the response to get active users for each time range
        let mut current = 0u32;
        let mut min1 = 0u32;
        let mut min5 = 0u32;
        let mut min10 = 0u32;
        let mut min30 = 0u32;

        if let Some(rows) = response.rows {
            for (i, row) in rows.iter().enumerate() {
                if let Some(vals) = &row.metric_values {
                    if let Some(val) = vals.get(0).and_then(|v| v.value.as_ref()) {
                        let count: u32 = val.parse().unwrap_or(0);
                        match i {
                            0 => current = count,
                            1 => min1 = count,
                            2 => min5 = count,
                            3 => min10 = count,
                            4 => min30 = count,
                            _ => {}
                        }
                    }
                }
            }
        }

        // If no minute ranges, use totals
        if let Some(totals) = response.totals.as_ref().and_then(|t| t.first()) {
            if let Some(vals) = &totals.metric_values {
                if let Some(val) = vals.get(0).and_then(|v| v.value.as_ref()) {
                    let count: u32 = val.parse().unwrap_or(0);
                    if current == 0 {
                        current = count;
                        min1 = count;
                        min5 = count;
                        min10 = count;
                        min30 = count;
                    }
                }
            }
        }

        Ok((current, min1, min5, min10, min30))
    }

    /// Get top active pages
    pub async fn get_top_active_pages(&self, limit: i64) -> Result<Vec<ActivePage>, ClientError> {
        let request = RunRealtimeReportRequest {
            property: format!("properties/{}", self.client.property_id()),
            dimensions: Some(vec![
                GoogleAnalyticsClient::dimension("unifiedPagePathScreen"),
                GoogleAnalyticsClient::dimension("unifiedScreenName"),
            ]),
            metrics: vec![GoogleAnalyticsClient::metric("activeUsers")],
            dimension_filter: None,
            metric_filter: None,
            limit: Some(limit),
            metric_aggregations: Some(vec!["TOTAL".to_string()]),
            order_bys: Some(vec![GoogleAnalyticsClient::order_by_metric_desc("activeUsers")]),
            return_property_quota: None,
            minute_ranges: Some(vec![MinuteRange {
                name: None,
                start_minutes_ago: Some(30),
                end_minutes_ago: Some(0),
            }]),
        };

        let response = self.client.run_realtime_report(request).await?;

        let mut pages = Vec::new();
        let mut total_users: u32 = 0;

        // Get total from totals row
        if let Some(totals) = response.totals.as_ref().and_then(|t| t.first()) {
            if let Some(vals) = &totals.metric_values {
                if let Some(val) = vals.get(0).and_then(|v| v.value.as_ref()) {
                    total_users = val.parse().unwrap_or(0);
                }
            }
        }

        if let Some(rows) = response.rows {
            for row in rows {
                if let (Some(dims), Some(vals)) = (&row.dimension_values, &row.metric_values) {
                    let page_path = dims.get(0).and_then(|d| d.value.clone()).unwrap_or_default();
                    let page_title = dims.get(1).and_then(|d| d.value.clone()).unwrap_or_default();
                    let active_users: u32 = vals
                        .get(0)
                        .and_then(|v| v.value.as_ref())
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);

                    pages.push(ActivePage {
                        page_path,
                        page_title,
                        active_users,
                        percentage: if total_users > 0 {
                            (active_users as f64 / total_users as f64) * 100.0
                        } else {
                            0.0
                        },
                    });
                }
            }
        }

        Ok(pages)
    }

    /// Get top traffic sources
    pub async fn get_top_traffic_sources(
        &self,
        limit: i64,
    ) -> Result<Vec<ActiveTrafficSource>, ClientError> {
        let request = RunRealtimeReportRequest {
            property: format!("properties/{}", self.client.property_id()),
            dimensions: Some(vec![
                GoogleAnalyticsClient::dimension("sessionSource"),
                GoogleAnalyticsClient::dimension("sessionMedium"),
            ]),
            metrics: vec![GoogleAnalyticsClient::metric("activeUsers")],
            dimension_filter: None,
            metric_filter: None,
            limit: Some(limit),
            metric_aggregations: Some(vec!["TOTAL".to_string()]),
            order_bys: Some(vec![GoogleAnalyticsClient::order_by_metric_desc("activeUsers")]),
            return_property_quota: None,
            minute_ranges: Some(vec![MinuteRange {
                name: None,
                start_minutes_ago: Some(30),
                end_minutes_ago: Some(0),
            }]),
        };

        let response = self.client.run_realtime_report(request).await?;

        let mut sources = Vec::new();
        let mut total_users: u32 = 0;

        if let Some(totals) = response.totals.as_ref().and_then(|t| t.first()) {
            if let Some(vals) = &totals.metric_values {
                if let Some(val) = vals.get(0).and_then(|v| v.value.as_ref()) {
                    total_users = val.parse().unwrap_or(0);
                }
            }
        }

        if let Some(rows) = response.rows {
            for row in rows {
                if let (Some(dims), Some(vals)) = (&row.dimension_values, &row.metric_values) {
                    let source = dims.get(0).and_then(|d| d.value.clone()).unwrap_or_default();
                    let medium = dims.get(1).and_then(|d| d.value.clone()).unwrap_or_default();
                    let active_users: u32 = vals
                        .get(0)
                        .and_then(|v| v.value.as_ref())
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);

                    sources.push(ActiveTrafficSource {
                        source,
                        medium,
                        active_users,
                        percentage: if total_users > 0 {
                            (active_users as f64 / total_users as f64) * 100.0
                        } else {
                            0.0
                        },
                    });
                }
            }
        }

        Ok(sources)
    }

    /// Get top locations
    pub async fn get_top_locations(&self, limit: i64) -> Result<Vec<ActiveLocation>, ClientError> {
        let request = RunRealtimeReportRequest {
            property: format!("properties/{}", self.client.property_id()),
            dimensions: Some(vec![
                GoogleAnalyticsClient::dimension("country"),
                GoogleAnalyticsClient::dimension("city"),
            ]),
            metrics: vec![GoogleAnalyticsClient::metric("activeUsers")],
            dimension_filter: None,
            metric_filter: None,
            limit: Some(limit),
            metric_aggregations: Some(vec!["TOTAL".to_string()]),
            order_bys: Some(vec![GoogleAnalyticsClient::order_by_metric_desc("activeUsers")]),
            return_property_quota: None,
            minute_ranges: Some(vec![MinuteRange {
                name: None,
                start_minutes_ago: Some(30),
                end_minutes_ago: Some(0),
            }]),
        };

        let response = self.client.run_realtime_report(request).await?;

        let mut locations = Vec::new();
        let mut total_users: u32 = 0;

        if let Some(totals) = response.totals.as_ref().and_then(|t| t.first()) {
            if let Some(vals) = &totals.metric_values {
                if let Some(val) = vals.get(0).and_then(|v| v.value.as_ref()) {
                    total_users = val.parse().unwrap_or(0);
                }
            }
        }

        if let Some(rows) = response.rows {
            for row in rows {
                if let (Some(dims), Some(vals)) = (&row.dimension_values, &row.metric_values) {
                    let country = dims.get(0).and_then(|d| d.value.clone()).unwrap_or_default();
                    let city = dims.get(1).and_then(|d| d.value.clone());
                    let active_users: u32 = vals
                        .get(0)
                        .and_then(|v| v.value.as_ref())
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);

                    locations.push(ActiveLocation {
                        country: country.clone(),
                        country_code: String::new(), // Would need to map
                        region: None,
                        city,
                        latitude: None,
                        longitude: None,
                        active_users,
                        percentage: if total_users > 0 {
                            (active_users as f64 / total_users as f64) * 100.0
                        } else {
                            0.0
                        },
                    });
                }
            }
        }

        Ok(locations)
    }

    /// Get device breakdown
    pub async fn get_device_breakdown(&self) -> Result<DeviceBreakdown, ClientError> {
        let request = RunRealtimeReportRequest {
            property: format!("properties/{}", self.client.property_id()),
            dimensions: Some(vec![GoogleAnalyticsClient::dimension("deviceCategory")]),
            metrics: vec![GoogleAnalyticsClient::metric("activeUsers")],
            dimension_filter: None,
            metric_filter: None,
            limit: None,
            metric_aggregations: Some(vec!["TOTAL".to_string()]),
            order_bys: None,
            return_property_quota: None,
            minute_ranges: Some(vec![MinuteRange {
                name: None,
                start_minutes_ago: Some(30),
                end_minutes_ago: Some(0),
            }]),
        };

        let response = self.client.run_realtime_report(request).await?;

        let mut desktop = DeviceStats {
            active_users: 0,
            percentage: 0.0,
        };
        let mut mobile = DeviceStats {
            active_users: 0,
            percentage: 0.0,
        };
        let mut tablet = DeviceStats {
            active_users: 0,
            percentage: 0.0,
        };
        let mut total_users: u32 = 0;

        if let Some(totals) = response.totals.as_ref().and_then(|t| t.first()) {
            if let Some(vals) = &totals.metric_values {
                if let Some(val) = vals.get(0).and_then(|v| v.value.as_ref()) {
                    total_users = val.parse().unwrap_or(0);
                }
            }
        }

        if let Some(rows) = response.rows {
            for row in rows {
                if let (Some(dims), Some(vals)) = (&row.dimension_values, &row.metric_values) {
                    let device = dims
                        .get(0)
                        .and_then(|d| d.value.as_ref())
                        .map(|s| s.to_lowercase())
                        .unwrap_or_default();
                    let users: u32 = vals
                        .get(0)
                        .and_then(|v| v.value.as_ref())
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);

                    let percentage = if total_users > 0 {
                        (users as f64 / total_users as f64) * 100.0
                    } else {
                        0.0
                    };

                    match device.as_str() {
                        "desktop" => {
                            desktop.active_users = users;
                            desktop.percentage = percentage;
                        }
                        "mobile" => {
                            mobile.active_users = users;
                            mobile.percentage = percentage;
                        }
                        "tablet" => {
                            tablet.active_users = users;
                            tablet.percentage = percentage;
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(DeviceBreakdown {
            desktop,
            mobile,
            tablet,
        })
    }

    /// Get real-time events
    pub async fn get_active_events(&self, limit: i64) -> Result<Vec<ActiveEvent>, ClientError> {
        let request = RunRealtimeReportRequest {
            property: format!("properties/{}", self.client.property_id()),
            dimensions: Some(vec![GoogleAnalyticsClient::dimension("eventName")]),
            metrics: vec![
                GoogleAnalyticsClient::metric("eventCount"),
                GoogleAnalyticsClient::metric("activeUsers"),
            ],
            dimension_filter: None,
            metric_filter: None,
            limit: Some(limit),
            metric_aggregations: None,
            order_bys: Some(vec![GoogleAnalyticsClient::order_by_metric_desc("eventCount")]),
            return_property_quota: None,
            minute_ranges: Some(vec![MinuteRange {
                name: None,
                start_minutes_ago: Some(30),
                end_minutes_ago: Some(0),
            }]),
        };

        let response = self.client.run_realtime_report(request).await?;

        let mut events = Vec::new();

        if let Some(rows) = response.rows {
            for row in rows {
                if let (Some(dims), Some(vals)) = (&row.dimension_values, &row.metric_values) {
                    let event_name = dims.get(0).and_then(|d| d.value.clone()).unwrap_or_default();
                    let event_count: u32 = vals
                        .get(0)
                        .and_then(|v| v.value.as_ref())
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                    let users: u32 = vals
                        .get(1)
                        .and_then(|v| v.value.as_ref())
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);

                    events.push(ActiveEvent {
                        event_category: event_name.clone(),
                        event_action: event_name,
                        event_label: None,
                        event_count,
                        users,
                    });
                }
            }
        }

        Ok(events)
    }

    /// Get real-time conversions
    pub async fn get_active_conversions(&self) -> Result<Vec<ActiveConversion>, ClientError> {
        let request = RunRealtimeReportRequest {
            property: format!("properties/{}", self.client.property_id()),
            dimensions: Some(vec![GoogleAnalyticsClient::dimension("eventName")]),
            metrics: vec![
                GoogleAnalyticsClient::metric("conversions"),
                GoogleAnalyticsClient::metric("totalRevenue"),
            ],
            dimension_filter: None,
            metric_filter: Some(GoogleAnalyticsClient::numeric_filter(
                "conversions",
                NumericFilterOperation::GreaterThan,
                0.0,
            )),
            limit: Some(10),
            metric_aggregations: None,
            order_bys: Some(vec![GoogleAnalyticsClient::order_by_metric_desc("conversions")]),
            return_property_quota: None,
            minute_ranges: Some(vec![MinuteRange {
                name: None,
                start_minutes_ago: Some(30),
                end_minutes_ago: Some(0),
            }]),
        };

        let response = self.client.run_realtime_report(request).await?;

        let mut conversions = Vec::new();

        if let Some(rows) = response.rows {
            for row in rows {
                if let (Some(dims), Some(vals)) = (&row.dimension_values, &row.metric_values) {
                    let goal_name = dims.get(0).and_then(|d| d.value.clone()).unwrap_or_default();
                    let completions: u32 = vals
                        .get(0)
                        .and_then(|v| v.value.as_ref())
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                    let value: f64 = vals
                        .get(1)
                        .and_then(|v| v.value.as_ref())
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0.0);

                    conversions.push(ActiveConversion {
                        goal_id: goal_name.clone(),
                        goal_name,
                        completions,
                        value,
                    });
                }
            }
        }

        Ok(conversions)
    }

    /// Get geographic distribution for real-time map
    pub async fn get_geo_distribution(&self) -> Result<RealtimeGeoDistribution, ClientError> {
        let request = RunRealtimeReportRequest {
            property: format!("properties/{}", self.client.property_id()),
            dimensions: Some(vec![
                GoogleAnalyticsClient::dimension("country"),
                GoogleAnalyticsClient::dimension("city"),
            ]),
            metrics: vec![
                GoogleAnalyticsClient::metric("activeUsers"),
                GoogleAnalyticsClient::metric("screenPageViews"),
            ],
            dimension_filter: None,
            metric_filter: None,
            limit: Some(100),
            metric_aggregations: None,
            order_bys: Some(vec![GoogleAnalyticsClient::order_by_metric_desc("activeUsers")]),
            return_property_quota: None,
            minute_ranges: Some(vec![MinuteRange {
                name: None,
                start_minutes_ago: Some(30),
                end_minutes_ago: Some(0),
            }]),
        };

        let response = self.client.run_realtime_report(request).await?;

        let mut countries: std::collections::HashMap<String, CountryActiveUsers> =
            std::collections::HashMap::new();
        let mut cities = Vec::new();
        let mut user_locations = Vec::new();

        if let Some(rows) = response.rows {
            for row in rows {
                if let (Some(dims), Some(vals)) = (&row.dimension_values, &row.metric_values) {
                    let country = dims.get(0).and_then(|d| d.value.clone()).unwrap_or_default();
                    let city = dims.get(1).and_then(|d| d.value.clone()).unwrap_or_default();
                    let active_users: u32 = vals
                        .get(0)
                        .and_then(|v| v.value.as_ref())
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                    let pageviews: u32 = vals
                        .get(1)
                        .and_then(|v| v.value.as_ref())
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);

                    // Aggregate by country
                    countries
                        .entry(country.clone())
                        .and_modify(|c| {
                            c.active_users += active_users;
                            c.pageviews += pageviews;
                        })
                        .or_insert(CountryActiveUsers {
                            country: country.clone(),
                            country_code: String::new(),
                            active_users,
                            pageviews,
                            latitude: 0.0,
                            longitude: 0.0,
                        });

                    // Add city
                    if !city.is_empty() && city != "(not set)" {
                        cities.push(CityActiveUsers {
                            city: city.clone(),
                            country: country.clone(),
                            country_code: String::new(),
                            active_users,
                            latitude: 0.0,
                            longitude: 0.0,
                        });

                        // Add to user locations
                        user_locations.push(UserLocation {
                            latitude: 0.0, // Would need geocoding
                            longitude: 0.0,
                            active_users,
                            city: Some(city),
                            country,
                        });
                    }
                }
            }
        }

        Ok(RealtimeGeoDistribution {
            countries: countries.into_values().collect(),
            cities,
            user_locations,
        })
    }
}

impl std::fmt::Debug for RealtimeService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RealtimeService")
            .field("client", &self.client)
            .finish()
    }
}
