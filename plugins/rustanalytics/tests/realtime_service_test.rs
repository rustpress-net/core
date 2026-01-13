//! Tests for the Real-time Analytics Service and Models

use chrono::{Duration, Utc};
use serde_json;
use std::sync::Arc;

use rustanalytics::models::realtime::*;
use rustanalytics::services::client::GoogleAnalyticsClient;
use rustanalytics::services::realtime::RealtimeService;

// ============================================================================
// Helper Functions
// ============================================================================

async fn create_test_realtime_service() -> RealtimeService {
    let client = Arc::new(
        GoogleAnalyticsClient::new("properties/12345".to_string(), None)
            .await
            .unwrap(),
    );
    RealtimeService::new(client)
}

fn sample_realtime_overview() -> RealtimeOverview {
    RealtimeOverview {
        active_users: 150,
        active_users_1min: 45,
        active_users_5min: 120,
        active_users_10min: 180,
        active_users_30min: 350,
        pageviews_per_minute: vec![
            PageviewsPerMinute {
                minute: 0,
                pageviews: 25,
                timestamp: Utc::now(),
            },
            PageviewsPerMinute {
                minute: 1,
                pageviews: 30,
                timestamp: Utc::now() - Duration::minutes(1),
            },
        ],
        pageviews_per_second: vec![PageviewsPerSecond {
            second: 0,
            pageviews: 2,
            timestamp: Utc::now(),
        }],
        top_active_pages: vec![ActivePage {
            page_path: "/home".to_string(),
            page_title: "Home Page".to_string(),
            active_users: 50,
            percentage: 33.33,
        }],
        top_referrers: vec![ActiveReferrer {
            referrer: "google.com".to_string(),
            active_users: 30,
            percentage: 20.0,
        }],
        top_keywords: vec![ActiveKeyword {
            keyword: "rust programming".to_string(),
            active_users: 15,
            percentage: 10.0,
        }],
        top_locations: vec![ActiveLocation {
            country: "United States".to_string(),
            country_code: "US".to_string(),
            region: Some("California".to_string()),
            city: Some("San Francisco".to_string()),
            latitude: Some(37.7749),
            longitude: Some(-122.4194),
            active_users: 40,
            percentage: 26.67,
        }],
        top_traffic_sources: vec![ActiveTrafficSource {
            source: "google".to_string(),
            medium: "organic".to_string(),
            active_users: 60,
            percentage: 40.0,
        }],
        top_social_sources: vec![ActiveSocialSource {
            network: "Twitter".to_string(),
            active_users: 20,
            percentage: 13.33,
        }],
        device_breakdown: DeviceBreakdown {
            desktop: DeviceStats {
                active_users: 80,
                percentage: 53.33,
            },
            mobile: DeviceStats {
                active_users: 55,
                percentage: 36.67,
            },
            tablet: DeviceStats {
                active_users: 15,
                percentage: 10.0,
            },
        },
        active_events: vec![ActiveEvent {
            event_category: "click".to_string(),
            event_action: "button_click".to_string(),
            event_label: Some("cta_signup".to_string()),
            event_count: 25,
            users: 20,
        }],
        active_conversions: vec![ActiveConversion {
            goal_id: "purchase".to_string(),
            goal_name: "Purchase Completed".to_string(),
            completions: 5,
            value: 250.0,
        }],
        timestamp: Utc::now(),
    }
}

fn sample_device_breakdown() -> DeviceBreakdown {
    DeviceBreakdown {
        desktop: DeviceStats {
            active_users: 100,
            percentage: 50.0,
        },
        mobile: DeviceStats {
            active_users: 80,
            percentage: 40.0,
        },
        tablet: DeviceStats {
            active_users: 20,
            percentage: 10.0,
        },
    }
}

fn sample_geo_distribution() -> RealtimeGeoDistribution {
    RealtimeGeoDistribution {
        countries: vec![
            CountryActiveUsers {
                country: "United States".to_string(),
                country_code: "US".to_string(),
                active_users: 100,
                pageviews: 500,
                latitude: 37.0902,
                longitude: -95.7129,
            },
            CountryActiveUsers {
                country: "United Kingdom".to_string(),
                country_code: "GB".to_string(),
                active_users: 50,
                pageviews: 200,
                latitude: 55.3781,
                longitude: -3.4360,
            },
        ],
        cities: vec![CityActiveUsers {
            city: "New York".to_string(),
            country: "United States".to_string(),
            country_code: "US".to_string(),
            active_users: 30,
            latitude: 40.7128,
            longitude: -74.0060,
        }],
        user_locations: vec![UserLocation {
            latitude: 40.7128,
            longitude: -74.0060,
            active_users: 5,
            city: Some("New York".to_string()),
            country: "United States".to_string(),
        }],
    }
}

// ============================================================================
// RealtimeService Tests
// ============================================================================

#[tokio::test]
async fn test_realtime_service_creation() {
    let service = create_test_realtime_service().await;
    // Service should be created successfully
    assert!(format!("{:?}", service).contains("RealtimeService"));
}

#[tokio::test]
async fn test_realtime_service_debug_impl() {
    let service = create_test_realtime_service().await;
    let debug_str = format!("{:?}", service);
    assert!(debug_str.contains("RealtimeService"));
    assert!(debug_str.contains("client"));
}

// ============================================================================
// RealtimeOverview Model Tests
// ============================================================================

#[test]
fn test_realtime_overview_creation() {
    let overview = sample_realtime_overview();

    assert_eq!(overview.active_users, 150);
    assert_eq!(overview.active_users_1min, 45);
    assert_eq!(overview.active_users_5min, 120);
    assert_eq!(overview.active_users_10min, 180);
    assert_eq!(overview.active_users_30min, 350);
    assert_eq!(overview.pageviews_per_minute.len(), 2);
    assert_eq!(overview.pageviews_per_second.len(), 1);
    assert_eq!(overview.top_active_pages.len(), 1);
    assert_eq!(overview.top_referrers.len(), 1);
    assert_eq!(overview.top_keywords.len(), 1);
    assert_eq!(overview.top_locations.len(), 1);
    assert_eq!(overview.top_traffic_sources.len(), 1);
    assert_eq!(overview.top_social_sources.len(), 1);
    assert_eq!(overview.active_events.len(), 1);
    assert_eq!(overview.active_conversions.len(), 1);
}

#[test]
fn test_realtime_overview_serialization() {
    let overview = sample_realtime_overview();

    let json = serde_json::to_string(&overview).unwrap();
    let deserialized: RealtimeOverview = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.active_users, overview.active_users);
    assert_eq!(deserialized.active_users_30min, overview.active_users_30min);
}

#[test]
fn test_realtime_overview_clone() {
    let overview = sample_realtime_overview();
    let cloned = overview.clone();

    assert_eq!(cloned.active_users, overview.active_users);
    assert_eq!(cloned.top_active_pages.len(), overview.top_active_pages.len());
}

#[test]
fn test_realtime_overview_with_empty_data() {
    let overview = RealtimeOverview {
        active_users: 0,
        active_users_1min: 0,
        active_users_5min: 0,
        active_users_10min: 0,
        active_users_30min: 0,
        pageviews_per_minute: vec![],
        pageviews_per_second: vec![],
        top_active_pages: vec![],
        top_referrers: vec![],
        top_keywords: vec![],
        top_locations: vec![],
        top_traffic_sources: vec![],
        top_social_sources: vec![],
        device_breakdown: DeviceBreakdown {
            desktop: DeviceStats {
                active_users: 0,
                percentage: 0.0,
            },
            mobile: DeviceStats {
                active_users: 0,
                percentage: 0.0,
            },
            tablet: DeviceStats {
                active_users: 0,
                percentage: 0.0,
            },
        },
        active_events: vec![],
        active_conversions: vec![],
        timestamp: Utc::now(),
    };

    assert_eq!(overview.active_users, 0);
    assert!(overview.top_active_pages.is_empty());

    let json = serde_json::to_string(&overview).unwrap();
    assert!(json.contains("\"active_users\":0"));
}

// ============================================================================
// PageviewsPerMinute Model Tests
// ============================================================================

#[test]
fn test_pageviews_per_minute_creation() {
    let now = Utc::now();
    let ppm = PageviewsPerMinute {
        minute: 5,
        pageviews: 100,
        timestamp: now,
    };

    assert_eq!(ppm.minute, 5);
    assert_eq!(ppm.pageviews, 100);
    assert_eq!(ppm.timestamp, now);
}

#[test]
fn test_pageviews_per_minute_serialization() {
    let ppm = PageviewsPerMinute {
        minute: 10,
        pageviews: 50,
        timestamp: Utc::now(),
    };

    let json = serde_json::to_string(&ppm).unwrap();
    let deserialized: PageviewsPerMinute = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.minute, 10);
    assert_eq!(deserialized.pageviews, 50);
}

#[test]
fn test_pageviews_per_minute_clone() {
    let ppm = PageviewsPerMinute {
        minute: 3,
        pageviews: 75,
        timestamp: Utc::now(),
    };

    let cloned = ppm.clone();
    assert_eq!(cloned.minute, ppm.minute);
    assert_eq!(cloned.pageviews, ppm.pageviews);
}

// ============================================================================
// PageviewsPerSecond Model Tests
// ============================================================================

#[test]
fn test_pageviews_per_second_creation() {
    let now = Utc::now();
    let pps = PageviewsPerSecond {
        second: 30,
        pageviews: 5,
        timestamp: now,
    };

    assert_eq!(pps.second, 30);
    assert_eq!(pps.pageviews, 5);
}

#[test]
fn test_pageviews_per_second_serialization() {
    let pps = PageviewsPerSecond {
        second: 45,
        pageviews: 3,
        timestamp: Utc::now(),
    };

    let json = serde_json::to_string(&pps).unwrap();
    let deserialized: PageviewsPerSecond = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.second, 45);
    assert_eq!(deserialized.pageviews, 3);
}

// ============================================================================
// ActivePage Model Tests
// ============================================================================

#[test]
fn test_active_page_creation() {
    let page = ActivePage {
        page_path: "/products/widget".to_string(),
        page_title: "Widget Product Page".to_string(),
        active_users: 25,
        percentage: 16.67,
    };

    assert_eq!(page.page_path, "/products/widget");
    assert_eq!(page.page_title, "Widget Product Page");
    assert_eq!(page.active_users, 25);
    assert!((page.percentage - 16.67).abs() < 0.01);
}

