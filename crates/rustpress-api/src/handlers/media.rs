//! Media handlers.
//!
//! This module provides API handlers for the enhanced media library
//! including folders, optimization, variants, and usage tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================
// MEDIA ITEM TYPES
// ============================================

/// Main media item response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    pub id: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub mime_type: String,
    pub file_size: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration: Option<i32>,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub alt_text: String,
    pub caption: String,
    pub folder_id: Option<Uuid>,
    pub folder_name: Option<String>,
    pub uploaded_by: Uuid,
    pub uploader_name: Option<String>,
    pub is_optimized: bool,
    pub original_size: Option<i64>,
    pub optimized_url: Option<String>,
    pub blurhash: Option<String>,
    pub dominant_color: Option<String>,
    pub focal_point_x: f64,
    pub focal_point_y: f64,
    pub tags: Vec<String>,
    pub is_favorite: bool,
    pub view_count: i32,
    pub download_count: i32,
    pub variants: Vec<MediaVariant>,
    pub usage_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub meta: serde_json::Value,
}

/// Media variant (responsive image sizes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaVariant {
    pub id: Uuid,
    pub variant_type: String,
    pub filename: String,
    pub url: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub file_size: Option<i64>,
    pub mime_type: String,
}

/// Thumbnail info (legacy support)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThumbnailInfo {
    pub size: String,
    pub url: String,
    pub width: i32,
    pub height: i32,
}

/// Uploader info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploaderInfo {
    pub id: Uuid,
    pub name: String,
    pub avatar_url: Option<String>,
}

// ============================================
// FOLDER TYPES
// ============================================

/// Media folder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaFolder {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub color: String,
    pub icon: String,
    pub item_count: i32,
    pub total_size: i64,
    pub is_system: bool,
    pub sort_order: i32,
    pub children: Vec<MediaFolder>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create folder request
#[derive(Debug, Deserialize)]
pub struct CreateFolderRequest {
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub color: Option<String>,
    pub icon: Option<String>,
}

/// Update folder request
#[derive(Debug, Deserialize)]
pub struct UpdateFolderRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub sort_order: Option<i32>,
}

// ============================================
// MEDIA REQUEST TYPES
// ============================================

/// Media list query parameters
#[derive(Debug, Deserialize)]
pub struct MediaListQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub folder_id: Option<Uuid>,
    pub mime_type: Option<String>,
    pub media_type: Option<MediaType>,
    pub uploader_id: Option<Uuid>,
    pub search: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_favorite: Option<bool>,
    pub is_optimized: Option<bool>,
    pub order_by: Option<MediaSortField>,
    pub order_dir: Option<SortDirection>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub min_size: Option<i64>,
    pub max_size: Option<i64>,
}

/// Media type filter
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    All,
    Image,
    Video,
    Audio,
    Document,
    Archive,
}

/// Sort field for media
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaSortField {
    CreatedAt,
    UpdatedAt,
    Filename,
    FileSize,
    ViewCount,
    DownloadCount,
}

/// Sort direction
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    Asc,
    Desc,
}

/// Update media request
#[derive(Debug, Deserialize)]
pub struct UpdateMediaRequest {
    pub title: Option<String>,
    pub alt_text: Option<String>,
    pub caption: Option<String>,
    pub description: Option<String>,
    pub folder_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
    pub focal_point_x: Option<f64>,
    pub focal_point_y: Option<f64>,
    pub is_favorite: Option<bool>,
    pub meta: Option<serde_json::Value>,
}

/// Upload request with options
#[derive(Debug, Deserialize)]
pub struct UploadOptions {
    pub folder_id: Option<Uuid>,
    pub alt_text: Option<String>,
    pub caption: Option<String>,
    pub tags: Option<Vec<String>>,
    pub auto_optimize: Option<bool>,
    pub generate_variants: Option<bool>,
}

/// Upload response
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub id: Uuid,
    pub url: String,
    pub filename: String,
    pub mime_type: String,
    pub size: i64,
    pub thumbnail_url: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

/// Bulk upload response
#[derive(Debug, Serialize)]
pub struct BulkUploadResponse {
    pub uploaded: Vec<UploadResponse>,
    pub failed: Vec<UploadError>,
    pub total_uploaded: usize,
    pub total_failed: usize,
}

/// Upload error
#[derive(Debug, Serialize)]
pub struct UploadError {
    pub filename: String,
    pub error: String,
}

// ============================================
// OPTIMIZATION TYPES
// ============================================

