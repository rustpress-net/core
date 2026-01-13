//! Tests for acquisition analytics models
//!
//! This module contains comprehensive tests for all acquisition model types
//! including serialization, deserialization, and edge cases.

use chrono::NaiveDate;
use rustanalytics::models::acquisition::*;
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

fn sample_channel_acquisition() -> ChannelAcquisition {
    ChannelAcquisition {
        channel: Channel::OrganicSearch,
        users: 10000,
        new_users: 8500,
        sessions: 15000,
        bounce_rate: 45.5,
        pages_per_session: 3.2,
        avg_session_duration: 185.5,
        goal_conversion_rate: 2.5,
        goal_completions: 375,
        goal_value: 18750.0,
        transactions: 150,
        revenue: 22500.0,
        percentage: 35.5,
    }
}

fn sample_source_medium_data() -> SourceMediumData {
    SourceMediumData {
        source: "google".to_string(),
        medium: "organic".to_string(),
        users: 5000,
        new_users: 4200,
        sessions: 7500,
        bounce_rate: 42.0,
        pages_per_session: 3.5,
        avg_session_duration: 195.0,
        goal_conversion_rate: 2.8,
        goal_completions: 210,
        revenue: 12600.0,
        percentage: 25.0,
    }
}

fn sample_channel_trend_data() -> ChannelTrendData {
    ChannelTrendData {
        date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        direct: 500,
        organic_search: 1200,
        paid_search: 300,
        social: 450,
        referral: 200,
        email: 150,
        display: 100,
        other: 50,
    }
}

fn sample_acquisition_totals() -> AcquisitionTotals {
    AcquisitionTotals {
        users: 25000,
        new_users: 21000,
        sessions: 40000,
        bounce_rate: 48.5,
        pages_per_session: 2.8,
        avg_session_duration: 165.0,
        goal_completions: 1000,
        revenue: 60000.0,
    }
}

fn sample_acquisition_comparison() -> AcquisitionComparison {
    AcquisitionComparison {
        users_change: 15.5,
        new_users_change: 18.2,
        sessions_change: 12.0,
        bounce_rate_change: -5.5,
        goal_completions_change: 22.0,
        revenue_change: 28.5,
    }
}

fn sample_acquisition_overview() -> AcquisitionOverview {
    AcquisitionOverview {
        date_range: sample_date_range(),
        channels: vec![sample_channel_acquisition()],
        source_medium: vec![sample_source_medium_data()],
        top_channels_trend: vec![sample_channel_trend_data()],
        totals: sample_acquisition_totals(),
        comparison: Some(sample_acquisition_comparison()),
    }
}

fn sample_treemap_node() -> TreemapNode {
    TreemapNode {
        name: "Organic Search".to_string(),
        value: 15000,
        color: "#4285F4".to_string(),
        children: vec![
            TreemapNode {
                name: "google / organic".to_string(),
                value: 12000,
                color: "#34A853".to_string(),
                children: vec![],
            },
            TreemapNode {
                name: "bing / organic".to_string(),
                value: 3000,
                color: "#FBBC05".to_string(),
                children: vec![],
            },
        ],
    }
}

fn sample_referral_data() -> ReferralData {
    ReferralData {
        source: "example.com".to_string(),
        full_referrer: Some("https://example.com/blog/article".to_string()),
        users: 500,
        new_users: 450,
        sessions: 750,
        bounce_rate: 55.0,
        pages_per_session: 2.5,
        avg_session_duration: 120.0,
        goal_conversion_rate: 1.5,
        goal_completions: 12,
        revenue: 720.0,
        percentage: 2.5,
    }
}

fn sample_campaign_detail_data() -> CampaignDetailData {
    CampaignDetailData {
        campaign: "summer_sale_2024".to_string(),
        source: "google".to_string(),
        medium: "cpc".to_string(),
        users: 3000,
        new_users: 2800,
        sessions: 4500,
        bounce_rate: 35.0,
        pages_per_session: 4.2,
        avg_session_duration: 240.0,
        goal_completions: 180,
        goal_conversion_rate: 4.0,
        goal_value: 9000.0,
        transactions: 90,
        revenue: 13500.0,
        cost: Some(2500.0),
        roas: Some(5.4),
        cpc: Some(0.56),
        cpm: Some(8.50),
    }
}

fn sample_source_medium_campaign() -> SourceMediumCampaign {
    SourceMediumCampaign {
        source: "google".to_string(),
        medium: "cpc".to_string(),
        campaign: "brand_awareness".to_string(),
        ad_content: Some("banner_v2".to_string()),
        keyword: Some("rust programming".to_string()),
        sessions: 1200,
        conversions: 48,
        revenue: 2880.0,
    }
}

fn sample_campaign_performance_data() -> CampaignPerformanceData {
    CampaignPerformanceData {
        date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        campaign: "summer_sale_2024".to_string(),
        sessions: 450,
        conversions: 18,
        revenue: 1350.0,
        cost: Some(250.0),
    }
}

fn sample_social_network_data() -> SocialNetworkData {
    SocialNetworkData {
        social_network: "Facebook".to_string(),
        sessions: 2500,
        users: 2200,
        pageviews: 7500,
        avg_session_duration: 145.0,
        pages_per_session: 3.0,
        percentage: 45.5,
    }
}

fn sample_social_landing_page() -> SocialLandingPage {
    SocialLandingPage {
        landing_page: "/blog/rust-tutorial".to_string(),
        social_network: "Twitter".to_string(),
        sessions: 800,
        pageviews: 2400,
        avg_session_duration: 180.0,
        percentage: 15.0,
    }
}

fn sample_social_conversion() -> SocialConversion {
    SocialConversion {
        social_network: "LinkedIn".to_string(),
        assisted_conversions: 25,
        assisted_value: 1250.0,
        last_interaction_conversions: 15,
        last_interaction_value: 750.0,
    }
}

fn sample_social_flow_node() -> SocialFlowNode {
    SocialFlowNode {
        node_name: "Homepage".to_string(),
        node_type: "landing".to_string(),
        sessions: 1500,
        exits: 300,
    }
}

fn sample_social_plugin_data() -> SocialPluginData {
    SocialPluginData {
        social_source: "Facebook".to_string(),
        social_action: "like".to_string(),
        social_entity: "/products/rust-course".to_string(),
        unique_actions: 250,
        total_actions: 380,
    }
}

fn sample_google_ads_campaign() -> GoogleAdsCampaign {
    GoogleAdsCampaign {
        campaign_id: "12345678".to_string(),
        campaign_name: "Brand Campaign".to_string(),
        campaign_type: "Search".to_string(),
        impressions: 100000,
        clicks: 5000,
        cost: 2500.0,
        ctr: 5.0,
        cpc: 0.50,
        sessions: 4800,
        bounce_rate: 30.0,
        pages_per_session: 4.5,
        goal_completions: 240,
        goal_conversion_rate: 5.0,
        revenue: 12000.0,
        roas: 4.8,
    }
}

fn sample_google_ads_ad_group() -> GoogleAdsAdGroup {
    GoogleAdsAdGroup {
        ad_group_id: "987654321".to_string(),
        ad_group_name: "Rust Programming Courses".to_string(),
        campaign_name: "Brand Campaign".to_string(),
        impressions: 25000,
        clicks: 1500,
        cost: 750.0,
        ctr: 6.0,
        cpc: 0.50,
        sessions: 1450,
        conversions: 72,
        revenue: 3600.0,
    }
}

