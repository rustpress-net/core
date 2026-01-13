//! Report models for custom analytics reports

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{DateRange, DateRangePreset, ReportFormat, ReportFrequency, SamplingInfo};

/// Custom report definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomReport {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub metrics: Vec<ReportMetric>,
    pub dimensions: Vec<ReportDimension>,
    pub filters: Vec<ReportFilter>,
    pub segments: Vec<String>,
    pub date_range: DateRangePreset,
    pub chart_type: ChartType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub is_public: bool,
    pub is_favorite: bool,
}

/// Scheduled report configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledReport {
    pub id: Uuid,
    pub report_id: Uuid,
    pub name: String,
    pub frequency: ReportFrequency,
    pub format: ReportFormat,
    pub recipients: Vec<String>,
    pub include_comparison: bool,
    pub enabled: bool,
    pub next_run: DateTime<Utc>,
    pub last_run: Option<DateTime<Utc>>,
    pub last_status: Option<ReportStatus>,
    pub created_at: DateTime<Utc>,
}

/// Report execution status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ReportStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Available metrics for reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetric {
    pub id: String,
    pub name: String,
    pub category: MetricCategory,
    pub data_type: MetricDataType,
    pub aggregation: MetricAggregation,
}

/// Metric categories
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MetricCategory {
    User,
    Session,
    TrafficSources,
    PageTracking,
    ContentGrouping,
    InternalSearch,
    GoalConversions,
    Ecommerce,
    EnhancedEcommerce,
    SocialInteractions,
    UserTimings,
    Exceptions,
    ContentExperiments,
    Custom,
    CalculatedMetrics,
    Lifetime,
    DoubleClick,
    Adsense,
    AdExchange,
    Publisher,
}

/// Metric data type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MetricDataType {
    Integer,
    Float,
    Percent,
    Time,
    Currency,
}

/// Metric aggregation type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MetricAggregation {
    Total,
    Average,
    Minimum,
    Maximum,
    Count,
    CountDistinct,
}

/// Available dimensions for reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportDimension {
    pub id: String,
    pub name: String,
    pub category: DimensionCategory,
}

/// Dimension categories
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DimensionCategory {
    User,
    Session,
    TrafficSources,
    Adwords,
    GoalConversions,
    Platform,
    GeoNetwork,
    System,
    PageTracking,
    ContentGrouping,
    InternalSearch,
    Ecommerce,
    EnhancedEcommerce,
    SocialInteractions,
    Time,
    DoubleClick,
    Audience,
    Publisher,
    Custom,
}

/// Report filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportFilter {
    pub dimension: String,
    pub operator: FilterOperator,
    pub value: String,
    pub case_sensitive: bool,
}

/// Filter operators
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    Regex,
    NotRegex,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Between,
    InList,
}

/// Chart types for reports
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ChartType {
    Line,
    Area,
    Bar,
    Column,
    Pie,
    Donut,
    Table,
    Scorecard,
    Geo,
    Scatter,
    Bubble,
    Treemap,
    Funnel,
    Timeline,
    Histogram,
    Heatmap,
}

/// Report result data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportResult {
    pub report_id: Uuid,
    pub date_range: DateRange,
    pub rows: Vec<ReportRow>,
    pub totals: Option<ReportRow>,
    pub row_count: u64,
    pub sampling_info: Option<SamplingInfo>,
    pub generated_at: DateTime<Utc>,
}

/// Report data row
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRow {
    pub dimensions: Vec<String>,
    pub metrics: Vec<ReportMetricValue>,
}

/// Report metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetricValue {
    pub metric_id: String,
    pub value: serde_json::Value,
    pub formatted_value: String,
}

/// Report export request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportExportRequest {
    pub report_id: Uuid,
    pub format: ReportFormat,
    pub date_range: Option<DateRange>,
    pub include_totals: bool,
    pub include_chart: bool,
}

/// Report export result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportExportResult {
    pub file_name: String,
    pub content_type: String,
    pub data: Vec<u8>,
    pub size_bytes: u64,
}

/// Pre-built report template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: ReportTemplateCategory,
    pub metrics: Vec<String>,
    pub dimensions: Vec<String>,
    pub chart_type: ChartType,
    pub is_premium: bool,
}

/// Report template categories
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ReportTemplateCategory {
    Audience,
    Acquisition,
    Behavior,
    Conversions,
    Ecommerce,
    RealTime,
    Custom,
}

/// Dashboard report widget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardWidget {
    pub id: Uuid,
    pub report_id: Option<Uuid>,
    pub widget_type: WidgetType,
    pub title: String,
    pub position: WidgetPosition,
    pub size: WidgetSize,
    pub refresh_interval: Option<u32>,
    pub settings: serde_json::Value,
}

/// Widget types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WidgetType {
    Metric,
    Chart,
    Table,
    Timeline,
    Geo,
    Funnel,
    RealTime,
    Custom,
}

/// Widget position on dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    pub x: u32,
    pub y: u32,
}

/// Widget size
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WidgetSize {
    Small,
    Medium,
    Large,
    Wide,
    Tall,
    Full,
}

/// Annotation for reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportAnnotation {
    pub id: Uuid,
    pub date: chrono::NaiveDate,
    pub title: String,
    pub description: Option<String>,
    pub annotation_type: AnnotationType,
    pub visibility: AnnotationVisibility,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Annotation types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationType {
    Note,
    Campaign,
    Release,
    Incident,
    Milestone,
    Custom,
}

/// Annotation visibility
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationVisibility {
    Private,
    Shared,
    Public,
}
