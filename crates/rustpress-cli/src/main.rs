//! RustPress CLI - Command-line interface for RustPress CMS
//!
//! A comprehensive CLI tool for managing RustPress installations,
//! including database operations, user management, content management,
//! themes, plugins, and more.

use clap::Parser;
use colored::Colorize;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod commands;
mod context;
mod error;
mod output;
mod prompts;
mod repl;

use commands::{Cli, Commands};
use context::CliContext;
use error::CliResult;

#[tokio::main]
async fn main() {
    // Initialize tracing
    init_tracing();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Run the CLI
    if let Err(e) = run(cli).await {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("warn"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();
}

async fn run(cli: Cli) -> CliResult<()> {
    // Load environment variables from .env if present
    let _ = dotenvy::dotenv();

    // Create CLI context with global options
    let ctx = CliContext::new(&cli)?;

    // Commands that don't require authentication
    match &cli.command {
        Commands::Auth(_) | Commands::Completion(_) | Commands::Config(_)
        | Commands::Interactive | Commands::Health { .. } | Commands::Info => {}
        // ImportExport analyze doesn't need auth
        Commands::ImportExport(ref cmd) => {
            if !matches!(cmd.command, commands::import_export::ImportExportSubcommand::Analyze { .. }) {
                ctx.require_auth()?;
            }
        }
        _ => {
            // All other commands require authentication
            ctx.require_auth()?;
        }
    }

    // Match and execute the command
    match cli.command {
        Commands::Auth(cmd) => commands::auth::execute(&ctx, cmd).await,
        Commands::Server(cmd) => commands::server::execute(&ctx, cmd).await,
        Commands::Db(cmd) => commands::db::execute(&ctx, cmd).await,
        Commands::Users(cmd) => commands::users::execute(&ctx, cmd).await,
        Commands::Posts(cmd) => commands::posts::execute(&ctx, cmd).await,
        Commands::Pages(cmd) => commands::pages::execute(&ctx, cmd).await,
        Commands::Media(cmd) => commands::media::execute(&ctx, cmd).await,
        Commands::Themes(cmd) => commands::themes::execute(&ctx, cmd).await,
        Commands::Plugins(cmd) => commands::plugins::execute(&ctx, cmd).await,
        Commands::Cache(cmd) => commands::cache::execute(&ctx, cmd).await,
        Commands::Settings(cmd) => commands::settings::execute(&ctx, cmd).await,
        Commands::Backup(cmd) => commands::backup::execute(&ctx, cmd).await,
        Commands::Seo(cmd) => commands::seo::execute(&ctx, cmd).await,
        Commands::Config(cmd) => commands::config::execute(&ctx, cmd).await,
        Commands::Completion(cmd) => commands::completion::execute(cmd).await,
        Commands::ImportExport(cmd) => commands::import_export::execute(&ctx, cmd).await,
        Commands::Cron(cmd) => commands::cron::execute(&ctx, cmd).await,
        Commands::Interactive => repl::run_repl().await,
        Commands::Health { detailed } => run_health_check(detailed).await,
        Commands::Info => run_system_info().await,
    }
}

/// Run system health check
async fn run_health_check(detailed: bool) -> CliResult<()> {
    use crate::context::CliCredentials;
    use crate::output::print_header;

    print_header("System Health Check");

    let creds = CliCredentials::load();
    let server_url = if creds.server_url.is_empty() {
        "http://localhost:3080".to_string()
    } else {
        creds.server_url.clone()
    };

    println!("  {} Checking server connectivity...", "→".cyan());

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();

    // Check health endpoint
    let health_url = format!("{}/api/v1/health", server_url);
    match client.get(&health_url).send().await {
        Ok(response) if response.status().is_success() => {
            println!("  {} Server is {}", "✓".green(), "online".green().bold());

            if detailed {
                if let Ok(body) = response.json::<serde_json::Value>().await {
                    if let Some(obj) = body.as_object() {
                        println!();
                        println!("  {}", "Health Details:".cyan());
                        for (key, value) in obj {
                            println!("    {}: {}", key.green(), value);
                        }
                    }
                }
            }
        }
        Ok(response) => {
            println!("  {} Server returned {}", "✗".red(), response.status().to_string().red());
        }
        Err(e) => {
            println!("  {} Server is {} ({})", "✗".red(), "unreachable".red().bold(), e);
        }
    }

    // Check authentication status
    println!();
    println!("  {} Checking authentication...", "→".cyan());
    if creds.access_token.is_some() {
        println!("  {} Authentication token {}", "✓".green(), "present".green());

        // Verify token is valid
        if let Some(token) = &creds.access_token {
            let me_url = format!("{}/api/v1/users/me", server_url);
            match client
                .get(&me_url)
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    println!("  {} Token is {}", "✓".green(), "valid".green());
                }
                Ok(response) if response.status() == reqwest::StatusCode::UNAUTHORIZED => {
                    println!("  {} Token is {}", "✗".yellow(), "expired".yellow());
                    println!("    {} Run 'rustpress auth login' to re-authenticate", "Tip:".dimmed());
                }
                _ => {
                    println!("  {} Token validation {}", "?".yellow(), "unknown".yellow());
                }
            }
        }
    } else {
        println!("  {} Not authenticated", "○".yellow());
    }

    println!();
    println!("{}", "Health check complete.".dimmed());

    Ok(())
}

/// Run system info command
async fn run_system_info() -> CliResult<()> {
    use crate::context::CliCredentials;
    use crate::output::print_header;

    print_header("System Information");

    println!();
    println!("  {}", "CLI:".cyan().bold());
    println!("    {} {}", "Version:".green(), env!("CARGO_PKG_VERSION"));
    println!("    {} {}", "OS:".green(), std::env::consts::OS);
    println!("    {} {}", "Architecture:".green(), std::env::consts::ARCH);

    let creds = CliCredentials::load();
    println!();
    println!("  {}", "Configuration:".cyan().bold());
    println!("    {} {}", "Server URL:".green(),
        if creds.server_url.is_empty() { "http://localhost:3080 (default)" } else { &creds.server_url });
    println!("    {} {}", "Authenticated:".green(),
        if creds.access_token.is_some() { "Yes".green().to_string() } else { "No".yellow().to_string() });
    if let Some(email) = &creds.email {
        println!("    {} {}", "User:".green(), email);
    }

    // Try to get server info
    if creds.access_token.is_some() {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap();

        let server_url = if creds.server_url.is_empty() {
            "http://localhost:3080"
        } else {
            &creds.server_url
        };

        let info_url = format!("{}/api/v1/info", server_url);
        if let Ok(response) = client
            .get(&info_url)
            .header("Authorization", format!("Bearer {}", creds.access_token.as_ref().unwrap()))
            .send()
            .await
        {
            if response.status().is_success() {
                if let Ok(info) = response.json::<serde_json::Value>().await {
                    println!();
                    println!("  {}", "Server:".cyan().bold());
                    if let Some(obj) = info.as_object() {
                        for (key, value) in obj {
                            let display_value = match value {
                                serde_json::Value::String(s) => s.clone(),
                                _ => value.to_string(),
                            };
                            println!("    {} {}", format!("{}:", key).green(), display_value);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
