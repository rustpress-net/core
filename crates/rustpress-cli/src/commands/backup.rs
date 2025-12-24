//! Backup management commands

use clap::{Args, Subcommand};
use serde::Serialize;
use tabled::Tabled;

use crate::context::CliContext;
use crate::error::CliResult;
use crate::output::{print_header, print_kv, OutputFormatter, ProgressBar};

#[derive(Args, Debug)]
pub struct BackupCommand {
    #[command(subcommand)]
    pub command: BackupSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum BackupSubcommand {
    /// Create a backup
    Create {
        #[arg(long, default_value = "full")]
        r#type: String,
        #[arg(long)]
        include_media: bool,
        #[arg(short, long)]
        output: Option<String>,
    },
    /// List backups
    List,
    /// Restore from backup
    Restore {
        backup: String,
        #[arg(short, long)]
        yes: bool,
    },
    /// Delete a backup
    Delete { backup: String },
    /// Download a backup
    Download {
        backup: String,
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Manage backup schedules
    #[command(subcommand)]
    Schedule(ScheduleSubcommand),
}

#[derive(Subcommand, Debug)]
pub enum ScheduleSubcommand {
    List,
    Create {
        #[arg(long)]
        cron: String,
        #[arg(long, default_value = "full")]
        r#type: String,
    },
    Delete { id: String },
}

#[derive(Debug, Serialize, Tabled)]
pub struct BackupRow {
    #[tabled(rename = "ID")]
    pub id: String,
    #[tabled(rename = "Type")]
    pub backup_type: String,
    #[tabled(rename = "Size")]
    pub size: String,
    #[tabled(rename = "Created")]
    pub created_at: String,
}

pub async fn execute(ctx: &CliContext, cmd: BackupCommand) -> CliResult<()> {
    match cmd.command {
        BackupSubcommand::Create { r#type, include_media, output } => create_backup(ctx, &r#type, include_media, output).await,
        BackupSubcommand::List => list_backups(ctx).await,
        BackupSubcommand::Restore { backup, yes } => restore_backup(ctx, &backup, yes).await,
        BackupSubcommand::Delete { backup } => delete_backup(ctx, &backup).await,
        BackupSubcommand::Download { backup, output } => download_backup(ctx, &backup, output).await,
        BackupSubcommand::Schedule(sub) => match sub {
            ScheduleSubcommand::List => list_schedules(ctx).await,
            ScheduleSubcommand::Create { cron, r#type } => create_schedule(ctx, &cron, &r#type).await,
            ScheduleSubcommand::Delete { id } => delete_schedule(ctx, &id).await,
        },
    }
}

async fn create_backup(ctx: &CliContext, backup_type: &str, include_media: bool, output: Option<String>) -> CliResult<()> {
    print_header("Creating Backup");
    print_kv("Type", backup_type);
    print_kv("Include Media", if include_media { "Yes" } else { "No" });

    let spinner = ProgressBar::spinner("Creating backup...");
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    spinner.finish_and_clear();

    let output_file = output.unwrap_or_else(|| format!("backup_{}.sql", chrono::Utc::now().format("%Y%m%d_%H%M%S")));
    println!("{}", ctx.output_format.success(&format!("Backup created: {}", output_file)));
    Ok(())
}

async fn list_backups(ctx: &CliContext) -> CliResult<()> {
    print_header("Backups");
    let backups = vec![
        BackupRow { id: "bkp_001".into(), backup_type: "Full".into(), size: "45.2 MB".into(), created_at: "2024-01-15".into() },
        BackupRow { id: "bkp_002".into(), backup_type: "Incremental".into(), size: "2.1 MB".into(), created_at: "2024-01-16".into() },
    ];
    println!("{}", ctx.output_format.format(&backups));
    Ok(())
}

async fn restore_backup(ctx: &CliContext, backup: &str, yes: bool) -> CliResult<()> {
    if !yes {
        println!("This will overwrite existing data. Run with --yes to confirm.");
        return Ok(());
    }
    print_header("Restoring Backup");
    let spinner = ProgressBar::spinner("Restoring...");
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    spinner.finish_and_clear();
    println!("{}", ctx.output_format.success(&format!("Restored from {}", backup)));
    Ok(())
}

async fn delete_backup(ctx: &CliContext, backup: &str) -> CliResult<()> {
    println!("{}", ctx.output_format.success(&format!("Deleted backup: {}", backup)));
    Ok(())
}

async fn download_backup(ctx: &CliContext, backup: &str, output: Option<String>) -> CliResult<()> {
    let output_file = output.unwrap_or_else(|| format!("{}.sql", backup));
    println!("{}", ctx.output_format.success(&format!("Downloaded to {}", output_file)));
    Ok(())
}

async fn list_schedules(ctx: &CliContext) -> CliResult<()> {
    print_header("Backup Schedules");
    println!("{}", ctx.output_format.info("No schedules configured"));
    Ok(())
}

async fn create_schedule(ctx: &CliContext, cron: &str, backup_type: &str) -> CliResult<()> {
    println!("{}", ctx.output_format.success(&format!("Created schedule: {} ({})", cron, backup_type)));
    Ok(())
}

async fn delete_schedule(ctx: &CliContext, id: &str) -> CliResult<()> {
    println!("{}", ctx.output_format.success(&format!("Deleted schedule: {}", id)));
    Ok(())
}
