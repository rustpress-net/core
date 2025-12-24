//! # Media Library
//!
//! Comprehensive media management system with image processing capabilities.
//!
//! Features:
//! - Media upload and storage
//! - Image resizing and thumbnail generation
//! - Lazy loading placeholders
//! - Image optimization
//! - Media metadata extraction
//! - Attachment management

use chrono::{DateTime, Utc};
use image::{self, DynamicImage, GenericImageView, ImageFormat, ImageError};
use image::imageops::FilterType;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

/// Media library errors
#[derive(Debug, Error)]
pub enum MediaError {
    #[error("Image processing error: {0}")]
    ImageError(#[from] ImageError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid file type: {0}")]
    InvalidFileType(String),

    #[error("File too large: {0} bytes (max: {1})")]
    FileTooLarge(u64, u64),

    #[error("Invalid dimensions: {0}x{1}")]
    InvalidDimensions(u32, u32),

    #[error("Media not found: {0}")]
    NotFound(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Metadata error: {0}")]
    MetadataError(String),
}

/// Media item representing an uploaded file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    /// Unique identifier
    pub id: Uuid,

    /// Original filename
    pub filename: String,

    /// File title (for display)
    pub title: String,

    /// Alt text for images
    pub alt_text: String,

    /// Caption
    pub caption: String,

    /// Description
    pub description: String,

    /// MIME type
    pub mime_type: String,

    /// Media type category
    pub media_type: MediaType,

    /// File size in bytes
    pub file_size: u64,

    /// Storage path relative to uploads directory
    pub path: String,

    /// Full URL to the media
    pub url: String,

    /// Image dimensions (if image)
    pub dimensions: Option<ImageDimensions>,

    /// Generated sizes (thumbnails, etc.)
    pub sizes: HashMap<String, MediaSize>,

    /// Extracted metadata
    pub metadata: MediaMetadata,

    /// Upload date
    pub uploaded_at: DateTime<Utc>,

    /// Last modified date
    pub modified_at: DateTime<Utc>,

    /// Uploader user ID
    pub uploaded_by: i64,

    /// Parent post ID (if attached)
    pub parent_id: Option<i64>,

    /// Focal point for smart cropping
    pub focal_point: Option<FocalPoint>,

    /// Custom metadata
    pub custom_meta: HashMap<String, String>,
}

impl MediaItem {
    pub fn new(filename: String, mime_type: String, uploaded_by: i64) -> Self {
        let media_type = MediaType::from_mime(&mime_type);

        Self {
            id: Uuid::new_v4(),
            filename: filename.clone(),
            title: Path::new(&filename)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default(),
            alt_text: String::new(),
            caption: String::new(),
            description: String::new(),
            mime_type,
            media_type,
            file_size: 0,
            path: String::new(),
            url: String::new(),
            dimensions: None,
            sizes: HashMap::new(),
            metadata: MediaMetadata::default(),
            uploaded_at: Utc::now(),
            modified_at: Utc::now(),
            uploaded_by,
            parent_id: None,
            focal_point: None,
            custom_meta: HashMap::new(),
        }
    }

    /// Get URL for specific size
    pub fn get_size_url(&self, size: &str) -> Option<&str> {
        self.sizes.get(size).map(|s| s.url.as_str())
    }

    /// Get srcset attribute value for responsive images
    pub fn srcset(&self) -> String {
        let mut srcset_parts: Vec<String> = self.sizes
            .iter()
            .filter_map(|(_, size)| {
                size.width.map(|w| format!("{} {}w", size.url, w))
            })
            .collect();

        // Add original size
        if let Some(dims) = &self.dimensions {
            srcset_parts.push(format!("{} {}w", self.url, dims.width));
        }

        srcset_parts.join(", ")
    }

    /// Check if this is an image
    pub fn is_image(&self) -> bool {
        matches!(self.media_type, MediaType::Image)
    }

    /// Check if this is a video
    pub fn is_video(&self) -> bool {
        matches!(self.media_type, MediaType::Video)
    }

