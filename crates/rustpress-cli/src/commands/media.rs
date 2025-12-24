//! Media management commands

use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use tabled::Tabled;

use crate::context::CliContext;
use crate::error::{CliError, CliResult};
use crate::output::{print_header, print_kv, OutputFormatter, ProgressBar};

#[derive(Args, Debug)]
pub struct MediaCommand {
    #[command(subcommand)]
    pub command: MediaSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum MediaSubcommand {
    /// List media files
    List {
        /// Filter by MIME type (image, video, audio, application)
        #[arg(short, long)]
        r#type: Option<String>,

        /// Maximum number of results
        #[arg(short, long, default_value = "50")]
        limit: u32,
    },

    /// Upload a media file
    Upload {
        /// File to upload
        file: String,

        /// Title for the media
        #[arg(short, long)]
        title: Option<String>,

        /// Alt text for images
        #[arg(short, long)]
        alt: Option<String>,
    },

    /// Get media details
    Get {
        /// Media ID
        id: String,
    },

    /// Delete media
    Delete {
        /// Media ID
        id: String,

        /// Also delete the file from storage
        #[arg(long)]
        delete_file: bool,
    },

    /// Optimize media files
    Optimize {
        /// Optimize all media
        #[arg(long)]
        all: bool,

        /// Specific media ID to optimize
        id: Option<String>,
    },

    /// Regenerate thumbnails
    RegenerateThumbnails {
        /// Regenerate for all media
        #[arg(long)]
        all: bool,

        /// Specific media ID
        id: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize, Tabled)]
pub struct MediaRow {
    #[tabled(rename = "ID")]
    pub id: String,
    #[tabled(rename = "Filename")]
    pub filename: String,
    #[tabled(rename = "Type")]
    pub mime_type: String,
    #[tabled(rename = "Size")]
    pub size: String,
    #[tabled(rename = "Uploaded")]
    pub uploaded_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaDetails {
    pub id: String,
    pub filename: String,
    pub mime_type: String,
    pub file_size: i64,
    pub url: String,
    pub alt_text: Option<String>,
    pub title: Option<String>,
    pub caption: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub uploaded_at: String,
}

pub async fn execute(ctx: &CliContext, cmd: MediaCommand) -> CliResult<()> {
    match cmd.command {
        MediaSubcommand::List { r#type, limit } => list_media(ctx, r#type, limit).await,
        MediaSubcommand::Upload { file, title, alt } => upload_media(ctx, &file, title, alt).await,
        MediaSubcommand::Get { id } => get_media(ctx, &id).await,
        MediaSubcommand::Delete { id, delete_file } => delete_media(ctx, &id, delete_file).await,
        MediaSubcommand::Optimize { all, id } => optimize_media(ctx, all, id).await,
        MediaSubcommand::RegenerateThumbnails { all, id } => {
            regenerate_thumbnails(ctx, all, id).await
        }
    }
}

fn auth_header(ctx: &CliContext) -> CliResult<String> {
    let token = ctx.require_auth()?;
    Ok(format!("Bearer {}", token))
}

async fn list_media(ctx: &CliContext, mime_type: Option<String>, limit: u32) -> CliResult<()> {
    print_header("Media Library");

    let client = ctx.http_client();
    let mut url = format!("{}/api/v1/media?limit={}", ctx.server_url(), limit);

    if let Some(ref t) = mime_type {
        url.push_str(&format!("&type={}", t));
    }

    let response = client
        .get(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to fetch media: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to list media ({}): {}",
            status, body
        )));
    }

    let media: Vec<MediaRow> = response
        .json()
        .await
        .map_err(|e| CliError::Serialization(format!("Failed to parse response: {}", e)))?;

    if media.is_empty() {
        println!("{}", ctx.output_format.info("No media found"));
    } else {
        println!("{}", ctx.output_format.format(&media));
        println!();
        println!("Total: {} file(s)", media.len());
    }

    Ok(())
}

async fn upload_media(
    ctx: &CliContext,
    file: &str,
    title: Option<String>,
    alt: Option<String>,
) -> CliResult<()> {
    print_header("Uploading Media");

    // Check if file exists
    let path = std::path::Path::new(file);
    if !path.exists() {
        return Err(CliError::NotFound(format!("File not found: {}", file)));
    }

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    let file_size = std::fs::metadata(path)?.len();

    // Detect MIME type from extension
    let mime_type = match path.extension().and_then(|e| e.to_str()) {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("svg") => "image/svg+xml",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        Some("pdf") => "application/pdf",
        Some("doc") | Some("docx") => "application/msword",
        _ => "application/octet-stream",
    };

    print_kv("File", filename);
    print_kv("Size", &format_size(file_size));
    print_kv("Type", mime_type);

    let spinner = ProgressBar::spinner("Uploading...");

    // Read file content
    let file_content = std::fs::read(path)?;

    let client = ctx.http_client();
    let url = format!("{}/api/v1/media", ctx.server_url());

    // Create multipart form
    let part = reqwest::multipart::Part::bytes(file_content)
        .file_name(filename.to_string())
        .mime_str(mime_type)
        .map_err(|e| CliError::InvalidInput(format!("Invalid mime type: {}", e)))?;

    let mut form = reqwest::multipart::Form::new()
        .part("file", part);

    if let Some(ref t) = title {
        form = form.text("title", t.clone());
    }
    if let Some(ref a) = alt {
        form = form.text("alt_text", a.clone());
    }

    let response = client
        .post(&url)
        .header("Authorization", auth_header(ctx)?)
        .multipart(form)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to upload media: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to upload media ({}): {}",
            status, body
        )));
    }

    let created: serde_json::Value = response
        .json()
        .await
        .map_err(|e| CliError::Serialization(format!("Failed to parse response: {}", e)))?;

    let id = created.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");

    println!();
    println!(
        "{}",
        ctx.output_format.success(&format!("Media uploaded with ID: {}", id))
    );

    Ok(())
}

