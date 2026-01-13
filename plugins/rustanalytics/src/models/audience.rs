//! Audience analytics models

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::DateRange;

/// Audience overview data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudienceOverview {
    pub date_range: DateRange,
    pub users: u64,
    pub new_users: u64,
    pub sessions: u64,
    pub sessions_per_user: f64,
    pub pageviews: u64,
    pub pages_per_session: f64,
    pub avg_session_duration: f64,
    pub bounce_rate: f64,
    pub user_trend: Vec<UserTrendData>,
    pub comparison: Option<AudienceComparison>,
}

/// User trend data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTrendData {
    pub date: NaiveDate,
    pub users: u64,
    pub new_users: u64,
    pub sessions: u64,
}

/// Audience comparison data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudienceComparison {
    pub users_change: f64,
    pub new_users_change: f64,
    pub sessions_change: f64,
    pub sessions_per_user_change: f64,
    pub pageviews_change: f64,
    pub pages_per_session_change: f64,
    pub avg_session_duration_change: f64,
    pub bounce_rate_change: f64,
}

/// Demographics overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemographicsOverview {
    pub date_range: DateRange,
    pub age_breakdown: Vec<AgeGroupData>,
    pub gender_breakdown: GenderBreakdown,
    pub interests: Vec<InterestCategoryData>,
}

/// Age group data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgeGroupData {
    pub age_group: AgeGroup,
    pub users: u64,
    pub sessions: u64,
    pub bounce_rate: f64,
    pub pages_per_session: f64,
    pub avg_session_duration: f64,
    pub conversions: u64,
    pub revenue: f64,
    pub percentage: f64,
}

/// Age groups
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AgeGroup {
    #[serde(rename = "18-24")]
    Age18To24,
    #[serde(rename = "25-34")]
    Age25To34,
    #[serde(rename = "35-44")]
    Age35To44,
    #[serde(rename = "45-54")]
    Age45To54,
    #[serde(rename = "55-64")]
    Age55To64,
    #[serde(rename = "65+")]
    Age65Plus,
}

/// Gender breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenderBreakdown {
    pub male: GenderData,
    pub female: GenderData,
}

/// Gender specific data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenderData {
    pub users: u64,
    pub sessions: u64,
    pub bounce_rate: f64,
    pub pages_per_session: f64,
    pub avg_session_duration: f64,
    pub conversions: u64,
    pub revenue: f64,
    pub percentage: f64,
}

/// Interest category data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterestCategoryData {
    pub category: String,
    pub subcategory: Option<String>,
    pub users: u64,
    pub sessions: u64,
    pub bounce_rate: f64,
    pub pages_per_session: f64,
    pub conversions: u64,
    pub percentage: f64,
}

/// Geographic data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoData {
    pub date_range: DateRange,
    pub countries: Vec<CountryData>,
    pub regions: Vec<RegionData>,
    pub cities: Vec<CityData>,
    pub languages: Vec<LanguageData>,
}

/// Country-level data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryData {
    pub country: String,
    pub country_code: String,
    pub users: u64,
    pub new_users: u64,
    pub sessions: u64,
    pub bounce_rate: f64,
    pub pages_per_session: f64,
    pub avg_session_duration: f64,
    pub conversions: u64,
    pub revenue: f64,
    pub percentage: f64,
}

/// Region-level data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionData {
    pub region: String,
    pub country: String,
    pub users: u64,
    pub sessions: u64,
    pub bounce_rate: f64,
    pub conversions: u64,
    pub percentage: f64,
}

/// City-level data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityData {
    pub city: String,
    pub region: Option<String>,
    pub country: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub users: u64,
    pub sessions: u64,
    pub bounce_rate: f64,
    pub pages_per_session: f64,
    pub avg_session_duration: f64,
    pub conversions: u64,
    pub percentage: f64,
}

/// Language data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageData {
    pub language: String,
    pub language_code: String,
    pub users: u64,
    pub sessions: u64,
    pub bounce_rate: f64,
    pub pages_per_session: f64,
    pub percentage: f64,
}

/// Behavior data for audience
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudienceBehavior {
    pub date_range: DateRange,
    pub new_vs_returning: NewVsReturning,
    pub frequency: Vec<FrequencyData>,
    pub recency: Vec<RecencyData>,
    pub engagement: EngagementData,
}

/// New vs returning users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewVsReturning {
    pub new_users: UserTypeData,
    pub returning_users: UserTypeData,
}

/// User type data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTypeData {
    pub users: u64,
    pub sessions: u64,
    pub bounce_rate: f64,
    pub pages_per_session: f64,
    pub avg_session_duration: f64,
    pub conversions: u64,
    pub revenue: f64,
    pub percentage: f64,
}

/// Frequency data (sessions per user)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyData {
    pub session_count: String,
    pub users: u64,
    pub sessions: u64,
    pub pageviews: u64,
    pub conversions: u64,
    pub revenue: f64,
    pub percentage: f64,
}

/// Recency data (days since last session)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecencyData {
    pub days_since_last_session: String,
    pub users: u64,
    pub sessions: u64,
    pub pageviews: u64,
    pub conversions: u64,
    pub percentage: f64,
}

/// User engagement metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementData {
    pub session_duration_breakdown: Vec<SessionDurationBucket>,
    pub page_depth_breakdown: Vec<PageDepthBucket>,
}

