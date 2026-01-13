//! Tests for conversion analytics models
//!
//! This module contains comprehensive tests for all conversion model types
//! including serialization, deserialization, and edge cases.

use chrono::{NaiveDate, Utc};
use rustanalytics::models::conversions::*;
use rustanalytics::models::DateRange;
use uuid::Uuid;

// ============================================================================
// Helper Functions
// ============================================================================

fn sample_date_range() -> DateRange {
    DateRange {
        start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
    }
}

fn sample_goal_data() -> GoalData {
    GoalData {
        goal_id: 1,
        goal_name: "Newsletter Signup".to_string(),
        goal_type: GoalType::Event,
        completions: 500,
        value: 2500.0,
        conversion_rate: 3.5,
        abandonment_rate: 45.0,
        percentage_of_total: 25.0,
    }
}

fn sample_goal_trend_data() -> GoalTrendData {
    GoalTrendData {
        date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        completions: 50,
        value: 250.0,
        conversion_rate: 3.2,
    }
}

fn sample_goals_comparison() -> GoalsComparison {
    GoalsComparison {
        completions_change: 15.5,
        value_change: 22.0,
        conversion_rate_change: 0.5,
    }
}

fn sample_goals_overview() -> GoalsOverview {
    GoalsOverview {
        date_range: sample_date_range(),
        goals: vec![sample_goal_data()],
        total_completions: 2000,
        total_value: 10000.0,
        overall_conversion_rate: 4.2,
        goal_trend: vec![sample_goal_trend_data()],
        comparison: Some(sample_goals_comparison()),
    }
}

fn sample_goal_url_data() -> GoalUrlData {
    GoalUrlData {
        goal_completion_location: "/thank-you".to_string(),
        goal_previous_step: Some("/checkout".to_string()),
        completions: 150,
        value: 750.0,
        percentage: 30.0,
    }
}

fn sample_reverse_path_data() -> ReversePathData {
    ReversePathData {
        goal_completion_location: "/thank-you".to_string(),
        step_minus_1: Some("/checkout".to_string()),
        step_minus_2: Some("/cart".to_string()),
        step_minus_3: Some("/product".to_string()),
        completions: 100,
        value: 500.0,
    }
}

fn sample_funnel_step() -> FunnelStep {
    FunnelStep {
        step_number: 1,
        step_name: "Product View".to_string(),
        step_url: "/product".to_string(),
        entrances: 1000,
        exits: 200,
        continuation: 800,
        continuation_rate: 80.0,
        abandonment_rate: 20.0,
    }
}

fn sample_funnel_vis_step() -> FunnelVisStep {
    FunnelVisStep {
        step_number: 1,
        step_name: "Product View".to_string(),
        value: 1000,
        percentage: 100.0,
    }
}

fn sample_drop_off_destination() -> DropOffDestination {
    DropOffDestination {
        destination: "/homepage".to_string(),
        count: 50,
        percentage: 25.0,
    }
}

fn sample_funnel_drop_off() -> FunnelDropOff {
    FunnelDropOff {
        from_step: 1,
        count: 200,
        destinations: vec![sample_drop_off_destination()],
    }
}

fn sample_backfill_source() -> BackfillSource {
    BackfillSource {
        source: "/search".to_string(),
        count: 30,
        percentage: 15.0,
    }
}

fn sample_funnel_backfill() -> FunnelBackfill {
    FunnelBackfill {
        to_step: 2,
        sources: vec![sample_backfill_source()],
    }
}

fn sample_funnel_visualization() -> FunnelVisualization {
    FunnelVisualization {
        steps: vec![sample_funnel_vis_step()],
        drop_offs: vec![sample_funnel_drop_off()],
        backfills: vec![sample_funnel_backfill()],
    }
}

fn sample_goal_funnel_data() -> GoalFunnelData {
    GoalFunnelData {
        date_range: sample_date_range(),
        goal_id: 1,
        goal_name: "Purchase".to_string(),
        funnel_steps: vec![sample_funnel_step()],
        overall_conversion_rate: 5.5,
        funnel_visualization: sample_funnel_visualization(),
    }
}

fn sample_engagement_score_bucket() -> EngagementScoreBucket {
    EngagementScoreBucket {
        score_range: "80-100".to_string(),
        sessions: 500,
        percentage: 25.0,
        conversion_rate: 8.5,
    }
}

fn sample_smart_goal_segment() -> SmartGoalSegment {
    SmartGoalSegment {
        segment_name: "High-value Users".to_string(),
        sessions: 1000,
        smart_goal_completions: 150,
        conversion_rate: 15.0,
    }
}

fn sample_smart_goals_data() -> SmartGoalsData {
    SmartGoalsData {
        date_range: sample_date_range(),
        smart_goal_completions: 500,
        smart_goal_conversion_rate: 5.0,
        smart_goal_value: 2500.0,
        engagement_score_distribution: vec![sample_engagement_score_bucket()],
        top_converting_segments: vec![sample_smart_goal_segment()],
    }
}

fn sample_assisted_vs_last_click() -> AssistedVsLastClick {
    AssistedVsLastClick {
        assisted_conversions: 300,
        assisted_value: 15000.0,
        last_click_conversions: 500,
        last_click_value: 25000.0,
        assisted_to_last_click_ratio: 0.6,
    }
}

fn sample_channel_contribution() -> ChannelContribution {
    ChannelContribution {
        channel: "Organic Search".to_string(),
        assisted_conversions: 150,
        assisted_conversion_value: 7500.0,
        last_interaction_conversions: 200,
        last_interaction_value: 10000.0,
        first_interaction_conversions: 180,
        first_interaction_value: 9000.0,
        assisted_to_last_click_ratio: 0.75,
    }
}

fn sample_multi_channel_overview() -> MultiChannelOverview {
    MultiChannelOverview {
        date_range: sample_date_range(),
        total_conversions: 1000,
        total_conversion_value: 50000.0,
        avg_time_to_conversion: 3.5,
        avg_path_length: 2.8,
        assisted_vs_last_click: sample_assisted_vs_last_click(),
        channel_contribution: vec![sample_channel_contribution()],
    }
}

fn sample_mcf_channel_data() -> McfChannelData {
    McfChannelData {
        channel: "Paid Search".to_string(),
        assisted_conversions: 100,
        assisted_conversion_value: 5000.0,
        last_interaction_conversions: 150,
        last_interaction_value: 7500.0,
        assisted_to_last_click_ratio: 0.67,
    }
}

fn sample_mcf_source_medium_data() -> McfSourceMediumData {
    McfSourceMediumData {
        source_medium: "google / cpc".to_string(),
        assisted_conversions: 80,
        assisted_conversion_value: 4000.0,
        last_interaction_conversions: 120,
        last_interaction_value: 6000.0,
    }
}

