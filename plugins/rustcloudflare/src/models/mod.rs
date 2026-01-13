//! Data models for Cloudflare API

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// ============================================================================
// API Response Types
// ============================================================================

/// Standard Cloudflare API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub result: Option<T>,
    pub errors: Option<Vec<ApiError>>,
    pub messages: Option<Vec<ApiMessage>>,
    pub result_info: Option<ResultInfo>,
}

/// API error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: i32,
    pub message: String,
    pub error_chain: Option<Vec<ApiError>>,
}

/// API message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMessage {
    pub code: i32,
    pub message: String,
}

/// Pagination info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultInfo {
    pub page: i32,
    pub per_page: i32,
    pub count: i32,
    pub total_count: i32,
    pub total_pages: i32,
}

/// Delete response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteResponse {
    pub id: String,
}

/// Purge response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurgeResponse {
    pub id: String,
}

// ============================================================================
// Zone Types
// ============================================================================

/// Zone details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zone {
    pub id: String,
    pub name: String,
    pub status: String,
    pub paused: bool,
    pub development_mode: i32,
    #[serde(rename = "type")]
    pub zone_type: String,
    pub name_servers: Vec<String>,
    pub original_name_servers: Option<Vec<String>>,
    pub original_registrar: Option<String>,
    pub original_dnshost: Option<String>,
    pub created_on: Option<DateTime<Utc>>,
    pub modified_on: Option<DateTime<Utc>>,
    pub activated_on: Option<DateTime<Utc>>,
    pub plan: Option<Plan>,
}

/// Cloudflare plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub name: String,
    pub price: f64,
    pub currency: String,
    pub frequency: String,
    pub is_subscribed: bool,
    pub can_subscribe: bool,
}

/// Zone setting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneSetting {
    pub id: String,
    pub value: serde_json::Value,
    pub editable: bool,
    pub modified_on: Option<DateTime<Utc>>,
}

// ============================================================================
// DNS Types
// ============================================================================

/// DNS record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub id: String,
    #[serde(rename = "type")]
    pub record_type: String,
    pub name: String,
    pub content: String,
    pub proxiable: bool,
    pub proxied: bool,
    pub ttl: i32,
    pub locked: bool,
    pub zone_id: String,
    pub zone_name: String,
    pub created_on: Option<DateTime<Utc>>,
    pub modified_on: Option<DateTime<Utc>>,
    pub priority: Option<i32>,
    pub data: Option<DnsRecordData>,
}

/// DNS record data (for SRV, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecordData {
    pub service: Option<String>,
    pub proto: Option<String>,
    pub name: Option<String>,
    pub priority: Option<i32>,
    pub weight: Option<i32>,
    pub port: Option<i32>,
    pub target: Option<String>,
}

/// Create DNS record request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDnsRecord {
    #[serde(rename = "type")]
    pub record_type: String,
    pub name: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxied: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
}

/// Update DNS record request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDnsRecord {
    #[serde(rename = "type")]
    pub record_type: String,
    pub name: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxied: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
}

/// DNS list parameters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DnsListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub record_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "match")]
    pub match_type: Option<String>,
}

// ============================================================================
// SSL/TLS Types
// ============================================================================

/// SSL settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslSettings {
    pub id: String,
    pub value: String,
    pub editable: bool,
    pub modified_on: Option<DateTime<Utc>>,
}

/// SSL certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    pub id: String,
    #[serde(rename = "type")]
    pub cert_type: String,
    pub hosts: Vec<String>,
    pub status: String,
    pub validation_type: Option<String>,
    pub validity_days: Option<i32>,
    pub certificate_authority: Option<String>,
    pub primary_certificate: Option<String>,
    pub certificates: Option<Vec<CertificateDetails>>,
}

/// Certificate details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateDetails {
    pub id: Option<String>,
    pub hosts: Vec<String>,
    pub issuer: Option<String>,
    pub signature: Option<String>,
    pub status: String,
    pub expires_on: Option<DateTime<Utc>>,
}

// ============================================================================
// Security / WAF Types
// ============================================================================

/// WAF rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WafRule {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub detection_mode: String,
    pub zone_id: String,
    pub status: String,
}

/// Firewall rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRule {
    pub id: String,
    pub paused: bool,
    pub description: Option<String>,
    pub action: String,
    pub priority: Option<i32>,
    pub filter: FirewallFilter,
    pub products: Option<Vec<String>>,
    pub created_on: Option<DateTime<Utc>>,
    pub modified_on: Option<DateTime<Utc>>,
}

