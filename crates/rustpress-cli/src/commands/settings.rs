//! Settings management commands

use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use tabled::Tabled;

use crate::context::CliContext;
use crate::error::{CliError, CliResult};
use crate::output::{print_header, OutputFormatter};

#[derive(Args, Debug)]
pub struct SettingsCommand {
    #[command(subcommand)]
    pub command: SettingsSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum SettingsSubcommand {
    /// List all settings
    List {
        #[arg(short, long)]
        group: Option<String>,
    },
    /// Get a setting value
    Get { key: String },
    /// Set a setting value
    Set { key: String, value: String },
    /// Export settings
    Export {
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Import settings
    Import { file: String },
}

#[derive(Debug, Serialize, Deserialize, Tabled)]
pub struct SettingRow {
    #[tabled(rename = "Key")]
    pub key: String,
    #[tabled(rename = "Value")]
    pub value: String,
    #[tabled(rename = "Group")]
    pub group: String,
}

pub async fn execute(ctx: &CliContext, cmd: SettingsCommand) -> CliResult<()> {
    match cmd.command {
        SettingsSubcommand::List { group } => list_settings(ctx, group).await,
        SettingsSubcommand::Get { key } => get_setting(ctx, &key).await,
        SettingsSubcommand::Set { key, value } => set_setting(ctx, &key, &value).await,
        SettingsSubcommand::Export { output } => export_settings(ctx, output).await,
        SettingsSubcommand::Import { file } => import_settings(ctx, &file).await,
    }
}

fn auth_header(ctx: &CliContext) -> CliResult<String> {
    let token = ctx.require_auth()?;
    Ok(format!("Bearer {}", token))
}

async fn list_settings(ctx: &CliContext, group: Option<String>) -> CliResult<()> {
    print_header("Settings");

    let client = ctx.http_client();
    let mut url = format!("{}/api/v1/settings", ctx.server_url());

    if let Some(ref g) = group {
        url.push_str(&format!("?group={}", g));
    }

    let response = client
        .get(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to fetch settings: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to list settings ({}): {}",
            status, body
        )));
    }

    let settings: Vec<SettingRow> = response
        .json()
        .await
        .map_err(|e| CliError::Serialization(format!("Failed to parse response: {}", e)))?;

    if settings.is_empty() {
        println!("{}", ctx.output_format.info("No settings found"));
    } else {
        println!("{}", ctx.output_format.format(&settings));
    }

    Ok(())
}

async fn get_setting(ctx: &CliContext, key: &str) -> CliResult<()> {
    let client = ctx.http_client();
    let url = format!("{}/api/v1/settings/{}", ctx.server_url(), key);

    let response = client
        .get(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to fetch setting: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            println!("{}", ctx.output_format.error("Setting not found"));
            return Ok(());
        }
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to get setting ({}): {}",
            status, body
        )));
    }

    let setting: serde_json::Value = response
        .json()
        .await
        .map_err(|e| CliError::Serialization(format!("Failed to parse response: {}", e)))?;

    let value = setting.get("value").and_then(|v| v.as_str()).unwrap_or("");
    println!("{}", value);

    Ok(())
}

async fn set_setting(ctx: &CliContext, key: &str, value: &str) -> CliResult<()> {
    let client = ctx.http_client();
    let url = format!("{}/api/v1/settings/{}", ctx.server_url(), key);

    let body = serde_json::json!({
        "value": value,
    });

    let response = client
        .put(&url)
        .header("Authorization", auth_header(ctx)?)
        .json(&body)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to set setting: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to set setting ({}): {}",
            status, body
        )));
    }

    println!("{}", ctx.output_format.success(&format!("Set {} = {}", key, value)));
    Ok(())
}

async fn export_settings(ctx: &CliContext, output: Option<String>) -> CliResult<()> {
    let output_file = output.unwrap_or_else(|| "settings.json".to_string());

    let client = ctx.http_client();
    let url = format!("{}/api/v1/settings", ctx.server_url());

    let response = client
        .get(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to fetch settings: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to export settings ({}): {}",
            status, body
        )));
    }

    let settings: Vec<SettingRow> = response
        .json()
        .await
        .map_err(|e| CliError::Serialization(format!("Failed to parse response: {}", e)))?;

    // Convert to key-value map
    let export_data: serde_json::Map<String, serde_json::Value> = settings
        .into_iter()
        .map(|s| (s.key, serde_json::Value::String(s.value)))
        .collect();

    let json = serde_json::to_string_pretty(&export_data)?;
    std::fs::write(&output_file, json)?;

    println!("{}", ctx.output_format.success(&format!("Exported to {}", output_file)));
    Ok(())
}

async fn import_settings(ctx: &CliContext, file: &str) -> CliResult<()> {
    let content = std::fs::read_to_string(file)?;
    let settings: serde_json::Value = serde_json::from_str(&content)?;

    let client = ctx.http_client();

    if let Some(obj) = settings.as_object() {
        let mut success = 0;
        let mut failed = 0;

        for (key, value) in obj {
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                _ => value.to_string(),
            };

            let url = format!("{}/api/v1/settings/{}", ctx.server_url(), key);
            let body = serde_json::json!({ "value": value_str });

            let response = client
                .put(&url)
                .header("Authorization", auth_header(ctx)?)
                .json(&body)
                .send()
                .await;

            match response {
                Ok(resp) if resp.status().is_success() => success += 1,
                _ => failed += 1,
            }
        }

        println!(
            "{}",
            ctx.output_format.success(&format!(
                "Imported {} settings{}",
                success,
                if failed > 0 { format!(", {} failed", failed) } else { String::new() }
            ))
        );
    } else {
        return Err(CliError::InvalidInput("Settings file must be a JSON object".to_string()));
    }

    Ok(())
}