fn sample_mcf_source_data() -> McfSourceData {
    McfSourceData {
        source: "google".to_string(),
        assisted_conversions: 200,
        assisted_conversion_value: 10000.0,
        last_interaction_conversions: 300,
        last_interaction_value: 15000.0,
    }
}

fn sample_path_node() -> PathNode {
    PathNode {
        channel: "Organic Search".to_string(),
        source: Some("google".to_string()),
        medium: Some("organic".to_string()),
        position: PathPosition::First,
    }
}

fn sample_conversion_path_data() -> ConversionPathData {
    ConversionPathData {
        path: vec![sample_path_node()],
        conversions: 50,
        conversion_value: 2500.0,
        path_length: 3,
        percentage: 5.0,
    }
}

fn sample_path_length_bucket() -> PathLengthBucket {
    PathLengthBucket {
        path_length: 3,
        conversions: 200,
        conversion_value: 10000.0,
        percentage: 20.0,
    }
}

fn sample_time_lag_bucket() -> TimeLagBucket {
    TimeLagBucket {
        days_to_conversion: "0".to_string(),
        conversions: 300,
        conversion_value: 15000.0,
        percentage: 30.0,
    }
}

fn sample_attribution_model_result() -> AttributionModelResult {
    AttributionModelResult {
        conversions: 100.5,
        conversion_value: 5025.0,
        percentage_change_from_last_interaction: 5.5,
    }
}

fn sample_attribution_channel_data() -> AttributionChannelData {
    AttributionChannelData {
        channel: "Email".to_string(),
        last_interaction: sample_attribution_model_result(),
        last_non_direct_click: sample_attribution_model_result(),
        first_interaction: sample_attribution_model_result(),
        linear: sample_attribution_model_result(),
        time_decay: sample_attribution_model_result(),
        position_based: sample_attribution_model_result(),
        data_driven: Some(sample_attribution_model_result()),
    }
}

fn sample_custom_attribution_model() -> CustomAttributionModel {
    CustomAttributionModel {
        id: Uuid::new_v4(),
        name: "My Custom Model".to_string(),
        baseline_model: AttributionModel::PositionBased,
        lookback_window: 30,
        first_interaction_weight: 0.4,
        middle_interactions_weight: 0.2,
        last_interaction_weight: 0.4,
        time_decay_half_life: Some(7),
        created_at: Utc::now(),
    }
}

fn sample_touchpoint_data() -> TouchpointData {
    TouchpointData {
        position: 1,
        value: "google / organic".to_string(),
        interaction_type: InteractionType::Click,
        timestamp: Some(Utc::now()),
    }
}

fn sample_detailed_path_data() -> DetailedPathData {
    DetailedPathData {
        path_id: "path_001".to_string(),
        touchpoints: vec![sample_touchpoint_data()],
        conversions: 25,
        conversion_value: 1250.0,
        days_to_conversion: 2.5,
    }
}

fn sample_sankey_node() -> SankeyNode {
    SankeyNode {
        id: "node_1".to_string(),
        name: "Organic Search".to_string(),
        column: 0,
        value: 500,
    }
}

fn sample_sankey_link() -> SankeyLink {
    SankeyLink {
        source: "node_1".to_string(),
        target: "node_2".to_string(),
        value: 300,
    }
}

fn sample_sankey_data() -> SankeyData {
    SankeyData {
        nodes: vec![sample_sankey_node()],
        links: vec![sample_sankey_link()],
    }
}

// ============================================================================
// GoalType Enum Tests
// ============================================================================

#[test]
fn test_goal_type_all_variants() {
    let types = vec![
        GoalType::Destination,
        GoalType::Duration,
        GoalType::PagesPerSession,
        GoalType::Event,
        GoalType::SmartGoal,
    ];
    assert_eq!(types.len(), 5);
}

#[test]
fn test_goal_type_serialization() {
    assert_eq!(serde_json::to_string(&GoalType::Destination).unwrap(), "\"destination\"");
    assert_eq!(serde_json::to_string(&GoalType::Duration).unwrap(), "\"duration\"");
    assert_eq!(serde_json::to_string(&GoalType::PagesPerSession).unwrap(), "\"pages_per_session\"");
    assert_eq!(serde_json::to_string(&GoalType::Event).unwrap(), "\"event\"");
    assert_eq!(serde_json::to_string(&GoalType::SmartGoal).unwrap(), "\"smart_goal\"");
}

#[test]
fn test_goal_type_deserialization() {
    assert_eq!(serde_json::from_str::<GoalType>("\"destination\"").unwrap(), GoalType::Destination);
    assert_eq!(serde_json::from_str::<GoalType>("\"event\"").unwrap(), GoalType::Event);
    assert_eq!(serde_json::from_str::<GoalType>("\"smart_goal\"").unwrap(), GoalType::SmartGoal);
}

#[test]
fn test_goal_type_clone_and_copy() {
    let goal_type = GoalType::Event;
    let cloned = goal_type.clone();
    let copied = goal_type;
    assert_eq!(goal_type, cloned);
    assert_eq!(goal_type, copied);
}

#[test]
fn test_goal_type_equality() {
    assert_eq!(GoalType::Destination, GoalType::Destination);
    assert_ne!(GoalType::Destination, GoalType::Event);
}

#[test]
fn test_goal_type_debug() {
    let debug = format!("{:?}", GoalType::PagesPerSession);
    assert!(debug.contains("PagesPerSession"));
}

// ============================================================================
// GoalData Tests
// ============================================================================

#[test]
fn test_goal_data_creation() {
    let goal = sample_goal_data();
    assert_eq!(goal.goal_id, 1);
    assert_eq!(goal.goal_name, "Newsletter Signup");
    assert_eq!(goal.goal_type, GoalType::Event);
    assert_eq!(goal.completions, 500);
}

#[test]
fn test_goal_data_serialization() {
    let goal = sample_goal_data();
    let json = serde_json::to_string(&goal).unwrap();
    assert!(json.contains("\"goal_id\":1"));
    assert!(json.contains("\"goal_name\":\"Newsletter Signup\""));
    assert!(json.contains("\"goal_type\":\"event\""));
}

#[test]
fn test_goal_data_roundtrip() {
    let goal = sample_goal_data();
    let json = serde_json::to_string(&goal).unwrap();
    let deserialized: GoalData = serde_json::from_str(&json).unwrap();
    assert_eq!(goal.goal_id, deserialized.goal_id);
    assert_eq!(goal.goal_name, deserialized.goal_name);
}

#[test]
fn test_goal_data_all_goal_types() {
    let types = vec![
        GoalType::Destination,
        GoalType::Duration,
        GoalType::PagesPerSession,
        GoalType::Event,
        GoalType::SmartGoal,
    ];

    for goal_type in types {
        let goal = GoalData {
            goal_id: 1,
            goal_name: format!("{:?} Goal", goal_type),
            goal_type,
            completions: 100,
            value: 500.0,
            conversion_rate: 2.5,
            abandonment_rate: 40.0,
            percentage_of_total: 10.0,
        };
        let json = serde_json::to_string(&goal).unwrap();
        let deserialized: GoalData = serde_json::from_str(&json).unwrap();
        assert_eq!(goal.goal_type, deserialized.goal_type);
    }
}

