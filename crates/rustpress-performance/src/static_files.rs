//! Static File Serving with Cache Busting
//!
//! Efficient static file serving with content hashing, compression, and cache busting.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use axum::{
    body::Body,
    http::{header, HeaderMap, HeaderValue, Request, Response, StatusCode},
    response::IntoResponse,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::fs;
use tokio::io::AsyncReadExt;

/// Static file serving errors
#[derive(Debug, Error)]
pub enum StaticFileError {
    #[error("File not found: {0}")]
    NotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Compression error: {0}")]
    Compression(String),
}

/// Asset manifest for cache busting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetManifest {
    /// Original path -> hashed path mapping
    pub assets: HashMap<String, HashedAsset>,
    /// Build timestamp
    pub build_time: i64,
    /// Version
    pub version: String,
}

/// Hashed asset entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashedAsset {
    /// Hashed filename
    pub hashed_path: String,
    /// Content hash
    pub hash: String,
    /// Original file size
    pub size: u64,
    /// GZIP compressed size (if available)
    pub gzip_size: Option<u64>,
    /// Brotli compressed size (if available)
    pub brotli_size: Option<u64>,
    /// MIME type
    pub mime_type: String,
    /// Integrity hash (SRI)
    pub integrity: String,
}

impl AssetManifest {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            build_time: chrono::Utc::now().timestamp(),
            version: "1.0.0".to_string(),
        }
    }

    /// Load manifest from file
    pub async fn load(path: &Path) -> Result<Self, StaticFileError> {
        let content = fs::read_to_string(path).await?;
        serde_json::from_str(&content)
            .map_err(|e| StaticFileError::InvalidPath(e.to_string()))
    }

    /// Save manifest to file
    pub async fn save(&self, path: &Path) -> Result<(), StaticFileError> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| StaticFileError::InvalidPath(e.to_string()))?;
        fs::write(path, content).await?;
        Ok(())
    }

    /// Get hashed URL for asset
    pub fn get_url(&self, original_path: &str) -> Option<String> {
        self.assets.get(original_path).map(|a| a.hashed_path.clone())
    }

    /// Get integrity hash for asset (for SRI)
    pub fn get_integrity(&self, original_path: &str) -> Option<String> {
        self.assets.get(original_path).map(|a| a.integrity.clone())
    }
}

impl Default for AssetManifest {
    fn default() -> Self {
        Self::new()
    }
}

/// Static file server configuration
#[derive(Debug, Clone)]
pub struct StaticFileConfig {
    /// Root directory for static files
    pub root_dir: PathBuf,
    /// URL prefix (e.g., "/static")
    pub url_prefix: String,
    /// Enable compression
    pub enable_compression: bool,
    /// Compression level (1-9)
    pub compression_level: u32,
    /// Minimum size for compression
    pub compression_threshold: usize,
    /// Enable cache busting via hash
    pub cache_busting: bool,
    /// Default max-age for assets
    pub max_age: Duration,
    /// Enable immutable for hashed assets
    pub immutable_hashed: bool,
    /// Enable precompression
    pub precompress: bool,
    /// MIME type overrides
    pub mime_overrides: HashMap<String, String>,
}

impl Default for StaticFileConfig {
    fn default() -> Self {
        Self {
            root_dir: PathBuf::from("static"),
            url_prefix: "/static".to_string(),
            enable_compression: true,
            compression_level: 6,
            compression_threshold: 1024,
            cache_busting: true,
            max_age: Duration::from_secs(31536000), // 1 year
            immutable_hashed: true,
            precompress: true,
            mime_overrides: HashMap::new(),
        }
    }
}

/// Cached file entry
struct CachedFile {
    content: Vec<u8>,
    gzip_content: Option<Vec<u8>>,
    brotli_content: Option<Vec<u8>>,
    mime_type: String,
    etag: String,
    last_modified: i64,
    size: u64,
}

/// Static file server
pub struct StaticFileServer {
    config: StaticFileConfig,
    manifest: Arc<RwLock<AssetManifest>>,
    file_cache: Arc<RwLock<HashMap<String, CachedFile>>>,
}

