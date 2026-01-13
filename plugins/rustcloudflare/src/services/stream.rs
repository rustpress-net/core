//! Stream Video Service
//!
//! Manage Cloudflare Stream for video hosting and live streaming

use crate::client::CloudflareClient;
use crate::error::{CloudflareError, CloudflareResult};
use crate::models::{StreamVideo, LiveInput, CreateLiveInput, StreamStats};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{debug, info};

/// Stream video service
pub struct StreamService {
    client: Option<Arc<CloudflareClient>>,
    #[allow(dead_code)]
    db: PgPool,
}

impl StreamService {
    /// Create a new Stream service
    pub fn new(client: Arc<CloudflareClient>, db: PgPool) -> Self {
        Self {
            client: Some(client),
            db,
        }
    }

    /// Create a new Stream service without a client
    pub fn new_unconfigured(db: PgPool) -> Self {
        Self { client: None, db }
    }

    /// Get the client or return an error
    fn client(&self) -> CloudflareResult<&CloudflareClient> {
        self.client
            .as_ref()
            .map(|c| c.as_ref())
            .ok_or(CloudflareError::NotConfigured)
    }

    // =========================================================================
    // Video Management
    // =========================================================================

    /// List all videos
    pub async fn list_videos(&self) -> CloudflareResult<Vec<StreamVideo>> {
        let client = self.client()?;
        let videos = client.list_stream_videos().await?;
        debug!("Found {} Stream videos", videos.len());
        Ok(videos)
    }

    /// Get a video by ID
    pub async fn get_video(&self, video_id: &str) -> CloudflareResult<StreamVideo> {
        let client = self.client()?;
        client.get_stream_video(video_id).await
    }

    /// Delete a video
    pub async fn delete_video(&self, video_id: &str) -> CloudflareResult<()> {
        let client = self.client()?;
        info!("Deleting Stream video: {}", video_id);
        client.delete_stream_video(video_id).await?;
        info!("Stream video deleted: {}", video_id);
        Ok(())
    }

    /// Get embed URL for a video
    pub fn get_embed_url(&self, video_id: &str) -> String {
        format!(
            "https://iframe.cloudflarestream.com/{}",
            video_id
        )
    }

    /// Get HLS URL for a video
    pub fn get_hls_url(&self, video_id: &str) -> String {
        format!(
            "https://cloudflarestream.com/{}/manifest/video.m3u8",
            video_id
        )
    }

    /// Get DASH URL for a video
    pub fn get_dash_url(&self, video_id: &str) -> String {
        format!(
            "https://cloudflarestream.com/{}/manifest/video.mpd",
            video_id
        )
    }

    /// Get thumbnail URL for a video
    pub fn get_thumbnail_url(&self, video_id: &str, time_seconds: Option<u32>) -> String {
        let time = time_seconds.unwrap_or(0);
        format!(
            "https://cloudflarestream.com/{}/thumbnails/thumbnail.jpg?time={}s",
            video_id, time
        )
    }

    /// Get animated thumbnail (GIF) URL
    pub fn get_animated_thumbnail_url(
        &self,
        video_id: &str,
        start: Option<u32>,
        end: Option<u32>,
    ) -> String {
        let start_time = start.unwrap_or(0);
        let end_time = end.unwrap_or(5);
        format!(
            "https://cloudflarestream.com/{}/thumbnails/thumbnail.gif?time={}s&duration={}s",
            video_id, start_time, end_time - start_time
        )
    }

    /// Generate iframe embed code for a video
    pub fn generate_embed_code(&self, video_id: &str, options: EmbedOptions) -> String {
        let mut params = Vec::new();

        if options.autoplay {
            params.push("autoplay=true".to_string());
        }
        if options.muted {
            params.push("muted=true".to_string());
        }
        if options.loop_video {
            params.push("loop=true".to_string());
        }
        if !options.controls {
            params.push("controls=false".to_string());
        }
        if let Some(poster) = options.poster_time {
            params.push(format!("poster=https://cloudflarestream.com/{}/thumbnails/thumbnail.jpg?time={}s", video_id, poster));
        }
        if let Some(start) = options.start_time {
            params.push(format!("startTime={}", start));
        }

        let query = if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        };