// ============================================================================
// GoalTrendData Tests
// ============================================================================

#[test]
fn test_goal_trend_data_creation() {
    let trend = sample_goal_trend_data();
    assert_eq!(trend.date, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
    assert_eq!(trend.completions, 50);
}

#[test]
fn test_goal_trend_data_serialization() {
    let trend = sample_goal_trend_data();
    let json = serde_json::to_string(&trend).unwrap();
    assert!(json.contains("\"date\":\"2024-01-15\""));
    assert!(json.contains("\"completions\":50"));
}

// ============================================================================
// GoalsComparison Tests
// ============================================================================

#[test]
fn test_goals_comparison_creation() {
    let comparison = sample_goals_comparison();
    assert!((comparison.completions_change - 15.5).abs() < f64::EPSILON);
    assert!((comparison.value_change - 22.0).abs() < f64::EPSILON);
}

#[test]
fn test_goals_comparison_negative_values() {
    let comparison = GoalsComparison {
        completions_change: -20.0,
        value_change: -15.0,
        conversion_rate_change: -1.5,
    };
    assert!(comparison.completions_change < 0.0);
    assert!(comparison.value_change < 0.0);
}

#[test]
fn test_goals_comparison_serialization() {
    let comparison = sample_goals_comparison();
    let json = serde_json::to_string(&comparison).unwrap();
    assert!(json.contains("completions_change"));
    assert!(json.contains("value_change"));
}

// ============================================================================
// GoalsOverview Tests
// ============================================================================

#[test]
fn test_goals_overview_creation() {
    let overview = sample_goals_overview();
    assert!(!overview.goals.is_empty());
    assert_eq!(overview.total_completions, 2000);
    assert!(overview.comparison.is_some());
}

#[test]
fn test_goals_overview_without_comparison() {
    let overview = GoalsOverview {
        date_range: sample_date_range(),
        goals: vec![],
        total_completions: 0,
        total_value: 0.0,
        overall_conversion_rate: 0.0,
        goal_trend: vec![],
        comparison: None,
    };
    assert!(overview.comparison.is_none());
    assert!(overview.goals.is_empty());
}

#[test]
fn test_goals_overview_serialization() {
    let overview = sample_goals_overview();
    let json = serde_json::to_string(&overview).unwrap();
    assert!(json.contains("\"total_completions\":2000"));
    assert!(json.contains("\"goals\""));
}

// ============================================================================
// GoalUrlData Tests
// ============================================================================

#[test]
fn test_goal_url_data_creation() {
    let data = sample_goal_url_data();
    assert_eq!(data.goal_completion_location, "/thank-you");
    assert!(data.goal_previous_step.is_some());
}

#[test]
fn test_goal_url_data_without_previous_step() {
    let data = GoalUrlData {
        goal_completion_location: "/success".to_string(),
        goal_previous_step: None,
        completions: 50,
        value: 250.0,
        percentage: 10.0,
    };
    assert!(data.goal_previous_step.is_none());
}

#[test]
fn test_goal_url_data_serialization() {
    let data = sample_goal_url_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"goal_completion_location\":\"/thank-you\""));
}

// ============================================================================
// GoalUrlsData Tests
// ============================================================================

#[test]
fn test_goal_urls_data_creation() {
    let data = GoalUrlsData {
        date_range: sample_date_range(),
        goal_id: 1,
        goal_name: "Purchase".to_string(),
        urls: vec![sample_goal_url_data()],
    };
    assert_eq!(data.goal_id, 1);
    assert!(!data.urls.is_empty());
}

#[test]
fn test_goal_urls_data_serialization() {
    let data = GoalUrlsData {
        date_range: sample_date_range(),
        goal_id: 1,
        goal_name: "Purchase".to_string(),
        urls: vec![],
    };
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"goal_id\":1"));
}

// ============================================================================
// ReversePathData Tests
// ============================================================================

#[test]
fn test_reverse_path_data_creation() {
    let path = sample_reverse_path_data();
    assert_eq!(path.goal_completion_location, "/thank-you");
    assert!(path.step_minus_1.is_some());
    assert!(path.step_minus_2.is_some());
    assert!(path.step_minus_3.is_some());
}

#[test]
fn test_reverse_path_data_partial_steps() {
    let path = ReversePathData {
        goal_completion_location: "/success".to_string(),
        step_minus_1: Some("/checkout".to_string()),
        step_minus_2: None,
        step_minus_3: None,
        completions: 50,
        value: 250.0,
    };
    assert!(path.step_minus_1.is_some());
    assert!(path.step_minus_2.is_none());
}

#[test]
fn test_reverse_path_data_serialization() {
    let path = sample_reverse_path_data();
    let json = serde_json::to_string(&path).unwrap();
    assert!(json.contains("\"step_minus_1\":\"/checkout\""));
}

// ============================================================================
// ReverseGoalPathData Tests
// ============================================================================

#[test]
fn test_reverse_goal_path_data_creation() {
    let data = ReverseGoalPathData {
        date_range: sample_date_range(),
        goal_id: 1,
        goal_name: "Purchase".to_string(),
        paths: vec![sample_reverse_path_data()],
    };
    assert_eq!(data.goal_id, 1);
    assert!(!data.paths.is_empty());
}

// ============================================================================
// FunnelStep Tests
// ============================================================================

#[test]
fn test_funnel_step_creation() {
    let step = sample_funnel_step();
    assert_eq!(step.step_number, 1);
    assert_eq!(step.step_name, "Product View");
    assert_eq!(step.entrances, 1000);
    assert_eq!(step.continuation, 800);
}

#[test]
fn test_funnel_step_metrics() {
    let step = sample_funnel_step();
    // continuation_rate should be continuation / entrances * 100
    assert!((step.continuation_rate - 80.0).abs() < f64::EPSILON);
    // abandonment_rate should be 100 - continuation_rate
    assert!((step.abandonment_rate - 20.0).abs() < f64::EPSILON);
}

#[test]
fn test_funnel_step_serialization() {
    let step = sample_funnel_step();
    let json = serde_json::to_string(&step).unwrap();
    assert!(json.contains("\"step_number\":1"));
    assert!(json.contains("\"step_name\":\"Product View\""));
}

// ============================================================================
// FunnelVisualization Tests
// ============================================================================

#[test]
fn test_funnel_visualization_creation() {
    let viz = sample_funnel_visualization();
    assert!(!viz.steps.is_empty());
    assert!(!viz.drop_offs.is_empty());
    assert!(!viz.backfills.is_empty());
}

