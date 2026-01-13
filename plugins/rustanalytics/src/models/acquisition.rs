//! Acquisition analytics models

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::DateRange;

/// Acquisition overview data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcquisitionOverview {
    pub date_range: DateRange,
    pub channels: Vec<ChannelAcquisition>,
    pub source_medium: Vec<SourceMediumData>,
    pub top_channels_trend: Vec<ChannelTrendData>,
    pub totals: AcquisitionTotals,
    pub comparison: Option<AcquisitionComparison>,
}

/// Channel acquisition data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelAcquisition {
    pub channel: Channel,
    pub users: u64,
    pub new_users: u64,
    pub sessions: u64,
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

/// Default channel groupings
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Channel {
    Direct,
    OrganicSearch,
    PaidSearch,
    Display,
    Social,
    Email,
    Referral,
    AffiliateMarketing,
    Video,
    Audio,
    Sms,
    Mobile,
    Other,
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Channel::Direct => write!(f, "Direct"),
            Channel::OrganicSearch => write!(f, "Organic Search"),
            Channel::PaidSearch => write!(f, "Paid Search"),
            Channel::Display => write!(f, "Display"),
            Channel::Social => write!(f, "Social"),
            Channel::Email => write!(f, "Email"),
            Channel::Referral => write!(f, "Referral"),
            Channel::AffiliateMarketing => write!(f, "Affiliate"),
            Channel::Video => write!(f, "Video"),
            Channel::Audio => write!(f, "Audio"),
            Channel::Sms => write!(f, "SMS"),
            Channel::Mobile => write!(f, "Mobile"),
            Channel::Other => write!(f, "(Other)"),
        }
    }
}

/// Source/Medium data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMediumData {
    pub source: String,
    pub medium: String,
    pub users: u64,
    pub new_users: u64,
    pub sessions: u64,
    pub bounce_rate: f64,
    pub pages_per_session: f64,
    pub avg_session_duration: f64,
    pub goal_conversion_rate: f64,
    pub goal_completions: u64,
    pub revenue: f64,
    pub percentage: f64,
}

/// Channel trend data over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelTrendData {
    pub date: NaiveDate,
    pub direct: u64,
    pub organic_search: u64,
    pub paid_search: u64,
    pub social: u64,
    pub referral: u64,
    pub email: u64,
    pub display: u64,
    pub other: u64,
}

/// Acquisition totals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcquisitionTotals {
    pub users: u64,
    pub new_users: u64,
    pub sessions: u64,
    pub bounce_rate: f64,
    pub pages_per_session: f64,
    pub avg_session_duration: f64,
    pub goal_completions: u64,
    pub revenue: f64,
}

/// Acquisition comparison data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcquisitionComparison {
    pub users_change: f64,
    pub new_users_change: f64,
    pub sessions_change: f64,
    pub bounce_rate_change: f64,
    pub goal_completions_change: f64,
    pub revenue_change: f64,
}

/// All traffic data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllTrafficData {
    pub date_range: DateRange,
    pub channels: Vec<ChannelAcquisition>,
    pub treemap_data: Vec<TreemapNode>,
    pub source_medium: Vec<SourceMediumData>,
    pub referrals: Vec<ReferralData>,
}

/// Treemap node for traffic visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreemapNode {
    pub name: String,
    pub value: u64,
    pub color: String,
    pub children: Vec<TreemapNode>,
}

/// Referral data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralData {
    pub source: String,
    pub full_referrer: Option<String>,
    pub users: u64,
    pub new_users: u64,
    pub sessions: u64,
    pub bounce_rate: f64,
    pub pages_per_session: f64,
    pub avg_session_duration: f64,
    pub goal_conversion_rate: f64,
    pub goal_completions: u64,
    pub revenue: f64,
    pub percentage: f64,
}

/// Campaign data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignAnalysis {
    pub date_range: DateRange,
    pub campaigns: Vec<CampaignDetailData>,
    pub source_medium_campaign: Vec<SourceMediumCampaign>,
    pub performance_over_time: Vec<CampaignPerformanceData>,
}

