//! RustPress Database Migration CLI
//!
//! A standalone migration runner for RustPress.
//!
//! Usage:
//!   migrate [OPTIONS]
//!
//! Options:
//!   --database-url <URL>  Database connection URL (or set DATABASE_URL env var)
//!   --migrations <DIR>    Migrations directory (default: ./migrations)
//!   --dry-run             Show what would be run without executing
//!   --rollback <N>        Rollback last N migrations
//!   --status              Show migration status

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Split SQL into statements, handling $$ quoted function bodies
fn split_sql_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut in_dollar_quote = false;
    let mut chars = sql.chars().peekable();

    while let Some(c) = chars.next() {
        current.push(c);

        // Handle $$ quoted strings (PostgreSQL function bodies)
        if c == '$' {
            if let Some(&next) = chars.peek() {
                if next == '$' {
                    current.push(chars.next().unwrap());
                    in_dollar_quote = !in_dollar_quote;
                    continue;
                }
            }
        }

        // Only split on semicolon when not inside $$ quotes
        if c == ';' && !in_dollar_quote {
            let stmt = current.trim().to_string();
            if !stmt.is_empty() && !is_only_comments(&stmt) {
                statements.push(stmt);
            }
            current.clear();
        }
    }

    // Add any remaining statement
    let stmt = current.trim().to_string();
    if !stmt.is_empty() && !is_only_comments(&stmt) {
        statements.push(stmt);
    }

    statements
}

/// Check if a statement contains only comments (no actual SQL)
fn is_only_comments(stmt: &str) -> bool {
    for line in stmt.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with("--") {
            return false;
        }
    }
    true
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse arguments
    let args: Vec<String> = env::args().collect();

    let mut database_url = env::var("DATABASE_URL").ok();
    let mut migrations_dir = PathBuf::from("./migrations");
    let mut dry_run = false;
    let mut show_status = false;
    let mut rollback: Option<usize> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--database-url" => {
                i += 1;
                database_url = args.get(i).cloned();
            }
            "--migrations" => {
                i += 1;
                if let Some(dir) = args.get(i) {
                    migrations_dir = PathBuf::from(dir);
                }
            }
            "--dry-run" => {
                dry_run = true;
            }
            "--status" => {
                show_status = true;
            }
            "--rollback" => {
                i += 1;
                if let Some(n) = args.get(i) {
                    rollback = n.parse().ok();
                }
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                print_help();
                return Ok(());
            }
        }
        i += 1;
    }

    let database_url = database_url.ok_or("DATABASE_URL not set")?;

    println!("RustPress Migration Tool");
    println!("========================");
    println!();

    // Connect to database
    println!("Connecting to database...");
    let pool = sqlx::PgPool::connect(&database_url).await?;
    println!("Connected!");
    println!();

    // Ensure migrations table exists
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version VARCHAR(255) PRIMARY KEY,
            applied_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )
        "#,
    )
    .execute(&pool)
    .await?;

    // Get applied migrations
    let applied: Vec<(String,)> = sqlx::query_as(
        "SELECT version FROM schema_migrations ORDER BY version",
    )
    .fetch_all(&pool)
    .await?;

    let applied_set: HashSet<String> = applied.into_iter().map(|(v,)| v).collect();

    if show_status {
        show_migration_status(&migrations_dir, &applied_set)?;
        return Ok(());
    }

    if let Some(n) = rollback {
        rollback_migrations(&pool, &migrations_dir, &applied_set, n, dry_run).await?;
        return Ok(());
    }

    // Run pending migrations
    run_migrations(&pool, &migrations_dir, &applied_set, dry_run).await?;

    println!();
    println!("Done!");

    Ok(())
}

fn print_help() {
    println!(
        r#"
RustPress Migration Tool

Usage: migrate [OPTIONS]

Options:
  --database-url <URL>  Database connection URL (or set DATABASE_URL env var)
  --migrations <DIR>    Migrations directory (default: ./migrations)
  --dry-run             Show what would be run without executing
  --rollback <N>        Rollback last N migrations
  --status              Show migration status
  --help, -h            Show this help message
"#
    );
}

