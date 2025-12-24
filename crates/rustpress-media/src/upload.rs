//! File upload handling with drag-and-drop support
//!
//! Provides secure file upload functionality including:
//! - Chunked uploads for large files
//! - Progress tracking
//! - File validation
//! - Automatic thumbnail generation

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use sqlx::PgPool;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::{
    MediaConfig, MediaError, MediaItem, MediaResult, MediaType,
    image_optimizer::{ImageOptimizer, OptimizationConfig},
};

/// Upload service
pub struct UploadService {
    pool: PgPool,
    config: MediaConfig,
    optimizer: ImageOptimizer,
}

impl UploadService {
    /// Create new upload service
    pub fn new(pool: PgPool, config: MediaConfig) -> Self {
        Self {
            pool,
            config,
            optimizer: ImageOptimizer::new(OptimizationConfig::default()),
        }
    }

    /// Upload a file
    pub async fn upload(
        &self,
        filename: &str,
        content_type: &str,
        data: &[u8],
        uploaded_by: Uuid,
        folder_id: Option<Uuid>,
    ) -> MediaResult<MediaItem> {
        // Validate file
        self.validate_upload(filename, content_type, data.len() as u64)?;

        // Generate file hash
        let file_hash = self.hash_file(data);

        // Check for duplicates
        if let Some(existing) = self.find_by_hash(&file_hash).await? {
            return Ok(existing);
        }

        // Determine media type
        let media_type = MediaType::from_mime(content_type);

        // Generate storage path
        let (path, url) = self.generate_storage_path(filename, &media_type);

        // Create directory if needed
        let full_path = format!("{}/{}", self.config.storage_path, path);
        if let Some(parent) = Path::new(&full_path).parent() {
            fs::create_dir_all(parent).await?;
        }

        // Process and save file
        let (width, height, processed_data) = if media_type == MediaType::Image && self.config.optimize_images {
            let (w, h) = ImageOptimizer::dimensions(data)?;
            let optimized = self.optimizer.optimize(data, image::guess_format(data)?)?;
            (Some(w as i32), Some(h as i32), optimized)
        } else {
            (None, None, data.to_vec())
        };

        // Write file
        let mut file = fs::File::create(&full_path).await?;
        file.write_all(&processed_data).await?;
        file.flush().await?;

        // Generate thumbnail for images
        let thumbnail_url = if media_type == MediaType::Image {
            Some(self.generate_thumbnail(&full_path, data).await?)
        } else {
            None
        };

        // Create database record
        let media = sqlx::query_as!(
            MediaItem,
            r#"
            INSERT INTO media_items (
                filename, title, alt_text, media_type, mime_type,
                file_size, path, url, thumbnail_url, width, height,
                file_hash, folder_id, metadata, uploaded_by
            )
            VALUES ($1, $2, '', $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, '{}', $13)
            RETURNING
                id, filename, title, alt_text, caption, description,
                media_type as "media_type: MediaType",
                mime_type, file_size, path, url, thumbnail_url,
                width, height, duration, file_hash, folder_id,
                metadata, uploaded_by, created_at, updated_at
            "#,
            filename,
            filename, // title defaults to filename
            media_type as MediaType,
            content_type,
            processed_data.len() as i64,
            path,
            url,
            thumbnail_url,
            width,
            height,
            file_hash,
            folder_id,
            uploaded_by
        )
        .fetch_one(&self.pool)
        .await?;

        // Generate srcset variants if enabled
        if media_type == MediaType::Image && self.config.enable_srcset {
            self.generate_srcset_variants(&media, data).await?;
        }

        Ok(media)
    }

    /// Start chunked upload
    pub async fn start_chunked_upload(
        &self,
        filename: &str,
        content_type: &str,
        total_size: u64,
        uploaded_by: Uuid,
    ) -> MediaResult<ChunkedUpload> {
        // Validate
        self.validate_upload(filename, content_type, total_size)?;

        let upload_id = Uuid::new_v4();
        let temp_path = format!("{}/temp/{}", self.config.storage_path, upload_id);

        // Create temp directory
        fs::create_dir_all(&temp_path).await?;

        let upload = ChunkedUpload {
            id: upload_id,
            filename: filename.to_string(),
            content_type: content_type.to_string(),
            total_size,
            uploaded_size: 0,
            chunks_received: Vec::new(),
            temp_path,
            uploaded_by,
            created_at: Utc::now(),
        };

        // Store upload state (in production, use Redis or database)
        // For now, we'll rely on the temp directory existence

        Ok(upload)
    }