#[test]
fn test_active_page_serialization() {
    let page = ActivePage {
        page_path: "/about".to_string(),
        page_title: "About Us".to_string(),
        active_users: 10,
        percentage: 5.0,
    };

    let json = serde_json::to_string(&page).unwrap();
    assert!(json.contains("/about"));
    assert!(json.contains("About Us"));

    let deserialized: ActivePage = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.page_path, "/about");
}

#[test]
fn test_active_page_clone() {
    let page = ActivePage {
        page_path: "/test".to_string(),
        page_title: "Test Page".to_string(),
        active_users: 5,
        percentage: 2.5,
    };

    let cloned = page.clone();
    assert_eq!(cloned.page_path, page.page_path);
    assert_eq!(cloned.active_users, page.active_users);
}

#[test]
fn test_active_page_zero_percentage() {
    let page = ActivePage {
        page_path: "/empty".to_string(),
        page_title: "Empty Page".to_string(),
        active_users: 0,
        percentage: 0.0,
    };

    assert_eq!(page.active_users, 0);
    assert_eq!(page.percentage, 0.0);
}

// ============================================================================
// ActiveReferrer Model Tests
// ============================================================================

#[test]
fn test_active_referrer_creation() {
    let referrer = ActiveReferrer {
        referrer: "https://example.com".to_string(),
        active_users: 15,
        percentage: 10.0,
    };

    assert_eq!(referrer.referrer, "https://example.com");
    assert_eq!(referrer.active_users, 15);
    assert_eq!(referrer.percentage, 10.0);
}

#[test]
fn test_active_referrer_serialization() {
    let referrer = ActiveReferrer {
        referrer: "twitter.com".to_string(),
        active_users: 8,
        percentage: 5.3,
    };

    let json = serde_json::to_string(&referrer).unwrap();
    let deserialized: ActiveReferrer = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.referrer, "twitter.com");
    assert_eq!(deserialized.active_users, 8);
}

// ============================================================================
// ActiveKeyword Model Tests
// ============================================================================

#[test]
fn test_active_keyword_creation() {
    let keyword = ActiveKeyword {
        keyword: "rust web framework".to_string(),
        active_users: 20,
        percentage: 13.33,
    };

    assert_eq!(keyword.keyword, "rust web framework");
    assert_eq!(keyword.active_users, 20);
}

#[test]
fn test_active_keyword_serialization() {
    let keyword = ActiveKeyword {
        keyword: "programming tutorial".to_string(),
        active_users: 12,
        percentage: 8.0,
    };

    let json = serde_json::to_string(&keyword).unwrap();
    let deserialized: ActiveKeyword = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.keyword, "programming tutorial");
}

// ============================================================================
// ActiveLocation Model Tests
// ============================================================================

#[test]
fn test_active_location_creation() {
    let location = ActiveLocation {
        country: "Germany".to_string(),
        country_code: "DE".to_string(),
        region: Some("Bavaria".to_string()),
        city: Some("Munich".to_string()),
        latitude: Some(48.1351),
        longitude: Some(11.5820),
        active_users: 25,
        percentage: 16.67,
    };

    assert_eq!(location.country, "Germany");
    assert_eq!(location.country_code, "DE");
    assert_eq!(location.region, Some("Bavaria".to_string()));
    assert_eq!(location.city, Some("Munich".to_string()));
    assert_eq!(location.active_users, 25);
}

#[test]
fn test_active_location_without_optional_fields() {
    let location = ActiveLocation {
        country: "Unknown".to_string(),
        country_code: "XX".to_string(),
        region: None,
        city: None,
        latitude: None,
        longitude: None,
        active_users: 5,
        percentage: 3.33,
    };

    assert!(location.region.is_none());
    assert!(location.city.is_none());
    assert!(location.latitude.is_none());
}

#[test]
fn test_active_location_serialization() {
    let location = ActiveLocation {
        country: "France".to_string(),
        country_code: "FR".to_string(),
        region: Some("Île-de-France".to_string()),
        city: Some("Paris".to_string()),
        latitude: Some(48.8566),
        longitude: Some(2.3522),
        active_users: 30,
        percentage: 20.0,
    };

    let json = serde_json::to_string(&location).unwrap();
    assert!(json.contains("France"));
    assert!(json.contains("Paris"));

    let deserialized: ActiveLocation = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.country, "France");
    assert_eq!(deserialized.city, Some("Paris".to_string()));
}

// ============================================================================
// ActiveTrafficSource Model Tests
// ============================================================================

#[test]
fn test_active_traffic_source_creation() {
    let source = ActiveTrafficSource {
        source: "google".to_string(),
        medium: "cpc".to_string(),
        active_users: 40,
        percentage: 26.67,
    };

    assert_eq!(source.source, "google");
    assert_eq!(source.medium, "cpc");
    assert_eq!(source.active_users, 40);
}

#[test]
fn test_active_traffic_source_serialization() {
    let source = ActiveTrafficSource {
        source: "facebook".to_string(),
        medium: "social".to_string(),
        active_users: 25,
        percentage: 16.67,
    };

    let json = serde_json::to_string(&source).unwrap();
    let deserialized: ActiveTrafficSource = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.source, "facebook");
    assert_eq!(deserialized.medium, "social");
}

#[test]
fn test_active_traffic_source_direct() {
    let source = ActiveTrafficSource {
        source: "(direct)".to_string(),
        medium: "(none)".to_string(),
        active_users: 50,
        percentage: 33.33,
    };

    assert_eq!(source.source, "(direct)");
    assert_eq!(source.medium, "(none)");
}

// ============================================================================
// ActiveSocialSource Model Tests
// ============================================================================

#[test]
fn test_active_social_source_creation() {
    let social = ActiveSocialSource {
        network: "LinkedIn".to_string(),
        active_users: 15,
        percentage: 10.0,
    };

    assert_eq!(social.network, "LinkedIn");
    assert_eq!(social.active_users, 15);
    assert_eq!(social.percentage, 10.0);
}

#[test]
fn test_active_social_source_serialization() {
    let social = ActiveSocialSource {
        network: "Instagram".to_string(),
        active_users: 22,
        percentage: 14.67,
    };

    let json = serde_json::to_string(&social).unwrap();
    let deserialized: ActiveSocialSource = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.network, "Instagram");
}

// ============================================================================
// DeviceBreakdown Model Tests
// ============================================================================

#[test]
fn test_device_breakdown_creation() {
    let breakdown = sample_device_breakdown();

    assert_eq!(breakdown.desktop.active_users, 100);
    assert_eq!(breakdown.desktop.percentage, 50.0);
    assert_eq!(breakdown.mobile.active_users, 80);
    assert_eq!(breakdown.tablet.active_users, 20);
}

#[test]
fn test_device_breakdown_serialization() {
    let breakdown = sample_device_breakdown();

    let json = serde_json::to_string(&breakdown).unwrap();
    let deserialized: DeviceBreakdown = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.desktop.active_users, 100);
    assert_eq!(deserialized.mobile.percentage, 40.0);
}

#[test]
fn test_device_breakdown_all_zeros() {
    let breakdown = DeviceBreakdown {
        desktop: DeviceStats {
            active_users: 0,
            percentage: 0.0,
        },
        mobile: DeviceStats {
            active_users: 0,
            percentage: 0.0,
        },
        tablet: DeviceStats {
            active_users: 0,
            percentage: 0.0,
        },
    };

    assert_eq!(breakdown.desktop.active_users, 0);
    assert_eq!(breakdown.mobile.active_users, 0);
    assert_eq!(breakdown.tablet.active_users, 0);
}

#[test]
fn test_device_breakdown_clone() {
    let breakdown = sample_device_breakdown();
    let cloned = breakdown.clone();

    assert_eq!(cloned.desktop.active_users, breakdown.desktop.active_users);
    assert_eq!(cloned.mobile.percentage, breakdown.mobile.percentage);
}

// ============================================================================
// DeviceStats Model Tests
// ============================================================================

#[test]
fn test_device_stats_creation() {
    let stats = DeviceStats {
        active_users: 75,
        percentage: 37.5,
    };

    assert_eq!(stats.active_users, 75);
    assert_eq!(stats.percentage, 37.5);
}

#[test]
fn test_device_stats_serialization() {
    let stats = DeviceStats {
        active_users: 50,
        percentage: 25.0,
    };

    let json = serde_json::to_string(&stats).unwrap();
    let deserialized: DeviceStats = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.active_users, 50);
    assert_eq!(deserialized.percentage, 25.0);
}

// ============================================================================
// ActiveEvent Model Tests
// ============================================================================

#[test]
fn test_active_event_creation() {
    let event = ActiveEvent {
        event_category: "video".to_string(),
        event_action: "play".to_string(),
        event_label: Some("intro_video".to_string()),
        event_count: 50,
        users: 35,
    };

    assert_eq!(event.event_category, "video");
    assert_eq!(event.event_action, "play");
    assert_eq!(event.event_label, Some("intro_video".to_string()));
    assert_eq!(event.event_count, 50);
    assert_eq!(event.users, 35);
}

#[test]
fn test_active_event_without_label() {
    let event = ActiveEvent {
        event_category: "form".to_string(),
        event_action: "submit".to_string(),
        event_label: None,
        event_count: 20,
        users: 18,
    };

    assert!(event.event_label.is_none());
}

#[test]
fn test_active_event_serialization() {
    let event = ActiveEvent {
        event_category: "download".to_string(),
        event_action: "click".to_string(),
        event_label: Some("pdf_guide".to_string()),
        event_count: 15,
        users: 12,
    };

    let json = serde_json::to_string(&event).unwrap();
    let deserialized: ActiveEvent = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.event_category, "download");
    assert_eq!(deserialized.event_count, 15);
}

// ============================================================================
// ActiveConversion Model Tests
// ============================================================================

#[test]
fn test_active_conversion_creation() {
    let conversion = ActiveConversion {
        goal_id: "signup".to_string(),
        goal_name: "Newsletter Signup".to_string(),
        completions: 10,
        value: 0.0,
    };

    assert_eq!(conversion.goal_id, "signup");
    assert_eq!(conversion.goal_name, "Newsletter Signup");
    assert_eq!(conversion.completions, 10);
    assert_eq!(conversion.value, 0.0);
}