    /// Check if this is audio
    pub fn is_audio(&self) -> bool {
        matches!(self.media_type, MediaType::Audio)
    }
}

/// Media type categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
        } else if mime.starts_with("application/pdf")
            || mime.starts_with("application/msword")
            || mime.starts_with("application/vnd.")
            || mime.starts_with("text/")
        {
            Self::Document
        } else if mime.contains("zip") || mime.contains("tar") || mime.contains("rar") {
            Self::Archive
        } else {
            Self::Other
        }
    }

    pub fn extensions(&self) -> &[&str] {
        match self {
            Self::Image => &["jpg", "jpeg", "png", "gif", "webp", "svg", "ico", "bmp", "tiff"],
            Self::Video => &["mp4", "webm", "ogg", "avi", "mov", "wmv", "flv", "mkv"],
            Self::Audio => &["mp3", "wav", "ogg", "flac", "aac", "wma", "m4a"],
            Self::Document => &["pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt", "rtf", "odt"],
            Self::Archive => &["zip", "tar", "gz", "rar", "7z"],
            Self::Other => &[],
        }
    }
}

/// Image dimensions
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}

impl ImageDimensions {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn aspect_ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }

    pub fn is_landscape(&self) -> bool {
        self.width > self.height
    }

    pub fn is_portrait(&self) -> bool {
        self.height > self.width
    }

    pub fn is_square(&self) -> bool {
        self.width == self.height
    }
}

/// Generated media size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaSize {
    /// Size name (e.g., "thumbnail", "medium", "large")
    pub name: String,

    /// URL to this size
    pub url: String,

    /// Storage path
    pub path: String,

    /// Width in pixels
    pub width: Option<u32>,

    /// Height in pixels
    pub height: Option<u32>,

    /// File size in bytes
    pub file_size: u64,

    /// MIME type (may differ from original for format conversion)
    pub mime_type: String,
}

/// Focal point for smart cropping
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FocalPoint {
    /// X coordinate as percentage (0.0 to 1.0)
    pub x: f64,

    /// Y coordinate as percentage (0.0 to 1.0)
    pub y: f64,
}

impl Default for FocalPoint {
    fn default() -> Self {
        Self { x: 0.5, y: 0.5 } // Center
    }
}

impl FocalPoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x: x.clamp(0.0, 1.0),
            y: y.clamp(0.0, 1.0),
        }
    }

    pub fn center() -> Self {
        Self::default()
    }

    pub fn top_left() -> Self {
        Self::new(0.0, 0.0)
    }

    pub fn top_right() -> Self {
        Self::new(1.0, 0.0)
    }

    pub fn bottom_left() -> Self {
        Self::new(0.0, 1.0)
    }

    pub fn bottom_right() -> Self {
        Self::new(1.0, 1.0)
    }
}

/// Media metadata extracted from files
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MediaMetadata {
    /// Image EXIF data
    pub exif: HashMap<String, String>,

    /// Audio/Video duration in seconds
    pub duration: Option<f64>,

    /// Bit rate for audio/video
    pub bitrate: Option<u32>,

    /// Frame rate for video
    pub framerate: Option<f64>,

    /// Audio channels
    pub channels: Option<u8>,

    /// Sample rate for audio
    pub sample_rate: Option<u32>,

    /// Color space
    pub color_space: Option<String>,

    /// Camera model
    pub camera: Option<String>,

    /// Lens info
    pub lens: Option<String>,

    /// GPS coordinates
    pub gps: Option<GpsCoordinates>,

    /// Creation date from metadata
    pub created_at: Option<DateTime<Utc>>,
}

/// GPS coordinates from EXIF
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GpsCoordinates {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
}

/// Image size definition for thumbnail generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSizeDefinition {
    /// Size name
    pub name: String,

    /// Maximum width
    pub width: u32,

    /// Maximum height
    pub height: u32,

    /// Crop mode
    pub crop: CropMode,

    /// Output quality (1-100)
    pub quality: u8,

    /// Output format (None = keep original)
    pub format: Option<ImageOutputFormat>,
}

