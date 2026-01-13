//! Google Analytics API request/response models

use serde::{Deserialize, Serialize};

/// Google Analytics Data API (GA4) run report request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunReportRequest {
    pub property: String,
    pub date_ranges: Vec<ApiDateRange>,
    pub dimensions: Option<Vec<Dimension>>,
    pub metrics: Vec<Metric>,
    pub dimension_filter: Option<FilterExpression>,
    pub metric_filter: Option<FilterExpression>,
    pub order_bys: Option<Vec<OrderBy>>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub metric_aggregations: Option<Vec<String>>,
    pub keep_empty_rows: Option<bool>,
    pub return_property_quota: Option<bool>,
}

/// Date range for API requests
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiDateRange {
    pub start_date: String,
    pub end_date: String,
    pub name: Option<String>,
}

/// Dimension definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dimension {
    pub name: String,
    pub dimension_expression: Option<DimensionExpression>,
}

/// Dimension expression
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DimensionExpression {
    pub lower_case: Option<CaseExpression>,
    pub upper_case: Option<CaseExpression>,
    pub concatenate: Option<ConcatenateExpression>,
}

/// Case expression
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseExpression {
    pub dimension_name: String,
}

/// Concatenate expression
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConcatenateExpression {
    pub dimension_names: Vec<String>,
    pub delimiter: Option<String>,
}

/// Metric definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metric {
    pub name: String,
    pub expression: Option<String>,
    pub invisible: Option<bool>,
}

/// Filter expression
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterExpression {
    pub and_group: Option<FilterExpressionList>,
    pub or_group: Option<FilterExpressionList>,
    pub not_expression: Option<Box<FilterExpression>>,
    pub filter: Option<Filter>,
}

/// Filter expression list
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterExpressionList {
    pub expressions: Vec<FilterExpression>,
}

/// Individual filter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Filter {
    pub field_name: String,
    pub string_filter: Option<StringFilter>,
    pub in_list_filter: Option<InListFilter>,
    pub numeric_filter: Option<NumericFilter>,
    pub between_filter: Option<BetweenFilter>,
}

/// String filter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StringFilter {
    pub match_type: StringFilterMatchType,
    pub value: String,
    pub case_sensitive: Option<bool>,
}

/// String filter match types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StringFilterMatchType {
    Exact,
    BeginsWith,
    EndsWith,
    Contains,
    FullRegexp,
    PartialRegexp,
}

/// In-list filter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InListFilter {
    pub values: Vec<String>,
    pub case_sensitive: Option<bool>,
}

/// Numeric filter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NumericFilter {
    pub operation: NumericFilterOperation,
    pub value: NumericValue,
}

/// Numeric filter operations
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NumericFilterOperation {
    Equal,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

/// Numeric value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NumericValue {
    pub int64_value: Option<String>,
    pub double_value: Option<f64>,
}

/// Between filter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BetweenFilter {
    pub from_value: NumericValue,
    pub to_value: NumericValue,
}

/// Order by specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderBy {
    pub desc: Option<bool>,
    pub metric: Option<MetricOrderBy>,
    pub dimension: Option<DimensionOrderBy>,
    pub pivot: Option<PivotOrderBy>,
}

/// Metric order by
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricOrderBy {
    pub metric_name: String,
}

/// Dimension order by
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DimensionOrderBy {
    pub dimension_name: String,
    pub order_type: Option<DimensionOrderType>,
}

/// Dimension order types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DimensionOrderType {
    Alphanumeric,
    CaseInsensitiveAlphanumeric,
    Numeric,
}

/// Pivot order by
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PivotOrderBy {
    pub metric_name: String,
    pub pivot_selections: Vec<PivotSelection>,
}

/// Pivot selection
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PivotSelection {
    pub dimension_name: String,
    pub dimension_value: String,
}

/// Run report response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunReportResponse {
    pub dimension_headers: Option<Vec<DimensionHeader>>,
    pub metric_headers: Option<Vec<MetricHeader>>,
    pub rows: Option<Vec<Row>>,
    pub totals: Option<Vec<Row>>,
    pub maximums: Option<Vec<Row>>,
    pub minimums: Option<Vec<Row>>,
    pub row_count: Option<i32>,
    pub metadata: Option<ResponseMetaData>,
    pub property_quota: Option<PropertyQuota>,
    pub kind: Option<String>,
}