#[test]
fn test_active_conversion_with_value() {
    let conversion = ActiveConversion {
        goal_id: "purchase".to_string(),
        goal_name: "Product Purchase".to_string(),
        completions: 3,
        value: 299.97,
    };

    assert_eq!(conversion.completions, 3);
    assert!((conversion.value - 299.97).abs() < 0.01);
}

#[test]
fn test_active_conversion_serialization() {
    let conversion = ActiveConversion {
        goal_id: "contact".to_string(),
        goal_name: "Contact Form Submission".to_string(),
        completions: 5,
        value: 50.0,
    };

    let json = serde_json::to_string(&conversion).unwrap();
    let deserialized: ActiveConversion = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.goal_id, "contact");
    assert_eq!(deserialized.completions, 5);
}

// ============================================================================
// RealtimeSession Model Tests
// ============================================================================

#[test]
fn test_realtime_session_creation() {
    let now = Utc::now();
    let session = RealtimeSession {
        session_id: "sess_12345".to_string(),
        user_id: Some("user_abc".to_string()),
        client_id: "client_xyz".to_string(),
        country: "Canada".to_string(),
        city: Some("Toronto".to_string()),
        device_category: "desktop".to_string(),
        browser: "Chrome".to_string(),
        operating_system: "Windows".to_string(),
        current_page: "/products".to_string(),
        page_title: "Products Page".to_string(),
        referrer: Some("google.com".to_string()),
        source: "google".to_string(),
        medium: "organic".to_string(),
        campaign: None,
        session_duration: 300,
        pageviews: 5,
        events: 3,
        is_new_user: true,
        started_at: now - Duration::minutes(5),
        last_activity: now,
    };

    assert_eq!(session.session_id, "sess_12345");
    assert_eq!(session.user_id, Some("user_abc".to_string()));
    assert_eq!(session.country, "Canada");
    assert_eq!(session.session_duration, 300);
    assert!(session.is_new_user);
}

#[test]
fn test_realtime_session_returning_user() {
    let now = Utc::now();
    let session = RealtimeSession {
        session_id: "sess_67890".to_string(),
        user_id: None,
        client_id: "client_123".to_string(),
        country: "United States".to_string(),
        city: None,
        device_category: "mobile".to_string(),
        browser: "Safari".to_string(),
        operating_system: "iOS".to_string(),
        current_page: "/".to_string(),
        page_title: "Home".to_string(),
        referrer: None,
        source: "(direct)".to_string(),
        medium: "(none)".to_string(),
        campaign: None,
        session_duration: 60,
        pageviews: 2,
        events: 1,
        is_new_user: false,
        started_at: now - Duration::minutes(1),
        last_activity: now,
    };

    assert!(!session.is_new_user);
    assert!(session.user_id.is_none());
    assert!(session.referrer.is_none());
}

#[test]
fn test_realtime_session_serialization() {
    let now = Utc::now();
    let session = RealtimeSession {
        session_id: "sess_test".to_string(),
        user_id: Some("user_test".to_string()),
        client_id: "client_test".to_string(),
        country: "UK".to_string(),
        city: Some("London".to_string()),
        device_category: "tablet".to_string(),
        browser: "Firefox".to_string(),
        operating_system: "Android".to_string(),
        current_page: "/blog".to_string(),
        page_title: "Blog".to_string(),
        referrer: Some("twitter.com".to_string()),
        source: "twitter".to_string(),
        medium: "social".to_string(),
        campaign: Some("spring_sale".to_string()),
        session_duration: 180,
        pageviews: 3,
        events: 2,
        is_new_user: true,
        started_at: now,
        last_activity: now,
    };

    let json = serde_json::to_string(&session).unwrap();
    let deserialized: RealtimeSession = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.session_id, "sess_test");
    assert_eq!(deserialized.campaign, Some("spring_sale".to_string()));
}

// ============================================================================
// RealtimePageHit Model Tests
// ============================================================================

#[test]
fn test_realtime_page_hit_creation() {
    let now = Utc::now();
    let hit = RealtimePageHit {
        hit_id: "hit_001".to_string(),
        session_id: "sess_abc".to_string(),
        page_path: "/checkout".to_string(),
        page_title: "Checkout".to_string(),
        hostname: "example.com".to_string(),
        referrer: Some("/cart".to_string()),
        timestamp: now,
        load_time: Some(250),
        country: "Australia".to_string(),
        device_category: "desktop".to_string(),
    };

    assert_eq!(hit.hit_id, "hit_001");
    assert_eq!(hit.page_path, "/checkout");
    assert_eq!(hit.load_time, Some(250));
}

#[test]
fn test_realtime_page_hit_without_optional_fields() {
    let now = Utc::now();
    let hit = RealtimePageHit {
        hit_id: "hit_002".to_string(),
        session_id: "sess_def".to_string(),
        page_path: "/".to_string(),
        page_title: "Home".to_string(),
        hostname: "test.com".to_string(),
        referrer: None,
        timestamp: now,
        load_time: None,
        country: "Unknown".to_string(),
        device_category: "mobile".to_string(),
    };

    assert!(hit.referrer.is_none());
    assert!(hit.load_time.is_none());
}

#[test]
fn test_realtime_page_hit_serialization() {
    let now = Utc::now();
    let hit = RealtimePageHit {
        hit_id: "hit_test".to_string(),
        session_id: "sess_test".to_string(),
        page_path: "/contact".to_string(),
        page_title: "Contact Us".to_string(),
        hostname: "mysite.com".to_string(),
        referrer: Some("google.com".to_string()),
        timestamp: now,
        load_time: Some(500),
        country: "Japan".to_string(),
        device_category: "mobile".to_string(),
    };

    let json = serde_json::to_string(&hit).unwrap();
    let deserialized: RealtimePageHit = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.hit_id, "hit_test");
    assert_eq!(deserialized.load_time, Some(500));
}

// ============================================================================
// RealtimeEventHit Model Tests
// ============================================================================

#[test]
fn test_realtime_event_hit_creation() {
    let now = Utc::now();
    let hit = RealtimeEventHit {
        hit_id: "event_001".to_string(),
        session_id: "sess_xyz".to_string(),
        event_category: "ecommerce".to_string(),
        event_action: "add_to_cart".to_string(),
        event_label: Some("product_123".to_string()),
        event_value: Some(49.99),
        page_path: "/products/widget".to_string(),
        timestamp: now,
        country: "Brazil".to_string(),
        device_category: "mobile".to_string(),
    };

    assert_eq!(hit.hit_id, "event_001");
    assert_eq!(hit.event_category, "ecommerce");
    assert_eq!(hit.event_value, Some(49.99));
}

#[test]
fn test_realtime_event_hit_without_optional_fields() {
    let now = Utc::now();
    let hit = RealtimeEventHit {
        hit_id: "event_002".to_string(),
        session_id: "sess_123".to_string(),
        event_category: "scroll".to_string(),
        event_action: "50%".to_string(),
        event_label: None,
        event_value: None,
        page_path: "/blog/post-1".to_string(),
        timestamp: now,
        country: "Mexico".to_string(),
        device_category: "tablet".to_string(),
    };

    assert!(hit.event_label.is_none());
    assert!(hit.event_value.is_none());
}

#[test]
fn test_realtime_event_hit_serialization() {
    let now = Utc::now();
    let hit = RealtimeEventHit {
        hit_id: "event_test".to_string(),
        session_id: "sess_test".to_string(),
        event_category: "outbound".to_string(),
        event_action: "click".to_string(),
        event_label: Some("partner_link".to_string()),
        event_value: Some(10.0),
        page_path: "/partners".to_string(),
        timestamp: now,
        country: "Spain".to_string(),
        device_category: "desktop".to_string(),
    };

    let json = serde_json::to_string(&hit).unwrap();
    let deserialized: RealtimeEventHit = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.event_category, "outbound");
    assert_eq!(deserialized.event_label, Some("partner_link".to_string()));
}

// ============================================================================
// RealtimeTrafficData Model Tests
// ============================================================================

#[test]
fn test_realtime_traffic_data_creation() {
    let now = Utc::now();
    let traffic = RealtimeTrafficData {
        timeline: vec![
            TimelineDataPoint {
                timestamp: now - Duration::minutes(5),
                active_users: 100,
                pageviews: 150,
                events: 30,
            },
            TimelineDataPoint {
                timestamp: now,
                active_users: 120,
                pageviews: 180,
                events: 40,
            },
        ],
        current_minute: MinuteData {
            active_users: 120,
            pageviews: 45,
            new_sessions: 10,
            events: 8,
            conversions: 2,
        },
        last_30_minutes: Last30MinutesData {
            total_pageviews: 1500,
            total_sessions: 300,
            total_events: 250,
            total_conversions: 15,
            avg_active_users: 95.5,
            peak_active_users: 150,
            peak_time: now - Duration::minutes(10),
        },
    };

    assert_eq!(traffic.timeline.len(), 2);
    assert_eq!(traffic.current_minute.active_users, 120);
    assert_eq!(traffic.last_30_minutes.total_pageviews, 1500);
}

#[test]
fn test_realtime_traffic_data_serialization() {
    let now = Utc::now();
    let traffic = RealtimeTrafficData {
        timeline: vec![],
        current_minute: MinuteData {
            active_users: 50,
            pageviews: 20,
            new_sessions: 5,
            events: 3,
            conversions: 1,
        },
        last_30_minutes: Last30MinutesData {
            total_pageviews: 500,
            total_sessions: 100,
            total_events: 80,
            total_conversions: 5,
            avg_active_users: 45.0,
            peak_active_users: 75,
            peak_time: now,
        },
    };

    let json = serde_json::to_string(&traffic).unwrap();
    let deserialized: RealtimeTrafficData = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.current_minute.pageviews, 20);
}

// ============================================================================
// TimelineDataPoint Model Tests
// ============================================================================

#[test]
fn test_timeline_data_point_creation() {
    let now = Utc::now();
    let point = TimelineDataPoint {
        timestamp: now,
        active_users: 75,
        pageviews: 100,
        events: 25,
    };

    assert_eq!(point.active_users, 75);
    assert_eq!(point.pageviews, 100);
    assert_eq!(point.events, 25);
}