#[test]
fn test_funnel_visualization_empty() {
    let viz = FunnelVisualization {
        steps: vec![],
        drop_offs: vec![],
        backfills: vec![],
    };
    assert!(viz.steps.is_empty());
}

#[test]
fn test_funnel_visualization_serialization() {
    let viz = sample_funnel_visualization();
    let json = serde_json::to_string(&viz).unwrap();
    assert!(json.contains("\"steps\""));
    assert!(json.contains("\"drop_offs\""));
}

// ============================================================================
// FunnelVisStep Tests
// ============================================================================

#[test]
fn test_funnel_vis_step_creation() {
    let step = sample_funnel_vis_step();
    assert_eq!(step.step_number, 1);
    assert_eq!(step.value, 1000);
    assert!((step.percentage - 100.0).abs() < f64::EPSILON);
}

#[test]
fn test_funnel_vis_step_serialization() {
    let step = sample_funnel_vis_step();
    let json = serde_json::to_string(&step).unwrap();
    assert!(json.contains("\"step_number\":1"));
}

// ============================================================================
// FunnelDropOff Tests
// ============================================================================

#[test]
fn test_funnel_drop_off_creation() {
    let drop_off = sample_funnel_drop_off();
    assert_eq!(drop_off.from_step, 1);
    assert_eq!(drop_off.count, 200);
    assert!(!drop_off.destinations.is_empty());
}

#[test]
fn test_funnel_drop_off_serialization() {
    let drop_off = sample_funnel_drop_off();
    let json = serde_json::to_string(&drop_off).unwrap();
    assert!(json.contains("\"from_step\":1"));
    assert!(json.contains("\"destinations\""));
}

// ============================================================================
// DropOffDestination Tests
// ============================================================================

#[test]
fn test_drop_off_destination_creation() {
    let dest = sample_drop_off_destination();
    assert_eq!(dest.destination, "/homepage");
    assert_eq!(dest.count, 50);
}

#[test]
fn test_drop_off_destination_serialization() {
    let dest = sample_drop_off_destination();
    let json = serde_json::to_string(&dest).unwrap();
    assert!(json.contains("\"destination\":\"/homepage\""));
}

// ============================================================================
// FunnelBackfill Tests
// ============================================================================

#[test]
fn test_funnel_backfill_creation() {
    let backfill = sample_funnel_backfill();
    assert_eq!(backfill.to_step, 2);
    assert!(!backfill.sources.is_empty());
}

#[test]
fn test_funnel_backfill_serialization() {
    let backfill = sample_funnel_backfill();
    let json = serde_json::to_string(&backfill).unwrap();
    assert!(json.contains("\"to_step\":2"));
}

// ============================================================================
// BackfillSource Tests
// ============================================================================

#[test]
fn test_backfill_source_creation() {
    let source = sample_backfill_source();
    assert_eq!(source.source, "/search");
    assert_eq!(source.count, 30);
}

#[test]
fn test_backfill_source_serialization() {
    let source = sample_backfill_source();
    let json = serde_json::to_string(&source).unwrap();
    assert!(json.contains("\"source\":\"/search\""));
}

// ============================================================================
// GoalFunnelData Tests
// ============================================================================

#[test]
fn test_goal_funnel_data_creation() {
    let funnel = sample_goal_funnel_data();
    assert_eq!(funnel.goal_id, 1);
    assert_eq!(funnel.goal_name, "Purchase");
    assert!(!funnel.funnel_steps.is_empty());
}

#[test]
fn test_goal_funnel_data_serialization() {
    let funnel = sample_goal_funnel_data();
    let json = serde_json::to_string(&funnel).unwrap();
    assert!(json.contains("\"goal_id\":1"));
    assert!(json.contains("\"funnel_visualization\""));
}

// ============================================================================
// SmartGoalsData Tests
// ============================================================================

#[test]
fn test_smart_goals_data_creation() {
    let data = sample_smart_goals_data();
    assert_eq!(data.smart_goal_completions, 500);
    assert!(!data.engagement_score_distribution.is_empty());
    assert!(!data.top_converting_segments.is_empty());
}

#[test]
fn test_smart_goals_data_serialization() {
    let data = sample_smart_goals_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"smart_goal_completions\":500"));
}

// ============================================================================
// EngagementScoreBucket Tests
// ============================================================================

#[test]
fn test_engagement_score_bucket_creation() {
    let bucket = sample_engagement_score_bucket();
    assert_eq!(bucket.score_range, "80-100");
    assert_eq!(bucket.sessions, 500);
}

#[test]
fn test_engagement_score_bucket_serialization() {
    let bucket = sample_engagement_score_bucket();
    let json = serde_json::to_string(&bucket).unwrap();
    assert!(json.contains("\"score_range\":\"80-100\""));
}

// ============================================================================
// SmartGoalSegment Tests
// ============================================================================

#[test]
fn test_smart_goal_segment_creation() {
    let segment = sample_smart_goal_segment();
    assert_eq!(segment.segment_name, "High-value Users");
    assert_eq!(segment.sessions, 1000);
}

#[test]
fn test_smart_goal_segment_serialization() {
    let segment = sample_smart_goal_segment();
    let json = serde_json::to_string(&segment).unwrap();
    assert!(json.contains("\"segment_name\":\"High-value Users\""));
}

// ============================================================================
// MultiChannelOverview Tests
// ============================================================================

#[test]
fn test_multi_channel_overview_creation() {
    let overview = sample_multi_channel_overview();
    assert_eq!(overview.total_conversions, 1000);
    assert!(!overview.channel_contribution.is_empty());
}

#[test]
fn test_multi_channel_overview_serialization() {
    let overview = sample_multi_channel_overview();
    let json = serde_json::to_string(&overview).unwrap();
    assert!(json.contains("\"total_conversions\":1000"));
    assert!(json.contains("\"assisted_vs_last_click\""));
}

// ============================================================================
// AssistedVsLastClick Tests
// ============================================================================

#[test]
fn test_assisted_vs_last_click_creation() {
    let data = sample_assisted_vs_last_click();
    assert_eq!(data.assisted_conversions, 300);
    assert_eq!(data.last_click_conversions, 500);
}

#[test]
fn test_assisted_vs_last_click_ratio() {
    let data = sample_assisted_vs_last_click();
    // Ratio should be assisted / last_click
    assert!((data.assisted_to_last_click_ratio - 0.6).abs() < f64::EPSILON);
}

#[test]
fn test_assisted_vs_last_click_serialization() {
    let data = sample_assisted_vs_last_click();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"assisted_conversions\":300"));
}

// ============================================================================
// ChannelContribution Tests
// ============================================================================

#[test]
fn test_channel_contribution_creation() {
    let contrib = sample_channel_contribution();
    assert_eq!(contrib.channel, "Organic Search");
    assert_eq!(contrib.assisted_conversions, 150);
}

