//! Theme management commands

use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use tabled::Tabled;

use crate::context::CliContext;
use crate::error::{CliError, CliResult};
use crate::output::{print_header, print_kv, OutputFormatter, ProgressBar};

#[derive(Args, Debug)]
pub struct ThemesCommand {
    #[command(subcommand)]
    pub command: ThemesSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum ThemesSubcommand {
    /// List installed themes
    List {
        /// Filter by status (active, inactive, all)
        #[arg(short, long, default_value = "all")]
        status: String,
    },

    /// Get theme details
    Get {
        /// Theme ID
        theme: String,
    },

    /// Activate a theme
    Activate {
        /// Theme ID
        theme: String,
    },

    /// Install a theme from path or URL
    Install {
        /// Path to theme directory or ZIP file
        source: String,
    },

    /// Delete a theme
    Delete {
        /// Theme ID
        theme: String,

        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Export a theme as ZIP
    Export {
        /// Theme ID
        theme: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,

        /// Export format (zip, tar)
        #[arg(short, long, default_value = "zip")]
        format: String,
    },

    /// Scan for new themes
    Scan,
}

#[derive(Debug, Serialize, Deserialize, Tabled)]
pub struct ThemeRow {
    #[tabled(rename = "ID")]
    pub id: String,
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Version")]
    pub version: String,
    #[tabled(rename = "Status")]
    pub status: String,
    #[tabled(rename = "Author")]
    pub author: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThemeDetails {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: String,
    pub author_uri: Option<String>,
    pub theme_uri: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_active: bool,
    pub supports_fse: Option<bool>,
    pub screenshot: Option<String>,
}

pub async fn execute(ctx: &CliContext, cmd: ThemesCommand) -> CliResult<()> {
    match cmd.command {
        ThemesSubcommand::List { status } => list_themes(ctx, &status).await,
        ThemesSubcommand::Get { theme } => get_theme(ctx, &theme).await,
        ThemesSubcommand::Activate { theme } => activate_theme(ctx, &theme).await,
        ThemesSubcommand::Install { source } => install_theme(ctx, &source).await,
        ThemesSubcommand::Delete { theme, force } => delete_theme(ctx, &theme, force).await,
        ThemesSubcommand::Export {
            theme,
            output,
            format,
        } => export_theme(ctx, &theme, output, &format).await,
        ThemesSubcommand::Scan => scan_themes(ctx).await,
    }
}

fn auth_header(ctx: &CliContext) -> CliResult<String> {
    let token = ctx.require_auth()?;
    Ok(format!("Bearer {}", token))
}

async fn list_themes(ctx: &CliContext, status: &str) -> CliResult<()> {
    print_header("Installed Themes");

    let client = ctx.http_client();
    let mut url = format!("{}/api/v1/themes", ctx.server_url());

    if status != "all" {
        url.push_str(&format!("?status={}", status));
    }

    let response = client
        .get(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to fetch themes: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to list themes ({}): {}",
            status, body
        )));
    }

    let themes: Vec<ThemeRow> = response
        .json()
        .await
        .map_err(|e| CliError::Serialization(format!("Failed to parse response: {}", e)))?;

    if themes.is_empty() {
        println!("{}", ctx.output_format.info("No themes found. Run 'themes scan' to discover themes."));
    } else {
        println!("{}", ctx.output_format.format(&themes));
    }

    Ok(())
}

async fn get_theme(ctx: &CliContext, theme: &str) -> CliResult<()> {
    let client = ctx.http_client();
    let url = format!("{}/api/v1/themes/{}", ctx.server_url(), theme);

    let response = client
        .get(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to fetch theme: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(CliError::NotFound(format!("Theme not found: {}", theme)));
        }
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to get theme ({}): {}",
            status, body
        )));
    }

    let details: ThemeDetails = response
        .json()
        .await
        .map_err(|e| CliError::Serialization(format!("Failed to parse response: {}", e)))?;

    print_header(&format!("Theme: {}", details.name));
    print_kv("ID", &details.id);
    print_kv("Version", &details.version);
    print_kv("Author", &details.author);
    print_kv("Status", if details.is_active { "Active" } else { "Inactive" });
    if let Some(ref desc) = details.description {
        print_kv("Description", desc);
    }
    if let Some(ref uri) = details.theme_uri {
        print_kv("Theme URI", uri);
    }
    if let Some(ref author_uri) = details.author_uri {
        print_kv("Author URI", author_uri);
    }
    if let Some(ref tags) = details.tags {
        print_kv("Tags", &tags.join(", "));
    }
    if let Some(fse) = details.supports_fse {
        print_kv("Full Site Editing", if fse { "Yes" } else { "No" });
    }

    Ok(())
}

async fn activate_theme(ctx: &CliContext, theme: &str) -> CliResult<()> {
    print_header("Activating Theme");

    let spinner = ProgressBar::spinner("Activating theme...");

    let client = ctx.http_client();
    let url = format!("{}/api/v1/themes/{}/activate", ctx.server_url(), theme);

    let response = client
        .post(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to activate theme: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to activate theme ({}): {}",
            status, body
        )));
    }

    println!("{}", ctx.output_format.success(&format!("Theme '{}' activated", theme)));
    Ok(())
}