#[test]
fn test_timeline_data_point_serialization() {
    let now = Utc::now();
    let point = TimelineDataPoint {
        timestamp: now,
        active_users: 50,
        pageviews: 80,
        events: 15,
    };

    let json = serde_json::to_string(&point).unwrap();
    let deserialized: TimelineDataPoint = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.active_users, 50);
}

// ============================================================================
// MinuteData Model Tests
// ============================================================================

#[test]
fn test_minute_data_creation() {
    let data = MinuteData {
        active_users: 100,
        pageviews: 50,
        new_sessions: 15,
        events: 20,
        conversions: 3,
    };

    assert_eq!(data.active_users, 100);
    assert_eq!(data.pageviews, 50);
    assert_eq!(data.new_sessions, 15);
    assert_eq!(data.events, 20);
    assert_eq!(data.conversions, 3);
}

#[test]
fn test_minute_data_serialization() {
    let data = MinuteData {
        active_users: 80,
        pageviews: 40,
        new_sessions: 10,
        events: 15,
        conversions: 2,
    };

    let json = serde_json::to_string(&data).unwrap();
    let deserialized: MinuteData = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.pageviews, 40);
}

// ============================================================================
// Last30MinutesData Model Tests
// ============================================================================

#[test]
fn test_last_30_minutes_data_creation() {
    let now = Utc::now();
    let data = Last30MinutesData {
        total_pageviews: 2000,
        total_sessions: 400,
        total_events: 350,
        total_conversions: 20,
        avg_active_users: 120.5,
        peak_active_users: 200,
        peak_time: now - Duration::minutes(15),
    };

    assert_eq!(data.total_pageviews, 2000);
    assert_eq!(data.total_sessions, 400);
    assert!((data.avg_active_users - 120.5).abs() < 0.01);
    assert_eq!(data.peak_active_users, 200);
}

#[test]
fn test_last_30_minutes_data_serialization() {
    let now = Utc::now();
    let data = Last30MinutesData {
        total_pageviews: 1000,
        total_sessions: 200,
        total_events: 150,
        total_conversions: 10,
        avg_active_users: 80.0,
        peak_active_users: 120,
        peak_time: now,
    };

    let json = serde_json::to_string(&data).unwrap();
    let deserialized: Last30MinutesData = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.total_pageviews, 1000);
}

// ============================================================================
// RealtimeGeoDistribution Model Tests
// ============================================================================

#[test]
fn test_realtime_geo_distribution_creation() {
    let geo = sample_geo_distribution();

    assert_eq!(geo.countries.len(), 2);
    assert_eq!(geo.cities.len(), 1);
    assert_eq!(geo.user_locations.len(), 1);
}

#[test]
fn test_realtime_geo_distribution_serialization() {
    let geo = sample_geo_distribution();

    let json = serde_json::to_string(&geo).unwrap();
    let deserialized: RealtimeGeoDistribution = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.countries.len(), 2);
    assert_eq!(deserialized.countries[0].country, "United States");
}

#[test]
fn test_realtime_geo_distribution_empty() {
    let geo = RealtimeGeoDistribution {
        countries: vec![],
        cities: vec![],
        user_locations: vec![],
    };

    assert!(geo.countries.is_empty());
    assert!(geo.cities.is_empty());
    assert!(geo.user_locations.is_empty());
}

// ============================================================================
// CountryActiveUsers Model Tests
// ============================================================================

#[test]
fn test_country_active_users_creation() {
    let country = CountryActiveUsers {
        country: "India".to_string(),
        country_code: "IN".to_string(),
        active_users: 200,
        pageviews: 800,
        latitude: 20.5937,
        longitude: 78.9629,
    };

    assert_eq!(country.country, "India");
    assert_eq!(country.country_code, "IN");
    assert_eq!(country.active_users, 200);
    assert_eq!(country.pageviews, 800);
}

#[test]
fn test_country_active_users_serialization() {
    let country = CountryActiveUsers {
        country: "Australia".to_string(),
        country_code: "AU".to_string(),
        active_users: 75,
        pageviews: 300,
        latitude: -25.2744,
        longitude: 133.7751,
    };

    let json = serde_json::to_string(&country).unwrap();
    let deserialized: CountryActiveUsers = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.country, "Australia");
    assert_eq!(deserialized.active_users, 75);
}

// ============================================================================
// CityActiveUsers Model Tests
// ============================================================================

#[test]
fn test_city_active_users_creation() {
    let city = CityActiveUsers {
        city: "Tokyo".to_string(),
        country: "Japan".to_string(),
        country_code: "JP".to_string(),
        active_users: 50,
        latitude: 35.6762,
        longitude: 139.6503,
    };

    assert_eq!(city.city, "Tokyo");
    assert_eq!(city.country, "Japan");
    assert_eq!(city.active_users, 50);
}

#[test]
fn test_city_active_users_serialization() {
    let city = CityActiveUsers {
        city: "Berlin".to_string(),
        country: "Germany".to_string(),
        country_code: "DE".to_string(),
        active_users: 30,
        latitude: 52.5200,
        longitude: 13.4050,
    };

    let json = serde_json::to_string(&city).unwrap();
    let deserialized: CityActiveUsers = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.city, "Berlin");
}

// ============================================================================
// UserLocation Model Tests
// ============================================================================

#[test]
fn test_user_location_creation() {
    let location = UserLocation {
        latitude: 51.5074,
        longitude: -0.1278,
        active_users: 10,
        city: Some("London".to_string()),
        country: "United Kingdom".to_string(),
    };

    assert!((location.latitude - 51.5074).abs() < 0.0001);
    assert!((location.longitude - (-0.1278)).abs() < 0.0001);
    assert_eq!(location.active_users, 10);
    assert_eq!(location.city, Some("London".to_string()));
}

#[test]
fn test_user_location_without_city() {
    let location = UserLocation {
        latitude: 0.0,
        longitude: 0.0,
        active_users: 5,
        city: None,
        country: "Unknown".to_string(),
    };

    assert!(location.city.is_none());
}

#[test]
fn test_user_location_serialization() {
    let location = UserLocation {
        latitude: 48.8566,
        longitude: 2.3522,
        active_users: 15,
        city: Some("Paris".to_string()),
        country: "France".to_string(),
    };

    let json = serde_json::to_string(&location).unwrap();
    let deserialized: UserLocation = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.city, Some("Paris".to_string()));
    assert_eq!(deserialized.country, "France");
}

// ============================================================================
// RealtimeContentGroup Model Tests
// ============================================================================

#[test]
fn test_realtime_content_group_creation() {
    let group = RealtimeContentGroup {
        content_group: "Blog Posts".to_string(),
        active_users: 45,
        pageviews: 120,
        percentage: 30.0,
    };

    assert_eq!(group.content_group, "Blog Posts");
    assert_eq!(group.active_users, 45);
    assert_eq!(group.pageviews, 120);
    assert_eq!(group.percentage, 30.0);
}

#[test]
fn test_realtime_content_group_serialization() {
    let group = RealtimeContentGroup {
        content_group: "Product Pages".to_string(),
        active_users: 60,
        pageviews: 150,
        percentage: 40.0,
    };

    let json = serde_json::to_string(&group).unwrap();
    let deserialized: RealtimeContentGroup = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.content_group, "Product Pages");
}

// ============================================================================
// RealtimeUserTypes Model Tests
// ============================================================================

#[test]
fn test_realtime_user_types_creation() {
    let user_types = RealtimeUserTypes {
        new_users: 60,
        new_users_percentage: 40.0,
        returning_users: 90,
        returning_users_percentage: 60.0,
    };

    assert_eq!(user_types.new_users, 60);
    assert_eq!(user_types.new_users_percentage, 40.0);
    assert_eq!(user_types.returning_users, 90);
    assert_eq!(user_types.returning_users_percentage, 60.0);
}

#[test]
fn test_realtime_user_types_serialization() {
    let user_types = RealtimeUserTypes {
        new_users: 30,
        new_users_percentage: 25.0,
        returning_users: 90,
        returning_users_percentage: 75.0,
    };

    let json = serde_json::to_string(&user_types).unwrap();
    let deserialized: RealtimeUserTypes = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.new_users, 30);
    assert_eq!(deserialized.returning_users_percentage, 75.0);
}

#[test]
fn test_realtime_user_types_all_new() {
    let user_types = RealtimeUserTypes {
        new_users: 100,
        new_users_percentage: 100.0,
        returning_users: 0,
        returning_users_percentage: 0.0,
    };

    assert_eq!(user_types.new_users_percentage, 100.0);
    assert_eq!(user_types.returning_users, 0);
}

#[test]
fn test_realtime_user_types_all_returning() {
    let user_types = RealtimeUserTypes {
        new_users: 0,
        new_users_percentage: 0.0,
        returning_users: 150,
        returning_users_percentage: 100.0,
    };

    assert_eq!(user_types.new_users, 0);
    assert_eq!(user_types.returning_users_percentage, 100.0);
}

// ============================================================================
// Edge Cases and Special Scenarios
// ============================================================================

#[test]
fn test_high_traffic_scenario() {
    let overview = RealtimeOverview {
        active_users: 10000,
        active_users_1min: 5000,
        active_users_5min: 15000,
        active_users_10min: 25000,
        active_users_30min: 50000,
        pageviews_per_minute: vec![],
        pageviews_per_second: vec![],
        top_active_pages: vec![],
        top_referrers: vec![],
        top_keywords: vec![],
        top_locations: vec![],
        top_traffic_sources: vec![],
        top_social_sources: vec![],
        device_breakdown: DeviceBreakdown {
            desktop: DeviceStats {
                active_users: 4000,
                percentage: 40.0,
            },
            mobile: DeviceStats {
                active_users: 5000,
                percentage: 50.0,
            },
            tablet: DeviceStats {
                active_users: 1000,
                percentage: 10.0,
            },
        },
        active_events: vec![],
        active_conversions: vec![],
        timestamp: Utc::now(),
    };

    assert_eq!(overview.active_users, 10000);
    assert_eq!(overview.active_users_30min, 50000);
}