#[test]
fn test_channel_contribution_serialization() {
    let contrib = sample_channel_contribution();
    let json = serde_json::to_string(&contrib).unwrap();
    assert!(json.contains("\"channel\":\"Organic Search\""));
}

// ============================================================================
// AssistedConversionsData Tests
// ============================================================================

#[test]
fn test_assisted_conversions_data_creation() {
    let data = AssistedConversionsData {
        date_range: sample_date_range(),
        mcf_channel_grouping: vec![sample_mcf_channel_data()],
        source_medium: vec![sample_mcf_source_medium_data()],
        source: vec![sample_mcf_source_data()],
    };
    assert!(!data.mcf_channel_grouping.is_empty());
}

#[test]
fn test_assisted_conversions_data_serialization() {
    let data = AssistedConversionsData {
        date_range: sample_date_range(),
        mcf_channel_grouping: vec![],
        source_medium: vec![],
        source: vec![],
    };
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"mcf_channel_grouping\":[]"));
}

// ============================================================================
// McfChannelData Tests
// ============================================================================

#[test]
fn test_mcf_channel_data_creation() {
    let data = sample_mcf_channel_data();
    assert_eq!(data.channel, "Paid Search");
    assert_eq!(data.assisted_conversions, 100);
}

#[test]
fn test_mcf_channel_data_serialization() {
    let data = sample_mcf_channel_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"channel\":\"Paid Search\""));
}

// ============================================================================
// McfSourceMediumData Tests
// ============================================================================

#[test]
fn test_mcf_source_medium_data_creation() {
    let data = sample_mcf_source_medium_data();
    assert_eq!(data.source_medium, "google / cpc");
}

#[test]
fn test_mcf_source_medium_data_serialization() {
    let data = sample_mcf_source_medium_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"source_medium\":\"google / cpc\""));
}

// ============================================================================
// McfSourceData Tests
// ============================================================================

#[test]
fn test_mcf_source_data_creation() {
    let data = sample_mcf_source_data();
    assert_eq!(data.source, "google");
    assert_eq!(data.assisted_conversions, 200);
}

#[test]
fn test_mcf_source_data_serialization() {
    let data = sample_mcf_source_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"source\":\"google\""));
}

// ============================================================================
// PathPosition Enum Tests
// ============================================================================

#[test]
fn test_path_position_all_variants() {
    let positions = vec![PathPosition::First, PathPosition::Middle, PathPosition::Last];
    assert_eq!(positions.len(), 3);
}

#[test]
fn test_path_position_serialization() {
    assert_eq!(serde_json::to_string(&PathPosition::First).unwrap(), "\"first\"");
    assert_eq!(serde_json::to_string(&PathPosition::Middle).unwrap(), "\"middle\"");
    assert_eq!(serde_json::to_string(&PathPosition::Last).unwrap(), "\"last\"");
}

#[test]
fn test_path_position_deserialization() {
    assert_eq!(serde_json::from_str::<PathPosition>("\"first\"").unwrap(), PathPosition::First);
    assert_eq!(serde_json::from_str::<PathPosition>("\"middle\"").unwrap(), PathPosition::Middle);
    assert_eq!(serde_json::from_str::<PathPosition>("\"last\"").unwrap(), PathPosition::Last);
}

#[test]
fn test_path_position_equality() {
    assert_eq!(PathPosition::First, PathPosition::First);
    assert_ne!(PathPosition::First, PathPosition::Last);
}

// ============================================================================
// PathNode Tests
// ============================================================================

#[test]
fn test_path_node_creation() {
    let node = sample_path_node();
    assert_eq!(node.channel, "Organic Search");
    assert!(node.source.is_some());
    assert!(node.medium.is_some());
}

#[test]
fn test_path_node_without_optional_fields() {
    let node = PathNode {
        channel: "Direct".to_string(),
        source: None,
        medium: None,
        position: PathPosition::Last,
    };
    assert!(node.source.is_none());
    assert!(node.medium.is_none());
}

#[test]
fn test_path_node_serialization() {
    let node = sample_path_node();
    let json = serde_json::to_string(&node).unwrap();
    assert!(json.contains("\"channel\":\"Organic Search\""));
    assert!(json.contains("\"position\":\"first\""));
}

// ============================================================================
// ConversionPathData Tests
// ============================================================================

#[test]
fn test_conversion_path_data_creation() {
    let path = sample_conversion_path_data();
    assert!(!path.path.is_empty());
    assert_eq!(path.conversions, 50);
    assert_eq!(path.path_length, 3);
}

#[test]
fn test_conversion_path_data_serialization() {
    let path = sample_conversion_path_data();
    let json = serde_json::to_string(&path).unwrap();
    assert!(json.contains("\"conversions\":50"));
    assert!(json.contains("\"path_length\":3"));
}

// ============================================================================
// TopPathsData Tests
// ============================================================================

#[test]
fn test_top_paths_data_creation() {
    let data = TopPathsData {
        date_range: sample_date_range(),
        paths: vec![sample_conversion_path_data()],
    };
    assert!(!data.paths.is_empty());
}

#[test]
fn test_top_paths_data_serialization() {
    let data = TopPathsData {
        date_range: sample_date_range(),
        paths: vec![],
    };
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"paths\":[]"));
}

// ============================================================================
// PathLengthBucket Tests
// ============================================================================

#[test]
fn test_path_length_bucket_creation() {
    let bucket = sample_path_length_bucket();
    assert_eq!(bucket.path_length, 3);
    assert_eq!(bucket.conversions, 200);
}

#[test]
fn test_path_length_bucket_serialization() {
    let bucket = sample_path_length_bucket();
    let json = serde_json::to_string(&bucket).unwrap();
    assert!(json.contains("\"path_length\":3"));
}

// ============================================================================
// PathLengthData Tests
// ============================================================================

#[test]
fn test_path_length_data_creation() {
    let data = PathLengthData {
        date_range: sample_date_range(),
        path_lengths: vec![sample_path_length_bucket()],
        avg_path_length: 2.8,
    };
    assert!(!data.path_lengths.is_empty());
}

#[test]
fn test_path_length_data_serialization() {
    let data = PathLengthData {
        date_range: sample_date_range(),
        path_lengths: vec![],
        avg_path_length: 2.5,
    };
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"avg_path_length\":2.5"));
}

// ============================================================================
// TimeLagBucket Tests
// ============================================================================

#[test]
fn test_time_lag_bucket_creation() {
    let bucket = sample_time_lag_bucket();
    assert_eq!(bucket.days_to_conversion, "0");
    assert_eq!(bucket.conversions, 300);
}

#[test]
fn test_time_lag_bucket_serialization() {
    let bucket = sample_time_lag_bucket();
    let json = serde_json::to_string(&bucket).unwrap();
    assert!(json.contains("\"days_to_conversion\":\"0\""));
}