/// Firewall filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallFilter {
    pub id: String,
    pub expression: String,
    pub paused: bool,
    pub description: Option<String>,
}

/// Create firewall rule request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFirewallRule {
    pub action: String,
    pub filter: CreateFirewallFilter,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paused: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
}

/// Create firewall filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFirewallFilter {
    pub expression: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// IP access rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpAccessRule {
    pub id: String,
    pub mode: String,
    pub allowed_modes: Vec<String>,
    pub notes: Option<String>,
    pub configuration: IpConfiguration,
    pub created_on: Option<DateTime<Utc>>,
    pub modified_on: Option<DateTime<Utc>>,
    pub scope: IpScope,
}

/// IP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpConfiguration {
    pub target: String,
    pub value: String,
}

/// IP scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpScope {
    pub id: String,
    #[serde(rename = "type")]
    pub scope_type: String,
}

/// Create IP access rule request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIpAccessRule {
    pub mode: String,
    pub configuration: IpConfiguration,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

// ============================================================================
// Page Rules Types
// ============================================================================

/// Page rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRule {
    pub id: String,
    pub targets: Vec<PageRuleTarget>,
    pub actions: Vec<PageRuleAction>,
    pub priority: i32,
    pub status: String,
    pub created_on: Option<DateTime<Utc>>,
    pub modified_on: Option<DateTime<Utc>>,
}

/// Page rule target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRuleTarget {
    pub target: String,
    pub constraint: PageRuleConstraint,
}

/// Page rule constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRuleConstraint {
    pub operator: String,
    pub value: String,
}

/// Page rule action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRuleAction {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
}

/// Create page rule request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePageRule {
    pub targets: Vec<PageRuleTarget>,
    pub actions: Vec<PageRuleAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Update page rule request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePageRule {
    pub targets: Vec<PageRuleTarget>,
    pub actions: Vec<PageRuleAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

// ============================================================================
// Workers Types
// ============================================================================

/// Worker script
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worker {
    pub id: String,
    pub etag: Option<String>,
    pub handlers: Option<Vec<String>>,
    pub modified_on: Option<DateTime<Utc>>,
    pub created_on: Option<DateTime<Utc>>,
    pub usage_model: Option<String>,
    pub compatibility_date: Option<String>,
    pub compatibility_flags: Option<Vec<String>>,
}

/// Worker route
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerRoute {
    pub id: String,
    pub pattern: String,
    pub script: Option<String>,
}

// ============================================================================
// Workers KV Types
// ============================================================================

/// KV namespace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvNamespace {
    pub id: String,
    pub title: String,
    pub supports_url_encoding: Option<bool>,
}

/// KV key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvKey {
    pub name: String,
    pub expiration: Option<i64>,
    pub metadata: Option<serde_json::Value>,
}

// ============================================================================
// R2 Storage Types
// ============================================================================

/// R2 bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct R2Bucket {
    pub name: String,
    pub creation_date: Option<DateTime<Utc>>,
    pub location: Option<String>,
}

/// R2 object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct R2Object {
    pub key: String,
    pub size: i64,
    pub etag: Option<String>,
    pub http_metadata: Option<R2HttpMetadata>,
    pub uploaded: Option<DateTime<Utc>>,
}

/// R2 HTTP metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct R2HttpMetadata {
    pub content_type: Option<String>,
    pub content_language: Option<String>,
    pub content_disposition: Option<String>,
    pub content_encoding: Option<String>,
    pub cache_control: Option<String>,
    pub cache_expiry: Option<DateTime<Utc>>,
}

// ============================================================================
// D1 Database Types
// ============================================================================

/// D1 database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct D1Database {
    pub uuid: String,
    pub name: String,
    pub version: String,
    pub num_tables: Option<i32>,
    pub file_size: Option<i64>,
    pub created_at: Option<DateTime<Utc>>,
}

/// D1 query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct D1QueryResult {
    #[serde(default)]
    pub results: Vec<serde_json::Value>,
    pub success: bool,
    pub meta: Option<D1QueryMeta>,
}

/// D1 query metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct D1QueryMeta {
    pub duration: Option<f64>,
    pub changes: Option<i64>,
    pub last_row_id: Option<i64>,
    pub rows_read: Option<i64>,
    pub rows_written: Option<i64>,
}

// ============================================================================
// Stream Types
// ============================================================================