fn show_migration_status(
    migrations_dir: &PathBuf,
    applied: &HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Migration Status");
    println!("-----------------");
    println!();

    let mut entries: Vec<_> = fs::read_dir(migrations_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "sql").unwrap_or(false))
        .collect();

    entries.sort_by_key(|e| e.file_name());

    for entry in &entries {
        let name = entry.file_name().to_string_lossy().to_string();
        let status = if applied.contains(&name) {
            "\x1b[32m[applied]\x1b[0m"
        } else {
            "\x1b[33m[pending]\x1b[0m"
        };
        println!("  {} {}", status, name);
    }

    println!();
    println!(
        "Applied: {} | Pending: {}",
        applied.len(),
        entries.len() - applied.len()
    );

    Ok(())
}

async fn run_migrations(
    pool: &sqlx::PgPool,
    migrations_dir: &PathBuf,
    applied: &HashSet<String>,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut entries: Vec<_> = fs::read_dir(migrations_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "sql").unwrap_or(false))
        .collect();

    entries.sort_by_key(|e| e.file_name());

    let pending: Vec<_> = entries
        .iter()
        .filter(|e| !applied.contains(&e.file_name().to_string_lossy().to_string()))
        .collect();

    if pending.is_empty() {
        println!("No pending migrations.");
        return Ok(());
    }

    println!("Pending migrations: {}", pending.len());
    println!();

    for entry in pending {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();

        if dry_run {
            println!("  [dry-run] Would apply: {}", name);
            continue;
        }

        println!("  Applying: {} ...", name);

        let sql = fs::read_to_string(&path)?;

        // Execute migration in a transaction
        let mut tx = pool.begin().await?;

        // Smart SQL splitting that handles $$ quoted function bodies
        let statements = split_sql_statements(&sql);
        for statement in statements {
            let statement = statement.trim();
            if statement.is_empty() {
                continue;
            }

            if let Err(e) = sqlx::query(statement).execute(&mut *tx).await {
                eprintln!("  Error in migration {}: {}", name, e);
                eprintln!("  Statement: {}", &statement[..statement.len().min(100)]);
                tx.rollback().await?;
                return Err(e.into());
            }
        }

        // Record migration
        sqlx::query("INSERT INTO schema_migrations (version) VALUES ($1)")
            .bind(&name)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        println!("  Applied: {}", name);
    }

    Ok(())
}

async fn rollback_migrations(
    pool: &sqlx::PgPool,
    migrations_dir: &PathBuf,
    applied: &HashSet<String>,
    count: usize,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get last N applied migrations
    let mut applied_list: Vec<_> = applied.iter().cloned().collect();
    applied_list.sort();
    applied_list.reverse();

    let to_rollback: Vec<_> = applied_list.into_iter().take(count).collect();

    if to_rollback.is_empty() {
        println!("No migrations to rollback.");
        return Ok(());
    }

    println!("Rollback {} migration(s):", to_rollback.len());
    println!();

    for name in to_rollback {
        if dry_run {
            println!("  [dry-run] Would rollback: {}", name);
            continue;
        }

        // Check for rollback file (e.g., 00001_create_users_table.down.sql)
        let rollback_name = name.replace(".sql", ".down.sql");
        let rollback_path = migrations_dir.join(&rollback_name);

        if rollback_path.exists() {
            println!("  Rolling back: {} ...", name);

            let sql = fs::read_to_string(&rollback_path)?;

            let mut tx = pool.begin().await?;

            for statement in sql.split(';') {
                let statement = statement.trim();
                if statement.is_empty() || statement.starts_with("--") {
                    continue;
                }

                if let Err(e) = sqlx::query(statement).execute(&mut *tx).await {
                    eprintln!("  Error rolling back {}: {}", name, e);
                    tx.rollback().await?;
                    return Err(e.into());
                }
            }

            // Remove migration record
            sqlx::query("DELETE FROM schema_migrations WHERE version = $1")
                .bind(&name)
                .execute(&mut *tx)
                .await?;

            tx.commit().await?;

            println!("  Rolled back: {}", name);
        } else {
            println!("  Warning: No rollback file for {}, only removing from tracking", name);

            sqlx::query("DELETE FROM schema_migrations WHERE version = $1")
                .bind(&name)
                .execute(pool)
                .await?;
        }
    }

    Ok(())
}