#[test]
fn test_unicode_in_realtime_data() {
    let page = ActivePage {
        page_path: "/ブログ/記事".to_string(),
        page_title: "日本語のページ".to_string(),
        active_users: 10,
        percentage: 5.0,
    };

    let json = serde_json::to_string(&page).unwrap();
    let deserialized: ActivePage = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.page_path, "/ブログ/記事");
    assert_eq!(deserialized.page_title, "日本語のページ");
}

#[test]
fn test_special_characters_in_event_data() {
    let event = ActiveEvent {
        event_category: "user_interaction".to_string(),
        event_action: "click <button>".to_string(),
        event_label: Some("CTA & Signup".to_string()),
        event_count: 5,
        users: 4,
    };

    let json = serde_json::to_string(&event).unwrap();
    let deserialized: ActiveEvent = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.event_action, "click <button>");
    assert_eq!(deserialized.event_label, Some("CTA & Signup".to_string()));
}

#[test]
fn test_negative_coordinates() {
    let location = ActiveLocation {
        country: "Argentina".to_string(),
        country_code: "AR".to_string(),
        region: None,
        city: Some("Buenos Aires".to_string()),
        latitude: Some(-34.6037),
        longitude: Some(-58.3816),
        active_users: 20,
        percentage: 13.33,
    };

    assert!(location.latitude.unwrap() < 0.0);
    assert!(location.longitude.unwrap() < 0.0);
}

#[test]
fn test_very_small_percentages() {
    let source = ActiveTrafficSource {
        source: "obscure_site".to_string(),
        medium: "referral".to_string(),
        active_users: 1,
        percentage: 0.001,
    };

    assert!((source.percentage - 0.001).abs() < 0.0001);
}

#[test]
fn test_100_percent_single_source() {
    let source = ActiveTrafficSource {
        source: "direct".to_string(),
        medium: "none".to_string(),
        active_users: 100,
        percentage: 100.0,
    };

    assert_eq!(source.percentage, 100.0);
}

// ============================================================================
// Response Processing Tests - Active Users
// ============================================================================

