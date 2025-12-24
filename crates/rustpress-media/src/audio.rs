//! Audio handling and player support
//!
//! Provides audio functionality including:
//! - Metadata extraction
//! - Waveform generation
//! - Audio player HTML generation

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{MediaError, MediaResult};

/// Audio metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMetadata {
    pub id: Uuid,
    pub media_id: Uuid,
    pub codec: Option<String>,
    pub bitrate: Option<i32>,
    pub sample_rate: Option<i32>,
    pub channels: Option<i32>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
    pub genre: Option<String>,
    pub year: Option<i32>,
    pub cover_art_url: Option<String>,
    pub waveform_data: Option<Vec<f32>>,
    pub created_at: DateTime<Utc>,
}

/// Audio service
pub struct AudioService {
    pool: PgPool,
}

impl AudioService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get audio metadata
    pub async fn get_metadata(&self, media_id: Uuid) -> MediaResult<AudioMetadata> {
        let row = sqlx::query!(
            r#"
            SELECT id, media_id, codec, bitrate, sample_rate, channels,
                   artist, album, title, genre, year, cover_art_url,
                   waveform_data, created_at
            FROM audio_metadata
            WHERE media_id = $1
            "#,
            media_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(MediaError::NotFound(media_id))?;

        let waveform: Option<Vec<f32>> = row.waveform_data
            .and_then(|v| serde_json::from_value(v).ok());

        Ok(AudioMetadata {
            id: row.id,
            media_id: row.media_id,
            codec: row.codec,
            bitrate: row.bitrate,
            sample_rate: row.sample_rate,
            channels: row.channels,
            artist: row.artist,
            album: row.album,
            title: row.title,
            genre: row.genre,
            year: row.year,
            cover_art_url: row.cover_art_url,
            waveform_data: waveform,
            created_at: row.created_at,
        })
    }

    /// Generate simple audio player HTML
    pub fn generate_player_html(
        &self,
        audio_url: &str,
        controls: bool,
        autoplay: bool,
        loop_audio: bool,
        preload: &str,
    ) -> String {
        let mut attrs = vec![format!("src=\"{}\"", audio_url)];

        if controls {
            attrs.push("controls".to_string());
        }
        if autoplay {
            attrs.push("autoplay".to_string());
        }
        if loop_audio {
            attrs.push("loop".to_string());
        }
        attrs.push(format!("preload=\"{}\"", preload));

        format!("<audio {}></audio>", attrs.join(" "))
    }

    /// Generate styled audio player with metadata
    pub fn generate_styled_player(
        &self,
        audio_url: &str,
        metadata: &AudioMetadata,
        player_id: &str,
    ) -> String {
        let title = metadata.title.as_deref().unwrap_or("Unknown Title");
        let artist = metadata.artist.as_deref().unwrap_or("Unknown Artist");
        let cover = metadata.cover_art_url.as_deref().unwrap_or("/images/default-cover.png");

        format!(
            r#"
<div class="audio-player" id="{id}">
    <div class="audio-player-cover">
        <img src="{cover}" alt="{title}">
    </div>
    <div class="audio-player-info">
        <div class="audio-player-title">{title}</div>
        <div class="audio-player-artist">{artist}</div>
    </div>
    <div class="audio-player-controls">
        <button class="audio-play-btn" aria-label="Play">
            <svg viewBox="0 0 24 24"><path d="M8 5v14l11-7z"/></svg>
        </button>
        <div class="audio-progress">
            <div class="audio-progress-bar"></div>
            <div class="audio-waveform" data-waveform='{waveform}'></div>
        </div>
        <div class="audio-time">
            <span class="audio-current">0:00</span>
            <span class="audio-duration">0:00</span>
        </div>
        <button class="audio-volume-btn" aria-label="Volume">
            <svg viewBox="0 0 24 24"><path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02z"/></svg>
        </button>
        <input type="range" class="audio-volume-slider" min="0" max="1" step="0.1" value="1">
    </div>
    <audio src="{url}" preload="metadata"></audio>
</div>
"#,
            id = player_id,
            cover = cover,
            title = title,
            artist = artist,
            url = audio_url,
            waveform = serde_json::to_string(&metadata.waveform_data.clone().unwrap_or_default()).unwrap_or_else(|_| "[]".to_string())
        )
    }

    /// Generate audio player CSS
    pub fn generate_player_css() -> &'static str {
        r#"
.audio-player {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem;
    background: #1f2937;
    border-radius: 0.5rem;
    color: white;
}

.audio-player-cover img {
    width: 60px;
    height: 60px;
    border-radius: 0.25rem;
    object-fit: cover;
}

.audio-player-info {
    flex: 0 0 auto;
    min-width: 120px;
}

.audio-player-title {
    font-weight: 600;
    font-size: 0.875rem;
}

.audio-player-artist {
    color: #9ca3af;
    font-size: 0.75rem;
}

.audio-player-controls {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex: 1;
}

.audio-play-btn {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background: #3b82f6;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.2s;
}

.audio-play-btn:hover {
    background: #2563eb;
}

.audio-play-btn svg {
    width: 16px;
    height: 16px;
    fill: white;
}

.audio-progress {
    flex: 1;
    height: 4px;
    background: #4b5563;
    border-radius: 2px;
    cursor: pointer;
    position: relative;
}

.audio-progress-bar {
    height: 100%;
    background: #3b82f6;
    border-radius: 2px;
    width: 0%;
    transition: width 0.1s linear;
}

.audio-waveform {
    position: absolute;
    top: -20px;
    left: 0;
    right: 0;
    height: 40px;
    opacity: 0.3;
}

.audio-time {
    display: flex;
    gap: 0.25rem;
    font-size: 0.75rem;
    color: #9ca3af;
    min-width: 80px;
}

.audio-volume-btn {
    background: none;
    border: none;
    cursor: pointer;
    padding: 0.25rem;
}

.audio-volume-btn svg {
    width: 20px;
    height: 20px;
    fill: #9ca3af;
}

.audio-volume-slider {
    width: 60px;
    height: 4px;
    -webkit-appearance: none;
    background: #4b5563;
    border-radius: 2px;
}

.audio-volume-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 12px;
    height: 12px;
    background: white;
    border-radius: 50%;
    cursor: pointer;
}
"#
    }

    /// Generate audio player JavaScript
    pub fn generate_player_js() -> &'static str {
        r#"