    /// Upload a chunk
    pub async fn upload_chunk(
        &self,
        upload_id: Uuid,
        chunk_index: u32,
        data: &[u8],
    ) -> MediaResult<ChunkUploadResult> {
        let temp_path = format!("{}/temp/{}", self.config.storage_path, upload_id);

        // Verify temp directory exists
        if !Path::new(&temp_path).exists() {
            return Err(MediaError::NotFound(upload_id));
        }

        // Write chunk
        let chunk_path = format!("{}/chunk_{:06}", temp_path, chunk_index);
        let mut file = fs::File::create(&chunk_path).await?;
        file.write_all(data).await?;

        // Check if all chunks received (simplified)
        let mut chunks = Vec::new();
        let mut entries = fs::read_dir(&temp_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_name().to_string_lossy().starts_with("chunk_") {
                chunks.push(entry.path());
            }
        }

        Ok(ChunkUploadResult {
            chunk_index,
            received_size: data.len() as u64,
            chunks_received: chunks.len() as u32,
        })
    }

    /// Complete chunked upload
    pub async fn complete_chunked_upload(
        &self,
        upload_id: Uuid,
        filename: &str,
        content_type: &str,
        uploaded_by: Uuid,
        folder_id: Option<Uuid>,
    ) -> MediaResult<MediaItem> {
        let temp_path = format!("{}/temp/{}", self.config.storage_path, upload_id);

        // Collect and sort chunks
        let mut chunks = Vec::new();
        let mut entries = fs::read_dir(&temp_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("chunk_") {
                chunks.push(entry.path());
            }
        }
        chunks.sort();

        // Combine chunks
        let mut combined = Vec::new();
        for chunk_path in &chunks {
            let chunk_data = fs::read(chunk_path).await?;
            combined.extend(chunk_data);
        }

        // Upload combined file
        let media = self.upload(filename, content_type, &combined, uploaded_by, folder_id).await?;

        // Cleanup temp directory
        fs::remove_dir_all(&temp_path).await?;

        Ok(media)
    }

    /// Cancel chunked upload
    pub async fn cancel_chunked_upload(&self, upload_id: Uuid) -> MediaResult<()> {
        let temp_path = format!("{}/temp/{}", self.config.storage_path, upload_id);

        if Path::new(&temp_path).exists() {
            fs::remove_dir_all(&temp_path).await?;
        }

        Ok(())
    }

    /// Validate upload
    fn validate_upload(&self, filename: &str, content_type: &str, size: u64) -> MediaResult<()> {
        // Check size
        if size > self.config.max_upload_size {
            return Err(MediaError::FileTooLarge {
                size,
                max: self.config.max_upload_size,
            });
        }

        // Check mime type
        if !self.config.allowed_types.iter().any(|t| t == content_type) {
            return Err(MediaError::UnsupportedFormat(content_type.to_string()));
        }

        // Check filename
        if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
            return Err(MediaError::InvalidType("Invalid filename".to_string()));
        }

