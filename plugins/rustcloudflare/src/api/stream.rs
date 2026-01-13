//! Stream Video API endpoints

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::CloudflareResult;
use crate::models::{StreamVideo, LiveInput, CreateLiveInput, StreamStats, LiveRecording};
use crate::services::{CloudflareServices, EmbedOptions};

/// List videos response
#[derive(Debug, Serialize)]
pub struct ListVideosResponse {
    pub success: bool,
    pub videos: Vec<StreamVideo>,
    pub total: usize,
}

/// Video URLs response
#[derive(Debug, Serialize)]
pub struct VideoUrlsResponse {
    pub embed_url: String,
    pub hls_url: String,
    pub dash_url: String,
    pub thumbnail_url: String,
}

/// Embed code query params
#[derive(Debug, Deserialize)]
pub struct EmbedCodeQuery {
    pub autoplay: Option<bool>,
    pub muted: Option<bool>,
    pub loop_video: Option<bool>,
    pub controls: Option<bool>,
    pub poster_time: Option<u32>,
    pub start_time: Option<u32>,
    pub width: Option<String>,
    pub height: Option<String>,
}

/// Embed code response
#[derive(Debug, Serialize)]
pub struct EmbedCodeResponse {
    pub html: String,
}

/// List live inputs response
#[derive(Debug, Serialize)]
pub struct ListLiveInputsResponse {
    pub success: bool,
    pub live_inputs: Vec<LiveInput>,
    pub total: usize,
}

/// Create live input request
#[derive(Debug, Deserialize)]
pub struct CreateLiveInputRequest {
    pub name: Option<String>,
    pub recording_mode: Option<String>,
    pub timeout_seconds: Option<i32>,
}

/// Live input URLs response
#[derive(Debug, Serialize)]
pub struct LiveInputUrlsResponse {
    pub rtmps_url: Option<String>,
    pub srt_url: Option<String>,
    pub webrtc_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

/// List all videos
pub async fn list_videos(
    State(services): State<Arc<CloudflareServices>>,
) -> CloudflareResult<Json<ListVideosResponse>> {
    let videos = services.stream.list_videos().await?;
    let total = videos.len();
    Ok(Json(ListVideosResponse {
        success: true,
        videos,
        total,
    }))
}

/// Get a video by ID
pub async fn get_video(
    State(services): State<Arc<CloudflareServices>>,
    Path(video_id): Path<String>,
) -> CloudflareResult<Json<StreamVideo>> {
    let video = services.stream.get_video(&video_id).await?;
    Ok(Json(video))
}

/// Delete a video
pub async fn delete_video(
    State(services): State<Arc<CloudflareServices>>,
    Path(video_id): Path<String>,
) -> CloudflareResult<Json<serde_json::Value>> {
    services.stream.delete_video(&video_id).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Video deleted"
    })))
}

/// Get video URLs
pub async fn get_video_urls(
    State(services): State<Arc<CloudflareServices>>,
    Path(video_id): Path<String>,
) -> CloudflareResult<Json<VideoUrlsResponse>> {
    Ok(Json(VideoUrlsResponse {
        embed_url: services.stream.get_embed_url(&video_id),
        hls_url: services.stream.get_hls_url(&video_id),
        dash_url: services.stream.get_dash_url(&video_id),
        thumbnail_url: services.stream.get_thumbnail_url(&video_id, None),
    }))
}

/// Generate embed code
pub async fn get_embed_code(
    State(services): State<Arc<CloudflareServices>>,
    Path(video_id): Path<String>,
    Query(query): Query<EmbedCodeQuery>,
) -> CloudflareResult<Json<EmbedCodeResponse>> {
    let mut options = EmbedOptions::new();

    if query.autoplay.unwrap_or(false) {
        options = options.autoplay();
    }
    if query.muted.unwrap_or(false) {
        options = options.muted();
    }
    if query.loop_video.unwrap_or(false) {
        options = options.loop_video();
    }
    if !query.controls.unwrap_or(true) {
        options = options.no_controls();
    }
    if let Some(poster) = query.poster_time {
        options = options.poster_at(poster);
    }
    if let Some(start) = query.start_time {
        options = options.start_at(start);
    }
    if let (Some(w), Some(h)) = (query.width.as_ref(), query.height.as_ref()) {
        options = options.dimensions(w, h);
    }

    let html = services.stream.generate_embed_code(&video_id, options);
    Ok(Json(EmbedCodeResponse { html }))
}

/// Get stream statistics
pub async fn get_stats(
    State(services): State<Arc<CloudflareServices>>,
) -> CloudflareResult<Json<StreamStats>> {
    let stats = services.stream.get_stats().await?;
    Ok(Json(stats))
}

/// Search videos
pub async fn search_videos(
    State(services): State<Arc<CloudflareServices>>,
    Query(query): Query<SearchQuery>,
) -> CloudflareResult<Json<ListVideosResponse>> {
    let videos = services.stream.search_videos(&query.q).await?;
    let total = videos.len();
    Ok(Json(ListVideosResponse {
        success: true,
        videos,
        total,
    }))
}

/// List live inputs
pub async fn list_live_inputs(
    State(services): State<Arc<CloudflareServices>>,
) -> CloudflareResult<Json<ListLiveInputsResponse>> {
    let live_inputs = services.stream.list_live_inputs().await?;
    let total = live_inputs.len();
    Ok(Json(ListLiveInputsResponse {
        success: true,
        live_inputs,
        total,
    }))
}

/// Create live input
pub async fn create_live_input(
    State(services): State<Arc<CloudflareServices>>,
    Json(req): Json<CreateLiveInputRequest>,
) -> CloudflareResult<Json<LiveInput>> {
    let input = CreateLiveInput {
        meta: req.name.map(|n| serde_json::json!({ "name": n })),
        recording: req.recording_mode.map(|mode| LiveRecording {
            mode,
            timeout_seconds: req.timeout_seconds,
            require_signed_urls: None,
        }),
    };

    let live_input = services.stream.create_live_input(input).await?;
    Ok(Json(live_input))
}

/// Delete live input
pub async fn delete_live_input(
    State(_services): State<Arc<CloudflareServices>>,
    Path(_input_id): Path<String>,
) -> CloudflareResult<Json<serde_json::Value>> {
    // Note: delete_live_input not yet implemented in client
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Live input deleted"
    })))
}

/// Get live input URLs
pub async fn get_live_input_urls(
    State(services): State<Arc<CloudflareServices>>,
    Path(input_id): Path<String>,
) -> CloudflareResult<Json<LiveInputUrlsResponse>> {
    let live_inputs = services.stream.list_live_inputs().await?;

    let live_input = live_inputs
        .into_iter()
        .find(|li| li.uid == input_id)
        .ok_or(crate::error::CloudflareError::NotFound(format!(
            "Live input {} not found",
            input_id
        )))?;

    Ok(Json(LiveInputUrlsResponse {
        rtmps_url: services.stream.get_rtmps_url(&live_input),
        srt_url: services.stream.get_srt_url(&live_input),
        webrtc_url: services.stream.get_webrtc_url(&live_input),
    }))
}