fn sample_google_ads_keyword() -> GoogleAdsKeyword {
    GoogleAdsKeyword {
        keyword: "rust programming course".to_string(),
        keyword_match_type: "exact".to_string(),
        ad_group: "Rust Programming Courses".to_string(),
        campaign: "Brand Campaign".to_string(),
        impressions: 5000,
        clicks: 400,
        cost: 200.0,
        ctr: 8.0,
        cpc: 0.50,
        quality_score: Some(9),
        sessions: 390,
        bounce_rate: 25.0,
        conversions: 20,
        revenue: 1000.0,
    }
}

fn sample_google_ads_search_query() -> GoogleAdsSearchQuery {
    GoogleAdsSearchQuery {
        search_query: "best rust programming course online".to_string(),
        matched_keyword: "rust programming course".to_string(),
        impressions: 1000,
        clicks: 120,
        cost: 60.0,
        conversions: 8,
    }
}

fn sample_google_ads_display_target() -> GoogleAdsDisplayTarget {
    GoogleAdsDisplayTarget {
        targeting_type: "audience".to_string(),
        target: "In-market: Software Developers".to_string(),
        impressions: 50000,
        clicks: 500,
        cost: 150.0,
        conversions: 10,
    }
}

fn sample_google_ads_performance_trend() -> GoogleAdsPerformanceTrend {
    GoogleAdsPerformanceTrend {
        date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        impressions: 5000,
        clicks: 250,
        cost: 125.0,
        conversions: 12,
        revenue: 600.0,
    }
}

fn sample_search_console_query() -> SearchConsoleQuery {
    SearchConsoleQuery {
        query: "rust programming tutorial".to_string(),
        clicks: 500,
        impressions: 10000,
        ctr: 5.0,
        position: 3.5,
        sessions: Some(480),
        goal_completions: Some(24),
    }
}

fn sample_search_console_page() -> SearchConsolePage {
    SearchConsolePage {
        page: "/tutorials/rust-basics".to_string(),
        clicks: 1200,
        impressions: 25000,
        ctr: 4.8,
        position: 4.2,
    }
}

fn sample_search_console_country() -> SearchConsoleCountry {
    SearchConsoleCountry {
        country: "United States".to_string(),
        country_code: "US".to_string(),
        clicks: 5000,
        impressions: 100000,
        ctr: 5.0,
        position: 3.8,
    }
}

fn sample_search_console_device() -> SearchConsoleDevice {
    SearchConsoleDevice {
        device: "DESKTOP".to_string(),
        clicks: 3500,
        impressions: 70000,
        ctr: 5.0,
        position: 3.5,
    }
}

fn sample_search_console_appearance() -> SearchConsoleAppearance {
    SearchConsoleAppearance {
        search_appearance: "AMP_ARTICLE".to_string(),
        clicks: 800,
        impressions: 20000,
        ctr: 4.0,
    }
}

fn sample_search_console_performance() -> SearchConsolePerformance {
    SearchConsolePerformance {
        date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        clicks: 350,
        impressions: 7000,
        ctr: 5.0,
        position: 3.6,
    }
}

fn sample_utm_source_data() -> UtmSourceData {
    UtmSourceData {
        utm_source: "newsletter".to_string(),
        sessions: 2500,
        users: 2200,
        conversions: 100,
        revenue: 5000.0,
        percentage: 12.5,
    }
}

fn sample_utm_medium_data() -> UtmMediumData {
    UtmMediumData {
        utm_medium: "email".to_string(),
        sessions: 3000,
        users: 2700,
        conversions: 120,
        revenue: 6000.0,
        percentage: 15.0,
    }
}

fn sample_utm_campaign_data() -> UtmCampaignData {
    UtmCampaignData {
        utm_campaign: "black_friday_2024".to_string(),
        sessions: 5000,
        users: 4500,
        conversions: 250,
        revenue: 12500.0,
        percentage: 25.0,
    }
}

fn sample_utm_content_data() -> UtmContentData {
    UtmContentData {
        utm_content: "hero_banner_v2".to_string(),
        sessions: 1500,
        users: 1400,
        conversions: 60,
        revenue: 3000.0,
        percentage: 7.5,
    }
}

fn sample_utm_term_data() -> UtmTermData {
    UtmTermData {
        utm_term: "rust+programming".to_string(),
        sessions: 800,
        users: 750,
        conversions: 32,
        revenue: 1600.0,
        percentage: 4.0,
    }
}

// ============================================================================
// Channel Enum Tests
// ============================================================================

#[test]
fn test_channel_all_variants() {
    let channels = vec![
        Channel::Direct,
        Channel::OrganicSearch,
        Channel::PaidSearch,
        Channel::Display,
        Channel::Social,
        Channel::Email,
        Channel::Referral,
        Channel::AffiliateMarketing,
        Channel::Video,
        Channel::Audio,
        Channel::Sms,
        Channel::Mobile,
        Channel::Other,
    ];

    assert_eq!(channels.len(), 13);
}

#[test]
fn test_channel_display_trait() {
    assert_eq!(format!("{}", Channel::Direct), "Direct");
    assert_eq!(format!("{}", Channel::OrganicSearch), "Organic Search");
    assert_eq!(format!("{}", Channel::PaidSearch), "Paid Search");
    assert_eq!(format!("{}", Channel::Display), "Display");
    assert_eq!(format!("{}", Channel::Social), "Social");
    assert_eq!(format!("{}", Channel::Email), "Email");
    assert_eq!(format!("{}", Channel::Referral), "Referral");
    assert_eq!(format!("{}", Channel::AffiliateMarketing), "Affiliate");
    assert_eq!(format!("{}", Channel::Video), "Video");
    assert_eq!(format!("{}", Channel::Audio), "Audio");
    assert_eq!(format!("{}", Channel::Sms), "SMS");
    assert_eq!(format!("{}", Channel::Mobile), "Mobile");
    assert_eq!(format!("{}", Channel::Other), "(Other)");
}

#[test]
fn test_channel_serialization() {
    let channel = Channel::OrganicSearch;
    let json = serde_json::to_string(&channel).unwrap();
    assert_eq!(json, "\"organic_search\"");

    let channel = Channel::PaidSearch;
    let json = serde_json::to_string(&channel).unwrap();
    assert_eq!(json, "\"paid_search\"");

    let channel = Channel::AffiliateMarketing;
    let json = serde_json::to_string(&channel).unwrap();
    assert_eq!(json, "\"affiliate_marketing\"");
}

#[test]
fn test_channel_deserialization() {
    let channel: Channel = serde_json::from_str("\"organic_search\"").unwrap();
    assert_eq!(channel, Channel::OrganicSearch);

    let channel: Channel = serde_json::from_str("\"paid_search\"").unwrap();
    assert_eq!(channel, Channel::PaidSearch);

    let channel: Channel = serde_json::from_str("\"direct\"").unwrap();
    assert_eq!(channel, Channel::Direct);
}

#[test]
fn test_channel_clone_and_copy() {
    let channel = Channel::Social;
    let cloned = channel.clone();
    let copied = channel;

    assert_eq!(channel, cloned);
    assert_eq!(channel, copied);
}

#[test]
fn test_channel_equality() {
    assert_eq!(Channel::Direct, Channel::Direct);
    assert_ne!(Channel::Direct, Channel::Social);
}

#[test]
fn test_channel_debug() {
    let debug = format!("{:?}", Channel::OrganicSearch);
    assert!(debug.contains("OrganicSearch"));
}

// ============================================================================
// ChannelAcquisition Tests
// ============================================================================