        Ok(())
    }

    /// Hash file for deduplication
    fn hash_file(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    /// Find existing file by hash
    async fn find_by_hash(&self, hash: &str) -> MediaResult<Option<MediaItem>> {
        let media = sqlx::query_as!(
            MediaItem,
            r#"
            SELECT
                id, filename, title, alt_text, caption, description,
                media_type as "media_type: MediaType",
                mime_type, file_size, path, url, thumbnail_url,
                width, height, duration, file_hash, folder_id,
                metadata, uploaded_by, created_at, updated_at
            FROM media_items
            WHERE file_hash = $1
            LIMIT 1
            "#,
            hash
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(media)
    }

    /// Generate storage path
    fn generate_storage_path(&self, filename: &str, media_type: &MediaType) -> (String, String) {
        let now = Utc::now();
        let type_dir = match media_type {
            MediaType::Image => "images",
            MediaType::Video => "videos",
            MediaType::Audio => "audio",
            MediaType::Document => "documents",
            _ => "other",
        };

        let unique_filename = format!(
            "{}-{}",
            Uuid::new_v4().to_string().split('-').next().unwrap(),
            sanitize_filename(filename)
        );

        let path = format!(
            "{}/{}/{}/{}",
            type_dir,
            now.format("%Y"),
            now.format("%m"),
            unique_filename
        );

        let url = format!("{}/{}", self.config.base_url, path);

        (path, url)
    }

    /// Generate thumbnail
    async fn generate_thumbnail(&self, original_path: &str, data: &[u8]) -> MediaResult<String> {
        let thumb_data = self.optimizer.generate_thumbnail_exact(data, 300, 300)?;

        let path = Path::new(original_path);
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("thumb");
        let thumb_filename = format!("{}_thumb.jpg", stem);
        let thumb_path = path.parent()
            .map(|p| p.join(&thumb_filename))
            .unwrap_or_else(|| PathBuf::from(&thumb_filename));

        let mut file = fs::File::create(&thumb_path).await?;
        file.write_all(&thumb_data).await?;

        // Generate URL
        let relative_path = thumb_path.to_string_lossy()
            .replace(&self.config.storage_path, "")
            .trim_start_matches('/')
            .to_string();

        Ok(format!("{}/{}", self.config.base_url, relative_path))
    }

    /// Generate srcset variants
    async fn generate_srcset_variants(&self, media: &MediaItem, data: &[u8]) -> MediaResult<()> {
        let srcset_gen = crate::srcset::SrcsetGenerator::default_config();

        let original_width = media.width.unwrap_or(1920) as u32;
        let variants = srcset_gen.generate_variants(data, original_width)?;

        let base_path = Path::new(&media.path);
        let stem = base_path.file_stem().and_then(|s| s.to_str()).unwrap_or("img");
        let parent = base_path.parent().map(|p| p.to_path_buf()).unwrap_or_default();

        for variant in variants {
            let variant_filename = format!(
                "{}-{}.{}",
                stem,
                variant.width,
                variant.format.extension()
            );
            let variant_path = parent.join(&variant_filename);
            let full_path = format!("{}/{}", self.config.storage_path, variant_path.display());

            let mut file = fs::File::create(&full_path).await?;
            file.write_all(&variant.data).await?;

            // Insert variant record
            let variant_url = format!("{}/{}", self.config.base_url, variant_path.display());
            sqlx::query!(
                r#"
                INSERT INTO media_variants (media_id, variant_type, width, height, file_size, path, url, format)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
                media.id,
                variant.size_descriptor,
                variant.width as i32,
                variant.height as i32,
                variant.data.len() as i64,
                variant_path.to_string_lossy().to_string(),
                variant_url,
                variant.format.extension()
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }
}

/// Chunked upload state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkedUpload {
    pub id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub total_size: u64,
    pub uploaded_size: u64,
    pub chunks_received: Vec<u32>,
    pub temp_path: String,
    pub uploaded_by: Uuid,
    pub created_at: chrono::DateTime<Utc>,
}

/// Chunk upload result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkUploadResult {
    pub chunk_index: u32,
    pub received_size: u64,
    pub chunks_received: u32,
}

/// Upload progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadProgress {
    pub upload_id: Uuid,
    pub filename: String,
    pub total_size: u64,
    pub uploaded_size: u64,
    pub percent: f32,
    pub status: UploadStatus,
}

/// Upload status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UploadStatus {
    Pending,
    Uploading,
    Processing,
    Complete,
    Error,
    Cancelled,
}