/// Detailed campaign data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignDetailData {
    pub campaign: String,
    pub source: String,
    pub medium: String,
    pub users: u64,
    pub new_users: u64,
    pub sessions: u64,
    pub bounce_rate: f64,
    pub pages_per_session: f64,
    pub avg_session_duration: f64,
    pub goal_completions: u64,
    pub goal_conversion_rate: f64,
    pub goal_value: f64,
    pub transactions: u64,
    pub revenue: f64,
    pub cost: Option<f64>,
    pub roas: Option<f64>,
    pub cpc: Option<f64>,
    pub cpm: Option<f64>,
}

/// Source/Medium/Campaign combination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMediumCampaign {
    pub source: String,
    pub medium: String,
    pub campaign: String,
    pub ad_content: Option<String>,
    pub keyword: Option<String>,
    pub sessions: u64,
    pub conversions: u64,
    pub revenue: f64,
}

/// Campaign performance over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignPerformanceData {
    pub date: NaiveDate,
    pub campaign: String,
    pub sessions: u64,
    pub conversions: u64,
    pub revenue: f64,
    pub cost: Option<f64>,
}

/// Social network data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialAnalysis {
    pub date_range: DateRange,
    pub network_referrals: Vec<SocialNetworkData>,
    pub landing_pages: Vec<SocialLandingPage>,
    pub conversions: Vec<SocialConversion>,
    pub user_flow: Vec<SocialFlowNode>,
    pub plugins: Vec<SocialPluginData>,
}

/// Social network referral data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialNetworkData {
    pub social_network: String,
    pub sessions: u64,
    pub users: u64,
    pub pageviews: u64,
    pub avg_session_duration: f64,
    pub pages_per_session: f64,
    pub percentage: f64,
}

/// Social landing page data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialLandingPage {
    pub landing_page: String,
    pub social_network: String,
    pub sessions: u64,
    pub pageviews: u64,
    pub avg_session_duration: f64,
    pub percentage: f64,
}

/// Social conversion data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialConversion {
    pub social_network: String,
    pub assisted_conversions: u64,
    pub assisted_value: f64,
    pub last_interaction_conversions: u64,
    pub last_interaction_value: f64,
}

/// Social flow node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialFlowNode {
    pub node_name: String,
    pub node_type: String,
    pub sessions: u64,
    pub exits: u64,
}

/// Social plugin activity data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialPluginData {
    pub social_source: String,
    pub social_action: String,
    pub social_entity: String,
    pub unique_actions: u64,
    pub total_actions: u64,
}

/// Google Ads integration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleAdsData {
    pub date_range: DateRange,
    pub campaigns: Vec<GoogleAdsCampaign>,
    pub ad_groups: Vec<GoogleAdsAdGroup>,
    pub keywords: Vec<GoogleAdsKeyword>,
    pub search_queries: Vec<GoogleAdsSearchQuery>,
    pub display_targeting: Vec<GoogleAdsDisplayTarget>,
    pub performance_trend: Vec<GoogleAdsPerformanceTrend>,
}

/// Google Ads campaign data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleAdsCampaign {
    pub campaign_id: String,
    pub campaign_name: String,
    pub campaign_type: String,
    pub impressions: u64,
    pub clicks: u64,
    pub cost: f64,
    pub ctr: f64,
    pub cpc: f64,
    pub sessions: u64,
    pub bounce_rate: f64,
    pub pages_per_session: f64,
    pub goal_completions: u64,
    pub goal_conversion_rate: f64,
    pub revenue: f64,
    pub roas: f64,
}

/// Google Ads ad group data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleAdsAdGroup {
    pub ad_group_id: String,
    pub ad_group_name: String,
    pub campaign_name: String,
    pub impressions: u64,
    pub clicks: u64,
    pub cost: f64,
    pub ctr: f64,
    pub cpc: f64,
    pub sessions: u64,
    pub conversions: u64,
    pub revenue: f64,
}