#[test]
fn test_channel_acquisition_creation() {
    let acquisition = sample_channel_acquisition();

    assert_eq!(acquisition.channel, Channel::OrganicSearch);
    assert_eq!(acquisition.users, 10000);
    assert_eq!(acquisition.new_users, 8500);
    assert_eq!(acquisition.sessions, 15000);
    assert!((acquisition.bounce_rate - 45.5).abs() < f64::EPSILON);
    assert!((acquisition.pages_per_session - 3.2).abs() < f64::EPSILON);
    assert_eq!(acquisition.goal_completions, 375);
    assert_eq!(acquisition.transactions, 150);
}

#[test]
fn test_channel_acquisition_serialization() {
    let acquisition = sample_channel_acquisition();
    let json = serde_json::to_string(&acquisition).unwrap();

    assert!(json.contains("\"channel\":\"organic_search\""));
    assert!(json.contains("\"users\":10000"));
    assert!(json.contains("\"new_users\":8500"));
    assert!(json.contains("\"sessions\":15000"));
}

#[test]
fn test_channel_acquisition_deserialization() {
    let json = r#"{
        "channel": "organic_search",
        "users": 10000,
        "new_users": 8500,
        "sessions": 15000,
        "bounce_rate": 45.5,
        "pages_per_session": 3.2,
        "avg_session_duration": 185.5,
        "goal_conversion_rate": 2.5,
        "goal_completions": 375,
        "goal_value": 18750.0,
        "transactions": 150,
        "revenue": 22500.0,
        "percentage": 35.5
    }"#;

    let acquisition: ChannelAcquisition = serde_json::from_str(json).unwrap();
    assert_eq!(acquisition.channel, Channel::OrganicSearch);
    assert_eq!(acquisition.users, 10000);
}

#[test]
fn test_channel_acquisition_clone() {
    let acquisition = sample_channel_acquisition();
    let cloned = acquisition.clone();

    assert_eq!(acquisition.users, cloned.users);
    assert_eq!(acquisition.channel, cloned.channel);
}

#[test]
fn test_channel_acquisition_zero_values() {
    let acquisition = ChannelAcquisition {
        channel: Channel::Direct,
        users: 0,
        new_users: 0,
        sessions: 0,
        bounce_rate: 0.0,
        pages_per_session: 0.0,
        avg_session_duration: 0.0,
        goal_conversion_rate: 0.0,
        goal_completions: 0,
        goal_value: 0.0,
        transactions: 0,
        revenue: 0.0,
        percentage: 0.0,
    };

    assert_eq!(acquisition.users, 0);
    assert_eq!(acquisition.transactions, 0);
}

// ============================================================================
// SourceMediumData Tests
// ============================================================================

#[test]
fn test_source_medium_data_creation() {
    let data = sample_source_medium_data();

    assert_eq!(data.source, "google");
    assert_eq!(data.medium, "organic");
    assert_eq!(data.users, 5000);
    assert_eq!(data.new_users, 4200);
}

#[test]
fn test_source_medium_data_serialization() {
    let data = sample_source_medium_data();
    let json = serde_json::to_string(&data).unwrap();

    assert!(json.contains("\"source\":\"google\""));
    assert!(json.contains("\"medium\":\"organic\""));
}

#[test]
fn test_source_medium_data_roundtrip() {
    let data = sample_source_medium_data();
    let json = serde_json::to_string(&data).unwrap();
    let deserialized: SourceMediumData = serde_json::from_str(&json).unwrap();

    assert_eq!(data.source, deserialized.source);
    assert_eq!(data.medium, deserialized.medium);
    assert_eq!(data.users, deserialized.users);
}

#[test]
fn test_source_medium_data_special_characters() {
    let data = SourceMediumData {
        source: "email-campaign".to_string(),
        medium: "cpc/display".to_string(),
        users: 100,
        new_users: 90,
        sessions: 150,
        bounce_rate: 40.0,
        pages_per_session: 2.0,
        avg_session_duration: 120.0,
        goal_conversion_rate: 1.0,
        goal_completions: 5,
        revenue: 250.0,
        percentage: 5.0,
    };

    let json = serde_json::to_string(&data).unwrap();
    let deserialized: SourceMediumData = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.source, "email-campaign");
    assert_eq!(deserialized.medium, "cpc/display");
}

// ============================================================================
// ChannelTrendData Tests
// ============================================================================

#[test]
fn test_channel_trend_data_creation() {
    let data = sample_channel_trend_data();

    assert_eq!(data.date, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
    assert_eq!(data.direct, 500);
    assert_eq!(data.organic_search, 1200);
    assert_eq!(data.paid_search, 300);
}

#[test]
fn test_channel_trend_data_serialization() {
    let data = sample_channel_trend_data();
    let json = serde_json::to_string(&data).unwrap();

    assert!(json.contains("\"date\":\"2024-01-15\""));
    assert!(json.contains("\"direct\":500"));
    assert!(json.contains("\"organic_search\":1200"));
}

#[test]
fn test_channel_trend_data_total() {
    let data = sample_channel_trend_data();
    let total = data.direct + data.organic_search + data.paid_search + data.social
        + data.referral + data.email + data.display + data.other;

    assert_eq!(total, 2950);
}

// ============================================================================
// AcquisitionTotals Tests
// ============================================================================

#[test]
fn test_acquisition_totals_creation() {
    let totals = sample_acquisition_totals();

    assert_eq!(totals.users, 25000);
    assert_eq!(totals.new_users, 21000);
    assert_eq!(totals.sessions, 40000);
    assert_eq!(totals.goal_completions, 1000);
}

#[test]
fn test_acquisition_totals_serialization() {
    let totals = sample_acquisition_totals();
    let json = serde_json::to_string(&totals).unwrap();

    assert!(json.contains("\"users\":25000"));
    assert!(json.contains("\"revenue\":60000"));
}

#[test]
fn test_acquisition_totals_roundtrip() {
    let totals = sample_acquisition_totals();
    let json = serde_json::to_string(&totals).unwrap();
    let deserialized: AcquisitionTotals = serde_json::from_str(&json).unwrap();

    assert_eq!(totals.users, deserialized.users);
    assert_eq!(totals.sessions, deserialized.sessions);
}

// ============================================================================
// AcquisitionComparison Tests
// ============================================================================

#[test]
fn test_acquisition_comparison_creation() {
    let comparison = sample_acquisition_comparison();

    assert!((comparison.users_change - 15.5).abs() < f64::EPSILON);
    assert!((comparison.new_users_change - 18.2).abs() < f64::EPSILON);
    assert!((comparison.bounce_rate_change - (-5.5)).abs() < f64::EPSILON);
}

#[test]
fn test_acquisition_comparison_negative_values() {
    let comparison = AcquisitionComparison {
        users_change: -10.0,
        new_users_change: -15.0,
        sessions_change: -8.0,
        bounce_rate_change: 5.0,
        goal_completions_change: -20.0,
        revenue_change: -25.0,
    };

    assert!(comparison.users_change < 0.0);
    assert!(comparison.bounce_rate_change > 0.0);
}

#[test]
fn test_acquisition_comparison_serialization() {
    let comparison = sample_acquisition_comparison();
    let json = serde_json::to_string(&comparison).unwrap();

    assert!(json.contains("users_change"));
    assert!(json.contains("revenue_change"));
}

// ============================================================================
// AcquisitionOverview Tests
// ============================================================================

#[test]
fn test_acquisition_overview_creation() {
    let overview = sample_acquisition_overview();

    assert!(!overview.channels.is_empty());
    assert!(!overview.source_medium.is_empty());
    assert!(!overview.top_channels_trend.is_empty());
    assert!(overview.comparison.is_some());
}

#[test]
fn test_acquisition_overview_without_comparison() {
    let overview = AcquisitionOverview {
        date_range: sample_date_range(),
        channels: vec![],
        source_medium: vec![],
        top_channels_trend: vec![],
        totals: sample_acquisition_totals(),
        comparison: None,
    };

    assert!(overview.comparison.is_none());
    assert!(overview.channels.is_empty());
}

#[test]
fn test_acquisition_overview_serialization() {
    let overview = sample_acquisition_overview();
    let json = serde_json::to_string(&overview).unwrap();

    assert!(json.contains("\"date_range\""));
    assert!(json.contains("\"channels\""));
    assert!(json.contains("\"totals\""));
}

#[test]
fn test_acquisition_overview_roundtrip() {
    let overview = sample_acquisition_overview();
    let json = serde_json::to_string(&overview).unwrap();
    let deserialized: AcquisitionOverview = serde_json::from_str(&json).unwrap();

    assert_eq!(overview.channels.len(), deserialized.channels.len());
    assert_eq!(overview.totals.users, deserialized.totals.users);
}

// ============================================================================
// TreemapNode Tests
// ============================================================================

#[test]
fn test_treemap_node_creation() {
    let node = sample_treemap_node();

    assert_eq!(node.name, "Organic Search");
    assert_eq!(node.value, 15000);
    assert_eq!(node.color, "#4285F4");
    assert_eq!(node.children.len(), 2);
}

#[test]
fn test_treemap_node_nested() {
    let node = sample_treemap_node();

    let first_child = &node.children[0];
    assert_eq!(first_child.name, "google / organic");
    assert_eq!(first_child.value, 12000);
    assert!(first_child.children.is_empty());
}

#[test]
fn test_treemap_node_empty_children() {
    let node = TreemapNode {
        name: "Leaf Node".to_string(),
        value: 100,
        color: "#000000".to_string(),
        children: vec![],
    };

    assert!(node.children.is_empty());
}

#[test]
fn test_treemap_node_serialization() {
    let node = sample_treemap_node();
    let json = serde_json::to_string(&node).unwrap();

    assert!(json.contains("\"name\":\"Organic Search\""));
    assert!(json.contains("\"children\""));
}

#[test]
fn test_treemap_node_deeply_nested() {
    let deep_node = TreemapNode {
        name: "Level 1".to_string(),
        value: 1000,
        color: "#FF0000".to_string(),
        children: vec![TreemapNode {
            name: "Level 2".to_string(),
            value: 500,
            color: "#00FF00".to_string(),
            children: vec![TreemapNode {
                name: "Level 3".to_string(),
                value: 250,
                color: "#0000FF".to_string(),
                children: vec![],
            }],
        }],
    };

    let json = serde_json::to_string(&deep_node).unwrap();
    let deserialized: TreemapNode = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.children[0].children[0].name, "Level 3");
}

