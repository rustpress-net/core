//! Media service for handling media-related business logic.

use chrono::{DateTime, Utc};
use rustpress_core::error::{Error, Result};
use rustpress_core::service::SortOrder;
use rustpress_database::models::MediaRow;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Media types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Image,
    Video,
    Audio,
    Document,
    Archive,
    Other,
}

impl MediaType {
    pub fn from_mime(mime: &str) -> Self {
        if mime.starts_with("image/") {
            Self::Image
        } else if mime.starts_with("video/") {
            Self::Video
        } else if mime.starts_with("audio/") {
            Self::Audio
        } else if mime == "application/pdf"
            || mime.contains("document")
            || mime.contains("text/")
            || mime.contains("spreadsheet")
            || mime.contains("presentation")
        {
            Self::Document
        } else if mime.contains("zip")
            || mime.contains("rar")
            || mime.contains("tar")
            || mime.contains("gzip")
        {
            Self::Archive
        } else {
            Self::Other
        }
    }
}

/// Media response for API
#[derive(Debug, Clone, Serialize)]
pub struct MediaResponse {
    pub id: Uuid,
    pub uploader_id: Option<Uuid>,
    pub filename: String,
    pub original_filename: String,
    pub mime_type: String,
    pub media_type: String,
    pub file_size: i64,
    pub storage_path: String,
    pub url: Option<String>,
    pub alt_text: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration: Option<i32>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Paginated media response
#[derive(Debug, Clone, Serialize)]
pub struct MediaListResponse {
    pub items: Vec<MediaResponse>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

/// Upload media request (metadata)
#[derive(Debug, Clone, Deserialize)]
pub struct UploadMediaMetadata {
    pub alt_text: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
}

/// Update media request
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateMediaRequest {
    pub alt_text: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
}

/// Media list query parameters
#[derive(Debug, Clone, Deserialize, Default)]
pub struct MediaListParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub media_type: Option<String>,
    pub mime_type: Option<String>,
    pub uploader_id: Option<Uuid>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

impl From<MediaRow> for MediaResponse {
    fn from(row: MediaRow) -> Self {
        let media_type = MediaType::from_mime(&row.mime_type);
        Self {
            id: row.id,
            uploader_id: row.uploader_id,
            filename: row.filename,
            original_filename: row.original_filename,
            mime_type: row.mime_type,
            media_type: format!("{:?}", media_type).to_lowercase(),
            file_size: row.file_size,
            storage_path: row.storage_path.clone(),
            url: Some(format!("/uploads/{}", row.storage_path)),
            alt_text: row.alt_text,
            title: row.title,
            description: row.description,
            width: row.width,
            height: row.height,
            duration: row.duration,
            metadata: row.metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// Media service for handling media operations
#[derive(Clone)]
pub struct MediaService {
    pool: PgPool,
    tenant_id: Option<Uuid>,
    base_url: String,
}

impl MediaService {
    /// Create a new media service
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            tenant_id: None,
            base_url: String::new(),
        }
    }

    /// Set the tenant ID for multi-site support
    pub fn with_tenant(mut self, tenant_id: Uuid) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    /// Set the base URL for media URLs
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    /// Site condition for queries
    fn site_condition(&self) -> String {
        match self.tenant_id {
            Some(id) => format!("site_id = '{}'", id),
            None => "site_id IS NULL".to_string(),
        }
    }

    /// Upload a new media file
    pub async fn upload_media(
        &self,
        uploader_id: Uuid,
        filename: String,
        original_filename: String,
        mime_type: String,
        file_size: i64,
        storage_path: String,
        metadata: Option<UploadMediaMetadata>,
        dimensions: Option<(i32, i32)>,
        duration: Option<i32>,
    ) -> Result<MediaResponse> {
        let now = Utc::now();
        let media_metadata = metadata.unwrap_or(UploadMediaMetadata {
            alt_text: None,
            title: None,
            description: None,
        });

        let media = MediaRow {
            id: Uuid::now_v7(),
            tenant_id: self.tenant_id,
            uploader_id: Some(uploader_id),
            filename,
            original_filename,
            mime_type,
            file_size,
            storage_path,
            storage_backend: Some("local".to_string()),
            alt_text: media_metadata.alt_text,
            title: media_metadata.title,
            description: media_metadata.description,
            width: dimensions.map(|(w, _)| w),
            height: dimensions.map(|(_, h)| h),
            duration,
            metadata: serde_json::json!({}),
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created = self.create(&media).await?;
        Ok(MediaResponse::from(created))
    }

    /// List media with pagination and filtering
    pub async fn list_media(&self, params: MediaListParams) -> Result<MediaListResponse> {
        let page = params.page.unwrap_or(1).max(1);
        let per_page = params.per_page.unwrap_or(20).min(100).max(1);

        let sort_order = match params.sort_order.as_deref() {
            Some("asc") => SortOrder::Asc,
            _ => SortOrder::Desc,
        };

        let mut conditions = vec![self.site_condition()];
        conditions.push("deleted_at IS NULL".to_string());

        if let Some(ref mime_type) = params.mime_type {
            conditions.push(format!("mime_type LIKE '{}%'", mime_type.replace('\'', "''")));
        }

        if let Some(uploader_id) = params.uploader_id {
            conditions.push(format!("uploader_id = '{}'", uploader_id));
        }

        if let Some(ref search) = params.search {
            let escaped = search.replace('\'', "''");
            conditions.push(format!(
                "(filename ILIKE '%{}%' OR original_filename ILIKE '%{}%' OR title ILIKE '%{}%' OR alt_text ILIKE '%{}%')",
                escaped, escaped, escaped, escaped
            ));
        }

        let where_clause = conditions.join(" AND ");
        let order_by = params.sort_by.as_deref().unwrap_or("created_at");
        let order_dir = if sort_order == SortOrder::Desc { "DESC" } else { "ASC" };

        // Count query
        let count_query = format!(
            "SELECT COUNT(*) as count FROM media WHERE {}",
            where_clause
        );
        let total: (i64,) = sqlx::query_as(&count_query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to count media", e))?;

        // Data query
        let offset = (page - 1) * per_page;
        let data_query = format!(
            "SELECT * FROM media WHERE {} ORDER BY {} {} LIMIT {} OFFSET {}",
            where_clause, order_by, order_dir, per_page, offset
        );

        let rows: Vec<MediaRow> = sqlx::query_as(&data_query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to list media", e))?;

        let items: Vec<MediaResponse> = rows.into_iter().map(MediaResponse::from).collect();
        let total_pages = ((total.0 as f64) / (per_page as f64)).ceil() as u64;

        Ok(MediaListResponse {
            items,
            total: total.0 as u64,
            page: page.into(),
            per_page: per_page.into(),
            total_pages,
        })
    }

    /// Get a media item by ID
    pub async fn get_media(&self, id: Uuid) -> Result<Option<MediaResponse>> {
        let media = self.find_by_id(id).await?;
        Ok(media.map(MediaResponse::from))
    }

    /// Update media metadata
    pub async fn update_media(&self, id: Uuid, request: UpdateMediaRequest) -> Result<MediaResponse> {
        let existing = self.find_by_id(id).await?
            .ok_or_else(|| Error::not_found("Media", id.to_string()))?;

        let query = r#"
            UPDATE media SET
                alt_text = $2,
                title = $3,
                description = $4,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
        "#;

        let updated: MediaRow = sqlx::query_as(query)
            .bind(id)
            .bind(request.alt_text.or(existing.alt_text))
            .bind(request.title.or(existing.title))
            .bind(request.description.or(existing.description))
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to update media", e))?;

        Ok(MediaResponse::from(updated))
    }

    /// Delete a media item (soft delete)
    pub async fn delete_media(&self, id: Uuid) -> Result<bool> {
        let _ = self.find_by_id(id).await?
            .ok_or_else(|| Error::not_found("Media", id.to_string()))?;

        sqlx::query("UPDATE media SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to delete media", e))?;

        Ok(true)
    }

    /// Get media by storage path
    pub async fn get_by_path(&self, path: &str) -> Result<Option<MediaResponse>> {
        let query = format!(
            "SELECT * FROM media WHERE storage_path = $1 AND {} AND deleted_at IS NULL",
            self.site_condition()
        );

        let media: Option<MediaRow> = sqlx::query_as(&query)
            .bind(path)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to find media by path", e))?;

        Ok(media.map(MediaResponse::from))
    }

    /// Get storage stats
    pub async fn get_storage_stats(&self) -> Result<StorageStats> {
        let query = format!(
            r#"
            SELECT
                COUNT(*) as file_count,
                COALESCE(SUM(file_size), 0) as total_size,
                COUNT(DISTINCT mime_type) as mime_type_count
            FROM media
            WHERE {} AND deleted_at IS NULL
            "#,
            self.site_condition()
        );

        let stats: (i64, i64, i64) = sqlx::query_as(&query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to get storage stats", e))?;

        // Get breakdown by mime type
        let breakdown_query = format!(
            r#"
            SELECT
                CASE
                    WHEN mime_type LIKE 'image/%' THEN 'image'
                    WHEN mime_type LIKE 'video/%' THEN 'video'
                    WHEN mime_type LIKE 'audio/%' THEN 'audio'
                    ELSE 'other'
                END as media_type,
                COUNT(*) as count,
                COALESCE(SUM(file_size), 0) as size
            FROM media
            WHERE {} AND deleted_at IS NULL
            GROUP BY 1
            "#,
            self.site_condition()
        );

        let breakdown: Vec<(String, i64, i64)> = sqlx::query_as(&breakdown_query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to get storage breakdown", e))?;

        Ok(StorageStats {
            total_files: stats.0 as u64,
            total_size: stats.1 as u64,
            breakdown: breakdown.into_iter().map(|(t, c, s)| MediaTypeStats {
                media_type: t,
                count: c as u64,
                size: s as u64,
            }).collect(),
        })
    }

    // =====================
    // Database operations
    // =====================

    async fn find_by_id(&self, id: Uuid) -> Result<Option<MediaRow>> {
        let query = format!(
            "SELECT * FROM media WHERE id = $1 AND {} AND deleted_at IS NULL",
            self.site_condition()
        );

        sqlx::query_as::<_, MediaRow>(&query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to find media", e))
    }

    async fn create(&self, media: &MediaRow) -> Result<MediaRow> {
        sqlx::query_as::<_, MediaRow>(
            r#"
            INSERT INTO media (id, site_id, uploader_id, filename, original_filename, mime_type, file_size, storage_path, storage_backend, alt_text, title, description, width, height, duration, metadata, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            RETURNING *
            "#,
        )
        .bind(media.id)
        .bind(media.tenant_id)
        .bind(media.uploader_id)
        .bind(&media.filename)
        .bind(&media.original_filename)
        .bind(&media.mime_type)
        .bind(media.file_size)
        .bind(&media.storage_path)
        .bind(&media.storage_backend)
        .bind(&media.alt_text)
        .bind(&media.title)
        .bind(&media.description)
        .bind(media.width)
        .bind(media.height)
        .bind(media.duration)
        .bind(&media.metadata)
        .bind(media.created_at)
        .bind(media.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to create media", e))
    }
}

/// Storage statistics
#[derive(Debug, Clone, Serialize)]
pub struct StorageStats {
    pub total_files: u64,
    pub total_size: u64,
    pub breakdown: Vec<MediaTypeStats>,
}

/// Media type statistics
#[derive(Debug, Clone, Serialize)]
pub struct MediaTypeStats {
    pub media_type: String,
    pub count: u64,
    pub size: u64,
}

impl Default for MediaService {
    fn default() -> Self {
        panic!("MediaService requires a database pool")
    }
}

/// Allowed file extensions
pub fn get_allowed_extensions() -> Vec<&'static str> {
    vec![
        // Images
        "jpg", "jpeg", "png", "gif", "webp", "svg", "ico", "bmp",
        // Videos
        "mp4", "webm", "ogg", "avi", "mov", "wmv",
        // Audio
        "mp3", "wav", "ogg", "flac", "aac",
        // Documents
        "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt", "csv",
        // Archives
        "zip", "rar", "tar", "gz", "7z",
    ]
}

/// Get max file size in bytes (default: 50MB)
pub fn get_max_file_size() -> u64 {
    50 * 1024 * 1024
}

/// Validate file for upload
pub fn validate_upload(filename: &str, file_size: u64) -> Result<()> {
    // Check file size
    if file_size > get_max_file_size() {
        return Err(Error::validation("File size exceeds maximum allowed"));
    }

    // Check extension
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    if !get_allowed_extensions().contains(&ext.as_str()) {
        return Err(Error::validation("File type not allowed"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_type_from_mime() {
        assert_eq!(MediaType::from_mime("image/png"), MediaType::Image);
        assert_eq!(MediaType::from_mime("video/mp4"), MediaType::Video);
        assert_eq!(MediaType::from_mime("audio/mpeg"), MediaType::Audio);
        assert_eq!(MediaType::from_mime("application/pdf"), MediaType::Document);
        assert_eq!(MediaType::from_mime("application/zip"), MediaType::Archive);
        assert_eq!(MediaType::from_mime("application/octet-stream"), MediaType::Other);
    }

    #[test]
    fn test_validate_upload() {
        assert!(validate_upload("test.jpg", 1000).is_ok());
        assert!(validate_upload("test.exe", 1000).is_err());
        assert!(validate_upload("test.jpg", 100 * 1024 * 1024).is_err());
    }
}
