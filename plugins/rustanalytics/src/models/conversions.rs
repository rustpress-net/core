//! Conversion analytics models

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::DateRange;

/// Goals overview data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalsOverview {
    pub date_range: DateRange,
    pub goals: Vec<GoalData>,
    pub total_completions: u64,
    pub total_value: f64,
    pub overall_conversion_rate: f64,
    pub goal_trend: Vec<GoalTrendData>,
    pub comparison: Option<GoalsComparison>,
}

/// Individual goal data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalData {
    pub goal_id: u32,
    pub goal_name: String,
    pub goal_type: GoalType,
    pub completions: u64,
    pub value: f64,
    pub conversion_rate: f64,
    pub abandonment_rate: f64,
    pub percentage_of_total: f64,
}

/// Goal types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GoalType {
    Destination,
    Duration,
    PagesPerSession,
    Event,
    SmartGoal,
}

/// Goal trend data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalTrendData {
    pub date: NaiveDate,
    pub completions: u64,
    pub value: f64,
    pub conversion_rate: f64,
}

/// Goals comparison data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalsComparison {
    pub completions_change: f64,
    pub value_change: f64,
    pub conversion_rate_change: f64,
}

/// Goal URLs data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalUrlsData {
    pub date_range: DateRange,
    pub goal_id: u32,
    pub goal_name: String,
    pub urls: Vec<GoalUrlData>,
}

/// Individual goal URL data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalUrlData {
    pub goal_completion_location: String,
    pub goal_previous_step: Option<String>,
    pub completions: u64,
    pub value: f64,
    pub percentage: f64,
}

/// Reverse goal path data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReverseGoalPathData {
    pub date_range: DateRange,
    pub goal_id: u32,
    pub goal_name: String,
    pub paths: Vec<ReversePathData>,
}

/// Individual reverse path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReversePathData {
    pub goal_completion_location: String,
    pub step_minus_1: Option<String>,
    pub step_minus_2: Option<String>,
    pub step_minus_3: Option<String>,
    pub completions: u64,
    pub value: f64,
}

/// Goal funnel data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalFunnelData {
    pub date_range: DateRange,
    pub goal_id: u32,
    pub goal_name: String,
    pub funnel_steps: Vec<FunnelStep>,
    pub overall_conversion_rate: f64,
    pub funnel_visualization: FunnelVisualization,
}

/// Funnel step data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelStep {
    pub step_number: u32,
    pub step_name: String,
    pub step_url: String,
    pub entrances: u64,
    pub exits: u64,
    pub continuation: u64,
    pub continuation_rate: f64,
    pub abandonment_rate: f64,
}

/// Funnel visualization data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelVisualization {
    pub steps: Vec<FunnelVisStep>,
    pub drop_offs: Vec<FunnelDropOff>,
    pub backfills: Vec<FunnelBackfill>,
}

/// Funnel visualization step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelVisStep {
    pub step_number: u32,
    pub step_name: String,
    pub value: u64,
    pub percentage: f64,
}

/// Funnel drop-off point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelDropOff {
    pub from_step: u32,
    pub count: u64,
    pub destinations: Vec<DropOffDestination>,
}

/// Drop-off destination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropOffDestination {
    pub destination: String,
    pub count: u64,
    pub percentage: f64,
}

/// Funnel backfill (entries from other pages)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelBackfill {
    pub to_step: u32,
    pub sources: Vec<BackfillSource>,
}

/// Backfill source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackfillSource {
    pub source: String,
    pub count: u64,
    pub percentage: f64,
}

/// Smart goals data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartGoalsData {
    pub date_range: DateRange,
    pub smart_goal_completions: u64,
    pub smart_goal_conversion_rate: f64,
    pub smart_goal_value: f64,
    pub engagement_score_distribution: Vec<EngagementScoreBucket>,
    pub top_converting_segments: Vec<SmartGoalSegment>,
}

/// Engagement score bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementScoreBucket {
    pub score_range: String,
    pub sessions: u64,
    pub percentage: f64,
    pub conversion_rate: f64,
}

/// Smart goal segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartGoalSegment {
    pub segment_name: String,
    pub sessions: u64,
    pub smart_goal_completions: u64,
    pub conversion_rate: f64,
}

/// Multi-channel funnel overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiChannelOverview {
    pub date_range: DateRange,
    pub total_conversions: u64,
    pub total_conversion_value: f64,
    pub avg_time_to_conversion: f64,
    pub avg_path_length: f64,
    pub assisted_vs_last_click: AssistedVsLastClick,
    pub channel_contribution: Vec<ChannelContribution>,
}

/// Assisted vs last-click comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistedVsLastClick {
    pub assisted_conversions: u64,
    pub assisted_value: f64,
    pub last_click_conversions: u64,
    pub last_click_value: f64,
    pub assisted_to_last_click_ratio: f64,
}

/// Channel contribution data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelContribution {
    pub channel: String,
    pub assisted_conversions: u64,
    pub assisted_conversion_value: f64,
    pub last_interaction_conversions: u64,
    pub last_interaction_value: f64,
    pub first_interaction_conversions: u64,
    pub first_interaction_value: f64,
    pub assisted_to_last_click_ratio: f64,
}

/// Assisted conversions data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistedConversionsData {
    pub date_range: DateRange,
    pub mcf_channel_grouping: Vec<McfChannelData>,
    pub source_medium: Vec<McfSourceMediumData>,
    pub source: Vec<McfSourceData>,
}