/// Optimization options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizeOptions {
    pub quality: Option<u8>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub convert_to: Option<String>,
    pub strip_metadata: Option<bool>,
    pub generate_blurhash: Option<bool>,
    pub generate_variants: Option<bool>,
}

/// Optimization result
#[derive(Debug, Serialize)]
pub struct OptimizationResult {
    pub media_id: Uuid,
    pub original_size: i64,
    pub optimized_size: i64,
    pub savings_percent: f64,
    pub optimized_url: String,
    pub variants_generated: Vec<String>,
    pub blurhash: Option<String>,
    pub dominant_color: Option<String>,
}

/// Bulk optimization request
#[derive(Debug, Deserialize)]
pub struct BulkOptimizeRequest {
    pub media_ids: Vec<Uuid>,
    pub options: OptimizeOptions,
}

/// Bulk optimization response
#[derive(Debug, Serialize)]
pub struct BulkOptimizationResponse {
    pub optimized: Vec<OptimizationResult>,
    pub failed: Vec<OptimizationError>,
    pub total_savings: i64,
}

/// Optimization error
#[derive(Debug, Serialize)]
pub struct OptimizationError {
    pub media_id: Uuid,
    pub error: String,
}

// ============================================
// USAGE TRACKING TYPES
// ============================================

/// Media usage record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaUsage {
    pub id: Uuid,
    pub media_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub entity_title: Option<String>,
    pub context: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Track usage request
#[derive(Debug, Deserialize)]
pub struct TrackUsageRequest {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub context: Option<String>,
}

// ============================================
// BULK OPERATIONS TYPES
// ============================================

/// Bulk move request
#[derive(Debug, Deserialize)]
pub struct BulkMoveRequest {
    pub media_ids: Vec<Uuid>,
    pub folder_id: Option<Uuid>,
}

/// Bulk delete request
#[derive(Debug, Deserialize)]
pub struct BulkDeleteRequest {
    pub media_ids: Vec<Uuid>,
    pub permanent: Option<bool>,
}

/// Bulk delete response
#[derive(Debug, Serialize)]
pub struct BulkDeleteResponse {
    pub deleted: Vec<Uuid>,
    pub failed: Vec<BulkDeleteError>,
    pub total_deleted: usize,
}

/// Bulk delete error
#[derive(Debug, Serialize)]
pub struct BulkDeleteError {
    pub media_id: Uuid,
    pub error: String,
}

/// Bulk tag request
#[derive(Debug, Deserialize)]
pub struct BulkTagRequest {
    pub media_ids: Vec<Uuid>,
    pub add_tags: Option<Vec<String>>,
    pub remove_tags: Option<Vec<String>>,
}

// ============================================
// LIBRARY RESPONSE TYPES
// ============================================

/// Full library response for modal
#[derive(Debug, Serialize)]
pub struct MediaLibraryResponse {
    pub media: PaginatedMediaResponse,
    pub folders: Vec<MediaFolder>,
    pub stats: MediaStats,
    pub user_preferences: Option<UserMediaPreferences>,
}

/// Paginated media response
#[derive(Debug, Serialize)]
pub struct PaginatedMediaResponse {
    pub items: Vec<MediaItem>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

/// Media statistics
#[derive(Debug, Serialize)]
pub struct MediaStats {
    pub total_items: i64,
    pub total_size: i64,
    pub images_count: i64,
    pub videos_count: i64,
    pub audio_count: i64,
    pub documents_count: i64,
    pub optimized_count: i64,
    pub total_savings: i64,
    pub favorites_count: i64,
}

/// User media preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMediaPreferences {
    pub default_folder_id: Option<Uuid>,
    pub view_mode: String,
    pub sort_by: String,
    pub sort_order: String,
    pub items_per_page: i32,
    pub auto_optimize: bool,
    pub default_quality: i32,
    pub generate_thumbnails: bool,
    pub generate_webp: bool,
    pub max_upload_size: i64,
    pub allowed_types: Vec<String>,
}

/// Update preferences request
#[derive(Debug, Deserialize)]
pub struct UpdatePreferencesRequest {
    pub default_folder_id: Option<Uuid>,
    pub view_mode: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub items_per_page: Option<i32>,
    pub auto_optimize: Option<bool>,
    pub default_quality: Option<i32>,
    pub generate_thumbnails: Option<bool>,
    pub generate_webp: Option<bool>,
    pub max_upload_size: Option<i64>,
    pub allowed_types: Option<Vec<String>>,
}

// ============================================
// SEARCH AND FILTER TYPES
// ============================================

