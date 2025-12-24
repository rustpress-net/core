//! Video processing and transcoding
//!
//! Provides video handling functionality including:
//! - Metadata extraction
//! - Thumbnail/poster generation
//! - Transcoding support (via external tools)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{MediaError, MediaResult};

/// Video metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub id: Uuid,
    pub media_id: Uuid,
    pub codec: Option<String>,
    pub bitrate: Option<i32>,
    pub framerate: Option<f64>,
    pub resolution: Option<String>,
    pub has_audio: bool,
    pub poster_url: Option<String>,
    pub transcoded_versions: Vec<TranscodedVersion>,
    pub created_at: DateTime<Utc>,
}

/// Transcoded video version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscodedVersion {
    pub quality: String,       // "1080p", "720p", "480p", "360p"
    pub format: String,        // "mp4", "webm"
    pub codec: String,         // "h264", "vp9"
    pub bitrate: i32,
    pub file_size: i64,
    pub url: String,
}

/// Video service
pub struct VideoService {
    pool: PgPool,
    storage_path: String,
}

impl VideoService {
    pub fn new(pool: PgPool, storage_path: String) -> Self {
        Self { pool, storage_path }
    }

    /// Get video metadata
    pub async fn get_metadata(&self, media_id: Uuid) -> MediaResult<VideoMetadata> {
        let row = sqlx::query!(
            r#"
            SELECT id, media_id, codec, bitrate, framerate, resolution,
                   has_audio, poster_url, transcoded_versions, created_at
            FROM video_metadata
            WHERE media_id = $1
            "#,
            media_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(MediaError::NotFound(media_id))?;

        let transcoded: Vec<TranscodedVersion> = serde_json::from_value(
            row.transcoded_versions.unwrap_or_else(|| serde_json::json!([]))
        ).unwrap_or_default();

        Ok(VideoMetadata {
            id: row.id,
            media_id: row.media_id,
            codec: row.codec,
            bitrate: row.bitrate,
            framerate: row.framerate,
            resolution: row.resolution,
            has_audio: row.has_audio.unwrap_or(true),
            poster_url: row.poster_url,
            transcoded_versions: transcoded,
            created_at: row.created_at,
        })
    }

    /// Generate video player HTML
    pub fn generate_player_html(
        &self,
        video_url: &str,
        poster_url: Option<&str>,
        width: Option<u32>,
        height: Option<u32>,
        autoplay: bool,
        controls: bool,
        loop_video: bool,
        muted: bool,
    ) -> String {
        let mut attrs = vec![format!("src=\"{}\"", video_url)];

        if let Some(poster) = poster_url {
            attrs.push(format!("poster=\"{}\"", poster));
        }
        if let Some(w) = width {
            attrs.push(format!("width=\"{}\"", w));
        }
        if let Some(h) = height {
            attrs.push(format!("height=\"{}\"", h));
        }
        if autoplay {
            attrs.push("autoplay".to_string());
        }
        if controls {
            attrs.push("controls".to_string());
        }
        if loop_video {
            attrs.push("loop".to_string());
        }
        if muted {
            attrs.push("muted".to_string());
        }

        attrs.push("playsinline".to_string());
        attrs.push("preload=\"metadata\"".to_string());

        format!("<video {}></video>", attrs.join(" "))
    }

    /// Generate responsive video with multiple sources
    pub fn generate_responsive_video(
        &self,
        metadata: &VideoMetadata,
        fallback_url: &str,
    ) -> String {
        let mut html = String::from("<video controls playsinline preload=\"metadata\"");

        if let Some(poster) = &metadata.poster_url {
            html.push_str(&format!(" poster=\"{}\"", poster));
        }

        html.push_str(">");

        // Add transcoded sources (prefer modern formats)
        let mut sources: Vec<&TranscodedVersion> = metadata.transcoded_versions.iter().collect();
        sources.sort_by(|a, b| {
            // Prefer webm, then mp4
            let format_order = |f: &str| match f {
                "webm" => 0,
                "mp4" => 1,
                _ => 2,
            };
            format_order(&a.format).cmp(&format_order(&b.format))
        });

        for version in sources {
            let mime = match version.format.as_str() {
                "mp4" => "video/mp4",
                "webm" => "video/webm",
                _ => "video/mp4",
            };
            html.push_str(&format!(
                "<source src=\"{}\" type=\"{}\" data-quality=\"{}\">",
                version.url, mime, version.quality
            ));
        }

        // Fallback
        html.push_str(&format!("<source src=\"{}\" type=\"video/mp4\">", fallback_url));
        html.push_str("Your browser does not support the video tag.");
        html.push_str("</video>");

        html
    }
}

/// Transcoding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscodeConfig {
    /// Output qualities to generate
    pub qualities: Vec<TranscodeQuality>,

    /// Output formats
    pub formats: Vec<String>,

    /// Use hardware acceleration if available
    pub hardware_acceleration: bool,

    /// Maximum concurrent transcodes
    pub max_concurrent: usize,
}

impl Default for TranscodeConfig {
    fn default() -> Self {
        Self {
            qualities: vec![
                TranscodeQuality::P1080,
                TranscodeQuality::P720,
                TranscodeQuality::P480,
            ],
            formats: vec!["mp4".to_string(), "webm".to_string()],
            hardware_acceleration: true,
            max_concurrent: 2,
        }
    }
}

/// Transcoding quality presets
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TranscodeQuality {
    P2160, // 4K
    P1440, // 2K
    P1080, // Full HD
    P720,  // HD
    P480,  // SD
    P360,  // Low
}

impl TranscodeQuality {
    pub fn resolution(&self) -> (u32, u32) {
        match self {
            Self::P2160 => (3840, 2160),
            Self::P1440 => (2560, 1440),
            Self::P1080 => (1920, 1080),
            Self::P720 => (1280, 720),
            Self::P480 => (854, 480),
            Self::P360 => (640, 360),
        }
    }

    pub fn bitrate(&self) -> i32 {
        match self {
            Self::P2160 => 15000,
            Self::P1440 => 8000,
            Self::P1080 => 5000,
            Self::P720 => 2500,
            Self::P480 => 1000,
            Self::P360 => 500,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::P2160 => "2160p",
            Self::P1440 => "1440p",
            Self::P1080 => "1080p",
            Self::P720 => "720p",
            Self::P480 => "480p",
            Self::P360 => "360p",
        }
    }
}

/// Video player configuration for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoPlayerConfig {
    pub autoplay: bool,
    pub controls: bool,
    pub loop_video: bool,
    pub muted: bool,
    pub preload: String, // "none", "metadata", "auto"
    pub playsinline: bool,
    pub quality_selector: bool,
    pub playback_speed: bool,
    pub fullscreen: bool,
    pub picture_in_picture: bool,
    pub keyboard_shortcuts: bool,
}

impl Default for VideoPlayerConfig {
    fn default() -> Self {
        Self {
            autoplay: false,
            controls: true,
            loop_video: false,
            muted: false,
            preload: "metadata".to_string(),
            playsinline: true,
            quality_selector: true,
            playback_speed: true,
            fullscreen: true,
            picture_in_picture: true,
            keyboard_shortcuts: true,
        }
    }
}