        format!(
            r#"<iframe
  src="https://iframe.cloudflarestream.com/{}{}"
  style="border: none; width: {}; height: {};"
  allow="accelerometer; autoplay; encrypted-media; gyroscope; picture-in-picture"
  allowfullscreen
></iframe>"#,
            video_id,
            query,
            options.width.unwrap_or("100%".to_string()),
            options.height.unwrap_or("auto".to_string())
        )
    }

    // =========================================================================
    // Live Streaming
    // =========================================================================

    /// List all live inputs
    pub async fn list_live_inputs(&self) -> CloudflareResult<Vec<LiveInput>> {
        let client = self.client()?;
        let inputs = client.list_live_inputs().await?;
        debug!("Found {} live inputs", inputs.len());
        Ok(inputs)
    }

    /// Create a live input
    pub async fn create_live_input(&self, input: CreateLiveInput) -> CloudflareResult<LiveInput> {
        let client = self.client()?;
        info!("Creating live input: {:?}", input.meta);
        let created = client.create_live_input(input).await?;
        info!("Live input created: {}", created.uid);
        Ok(created)
    }

    /// Get RTMPS URL for a live input
    pub fn get_rtmps_url(&self, live_input: &LiveInput) -> Option<String> {
        live_input
            .rtmps
            .as_ref()
            .and_then(|r| {
                r.stream_key.as_ref().map(|key| {
                    format!("rtmps://{}:443/{}", r.url, key)
                })
            })
    }

    /// Get SRT URL for a live input
    pub fn get_srt_url(&self, live_input: &LiveInput) -> Option<String> {
        live_input
            .srt
            .as_ref()
            .map(|s| {
                // SRT URL format: srt://host?streamid=xxx&passphrase=yyy
                let mut url = format!("srt://{}", s.url);
                let mut params = vec![];
                if let Some(ref stream_id) = s.stream_id {
                    params.push(format!("streamid={}", stream_id));
                }
                if let Some(ref passphrase) = s.passphrase {
                    params.push(format!("passphrase={}", passphrase));
                }
                if !params.is_empty() {
                    url.push_str("?");
                    url.push_str(&params.join("&"));
                }
                url
            })
    }

    /// Get WebRTC URL for a live input
    pub fn get_webrtc_url(&self, live_input: &LiveInput) -> Option<String> {
        live_input.webrtc.as_ref().map(|w| w.url.clone())
    }

    // =========================================================================
    // Statistics
    // =========================================================================

    /// Get stream statistics
    pub async fn get_stats(&self) -> CloudflareResult<StreamStats> {
        let videos = self.list_videos().await?;
        let live_inputs = self.list_live_inputs().await?;

        let total_duration: u64 = videos
            .iter()
            .filter_map(|v| v.duration)
            .map(|d| d as u64)
            .sum();

        let total_size: u64 = videos
            .iter()
            .filter_map(|v| v.size)
            .map(|s| s.max(0) as u64)
            .sum();

        let ready_count = videos.iter().filter(|v| v.ready_to_stream).count();

        Ok(StreamStats {
            total_videos: videos.len(),
            total_duration_seconds: total_duration,
            total_storage_bytes: total_size,
            ready_videos: ready_count,
            processing_videos: videos.len() - ready_count,
            live_inputs: live_inputs.len(),
            active_live_streams: live_inputs.iter().filter(|l| l.status.as_deref() == Some("connected")).count(),
        })
    }

    /// Get videos by status
    pub async fn get_videos_by_status(&self, ready: bool) -> CloudflareResult<Vec<StreamVideo>> {
        let videos = self.list_videos().await?;
        Ok(videos.into_iter().filter(|v| v.ready_to_stream == ready).collect())
    }

    /// Search videos by name
    pub async fn search_videos(&self, query: &str) -> CloudflareResult<Vec<StreamVideo>> {
        let videos = self.list_videos().await?;
        let query_lower = query.to_lowercase();
        Ok(videos
            .into_iter()
            .filter(|v| {
                v.meta
                    .as_ref()
                    .and_then(|m| m.get("name"))
                    .and_then(|n| n.as_str())
                    .map(|n| n.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
            })
            .collect())
    }
}

/// Options for video embed code generation
#[derive(Debug, Default)]
pub struct EmbedOptions {
    pub autoplay: bool,
    pub muted: bool,
    pub loop_video: bool,
    pub controls: bool,
    pub poster_time: Option<u32>,
    pub start_time: Option<u32>,
    pub width: Option<String>,
    pub height: Option<String>,
}

impl EmbedOptions {
    pub fn new() -> Self {
        Self {
            controls: true,
            ..Default::default()
        }
    }

    pub fn autoplay(mut self) -> Self {
        self.autoplay = true;
        self.muted = true; // Autoplay requires muted
        self
    }

    pub fn muted(mut self) -> Self {
        self.muted = true;
        self
    }

    pub fn loop_video(mut self) -> Self {
        self.loop_video = true;
        self
    }

    pub fn no_controls(mut self) -> Self {
        self.controls = false;
        self
    }

    pub fn poster_at(mut self, seconds: u32) -> Self {
        self.poster_time = Some(seconds);
        self
    }

    pub fn start_at(mut self, seconds: u32) -> Self {
        self.start_time = Some(seconds);
        self
    }

    pub fn dimensions(mut self, width: &str, height: &str) -> Self {
        self.width = Some(width.to_string());
        self.height = Some(height.to_string());
        self
    }
}