async fn install_theme(ctx: &CliContext, source: &str) -> CliResult<()> {
    print_header("Installing Theme");
    print_kv("Source", source);

    let spinner = ProgressBar::spinner("Installing theme...");

    // Check if source is a local path
    if std::path::Path::new(source).exists() {
        let theme_id = std::path::Path::new(source)
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("theme")
            .to_string();

        let client = ctx.http_client();
        let url = format!("{}/api/v1/themes", ctx.server_url());

        let body = serde_json::json!({
            "theme_id": theme_id,
            "source": source,
        });

        let response = client
            .post(&url)
            .header("Authorization", auth_header(ctx)?)
            .json(&body)
            .send()
            .await
            .map_err(|e| CliError::Network(format!("Failed to install theme: {}", e)))?;

        spinner.finish_and_clear();

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::OperationFailed(format!(
                "Failed to install theme ({}): {}",
                status, body
            )));
        }

        println!();
        println!("{}", ctx.output_format.success(&format!("Theme '{}' installed", theme_id)));
    } else {
        // URL-based install
        let client = ctx.http_client();
        let url = format!("{}/api/v1/themes/install", ctx.server_url());

        let body = serde_json::json!({
            "url": source,
        });

        let response = client
            .post(&url)
            .header("Authorization", auth_header(ctx)?)
            .json(&body)
            .send()
            .await
            .map_err(|e| CliError::Network(format!("Failed to install theme: {}", e)))?;

        spinner.finish_and_clear();

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::OperationFailed(format!(
                "Failed to install theme ({}): {}",
                status, body
            )));
        }

        println!();
        println!("{}", ctx.output_format.success("Theme installed"));
    }

    Ok(())
}

async fn delete_theme(ctx: &CliContext, theme: &str, force: bool) -> CliResult<()> {
    // First check if theme is active
    let client = ctx.http_client();
    let url = format!("{}/api/v1/themes/{}", ctx.server_url(), theme);

    let response = client
        .get(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to fetch theme: {}", e)))?;

    if response.status().is_success() {
        if let Ok(details) = response.json::<ThemeDetails>().await {
            if details.is_active {
                return Err(CliError::OperationFailed(
                    "Cannot delete the active theme. Activate another theme first.".to_string(),
                ));
            }
        }
    }

    if !force {
        println!("This will permanently delete the theme files.");
        println!("Run with --force to confirm.");
        return Ok(());
    }

    print_header("Deleting Theme");

    let spinner = ProgressBar::spinner("Deleting theme...");

    let delete_url = format!("{}/api/v1/themes/{}", ctx.server_url(), theme);

    let response = client
        .delete(&delete_url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to delete theme: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to delete theme ({}): {}",
            status, body
        )));
    }

    println!("{}", ctx.output_format.success(&format!("Theme '{}' deleted", theme)));
    Ok(())
}

async fn export_theme(
    ctx: &CliContext,
    theme: &str,
    output: Option<String>,
    format: &str,
) -> CliResult<()> {
    print_header("Exporting Theme");

    let extension = match format {
        "tar" => "tar.gz",
        _ => "zip",
    };

    let output_file = output.unwrap_or_else(|| format!("{}.{}", theme, extension));

    print_kv("Theme", theme);
    print_kv("Format", format);
    print_kv("Output", &output_file);

    let spinner = ProgressBar::spinner("Exporting theme...");

    let client = ctx.http_client();
    let url = format!("{}/api/v1/themes/{}/export?format={}", ctx.server_url(), theme, format);

    let response = client
        .get(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to export theme: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        let status = response.status();

        // If API doesn't support export, provide manual instructions
        if status == reqwest::StatusCode::NOT_FOUND || status == reqwest::StatusCode::NOT_IMPLEMENTED {
            println!();
            println!(
                "{}",
                ctx.output_format.info(&format!(
                    "Theme export not available via API.\nTo export manually, archive the theme directory."
                ))
            );
            return Ok(());
        }

        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to export theme ({}): {}",
            status, body
        )));
    }

    // Save the response body to file
    let bytes = response.bytes().await
        .map_err(|e| CliError::Network(format!("Failed to download theme: {}", e)))?;

    std::fs::write(&output_file, &bytes)?;

    println!();
    println!("{}", ctx.output_format.success(&format!("Theme exported to {}", output_file)));

    Ok(())
}

async fn scan_themes(ctx: &CliContext) -> CliResult<()> {
    print_header("Scanning for Themes");

    let spinner = ProgressBar::spinner("Scanning themes directory...");

    let client = ctx.http_client();
    let url = format!("{}/api/v1/themes/scan", ctx.server_url());

    let response = client
        .post(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to scan themes: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to scan themes ({}): {}",
            status, body
        )));
    }

    let result: serde_json::Value = response
        .json()
        .await
        .unwrap_or(serde_json::json!({"found": 0, "registered": 0}));

    let found = result.get("found").and_then(|v| v.as_i64()).unwrap_or(0);
    let registered = result.get("registered").and_then(|v| v.as_i64()).unwrap_or(0);

    println!();
    println!(
        "{}",
        ctx.output_format.success(&format!(
            "Found {} theme(s), registered {} new theme(s)",
            found, registered
        ))
    );

    Ok(())
}