/// Google Ads keyword data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleAdsKeyword {
    pub keyword: String,
    pub keyword_match_type: String,
    pub ad_group: String,
    pub campaign: String,
    pub impressions: u64,
    pub clicks: u64,
    pub cost: f64,
    pub ctr: f64,
    pub cpc: f64,
    pub quality_score: Option<u32>,
    pub sessions: u64,
    pub bounce_rate: f64,
    pub conversions: u64,
    pub revenue: f64,
}

/// Google Ads search query data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleAdsSearchQuery {
    pub search_query: String,
    pub matched_keyword: String,
    pub impressions: u64,
    pub clicks: u64,
    pub cost: f64,
    pub conversions: u64,
}

/// Google Ads display targeting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleAdsDisplayTarget {
    pub targeting_type: String,
    pub target: String,
    pub impressions: u64,
    pub clicks: u64,
    pub cost: f64,
    pub conversions: u64,
}

/// Google Ads performance trend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleAdsPerformanceTrend {
    pub date: NaiveDate,
    pub impressions: u64,
    pub clicks: u64,
    pub cost: f64,
    pub conversions: u64,
    pub revenue: f64,
}

/// Search Console integration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConsoleData {
    pub date_range: DateRange,
    pub queries: Vec<SearchConsoleQuery>,
    pub pages: Vec<SearchConsolePage>,
    pub countries: Vec<SearchConsoleCountry>,
    pub devices: Vec<SearchConsoleDevice>,
    pub search_appearance: Vec<SearchConsoleAppearance>,
    pub performance_trend: Vec<SearchConsolePerformance>,
}

/// Search Console query data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConsoleQuery {
    pub query: String,
    pub clicks: u64,
    pub impressions: u64,
    pub ctr: f64,
    pub position: f64,
    pub sessions: Option<u64>,
    pub goal_completions: Option<u64>,
}

/// Search Console page data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConsolePage {
    pub page: String,
    pub clicks: u64,
    pub impressions: u64,
    pub ctr: f64,
    pub position: f64,
}

/// Search Console country data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConsoleCountry {
    pub country: String,
    pub country_code: String,
    pub clicks: u64,
    pub impressions: u64,
    pub ctr: f64,
    pub position: f64,
}

/// Search Console device data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConsoleDevice {
    pub device: String,
    pub clicks: u64,
    pub impressions: u64,
    pub ctr: f64,
    pub position: f64,
}

/// Search Console search appearance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConsoleAppearance {
    pub search_appearance: String,
    pub clicks: u64,
    pub impressions: u64,
    pub ctr: f64,
}

/// Search Console performance over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConsolePerformance {
    pub date: NaiveDate,
    pub clicks: u64,
    pub impressions: u64,
    pub ctr: f64,
    pub position: f64,
}

/// UTM parameter analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtmAnalysis {
    pub date_range: DateRange,
    pub by_source: Vec<UtmSourceData>,
    pub by_medium: Vec<UtmMediumData>,
    pub by_campaign: Vec<UtmCampaignData>,
    pub by_content: Vec<UtmContentData>,
    pub by_term: Vec<UtmTermData>,
}

/// UTM source data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtmSourceData {
    pub utm_source: String,
    pub sessions: u64,
    pub users: u64,
    pub conversions: u64,
    pub revenue: f64,
    pub percentage: f64,
}

/// UTM medium data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtmMediumData {
    pub utm_medium: String,
    pub sessions: u64,
    pub users: u64,
    pub conversions: u64,
    pub revenue: f64,
    pub percentage: f64,
}

/// UTM campaign data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtmCampaignData {
    pub utm_campaign: String,
    pub sessions: u64,
    pub users: u64,
    pub conversions: u64,
    pub revenue: f64,
    pub percentage: f64,
}

/// UTM content data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtmContentData {
    pub utm_content: String,
    pub sessions: u64,
    pub users: u64,
    pub conversions: u64,
    pub revenue: f64,
    pub percentage: f64,
}

/// UTM term data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtmTermData {
    pub utm_term: String,
    pub sessions: u64,
    pub users: u64,
    pub conversions: u64,
    pub revenue: f64,
    pub percentage: f64,
}
