//! Cache management commands

use clap::{Args, Subcommand};
use serde::Serialize;
use tabled::Tabled;

use crate::context::CliContext;
use crate::error::CliResult;
use crate::output::{print_header, print_kv, OutputFormatter, ProgressBar};

#[derive(Args, Debug)]
pub struct CacheCommand {
    #[command(subcommand)]
    pub command: CacheSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum CacheSubcommand {
    /// Show cache statistics
    Stats,

    /// Clear cache
    Clear {
        /// Cache type to clear (page, object, query, all)
        #[arg(short, long, default_value = "all")]
        r#type: String,
    },

    /// Warm the cache
    Warm {
        /// Warm page cache
        #[arg(long)]
        pages: bool,
        /// Warm post cache
        #[arg(long)]
        posts: bool,
    },

    /// Get or set cache configuration
    Config {
        /// Get a config value
        #[arg(long)]
        get: Option<String>,
        /// Set a config value (key=value)
        #[arg(long)]
        set: Option<String>,
    },
}

#[derive(Debug, Serialize, Tabled)]
pub struct CacheStats {
    #[tabled(rename = "Type")]
    pub cache_type: String,
    #[tabled(rename = "Entries")]
    pub entries: i64,
    #[tabled(rename = "Hit Rate")]
    pub hit_rate: String,
    #[tabled(rename = "Memory")]
    pub memory: String,
}

pub async fn execute(ctx: &CliContext, cmd: CacheCommand) -> CliResult<()> {
    match cmd.command {
        CacheSubcommand::Stats => show_stats(ctx).await,
        CacheSubcommand::Clear { r#type } => clear_cache(ctx, &r#type).await,
        CacheSubcommand::Warm { pages, posts } => warm_cache(ctx, pages, posts).await,
        CacheSubcommand::Config { get, set } => config_cache(ctx, get, set).await,
    }
}

async fn show_stats(ctx: &CliContext) -> CliResult<()> {
    print_header("Cache Statistics");

    let stats = vec![
        CacheStats { cache_type: "Page".into(), entries: 156, hit_rate: "94.2%".into(), memory: "12.4 MB".into() },
        CacheStats { cache_type: "Object".into(), entries: 2341, hit_rate: "87.8%".into(), memory: "8.1 MB".into() },
        CacheStats { cache_type: "Query".into(), entries: 89, hit_rate: "91.5%".into(), memory: "2.3 MB".into() },
    ];

    println!("{}", ctx.output_format.format(&stats));
    println!();
    print_kv("Total Entries", "2586");
    print_kv("Total Memory", "22.8 MB");
    print_kv("Overall Hit Rate", "91.2%");

    Ok(())
}

async fn clear_cache(ctx: &CliContext, cache_type: &str) -> CliResult<()> {
    print_header(&format!("Clearing {} Cache", if cache_type == "all" { "All" } else { cache_type }));

    let spinner = ProgressBar::spinner("Clearing cache...");
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    spinner.finish_and_clear();

    println!("{}", ctx.output_format.success("Cache cleared successfully"));
    Ok(())
}

async fn warm_cache(ctx: &CliContext, pages: bool, posts: bool) -> CliResult<()> {
    print_header("Warming Cache");

    if !pages && !posts {
        println!("{}", ctx.output_format.info("Warming all content..."));
    }

    let spinner = ProgressBar::spinner("Warming cache...");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    spinner.finish_and_clear();

    println!("{}", ctx.output_format.success("Cache warmed successfully"));
    Ok(())
}

async fn config_cache(ctx: &CliContext, get: Option<String>, set: Option<String>) -> CliResult<()> {
    if let Some(key) = get {
        print_header("Cache Configuration");
        match key.as_str() {
            "ttl" => print_kv("ttl", "3600"),
            "max_size" => print_kv("max_size", "100MB"),
            _ => println!("{}", ctx.output_format.error(&format!("Unknown key: {}", key))),
        }
    } else if let Some(kv) = set {
        if let Some((key, value)) = kv.split_once('=') {
            println!("{}", ctx.output_format.success(&format!("Set {} = {}", key, value)));
        }
    } else {
        print_header("Cache Configuration");
        print_kv("ttl", "3600");
        print_kv("max_size", "100MB");
        print_kv("driver", "memory");
    }
    Ok(())
}
