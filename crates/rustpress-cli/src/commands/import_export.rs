//! WordPress Import/Export commands
//!
//! Provides functionality to import from and export to WordPress WXR format.

use clap::{Args, Subcommand};
use colored::Colorize;
use serde::{Deserialize, Serialize};

use dialoguer::Confirm;

use crate::context::CliContext;
use crate::error::{CliError, CliResult};
use crate::output::{print_header, print_kv, ProgressBar};

#[derive(Args, Debug)]
pub struct ImportExportCommand {
    #[command(subcommand)]
    pub command: ImportExportSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum ImportExportSubcommand {
    /// Import from WordPress WXR (XML) file
    Import {
        /// Path to the WXR file
        file: String,

        /// Import posts
        #[arg(long, default_value = "true")]
        posts: bool,

        /// Import pages
        #[arg(long, default_value = "true")]
        pages: bool,

        /// Import media (download attachments)
        #[arg(long)]
        media: bool,

        /// Import users
        #[arg(long)]
        users: bool,

        /// Dry run - don't actually import
        #[arg(long)]
        dry_run: bool,
    },

    /// Export to WordPress WXR (XML) format
    Export {
        /// Output file path
        #[arg(short, long)]
        output: Option<String>,

        /// Export posts
        #[arg(long, default_value = "true")]
        posts: bool,

        /// Export pages
        #[arg(long, default_value = "true")]
        pages: bool,

        /// Export media references
        #[arg(long)]
        media: bool,

        /// Export users
        #[arg(long)]
        users: bool,

        /// Export only published content
        #[arg(long)]
        published_only: bool,
    },

    /// Analyze a WXR file without importing
    Analyze {
        /// Path to the WXR file
        file: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct WxrAnalysis {
    pub posts: usize,
    pub pages: usize,
    pub attachments: usize,
    pub users: usize,
    pub categories: usize,
    pub tags: usize,
    pub comments: usize,
}

pub async fn execute(ctx: &CliContext, cmd: ImportExportCommand) -> CliResult<()> {
    match cmd.command {
        ImportExportSubcommand::Import { file, posts, pages, media, users, dry_run } => {
            import_wxr(ctx, &file, posts, pages, media, users, dry_run).await
        }
        ImportExportSubcommand::Export { output, posts, pages, media, users, published_only } => {
            export_wxr(ctx, output, posts, pages, media, users, published_only).await
        }
        ImportExportSubcommand::Analyze { file } => {
            analyze_wxr(&file).await
        }
    }
}

async fn import_wxr(
    ctx: &CliContext,
    file: &str,
    import_posts: bool,
    import_pages: bool,
    import_media: bool,
    import_users: bool,
    dry_run: bool,
) -> CliResult<()> {
    print_header("WordPress Import");

    // Check file exists
    let path = std::path::Path::new(file);
    if !path.exists() {
        return Err(CliError::NotFound(format!("File not found: {}", file)));
    }

    print_kv("File", file);
    print_kv("Dry Run", if dry_run { "Yes" } else { "No" });
    println!();

    // Analyze file first
    let analysis = analyze_wxr_file(file)?;

    println!("{}", "Content found:".cyan());
    if import_posts {
        println!("  {} {}", "Posts:".green(), analysis.posts);
    }
    if import_pages {
        println!("  {} {}", "Pages:".green(), analysis.pages);
    }
    if import_media {
        println!("  {} {}", "Attachments:".green(), analysis.attachments);
    }
    if import_users {
        println!("  {} {}", "Users:".green(), analysis.users);
    }
    println!("  {} {}", "Categories:".green(), analysis.categories);
    println!("  {} {}", "Tags:".green(), analysis.tags);
    println!();

    if dry_run {
        println!("{}", "Dry run complete. No changes were made.".yellow());
        return Ok(());
    }

    // Confirm import
    let confirmed = Confirm::new()
        .with_prompt("Proceed with import?")
        .default(false)
        .interact()
        .map_err(|e| CliError::InvalidInput(format!("Failed to get confirmation: {}", e)))?;

    if !confirmed {
        println!("{}", "Import cancelled.".yellow());
        return Ok(());
    }

    let spinner = ProgressBar::spinner("Importing content...");

    // Call API to import
    let client = ctx.http_client();
    let token = ctx.require_auth()?;

    // Read file content
    let content = std::fs::read_to_string(file)?;

    let url = format!("{}/api/v1/import/wordpress", ctx.server_url());

    #[derive(Serialize)]
    struct ImportRequest {
        content: String,
        import_posts: bool,
        import_pages: bool,
        import_media: bool,
        import_users: bool,
    }

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&ImportRequest {
            content,
            import_posts,
            import_pages,
            import_media,
            import_users,
        })
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to import: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            // API endpoint doesn't exist, show local import message
            println!();
            println!("{}", "Note: WordPress import API is not available.".yellow());
            println!("The WXR file has been analyzed. To import the content,");
            println!("consider using the web admin interface or a future CLI version");
            println!("with direct database access.");
            return Ok(());
        }
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!("Import failed ({}): {}", status, body)));
    }

    #[derive(Deserialize)]
    struct ImportResult {
        posts_imported: usize,
        pages_imported: usize,
        media_imported: usize,
        users_imported: usize,
    }

    let result: ImportResult = response.json().await
        .map_err(|e| CliError::Serialization(format!("Failed to parse response: {}", e)))?;

    println!();
    println!("{}", "Import complete!".green().bold());
    println!("  {} {}", "Posts imported:".green(), result.posts_imported);
    println!("  {} {}", "Pages imported:".green(), result.pages_imported);
    println!("  {} {}", "Media imported:".green(), result.media_imported);
    println!("  {} {}", "Users imported:".green(), result.users_imported);

    Ok(())
}