impl ImageSizeDefinition {
    pub fn new(name: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            name: name.into(),
            width,
            height,
            crop: CropMode::None,
            quality: 85,
            format: None,
        }
    }

    pub fn thumbnail() -> Self {
        Self {
            name: "thumbnail".to_string(),
            width: 150,
            height: 150,
            crop: CropMode::Center,
            quality: 85,
            format: None,
        }
    }

    pub fn medium() -> Self {
        Self::new("medium", 300, 300)
    }

    pub fn medium_large() -> Self {
        Self::new("medium_large", 768, 0) // 0 = auto height
    }

    pub fn large() -> Self {
        Self::new("large", 1024, 1024)
    }

    pub fn full_hd() -> Self {
        Self::new("full_hd", 1920, 1080)
    }

    pub fn with_crop(mut self, crop: CropMode) -> Self {
        self.crop = crop;
        self
    }

    pub fn with_quality(mut self, quality: u8) -> Self {
        self.quality = quality.min(100);
        self
    }

    pub fn with_format(mut self, format: ImageOutputFormat) -> Self {
        self.format = Some(format);
        self
    }
}

/// Crop mode for image resizing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CropMode {
    /// No cropping, just resize
    None,

    /// Crop from center
    Center,

    /// Crop from focal point
    FocalPoint,

    /// Crop from specific position
    Position(CropPosition),
}

/// Crop position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CropPosition {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

/// Output format for processed images
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageOutputFormat {
    Jpeg,
    Png,
    WebP,
    Gif,
}

impl ImageOutputFormat {
    pub fn mime_type(&self) -> &str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::WebP => "image/webp",
            Self::Gif => "image/gif",
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::WebP => "webp",
            Self::Gif => "gif",
        }
    }

    pub fn from_mime(mime: &str) -> Option<Self> {
        match mime {
            "image/jpeg" | "image/jpg" => Some(Self::Jpeg),
            "image/png" => Some(Self::Png),
            "image/webp" => Some(Self::WebP),
            "image/gif" => Some(Self::Gif),
            _ => None,
        }
    }

    pub fn to_image_format(&self) -> ImageFormat {
        match self {
            Self::Jpeg => ImageFormat::Jpeg,
            Self::Png => ImageFormat::Png,
            Self::WebP => ImageFormat::WebP,
            Self::Gif => ImageFormat::Gif,
        }
    }
}

/// Image processor for resizing and optimization
pub struct ImageProcessor {
    /// Registered size definitions
    sizes: HashMap<String, ImageSizeDefinition>,

    /// Default quality for JPEG output
    default_quality: u8,

    /// Enable WebP conversion
    convert_to_webp: bool,

    /// Maximum dimension for uploaded images
    max_dimension: u32,

    /// Strip EXIF data from output
    strip_exif: bool,

    /// Enable lazy loading placeholder generation
    generate_placeholders: bool,

    /// Placeholder quality (for LQIP)
    placeholder_quality: u8,

    /// Placeholder max dimension
    placeholder_size: u32,
}

impl Default for ImageProcessor {
    fn default() -> Self {
        let mut sizes = HashMap::new();
        sizes.insert("thumbnail".to_string(), ImageSizeDefinition::thumbnail());
        sizes.insert("medium".to_string(), ImageSizeDefinition::medium());
        sizes.insert("medium_large".to_string(), ImageSizeDefinition::medium_large());
        sizes.insert("large".to_string(), ImageSizeDefinition::large());

        Self {
            sizes,
            default_quality: 85,
            convert_to_webp: false,
            max_dimension: 2560,
            strip_exif: true,
            generate_placeholders: true,
            placeholder_quality: 20,
            placeholder_size: 20,
        }
    }
}

