//! Real-time analytics models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Real-time overview data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeOverview {
    pub active_users: u32,
    pub active_users_1min: u32,
    pub active_users_5min: u32,
    pub active_users_10min: u32,
    pub active_users_30min: u32,
    pub pageviews_per_minute: Vec<PageviewsPerMinute>,
    pub pageviews_per_second: Vec<PageviewsPerSecond>,
    pub top_active_pages: Vec<ActivePage>,
    pub top_referrers: Vec<ActiveReferrer>,
    pub top_keywords: Vec<ActiveKeyword>,
    pub top_locations: Vec<ActiveLocation>,
    pub top_traffic_sources: Vec<ActiveTrafficSource>,
    pub top_social_sources: Vec<ActiveSocialSource>,
    pub device_breakdown: DeviceBreakdown,
    pub active_events: Vec<ActiveEvent>,
    pub active_conversions: Vec<ActiveConversion>,
    pub timestamp: DateTime<Utc>,
}

/// Pageviews per minute data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageviewsPerMinute {
    pub minute: u32,
    pub pageviews: u32,
    pub timestamp: DateTime<Utc>,
}

/// Pageviews per second data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageviewsPerSecond {
    pub second: u32,
    pub pageviews: u32,
    pub timestamp: DateTime<Utc>,
}

/// Currently active page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivePage {
    pub page_path: String,
    pub page_title: String,
    pub active_users: u32,
    pub percentage: f64,
}

/// Active referrer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveReferrer {
    pub referrer: String,
    pub active_users: u32,
    pub percentage: f64,
}

/// Active keyword
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveKeyword {
    pub keyword: String,
    pub active_users: u32,
    pub percentage: f64,
}

/// Active location/geo data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveLocation {
    pub country: String,
    pub country_code: String,
    pub region: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub active_users: u32,
    pub percentage: f64,
}

/// Active traffic source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveTrafficSource {
    pub source: String,
    pub medium: String,
    pub active_users: u32,
    pub percentage: f64,
}

/// Active social source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSocialSource {
    pub network: String,
    pub active_users: u32,
    pub percentage: f64,
}

/// Device breakdown for real-time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceBreakdown {
    pub desktop: DeviceStats,
    pub mobile: DeviceStats,
    pub tablet: DeviceStats,
}

/// Stats for a device category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStats {
    pub active_users: u32,
    pub percentage: f64,
}

/// Active event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveEvent {
    pub event_category: String,
    pub event_action: String,
    pub event_label: Option<String>,
    pub event_count: u32,
    pub users: u32,
}

/// Active conversion/goal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveConversion {
    pub goal_id: String,
    pub goal_name: String,
    pub completions: u32,
    pub value: f64,
}

/// Real-time user session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeSession {
    pub session_id: String,
    pub user_id: Option<String>,
    pub client_id: String,
    pub country: String,
    pub city: Option<String>,
    pub device_category: String,
    pub browser: String,
    pub operating_system: String,
    pub current_page: String,
    pub page_title: String,
    pub referrer: Option<String>,
    pub source: String,
    pub medium: String,
    pub campaign: Option<String>,
    pub session_duration: u32,
    pub pageviews: u32,
    pub events: u32,
    pub is_new_user: bool,
    pub started_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

/// Real-time page hit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimePageHit {
    pub hit_id: String,
    pub session_id: String,
    pub page_path: String,
    pub page_title: String,
    pub hostname: String,
    pub referrer: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub load_time: Option<u32>,
    pub country: String,
    pub device_category: String,
}

/// Real-time event hit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeEventHit {
    pub hit_id: String,
    pub session_id: String,
    pub event_category: String,
    pub event_action: String,
    pub event_label: Option<String>,
    pub event_value: Option<f64>,
    pub page_path: String,
    pub timestamp: DateTime<Utc>,
    pub country: String,
    pub device_category: String,
}

/// Real-time traffic data for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTrafficData {
    pub timeline: Vec<TimelineDataPoint>,
    pub current_minute: MinuteData,
    pub last_30_minutes: Last30MinutesData,
}

/// Timeline data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineDataPoint {
    pub timestamp: DateTime<Utc>,
    pub active_users: u32,
    pub pageviews: u32,
    pub events: u32,
}

/// Current minute data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinuteData {
    pub active_users: u32,
    pub pageviews: u32,
    pub new_sessions: u32,
    pub events: u32,
    pub conversions: u32,
}

/// Last 30 minutes aggregated data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Last30MinutesData {
    pub total_pageviews: u32,
    pub total_sessions: u32,
    pub total_events: u32,
    pub total_conversions: u32,
    pub avg_active_users: f64,
    pub peak_active_users: u32,
    pub peak_time: DateTime<Utc>,
}

/// Geographic distribution for real-time map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeGeoDistribution {
    pub countries: Vec<CountryActiveUsers>,
    pub cities: Vec<CityActiveUsers>,
    pub user_locations: Vec<UserLocation>,
}

/// Country-level active users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryActiveUsers {
    pub country: String,
    pub country_code: String,
    pub active_users: u32,
    pub pageviews: u32,
    pub latitude: f64,
    pub longitude: f64,
}

/// City-level active users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityActiveUsers {
    pub city: String,
    pub country: String,
    pub country_code: String,
    pub active_users: u32,
    pub latitude: f64,
    pub longitude: f64,
}

/// Individual user location marker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub active_users: u32,
    pub city: Option<String>,
    pub country: String,
}

/// Real-time content group data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeContentGroup {
    pub content_group: String,
    pub active_users: u32,
    pub pageviews: u32,
    pub percentage: f64,
}

/// Real-time user type breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeUserTypes {
    pub new_users: u32,
    pub new_users_percentage: f64,
    pub returning_users: u32,
    pub returning_users_percentage: f64,
}