// ============================================================================
// AllTrafficData Tests
// ============================================================================

#[test]
fn test_all_traffic_data_creation() {
    let data = AllTrafficData {
        date_range: sample_date_range(),
        channels: vec![sample_channel_acquisition()],
        treemap_data: vec![sample_treemap_node()],
        source_medium: vec![sample_source_medium_data()],
        referrals: vec![sample_referral_data()],
    };

    assert!(!data.channels.is_empty());
    assert!(!data.treemap_data.is_empty());
    assert!(!data.referrals.is_empty());
}

#[test]
fn test_all_traffic_data_serialization() {
    let data = AllTrafficData {
        date_range: sample_date_range(),
        channels: vec![],
        treemap_data: vec![],
        source_medium: vec![],
        referrals: vec![],
    };

    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"date_range\""));
    assert!(json.contains("\"channels\":[]"));
    assert!(json.contains("\"treemap_data\":[]"));
}

// ============================================================================
// ReferralData Tests
// ============================================================================

#[test]
fn test_referral_data_creation() {
    let data = sample_referral_data();

    assert_eq!(data.source, "example.com");
    assert!(data.full_referrer.is_some());
    assert_eq!(data.users, 500);
}

#[test]
fn test_referral_data_without_full_referrer() {
    let data = ReferralData {
        source: "partner.com".to_string(),
        full_referrer: None,
        users: 200,
        new_users: 180,
        sessions: 300,
        bounce_rate: 50.0,
        pages_per_session: 2.0,
        avg_session_duration: 100.0,
        goal_conversion_rate: 1.0,
        goal_completions: 3,
        revenue: 150.0,
        percentage: 1.0,
    };

    assert!(data.full_referrer.is_none());
}

#[test]
fn test_referral_data_serialization() {
    let data = sample_referral_data();
    let json = serde_json::to_string(&data).unwrap();

    assert!(json.contains("\"source\":\"example.com\""));
    assert!(json.contains("\"full_referrer\""));
}

// ============================================================================
// CampaignAnalysis Tests
// ============================================================================

#[test]
fn test_campaign_analysis_creation() {
    let analysis = CampaignAnalysis {
        date_range: sample_date_range(),
        campaigns: vec![sample_campaign_detail_data()],
        source_medium_campaign: vec![sample_source_medium_campaign()],
        performance_over_time: vec![sample_campaign_performance_data()],
    };

    assert!(!analysis.campaigns.is_empty());
    assert!(!analysis.source_medium_campaign.is_empty());
    assert!(!analysis.performance_over_time.is_empty());
}

#[test]
fn test_campaign_analysis_serialization() {
    let analysis = CampaignAnalysis {
        date_range: sample_date_range(),
        campaigns: vec![],
        source_medium_campaign: vec![],
        performance_over_time: vec![],
    };

    let json = serde_json::to_string(&analysis).unwrap();
    assert!(json.contains("\"campaigns\":[]"));
}

// ============================================================================
// CampaignDetailData Tests
// ============================================================================

#[test]
fn test_campaign_detail_data_creation() {
    let data = sample_campaign_detail_data();

    assert_eq!(data.campaign, "summer_sale_2024");
    assert_eq!(data.source, "google");
    assert_eq!(data.medium, "cpc");
    assert!(data.cost.is_some());
    assert!(data.roas.is_some());
}

#[test]
fn test_campaign_detail_data_without_optional_fields() {
    let data = CampaignDetailData {
        campaign: "organic_campaign".to_string(),
        source: "direct".to_string(),
        medium: "(none)".to_string(),
        users: 1000,
        new_users: 900,
        sessions: 1500,
        bounce_rate: 40.0,
        pages_per_session: 3.0,
        avg_session_duration: 150.0,
        goal_completions: 50,
        goal_conversion_rate: 3.3,
        goal_value: 2500.0,
        transactions: 25,
        revenue: 3750.0,
        cost: None,
        roas: None,
        cpc: None,
        cpm: None,
    };

    assert!(data.cost.is_none());
    assert!(data.roas.is_none());
    assert!(data.cpc.is_none());
    assert!(data.cpm.is_none());
}

#[test]
fn test_campaign_detail_data_serialization() {
    let data = sample_campaign_detail_data();
    let json = serde_json::to_string(&data).unwrap();

    assert!(json.contains("\"campaign\":\"summer_sale_2024\""));
    assert!(json.contains("\"cost\":2500"));
}

// ============================================================================
// SourceMediumCampaign Tests
// ============================================================================