impl ImageProcessor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new image size
    pub fn register_size(&mut self, definition: ImageSizeDefinition) {
        self.sizes.insert(definition.name.clone(), definition);
    }

    /// Remove an image size
    pub fn unregister_size(&mut self, name: &str) {
        self.sizes.remove(name);
    }

    /// Get all registered sizes
    pub fn get_sizes(&self) -> &HashMap<String, ImageSizeDefinition> {
        &self.sizes
    }

    /// Process an image and generate all sizes
    pub fn process(&self, image_data: &[u8], original_mime: &str) -> Result<ProcessedImage, MediaError> {
        let img = image::load_from_memory(image_data)?;
        let (width, height) = img.dimensions();

        // Check if resizing of original is needed
        let img = if width > self.max_dimension || height > self.max_dimension {
            self.resize_image(&img, self.max_dimension, self.max_dimension, CropMode::None, None)?
        } else {
            img
        };

        let (width, height) = img.dimensions();
        let original_dimensions = ImageDimensions::new(width, height);

        // Generate all registered sizes
        let mut generated_sizes = HashMap::new();
        for (name, definition) in &self.sizes {
            // Skip if original is smaller than target
            if width <= definition.width && height <= definition.height && definition.crop == CropMode::None {
                continue;
            }

            let resized = self.resize_image(
                &img,
                definition.width,
                definition.height,
                definition.crop,
                None,
            )?;

            let (w, h) = resized.dimensions();
            let format = definition.format.unwrap_or_else(|| {
                ImageOutputFormat::from_mime(original_mime).unwrap_or(ImageOutputFormat::Jpeg)
            });

            let data = self.encode_image(&resized, format, definition.quality)?;

            generated_sizes.insert(name.clone(), GeneratedSize {
                data,
                width: w,
                height: h,
                format,
                quality: definition.quality,
            });
        }

        // Generate lazy loading placeholder (LQIP)
        let placeholder = if self.generate_placeholders {
            Some(self.generate_placeholder(&img)?)
        } else {
            None
        };

        // Generate WebP version if enabled
        let webp_data = if self.convert_to_webp {
            Some(self.encode_image(&img, ImageOutputFormat::WebP, self.default_quality)?)
        } else {
            None
        };

        // Encode original (possibly resized)
        let original_format = ImageOutputFormat::from_mime(original_mime).unwrap_or(ImageOutputFormat::Jpeg);
        let original_data = self.encode_image(&img, original_format, self.default_quality)?;

        Ok(ProcessedImage {
            original_data,
            original_dimensions,
            original_format,
            sizes: generated_sizes,
            placeholder,
            webp_data,
        })
    }

    /// Resize an image
    fn resize_image(
        &self,
        img: &DynamicImage,
        max_width: u32,
        max_height: u32,
        crop: CropMode,
        focal_point: Option<FocalPoint>,
    ) -> Result<DynamicImage, MediaError> {
        let (orig_width, orig_height) = img.dimensions();

        // Handle 0 values (auto dimension)
        let (target_width, target_height) = if max_width == 0 && max_height == 0 {
            (orig_width, orig_height)
        } else if max_width == 0 {
            let ratio = max_height as f64 / orig_height as f64;
            ((orig_width as f64 * ratio) as u32, max_height)
        } else if max_height == 0 {
            let ratio = max_width as f64 / orig_width as f64;
            (max_width, (orig_height as f64 * ratio) as u32)
        } else {
            (max_width, max_height)
        };

        match crop {
            CropMode::None => {
                // Scale to fit within bounds, maintaining aspect ratio
                Ok(img.resize(target_width, target_height, FilterType::Lanczos3))
            }
            CropMode::Center | CropMode::FocalPoint => {
                // Crop to exact dimensions
                let focal = if crop == CropMode::FocalPoint {
                    focal_point.unwrap_or_default()
                } else {
                    FocalPoint::center()
                };

                Ok(self.crop_to_focal(img, target_width, target_height, focal))
            }
            CropMode::Position(pos) => {
                let focal = match pos {
                    CropPosition::TopLeft => FocalPoint::new(0.0, 0.0),
                    CropPosition::TopCenter => FocalPoint::new(0.5, 0.0),
                    CropPosition::TopRight => FocalPoint::new(1.0, 0.0),
                    CropPosition::CenterLeft => FocalPoint::new(0.0, 0.5),
                    CropPosition::Center => FocalPoint::center(),
                    CropPosition::CenterRight => FocalPoint::new(1.0, 0.5),
                    CropPosition::BottomLeft => FocalPoint::new(0.0, 1.0),
                    CropPosition::BottomCenter => FocalPoint::new(0.5, 1.0),
                    CropPosition::BottomRight => FocalPoint::new(1.0, 1.0),
                };
                Ok(self.crop_to_focal(img, target_width, target_height, focal))
            }
        }
    }

    /// Crop image centered on focal point
    fn crop_to_focal(
        &self,
        img: &DynamicImage,
        target_width: u32,
        target_height: u32,
        focal: FocalPoint,
    ) -> DynamicImage {
        let (orig_width, orig_height) = img.dimensions();

        // Calculate scale to cover target dimensions
        let scale_x = target_width as f64 / orig_width as f64;
        let scale_y = target_height as f64 / orig_height as f64;
        let scale = scale_x.max(scale_y);

        let scaled_width = (orig_width as f64 * scale) as u32;
        let scaled_height = (orig_height as f64 * scale) as u32;

        // Scale up first
        let scaled = img.resize_exact(scaled_width, scaled_height, FilterType::Lanczos3);

        // Calculate crop position based on focal point
        let max_x = scaled_width.saturating_sub(target_width);
        let max_y = scaled_height.saturating_sub(target_height);

        let crop_x = (max_x as f64 * focal.x) as u32;
        let crop_y = (max_y as f64 * focal.y) as u32;

        scaled.crop_imm(crop_x, crop_y, target_width, target_height)
    }

    /// Encode image to bytes
    fn encode_image(
        &self,
        img: &DynamicImage,
        format: ImageOutputFormat,
        quality: u8,
    ) -> Result<Vec<u8>, MediaError> {
        let mut buffer = Cursor::new(Vec::new());

        match format {
            ImageOutputFormat::Jpeg => {
                img.write_to(&mut buffer, ImageFormat::Jpeg)?;
            }
            ImageOutputFormat::Png => {
                img.write_to(&mut buffer, ImageFormat::Png)?;
            }
            ImageOutputFormat::WebP => {
                img.write_to(&mut buffer, ImageFormat::WebP)?;
            }
            ImageOutputFormat::Gif => {
                img.write_to(&mut buffer, ImageFormat::Gif)?;
            }
        }

        Ok(buffer.into_inner())
    }

    /// Generate low-quality image placeholder (LQIP)
    fn generate_placeholder(&self, img: &DynamicImage) -> Result<LazyLoadPlaceholder, MediaError> {
        let (orig_width, orig_height) = img.dimensions();

        // Create tiny version
        let tiny = img.resize(self.placeholder_size, self.placeholder_size, FilterType::Triangle);
        let (w, h) = tiny.dimensions();

        // Encode as JPEG with low quality
        let data = self.encode_image(&tiny, ImageOutputFormat::Jpeg, self.placeholder_quality)?;

        // Create base64 data URL
        let base64 = base64_encode(&data);
        let data_url = format!("data:image/jpeg;base64,{}", base64);

        Ok(LazyLoadPlaceholder {
            data_url,
            width: w,
            height: h,
            aspect_ratio: orig_width as f64 / orig_height as f64,
            blurhash: None, // Could implement blurhash here
        })
    }

    /// Generate dominant color from image
    pub fn get_dominant_color(&self, img: &DynamicImage) -> String {
        let tiny = img.resize(1, 1, FilterType::Triangle);
        let pixel = tiny.get_pixel(0, 0);
        format!("#{:02x}{:02x}{:02x}", pixel[0], pixel[1], pixel[2])
    }

    /// Extract basic metadata from image
    pub fn extract_metadata(&self, _image_data: &[u8]) -> MediaMetadata {
        // Basic implementation - could be expanded with exif crate
        MediaMetadata::default()
    }
}