#[test]
fn test_time_lag_bucket_various_ranges() {
    let ranges = vec!["0", "1", "2-3", "4-7", "8-14", "15-30", "31+"];
    for range in ranges {
        let bucket = TimeLagBucket {
            days_to_conversion: range.to_string(),
            conversions: 100,
            conversion_value: 5000.0,
            percentage: 10.0,
        };
        assert_eq!(bucket.days_to_conversion, range);
    }
}

// ============================================================================
// TimeLagData Tests
// ============================================================================

#[test]
fn test_time_lag_data_creation() {
    let data = TimeLagData {
        date_range: sample_date_range(),
        time_lags: vec![sample_time_lag_bucket()],
        avg_time_to_conversion: 3.5,
    };
    assert!(!data.time_lags.is_empty());
}

#[test]
fn test_time_lag_data_serialization() {
    let data = TimeLagData {
        date_range: sample_date_range(),
        time_lags: vec![],
        avg_time_to_conversion: 5.0,
    };
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"avg_time_to_conversion\":5"));
}

// ============================================================================
// AttributionModel Enum Tests
// ============================================================================

#[test]
fn test_attribution_model_all_variants() {
    let models = vec![
        AttributionModel::LastInteraction,
        AttributionModel::LastNonDirectClick,
        AttributionModel::LastGoogleAdsClick,
        AttributionModel::FirstInteraction,
        AttributionModel::Linear,
        AttributionModel::TimeDecay,
        AttributionModel::PositionBased,
        AttributionModel::DataDriven,
        AttributionModel::Custom,
    ];
    assert_eq!(models.len(), 9);
}

#[test]
fn test_attribution_model_display() {
    assert_eq!(format!("{}", AttributionModel::LastInteraction), "Last Interaction");
    assert_eq!(format!("{}", AttributionModel::LastNonDirectClick), "Last Non-Direct Click");
    assert_eq!(format!("{}", AttributionModel::LastGoogleAdsClick), "Last Google Ads Click");
    assert_eq!(format!("{}", AttributionModel::FirstInteraction), "First Interaction");
    assert_eq!(format!("{}", AttributionModel::Linear), "Linear");
    assert_eq!(format!("{}", AttributionModel::TimeDecay), "Time Decay");
    assert_eq!(format!("{}", AttributionModel::PositionBased), "Position Based");
    assert_eq!(format!("{}", AttributionModel::DataDriven), "Data-Driven");
    assert_eq!(format!("{}", AttributionModel::Custom), "Custom");
}

#[test]
fn test_attribution_model_serialization() {
    assert_eq!(serde_json::to_string(&AttributionModel::LastInteraction).unwrap(), "\"last_interaction\"");
    assert_eq!(serde_json::to_string(&AttributionModel::DataDriven).unwrap(), "\"data_driven\"");
    assert_eq!(serde_json::to_string(&AttributionModel::PositionBased).unwrap(), "\"position_based\"");
}

#[test]
fn test_attribution_model_deserialization() {
    assert_eq!(serde_json::from_str::<AttributionModel>("\"last_interaction\"").unwrap(), AttributionModel::LastInteraction);
    assert_eq!(serde_json::from_str::<AttributionModel>("\"time_decay\"").unwrap(), AttributionModel::TimeDecay);
}

#[test]
fn test_attribution_model_equality() {
    assert_eq!(AttributionModel::Linear, AttributionModel::Linear);
    assert_ne!(AttributionModel::Linear, AttributionModel::TimeDecay);
}

// ============================================================================
// AttributionModelResult Tests
// ============================================================================

#[test]
fn test_attribution_model_result_creation() {
    let result = sample_attribution_model_result();
    assert!((result.conversions - 100.5).abs() < f64::EPSILON);
    assert!((result.conversion_value - 5025.0).abs() < f64::EPSILON);
}

#[test]
fn test_attribution_model_result_serialization() {
    let result = sample_attribution_model_result();
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"conversions\":100.5"));
}

// ============================================================================
// AttributionChannelData Tests
// ============================================================================

#[test]
fn test_attribution_channel_data_creation() {
    let data = sample_attribution_channel_data();
    assert_eq!(data.channel, "Email");
    assert!(data.data_driven.is_some());
}

#[test]
fn test_attribution_channel_data_without_data_driven() {
    let data = AttributionChannelData {
        channel: "Direct".to_string(),
        last_interaction: sample_attribution_model_result(),
        last_non_direct_click: sample_attribution_model_result(),
        first_interaction: sample_attribution_model_result(),
        linear: sample_attribution_model_result(),
        time_decay: sample_attribution_model_result(),
        position_based: sample_attribution_model_result(),
        data_driven: None,
    };
    assert!(data.data_driven.is_none());
}

#[test]
fn test_attribution_channel_data_serialization() {
    let data = sample_attribution_channel_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"channel\":\"Email\""));
    assert!(json.contains("\"last_interaction\""));
}

// ============================================================================
// AttributionModelComparison Tests
// ============================================================================

#[test]
fn test_attribution_model_comparison_creation() {
    let comparison = AttributionModelComparison {
        date_range: sample_date_range(),
        channels: vec![sample_attribution_channel_data()],
        models_compared: vec![AttributionModel::LastInteraction, AttributionModel::Linear],
    };
    assert!(!comparison.channels.is_empty());
    assert_eq!(comparison.models_compared.len(), 2);
}

#[test]
fn test_attribution_model_comparison_serialization() {
    let comparison = AttributionModelComparison {
        date_range: sample_date_range(),
        channels: vec![],
        models_compared: vec![],
    };
    let json = serde_json::to_string(&comparison).unwrap();
    assert!(json.contains("\"channels\":[]"));
}

// ============================================================================
// CustomAttributionModel Tests
// ============================================================================

#[test]
fn test_custom_attribution_model_creation() {
    let model = sample_custom_attribution_model();
    assert_eq!(model.name, "My Custom Model");
    assert_eq!(model.baseline_model, AttributionModel::PositionBased);
    assert_eq!(model.lookback_window, 30);
}

