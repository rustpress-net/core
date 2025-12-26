//! CLI Commands Module
//!
//! This module contains all CLI command definitions and their implementations.

use clap::{Parser, Subcommand};
use crate::output::OutputFormat;

pub mod artifacts;
pub mod auth;
pub mod backup;
pub mod cache;
pub mod completion;
pub mod config;
pub mod cron;
pub mod db;
pub mod import_export;
pub mod media;
pub mod pages;
pub mod plugins;
pub mod posts;
pub mod seo;
pub mod server;
pub mod settings;
pub mod themes;
pub mod users;

/// RustPress CLI - Command-line interface for RustPress CMS
#[derive(Parser, Debug)]
#[command(
    name = "rustpress",
    author = "RustPress Team",
    version,
    about = "Command-line interface for RustPress CMS",
    long_about = "RustPress CLI provides comprehensive management capabilities for RustPress CMS installations.\n\n\
                  Use this tool to manage database operations, users, content, themes, plugins, and more.",
    after_help = "For more information, visit: https://github.com/rustpress/rustpress"
)]
pub struct Cli {
    /// Output format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Table, global = true)]
    pub output: OutputFormat,

    /// Suppress non-essential output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Configuration file path
    #[arg(short, long, global = true, env = "RUSTPRESS_CONFIG")]
    pub config: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Artifact management (list, upload, download from artifactory)
    #[command(alias = "artifact")]
    Artifacts {
        #[command(subcommand)]
        command: artifacts::ArtifactCommands,
    },

    /// Authentication (login, logout, whoami)
    #[command(alias = "login")]
    Auth(auth::AuthCommand),

    /// Server management (start, stop, status)
    #[command(alias = "srv")]
    Server(server::ServerCommand),

    /// Database operations (migrate, backup, restore, query)
    #[command(alias = "database")]
    Db(db::DbCommand),

    /// User management (create, list, delete, import, export)
    #[command(alias = "user")]
    Users(users::UsersCommand),

    /// Post management (create, list, publish, delete)
    #[command(alias = "post")]
    Posts(posts::PostsCommand),

    /// Page management (create, list, delete)
    #[command(alias = "page")]
    Pages(pages::PagesCommand),

    /// Media management (upload, list, delete, optimize)
    Media(media::MediaCommand),

    /// Theme management (list, activate, install, export)
    #[command(alias = "theme")]
    Themes(themes::ThemesCommand),

    /// Plugin management (list, activate, deactivate, install)
    #[command(alias = "plugin")]
    Plugins(plugins::PluginsCommand),

    /// Cache operations (clear, warm, stats)
    Cache(cache::CacheCommand),

    /// Settings management (get, set, list)
    #[command(alias = "setting")]
    Settings(settings::SettingsCommand),

    /// Backup/restore operations
    Backup(backup::BackupCommand),

    /// SEO tools (sitemap, analyze)
    Seo(seo::SeoCommand),

    /// Configuration management
    #[command(alias = "cfg")]
    Config(config::ConfigCommand),

    /// Generate shell completions
    Completion(completion::CompletionCommand),

    /// WordPress import/export
    #[command(alias = "wp")]
    ImportExport(import_export::ImportExportCommand),

    /// Scheduled tasks management
    Cron(cron::CronCommand),

    /// Start interactive shell (REPL)
    #[command(alias = "shell", alias = "repl")]
    Interactive,

    /// System health check
    Health {
        /// Run detailed health check
        #[arg(long)]
        detailed: bool,
    },

    /// Show system information
    #[command(alias = "system")]
    Info,
}