/// Result of image processing
#[derive(Debug)]
pub struct ProcessedImage {
    /// Processed original image data
    pub original_data: Vec<u8>,

    /// Original image dimensions
    pub original_dimensions: ImageDimensions,

    /// Original image format
    pub original_format: ImageOutputFormat,

    /// Generated sizes
    pub sizes: HashMap<String, GeneratedSize>,

    /// Lazy loading placeholder
    pub placeholder: Option<LazyLoadPlaceholder>,

    /// WebP version of original
    pub webp_data: Option<Vec<u8>>,
}

/// Generated image size
#[derive(Debug)]
pub struct GeneratedSize {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: ImageOutputFormat,
    pub quality: u8,
}

/// Lazy loading placeholder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LazyLoadPlaceholder {
    /// Base64 encoded data URL for inline use
    pub data_url: String,

    /// Placeholder width
    pub width: u32,

    /// Placeholder height
    pub height: u32,

    /// Original aspect ratio
    pub aspect_ratio: f64,

    /// BlurHash string (if generated)
    pub blurhash: Option<String>,
}

impl LazyLoadPlaceholder {
    /// Generate CSS for aspect ratio box
    pub fn aspect_ratio_css(&self) -> String {
        format!("padding-bottom: {:.2}%", 100.0 / self.aspect_ratio)
    }