impl StaticFileServer {
    pub fn new(config: StaticFileConfig) -> Self {
        Self {
            config,
            manifest: Arc::new(RwLock::new(AssetManifest::new())),
            file_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load asset manifest
    pub async fn load_manifest(&self, path: &Path) -> Result<(), StaticFileError> {
        let manifest = AssetManifest::load(path).await?;
        *self.manifest.write() = manifest;
        Ok(())
    }

    /// Build asset manifest from directory
    pub async fn build_manifest(&self) -> Result<AssetManifest, StaticFileError> {
        let mut manifest = AssetManifest::new();

        self.scan_directory(&self.config.root_dir, &mut manifest).await?;

        *self.manifest.write() = manifest.clone();
        Ok(manifest)
    }

    async fn scan_directory(&self, dir: &Path, manifest: &mut AssetManifest) -> Result<(), StaticFileError> {
        let mut entries = fs::read_dir(dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.is_dir() {
                Box::pin(self.scan_directory(&path, manifest)).await?;
            } else if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if is_static_asset(&ext_str) {
                        self.process_asset(&path, manifest).await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn process_asset(&self, path: &Path, manifest: &mut AssetManifest) -> Result<(), StaticFileError> {
        let content = fs::read(path).await?;
        let hash = blake3::hash(&content);
        let hash_str = &hash.to_hex()[..8];

        // Generate hashed filename
        let file_stem = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
        let extension = path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let hashed_filename = format!("{}.{}.{}", file_stem, hash_str, extension);

        // Calculate relative paths
        let relative_path = path.strip_prefix(&self.config.root_dir)
            .map_err(|_| StaticFileError::InvalidPath(path.display().to_string()))?;

        let original_url = format!(
            "{}/{}",
            self.config.url_prefix,
            relative_path.display()
        );

        let hashed_dir = relative_path.parent().unwrap_or(Path::new(""));
        let hashed_url = format!(
            "{}/{}/{}",
            self.config.url_prefix,
            hashed_dir.display(),
            hashed_filename
        );

        // MIME type
        let mime_type = get_mime_type(extension, &self.config.mime_overrides);

        // Compress if enabled
        let gzip_size = if self.config.precompress && content.len() >= self.config.compression_threshold {
            let gzip_content = compress_gzip(&content, self.config.compression_level)?;
            let gzip_path = path.with_extension(format!("{}.gz", extension));
            fs::write(&gzip_path, &gzip_content).await?;
            Some(gzip_content.len() as u64)
        } else {
            None
        };

        let brotli_size = if self.config.precompress && content.len() >= self.config.compression_threshold {
            let brotli_content = compress_brotli(&content, self.config.compression_level)?;
            let brotli_path = path.with_extension(format!("{}.br", extension));
            fs::write(&brotli_path, &brotli_content).await?;
            Some(brotli_content.len() as u64)
        } else {
            None
        };

        // Write hashed file
        if self.config.cache_busting {
            let hashed_path = path.parent()
                .unwrap_or(Path::new(""))
                .join(&hashed_filename);
            fs::copy(path, &hashed_path).await?;
        }

        // Integrity hash for SRI
        let integrity = format!("sha384-{}", base64_encode(&blake3::hash(&content).as_bytes()[..32]));

        manifest.assets.insert(original_url.clone(), HashedAsset {
            hashed_path: hashed_url,
            hash: hash_str.to_string(),
            size: content.len() as u64,
            gzip_size,
            brotli_size,
            mime_type,
            integrity,
        });

        Ok(())
    }

    /// Serve static file
    pub async fn serve(&self, path: &str, accept_encoding: Option<&str>) -> Result<StaticFileResponse, StaticFileError> {
        // Normalize path
        let normalized = normalize_path(path);

        // Security check
        if normalized.contains("..") {
            return Err(StaticFileError::InvalidPath("Path traversal not allowed".to_string()));
        }

        // Remove URL prefix
        let relative_path = normalized
            .strip_prefix(&self.config.url_prefix)
            .unwrap_or(&normalized)
            .trim_start_matches('/');

        // Check file cache first
        if let Some(cached) = self.file_cache.read().get(relative_path) {
            return self.build_response(cached, accept_encoding);
        }

        // Load file
        let file_path = self.config.root_dir.join(relative_path);

        if !file_path.exists() {
            return Err(StaticFileError::NotFound(path.to_string()));
        }

        let content = fs::read(&file_path).await?;
        let metadata = fs::metadata(&file_path).await?;

        let extension = file_path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let mime_type = get_mime_type(extension, &self.config.mime_overrides);
        let etag = format!("\"{}\"", &blake3::hash(&content).to_hex()[..16]);
        let last_modified = metadata.modified()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        // Compress if needed
        let gzip_content = if self.config.enable_compression && content.len() >= self.config.compression_threshold {
            // Try precompressed first
            let gzip_path = file_path.with_extension(format!("{}.gz", extension));
            if gzip_path.exists() {
                Some(fs::read(&gzip_path).await?)
            } else {
                compress_gzip(&content, self.config.compression_level).ok()
            }
        } else {
            None
        };

        let brotli_content = if self.config.enable_compression && content.len() >= self.config.compression_threshold {
            let brotli_path = file_path.with_extension(format!("{}.br", extension));
            if brotli_path.exists() {
                Some(fs::read(&brotli_path).await?)
            } else {
                compress_brotli(&content, self.config.compression_level).ok()
            }
        } else {
            None
        };

        let cached = CachedFile {
            content,
            gzip_content,
            brotli_content,
            mime_type,
            etag,
            last_modified,
            size: metadata.len(),
        };

        // Cache the file
        self.file_cache.write().insert(relative_path.to_string(), cached);

        // Get from cache and return
        let cached = self.file_cache.read().get(relative_path).cloned();
        if let Some(cached) = cached {
            self.build_response(&cached, accept_encoding)
        } else {
            Err(StaticFileError::NotFound(path.to_string()))
        }
    }

    fn build_response(&self, file: &CachedFile, accept_encoding: Option<&str>) -> Result<StaticFileResponse, StaticFileError> {
        let encodings = parse_accept_encoding(accept_encoding);

        // Choose best encoding
        let (content, encoding) = if encodings.contains(&"br") {
            if let Some(ref br) = file.brotli_content {
                (br.clone(), Some("br"))
            } else if encodings.contains(&"gzip") {
                if let Some(ref gz) = file.gzip_content {
                    (gz.clone(), Some("gzip"))
                } else {
                    (file.content.clone(), None)
                }
            } else {
                (file.content.clone(), None)
            }
        } else if encodings.contains(&"gzip") {
            if let Some(ref gz) = file.gzip_content {
                (gz.clone(), Some("gzip"))
            } else {
                (file.content.clone(), None)
            }
        } else {
            (file.content.clone(), None)
        };

        Ok(StaticFileResponse {
            content,
            mime_type: file.mime_type.clone(),
            etag: file.etag.clone(),
            last_modified: file.last_modified,
            max_age: self.config.max_age.as_secs(),
            immutable: self.config.immutable_hashed,
            encoding: encoding.map(|s| s.to_string()),
        })
    }

    /// Get URL for asset with cache busting
    pub fn asset_url(&self, path: &str) -> String {
        self.manifest.read()
            .get_url(path)
            .unwrap_or_else(|| path.to_string())
    }

    /// Get integrity hash for asset
    pub fn asset_integrity(&self, path: &str) -> Option<String> {
        self.manifest.read().get_integrity(path)
    }

    /// Clear file cache
    pub fn clear_cache(&self) {
        self.file_cache.write().clear();
    }

    /// Invalidate specific file
    pub fn invalidate(&self, path: &str) {
        self.file_cache.write().remove(path);
    }
}

/// Static file response
pub struct StaticFileResponse {
    pub content: Vec<u8>,
    pub mime_type: String,
    pub etag: String,
    pub last_modified: i64,
    pub max_age: u64,
    pub immutable: bool,
    pub encoding: Option<String>,
}

impl IntoResponse for StaticFileResponse {
    fn into_response(self) -> Response<Body> {
        let mut builder = Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, &self.mime_type)
            .header(header::ETAG, &self.etag)
            .header(header::CONTENT_LENGTH, self.content.len());

        // Cache-Control
        let cache_control = if self.immutable {
            format!("public, max-age={}, immutable", self.max_age)
        } else {
            format!("public, max-age={}", self.max_age)
        };
        builder = builder.header(header::CACHE_CONTROL, cache_control);

        // Last-Modified
        if self.last_modified > 0 {
            let datetime = httpdate::HttpDate::from(
                std::time::UNIX_EPOCH + Duration::from_secs(self.last_modified as u64)
            );
            builder = builder.header(header::LAST_MODIFIED, datetime.to_string());
        }

        // Content-Encoding
        if let Some(encoding) = &self.encoding {
            builder = builder.header(header::CONTENT_ENCODING, encoding.as_str());
        }

        builder.body(Body::from(self.content)).unwrap()
    }
}

/// Check if file extension is a static asset
fn is_static_asset(ext: &str) -> bool {
    matches!(
        ext,
        "css" | "js" | "mjs" | "json" |
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "ico" | "avif" |
        "woff" | "woff2" | "ttf" | "eot" | "otf" |
        "pdf" | "xml" | "txt" | "map"
    )
}

/// Get MIME type for extension
fn get_mime_type(ext: &str, overrides: &HashMap<String, String>) -> String {
    if let Some(mime) = overrides.get(ext) {
        return mime.clone();
    }

    match ext.to_lowercase().as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "xml" => "application/xml; charset=utf-8",
        "txt" => "text/plain; charset=utf-8",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "eot" => "application/vnd.ms-fontobject",
        "otf" => "font/otf",
        "pdf" => "application/pdf",
        "map" => "application/json",
        _ => "application/octet-stream",
    }.to_string()
}

/// Normalize URL path
fn normalize_path(path: &str) -> String {
    path.replace("//", "/")
        .trim_end_matches('/')
        .to_string()
}

/// Parse Accept-Encoding header
fn parse_accept_encoding(header: Option<&str>) -> Vec<&str> {
    header
        .map(|h| {
            h.split(',')
                .map(|s| s.split(';').next().unwrap_or("").trim())
                .collect()
        })
        .unwrap_or_default()
}

/// Compress with GZIP
fn compress_gzip(data: &[u8], level: u32) -> Result<Vec<u8>, StaticFileError> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::new(level));
    encoder.write_all(data)
        .map_err(|e| StaticFileError::Compression(e.to_string()))?;
    encoder.finish()
        .map_err(|e| StaticFileError::Compression(e.to_string()))
}

/// Compress with Brotli
fn compress_brotli(data: &[u8], level: u32) -> Result<Vec<u8>, StaticFileError> {
    let mut output = Vec::new();
    let mut writer = brotli::CompressorWriter::new(&mut output, 4096, level, 22);
    std::io::Write::write_all(&mut writer, data)
        .map_err(|e| StaticFileError::Compression(e.to_string()))?;
    drop(writer);
    Ok(output)
}

/// Base64 encode
fn base64_encode(data: &[u8]) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    for chunk in data.chunks(3) {
        let b0 = chunk.get(0).copied().unwrap_or(0) as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

        result.push(CHARSET[b0 >> 2] as char);
        result.push(CHARSET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);

        if chunk.len() > 1 {
            result.push(CHARSET[((b1 & 0x0F) << 2) | (b2 >> 6)] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(CHARSET[b2 & 0x3F] as char);
        } else {
            result.push('=');
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mime_types() {
        let overrides = HashMap::new();
        assert_eq!(get_mime_type("css", &overrides), "text/css; charset=utf-8");
        assert_eq!(get_mime_type("js", &overrides), "application/javascript; charset=utf-8");
        assert_eq!(get_mime_type("png", &overrides), "image/png");
    }

    #[test]
    fn test_accept_encoding_parsing() {
        let encodings = parse_accept_encoding(Some("gzip, deflate, br"));
        assert!(encodings.contains(&"gzip"));
        assert!(encodings.contains(&"br"));
    }

    #[test]
    fn test_path_normalization() {
        assert_eq!(normalize_path("/static//file.js/"), "/static/file.js");
    }

    #[test]
    fn test_gzip_compression() {
        let data = b"Hello, World! This is some test data for compression.";
        let compressed = compress_gzip(data, 6).unwrap();
        assert!(compressed.len() < data.len());
    }
}