#[test]
fn test_source_medium_campaign_creation() {
    let data = sample_source_medium_campaign();

    assert_eq!(data.source, "google");
    assert_eq!(data.medium, "cpc");
    assert_eq!(data.campaign, "brand_awareness");
    assert!(data.ad_content.is_some());
    assert!(data.keyword.is_some());
}

#[test]
fn test_source_medium_campaign_without_optional_fields() {
    let data = SourceMediumCampaign {
        source: "facebook".to_string(),
        medium: "social".to_string(),
        campaign: "awareness".to_string(),
        ad_content: None,
        keyword: None,
        sessions: 500,
        conversions: 20,
        revenue: 1000.0,
    };

    assert!(data.ad_content.is_none());
    assert!(data.keyword.is_none());
}

#[test]
fn test_source_medium_campaign_serialization() {
    let data = sample_source_medium_campaign();
    let json = serde_json::to_string(&data).unwrap();

    assert!(json.contains("\"source\":\"google\""));
    assert!(json.contains("\"ad_content\""));
    assert!(json.contains("\"keyword\""));
}

// ============================================================================
// CampaignPerformanceData Tests
// ============================================================================

#[test]
fn test_campaign_performance_data_creation() {
    let data = sample_campaign_performance_data();

    assert_eq!(data.date, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
    assert_eq!(data.campaign, "summer_sale_2024");
    assert_eq!(data.sessions, 450);
    assert!(data.cost.is_some());
}

#[test]
fn test_campaign_performance_data_without_cost() {
    let data = CampaignPerformanceData {
        date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        campaign: "organic_campaign".to_string(),
        sessions: 200,
        conversions: 10,
        revenue: 500.0,
        cost: None,
    };

    assert!(data.cost.is_none());
}

#[test]
fn test_campaign_performance_data_serialization() {
    let data = sample_campaign_performance_data();
    let json = serde_json::to_string(&data).unwrap();

    assert!(json.contains("\"date\":\"2024-01-15\""));
    assert!(json.contains("\"campaign\":\"summer_sale_2024\""));
}

// ============================================================================
// SocialAnalysis Tests
// ============================================================================

#[test]
fn test_social_analysis_creation() {
    let analysis = SocialAnalysis {
        date_range: sample_date_range(),
        network_referrals: vec![sample_social_network_data()],
        landing_pages: vec![sample_social_landing_page()],
        conversions: vec![sample_social_conversion()],
        user_flow: vec![sample_social_flow_node()],
        plugins: vec![sample_social_plugin_data()],
    };

    assert!(!analysis.network_referrals.is_empty());
    assert!(!analysis.landing_pages.is_empty());
    assert!(!analysis.conversions.is_empty());
    assert!(!analysis.user_flow.is_empty());
    assert!(!analysis.plugins.is_empty());
}

#[test]
fn test_social_analysis_serialization() {
    let analysis = SocialAnalysis {
        date_range: sample_date_range(),
        network_referrals: vec![],
        landing_pages: vec![],
        conversions: vec![],
        user_flow: vec![],
        plugins: vec![],
    };

    let json = serde_json::to_string(&analysis).unwrap();
    assert!(json.contains("\"network_referrals\":[]"));
}

// ============================================================================
// SocialNetworkData Tests
// ============================================================================

#[test]
fn test_social_network_data_creation() {
    let data = sample_social_network_data();

    assert_eq!(data.social_network, "Facebook");
    assert_eq!(data.sessions, 2500);
    assert_eq!(data.users, 2200);
    assert_eq!(data.pageviews, 7500);
}

#[test]
fn test_social_network_data_serialization() {
    let data = sample_social_network_data();
    let json = serde_json::to_string(&data).unwrap();

    assert!(json.contains("\"social_network\":\"Facebook\""));
    assert!(json.contains("\"sessions\":2500"));
}

#[test]
fn test_social_network_data_various_networks() {
    let networks = vec!["Facebook", "Twitter", "LinkedIn", "Instagram", "Pinterest", "TikTok"];

    for network_name in networks {
        let data = SocialNetworkData {
            social_network: network_name.to_string(),
            sessions: 100,
            users: 90,
            pageviews: 300,
            avg_session_duration: 120.0,
            pages_per_session: 3.0,
            percentage: 10.0,
        };

        assert_eq!(data.social_network, network_name);
    }
}

// ============================================================================
// SocialLandingPage Tests
// ============================================================================

#[test]
fn test_social_landing_page_creation() {
    let data = sample_social_landing_page();

    assert_eq!(data.landing_page, "/blog/rust-tutorial");
    assert_eq!(data.social_network, "Twitter");
    assert_eq!(data.sessions, 800);
}

#[test]
fn test_social_landing_page_serialization() {
    let data = sample_social_landing_page();
    let json = serde_json::to_string(&data).unwrap();

    assert!(json.contains("\"landing_page\":\"/blog/rust-tutorial\""));
    assert!(json.contains("\"social_network\":\"Twitter\""));
}

// ============================================================================
// SocialConversion Tests
// ============================================================================

#[test]
fn test_social_conversion_creation() {
    let data = sample_social_conversion();

    assert_eq!(data.social_network, "LinkedIn");
    assert_eq!(data.assisted_conversions, 25);
    assert!((data.assisted_value - 1250.0).abs() < f64::EPSILON);
}

#[test]
fn test_social_conversion_serialization() {
    let data = sample_social_conversion();
    let json = serde_json::to_string(&data).unwrap();

    assert!(json.contains("\"social_network\":\"LinkedIn\""));
    assert!(json.contains("\"assisted_conversions\":25"));
}

// ============================================================================
// SocialFlowNode Tests
// ============================================================================

#[test]
fn test_social_flow_node_creation() {
    let node = sample_social_flow_node();

    assert_eq!(node.node_name, "Homepage");
    assert_eq!(node.node_type, "landing");
    assert_eq!(node.sessions, 1500);
    assert_eq!(node.exits, 300);
}

#[test]
fn test_social_flow_node_serialization() {
    let node = sample_social_flow_node();
    let json = serde_json::to_string(&node).unwrap();

    assert!(json.contains("\"node_name\":\"Homepage\""));
    assert!(json.contains("\"node_type\":\"landing\""));
}

// ============================================================================
// SocialPluginData Tests
// ============================================================================

#[test]
fn test_social_plugin_data_creation() {
    let data = sample_social_plugin_data();

    assert_eq!(data.social_source, "Facebook");
    assert_eq!(data.social_action, "like");
    assert_eq!(data.social_entity, "/products/rust-course");
    assert_eq!(data.unique_actions, 250);
    assert_eq!(data.total_actions, 380);
}

#[test]
fn test_social_plugin_data_serialization() {
    let data = sample_social_plugin_data();
    let json = serde_json::to_string(&data).unwrap();

    assert!(json.contains("\"social_source\":\"Facebook\""));
    assert!(json.contains("\"social_action\":\"like\""));
}

// ============================================================================
// GoogleAdsData Tests
// ============================================================================

#[test]
fn test_google_ads_data_creation() {
    let data = GoogleAdsData {
        date_range: sample_date_range(),
        campaigns: vec![sample_google_ads_campaign()],
        ad_groups: vec![sample_google_ads_ad_group()],
        keywords: vec![sample_google_ads_keyword()],
        search_queries: vec![sample_google_ads_search_query()],
        display_targeting: vec![sample_google_ads_display_target()],
        performance_trend: vec![sample_google_ads_performance_trend()],
    };

    assert!(!data.campaigns.is_empty());
    assert!(!data.ad_groups.is_empty());
    assert!(!data.keywords.is_empty());
}

#[test]
fn test_google_ads_data_serialization() {
    let data = GoogleAdsData {
        date_range: sample_date_range(),
        campaigns: vec![],
        ad_groups: vec![],
        keywords: vec![],
        search_queries: vec![],
        display_targeting: vec![],
        performance_trend: vec![],
    };

    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"campaigns\":[]"));
}