/// Advanced search request
#[derive(Debug, Deserialize)]
pub struct MediaSearchRequest {
    pub query: String,
    pub search_in: Option<Vec<SearchField>>,
    pub filters: Option<MediaFilters>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

/// Fields to search in
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchField {
    Filename,
    AltText,
    Caption,
    Tags,
    Meta,
}

/// Advanced filters
#[derive(Debug, Deserialize)]
pub struct MediaFilters {
    pub media_type: Option<MediaType>,
    pub folder_ids: Option<Vec<Uuid>>,
    pub uploader_ids: Option<Vec<Uuid>>,
    pub tags: Option<Vec<String>>,
    pub date_range: Option<DateRange>,
    pub size_range: Option<SizeRange>,
    pub dimensions: Option<DimensionFilter>,
    pub is_optimized: Option<bool>,
    pub is_favorite: Option<bool>,
}

/// Date range filter
#[derive(Debug, Deserialize)]
pub struct DateRange {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

/// Size range filter
#[derive(Debug, Deserialize)]
pub struct SizeRange {
    pub min: Option<i64>,
    pub max: Option<i64>,
}

/// Dimension filter
#[derive(Debug, Deserialize)]
pub struct DimensionFilter {
    pub min_width: Option<i32>,
    pub max_width: Option<i32>,
    pub min_height: Option<i32>,
    pub max_height: Option<i32>,
    pub aspect_ratio: Option<String>,
}

// ============================================
// IMAGE EDITING TYPES
// ============================================

/// Image edit request
#[derive(Debug, Deserialize)]
pub struct ImageEditRequest {
    pub crop: Option<CropParams>,
    pub resize: Option<ResizeParams>,
    pub rotate: Option<i32>,
    pub flip_horizontal: Option<bool>,
    pub flip_vertical: Option<bool>,
    pub filters: Option<ImageFilters>,
    pub save_as_new: Option<bool>,
}

/// Crop parameters
#[derive(Debug, Deserialize)]
pub struct CropParams {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Resize parameters
#[derive(Debug, Deserialize)]
pub struct ResizeParams {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub maintain_aspect_ratio: Option<bool>,
    pub fit: Option<ResizeFit>,
}

/// Resize fit mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResizeFit {
    Cover,
    Contain,
    Fill,
    Inside,
    Outside,
}

/// Image filters
#[derive(Debug, Deserialize)]
pub struct ImageFilters {
    pub brightness: Option<f32>,
    pub contrast: Option<f32>,
    pub saturation: Option<f32>,
    pub blur: Option<f32>,
    pub sharpen: Option<f32>,
    pub grayscale: Option<bool>,
    pub sepia: Option<bool>,
}

// ============================================
// ANALYTICS TYPES
// ============================================

/// Media analytics
#[derive(Debug, Serialize)]
pub struct MediaAnalytics {
    pub media_id: Uuid,
    pub total_views: i64,
    pub total_downloads: i64,
    pub views_by_date: Vec<DateCount>,
    pub downloads_by_date: Vec<DateCount>,
    pub usage_locations: Vec<MediaUsage>,
    pub referrers: Vec<ReferrerStat>,
}

/// Date count for analytics
#[derive(Debug, Serialize)]
pub struct DateCount {
    pub date: String,
    pub count: i64,
}

/// Referrer statistics
#[derive(Debug, Serialize)]
pub struct ReferrerStat {
    pub referrer: String,
    pub count: i64,
}

// ============================================
// LEGACY TYPES (for backward compatibility)
// ============================================

/// Legacy media response
#[derive(Debug, Serialize)]
pub struct MediaResponse {
    pub id: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub mime_type: String,
    pub size: i64,
    pub url: String,
    pub title: Option<String>,
    pub alt_text: Option<String>,
    pub caption: Option<String>,
    pub description: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub thumbnails: Vec<ThumbnailInfo>,
    pub uploader: UploaderInfo,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<MediaItem> for MediaResponse {
    fn from(item: MediaItem) -> Self {
        let title = item.original_filename.clone();
        Self {
            id: item.id,
            filename: item.filename,
            original_filename: item.original_filename,
            mime_type: item.mime_type,
            size: item.file_size,
            url: item.url,
            title: Some(title),
            alt_text: Some(item.alt_text),
            caption: Some(item.caption),
            description: None,
            width: item.width,
            height: item.height,
            thumbnails: item.variants.iter().map(|v| ThumbnailInfo {
                size: v.variant_type.clone(),
                url: v.url.clone(),
                width: v.width.unwrap_or(0),
                height: v.height.unwrap_or(0),
            }).collect(),
            uploader: UploaderInfo {
                id: item.uploaded_by,
                name: item.uploader_name.unwrap_or_default(),
                avatar_url: None,
            },
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}