#[test]
fn test_custom_attribution_model_weights_sum_to_one() {
    let model = sample_custom_attribution_model();
    let sum = model.first_interaction_weight + model.middle_interactions_weight + model.last_interaction_weight;
    assert!((sum - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_custom_attribution_model_without_time_decay() {
    let model = CustomAttributionModel {
        id: Uuid::new_v4(),
        name: "Linear Model".to_string(),
        baseline_model: AttributionModel::Linear,
        lookback_window: 90,
        first_interaction_weight: 0.33,
        middle_interactions_weight: 0.34,
        last_interaction_weight: 0.33,
        time_decay_half_life: None,
        created_at: Utc::now(),
    };
    assert!(model.time_decay_half_life.is_none());
}

#[test]
fn test_custom_attribution_model_serialization() {
    let model = sample_custom_attribution_model();
    let json = serde_json::to_string(&model).unwrap();
    assert!(json.contains("\"name\":\"My Custom Model\""));
    assert!(json.contains("\"baseline_model\":\"position_based\""));
}

// ============================================================================
// PathDimension Enum Tests
// ============================================================================

#[test]
fn test_path_dimension_all_variants() {
    let dimensions = vec![
        PathDimension::Channel,
        PathDimension::Source,
        PathDimension::Medium,
        PathDimension::SourceMedium,
        PathDimension::Campaign,
        PathDimension::Keyword,
    ];
    assert_eq!(dimensions.len(), 6);
}

#[test]
fn test_path_dimension_serialization() {
    assert_eq!(serde_json::to_string(&PathDimension::Channel).unwrap(), "\"channel\"");
    assert_eq!(serde_json::to_string(&PathDimension::SourceMedium).unwrap(), "\"source_medium\"");
}

#[test]
fn test_path_dimension_deserialization() {
    assert_eq!(serde_json::from_str::<PathDimension>("\"channel\"").unwrap(), PathDimension::Channel);
    assert_eq!(serde_json::from_str::<PathDimension>("\"keyword\"").unwrap(), PathDimension::Keyword);
}

// ============================================================================
// InteractionType Enum Tests
// ============================================================================

#[test]
fn test_interaction_type_all_variants() {
    let types = vec![
        InteractionType::Impression,
        InteractionType::Click,
        InteractionType::Visit,
        InteractionType::Conversion,
    ];
    assert_eq!(types.len(), 4);
}

#[test]
fn test_interaction_type_serialization() {
    assert_eq!(serde_json::to_string(&InteractionType::Impression).unwrap(), "\"impression\"");
    assert_eq!(serde_json::to_string(&InteractionType::Click).unwrap(), "\"click\"");
    assert_eq!(serde_json::to_string(&InteractionType::Visit).unwrap(), "\"visit\"");
    assert_eq!(serde_json::to_string(&InteractionType::Conversion).unwrap(), "\"conversion\"");
}

#[test]
fn test_interaction_type_deserialization() {
    assert_eq!(serde_json::from_str::<InteractionType>("\"impression\"").unwrap(), InteractionType::Impression);
    assert_eq!(serde_json::from_str::<InteractionType>("\"conversion\"").unwrap(), InteractionType::Conversion);
}

#[test]
fn test_interaction_type_equality() {
    assert_eq!(InteractionType::Click, InteractionType::Click);
    assert_ne!(InteractionType::Click, InteractionType::Visit);
}

// ============================================================================
// TouchpointData Tests
// ============================================================================

#[test]
fn test_touchpoint_data_creation() {
    let touchpoint = sample_touchpoint_data();
    assert_eq!(touchpoint.position, 1);
    assert_eq!(touchpoint.value, "google / organic");
    assert_eq!(touchpoint.interaction_type, InteractionType::Click);
}

#[test]
fn test_touchpoint_data_without_timestamp() {
    let touchpoint = TouchpointData {
        position: 2,
        value: "facebook / cpc".to_string(),
        interaction_type: InteractionType::Impression,
        timestamp: None,
    };
    assert!(touchpoint.timestamp.is_none());
}

#[test]
fn test_touchpoint_data_serialization() {
    let touchpoint = sample_touchpoint_data();
    let json = serde_json::to_string(&touchpoint).unwrap();
    assert!(json.contains("\"position\":1"));
    assert!(json.contains("\"interaction_type\":\"click\""));
}

// ============================================================================
// DetailedPathData Tests
// ============================================================================

#[test]
fn test_detailed_path_data_creation() {
    let path = sample_detailed_path_data();
    assert_eq!(path.path_id, "path_001");
    assert!(!path.touchpoints.is_empty());
    assert_eq!(path.conversions, 25);
}

#[test]
fn test_detailed_path_data_serialization() {
    let path = sample_detailed_path_data();
    let json = serde_json::to_string(&path).unwrap();
    assert!(json.contains("\"path_id\":\"path_001\""));
}

// ============================================================================
// ConversionPathExplorer Tests
// ============================================================================

#[test]
fn test_conversion_path_explorer_creation() {
    let explorer = ConversionPathExplorer {
        date_range: sample_date_range(),
        dimension: PathDimension::Channel,
        paths: vec![sample_detailed_path_data()],
        sankey_data: sample_sankey_data(),
    };
    assert_eq!(explorer.dimension, PathDimension::Channel);
    assert!(!explorer.paths.is_empty());
}

#[test]
fn test_conversion_path_explorer_serialization() {
    let explorer = ConversionPathExplorer {
        date_range: sample_date_range(),
        dimension: PathDimension::SourceMedium,
        paths: vec![],
        sankey_data: SankeyData { nodes: vec![], links: vec![] },
    };
    let json = serde_json::to_string(&explorer).unwrap();
    assert!(json.contains("\"dimension\":\"source_medium\""));
}

// ============================================================================
// SankeyNode Tests
// ============================================================================

#[test]
fn test_sankey_node_creation() {
    let node = sample_sankey_node();
    assert_eq!(node.id, "node_1");
    assert_eq!(node.name, "Organic Search");
    assert_eq!(node.column, 0);
    assert_eq!(node.value, 500);
}

#[test]
fn test_sankey_node_serialization() {
    let node = sample_sankey_node();
    let json = serde_json::to_string(&node).unwrap();
    assert!(json.contains("\"id\":\"node_1\""));
    assert!(json.contains("\"column\":0"));
}

// ============================================================================
// SankeyLink Tests
// ============================================================================

#[test]
fn test_sankey_link_creation() {
    let link = sample_sankey_link();
    assert_eq!(link.source, "node_1");
    assert_eq!(link.target, "node_2");
    assert_eq!(link.value, 300);
}

#[test]
fn test_sankey_link_serialization() {
    let link = sample_sankey_link();
    let json = serde_json::to_string(&link).unwrap();
    assert!(json.contains("\"source\":\"node_1\""));
    assert!(json.contains("\"target\":\"node_2\""));
}

// ============================================================================
// SankeyData Tests
// ============================================================================

#[test]
fn test_sankey_data_creation() {
    let data = sample_sankey_data();
    assert!(!data.nodes.is_empty());
    assert!(!data.links.is_empty());
}

#[test]
fn test_sankey_data_empty() {
    let data = SankeyData {
        nodes: vec![],
        links: vec![],
    };
    assert!(data.nodes.is_empty());
    assert!(data.links.is_empty());
}

#[test]
fn test_sankey_data_serialization() {
    let data = sample_sankey_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"nodes\""));
    assert!(json.contains("\"links\""));
}

// ============================================================================
// Edge Cases and Complex Scenarios
// ============================================================================

#[test]
fn test_unicode_in_goal_names() {
    let goal = GoalData {
        goal_id: 1,
        goal_name: "購入完了".to_string(),
        goal_type: GoalType::Destination,
        completions: 100,
        value: 500.0,
        conversion_rate: 2.5,
        abandonment_rate: 40.0,
        percentage_of_total: 10.0,
    };
    let json = serde_json::to_string(&goal).unwrap();
    let deserialized: GoalData = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.goal_name, "購入完了");
}

