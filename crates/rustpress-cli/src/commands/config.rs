//! Configuration management commands

use clap::{Args, Subcommand};
use colored::Colorize;

use crate::context::CliContext;
use crate::error::CliResult;
use crate::output::{print_header, print_kv, OutputFormatter};

#[derive(Args, Debug)]
pub struct ConfigCommand {
    #[command(subcommand)]
    pub command: ConfigSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum ConfigSubcommand {
    /// Show current configuration
    Show {
        /// Show secret values
        #[arg(long)]
        secrets: bool,
    },
    /// Validate configuration
    Validate,
    /// Initialize a new configuration file
    Init {
        /// Output file path
        #[arg(short, long, default_value = ".env")]
        output: String,
    },
    /// Show required environment variables
    Env,
}

pub async fn execute(ctx: &CliContext, cmd: ConfigCommand) -> CliResult<()> {
    match cmd.command {
        ConfigSubcommand::Show { secrets } => show_config(ctx, secrets),
        ConfigSubcommand::Validate => validate_config(ctx),
        ConfigSubcommand::Init { output } => init_config(ctx, &output),
        ConfigSubcommand::Env => show_env(ctx),
    }
}

fn show_config(ctx: &CliContext, secrets: bool) -> CliResult<()> {
    print_header("Current Configuration");

    println!("{}", "Server".bold());
    print_kv("  Host", &std::env::var("RUSTPRESS_HOST").unwrap_or_else(|_| "127.0.0.1".into()));
    print_kv("  Port", &std::env::var("RUSTPRESS_PORT").unwrap_or_else(|_| "3080".into()));

    println!();
    println!("{}", "Database".bold());
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "(not set)".into());
    if secrets {
        print_kv("  URL", &db_url);
    } else {
        let masked = if db_url.contains('@') {
            let parts: Vec<&str> = db_url.split('@').collect();
            format!("***@{}", parts.get(1).unwrap_or(&"***"))
        } else {
            "***".into()
        };
        print_kv("  URL", &masked);
    }

    println!();
    println!("{}", "Storage".bold());
    print_kv("  Path", &std::env::var("STORAGE_PATH").unwrap_or_else(|_| "./storage".into()));
    print_kv("  Themes", &std::env::var("THEMES_PATH").unwrap_or_else(|_| "./themes".into()));

    println!();
    println!("{}", "Auth".bold());
    if secrets {
        print_kv("  JWT Secret", &std::env::var("JWT_SECRET").unwrap_or_else(|_| "(not set)".into()));
    } else {
        print_kv("  JWT Secret", if std::env::var("JWT_SECRET").is_ok() { "***" } else { "(not set)" });
    }

    Ok(())
}

fn validate_config(ctx: &CliContext) -> CliResult<()> {
    print_header("Validating Configuration");

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Required
    if std::env::var("DATABASE_URL").is_err() {
        errors.push("DATABASE_URL is not set");
    }

    // Recommended
    if std::env::var("JWT_SECRET").is_err() {
        warnings.push("JWT_SECRET is not set (will use default, not secure for production)");
    }

    if errors.is_empty() && warnings.is_empty() {
        println!("{}", ctx.output_format.success("Configuration is valid"));
    } else {
        for error in &errors {
            println!("{}", ctx.output_format.error(error));
        }
        for warning in &warnings {
            println!("{}", ctx.output_format.warning(warning));
        }
        if !errors.is_empty() {
            println!();
            println!("{}", "Configuration has errors".red());
        }
    }

    Ok(())
}

fn init_config(_ctx: &CliContext, output: &str) -> CliResult<()> {
    print_header("Initializing Configuration");

    let template = r#"# RustPress Configuration
# Copy this file to .env and update the values

# Database connection URL (required)
DATABASE_URL=postgres://user:password@localhost:5432/rustpress

# Server configuration
RUSTPRESS_HOST=127.0.0.1
RUSTPRESS_PORT=3080

# Authentication
JWT_SECRET=your-secret-key-here

# Storage paths
STORAGE_PATH=./storage
THEMES_PATH=./themes

# Logging
RUST_LOG=info

# Cache
CACHE_MAX_CAPACITY=10000
"#;

    std::fs::write(output, template)?;
    println!("Created configuration file: {}", output.green());
    println!();
    println!("Edit the file and set your database connection URL.");

    Ok(())
}

fn show_env(_ctx: &CliContext) -> CliResult<()> {
    print_header("Required Environment Variables");

    println!("{}", "Required:".bold());
    print_kv("  DATABASE_URL", "PostgreSQL connection URL");

    println!();
    println!("{}", "Optional:".bold());
    print_kv("  RUSTPRESS_HOST", "Server bind host (default: 127.0.0.1)");
    print_kv("  RUSTPRESS_PORT", "Server bind port (default: 3080)");
    print_kv("  JWT_SECRET", "JWT signing secret");
    print_kv("  STORAGE_PATH", "File storage path (default: ./storage)");
    print_kv("  THEMES_PATH", "Themes directory (default: ./themes)");
    print_kv("  RUST_LOG", "Logging level (default: info)");
    print_kv("  CACHE_MAX_CAPACITY", "Maximum cache entries (default: 10000)");

    Ok(())
}