// ============================================================================
// GoogleAdsCampaign Tests
// ============================================================================

#[test]
fn test_google_ads_campaign_creation() {
    let campaign = sample_google_ads_campaign();

    assert_eq!(campaign.campaign_id, "12345678");
    assert_eq!(campaign.campaign_name, "Brand Campaign");
    assert_eq!(campaign.campaign_type, "Search");
    assert_eq!(campaign.impressions, 100000);
    assert_eq!(campaign.clicks, 5000);
}

#[test]
fn test_google_ads_campaign_metrics() {
    let campaign = sample_google_ads_campaign();

    // CTR should be clicks / impressions * 100
    assert!((campaign.ctr - 5.0).abs() < f64::EPSILON);
    // CPC should be cost / clicks
    assert!((campaign.cpc - 0.50).abs() < f64::EPSILON);
}

#[test]
fn test_google_ads_campaign_serialization() {
    let campaign = sample_google_ads_campaign();
    let json = serde_json::to_string(&campaign).unwrap();

    assert!(json.contains("\"campaign_id\":\"12345678\""));
    assert!(json.contains("\"campaign_name\":\"Brand Campaign\""));
}

// ============================================================================
// GoogleAdsAdGroup Tests
// ============================================================================

#[test]
fn test_google_ads_ad_group_creation() {
    let ad_group = sample_google_ads_ad_group();

    assert_eq!(ad_group.ad_group_id, "987654321");
    assert_eq!(ad_group.ad_group_name, "Rust Programming Courses");
    assert_eq!(ad_group.campaign_name, "Brand Campaign");
}

#[test]
fn test_google_ads_ad_group_serialization() {
    let ad_group = sample_google_ads_ad_group();
    let json = serde_json::to_string(&ad_group).unwrap();

    assert!(json.contains("\"ad_group_id\":\"987654321\""));
    assert!(json.contains("\"ad_group_name\":\"Rust Programming Courses\""));
}

// ============================================================================
// GoogleAdsKeyword Tests
// ============================================================================

#[test]
fn test_google_ads_keyword_creation() {
    let keyword = sample_google_ads_keyword();

    assert_eq!(keyword.keyword, "rust programming course");
    assert_eq!(keyword.keyword_match_type, "exact");
    assert!(keyword.quality_score.is_some());
    assert_eq!(keyword.quality_score.unwrap(), 9);
}

#[test]
fn test_google_ads_keyword_without_quality_score() {
    let keyword = GoogleAdsKeyword {
        keyword: "new keyword".to_string(),
        keyword_match_type: "broad".to_string(),
        ad_group: "Test Ad Group".to_string(),
        campaign: "Test Campaign".to_string(),
        impressions: 100,
        clicks: 10,
        cost: 5.0,
        ctr: 10.0,
        cpc: 0.50,
        quality_score: None,
        sessions: 9,
        bounce_rate: 30.0,
        conversions: 1,
        revenue: 50.0,
    };

    assert!(keyword.quality_score.is_none());
}

#[test]
fn test_google_ads_keyword_serialization() {
    let keyword = sample_google_ads_keyword();
    let json = serde_json::to_string(&keyword).unwrap();

    assert!(json.contains("\"keyword\":\"rust programming course\""));
    assert!(json.contains("\"quality_score\":9"));
}

// ============================================================================
// GoogleAdsSearchQuery Tests
// ============================================================================

#[test]
fn test_google_ads_search_query_creation() {
    let query = sample_google_ads_search_query();

    assert_eq!(query.search_query, "best rust programming course online");
    assert_eq!(query.matched_keyword, "rust programming course");
    assert_eq!(query.impressions, 1000);
}

#[test]
fn test_google_ads_search_query_serialization() {
    let query = sample_google_ads_search_query();
    let json = serde_json::to_string(&query).unwrap();

    assert!(json.contains("\"search_query\":\"best rust programming course online\""));
}

// ============================================================================
// GoogleAdsDisplayTarget Tests
// ============================================================================

#[test]
fn test_google_ads_display_target_creation() {
    let target = sample_google_ads_display_target();

    assert_eq!(target.targeting_type, "audience");
    assert_eq!(target.target, "In-market: Software Developers");
}

#[test]
fn test_google_ads_display_target_serialization() {
    let target = sample_google_ads_display_target();
    let json = serde_json::to_string(&target).unwrap();

    assert!(json.contains("\"targeting_type\":\"audience\""));
}

// ============================================================================
// GoogleAdsPerformanceTrend Tests
// ============================================================================

#[test]
fn test_google_ads_performance_trend_creation() {
    let trend = sample_google_ads_performance_trend();

    assert_eq!(trend.date, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
    assert_eq!(trend.impressions, 5000);
    assert_eq!(trend.clicks, 250);
}

#[test]
fn test_google_ads_performance_trend_serialization() {
    let trend = sample_google_ads_performance_trend();
    let json = serde_json::to_string(&trend).unwrap();

    assert!(json.contains("\"date\":\"2024-01-15\""));
    assert!(json.contains("\"impressions\":5000"));
}

// ============================================================================
// SearchConsoleData Tests
// ============================================================================

#[test]
fn test_search_console_data_creation() {
    let data = SearchConsoleData {
        date_range: sample_date_range(),
        queries: vec![sample_search_console_query()],
        pages: vec![sample_search_console_page()],
        countries: vec![sample_search_console_country()],
        devices: vec![sample_search_console_device()],
        search_appearance: vec![sample_search_console_appearance()],
        performance_trend: vec![sample_search_console_performance()],
    };

    assert!(!data.queries.is_empty());
    assert!(!data.pages.is_empty());
    assert!(!data.countries.is_empty());
}

#[test]
fn test_search_console_data_serialization() {
    let data = SearchConsoleData {
        date_range: sample_date_range(),
        queries: vec![],
        pages: vec![],
        countries: vec![],
        devices: vec![],
        search_appearance: vec![],
        performance_trend: vec![],
    };

    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"queries\":[]"));
}

// ============================================================================
// SearchConsoleQuery Tests
// ============================================================================

#[test]
fn test_search_console_query_creation() {
    let query = sample_search_console_query();

    assert_eq!(query.query, "rust programming tutorial");
    assert_eq!(query.clicks, 500);
    assert_eq!(query.impressions, 10000);
    assert!(query.sessions.is_some());
    assert!(query.goal_completions.is_some());
}

#[test]
fn test_search_console_query_without_optional_fields() {
    let query = SearchConsoleQuery {
        query: "test query".to_string(),
        clicks: 100,
        impressions: 2000,
        ctr: 5.0,
        position: 5.0,
        sessions: None,
        goal_completions: None,
    };

    assert!(query.sessions.is_none());
    assert!(query.goal_completions.is_none());
}

#[test]
fn test_search_console_query_serialization() {
    let query = sample_search_console_query();
    let json = serde_json::to_string(&query).unwrap();

    assert!(json.contains("\"query\":\"rust programming tutorial\""));
    assert!(json.contains("\"sessions\":480"));
}

// ============================================================================
// SearchConsolePage Tests
// ============================================================================