    /// Generate img tag with placeholder
    pub fn img_tag(&self, src: &str, alt: &str, class: &str) -> String {
        format!(
            r#"<img src="{}" data-src="{}" alt="{}" class="lazy {}" style="background-image: url({})">"#,
            self.data_url, src, alt, class, self.data_url
        )
    }
}

/// Generate responsive image HTML
pub fn responsive_image(media: &MediaItem, sizes_attr: &str, class: &str) -> String {
    let srcset = media.srcset();
    let alt = if media.alt_text.is_empty() { &media.title } else { &media.alt_text };

    let mut html = format!(
        r#"<img src="{}" srcset="{}" sizes="{}" alt="{}" class="{}""#,
        media.url, srcset, sizes_attr, alt, class
    );

    if let Some(dims) = &media.dimensions {
        html.push_str(&format!(r#" width="{}" height="{}""#, dims.width, dims.height));
    }

    html.push_str(" loading=\"lazy\">");
    html
}

/// Generate picture element with WebP support
pub fn picture_element(media: &MediaItem, sizes_attr: &str, class: &str) -> String {
    let alt = if media.alt_text.is_empty() { &media.title } else { &media.alt_text };

    let mut html = String::from("<picture>");

    // WebP source (if available)
    if let Some(webp_url) = media.custom_meta.get("webp_url") {
        let webp_srcset = media.sizes.iter()
            .filter_map(|(name, size)| {
                size.width.map(|w| format!("{}.webp {}w", size.url.trim_end_matches(&format!(".{}", media.mime_type.split('/').last().unwrap_or("jpg"))), w))
            })
            .collect::<Vec<_>>()
            .join(", ");

        if !webp_srcset.is_empty() {
            html.push_str(&format!(
                r#"<source type="image/webp" srcset="{}" sizes="{}">"#,
                webp_srcset, sizes_attr
            ));
        }
    }

    // Original format source
    let srcset = media.srcset();
    html.push_str(&format!(
        r#"<source srcset="{}" sizes="{}">"#,
        srcset, sizes_attr
    ));

    // Fallback img
    html.push_str(&format!(
        r#"<img src="{}" alt="{}" class="{}" loading="lazy">"#,
        media.url, alt, class
    ));

    html.push_str("</picture>");
    html
}

/// Base64 encode helper
fn base64_encode(data: &[u8]) -> String {
    use base64::{Engine, engine::general_purpose::STANDARD};
    STANDARD.encode(data)
}

/// Media library manager
pub struct MediaLibrary {
    /// Image processor
    processor: ImageProcessor,

    /// Allowed MIME types
    allowed_types: HashMap<String, Vec<String>>,

    /// Maximum file size in bytes
    max_file_size: u64,

    /// Upload directory base path
    upload_dir: PathBuf,

    /// Base URL for media files
    base_url: String,
}

impl MediaLibrary {
    pub fn new(upload_dir: PathBuf, base_url: String) -> Self {
        let mut allowed_types = HashMap::new();

        // Images
        allowed_types.insert("image".to_string(), vec![
            "image/jpeg".to_string(),
            "image/png".to_string(),
            "image/gif".to_string(),
            "image/webp".to_string(),
            "image/svg+xml".to_string(),
        ]);

        // Videos
        allowed_types.insert("video".to_string(), vec![
            "video/mp4".to_string(),
            "video/webm".to_string(),
            "video/ogg".to_string(),
        ]);

        // Audio
        allowed_types.insert("audio".to_string(), vec![
            "audio/mpeg".to_string(),
            "audio/wav".to_string(),
            "audio/ogg".to_string(),
            "audio/webm".to_string(),
        ]);

        // Documents
        allowed_types.insert("document".to_string(), vec![
            "application/pdf".to_string(),
            "application/msword".to_string(),
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
        ]);

        Self {
            processor: ImageProcessor::new(),
            allowed_types,
            max_file_size: 50 * 1024 * 1024, // 50MB default
            upload_dir,
            base_url,
        }
    }

    /// Check if MIME type is allowed
    pub fn is_allowed(&self, mime_type: &str) -> bool {
        self.allowed_types.values().any(|types| types.contains(&mime_type.to_string()))
    }

    /// Validate upload
    pub fn validate(&self, data: &[u8], mime_type: &str) -> Result<(), MediaError> {
        // Check file size
        if data.len() as u64 > self.max_file_size {
            return Err(MediaError::FileTooLarge(data.len() as u64, self.max_file_size));
        }

        // Check MIME type
        if !self.is_allowed(mime_type) {
            return Err(MediaError::InvalidFileType(mime_type.to_string()));
        }

        Ok(())
    }

    /// Generate upload path based on date
    pub fn generate_path(&self, filename: &str) -> (PathBuf, String) {
        let now = Utc::now();
        let year = now.format("%Y").to_string();
        let month = now.format("%m").to_string();

        // Sanitize filename
        let safe_name = self.sanitize_filename(filename);

        let relative_path = format!("{}/{}/{}", year, month, safe_name);
        let full_path = self.upload_dir.join(&relative_path);

        (full_path, relative_path)
    }

    /// Sanitize filename for safe storage
    fn sanitize_filename(&self, filename: &str) -> String {
        let re = Regex::new(r"[^a-zA-Z0-9._-]").unwrap();
        let sanitized = re.replace_all(filename, "_").to_string();

        // Ensure unique name
        let stem = Path::new(&sanitized)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let ext = Path::new(&sanitized)
            .extension()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        let unique_id = Uuid::new_v4().to_string().split('-').next().unwrap().to_string();

        if ext.is_empty() {
            format!("{}_{}", stem, unique_id)
        } else {
            format!("{}_{}.{}", stem, unique_id, ext)
        }
    }

    /// Get image processor
    pub fn processor(&self) -> &ImageProcessor {
        &self.processor
    }

    /// Get mutable image processor
    pub fn processor_mut(&mut self) -> &mut ImageProcessor {
        &mut self.processor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_type_from_mime() {
        assert_eq!(MediaType::from_mime("image/jpeg"), MediaType::Image);
        assert_eq!(MediaType::from_mime("video/mp4"), MediaType::Video);
        assert_eq!(MediaType::from_mime("audio/mpeg"), MediaType::Audio);
        assert_eq!(MediaType::from_mime("application/pdf"), MediaType::Document);
    }

    #[test]
    fn test_focal_point() {
        let fp = FocalPoint::new(0.5, 0.5);
        assert_eq!(fp.x, 0.5);
        assert_eq!(fp.y, 0.5);

        let fp_clamped = FocalPoint::new(1.5, -0.5);
        assert_eq!(fp_clamped.x, 1.0);
        assert_eq!(fp_clamped.y, 0.0);
    }

    #[test]
    fn test_image_dimensions() {
        let dims = ImageDimensions::new(1920, 1080);
        assert!(dims.is_landscape());
        assert!(!dims.is_portrait());
        assert!((dims.aspect_ratio() - 16.0/9.0).abs() < 0.01);
    }

    #[test]
    fn test_size_definitions() {
        let thumb = ImageSizeDefinition::thumbnail();
        assert_eq!(thumb.width, 150);
        assert_eq!(thumb.height, 150);
        assert_eq!(thumb.crop, CropMode::Center);
    }
}