/// Dimension header
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DimensionHeader {
    pub name: String,
}

/// Metric header
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricHeader {
    pub name: String,
    #[serde(rename = "type")]
    pub metric_type: Option<MetricType>,
}

/// Metric types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricType {
    MetricTypeUnspecified,
    TypeInteger,
    TypeFloat,
    TypeSeconds,
    TypeMilliseconds,
    TypeMinutes,
    TypeHours,
    TypeStandard,
    TypeCurrency,
    TypeFeet,
    TypeMiles,
    TypeMeters,
    TypeKilometers,
}

/// Data row
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    pub dimension_values: Option<Vec<DimensionValue>>,
    pub metric_values: Option<Vec<MetricValue>>,
}

/// Dimension value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DimensionValue {
    pub value: Option<String>,
    pub one_value: Option<String>,
}

/// Metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricValue {
    pub value: Option<String>,
    pub one_value: Option<String>,
}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMetaData {
    pub data_loss_from_other_row: Option<bool>,
    pub schema_restriction_response: Option<SchemaRestrictionResponse>,
    pub currency_code: Option<String>,
    pub time_zone: Option<String>,
}

/// Schema restriction response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaRestrictionResponse {
    pub active_metric_restrictions: Option<Vec<ActiveMetricRestriction>>,
}

/// Active metric restriction
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveMetricRestriction {
    pub metric_name: Option<String>,
    pub restricted_metric_types: Option<Vec<String>>,
}

/// Property quota information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyQuota {
    pub tokens_per_day: Option<QuotaStatus>,
    pub tokens_per_hour: Option<QuotaStatus>,
    pub concurrent_requests: Option<QuotaStatus>,
    pub server_errors_per_project_per_hour: Option<QuotaStatus>,
    pub potentially_thresholded_requests_per_hour: Option<QuotaStatus>,
    pub tokens_per_project_per_hour: Option<QuotaStatus>,
}

/// Quota status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaStatus {
    pub consumed: Option<i32>,
    pub remaining: Option<i32>,
}

/// Real-time report request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunRealtimeReportRequest {
    pub property: String,
    pub dimensions: Option<Vec<Dimension>>,
    pub metrics: Vec<Metric>,
    pub dimension_filter: Option<FilterExpression>,
    pub metric_filter: Option<FilterExpression>,
    pub limit: Option<i64>,
    pub metric_aggregations: Option<Vec<String>>,
    pub order_bys: Option<Vec<OrderBy>>,
    pub return_property_quota: Option<bool>,
    pub minute_ranges: Option<Vec<MinuteRange>>,
}

/// Minute range for real-time reports
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MinuteRange {
    pub name: Option<String>,
    pub start_minutes_ago: Option<i32>,
    pub end_minutes_ago: Option<i32>,
}

/// Real-time report response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunRealtimeReportResponse {
    pub dimension_headers: Option<Vec<DimensionHeader>>,
    pub metric_headers: Option<Vec<MetricHeader>>,
    pub rows: Option<Vec<Row>>,
    pub totals: Option<Vec<Row>>,
    pub maximums: Option<Vec<Row>>,
    pub minimums: Option<Vec<Row>>,
    pub row_count: Option<i32>,
    pub property_quota: Option<PropertyQuota>,
    pub kind: Option<String>,
}

/// Batch run reports request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchRunReportsRequest {
    pub property: String,
    pub requests: Vec<RunReportRequest>,
}

/// Batch run reports response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchRunReportsResponse {
    pub reports: Option<Vec<RunReportResponse>>,
    pub kind: Option<String>,
}

/// Pivot report request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunPivotReportRequest {
    pub property: String,
    pub date_ranges: Vec<ApiDateRange>,
    pub pivots: Vec<Pivot>,
    pub dimensions: Option<Vec<Dimension>>,
    pub metrics: Vec<Metric>,
    pub dimension_filter: Option<FilterExpression>,
    pub metric_filter: Option<FilterExpression>,
    pub keep_empty_rows: Option<bool>,
    pub return_property_quota: Option<bool>,
}

/// Pivot definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pivot {
    pub field_names: Vec<String>,
    pub order_bys: Option<Vec<OrderBy>>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub metric_aggregations: Option<Vec<String>>,
}