#[test]
fn test_search_console_page_creation() {
    let page = sample_search_console_page();

    assert_eq!(page.page, "/tutorials/rust-basics");
    assert_eq!(page.clicks, 1200);
    assert_eq!(page.impressions, 25000);
}

#[test]
fn test_search_console_page_serialization() {
    let page = sample_search_console_page();
    let json = serde_json::to_string(&page).unwrap();

    assert!(json.contains("\"page\":\"/tutorials/rust-basics\""));
}

// ============================================================================
// SearchConsoleCountry Tests
// ============================================================================

#[test]
fn test_search_console_country_creation() {
    let country = sample_search_console_country();

    assert_eq!(country.country, "United States");
    assert_eq!(country.country_code, "US");
    assert_eq!(country.clicks, 5000);
}

#[test]
fn test_search_console_country_serialization() {
    let country = sample_search_console_country();
    let json = serde_json::to_string(&country).unwrap();

    assert!(json.contains("\"country\":\"United States\""));
    assert!(json.contains("\"country_code\":\"US\""));
}

#[test]
fn test_search_console_country_various_countries() {
    let countries = vec![
        ("United States", "US"),
        ("Germany", "DE"),
        ("Japan", "JP"),
        ("Brazil", "BR"),
        ("United Kingdom", "GB"),
    ];

    for (country_name, code) in countries {
        let country = SearchConsoleCountry {
            country: country_name.to_string(),
            country_code: code.to_string(),
            clicks: 100,
            impressions: 2000,
            ctr: 5.0,
            position: 4.0,
        };

        assert_eq!(country.country, country_name);
        assert_eq!(country.country_code, code);
    }
}

// ============================================================================
// SearchConsoleDevice Tests
// ============================================================================

#[test]
fn test_search_console_device_creation() {
    let device = sample_search_console_device();

    assert_eq!(device.device, "DESKTOP");
    assert_eq!(device.clicks, 3500);
    assert_eq!(device.impressions, 70000);
}

#[test]
fn test_search_console_device_all_types() {
    let devices = vec!["DESKTOP", "MOBILE", "TABLET"];

    for device_type in devices {
        let device = SearchConsoleDevice {
            device: device_type.to_string(),
            clicks: 100,
            impressions: 2000,
            ctr: 5.0,
            position: 4.0,
        };

        assert_eq!(device.device, device_type);
    }
}

#[test]
fn test_search_console_device_serialization() {
    let device = sample_search_console_device();
    let json = serde_json::to_string(&device).unwrap();

    assert!(json.contains("\"device\":\"DESKTOP\""));
}

// ============================================================================
// SearchConsoleAppearance Tests
// ============================================================================

#[test]
fn test_search_console_appearance_creation() {
    let appearance = sample_search_console_appearance();

    assert_eq!(appearance.search_appearance, "AMP_ARTICLE");
    assert_eq!(appearance.clicks, 800);
    assert_eq!(appearance.impressions, 20000);
}

#[test]
fn test_search_console_appearance_types() {
    let appearances = vec![
        "AMP_ARTICLE",
        "RICH_RESULT",
        "FAQ_RESULT",
        "HOW_TO_RESULT",
        "VIDEO_RESULT",
    ];

    for appearance_type in appearances {
        let appearance = SearchConsoleAppearance {
            search_appearance: appearance_type.to_string(),
            clicks: 100,
            impressions: 2000,
            ctr: 5.0,
        };

        assert_eq!(appearance.search_appearance, appearance_type);
    }
}

#[test]
fn test_search_console_appearance_serialization() {
    let appearance = sample_search_console_appearance();
    let json = serde_json::to_string(&appearance).unwrap();

    assert!(json.contains("\"search_appearance\":\"AMP_ARTICLE\""));
}

// ============================================================================
// SearchConsolePerformance Tests
// ============================================================================

#[test]
fn test_search_console_performance_creation() {
    let performance = sample_search_console_performance();

    assert_eq!(performance.date, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
    assert_eq!(performance.clicks, 350);
    assert_eq!(performance.impressions, 7000);
}

#[test]
fn test_search_console_performance_serialization() {
    let performance = sample_search_console_performance();
    let json = serde_json::to_string(&performance).unwrap();

    assert!(json.contains("\"date\":\"2024-01-15\""));
    assert!(json.contains("\"clicks\":350"));
}

// ============================================================================
// UtmAnalysis Tests
// ============================================================================

#[test]
fn test_utm_analysis_creation() {
    let analysis = UtmAnalysis {
        date_range: sample_date_range(),
        by_source: vec![sample_utm_source_data()],
        by_medium: vec![sample_utm_medium_data()],
        by_campaign: vec![sample_utm_campaign_data()],
        by_content: vec![sample_utm_content_data()],
        by_term: vec![sample_utm_term_data()],
    };

    assert!(!analysis.by_source.is_empty());
    assert!(!analysis.by_medium.is_empty());
    assert!(!analysis.by_campaign.is_empty());
}

#[test]
fn test_utm_analysis_serialization() {
    let analysis = UtmAnalysis {
        date_range: sample_date_range(),
        by_source: vec![],
        by_medium: vec![],
        by_campaign: vec![],
        by_content: vec![],
        by_term: vec![],
    };

    let json = serde_json::to_string(&analysis).unwrap();
    assert!(json.contains("\"by_source\":[]"));
}

// ============================================================================
// UtmSourceData Tests
// ============================================================================

#[test]
fn test_utm_source_data_creation() {
    let data = sample_utm_source_data();

    assert_eq!(data.utm_source, "newsletter");
    assert_eq!(data.sessions, 2500);
    assert_eq!(data.users, 2200);
}

#[test]
fn test_utm_source_data_serialization() {
    let data = sample_utm_source_data();
    let json = serde_json::to_string(&data).unwrap();

    assert!(json.contains("\"utm_source\":\"newsletter\""));
}

// ============================================================================
// UtmMediumData Tests
// ============================================================================

#[test]
fn test_utm_medium_data_creation() {
    let data = sample_utm_medium_data();

    assert_eq!(data.utm_medium, "email");
    assert_eq!(data.sessions, 3000);
    assert_eq!(data.users, 2700);
}

#[test]
fn test_utm_medium_data_serialization() {
    let data = sample_utm_medium_data();
    let json = serde_json::to_string(&data).unwrap();

    assert!(json.contains("\"utm_medium\":\"email\""));
}

// ============================================================================
// UtmCampaignData Tests
// ============================================================================

#[test]
fn test_utm_campaign_data_creation() {
    let data = sample_utm_campaign_data();

    assert_eq!(data.utm_campaign, "black_friday_2024");
    assert_eq!(data.sessions, 5000);
    assert_eq!(data.users, 4500);
}

#[test]
fn test_utm_campaign_data_serialization() {
    let data = sample_utm_campaign_data();
    let json = serde_json::to_string(&data).unwrap();

    assert!(json.contains("\"utm_campaign\":\"black_friday_2024\""));
}

// ============================================================================
// UtmContentData Tests
// ============================================================================

#[test]
fn test_utm_content_data_creation() {
    let data = sample_utm_content_data();

    assert_eq!(data.utm_content, "hero_banner_v2");
    assert_eq!(data.sessions, 1500);
    assert_eq!(data.users, 1400);
}

#[test]
fn test_utm_content_data_serialization() {
    let data = sample_utm_content_data();
    let json = serde_json::to_string(&data).unwrap();

    assert!(json.contains("\"utm_content\":\"hero_banner_v2\""));
}

// ============================================================================
// UtmTermData Tests
// ============================================================================

#[test]
fn test_utm_term_data_creation() {
    let data = sample_utm_term_data();

    assert_eq!(data.utm_term, "rust+programming");
    assert_eq!(data.sessions, 800);
    assert_eq!(data.users, 750);
}

#[test]
fn test_utm_term_data_serialization() {
    let data = sample_utm_term_data();
    let json = serde_json::to_string(&data).unwrap();

    assert!(json.contains("\"utm_term\":\"rust+programming\""));
}

// ============================================================================
// Edge Cases and Complex Scenarios
// ============================================================================

#[test]
fn test_unicode_in_campaign_names() {
    let campaign = CampaignDetailData {
        campaign: "夏季セール2024".to_string(),
        source: "google".to_string(),
        medium: "cpc".to_string(),
        users: 1000,
        new_users: 900,
        sessions: 1500,
        bounce_rate: 40.0,
        pages_per_session: 3.0,
        avg_session_duration: 150.0,
        goal_completions: 50,
        goal_conversion_rate: 3.3,
        goal_value: 2500.0,
        transactions: 25,
        revenue: 3750.0,
        cost: None,
        roas: None,
        cpc: None,
        cpm: None,
    };

    let json = serde_json::to_string(&campaign).unwrap();
    let deserialized: CampaignDetailData = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.campaign, "夏季セール2024");
}