async fn export_wxr(
    ctx: &CliContext,
    output: Option<String>,
    export_posts: bool,
    export_pages: bool,
    export_media: bool,
    export_users: bool,
    published_only: bool,
) -> CliResult<()> {
    print_header("WordPress Export");

    let output_path = output.unwrap_or_else(|| {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        format!("rustpress_export_{}.xml", timestamp)
    });

    print_kv("Output", &output_path);
    print_kv("Posts", if export_posts { "Yes" } else { "No" });
    print_kv("Pages", if export_pages { "Yes" } else { "No" });
    print_kv("Media", if export_media { "Yes" } else { "No" });
    print_kv("Users", if export_users { "Yes" } else { "No" });
    print_kv("Published Only", if published_only { "Yes" } else { "No" });
    println!();

    let spinner = ProgressBar::spinner("Exporting content...");

    let client = ctx.http_client();
    let token = ctx.require_auth()?;

    let url = format!(
        "{}/api/v1/export/wordpress?posts={}&pages={}&media={}&users={}&published_only={}",
        ctx.server_url(),
        export_posts,
        export_pages,
        export_media,
        export_users,
        published_only
    );

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to export: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            println!();
            println!("{}", "Note: WordPress export API is not available yet.".yellow());
            println!("This feature will be available in a future release.");
            return Ok(());
        }
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!("Export failed ({}): {}", status, body)));
    }

    let content = response.text().await
        .map_err(|e| CliError::Network(format!("Failed to read response: {}", e)))?;

    // Write to file
    std::fs::write(&output_path, &content)?;

    let file_size = std::fs::metadata(&output_path)?.len();
    let size_str = if file_size >= 1024 * 1024 {
        format!("{:.2} MB", file_size as f64 / (1024.0 * 1024.0))
    } else if file_size >= 1024 {
        format!("{:.2} KB", file_size as f64 / 1024.0)
    } else {
        format!("{} bytes", file_size)
    };

    println!();
    println!("{}", "Export complete!".green().bold());
    println!("  {} {}", "File:".green(), output_path);
    println!("  {} {}", "Size:".green(), size_str);

    Ok(())
}

async fn analyze_wxr(file: &str) -> CliResult<()> {
    print_header("WXR File Analysis");

    let path = std::path::Path::new(file);
    if !path.exists() {
        return Err(CliError::NotFound(format!("File not found: {}", file)));
    }

    let file_size = std::fs::metadata(path)?.len();
    let size_str = if file_size >= 1024 * 1024 {
        format!("{:.2} MB", file_size as f64 / (1024.0 * 1024.0))
    } else if file_size >= 1024 {
        format!("{:.2} KB", file_size as f64 / 1024.0)
    } else {
        format!("{} bytes", file_size)
    };

    print_kv("File", file);
    print_kv("Size", &size_str);
    println!();

    let analysis = analyze_wxr_file(file)?;

    println!("{}", "Content Summary:".cyan().bold());
    println!();
    println!("  {} {}", "Posts:".green().bold(), analysis.posts);
    println!("  {} {}", "Pages:".green().bold(), analysis.pages);
    println!("  {} {}", "Attachments:".green().bold(), analysis.attachments);
    println!("  {} {}", "Users/Authors:".green().bold(), analysis.users);
    println!("  {} {}", "Categories:".green().bold(), analysis.categories);
    println!("  {} {}", "Tags:".green().bold(), analysis.tags);
    println!("  {} {}", "Comments:".green().bold(), analysis.comments);

    Ok(())
}

fn analyze_wxr_file(file: &str) -> CliResult<WxrAnalysis> {
    let content = std::fs::read_to_string(file)?;

    let mut analysis = WxrAnalysis::default();

    // Simple pattern matching for WXR content types
    // This is a basic implementation - a full parser would use quick-xml properly

    // Count items by post_type
    for line in content.lines() {
        let line = line.trim();
        if line.contains("<wp:post_type>") {
            if line.contains("post") && !line.contains("attachment") {
                analysis.posts += 1;
            } else if line.contains("page") {
                analysis.pages += 1;
            } else if line.contains("attachment") {
                analysis.attachments += 1;
            }
        } else if line.contains("<wp:author>") {
            analysis.users += 1;
        } else if line.contains("<wp:category>") || line.contains("<category domain=\"category\"") {
            analysis.categories += 1;
        } else if line.contains("<wp:tag>") || line.contains("<category domain=\"post_tag\"") {
            analysis.tags += 1;
        } else if line.contains("<wp:comment>") {
            analysis.comments += 1;
        }
    }

    // Deduplicate category/tag counts (they appear in both places)
    analysis.categories = (analysis.categories + 1) / 2;
    analysis.tags = (analysis.tags + 1) / 2;

    Ok(analysis)
}
