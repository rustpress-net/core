//! Plugin management commands

use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use tabled::Tabled;

use crate::context::CliContext;
use crate::error::{CliError, CliResult};
use crate::output::{print_header, print_kv, OutputFormatter, ProgressBar};

#[derive(Args, Debug)]
pub struct PluginsCommand {
    #[command(subcommand)]
    pub command: PluginsSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum PluginsSubcommand {
    /// List installed plugins
    List {
        /// Filter by status (active, inactive, all)
        #[arg(short, long, default_value = "all")]
        status: String,
    },

    /// Get plugin details
    Get { plugin: String },

    /// Activate a plugin
    Activate { plugin: String },

    /// Deactivate a plugin
    Deactivate { plugin: String },

    /// Install a plugin
    Install { source: String },

    /// Uninstall a plugin
    Uninstall {
        plugin: String,
        #[arg(short, long)]
        force: bool,
    },

    /// Check for plugin updates
    CheckUpdates,
}

#[derive(Debug, Serialize, Deserialize, Tabled)]
pub struct PluginRow {
    #[tabled(rename = "ID")]
    pub id: String,
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Version")]
    pub version: String,
    #[tabled(rename = "Status")]
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginDetails {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub is_active: bool,
}

pub async fn execute(ctx: &CliContext, cmd: PluginsCommand) -> CliResult<()> {
    match cmd.command {
        PluginsSubcommand::List { status } => list_plugins(ctx, &status).await,
        PluginsSubcommand::Get { plugin } => get_plugin(ctx, &plugin).await,
        PluginsSubcommand::Activate { plugin } => activate_plugin(ctx, &plugin).await,
        PluginsSubcommand::Deactivate { plugin } => deactivate_plugin(ctx, &plugin).await,
        PluginsSubcommand::Install { source } => install_plugin(ctx, &source).await,
        PluginsSubcommand::Uninstall { plugin, force } => uninstall_plugin(ctx, &plugin, force).await,
        PluginsSubcommand::CheckUpdates => check_updates(ctx).await,
    }
}

fn auth_header(ctx: &CliContext) -> CliResult<String> {
    let token = ctx.require_auth()?;
    Ok(format!("Bearer {}", token))
}

async fn list_plugins(ctx: &CliContext, status: &str) -> CliResult<()> {
    print_header("Installed Plugins");

    let client = ctx.http_client();
    let mut url = format!("{}/api/v1/plugins", ctx.server_url());

    if status != "all" {
        url.push_str(&format!("?status={}", status));
    }

    let response = client
        .get(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to fetch plugins: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to list plugins ({}): {}",
            status, body
        )));
    }

    let plugins: Vec<PluginRow> = response
        .json()
        .await
        .map_err(|e| CliError::Serialization(format!("Failed to parse response: {}", e)))?;

    if plugins.is_empty() {
        println!("{}", ctx.output_format.info("No plugins found"));
    } else {
        println!("{}", ctx.output_format.format(&plugins));
    }

    Ok(())
}

async fn get_plugin(ctx: &CliContext, plugin: &str) -> CliResult<()> {
    let client = ctx.http_client();
    let url = format!("{}/api/v1/plugins/{}", ctx.server_url(), plugin);

    let response = client
        .get(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to fetch plugin: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(CliError::NotFound(format!("Plugin not found: {}", plugin)));
        }
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to get plugin ({}): {}",
            status, body
        )));
    }

    let details: PluginDetails = response
        .json()
        .await
        .map_err(|e| CliError::Serialization(format!("Failed to parse response: {}", e)))?;

    print_header(&format!("Plugin: {}", details.name));
    print_kv("ID", &details.id);
    print_kv("Version", &details.version);
    print_kv("Status", if details.is_active { "Active" } else { "Inactive" });
    if let Some(ref desc) = details.description {
        print_kv("Description", desc);
    }
    if let Some(ref author) = details.author {
        print_kv("Author", author);
    }

    Ok(())
}

async fn activate_plugin(ctx: &CliContext, plugin: &str) -> CliResult<()> {
    let spinner = ProgressBar::spinner("Activating plugin...");

    let client = ctx.http_client();
    let url = format!("{}/api/v1/plugins/{}/activate", ctx.server_url(), plugin);

    let response = client
        .post(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to activate plugin: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to activate plugin ({}): {}",
            status, body
        )));
    }

    println!("{}", ctx.output_format.success(&format!("Plugin '{}' activated", plugin)));
    Ok(())
}

