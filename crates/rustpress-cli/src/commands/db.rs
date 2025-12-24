//! Database management commands

use clap::{Args, Subcommand};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use tabled::Tabled;

use crate::context::CliContext;
use crate::error::{CliError, CliResult};
use crate::output::{print_header, print_kv, OutputFormatter, ProgressBar};

#[derive(Args, Debug)]
pub struct DbCommand {
    #[command(subcommand)]
    pub command: DbSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum DbSubcommand {
    /// Run database migrations
    Migrate {
        /// Show what would be run without executing
        #[arg(long)]
        dry_run: bool,

        /// Rollback the last N migrations
        #[arg(long)]
        rollback: Option<u32>,

        /// Show migration status only
        #[arg(long)]
        status: bool,
    },

    /// Show database status and statistics
    Status,

    /// Create a database backup
    Backup {
        /// Output file path
        #[arg(short, long)]
        output: Option<String>,

        /// Include media files in backup
        #[arg(long)]
        include_media: bool,

        /// Backup type
        #[arg(long, default_value = "full")]
        r#type: String,
    },

    /// Restore from a backup
    Restore {
        /// Backup file to restore from
        file: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },

    /// Execute a SQL query
    Query {
        /// SQL query to execute
        sql: String,

        /// Show query execution plan
        #[arg(long)]
        explain: bool,

        /// Limit number of results
        #[arg(short, long, default_value = "100")]
        limit: u32,
    },

    /// List database tables
    Tables {
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Export table data
    Export {
        /// Table name to export
        table: String,

        /// Output format (csv, json, sql)
        #[arg(short, long, default_value = "csv")]
        format: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,

        /// WHERE clause for filtering
        #[arg(long)]
        r#where: Option<String>,
    },

    /// Import data into a table
    Import {
        /// File to import
        file: String,

        /// Target table name
        #[arg(short, long)]
        table: Option<String>,

        /// Input format (csv, sql)
        #[arg(short, long)]
        format: Option<String>,

        /// Dry run - validate without importing
        #[arg(long)]
        dry_run: bool,
    },

    /// Optimize database tables
    Optimize {
        /// Specific table to optimize (all if not specified)
        #[arg(short, long)]
        table: Option<String>,
    },

    /// View or clear audit log
    AuditLog {
        /// Clear the audit log
        #[arg(long)]
        clear: bool,

        /// Filter by date (since)
        #[arg(long)]
        since: Option<String>,

        /// Limit number of entries
        #[arg(short, long, default_value = "50")]
        limit: u32,
    },
}

#[derive(Debug, Serialize, Deserialize, Tabled)]
pub struct TableInfo {
    #[tabled(rename = "Table")]
    pub name: String,
    #[tabled(rename = "Rows")]
    pub row_count: i64,
    #[tabled(rename = "Size")]
    pub size: String,
    #[tabled(rename = "Last Modified")]
    pub last_modified: String,
}

#[derive(Debug, Serialize, Deserialize, Tabled)]
pub struct MigrationStatus {
    #[tabled(rename = "Version")]
    pub version: i64,
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Applied")]
    pub applied: String,
    #[tabled(rename = "Applied At")]
    pub applied_at: String,
}

#[derive(Debug, Serialize, Tabled)]
pub struct QueryResult {
    #[tabled(rename = "#")]
    pub row_num: usize,
    #[tabled(rename = "Data")]
    pub data: String,
}

#[derive(Debug, Serialize, Deserialize, Tabled)]
pub struct AuditLogEntry {
    #[tabled(rename = "ID")]
    pub id: i64,
    #[tabled(rename = "Action")]
    pub action: String,
    #[tabled(rename = "Table")]
    pub table_name: String,
    #[tabled(rename = "User")]
    pub user: String,
    #[tabled(rename = "Timestamp")]
    pub timestamp: String,
}

pub async fn execute(ctx: &CliContext, cmd: DbCommand) -> CliResult<()> {
    match cmd.command {
        DbSubcommand::Migrate {
            dry_run,
            rollback,
            status,
        } => run_migrate(ctx, dry_run, rollback, status).await,
        DbSubcommand::Status => show_status(ctx).await,
        DbSubcommand::Backup {
            output,
            include_media,
            r#type,
        } => create_backup(ctx, output, include_media, &r#type).await,
        DbSubcommand::Restore { file, yes } => restore_backup(ctx, &file, yes).await,
        DbSubcommand::Query { sql, explain, limit } => run_query(ctx, &sql, explain, limit).await,
        DbSubcommand::Tables { verbose } => list_tables(ctx, verbose).await,
        DbSubcommand::Export {
            table,
            format,
            output,
            r#where,
        } => export_table(ctx, &table, &format, output, r#where).await,
        DbSubcommand::Import {
            file,
            table,
            format,
            dry_run,
        } => import_data(ctx, &file, table, format, dry_run).await,
        DbSubcommand::Optimize { table } => optimize_tables(ctx, table).await,
        DbSubcommand::AuditLog {
            clear,
            since,
            limit,
        } => audit_log(ctx, clear, since, limit).await,
    }
}

fn auth_header(ctx: &CliContext) -> CliResult<String> {
    let token = ctx.require_auth()?;
    Ok(format!("Bearer {}", token))
}

async fn run_migrate(
    ctx: &CliContext,
    dry_run: bool,
    rollback: Option<u32>,
    status_only: bool,
) -> CliResult<()> {
    let client = ctx.http_client();

    if status_only {
        print_header("Migration Status");

        let url = format!("{}/api/v1/db/migrations", ctx.server_url());
        let response = client
            .get(&url)
            .header("Authorization", auth_header(ctx)?)
            .send()
            .await
            .map_err(|e| CliError::Network(format!("Failed to fetch migrations: {}", e)))?;

        if !response.status().is_success() {
            println!("{}", ctx.output_format.info("Migration status is not available via API. Use direct database access."));
            return Ok(());
        }

        let migrations: Vec<MigrationStatus> = response.json().await.unwrap_or_default();

        if migrations.is_empty() {
            println!("{}", ctx.output_format.info("No migrations found"));
        } else {
            println!("{}", ctx.output_format.format(&migrations));
        }

        return Ok(());
    }

    if let Some(n) = rollback {
        print_header(&format!("Rolling back {} migration(s)", n));

        if dry_run {
            println!("{}", ctx.output_format.warning("Dry run - no changes will be made"));
        }

        let url = format!("{}/api/v1/db/migrations/rollback?count={}&dry_run={}", ctx.server_url(), n, dry_run);
        let response = client
            .post(&url)
            .header("Authorization", auth_header(ctx)?)
            .send()
            .await
            .map_err(|e| CliError::Network(format!("Failed to rollback migrations: {}", e)))?;

        if !response.status().is_success() {
            println!("{}", ctx.output_format.info("Migration rollback is not available via API. Use direct database access."));
            return Ok(());
        }

        if !dry_run {
            println!("{}", ctx.output_format.success("Rollback completed"));
        }
    } else {
        print_header("Running Migrations");

        if dry_run {
            println!("{}", ctx.output_format.warning("Dry run - no changes will be made"));
        }

        let spinner = ProgressBar::spinner("Running migrations...");

        let url = format!("{}/api/v1/db/migrations/run?dry_run={}", ctx.server_url(), dry_run);
        let response = client
            .post(&url)
            .header("Authorization", auth_header(ctx)?)
            .send()
            .await
            .map_err(|e| CliError::Network(format!("Failed to run migrations: {}", e)))?;

        spinner.finish_and_clear();

        if !response.status().is_success() {
            println!("{}", ctx.output_format.info("Migrations are not available via API. Use direct database access."));
            return Ok(());
        }

        let result: serde_json::Value = response.json().await.unwrap_or_default();
        let applied = result.get("applied").and_then(|v| v.as_i64()).unwrap_or(0);

        if applied == 0 {
            println!("{}", ctx.output_format.success("Database is up to date"));
        } else if !dry_run {
            println!("{}", ctx.output_format.success(&format!("Applied {} migration(s)", applied)));
        }
    }

    Ok(())
}

async fn show_status(ctx: &CliContext) -> CliResult<()> {
    print_header("Database Status");

    let client = ctx.http_client();
    let url = format!("{}/api/v1/db/status", ctx.server_url());

    let response = client
        .get(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to fetch database status: {}", e)))?;

    if !response.status().is_success() {
        // Fallback to health endpoint
        let health_url = format!("{}/health", ctx.server_url());
        let health_response = client.get(&health_url).send().await;

        match health_response {
            Ok(r) if r.status().is_success() => {
                print_kv("Server", &ctx.server_url());
                print_kv("Status", "Connected");
                println!();
                println!("{}", ctx.output_format.success("Database connection healthy"));
            }
            _ => {
                print_kv("Server", &ctx.server_url());
                print_kv("Status", "Unable to connect");
                println!();
                println!("{}", ctx.output_format.error("Could not connect to server"));
            }
        }
        return Ok(());
    }

    let status: serde_json::Value = response.json().await.unwrap_or_default();

    if let Some(database) = status.get("database").and_then(|v| v.as_str()) {
        print_kv("Database", database);
    }
    if let Some(version) = status.get("version").and_then(|v| v.as_str()) {
        print_kv("Version", version);
    }
    if let Some(connections) = status.get("connections").and_then(|v| v.as_i64()) {
        print_kv("Active Connections", &connections.to_string());
    }
    if let Some(size) = status.get("size").and_then(|v| v.as_str()) {
        print_kv("Database Size", size);
    }
    if let Some(tables) = status.get("tables").and_then(|v| v.as_i64()) {
        print_kv("Tables", &tables.to_string());
    }

    println!();
    println!("{}", ctx.output_format.success("Database connection healthy"));

    Ok(())
}

async fn create_backup(
    ctx: &CliContext,
    output: Option<String>,
    include_media: bool,
    backup_type: &str,
) -> CliResult<()> {
    print_header("Creating Database Backup");

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let output_file = output.unwrap_or_else(|| format!("rustpress_backup_{}.sql", timestamp));

    print_kv("Type", backup_type);
    print_kv("Output", &output_file);
    print_kv("Include Media", if include_media { "Yes" } else { "No" });

    let spinner = ProgressBar::spinner("Creating backup...");

    let client = ctx.http_client();
    let url = format!("{}/api/v1/db/backup?type={}&include_media={}", ctx.server_url(), backup_type, include_media);

    let response = client
        .post(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to create backup: {}", e)))?;

    spinner.finish_and_clear();

    if response.status().is_success() {
        // Save the backup file
        let bytes = response.bytes().await
            .map_err(|e| CliError::Network(format!("Failed to download backup: {}", e)))?;
        std::fs::write(&output_file, &bytes)?;
        println!("{}", ctx.output_format.success(&format!("Backup created: {}", output_file)));
    } else {
        println!();
        println!(
            "{}",
            ctx.output_format.info("Backup via API not available. To create a backup, use pg_dump:")
        );
        println!();
        println!(
            "  {} pg_dump $DATABASE_URL > {}",
            "$".dimmed(),
            output_file
        );

        if include_media {
            println!();
            println!("  {} # Also backup media files:", "#".dimmed());
            println!(
                "  {} tar -czf rustpress_media_{}.tar.gz ./storage/media",
                "$".dimmed(),
                timestamp
            );
        }
    }

    Ok(())
}

async fn restore_backup(ctx: &CliContext, file: &str, yes: bool) -> CliResult<()> {
    print_header("Restore Database Backup");

    print_kv("File", file);

    if !yes {
        println!();
        println!(
            "{}",
            "WARNING: This will overwrite existing data!".red().bold()
        );
        println!();
        println!("To confirm, run with --yes flag");
        return Ok(());
    }

    println!();
    println!(
        "{}",
        ctx.output_format.info("To restore a backup, use psql:")
    );
    println!();
    println!("  {} psql $DATABASE_URL < {}", "$".dimmed(), file);

    Ok(())
}

async fn run_query(ctx: &CliContext, sql: &str, explain: bool, limit: u32) -> CliResult<()> {
    // Safety check - prevent destructive queries in CLI
    let sql_upper = sql.to_uppercase();
    if sql_upper.contains("DROP") || sql_upper.contains("TRUNCATE") || sql_upper.contains("DELETE") {
        return Err(CliError::InvalidInput(
            "Destructive queries (DROP, TRUNCATE, DELETE) are not allowed via CLI. Use direct database access.".to_string()
        ));
    }

    print_header(if explain { "Query Execution Plan" } else { "Query Results" });

    let client = ctx.http_client();
    let url = format!("{}/api/v1/db/query", ctx.server_url());

    let body = serde_json::json!({
        "sql": sql,
        "explain": explain,
        "limit": limit,
    });

    let response = client
        .post(&url)
        .header("Authorization", auth_header(ctx)?)
        .json(&body)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to execute query: {}", e)))?;

    if !response.status().is_success() {
        println!("{}", ctx.output_format.error("Query execution via API is not available. Use direct database access."));
        return Ok(());
    }

    let result: serde_json::Value = response.json().await.unwrap_or_default();

    if let Some(rows) = result.get("rows").and_then(|v| v.as_array()) {
        if rows.is_empty() {
            println!("{}", ctx.output_format.info("No results"));
        } else {
            println!("{}", serde_json::to_string_pretty(&rows)?);
            println!();
            println!("{} row(s) returned", rows.len());
        }
    }

    Ok(())
}

async fn list_tables(ctx: &CliContext, verbose: bool) -> CliResult<()> {
    print_header("Database Tables");

    let client = ctx.http_client();
    let url = format!("{}/api/v1/db/tables", ctx.server_url());

    let response = client
        .get(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to fetch tables: {}", e)))?;

    if !response.status().is_success() {
        println!("{}", ctx.output_format.info("Table listing via API is not available. Use direct database access."));
        return Ok(());
    }

    let tables: Vec<TableInfo> = response.json().await.unwrap_or_default();

    if tables.is_empty() {
        println!("{}", ctx.output_format.info("No tables found"));
    } else {
        println!("{}", ctx.output_format.format(&tables));

        if verbose {
            println!();
            println!("Total tables: {}", tables.len());
        }
    }

    Ok(())
}

async fn export_table(
    ctx: &CliContext,
    table: &str,
    format: &str,
    output: Option<String>,
    where_clause: Option<String>,
) -> CliResult<()> {
    print_header(&format!("Exporting table: {}", table));

    let output_file = output.unwrap_or_else(|| format!("{}_{}.{}", table, chrono::Utc::now().format("%Y%m%d"), format));

    print_kv("Format", format);
    print_kv("Output", &output_file);
    if let Some(ref w) = where_clause {
        print_kv("Filter", w);
    }

    let spinner = ProgressBar::spinner("Exporting data...");

    let client = ctx.http_client();
    let mut url = format!("{}/api/v1/db/export/{}?format={}", ctx.server_url(), table, format);
    if let Some(ref w) = where_clause {
        url.push_str(&format!("&where={}", w));
    }

    let response = client
        .get(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to export table: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        println!("{}", ctx.output_format.info("Table export via API is not available. Use direct database access."));
        return Ok(());
    }

    let content = response.text().await.unwrap_or_default();
    std::fs::write(&output_file, content)?;

    println!("{}", ctx.output_format.success(&format!("Exported to {}", output_file)));

    Ok(())
}

async fn import_data(
    ctx: &CliContext,
    file: &str,
    table: Option<String>,
    format: Option<String>,
    dry_run: bool,
) -> CliResult<()> {
    print_header("Importing Data");

    let format = format.unwrap_or_else(|| {
        if file.ends_with(".csv") {
            "csv".to_string()
        } else if file.ends_with(".sql") {
            "sql".to_string()
        } else {
            "json".to_string()
        }
    });

    print_kv("File", file);
    print_kv("Format", &format);
    if let Some(ref t) = table {
        print_kv("Target Table", t);
    }

    if dry_run {
        println!();
        println!("{}", ctx.output_format.warning("Dry run - validating only"));
    }

    let content = std::fs::read_to_string(file)?;

    let row_count = match format.as_str() {
        "csv" => content.lines().count().saturating_sub(1),
        "json" => {
            let data: serde_json::Value = serde_json::from_str(&content)?;
            data.as_array().map(|a| a.len()).unwrap_or(0)
        }
        "sql" => content.matches("INSERT").count(),
        _ => 0,
    };

    println!();
    println!("Found {} record(s) to import", row_count);

    if !dry_run {
        println!();
        println!("{}", ctx.output_format.info("Data import via API is not available. Use direct database access or psql."));
    }

    Ok(())
}

async fn optimize_tables(ctx: &CliContext, table: Option<String>) -> CliResult<()> {
    print_header("Optimizing Database");

    let spinner = ProgressBar::spinner("Optimizing tables...");

    let client = ctx.http_client();
    let url = if let Some(ref t) = table {
        format!("{}/api/v1/db/optimize/{}", ctx.server_url(), t)
    } else {
        format!("{}/api/v1/db/optimize", ctx.server_url())
    };

    let response = client
        .post(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to optimize tables: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        println!("{}", ctx.output_format.info("Database optimization via API is not available. Use VACUUM ANALYZE directly."));
        return Ok(());
    }

    let result: serde_json::Value = response.json().await.unwrap_or_default();
    let count = result.get("optimized").and_then(|v| v.as_i64()).unwrap_or(1);

    println!(
        "{}",
        ctx.output_format.success(&format!("Optimized {} table(s)", count))
    );

    Ok(())
}

async fn audit_log(
    ctx: &CliContext,
    clear: bool,
    since: Option<String>,
    limit: u32,
) -> CliResult<()> {
    let client = ctx.http_client();

    if clear {
        print_header("Clearing Audit Log");

        let url = format!("{}/api/v1/db/audit-log", ctx.server_url());
        let response = client
            .delete(&url)
            .header("Authorization", auth_header(ctx)?)
            .send()
            .await
            .map_err(|e| CliError::Network(format!("Failed to clear audit log: {}", e)))?;

        if response.status().is_success() {
            println!("{}", ctx.output_format.success("Audit log cleared"));
        } else {
            println!("{}", ctx.output_format.info("Audit log clearing via API is not available."));
        }
        return Ok(());
    }

    print_header("Audit Log");

    let mut url = format!("{}/api/v1/db/audit-log?limit={}", ctx.server_url(), limit);
    if let Some(ref s) = since {
        url.push_str(&format!("&since={}", s));
    }

    let response = client
        .get(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to fetch audit log: {}", e)))?;

    if !response.status().is_success() {
        println!("{}", ctx.output_format.info("Audit log via API is not available."));
        return Ok(());
    }

    let logs: Vec<AuditLogEntry> = response.json().await.unwrap_or_default();

    if logs.is_empty() {
        println!("{}", ctx.output_format.info("No audit log entries found"));
    } else {
        println!("{}", ctx.output_format.format(&logs));
    }

    Ok(())
}