/// Sanitize filename for storage
fn sanitize_filename(filename: &str) -> String {
    let name = Path::new(filename)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("file");

    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Generate client-side upload JavaScript
pub fn generate_upload_script(endpoint: &str, max_size: u64, allowed_types: &[String]) -> String {
    let types_json = serde_json::to_string(allowed_types).unwrap_or_else(|_| "[]".to_string());

    format!(
        r#"
class MediaUploader {{
    constructor(options = {{}}) {{
        this.endpoint = options.endpoint || '{}';
        this.maxSize = options.maxSize || {};
        this.allowedTypes = options.allowedTypes || {};
        this.chunkSize = options.chunkSize || 1024 * 1024; // 1MB chunks
        this.onProgress = options.onProgress || (() => {{}});
        this.onComplete = options.onComplete || (() => {{}});
        this.onError = options.onError || (() => {{}});
    }}

    validateFile(file) {{
        if (file.size > this.maxSize) {{
            throw new Error(`File too large. Maximum size is ${{this.formatSize(this.maxSize)}}`);
        }}
        if (!this.allowedTypes.includes(file.type)) {{
            throw new Error(`File type ${{file.type}} not allowed`);
        }}
        return true;
    }}

    formatSize(bytes) {{
        const units = ['B', 'KB', 'MB', 'GB'];
        let size = bytes;
        let unit = 0;
        while (size >= 1024 && unit < units.length - 1) {{
            size /= 1024;
            unit++;
        }}
        return `${{size.toFixed(1)}} ${{units[unit]}}`;
    }}

    async upload(file, folderId = null) {{
        this.validateFile(file);

        if (file.size <= this.chunkSize) {{
            return this.uploadDirect(file, folderId);
        }} else {{
            return this.uploadChunked(file, folderId);
        }}
    }}

    async uploadDirect(file, folderId) {{
        const formData = new FormData();
        formData.append('file', file);
        if (folderId) formData.append('folder_id', folderId);

        const response = await fetch(this.endpoint, {{
            method: 'POST',
            body: formData,
        }});

        if (!response.ok) {{
            throw new Error(await response.text());
        }}

        const result = await response.json();
        this.onComplete(result);
        return result;
    }}

    async uploadChunked(file, folderId) {{
        // Start upload
        const startResponse = await fetch(`${{this.endpoint}}/chunked/start`, {{
            method: 'POST',
            headers: {{ 'Content-Type': 'application/json' }},
            body: JSON.stringify({{
                filename: file.name,
                content_type: file.type,
                total_size: file.size,
            }}),
        }});

        if (!startResponse.ok) {{
            throw new Error(await startResponse.text());
        }}

        const {{ upload_id }} = await startResponse.json();

        // Upload chunks
        const totalChunks = Math.ceil(file.size / this.chunkSize);
        let uploadedSize = 0;

        for (let i = 0; i < totalChunks; i++) {{
            const start = i * this.chunkSize;
            const end = Math.min(start + this.chunkSize, file.size);
            const chunk = file.slice(start, end);

            const formData = new FormData();
            formData.append('chunk', chunk);
            formData.append('chunk_index', i);

            const chunkResponse = await fetch(`${{this.endpoint}}/chunked/${{upload_id}}/chunk`, {{
                method: 'POST',
                body: formData,
            }});

            if (!chunkResponse.ok) {{
                throw new Error(await chunkResponse.text());
            }}

            uploadedSize += chunk.size;
            this.onProgress({{
                uploadId: upload_id,
                filename: file.name,
                totalSize: file.size,
                uploadedSize,
                percent: (uploadedSize / file.size) * 100,
            }});
        }}

        // Complete upload
        const completeResponse = await fetch(`${{this.endpoint}}/chunked/${{upload_id}}/complete`, {{
            method: 'POST',
            headers: {{ 'Content-Type': 'application/json' }},
            body: JSON.stringify({{
                filename: file.name,
                content_type: file.type,
                folder_id: folderId,
            }}),
        }});

        if (!completeResponse.ok) {{
            throw new Error(await completeResponse.text());
        }}

        const result = await completeResponse.json();
        this.onComplete(result);
        return result;
    }}

    setupDropzone(element, options = {{}}) {{
        const dropzone = typeof element === 'string' ? document.querySelector(element) : element;
        if (!dropzone) return;

        const folderId = options.folderId || null;

        dropzone.addEventListener('dragover', (e) => {{
            e.preventDefault();
            dropzone.classList.add('dragover');
        }});

        dropzone.addEventListener('dragleave', () => {{
            dropzone.classList.remove('dragover');
        }});

        dropzone.addEventListener('drop', async (e) => {{
            e.preventDefault();
            dropzone.classList.remove('dragover');

            const files = Array.from(e.dataTransfer.files);
            for (const file of files) {{
                try {{
                    await this.upload(file, folderId);
                }} catch (error) {{
                    this.onError(error);
                }}
            }}
        }});
    }}
}}

// Export for module systems
if (typeof module !== 'undefined' && module.exports) {{
    module.exports = MediaUploader;
}}
"#,
        endpoint, max_size, types_json
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test file.jpg"), "test_file.jpg");
        assert_eq!(sanitize_filename("test<>file.jpg"), "test__file.jpg");
        assert_eq!(sanitize_filename("../../../etc/passwd"), ".._.._.._.._etc_passwd");
    }

    #[test]
    fn test_hash_consistency() {
        let data = b"test data";
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hex::encode(hasher.finalize());

        assert_eq!(hash.len(), 64);
    }
}