#[test]
fn test_empty_string_fields() {
    let node = PathNode {
        channel: "".to_string(),
        source: Some("".to_string()),
        medium: Some("".to_string()),
        position: PathPosition::First,
    };
    let json = serde_json::to_string(&node).unwrap();
    assert!(json.contains("\"channel\":\"\""));
}

#[test]
fn test_large_values() {
    let overview = GoalsOverview {
        date_range: sample_date_range(),
        goals: vec![],
        total_completions: u64::MAX / 2,
        total_value: 1_000_000_000_000.0,
        overall_conversion_rate: 99.99,
        goal_trend: vec![],
        comparison: None,
    };
    let json = serde_json::to_string(&overview).unwrap();
    let deserialized: GoalsOverview = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.total_completions, u64::MAX / 2);
}

#[test]
fn test_floating_point_precision() {
    let result = AttributionModelResult {
        conversions: 33.333333333333336,
        conversion_value: 1666.6666666666667,
        percentage_change_from_last_interaction: -0.00000001,
    };
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: AttributionModelResult = serde_json::from_str(&json).unwrap();
    assert!((deserialized.conversions - 33.333333333333336).abs() < 1e-10);
}

#[test]
fn test_negative_comparison_values() {
    let comparison = GoalsComparison {
        completions_change: -100.0,
        value_change: -100.0,
        conversion_rate_change: -50.0,
    };
    assert!(comparison.completions_change < 0.0);
    assert!(comparison.value_change < 0.0);
}

#[test]
fn test_many_funnel_steps() {
    let steps: Vec<FunnelStep> = (1..=20).map(|i| {
        FunnelStep {
            step_number: i,
            step_name: format!("Step {}", i),
            step_url: format!("/step/{}", i),
            entrances: 1000 - (i as u64 * 40),
            exits: 40,
            continuation: 960 - (i as u64 * 40),
            continuation_rate: 96.0 - (i as f64 * 4.0),
            abandonment_rate: 4.0 + (i as f64 * 0.2),
        }
    }).collect();

    let funnel = GoalFunnelData {
        date_range: sample_date_range(),
        goal_id: 1,
        goal_name: "Complex Funnel".to_string(),
        funnel_steps: steps,
        overall_conversion_rate: 5.0,
        funnel_visualization: sample_funnel_visualization(),
    };

    assert_eq!(funnel.funnel_steps.len(), 20);
}

#[test]
fn test_complex_conversion_path() {
    let path = ConversionPathData {
        path: vec![
            PathNode {
                channel: "Paid Search".to_string(),
                source: Some("google".to_string()),
                medium: Some("cpc".to_string()),
                position: PathPosition::First,
            },
            PathNode {
                channel: "Email".to_string(),
                source: Some("newsletter".to_string()),
                medium: Some("email".to_string()),
                position: PathPosition::Middle,
            },
            PathNode {
                channel: "Direct".to_string(),
                source: None,
                medium: None,
                position: PathPosition::Middle,
            },
            PathNode {
                channel: "Organic Search".to_string(),
                source: Some("google".to_string()),
                medium: Some("organic".to_string()),
                position: PathPosition::Last,
            },
        ],
        conversions: 100,
        conversion_value: 5000.0,
        path_length: 4,
        percentage: 10.0,
    };

    assert_eq!(path.path.len(), 4);
    assert_eq!(path.path[0].position, PathPosition::First);
    assert_eq!(path.path[3].position, PathPosition::Last);
}

#[test]
fn test_sankey_data_complex() {
    let nodes = vec![
        SankeyNode { id: "organic".to_string(), name: "Organic Search".to_string(), column: 0, value: 1000 },
        SankeyNode { id: "paid".to_string(), name: "Paid Search".to_string(), column: 0, value: 800 },
        SankeyNode { id: "email".to_string(), name: "Email".to_string(), column: 1, value: 600 },
        SankeyNode { id: "conversion".to_string(), name: "Conversion".to_string(), column: 2, value: 400 },
    ];

    let links = vec![
        SankeyLink { source: "organic".to_string(), target: "email".to_string(), value: 300 },
        SankeyLink { source: "paid".to_string(), target: "email".to_string(), value: 200 },
        SankeyLink { source: "organic".to_string(), target: "conversion".to_string(), value: 200 },
        SankeyLink { source: "email".to_string(), target: "conversion".to_string(), value: 200 },
    ];

    let data = SankeyData { nodes, links };
    assert_eq!(data.nodes.len(), 4);
    assert_eq!(data.links.len(), 4);
}

#[test]
fn test_clone_all_major_types() {
    let overview = sample_goals_overview();
    let cloned = overview.clone();
    assert_eq!(overview.total_completions, cloned.total_completions);

    let funnel = sample_goal_funnel_data();
    let cloned_funnel = funnel.clone();
    assert_eq!(funnel.goal_id, cloned_funnel.goal_id);

    let multi = sample_multi_channel_overview();
    let cloned_multi = multi.clone();
    assert_eq!(multi.total_conversions, cloned_multi.total_conversions);
}

#[test]
fn test_debug_trait_implementation() {
    let goal = sample_goal_data();
    let debug_str = format!("{:?}", goal);
    assert!(debug_str.contains("GoalData"));

    let model = AttributionModel::Linear;
    let debug_str = format!("{:?}", model);
    assert!(debug_str.contains("Linear"));
}

#[test]
fn test_special_characters_in_urls() {
    let url_data = GoalUrlData {
        goal_completion_location: "/thank-you?order_id=123&status=complete".to_string(),
        goal_previous_step: Some("/checkout?step=payment&method=card".to_string()),
        completions: 50,
        value: 250.0,
        percentage: 10.0,
    };
    let json = serde_json::to_string(&url_data).unwrap();
    let deserialized: GoalUrlData = serde_json::from_str(&json).unwrap();
    assert!(deserialized.goal_completion_location.contains("order_id=123"));
}

#[test]
fn test_all_attribution_models_in_channel() {
    let data = sample_attribution_channel_data();

    // Verify all attribution model results exist
    assert!(data.last_interaction.conversions > 0.0);
    assert!(data.last_non_direct_click.conversions > 0.0);
    assert!(data.first_interaction.conversions > 0.0);
    assert!(data.linear.conversions > 0.0);
    assert!(data.time_decay.conversions > 0.0);
    assert!(data.position_based.conversions > 0.0);
    assert!(data.data_driven.is_some());
}

#[test]
fn test_zero_conversions() {
    let comparison = GoalsComparison {
        completions_change: 0.0,
        value_change: 0.0,
        conversion_rate_change: 0.0,
    };

    let json = serde_json::to_string(&comparison).unwrap();
    assert!(json.contains(":0"));
}
