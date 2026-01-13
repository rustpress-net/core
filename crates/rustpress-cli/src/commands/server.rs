//! Server management commands

use clap::{Args, Subcommand};
use colored::Colorize;

use crate::context::CliContext;
use crate::error::CliResult;
use crate::output::{print_header, print_kv, OutputFormatter};

#[derive(Args, Debug)]
pub struct ServerCommand {
    #[command(subcommand)]
    pub command: ServerSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum ServerSubcommand {
    /// Start the RustPress server
    Start {
        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Port to bind to
        #[arg(short, long, default_value = "3080")]
        port: u16,

        /// Number of worker threads
        #[arg(short, long)]
        workers: Option<usize>,

        /// Run in foreground (don't daemonize)
        #[arg(long)]
        foreground: bool,
    },

    /// Stop the RustPress server
    Stop {
        /// Force stop without graceful shutdown
        #[arg(short, long)]
        force: bool,
    },

    /// Show server status
    Status,

    /// Check server health
    Health {
        /// Server URL to check
        #[arg(long, default_value = "http://127.0.0.1:3080")]
        url: String,
    },
}

pub async fn execute(ctx: &CliContext, cmd: ServerCommand) -> CliResult<()> {
    match cmd.command {
        ServerSubcommand::Start {
            host,
            port,
            workers,
            foreground,
        } => start_server(ctx, &host, port, workers, foreground).await,
        ServerSubcommand::Stop { force } => stop_server(ctx, force).await,
        ServerSubcommand::Status => show_status(ctx).await,
        ServerSubcommand::Health { url } => check_health(ctx, &url).await,
    }
}

async fn start_server(
    _ctx: &CliContext,
    host: &str,
    port: u16,
    workers: Option<usize>,
    foreground: bool,
) -> CliResult<()> {
    print_header("Starting RustPress Server");

    print_kv("Host", host);
    print_kv("Port", &port.to_string());
    if let Some(w) = workers {
        print_kv("Workers", &w.to_string());
    }
    print_kv("Mode", if foreground { "Foreground" } else { "Daemon" });

    println!();

    // In a real implementation, this would start the actual server
    // For now, we provide guidance
    println!(
        "{}",
        "To start the server, use the rustpress-server binary:".yellow()
    );
    println!();
    println!(
        "  {} RUSTPRESS_HOST={} RUSTPRESS_PORT={} ./rustpress-server",
        "$".dimmed(),
        host,
        port
    );
    println!();
    println!(
        "{}",
        "Or set environment variables in your .env file.".dimmed()
    );

    Ok(())
}

async fn stop_server(ctx: &CliContext, force: bool) -> CliResult<()> {
    if force {
        ctx.print(&ctx.output_format.warning("Force stopping server..."));
    } else {
        ctx.print(&ctx.output_format.info("Gracefully stopping server..."));
    }

    // In a real implementation, this would send a signal to the server process
    println!(
        "{}",
        "Server stop functionality requires the server to be running with PID tracking.".yellow()
    );
    println!();
    println!("To stop the server manually:");
    println!("  {} pkill -f rustpress-server", "$".dimmed());

    Ok(())
}

async fn show_status(ctx: &CliContext) -> CliResult<()> {
    print_header("Server Status");

    // Try to connect to the server to check status
    let client = reqwest::Client::new();
    let url = "http://127.0.0.1:3080/health";

    match client.get(url).timeout(std::time::Duration::from_secs(5)).send().await {
        Ok(response) if response.status().is_success() => {
            println!("{}", ctx.output_format.success("Server is running"));
            print_kv("URL", "http://127.0.0.1:3080");
            print_kv("Status", "Healthy");
        }
        Ok(response) => {
            println!("{}", ctx.output_format.warning("Server responding but unhealthy"));
            print_kv("Status Code", &response.status().to_string());
        }
        Err(_) => {
            println!("{}", ctx.output_format.error("Server is not running or not reachable"));
            print_kv("Checked URL", url);
        }
    }

    Ok(())
}

async fn check_health(ctx: &CliContext, url: &str) -> CliResult<()> {
    let spinner = crate::output::ProgressBar::spinner("Checking server health...");

    let client = reqwest::Client::new();
    let health_url = format!("{}/health", url.trim_end_matches('/'));

    match client
        .get(&health_url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(response) => {
            spinner.finish_and_clear();

            if response.status().is_success() {
                println!("{}", ctx.output_format.success("Server is healthy"));

                if let Ok(body) = response.json::<serde_json::Value>().await {
                    println!();
                    print_header("Health Details");
                    println!("{}", ctx.output_format.format_one(&body));
                }
            } else {
                println!(
                    "{}",
                    ctx.output_format.error(&format!(
                        "Server returned status: {}",
                        response.status()
                    ))
                );
            }
        }
        Err(e) => {
            spinner.finish_and_clear();
            println!(
                "{}",
                ctx.output_format.error(&format!("Failed to connect: {}", e))
            );
        }
    }

    Ok(())
}