/// Pivot report response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunPivotReportResponse {
    pub pivot_headers: Option<Vec<PivotHeader>>,
    pub dimension_headers: Option<Vec<DimensionHeader>>,
    pub metric_headers: Option<Vec<MetricHeader>>,
    pub rows: Option<Vec<Row>>,
    pub aggregates: Option<Vec<Row>>,
    pub metadata: Option<ResponseMetaData>,
    pub property_quota: Option<PropertyQuota>,
    pub kind: Option<String>,
}

/// Pivot header
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PivotHeader {
    pub pivot_dimension_headers: Option<Vec<PivotDimensionHeader>>,
    pub row_count: Option<i32>,
}

/// Pivot dimension header
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PivotDimensionHeader {
    pub dimension_values: Option<Vec<DimensionValue>>,
}

/// Get metadata request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetMetadataRequest {
    pub name: String,
}

/// Metadata response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub name: Option<String>,
    pub dimensions: Option<Vec<DimensionMetadata>>,
    pub metrics: Option<Vec<MetricMetadata>>,
}

/// Dimension metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DimensionMetadata {
    pub api_name: Option<String>,
    pub ui_name: Option<String>,
    pub description: Option<String>,
    pub deprecated_api_names: Option<Vec<String>>,
    pub custom_definition: Option<bool>,
    pub category: Option<String>,
}

/// Metric metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricMetadata {
    pub api_name: Option<String>,
    pub ui_name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub metric_type: Option<MetricType>,
    pub expression: Option<String>,
    pub deprecated_api_names: Option<Vec<String>>,
    pub custom_definition: Option<bool>,
    pub blocked_reasons: Option<Vec<String>>,
    pub category: Option<String>,
}

/// OAuth2 token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub scope: Option<String>,
}

/// Google Analytics Admin API - Account summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSummary {
    pub name: Option<String>,
    pub account: Option<String>,
    pub display_name: Option<String>,
    pub property_summaries: Option<Vec<PropertySummary>>,
}

/// Property summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertySummary {
    pub property: Option<String>,
    pub display_name: Option<String>,
    pub property_type: Option<String>,
}

/// List account summaries response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAccountSummariesResponse {
    pub account_summaries: Option<Vec<AccountSummary>>,
    pub next_page_token: Option<String>,
}

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub error: ApiErrorDetail,
}

/// API error detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorDetail {
    pub code: i32,
    pub message: String,
    pub status: Option<String>,
    pub details: Option<Vec<serde_json::Value>>,
}

/// Funnel report request (GA4)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunFunnelReportRequest {
    pub property: String,
    pub date_ranges: Vec<ApiDateRange>,
    pub funnel: Funnel,
    pub funnel_breakdown: Option<FunnelBreakdown>,
    pub funnel_next_action: Option<FunnelNextAction>,
    pub funnel_visualization_type: Option<String>,
    pub segments: Option<Vec<Segment>>,
    pub limit: Option<i64>,
    pub dimension_filter: Option<FilterExpression>,
    pub return_property_quota: Option<bool>,
}

/// Funnel definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Funnel {
    pub is_open_funnel: Option<bool>,
    pub steps: Vec<FunnelStep>,
}

/// Funnel step
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunnelStep {
    pub name: Option<String>,
    pub is_directly_followed_by: Option<bool>,
    pub within_duration_from_prior_step: Option<String>,
    pub filter_expression: Option<FunnelFilterExpression>,
}

/// Funnel filter expression
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunnelFilterExpression {
    pub and_group: Option<FunnelFilterExpressionList>,
    pub or_group: Option<FunnelFilterExpressionList>,
    pub not_expression: Option<Box<FunnelFilterExpression>>,
    pub funnel_field_filter: Option<FunnelFieldFilter>,
    pub funnel_event_filter: Option<FunnelEventFilter>,
}

/// Funnel filter expression list
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunnelFilterExpressionList {
    pub expressions: Vec<FunnelFilterExpression>,
}

/// Funnel field filter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunnelFieldFilter {
    pub field_name: String,
    pub string_filter: Option<StringFilter>,
    pub in_list_filter: Option<InListFilter>,
    pub numeric_filter: Option<NumericFilter>,
    pub between_filter: Option<BetweenFilter>,
}

/// Funnel event filter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunnelEventFilter {
    pub event_name: Option<String>,
    pub funnel_parameter_filter_expression: Option<Box<FunnelFilterExpression>>,
}