async fn get_media(ctx: &CliContext, id: &str) -> CliResult<()> {
    let client = ctx.http_client();
    let url = format!("{}/api/v1/media/{}", ctx.server_url(), id);

    let response = client
        .get(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to fetch media: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(CliError::NotFound(format!("Media not found: {}", id)));
        }
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to get media ({}): {}",
            status, body
        )));
    }

    let details: MediaDetails = response
        .json()
        .await
        .map_err(|e| CliError::Serialization(format!("Failed to parse response: {}", e)))?;

    print_header("Media Details");
    print_kv("ID", &details.id);
    print_kv("Filename", &details.filename);
    print_kv("Type", &details.mime_type);
    print_kv("Size", &format_size(details.file_size as u64));
    print_kv("URL", &details.url);
    if let Some(ref title) = details.title {
        print_kv("Title", title);
    }
    if let Some(ref alt) = details.alt_text {
        print_kv("Alt Text", alt);
    }
    if let (Some(w), Some(h)) = (details.width, details.height) {
        print_kv("Dimensions", &format!("{}x{}", w, h));
    }
    print_kv("Uploaded", &details.uploaded_at);

    Ok(())
}

async fn delete_media(ctx: &CliContext, id: &str, delete_file: bool) -> CliResult<()> {
    print_header("Deleting Media");

    let spinner = ProgressBar::spinner("Deleting media...");

    let client = ctx.http_client();
    let url = if delete_file {
        format!("{}/api/v1/media/{}?delete_file=true", ctx.server_url(), id)
    } else {
        format!("{}/api/v1/media/{}", ctx.server_url(), id)
    };

    let response = client
        .delete(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to delete media: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to delete media ({}): {}",
            status, body
        )));
    }

    println!("{}", ctx.output_format.success("Media deleted"));
    Ok(())
}

async fn optimize_media(ctx: &CliContext, all: bool, id: Option<String>) -> CliResult<()> {
    print_header("Optimizing Media");

    if !all && id.is_none() {
        return Err(CliError::InvalidInput(
            "Specify --all or provide a media ID".to_string(),
        ));
    }

    let spinner = ProgressBar::spinner("Optimizing media...");

    let client = ctx.http_client();
    let url = if all {
        format!("{}/api/v1/media/optimize", ctx.server_url())
    } else {
        format!("{}/api/v1/media/{}/optimize", ctx.server_url(), id.as_ref().unwrap())
    };

    let response = client
        .post(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to optimize media: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        let status = response.status();
        // If endpoint doesn't exist, provide info
        if status == reqwest::StatusCode::NOT_FOUND {
            println!("{}", ctx.output_format.info("Media optimization is not available via API"));
            return Ok(());
        }
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to optimize media ({}): {}",
            status, body
        )));
    }

    let result: serde_json::Value = response.json().await.unwrap_or_default();
    let count = result.get("optimized").and_then(|v| v.as_i64()).unwrap_or(1);

    println!();
    println!(
        "{}",
        ctx.output_format.success(&format!("Optimized {} file(s)", count))
    );

    Ok(())
}

async fn regenerate_thumbnails(ctx: &CliContext, all: bool, id: Option<String>) -> CliResult<()> {
    print_header("Regenerating Thumbnails");

    if !all && id.is_none() {
        return Err(CliError::InvalidInput(
            "Specify --all or provide a media ID".to_string(),
        ));
    }

    let spinner = ProgressBar::spinner("Regenerating thumbnails...");

    let client = ctx.http_client();
    let url = if all {
        format!("{}/api/v1/media/regenerate-thumbnails", ctx.server_url())
    } else {
        format!("{}/api/v1/media/{}/regenerate-thumbnails", ctx.server_url(), id.as_ref().unwrap())
    };

    let response = client
        .post(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to regenerate thumbnails: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        let status = response.status();
        // If endpoint doesn't exist, provide info
        if status == reqwest::StatusCode::NOT_FOUND {
            println!("{}", ctx.output_format.info("Thumbnail regeneration is not available via API"));
            return Ok(());
        }
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to regenerate thumbnails ({}): {}",
            status, body
        )));
    }

    let result: serde_json::Value = response.json().await.unwrap_or_default();
    let count = result.get("regenerated").and_then(|v| v.as_i64()).unwrap_or(1);

    println!();
    println!(
        "{}",
        ctx.output_format.success(&format!("Regenerated thumbnails for {} file(s)", count))
    );

    Ok(())
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