class AudioPlayer {
    constructor(container) {
        this.container = container;
        this.audio = container.querySelector('audio');
        this.playBtn = container.querySelector('.audio-play-btn');
        this.progress = container.querySelector('.audio-progress');
        this.progressBar = container.querySelector('.audio-progress-bar');
        this.currentTime = container.querySelector('.audio-current');
        this.duration = container.querySelector('.audio-duration');
        this.volumeBtn = container.querySelector('.audio-volume-btn');
        this.volumeSlider = container.querySelector('.audio-volume-slider');

        this.bindEvents();
    }

    bindEvents() {
        this.playBtn.addEventListener('click', () => this.togglePlay());

        this.audio.addEventListener('timeupdate', () => this.updateProgress());
        this.audio.addEventListener('loadedmetadata', () => this.updateDuration());
        this.audio.addEventListener('ended', () => this.onEnded());

        this.progress.addEventListener('click', (e) => this.seek(e));

        this.volumeSlider.addEventListener('input', (e) => {
            this.audio.volume = e.target.value;
        });

        this.volumeBtn.addEventListener('click', () => this.toggleMute());
    }

    togglePlay() {
        if (this.audio.paused) {
            this.audio.play();
            this.playBtn.innerHTML = '<svg viewBox="0 0 24 24"><path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/></svg>';
        } else {
            this.audio.pause();
            this.playBtn.innerHTML = '<svg viewBox="0 0 24 24"><path d="M8 5v14l11-7z"/></svg>';
        }
    }

    updateProgress() {
        const percent = (this.audio.currentTime / this.audio.duration) * 100;
        this.progressBar.style.width = percent + '%';
        this.currentTime.textContent = this.formatTime(this.audio.currentTime);
    }

    updateDuration() {
        this.duration.textContent = this.formatTime(this.audio.duration);
    }

    seek(e) {
        const rect = this.progress.getBoundingClientRect();
        const percent = (e.clientX - rect.left) / rect.width;
        this.audio.currentTime = percent * this.audio.duration;
    }

    toggleMute() {
        this.audio.muted = !this.audio.muted;
        this.volumeSlider.value = this.audio.muted ? 0 : this.audio.volume;
    }

    onEnded() {
        this.playBtn.innerHTML = '<svg viewBox="0 0 24 24"><path d="M8 5v14l11-7z"/></svg>';
        this.progressBar.style.width = '0%';
    }

    formatTime(seconds) {
        const mins = Math.floor(seconds / 60);
        const secs = Math.floor(seconds % 60);
        return `${mins}:${secs.toString().padStart(2, '0')}`;
    }
}

// Auto-initialize all players
document.querySelectorAll('.audio-player').forEach(el => {
    new AudioPlayer(el);
});
"#
    }
}

/// Audio player configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioPlayerConfig {
    pub controls: bool,
    pub autoplay: bool,
    pub loop_audio: bool,
    pub preload: String,
    pub show_waveform: bool,
    pub show_cover_art: bool,
    pub show_metadata: bool,
    pub volume_control: bool,
    pub seek_control: bool,
    pub download_button: bool,
    pub share_button: bool,
}

impl Default for AudioPlayerConfig {
    fn default() -> Self {
        Self {
            controls: true,
            autoplay: false,
            loop_audio: false,
            preload: "metadata".to_string(),
            show_waveform: true,
            show_cover_art: true,
            show_metadata: true,
            volume_control: true,
            seek_control: true,
            download_button: false,
            share_button: false,
        }
    }
}

/// Playlist support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: Uuid,
    pub name: String,
    pub tracks: Vec<PlaylistTrack>,
    pub current_index: usize,
    pub shuffle: bool,
    pub repeat_mode: RepeatMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistTrack {
    pub media_id: Uuid,
    pub url: String,
    pub title: String,
    pub artist: Option<String>,
    pub duration: f64,
    pub cover_url: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RepeatMode {
    None,
    One,
    All,
}