/// Funnel breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunnelBreakdown {
    pub breakdown_dimension: Option<Dimension>,
    pub limit: Option<i64>,
}

/// Funnel next action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunnelNextAction {
    pub next_action_dimension: Option<Dimension>,
    pub limit: Option<i64>,
}

/// Segment definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    pub name: Option<String>,
    pub user_segment: Option<UserSegment>,
    pub session_segment: Option<SessionSegment>,
    pub event_segment: Option<EventSegment>,
}

/// User segment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSegment {
    pub user_inclusion_criteria: Option<UserSegmentCriteria>,
    pub exclusion: Option<UserSegmentExclusion>,
}

/// User segment criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSegmentCriteria {
    pub and_condition_groups: Option<Vec<UserSegmentConditionGroup>>,
    pub and_sequence_groups: Option<Vec<UserSegmentSequenceGroup>>,
}

/// User segment condition group
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSegmentConditionGroup {
    pub condition_scoping: Option<String>,
    pub segment_filter_expression: Option<SegmentFilterExpression>,
}

/// User segment sequence group
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSegmentSequenceGroup {
    pub sequence_scoping: Option<String>,
    pub sequence_maximum_duration: Option<String>,
    pub user_sequence_steps: Option<Vec<UserSequenceStep>>,
}

/// User sequence step
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSequenceStep {
    pub is_directly_followed_by: Option<bool>,
    pub step_scoping: Option<String>,
    pub segment_filter_expression: Option<SegmentFilterExpression>,
}

/// User segment exclusion
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSegmentExclusion {
    pub user_exclusion_duration: Option<String>,
    pub user_exclusion_criteria: Option<UserSegmentCriteria>,
}

/// Session segment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSegment {
    pub session_inclusion_criteria: Option<SessionSegmentCriteria>,
    pub exclusion: Option<SessionSegmentExclusion>,
}

/// Session segment criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSegmentCriteria {
    pub and_condition_groups: Option<Vec<SessionSegmentConditionGroup>>,
}

/// Session segment condition group
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSegmentConditionGroup {
    pub condition_scoping: Option<String>,
    pub segment_filter_expression: Option<SegmentFilterExpression>,
}

/// Session segment exclusion
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSegmentExclusion {
    pub session_exclusion_duration: Option<String>,
    pub session_exclusion_criteria: Option<SessionSegmentCriteria>,
}

/// Event segment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventSegment {
    pub event_inclusion_criteria: Option<EventSegmentCriteria>,
    pub exclusion: Option<EventSegmentExclusion>,
}

/// Event segment criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventSegmentCriteria {
    pub and_condition_groups: Option<Vec<EventSegmentConditionGroup>>,
}

/// Event segment condition group
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventSegmentConditionGroup {
    pub condition_scoping: Option<String>,
    pub segment_filter_expression: Option<SegmentFilterExpression>,
}

/// Event segment exclusion
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventSegmentExclusion {
    pub event_exclusion_duration: Option<String>,
    pub event_exclusion_criteria: Option<EventSegmentCriteria>,
}

/// Segment filter expression
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentFilterExpression {
    pub and_group: Option<SegmentFilterExpressionList>,
    pub or_group: Option<SegmentFilterExpressionList>,
    pub not_expression: Option<Box<SegmentFilterExpression>>,
    pub segment_filter: Option<SegmentFilter>,
    pub segment_event_filter: Option<SegmentEventFilter>,
}

/// Segment filter expression list
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentFilterExpressionList {
    pub expressions: Vec<SegmentFilterExpression>,
}

/// Segment filter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentFilter {
    pub field_name: String,
    pub string_filter: Option<StringFilter>,
    pub in_list_filter: Option<InListFilter>,
    pub numeric_filter: Option<NumericFilter>,
    pub between_filter: Option<BetweenFilter>,
    pub filter_scoping: Option<SegmentFilterScoping>,
}

/// Segment filter scoping
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentFilterScoping {
    pub at_any_point_in_time: Option<bool>,
    pub in_any_n_day_period: Option<i64>,
}

/// Segment event filter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentEventFilter {
    pub event_name: Option<String>,
    pub segment_parameter_filter_expression: Option<Box<SegmentFilterExpression>>,
}