/// Stream video
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamVideo {
    pub uid: String,
    pub thumbnail: Option<String>,
    pub thumbnail_timestamp_pct: Option<f64>,
    pub ready_to_stream: bool,
    pub status: Option<StreamStatus>,
    pub meta: Option<serde_json::Value>,
    pub created: Option<DateTime<Utc>>,
    pub modified: Option<DateTime<Utc>>,
    pub uploaded: Option<DateTime<Utc>>,
    pub size: Option<i64>,
    pub preview: Option<String>,
    pub allow_download: Option<bool>,
    pub has_audio: Option<bool>,
    pub playback: Option<StreamPlayback>,
    pub input: Option<StreamInput>,
    pub duration: Option<f64>,
}

/// Stream status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStatus {
    pub state: String,
    pub pct_complete: Option<f64>,
    pub error_reason_code: Option<String>,
    pub error_reason_text: Option<String>,
}

/// Stream playback URLs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamPlayback {
    pub hls: Option<String>,
    pub dash: Option<String>,
}

/// Stream input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamInput {
    pub height: Option<i32>,
    pub width: Option<i32>,
}

/// Live input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveInput {
    pub uid: String,
    pub rtmps: Option<RtmpsInfo>,
    pub rtmps_playback: Option<RtmpsInfo>,
    pub srt: Option<SrtInfo>,
    pub srt_playback: Option<SrtInfo>,
    pub webrtc: Option<WebRtcInfo>,
    pub webrtc_playback: Option<WebRtcInfo>,
    pub created: Option<DateTime<Utc>>,
    pub modified: Option<DateTime<Utc>>,
    pub meta: Option<serde_json::Value>,
    pub status: Option<String>,
    pub recording: Option<LiveRecording>,
}

/// Stream statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStats {
    pub total_videos: usize,
    pub total_duration_seconds: u64,
    pub total_storage_bytes: u64,
    pub ready_videos: usize,
    pub processing_videos: usize,
    pub live_inputs: usize,
    pub active_live_streams: usize,
}

/// RTMPS info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtmpsInfo {
    pub url: String,
    pub stream_key: Option<String>,
}

/// SRT info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SrtInfo {
    pub url: String,
    pub stream_id: Option<String>,
    pub passphrase: Option<String>,
}

/// WebRTC info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRtcInfo {
    pub url: String,
}

/// Live input status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveInputStatus {
    pub current: Option<LiveInputCurrent>,
}

/// Live input current status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveInputCurrent {
    pub state: String,
    pub reason: Option<String>,
}

/// Live recording settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveRecording {
    pub mode: String,
    pub timeout_seconds: Option<i32>,
    pub require_signed_urls: Option<bool>,
}

/// Create live input request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLiveInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recording: Option<LiveRecording>,
}

// ============================================================================
// Analytics Types
// ============================================================================

/// Zone analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analytics {
    pub totals: Option<AnalyticsTotals>,
    pub timeseries: Option<Vec<AnalyticsTimeseries>>,
}

/// Analytics totals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsTotals {
    pub requests: Option<AnalyticsRequests>,
    pub bandwidth: Option<AnalyticsBandwidth>,
    pub threats: Option<AnalyticsThreats>,
    pub pageviews: Option<AnalyticsPageviews>,
    pub uniques: Option<AnalyticsUniques>,
}

/// Analytics requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsRequests {
    pub all: i64,
    pub cached: i64,
    pub uncached: i64,
    pub content_type: Option<serde_json::Value>,
    pub country: Option<serde_json::Value>,
    pub ssl: Option<AnalyticsSsl>,
    pub ssl_protocols: Option<serde_json::Value>,
    pub http_status: Option<serde_json::Value>,
}

/// Analytics bandwidth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsBandwidth {
    pub all: i64,
    pub cached: i64,
    pub uncached: i64,
    pub content_type: Option<serde_json::Value>,
    pub country: Option<serde_json::Value>,
    pub ssl: Option<AnalyticsSsl>,
}

/// Analytics SSL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSsl {
    pub encrypted: i64,
    pub unencrypted: i64,
}

/// Analytics threats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsThreats {
    pub all: i64,
    pub country: Option<serde_json::Value>,
    #[serde(rename = "type")]
    pub threat_type: Option<serde_json::Value>,
}

/// Analytics pageviews
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsPageviews {
    pub all: i64,
    pub search_engine: Option<serde_json::Value>,
}

/// Analytics uniques
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsUniques {
    pub all: i64,
}

/// Analytics timeseries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsTimeseries {
    pub since: DateTime<Utc>,
    pub until: DateTime<Utc>,
    pub requests: Option<AnalyticsRequests>,
    pub bandwidth: Option<AnalyticsBandwidth>,
    pub threats: Option<AnalyticsThreats>,
    pub pageviews: Option<AnalyticsPageviews>,
    pub uniques: Option<AnalyticsUniques>,
}