/// Session duration bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDurationBucket {
    pub duration_range: String,
    pub sessions: u64,
    pub percentage: f64,
}

/// Page depth bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageDepthBucket {
    pub page_depth: String,
    pub sessions: u64,
    pub percentage: f64,
}

/// Technology data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnologyData {
    pub date_range: DateRange,
    pub browsers: Vec<BrowserData>,
    pub operating_systems: Vec<OperatingSystemData>,
    pub screen_resolutions: Vec<ScreenResolutionData>,
    pub screen_colors: Vec<ScreenColorData>,
    pub flash_versions: Vec<FlashVersionData>,
    pub java_support: JavaSupportData,
}

/// Browser data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserData {
    pub browser: String,
    pub browser_version: Option<String>,
    pub users: u64,
    pub sessions: u64,
    pub bounce_rate: f64,
    pub pages_per_session: f64,
    pub avg_session_duration: f64,
    pub conversions: u64,
    pub percentage: f64,
}

/// Operating system data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatingSystemData {
    pub operating_system: String,
    pub os_version: Option<String>,
    pub users: u64,
    pub sessions: u64,
    pub bounce_rate: f64,
    pub pages_per_session: f64,
    pub avg_session_duration: f64,
    pub conversions: u64,
    pub percentage: f64,
}

/// Screen resolution data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenResolutionData {
    pub resolution: String,
    pub users: u64,
    pub sessions: u64,
    pub bounce_rate: f64,
    pub percentage: f64,
}

/// Screen color depth data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenColorData {
    pub color_depth: String,
    pub users: u64,
    pub sessions: u64,
    pub percentage: f64,
}

/// Flash version data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashVersionData {
    pub flash_version: String,
    pub users: u64,
    pub sessions: u64,
    pub percentage: f64,
}

/// Java support data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaSupportData {
    pub java_enabled: u64,
    pub java_disabled: u64,
    pub enabled_percentage: f64,
}

/// Mobile/device data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileData {
    pub date_range: DateRange,
    pub device_categories: Vec<DeviceCategoryData>,
    pub mobile_devices: Vec<MobileDeviceData>,
    pub mobile_operating_systems: Vec<MobileOsData>,
    pub service_providers: Vec<ServiceProviderData>,
}

/// Device category data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCategoryData {
    pub device_category: DeviceCategory,
    pub users: u64,
    pub sessions: u64,
    pub bounce_rate: f64,
    pub pages_per_session: f64,
    pub avg_session_duration: f64,
    pub conversions: u64,
    pub revenue: f64,
    pub percentage: f64,
}

/// Device categories
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DeviceCategory {
    Desktop,
    Mobile,
    Tablet,
}

/// Mobile device data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileDeviceData {
    pub device_brand: String,
    pub device_model: Option<String>,
    pub users: u64,
    pub sessions: u64,
    pub bounce_rate: f64,
    pub pages_per_session: f64,
    pub percentage: f64,
}

/// Mobile OS data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileOsData {
    pub operating_system: String,
    pub os_version: Option<String>,
    pub users: u64,
    pub sessions: u64,
    pub bounce_rate: f64,
    pub percentage: f64,
}

/// Service provider data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceProviderData {
    pub service_provider: String,
    pub users: u64,
    pub sessions: u64,
    pub bounce_rate: f64,
    pub percentage: f64,
}

/// User flow data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFlowData {
    pub date_range: DateRange,
    pub nodes: Vec<FlowNode>,
    pub connections: Vec<FlowConnection>,
    pub drop_offs: Vec<FlowDropOff>,
}

/// Flow visualization node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowNode {
    pub id: String,
    pub name: String,
    pub node_type: FlowNodeType,
    pub step: u32,
    pub sessions: u64,
    pub percentage: f64,
}

/// Flow node types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FlowNodeType {
    Source,
    Page,
    Event,
    Exit,
}

/// Flow connection between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowConnection {
    pub from_node: String,
    pub to_node: String,
    pub sessions: u64,
    pub percentage: f64,
}

/// Drop-off point in flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowDropOff {
    pub node: String,
    pub step: u32,
    pub sessions: u64,
    pub drop_off_rate: f64,
}

/// Cohort analysis data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohortAnalysis {
    pub date_range: DateRange,
    pub cohort_type: CohortType,
    pub cohort_size: CohortSize,
    pub metric: String,
    pub cohorts: Vec<CohortData>,
}

/// Cohort type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CohortType {
    AcquisitionDate,
    FirstVisit,
    Transaction,
}

/// Cohort size
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CohortSize {
    Day,
    Week,
    Month,
}

/// Individual cohort data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohortData {
    pub cohort_date: NaiveDate,
    pub users: u64,
    pub retention: Vec<RetentionData>,
}

/// Retention data for cohort
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionData {
    pub period: u32,
    pub users: u64,
    pub retention_rate: f64,
    pub metric_value: f64,
}

/// Lifetime value data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifetimeValueData {
    pub date_range: DateRange,
    pub channels: Vec<ChannelLtvData>,
    pub overall_ltv: f64,
    pub avg_sessions_to_conversion: f64,
    pub avg_days_to_conversion: f64,
}

/// Channel-level LTV data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelLtvData {
    pub channel: String,
    pub users: u64,
    pub ltv_per_user: f64,
    pub revenue_per_user: f64,
    pub transactions_per_user: f64,
    pub sessions_per_user: f64,
    pub pageviews_per_user: f64,
}