/// MCF channel data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McfChannelData {
    pub channel: String,
    pub assisted_conversions: u64,
    pub assisted_conversion_value: f64,
    pub last_interaction_conversions: u64,
    pub last_interaction_value: f64,
    pub assisted_to_last_click_ratio: f64,
}

/// MCF source/medium data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McfSourceMediumData {
    pub source_medium: String,
    pub assisted_conversions: u64,
    pub assisted_conversion_value: f64,
    pub last_interaction_conversions: u64,
    pub last_interaction_value: f64,
}

/// MCF source data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McfSourceData {
    pub source: String,
    pub assisted_conversions: u64,
    pub assisted_conversion_value: f64,
    pub last_interaction_conversions: u64,
    pub last_interaction_value: f64,
}

/// Top conversion paths data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopPathsData {
    pub date_range: DateRange,
    pub paths: Vec<ConversionPathData>,
}

/// Conversion path data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionPathData {
    pub path: Vec<PathNode>,
    pub conversions: u64,
    pub conversion_value: f64,
    pub path_length: u32,
    pub percentage: f64,
}

/// Node in conversion path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathNode {
    pub channel: String,
    pub source: Option<String>,
    pub medium: Option<String>,
    pub position: PathPosition,
}

/// Position in conversion path
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PathPosition {
    First,
    Middle,
    Last,
}

/// Path length data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathLengthData {
    pub date_range: DateRange,
    pub path_lengths: Vec<PathLengthBucket>,
    pub avg_path_length: f64,
}

/// Path length bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathLengthBucket {
    pub path_length: u32,
    pub conversions: u64,
    pub conversion_value: f64,
    pub percentage: f64,
}

/// Time lag data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeLagData {
    pub date_range: DateRange,
    pub time_lags: Vec<TimeLagBucket>,
    pub avg_time_to_conversion: f64,
}

/// Time lag bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeLagBucket {
    pub days_to_conversion: String,
    pub conversions: u64,
    pub conversion_value: f64,
    pub percentage: f64,
}

/// Attribution model comparison data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributionModelComparison {
    pub date_range: DateRange,
    pub channels: Vec<AttributionChannelData>,
    pub models_compared: Vec<AttributionModel>,
}

/// Attribution channel data across models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributionChannelData {
    pub channel: String,
    pub last_interaction: AttributionModelResult,
    pub last_non_direct_click: AttributionModelResult,
    pub first_interaction: AttributionModelResult,
    pub linear: AttributionModelResult,
    pub time_decay: AttributionModelResult,
    pub position_based: AttributionModelResult,
    pub data_driven: Option<AttributionModelResult>,
}

/// Attribution model result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributionModelResult {
    pub conversions: f64,
    pub conversion_value: f64,
    pub percentage_change_from_last_interaction: f64,
}

/// Attribution models
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AttributionModel {
    LastInteraction,
    LastNonDirectClick,
    LastGoogleAdsClick,
    FirstInteraction,
    Linear,
    TimeDecay,
    PositionBased,
    DataDriven,
    Custom,
}

impl std::fmt::Display for AttributionModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributionModel::LastInteraction => write!(f, "Last Interaction"),
            AttributionModel::LastNonDirectClick => write!(f, "Last Non-Direct Click"),
            AttributionModel::LastGoogleAdsClick => write!(f, "Last Google Ads Click"),
            AttributionModel::FirstInteraction => write!(f, "First Interaction"),
            AttributionModel::Linear => write!(f, "Linear"),
            AttributionModel::TimeDecay => write!(f, "Time Decay"),
            AttributionModel::PositionBased => write!(f, "Position Based"),
            AttributionModel::DataDriven => write!(f, "Data-Driven"),
            AttributionModel::Custom => write!(f, "Custom"),
        }
    }
}

/// Custom attribution model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAttributionModel {
    pub id: uuid::Uuid,
    pub name: String,
    pub baseline_model: AttributionModel,
    pub lookback_window: u32,
    pub first_interaction_weight: f64,
    pub middle_interactions_weight: f64,
    pub last_interaction_weight: f64,
    pub time_decay_half_life: Option<u32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Conversion path explorer data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionPathExplorer {
    pub date_range: DateRange,
    pub dimension: PathDimension,
    pub paths: Vec<DetailedPathData>,
    pub sankey_data: SankeyData,
}

/// Path dimension for exploration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PathDimension {
    Channel,
    Source,
    Medium,
    SourceMedium,
    Campaign,
    Keyword,
}

/// Detailed path data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedPathData {
    pub path_id: String,
    pub touchpoints: Vec<TouchpointData>,
    pub conversions: u64,
    pub conversion_value: f64,
    pub days_to_conversion: f64,
}

/// Touchpoint data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TouchpointData {
    pub position: u32,
    pub value: String,
    pub interaction_type: InteractionType,
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

/// Interaction type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InteractionType {
    Impression,
    Click,
    Visit,
    Conversion,
}

/// Sankey diagram data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SankeyData {
    pub nodes: Vec<SankeyNode>,
    pub links: Vec<SankeyLink>,
}

/// Sankey diagram node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SankeyNode {
    pub id: String,
    pub name: String,
    pub column: u32,
    pub value: u64,
}

/// Sankey diagram link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SankeyLink {
    pub source: String,
    pub target: String,
    pub value: u64,
}