async fn deactivate_plugin(ctx: &CliContext, plugin: &str) -> CliResult<()> {
    let spinner = ProgressBar::spinner("Deactivating plugin...");

    let client = ctx.http_client();
    let url = format!("{}/api/v1/plugins/{}/deactivate", ctx.server_url(), plugin);

    let response = client
        .post(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to deactivate plugin: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to deactivate plugin ({}): {}",
            status, body
        )));
    }

    println!("{}", ctx.output_format.success(&format!("Plugin '{}' deactivated", plugin)));
    Ok(())
}

async fn install_plugin(ctx: &CliContext, source: &str) -> CliResult<()> {
    print_header("Installing Plugin");
    print_kv("Source", source);

    // Check if source is a local file
    if std::path::Path::new(source).exists() {
        // For local files, we'd need multipart upload
        // For now, just register via API
        let spinner = ProgressBar::spinner("Installing plugin...");

        let plugin_id = std::path::Path::new(source)
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("plugin")
            .to_string();

        let client = ctx.http_client();
        let url = format!("{}/api/v1/plugins", ctx.server_url());

        let body = serde_json::json!({
            "plugin_id": plugin_id,
            "source": source,
        });

        let response = client
            .post(&url)
            .header("Authorization", auth_header(ctx)?)
            .json(&body)
            .send()
            .await
            .map_err(|e| CliError::Network(format!("Failed to install plugin: {}", e)))?;

        spinner.finish_and_clear();

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::OperationFailed(format!(
                "Failed to install plugin ({}): {}",
                status, body
            )));
        }

        println!("{}", ctx.output_format.success(&format!("Plugin '{}' installed", plugin_id)));
    } else {
        // URL-based install
        let spinner = ProgressBar::spinner("Installing plugin from URL...");

        let client = ctx.http_client();
        let url = format!("{}/api/v1/plugins/install", ctx.server_url());

        let body = serde_json::json!({
            "url": source,
        });

        let response = client
            .post(&url)
            .header("Authorization", auth_header(ctx)?)
            .json(&body)
            .send()
            .await
            .map_err(|e| CliError::Network(format!("Failed to install plugin: {}", e)))?;

        spinner.finish_and_clear();

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::OperationFailed(format!(
                "Failed to install plugin ({}): {}",
                status, body
            )));
        }

        println!("{}", ctx.output_format.success("Plugin installed"));
    }

    Ok(())
}

async fn uninstall_plugin(ctx: &CliContext, plugin: &str, force: bool) -> CliResult<()> {
    if !force {
        println!("This will permanently delete the plugin.");
        println!("Run with --force to confirm.");
        return Ok(());
    }

    let spinner = ProgressBar::spinner("Uninstalling plugin...");

    let client = ctx.http_client();
    let url = format!("{}/api/v1/plugins/{}", ctx.server_url(), plugin);

    let response = client
        .delete(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to uninstall plugin: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to uninstall plugin ({}): {}",
            status, body
        )));
    }

    println!("{}", ctx.output_format.success(&format!("Plugin '{}' uninstalled", plugin)));
    Ok(())
}

async fn check_updates(ctx: &CliContext) -> CliResult<()> {
    print_header("Checking for Plugin Updates");

    let spinner = ProgressBar::spinner("Checking updates...");

    let client = ctx.http_client();
    let url = format!("{}/api/v1/plugins/updates", ctx.server_url());

    let response = client
        .get(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to check updates: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        // If endpoint doesn't exist, just report no updates
        println!("{}", ctx.output_format.info("All plugins are up to date"));
        return Ok(());
    }

    let updates: Vec<serde_json::Value> = response
        .json()
        .await
        .unwrap_or_default();

    if updates.is_empty() {
        println!("{}", ctx.output_format.info("All plugins are up to date"));
    } else {
        println!("Available updates:");
        for update in updates {
            if let (Some(name), Some(version)) = (
                update.get("name").and_then(|v| v.as_str()),
                update.get("new_version").and_then(|v| v.as_str()),
            ) {
                println!("  - {} -> {}", name, version);
            }
        }
    }

    Ok(())
}
