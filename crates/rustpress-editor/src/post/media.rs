//! Post Media Management
//!
//! Featured images, galleries, and media attachments.

use serde::{Deserialize, Serialize};

/// Featured media (image or video)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturedMedia {
    /// Media attachment ID
    pub id: i64,

    /// Media type
    pub media_type: MediaType,

    /// Full URL
    pub url: String,

    /// Alt text for accessibility
    pub alt: Option<String>,

    /// Caption
    pub caption: Option<String>,

    /// Title
    pub title: Option<String>,

    /// Focal point for cropping (0.0-1.0 coordinates)
    pub focal_point: Option<FocalPoint>,

    /// Available sizes
    #[serde(default)]
    pub sizes: MediaSizes,

    /// Dimensions
    pub width: Option<u32>,
    pub height: Option<u32>,

    /// File info
    pub file_size: Option<u64>,
    pub mime_type: Option<String>,

    /// Video-specific
    pub video_url: Option<String>,
    pub video_poster: Option<String>,
    pub duration: Option<u32>,
}

impl FeaturedMedia {
    /// Create from an image URL
    pub fn from_url(id: i64, url: String) -> Self {
        Self {
            id,
            media_type: MediaType::Image,
            url,
            alt: None,
            caption: None,
            title: None,
            focal_point: None,
            sizes: MediaSizes::default(),
            width: None,
            height: None,
            file_size: None,
            mime_type: None,
            video_url: None,
            video_poster: None,
            duration: None,
        }
    }

    /// Get best URL for given size
    pub fn get_url_for_size(&self, size: ImageSize) -> &str {
        match size {
            ImageSize::Thumbnail => self.sizes.thumbnail.as_deref().unwrap_or(&self.url),
            ImageSize::Medium => self.sizes.medium.as_deref().unwrap_or(&self.url),
            ImageSize::MediumLarge => self.sizes.medium_large.as_deref().unwrap_or(&self.url),
            ImageSize::Large => self.sizes.large.as_deref().unwrap_or(&self.url),
            ImageSize::Full => &self.url,
            ImageSize::Custom(_) => &self.url,
        }
    }

    /// Get srcset for responsive images
    pub fn get_srcset(&self) -> String {
        let mut srcset = Vec::new();

        if let Some(thumb) = &self.sizes.thumbnail {
            srcset.push(format!("{} 150w", thumb));
        }
        if let Some(medium) = &self.sizes.medium {
            srcset.push(format!("{} 300w", medium));
        }
        if let Some(medium_large) = &self.sizes.medium_large {
            srcset.push(format!("{} 768w", medium_large));
        }
        if let Some(large) = &self.sizes.large {
            srcset.push(format!("{} 1024w", large));
        }
        if let Some(width) = self.width {
            srcset.push(format!("{} {}w", self.url, width));
        }

        srcset.join(", ")
    }
}

/// Media type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Image,
    Video,
    Audio,
    Document,
    Embed,
}

/// Focal point for image cropping
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FocalPoint {
    /// X coordinate (0.0 = left, 1.0 = right)
    pub x: f32,
    /// Y coordinate (0.0 = top, 1.0 = bottom)
    pub y: f32,
}

impl Default for FocalPoint {
    fn default() -> Self {
        Self { x: 0.5, y: 0.5 }
    }
}

impl FocalPoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x: x.clamp(0.0, 1.0),
            y: y.clamp(0.0, 1.0),
        }
    }

    /// Get CSS object-position value
    pub fn to_css_position(&self) -> String {
        format!("{}% {}%", (self.x * 100.0) as u32, (self.y * 100.0) as u32)
    }
}

/// Available image sizes
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MediaSizes {
    /// 150x150
    pub thumbnail: Option<String>,
    /// 300xAuto
    pub medium: Option<String>,
    /// 768xAuto
    pub medium_large: Option<String>,
    /// 1024xAuto
    pub large: Option<String>,
    /// 1536xAuto
    pub extra_large: Option<String>,
    /// 2048xAuto
    pub extra_extra_large: Option<String>,
    /// Custom sizes
    #[serde(default)]
    pub custom: std::collections::HashMap<String, String>,
}

/// Image size preset
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageSize {
    Thumbnail,
    Medium,
    MediumLarge,
    Large,
    Full,
    Custom(String),
}

/// Media attachment for posts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaAttachment {
    /// Attachment ID
    pub id: i64,

    /// Original filename
    pub filename: String,

    /// File URL
    pub url: String,

    /// MIME type
    pub mime_type: String,

    /// File size in bytes
    pub file_size: u64,

    /// Upload date
    pub uploaded_at: String,

    /// Uploader ID
    pub uploaded_by: i64,

    /// Alt text
    pub alt: Option<String>,

    /// Title
    pub title: Option<String>,

    /// Caption
    pub caption: Option<String>,

    /// Description
    pub description: Option<String>,

    /// For images
    pub image_meta: Option<ImageMeta>,

    /// For videos
    pub video_meta: Option<VideoMeta>,

    /// For audio
    pub audio_meta: Option<AudioMeta>,
}

/// Image-specific metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMeta {
    pub width: u32,
    pub height: u32,
    pub orientation: Option<u8>,
    pub camera: Option<String>,
    pub aperture: Option<String>,
    pub shutter_speed: Option<String>,
    pub iso: Option<u32>,
    pub focal_length: Option<String>,
    pub created_at: Option<String>,
    pub gps: Option<GpsCoordinates>,
    pub keywords: Vec<String>,
    pub copyright: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpsCoordinates {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
}

/// Video-specific metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMeta {
    pub width: u32,
    pub height: u32,
    pub duration: u32,    // seconds
    pub frame_rate: Option<f32>,
    pub bitrate: Option<u32>,
    pub codec: Option<String>,
    pub audio_codec: Option<String>,
    pub audio_channels: Option<u8>,
    pub has_audio: bool,
}

/// Audio-specific metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMeta {
    pub duration: u32,    // seconds
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u8>,
    pub codec: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
    pub track_number: Option<u32>,
    pub year: Option<u32>,
    pub genre: Option<String>,
}

/// Gallery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryConfig {
    /// Gallery images
    pub images: Vec<GalleryImage>,

    /// Layout style
    pub layout: GalleryLayout,

    /// Number of columns
    pub columns: u8,

    /// Gap between images
    pub gap: String,

    /// Image crop mode
    pub crop: bool,

    /// Link images to
    pub link_to: GalleryLinkTo,

    /// Lightbox enabled
    pub lightbox: bool,

    /// Captions visible
    pub show_captions: bool,

    /// Random order
    pub random_order: bool,
}

impl Default for GalleryConfig {
    fn default() -> Self {
        Self {
            images: Vec::new(),
            layout: GalleryLayout::Grid,
            columns: 3,
            gap: "8px".to_string(),
            crop: true,
            link_to: GalleryLinkTo::Media,
            lightbox: true,
            show_captions: false,
            random_order: false,
        }
    }
}

/// Gallery image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryImage {
    pub id: i64,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub alt: Option<String>,
    pub caption: Option<String>,
    pub link: Option<String>,
}

/// Gallery layout styles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GalleryLayout {
    Grid,
    Masonry,
    Carousel,
    Slider,
    Justified,
    Tiles,
}

/// Gallery link destination
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GalleryLinkTo {
    None,
    Media,
    Attachment,
    Custom,
}