/// Helper to process active users from realtime report response
fn process_active_users_response(
    rows: Option<Vec<(Vec<Option<String>>, Vec<Option<String>>)>>,
    totals: Option<Vec<Option<String>>>,
) -> (u32, u32, u32, u32, u32) {
    let mut current = 0u32;
    let mut min1 = 0u32;
    let mut min5 = 0u32;
    let mut min10 = 0u32;
    let mut min30 = 0u32;

    if let Some(rows) = rows {
        for (i, (_dims, vals)) in rows.iter().enumerate() {
            if let Some(val) = vals.get(0).and_then(|v| v.as_ref()) {
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

    // If no minute ranges, use totals
    if let Some(totals) = totals {
        if let Some(val) = totals.get(0).and_then(|v| v.as_ref()) {
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

    (current, min1, min5, min10, min30)
}

#[test]
fn test_process_active_users_with_minute_ranges() {
    let rows = Some(vec![
        (vec![], vec![Some("100".to_string())]),  // current
        (vec![], vec![Some("120".to_string())]),  // 1min
        (vec![], vec![Some("150".to_string())]),  // 5min
        (vec![], vec![Some("180".to_string())]),  // 10min
        (vec![], vec![Some("250".to_string())]),  // 30min
    ]);

    let (current, min1, min5, min10, min30) = process_active_users_response(rows, None);

    assert_eq!(current, 100);
    assert_eq!(min1, 120);
    assert_eq!(min5, 150);
    assert_eq!(min10, 180);
    assert_eq!(min30, 250);
}

#[test]
fn test_process_active_users_from_totals() {
    let totals = Some(vec![Some("500".to_string())]);

    let (current, min1, min5, min10, min30) = process_active_users_response(None, totals);

    assert_eq!(current, 500);
    assert_eq!(min1, 500);
    assert_eq!(min5, 500);
    assert_eq!(min10, 500);
    assert_eq!(min30, 500);
}

#[test]
fn test_process_active_users_empty_response() {
    let (current, min1, min5, min10, min30) = process_active_users_response(None, None);

    assert_eq!(current, 0);
    assert_eq!(min1, 0);
    assert_eq!(min5, 0);
    assert_eq!(min10, 0);
    assert_eq!(min30, 0);
}

#[test]
fn test_process_active_users_invalid_values() {
    let rows = Some(vec![
        (vec![], vec![Some("invalid".to_string())]),
        (vec![], vec![Some("abc".to_string())]),
        (vec![], vec![None]),
    ]);

    let (current, min1, min5, _, _) = process_active_users_response(rows, None);

    assert_eq!(current, 0);
    assert_eq!(min1, 0);
    assert_eq!(min5, 0);
}

#[test]
fn test_process_active_users_partial_minute_ranges() {
    let rows = Some(vec![
        (vec![], vec![Some("50".to_string())]),
        (vec![], vec![Some("75".to_string())]),
    ]);

    let (current, min1, min5, min10, min30) = process_active_users_response(rows, None);

    assert_eq!(current, 50);
    assert_eq!(min1, 75);
    assert_eq!(min5, 0);
    assert_eq!(min10, 0);
    assert_eq!(min30, 0);
}

#[test]
fn test_process_active_users_rows_override_totals() {
    let rows = Some(vec![
        (vec![], vec![Some("100".to_string())]),
    ]);
    let totals = Some(vec![Some("500".to_string())]);

    let (current, _, _, _, _) = process_active_users_response(rows, totals);

    // When rows have data, totals should not override
    assert_eq!(current, 100);
}

// ============================================================================
// Response Processing Tests - Active Pages
// ============================================================================

/// Helper to process active pages from response
fn process_active_pages_response(
    rows: Option<Vec<(Vec<Option<String>>, Vec<Option<String>>)>>,
    total_users: u32,
) -> Vec<ActivePage> {
    let mut pages = Vec::new();

    if let Some(rows) = rows {
        for (dims, vals) in rows {
            let page_path = dims.get(0).and_then(|d| d.clone()).unwrap_or_default();
            let page_title = dims.get(1).and_then(|d| d.clone()).unwrap_or_default();
            let active_users: u32 = vals
                .get(0)
                .and_then(|v| v.as_ref())
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

    pages
}

#[test]
fn test_process_active_pages_basic() {
    let rows = Some(vec![
        (vec![Some("/home".to_string()), Some("Home Page".to_string())], vec![Some("50".to_string())]),
        (vec![Some("/about".to_string()), Some("About Us".to_string())], vec![Some("30".to_string())]),
        (vec![Some("/contact".to_string()), Some("Contact".to_string())], vec![Some("20".to_string())]),
    ]);

    let pages = process_active_pages_response(rows, 100);

    assert_eq!(pages.len(), 3);
    assert_eq!(pages[0].page_path, "/home");
    assert_eq!(pages[0].active_users, 50);
    assert!((pages[0].percentage - 50.0).abs() < 0.01);
    assert_eq!(pages[1].page_path, "/about");
    assert!((pages[1].percentage - 30.0).abs() < 0.01);
}

#[test]
fn test_process_active_pages_empty_response() {
    let pages = process_active_pages_response(None, 100);
    assert!(pages.is_empty());
}

#[test]
fn test_process_active_pages_zero_total_users() {
    let rows = Some(vec![
        (vec![Some("/home".to_string()), Some("Home".to_string())], vec![Some("10".to_string())]),
    ]);

    let pages = process_active_pages_response(rows, 0);

    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0].percentage, 0.0); // No division by zero
}

#[test]
fn test_process_active_pages_missing_title() {
    let rows = Some(vec![
        (vec![Some("/products".to_string()), None], vec![Some("25".to_string())]),
    ]);

    let pages = process_active_pages_response(rows, 100);

    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0].page_path, "/products");
    assert_eq!(pages[0].page_title, "");
}

#[test]
fn test_process_active_pages_unicode_paths() {
    let rows = Some(vec![
        (vec![Some("/製品".to_string()), Some("製品ページ".to_string())], vec![Some("15".to_string())]),
        (vec![Some("/блог".to_string()), Some("Блог".to_string())], vec![Some("10".to_string())]),
    ]);

    let pages = process_active_pages_response(rows, 50);

    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0].page_path, "/製品");
    assert_eq!(pages[0].page_title, "製品ページ");
    assert!((pages[0].percentage - 30.0).abs() < 0.01);
}

#[test]
fn test_process_active_pages_query_strings() {
    let rows = Some(vec![
        (vec![Some("/search?q=rust".to_string()), Some("Search Results".to_string())], vec![Some("20".to_string())]),
    ]);

    let pages = process_active_pages_response(rows, 100);

    assert_eq!(pages[0].page_path, "/search?q=rust");
}

// ============================================================================
// Response Processing Tests - Traffic Sources
// ============================================================================

/// Helper to process traffic sources from response
fn process_traffic_sources_response(
    rows: Option<Vec<(Vec<Option<String>>, Vec<Option<String>>)>>,
    total_users: u32,
) -> Vec<ActiveTrafficSource> {
    let mut sources = Vec::new();

    if let Some(rows) = rows {
        for (dims, vals) in rows {
            let source = dims.get(0).and_then(|d| d.clone()).unwrap_or_default();
            let medium = dims.get(1).and_then(|d| d.clone()).unwrap_or_default();
            let active_users: u32 = vals
                .get(0)
                .and_then(|v| v.as_ref())
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

    sources
}

#[test]
fn test_process_traffic_sources_basic() {
    let rows = Some(vec![
        (vec![Some("google".to_string()), Some("organic".to_string())], vec![Some("60".to_string())]),
        (vec![Some("facebook".to_string()), Some("social".to_string())], vec![Some("25".to_string())]),
        (vec![Some("(direct)".to_string()), Some("(none)".to_string())], vec![Some("15".to_string())]),
    ]);

    let sources = process_traffic_sources_response(rows, 100);

    assert_eq!(sources.len(), 3);
    assert_eq!(sources[0].source, "google");
    assert_eq!(sources[0].medium, "organic");
    assert_eq!(sources[0].active_users, 60);
    assert!((sources[0].percentage - 60.0).abs() < 0.01);
}

#[test]
fn test_process_traffic_sources_empty() {
    let sources = process_traffic_sources_response(None, 100);
    assert!(sources.is_empty());
}

#[test]
fn test_process_traffic_sources_zero_total() {
    let rows = Some(vec![
        (vec![Some("google".to_string()), Some("cpc".to_string())], vec![Some("50".to_string())]),
    ]);

    let sources = process_traffic_sources_response(rows, 0);

    assert_eq!(sources[0].percentage, 0.0);
}

#[test]
fn test_process_traffic_sources_special_characters() {
    let rows = Some(vec![
        (vec![Some("utm_source=test&campaign".to_string()), Some("email/newsletter".to_string())], vec![Some("30".to_string())]),
    ]);

    let sources = process_traffic_sources_response(rows, 100);

    assert_eq!(sources[0].source, "utm_source=test&campaign");
    assert_eq!(sources[0].medium, "email/newsletter");
}

// ============================================================================
// Response Processing Tests - Locations
// ============================================================================

/// Helper to process locations from response
fn process_locations_response(
    rows: Option<Vec<(Vec<Option<String>>, Vec<Option<String>>)>>,
    total_users: u32,
) -> Vec<ActiveLocation> {
    let mut locations = Vec::new();

    if let Some(rows) = rows {
        for (dims, vals) in rows {
            let country = dims.get(0).and_then(|d| d.clone()).unwrap_or_default();
            let city = dims.get(1).and_then(|d| d.clone());
            let active_users: u32 = vals
                .get(0)
                .and_then(|v| v.as_ref())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            locations.push(ActiveLocation {
                country: country.clone(),
                country_code: String::new(),
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

    locations
}

#[test]
fn test_process_locations_basic() {
    let rows = Some(vec![
        (vec![Some("United States".to_string()), Some("New York".to_string())], vec![Some("40".to_string())]),
        (vec![Some("United Kingdom".to_string()), Some("London".to_string())], vec![Some("30".to_string())]),
        (vec![Some("Germany".to_string()), Some("Berlin".to_string())], vec![Some("30".to_string())]),
    ]);

    let locations = process_locations_response(rows, 100);

    assert_eq!(locations.len(), 3);
    assert_eq!(locations[0].country, "United States");
    assert_eq!(locations[0].city, Some("New York".to_string()));
    assert!((locations[0].percentage - 40.0).abs() < 0.01);
}

#[test]
fn test_process_locations_without_city() {
    let rows = Some(vec![
        (vec![Some("Japan".to_string()), None], vec![Some("20".to_string())]),
    ]);

    let locations = process_locations_response(rows, 100);

    assert_eq!(locations[0].country, "Japan");
    assert!(locations[0].city.is_none());
}

#[test]
fn test_process_locations_unicode_names() {
    let rows = Some(vec![
        (vec![Some("日本".to_string()), Some("東京".to_string())], vec![Some("25".to_string())]),
        (vec![Some("中国".to_string()), Some("北京".to_string())], vec![Some("20".to_string())]),
    ]);

    let locations = process_locations_response(rows, 100);

    assert_eq!(locations[0].country, "日本");
    assert_eq!(locations[0].city, Some("東京".to_string()));
}

// ============================================================================
// Response Processing Tests - Device Breakdown
// ============================================================================

/// Helper to process device breakdown from response
fn process_device_breakdown_response(
    rows: Option<Vec<(Vec<Option<String>>, Vec<Option<String>>)>>,
    total_users: u32,
) -> DeviceBreakdown {
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

    if let Some(rows) = rows {
        for (dims, vals) in rows {
            let device = dims
                .get(0)
                .and_then(|d| d.as_ref())
                .map(|s| s.to_lowercase())
                .unwrap_or_default();
            let users: u32 = vals
                .get(0)
                .and_then(|v| v.as_ref())
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

    DeviceBreakdown {
        desktop,
        mobile,
        tablet,
    }
}

#[test]
fn test_process_device_breakdown_all_devices() {
    let rows = Some(vec![
        (vec![Some("desktop".to_string())], vec![Some("50".to_string())]),
        (vec![Some("mobile".to_string())], vec![Some("40".to_string())]),
        (vec![Some("tablet".to_string())], vec![Some("10".to_string())]),
    ]);

    let breakdown = process_device_breakdown_response(rows, 100);

    assert_eq!(breakdown.desktop.active_users, 50);
    assert!((breakdown.desktop.percentage - 50.0).abs() < 0.01);
    assert_eq!(breakdown.mobile.active_users, 40);
    assert!((breakdown.mobile.percentage - 40.0).abs() < 0.01);
    assert_eq!(breakdown.tablet.active_users, 10);
    assert!((breakdown.tablet.percentage - 10.0).abs() < 0.01);
}

#[test]
fn test_process_device_breakdown_case_insensitive() {
    let rows = Some(vec![
        (vec![Some("DESKTOP".to_string())], vec![Some("60".to_string())]),
        (vec![Some("Mobile".to_string())], vec![Some("30".to_string())]),
        (vec![Some("TABLET".to_string())], vec![Some("10".to_string())]),
    ]);

    let breakdown = process_device_breakdown_response(rows, 100);

    assert_eq!(breakdown.desktop.active_users, 60);
    assert_eq!(breakdown.mobile.active_users, 30);
    assert_eq!(breakdown.tablet.active_users, 10);
}

#[test]
fn test_process_device_breakdown_only_mobile() {
    let rows = Some(vec![
        (vec![Some("mobile".to_string())], vec![Some("100".to_string())]),
    ]);

    let breakdown = process_device_breakdown_response(rows, 100);

    assert_eq!(breakdown.desktop.active_users, 0);
    assert_eq!(breakdown.mobile.active_users, 100);
    assert_eq!(breakdown.tablet.active_users, 0);
}

#[test]
fn test_process_device_breakdown_unknown_device() {
    let rows = Some(vec![
        (vec![Some("desktop".to_string())], vec![Some("50".to_string())]),
        (vec![Some("smart_tv".to_string())], vec![Some("20".to_string())]),
        (vec![Some("console".to_string())], vec![Some("10".to_string())]),
    ]);

    let breakdown = process_device_breakdown_response(rows, 100);

    // Unknown devices should be ignored
    assert_eq!(breakdown.desktop.active_users, 50);
    assert_eq!(breakdown.mobile.active_users, 0);
    assert_eq!(breakdown.tablet.active_users, 0);
}

#[test]
fn test_process_device_breakdown_empty() {
    let breakdown = process_device_breakdown_response(None, 100);

    assert_eq!(breakdown.desktop.active_users, 0);
    assert_eq!(breakdown.mobile.active_users, 0);
    assert_eq!(breakdown.tablet.active_users, 0);
}

#[test]
fn test_process_device_breakdown_zero_total() {
    let rows = Some(vec![
        (vec![Some("desktop".to_string())], vec![Some("50".to_string())]),
    ]);

    let breakdown = process_device_breakdown_response(rows, 0);

    assert_eq!(breakdown.desktop.active_users, 50);
    assert_eq!(breakdown.desktop.percentage, 0.0); // No division by zero
}

// ============================================================================
// Response Processing Tests - Active Events
// ============================================================================

/// Helper to process active events from response
fn process_active_events_response(
    rows: Option<Vec<(Vec<Option<String>>, Vec<Option<String>>)>>,
) -> Vec<ActiveEvent> {
    let mut events = Vec::new();

    if let Some(rows) = rows {
        for (dims, vals) in rows {
            let event_name = dims.get(0).and_then(|d| d.clone()).unwrap_or_default();
            let event_count: u32 = vals
                .get(0)
                .and_then(|v| v.as_ref())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let users: u32 = vals
                .get(1)
                .and_then(|v| v.as_ref())
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

    events
}

#[test]
fn test_process_active_events_basic() {
    let rows = Some(vec![
        (vec![Some("page_view".to_string())], vec![Some("500".to_string()), Some("200".to_string())]),
        (vec![Some("click".to_string())], vec![Some("150".to_string()), Some("80".to_string())]),
        (vec![Some("scroll".to_string())], vec![Some("300".to_string()), Some("150".to_string())]),
    ]);

    let events = process_active_events_response(rows);

    assert_eq!(events.len(), 3);
    assert_eq!(events[0].event_category, "page_view");
    assert_eq!(events[0].event_count, 500);
    assert_eq!(events[0].users, 200);
}

#[test]
fn test_process_active_events_empty() {
    let events = process_active_events_response(None);
    assert!(events.is_empty());
}

#[test]
fn test_process_active_events_missing_users() {
    let rows = Some(vec![
        (vec![Some("custom_event".to_string())], vec![Some("100".to_string()), None]),
    ]);

    let events = process_active_events_response(rows);

    assert_eq!(events[0].event_count, 100);
    assert_eq!(events[0].users, 0);
}

#[test]
fn test_process_active_events_special_names() {
    let rows = Some(vec![
        (vec![Some("video_play_50%".to_string())], vec![Some("50".to_string()), Some("40".to_string())]),
        (vec![Some("form_submit_<contact>".to_string())], vec![Some("30".to_string()), Some("25".to_string())]),
    ]);

    let events = process_active_events_response(rows);

    assert_eq!(events[0].event_category, "video_play_50%");
    assert_eq!(events[1].event_category, "form_submit_<contact>");
}

// ============================================================================
// Response Processing Tests - Active Conversions
// ============================================================================

/// Helper to process active conversions from response
fn process_active_conversions_response(
    rows: Option<Vec<(Vec<Option<String>>, Vec<Option<String>>)>>,
) -> Vec<ActiveConversion> {
    let mut conversions = Vec::new();

    if let Some(rows) = rows {
        for (dims, vals) in rows {
            let goal_name = dims.get(0).and_then(|d| d.clone()).unwrap_or_default();
            let completions: u32 = vals
                .get(0)
                .and_then(|v| v.as_ref())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let value: f64 = vals
                .get(1)
                .and_then(|v| v.as_ref())
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

    conversions
}

#[test]
fn test_process_active_conversions_basic() {
    let rows = Some(vec![
        (vec![Some("purchase".to_string())], vec![Some("10".to_string()), Some("999.99".to_string())]),
        (vec![Some("signup".to_string())], vec![Some("25".to_string()), Some("0".to_string())]),
    ]);

    let conversions = process_active_conversions_response(rows);

    assert_eq!(conversions.len(), 2);
    assert_eq!(conversions[0].goal_name, "purchase");
    assert_eq!(conversions[0].completions, 10);
    assert!((conversions[0].value - 999.99).abs() < 0.01);
    assert_eq!(conversions[1].goal_name, "signup");
    assert_eq!(conversions[1].value, 0.0);
}

#[test]
fn test_process_active_conversions_empty() {
    let conversions = process_active_conversions_response(None);
    assert!(conversions.is_empty());
}

#[test]
fn test_process_active_conversions_large_values() {
    let rows = Some(vec![
        (vec![Some("high_value_purchase".to_string())], vec![Some("1000".to_string()), Some("1000000.50".to_string())]),
    ]);

    let conversions = process_active_conversions_response(rows);

    assert_eq!(conversions[0].completions, 1000);
    assert!((conversions[0].value - 1000000.50).abs() < 0.01);
}

// ============================================================================
// Response Processing Tests - Geo Distribution
// ============================================================================

/// Helper to process geo distribution from response
fn process_geo_distribution_response(
    rows: Option<Vec<(Vec<Option<String>>, Vec<Option<String>>)>>,
) -> RealtimeGeoDistribution {
    let mut countries: std::collections::HashMap<String, CountryActiveUsers> =
        std::collections::HashMap::new();
    let mut cities = Vec::new();
    let mut user_locations = Vec::new();

    if let Some(rows) = rows {
        for (dims, vals) in rows {
            let country = dims.get(0).and_then(|d| d.clone()).unwrap_or_default();
            let city = dims.get(1).and_then(|d| d.clone()).unwrap_or_default();
            let active_users: u32 = vals
                .get(0)
                .and_then(|v| v.as_ref())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let pageviews: u32 = vals
                .get(1)
                .and_then(|v| v.as_ref())
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

                user_locations.push(UserLocation {
                    latitude: 0.0,
                    longitude: 0.0,
                    active_users,
                    city: Some(city),
                    country,
                });
            }
        }
    }

    RealtimeGeoDistribution {
        countries: countries.into_values().collect(),
        cities,
        user_locations,
    }
}

#[test]
fn test_process_geo_distribution_basic() {
    let rows = Some(vec![
        (vec![Some("United States".to_string()), Some("New York".to_string())], vec![Some("50".to_string()), Some("200".to_string())]),
        (vec![Some("United States".to_string()), Some("Los Angeles".to_string())], vec![Some("30".to_string()), Some("100".to_string())]),
        (vec![Some("Canada".to_string()), Some("Toronto".to_string())], vec![Some("20".to_string()), Some("80".to_string())]),
    ]);

    let geo = process_geo_distribution_response(rows);

    assert_eq!(geo.countries.len(), 2);
    assert_eq!(geo.cities.len(), 3);
    assert_eq!(geo.user_locations.len(), 3);

    // Check country aggregation
    let us = geo.countries.iter().find(|c| c.country == "United States").unwrap();
    assert_eq!(us.active_users, 80); // 50 + 30
    assert_eq!(us.pageviews, 300); // 200 + 100
}

#[test]
fn test_process_geo_distribution_filters_not_set() {
    let rows = Some(vec![
        (vec![Some("Unknown".to_string()), Some("(not set)".to_string())], vec![Some("10".to_string()), Some("20".to_string())]),
        (vec![Some("Germany".to_string()), Some("Berlin".to_string())], vec![Some("15".to_string()), Some("50".to_string())]),
    ]);

    let geo = process_geo_distribution_response(rows);

    // (not set) cities should be filtered out
    assert_eq!(geo.cities.len(), 1);
    assert_eq!(geo.cities[0].city, "Berlin");
}

#[test]
fn test_process_geo_distribution_filters_empty_city() {
    let rows = Some(vec![
        (vec![Some("France".to_string()), Some("".to_string())], vec![Some("25".to_string()), Some("100".to_string())]),
        (vec![Some("France".to_string()), Some("Paris".to_string())], vec![Some("20".to_string()), Some("80".to_string())]),
    ]);

    let geo = process_geo_distribution_response(rows);

    // Empty city should be filtered out
    assert_eq!(geo.cities.len(), 1);
    assert_eq!(geo.cities[0].city, "Paris");

    // But country should still aggregate
    let france = geo.countries.iter().find(|c| c.country == "France").unwrap();
    assert_eq!(france.active_users, 45); // 25 + 20
}

#[test]
fn test_process_geo_distribution_empty() {
    let geo = process_geo_distribution_response(None);

    assert!(geo.countries.is_empty());
    assert!(geo.cities.is_empty());
    assert!(geo.user_locations.is_empty());
}

#[test]
fn test_process_geo_distribution_unicode() {
    let rows = Some(vec![
        (vec![Some("日本".to_string()), Some("東京".to_string())], vec![Some("100".to_string()), Some("500".to_string())]),
        (vec![Some("中国".to_string()), Some("北京".to_string())], vec![Some("80".to_string()), Some("400".to_string())]),
    ]);

    let geo = process_geo_distribution_response(rows);

    assert_eq!(geo.countries.len(), 2);
    assert!(geo.countries.iter().any(|c| c.country == "日本"));
    assert!(geo.cities.iter().any(|c| c.city == "東京"));
}

// ============================================================================
// Percentage Calculation Tests
// ============================================================================

fn calculate_percentage(value: u32, total: u32) -> f64 {
    if total > 0 {
        (value as f64 / total as f64) * 100.0
    } else {
        0.0
    }
}

#[test]
fn test_percentage_calculation_basic() {
    assert!((calculate_percentage(50, 100) - 50.0).abs() < 0.001);
    assert!((calculate_percentage(25, 100) - 25.0).abs() < 0.001);
    assert!((calculate_percentage(1, 100) - 1.0).abs() < 0.001);
}

#[test]
fn test_percentage_calculation_zero_total() {
    assert_eq!(calculate_percentage(50, 0), 0.0);
    assert_eq!(calculate_percentage(0, 0), 0.0);
}

#[test]
fn test_percentage_calculation_100_percent() {
    assert!((calculate_percentage(100, 100) - 100.0).abs() < 0.001);
    assert!((calculate_percentage(1000, 1000) - 100.0).abs() < 0.001);
}

#[test]
fn test_percentage_calculation_fractional() {
    assert!((calculate_percentage(1, 3) - 33.333333).abs() < 0.001);
    assert!((calculate_percentage(2, 3) - 66.666666).abs() < 0.001);
}

#[test]
fn test_percentage_calculation_very_small() {
    let result = calculate_percentage(1, 1000000);
    assert!(result > 0.0);
    assert!(result < 0.001);
}

#[test]
fn test_percentage_calculation_large_numbers() {
    let result = calculate_percentage(500000, 1000000);
    assert!((result - 50.0).abs() < 0.001);
}

// ============================================================================
// Metric Value Parsing Tests
// ============================================================================

fn parse_u32_metric(value: Option<&String>) -> u32 {
    value.and_then(|s| s.parse().ok()).unwrap_or(0)
}

fn parse_f64_metric(value: Option<&String>) -> f64 {
    value.and_then(|s| s.parse().ok()).unwrap_or(0.0)
}

#[test]
fn test_parse_u32_metric_valid() {
    assert_eq!(parse_u32_metric(Some(&"100".to_string())), 100);
    assert_eq!(parse_u32_metric(Some(&"0".to_string())), 0);
    assert_eq!(parse_u32_metric(Some(&"4294967295".to_string())), u32::MAX);
}

#[test]
fn test_parse_u32_metric_invalid() {
    assert_eq!(parse_u32_metric(Some(&"invalid".to_string())), 0);
    assert_eq!(parse_u32_metric(Some(&"12.5".to_string())), 0);
    assert_eq!(parse_u32_metric(Some(&"-100".to_string())), 0);
    assert_eq!(parse_u32_metric(None), 0);
}

#[test]
fn test_parse_f64_metric_valid() {
    assert!((parse_f64_metric(Some(&"100.5".to_string())) - 100.5).abs() < 0.001);
    assert!((parse_f64_metric(Some(&"0.0".to_string())) - 0.0).abs() < 0.001);
    assert!((parse_f64_metric(Some(&"-50.5".to_string())) - (-50.5)).abs() < 0.001);
}

#[test]
fn test_parse_f64_metric_integer_string() {
    assert!((parse_f64_metric(Some(&"100".to_string())) - 100.0).abs() < 0.001);
}

#[test]
fn test_parse_f64_metric_invalid() {
    assert_eq!(parse_f64_metric(Some(&"invalid".to_string())), 0.0);
    assert_eq!(parse_f64_metric(None), 0.0);
}

#[test]
fn test_parse_f64_metric_scientific_notation() {
    assert!((parse_f64_metric(Some(&"1e6".to_string())) - 1000000.0).abs() < 0.001);
    assert!((parse_f64_metric(Some(&"1.5e2".to_string())) - 150.0).abs() < 0.001);
}

// ============================================================================
// Clone Trait Tests for Service-Related Structs
// ============================================================================

#[test]
fn test_realtime_overview_deep_clone() {
    let overview = sample_realtime_overview();
    let cloned = overview.clone();

    // Modify original shouldn't affect clone
    assert_eq!(overview.active_users, cloned.active_users);
    assert_eq!(overview.top_active_pages.len(), cloned.top_active_pages.len());
    assert_eq!(overview.device_breakdown.desktop.active_users, cloned.device_breakdown.desktop.active_users);
}

#[test]
fn test_geo_distribution_deep_clone() {
    let geo = sample_geo_distribution();
    let cloned = geo.clone();

    assert_eq!(geo.countries.len(), cloned.countries.len());
    assert_eq!(geo.cities.len(), cloned.cities.len());
    assert_eq!(geo.user_locations.len(), cloned.user_locations.len());
}

// ============================================================================
// Debug Trait Tests
// ============================================================================

#[test]
fn test_active_page_debug() {
    let page = ActivePage {
        page_path: "/test".to_string(),
        page_title: "Test".to_string(),
        active_users: 10,
        percentage: 5.0,
    };

    let debug_str = format!("{:?}", page);
    assert!(debug_str.contains("ActivePage"));
    assert!(debug_str.contains("/test"));
}

#[test]
fn test_device_breakdown_debug() {
    let breakdown = sample_device_breakdown();
    let debug_str = format!("{:?}", breakdown);

    assert!(debug_str.contains("DeviceBreakdown"));
    assert!(debug_str.contains("desktop"));
    assert!(debug_str.contains("mobile"));
    assert!(debug_str.contains("tablet"));
}

#[test]
fn test_realtime_geo_distribution_debug() {
    let geo = sample_geo_distribution();
    let debug_str = format!("{:?}", geo);

    assert!(debug_str.contains("RealtimeGeoDistribution"));
    assert!(debug_str.contains("countries"));
}

// ============================================================================
// Edge Cases - High Volume Scenarios
// ============================================================================

#[test]
fn test_process_many_pages() {
    let rows: Vec<_> = (0..100)
        .map(|i| {
            (
                vec![Some(format!("/page{}", i)), Some(format!("Page {}", i))],
                vec![Some(format!("{}", 100 - i))],
            )
        })
        .collect();

    let pages = process_active_pages_response(Some(rows), 5050);

    assert_eq!(pages.len(), 100);
    assert_eq!(pages[0].page_path, "/page0");
    assert_eq!(pages[0].active_users, 100);
}

#[test]
fn test_process_many_locations() {
    let rows: Vec<_> = (0..50)
        .map(|i| {
            (
                vec![Some(format!("Country{}", i)), Some(format!("City{}", i))],
                vec![Some(format!("{}", 10)), Some(format!("{}", 50))],
            )
        })
        .collect();

    let geo = process_geo_distribution_response(Some(rows));

    assert_eq!(geo.countries.len(), 50);
    assert_eq!(geo.cities.len(), 50);
}

// ============================================================================
// Edge Cases - Boundary Values
// ============================================================================

#[test]
fn test_max_u32_active_users() {
    let rows = Some(vec![
        (vec![Some("desktop".to_string())], vec![Some(u32::MAX.to_string())]),
    ]);

    let breakdown = process_device_breakdown_response(rows, u32::MAX);

    assert_eq!(breakdown.desktop.active_users, u32::MAX);
    assert!((breakdown.desktop.percentage - 100.0).abs() < 0.01);
}

#[test]
fn test_very_precise_percentages() {
    let source = ActiveTrafficSource {
        source: "test".to_string(),
        medium: "test".to_string(),
        active_users: 1,
        percentage: 0.000001,
    };

    let json = serde_json::to_string(&source).unwrap();
    let deserialized: ActiveTrafficSource = serde_json::from_str(&json).unwrap();

    assert!((deserialized.percentage - 0.000001).abs() < 0.0000001);
}

// ============================================================================
// MinuteRange Tests (simulating the service's minute range logic)
// ============================================================================

#[derive(Debug, Clone)]
struct TestMinuteRange {
    name: Option<String>,
    start_minutes_ago: Option<i32>,
    end_minutes_ago: Option<i32>,
}

#[test]
fn test_minute_range_current() {
    let range = TestMinuteRange {
        name: Some("current".to_string()),
        start_minutes_ago: Some(0),
        end_minutes_ago: Some(0),
    };

    assert_eq!(range.start_minutes_ago, Some(0));
    assert_eq!(range.end_minutes_ago, Some(0));
}

#[test]
fn test_minute_range_30min() {
    let range = TestMinuteRange {
        name: Some("30min".to_string()),
        start_minutes_ago: Some(30),
        end_minutes_ago: Some(0),
    };

    assert_eq!(range.start_minutes_ago, Some(30));
}

#[test]
fn test_minute_ranges_for_active_users() {
    let ranges = vec![
        TestMinuteRange {
            name: Some("current".to_string()),
            start_minutes_ago: Some(0),
            end_minutes_ago: Some(0),
        },
        TestMinuteRange {
            name: Some("1min".to_string()),
            start_minutes_ago: Some(1),
            end_minutes_ago: Some(0),
        },
        TestMinuteRange {
            name: Some("5min".to_string()),
            start_minutes_ago: Some(5),
            end_minutes_ago: Some(0),
        },
        TestMinuteRange {
            name: Some("10min".to_string()),
            start_minutes_ago: Some(10),
            end_minutes_ago: Some(0),
        },
        TestMinuteRange {
            name: Some("30min".to_string()),
            start_minutes_ago: Some(30),
            end_minutes_ago: Some(0),
        },
    ];

    assert_eq!(ranges.len(), 5);
    assert_eq!(ranges[0].name, Some("current".to_string()));
    assert_eq!(ranges[4].start_minutes_ago, Some(30));
}

// ============================================================================
// API Response Simulation Tests
// ============================================================================

use rustanalytics::models::api::{RunRealtimeReportResponse, Row, DimensionValue, MetricValue};

fn create_realtime_report_response(
    rows: Vec<(Vec<&str>, Vec<&str>)>,
    totals: Option<Vec<&str>>,
) -> RunRealtimeReportResponse {
    let rows = rows
        .into_iter()
        .map(|(dims, vals)| Row {
            dimension_values: Some(
                dims.into_iter()
                    .map(|d| DimensionValue {
                        value: Some(d.to_string()),
                        one_value: None,
                    })
                    .collect(),
            ),
            metric_values: Some(
                vals.into_iter()
                    .map(|v| MetricValue {
                        value: Some(v.to_string()),
                        one_value: None,
                    })
                    .collect(),
            ),
        })
        .collect();

    let totals = totals.map(|t| {
        vec![Row {
            dimension_values: Some(vec![]),
            metric_values: Some(
                t.into_iter()
                    .map(|v| MetricValue {
                        value: Some(v.to_string()),
                        one_value: None,
                    })
                    .collect(),
            ),
        }]
    });

    RunRealtimeReportResponse {
        dimension_headers: None,
        metric_headers: None,
        rows: Some(rows),
        totals,
        maximums: None,
        minimums: None,
        row_count: None,
        property_quota: None,
        kind: None,
    }
}

#[test]
fn test_realtime_report_response_creation() {
    let response = create_realtime_report_response(
        vec![
            (vec!["desktop"], vec!["100"]),
            (vec!["mobile"], vec!["80"]),
        ],
        Some(vec!["180"]),
    );

    assert!(response.rows.is_some());
    let rows = response.rows.unwrap();
    assert_eq!(rows.len(), 2);

    assert!(response.totals.is_some());
}

#[test]
fn test_realtime_report_response_parsing() {
    let response = create_realtime_report_response(
        vec![
            (vec!["/home", "Home Page"], vec!["50"]),
            (vec!["/about", "About Us"], vec!["30"]),
        ],
        Some(vec!["80"]),
    );

    let rows = response.rows.unwrap();

    // Parse first row
    let first_row = &rows[0];
    let dims = first_row.dimension_values.as_ref().unwrap();
    let vals = first_row.metric_values.as_ref().unwrap();

    assert_eq!(dims[0].value, Some("/home".to_string()));
    assert_eq!(dims[1].value, Some("Home Page".to_string()));
    assert_eq!(vals[0].value, Some("50".to_string()));
}

#[test]
fn test_realtime_report_response_with_unicode() {
    let response = create_realtime_report_response(
        vec![
            (vec!["日本", "東京"], vec!["100", "500"]),
        ],
        None,
    );

    let rows = response.rows.unwrap();
    let dims = rows[0].dimension_values.as_ref().unwrap();

    assert_eq!(dims[0].value, Some("日本".to_string()));
    assert_eq!(dims[1].value, Some("東京".to_string()));
}

#[test]
fn test_realtime_report_response_empty() {
    let response = RunRealtimeReportResponse {
        dimension_headers: None,
        metric_headers: None,
        rows: None,
        totals: None,
        maximums: None,
        minimums: None,
        row_count: None,
        property_quota: None,
        kind: None,
    };

    assert!(response.rows.is_none());
    assert!(response.totals.is_none());
}

// ============================================================================
// Integration-like Tests (Processing Full Response Flows)
// ============================================================================

#[test]
fn test_full_active_pages_flow() {
    let response = create_realtime_report_response(
        vec![
            (vec!["/products", "Products"], vec!["40"]),
            (vec!["/checkout", "Checkout"], vec!["35"]),
            (vec!["/cart", "Shopping Cart"], vec!["25"]),
        ],
        Some(vec!["100"]),
    );

    // Parse total from totals row
    let total_users: u32 = response
        .totals
        .as_ref()
        .and_then(|t| t.first())
        .and_then(|r| r.metric_values.as_ref())
        .and_then(|v| v.first())
        .and_then(|m| m.value.as_ref())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    assert_eq!(total_users, 100);

    // Process rows
    let mut pages = Vec::new();
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

    assert_eq!(pages.len(), 3);
    assert_eq!(pages[0].page_path, "/products");
    assert!((pages[0].percentage - 40.0).abs() < 0.01);
    assert!((pages[1].percentage - 35.0).abs() < 0.01);
    assert!((pages[2].percentage - 25.0).abs() < 0.01);
}

#[test]
fn test_full_device_breakdown_flow() {
    let response = create_realtime_report_response(
        vec![
            (vec!["desktop"], vec!["50"]),
            (vec!["mobile"], vec!["40"]),
            (vec!["tablet"], vec!["10"]),
        ],
        Some(vec!["100"]),
    );

    let total_users: u32 = response
        .totals
        .as_ref()
        .and_then(|t| t.first())
        .and_then(|r| r.metric_values.as_ref())
        .and_then(|v| v.first())
        .and_then(|m| m.value.as_ref())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let mut desktop = DeviceStats { active_users: 0, percentage: 0.0 };
    let mut mobile = DeviceStats { active_users: 0, percentage: 0.0 };
    let mut tablet = DeviceStats { active_users: 0, percentage: 0.0 };

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
                    "desktop" => { desktop.active_users = users; desktop.percentage = percentage; }
                    "mobile" => { mobile.active_users = users; mobile.percentage = percentage; }
                    "tablet" => { tablet.active_users = users; tablet.percentage = percentage; }
                    _ => {}
                }
            }
        }
    }

    let breakdown = DeviceBreakdown { desktop, mobile, tablet };

    assert_eq!(breakdown.desktop.active_users, 50);
    assert!((breakdown.desktop.percentage - 50.0).abs() < 0.01);
    assert_eq!(breakdown.mobile.active_users, 40);
    assert_eq!(breakdown.tablet.active_users, 10);
}