#[test]
fn test_empty_string_fields() {
    let data = SourceMediumData {
        source: "".to_string(),
        medium: "".to_string(),
        users: 0,
        new_users: 0,
        sessions: 0,
        bounce_rate: 0.0,
        pages_per_session: 0.0,
        avg_session_duration: 0.0,
        goal_conversion_rate: 0.0,
        goal_completions: 0,
        revenue: 0.0,
        percentage: 0.0,
    };

    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"source\":\"\""));
    assert!(json.contains("\"medium\":\"\""));
}

#[test]
fn test_large_values() {
    let campaign = GoogleAdsCampaign {
        campaign_id: "999999999".to_string(),
        campaign_name: "Large Scale Campaign".to_string(),
        campaign_type: "Search".to_string(),
        impressions: u64::MAX / 2,
        clicks: u64::MAX / 4,
        cost: 1_000_000_000.0,
        ctr: 50.0,
        cpc: 0.001,
        sessions: u64::MAX / 4,
        bounce_rate: 1.0,
        pages_per_session: 100.0,
        goal_completions: u64::MAX / 8,
        goal_conversion_rate: 99.9,
        revenue: 1_000_000_000_000.0,
        roas: 1000.0,
    };

    let json = serde_json::to_string(&campaign).unwrap();
    let deserialized: GoogleAdsCampaign = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.impressions, u64::MAX / 2);
}

#[test]
fn test_special_characters_in_urls() {
    let page = SearchConsolePage {
        page: "/products?category=electronics&sort=price%20desc".to_string(),
        clicks: 100,
        impressions: 2000,
        ctr: 5.0,
        position: 3.5,
    };

    let json = serde_json::to_string(&page).unwrap();
    let deserialized: SearchConsolePage = serde_json::from_str(&json).unwrap();

    assert!(deserialized.page.contains("category=electronics"));
}

#[test]
fn test_floating_point_precision() {
    let totals = AcquisitionTotals {
        users: 100,
        new_users: 90,
        sessions: 150,
        bounce_rate: 33.333333333333336,
        pages_per_session: 2.7777777777777777,
        avg_session_duration: 123.45678901234567,
        goal_completions: 10,
        revenue: 999.9999999999999,
    };

    let json = serde_json::to_string(&totals).unwrap();
    let deserialized: AcquisitionTotals = serde_json::from_str(&json).unwrap();

    assert!((deserialized.bounce_rate - 33.333333333333336).abs() < 1e-10);
}

#[test]
fn test_negative_comparison_values() {
    let comparison = AcquisitionComparison {
        users_change: -100.0,
        new_users_change: -100.0,
        sessions_change: -100.0,
        bounce_rate_change: 100.0,
        goal_completions_change: -100.0,
        revenue_change: -100.0,
    };

    let json = serde_json::to_string(&comparison).unwrap();
    let deserialized: AcquisitionComparison = serde_json::from_str(&json).unwrap();

    assert!(deserialized.users_change < 0.0);
    assert!(deserialized.bounce_rate_change > 0.0);
}

#[test]
fn test_many_channels() {
    let channels: Vec<ChannelAcquisition> = (0..100).map(|i| {
        ChannelAcquisition {
            channel: if i % 13 == 0 { Channel::Direct }
                else if i % 13 == 1 { Channel::OrganicSearch }
                else if i % 13 == 2 { Channel::PaidSearch }
                else if i % 13 == 3 { Channel::Display }
                else if i % 13 == 4 { Channel::Social }
                else if i % 13 == 5 { Channel::Email }
                else if i % 13 == 6 { Channel::Referral }
                else if i % 13 == 7 { Channel::AffiliateMarketing }
                else if i % 13 == 8 { Channel::Video }
                else if i % 13 == 9 { Channel::Audio }
                else if i % 13 == 10 { Channel::Sms }
                else if i % 13 == 11 { Channel::Mobile }
                else { Channel::Other },
            users: i as u64 * 100,
            new_users: i as u64 * 80,
            sessions: i as u64 * 150,
            bounce_rate: 50.0 - (i as f64 * 0.5),
            pages_per_session: 2.0 + (i as f64 * 0.1),
            avg_session_duration: 120.0 + (i as f64 * 5.0),
            goal_conversion_rate: 1.0 + (i as f64 * 0.1),
            goal_completions: i as u64 * 10,
            goal_value: i as f64 * 500.0,
            transactions: i as u64 * 5,
            revenue: i as f64 * 750.0,
            percentage: 1.0,
        }
    }).collect();

    let overview = AcquisitionOverview {
        date_range: sample_date_range(),
        channels,
        source_medium: vec![],
        top_channels_trend: vec![],
        totals: sample_acquisition_totals(),
        comparison: None,
    };

    assert_eq!(overview.channels.len(), 100);

    let json = serde_json::to_string(&overview).unwrap();
    let deserialized: AcquisitionOverview = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.channels.len(), 100);
}

#[test]
fn test_date_range_boundaries() {
    let _date_range = DateRange {
        start_date: NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2099, 12, 31).unwrap(),
    };

    let trend = ChannelTrendData {
        date: NaiveDate::from_ymd_opt(2050, 6, 15).unwrap(),
        direct: 100,
        organic_search: 200,
        paid_search: 50,
        social: 75,
        referral: 25,
        email: 30,
        display: 20,
        other: 10,
    };

    let json = serde_json::to_string(&trend).unwrap();
    assert!(json.contains("2050-06-15"));
}

#[test]
fn test_clone_all_major_types() {
    let overview = sample_acquisition_overview();
    let cloned = overview.clone();
    assert_eq!(overview.totals.users, cloned.totals.users);

    let treemap = sample_treemap_node();
    let cloned_treemap = treemap.clone();
    assert_eq!(treemap.name, cloned_treemap.name);

    let campaign = sample_campaign_detail_data();
    let cloned_campaign = campaign.clone();
    assert_eq!(campaign.campaign, cloned_campaign.campaign);
}

#[test]
fn test_debug_trait_implementation() {
    let overview = sample_acquisition_overview();
    let debug_str = format!("{:?}", overview);
    assert!(debug_str.contains("AcquisitionOverview"));

    let channel = sample_channel_acquisition();
    let debug_str = format!("{:?}", channel);
    assert!(debug_str.contains("ChannelAcquisition"));
}
