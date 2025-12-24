//! Cron/Scheduled Tasks management commands

use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use tabled::Tabled;

use crate::context::CliContext;
use crate::error::{CliError, CliResult};
use crate::output::{print_header, print_kv, OutputFormatter, ProgressBar};

#[derive(Args, Debug)]
pub struct CronCommand {
    #[command(subcommand)]
    pub command: CronSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum CronSubcommand {
    /// List scheduled tasks
    List,

    /// Show cron task details
    Get {
        /// Task ID or name
        task: String,
    },

    /// Run a scheduled task immediately
    Run {
        /// Task ID or name
        task: String,
    },

    /// Enable a scheduled task
    Enable {
        /// Task ID or name
        task: String,
    },

    /// Disable a scheduled task
    Disable {
        /// Task ID or name
        task: String,
    },

    /// Show task execution history
    History {
        /// Task ID or name (optional, shows all if not specified)
        task: Option<String>,

        /// Maximum number of entries
        #[arg(short, long, default_value = "20")]
        limit: u32,
    },

    /// Create a new scheduled task
    Create {
        /// Task name
        #[arg(short, long)]
        name: String,

        /// Cron expression (e.g., "0 0 * * *" for daily at midnight)
        #[arg(long)]
        schedule: String,

        /// Command to run
        #[arg(long)]
        command: String,

        /// Task description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Delete a scheduled task
    Delete {
        /// Task ID or name
        task: String,

        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Debug, Serialize, Deserialize, Tabled)]
pub struct CronTaskRow {
    #[tabled(rename = "ID")]
    pub id: String,
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Schedule")]
    pub schedule: String,
    #[tabled(rename = "Status")]
    pub status: String,
    #[tabled(rename = "Last Run")]
    pub last_run: String,
    #[tabled(rename = "Next Run")]
    pub next_run: String,
}

#[derive(Debug, Serialize, Deserialize, Tabled)]
pub struct CronHistoryRow {
    #[tabled(rename = "Task")]
    pub task_name: String,
    #[tabled(rename = "Started")]
    pub started_at: String,
    #[tabled(rename = "Duration")]
    pub duration: String,
    #[tabled(rename = "Status")]
    pub status: String,
}

pub async fn execute(ctx: &CliContext, cmd: CronCommand) -> CliResult<()> {
    match cmd.command {
        CronSubcommand::List => list_tasks(ctx).await,
        CronSubcommand::Get { task } => get_task(ctx, &task).await,
        CronSubcommand::Run { task } => run_task(ctx, &task).await,
        CronSubcommand::Enable { task } => enable_task(ctx, &task).await,
        CronSubcommand::Disable { task } => disable_task(ctx, &task).await,
        CronSubcommand::History { task, limit } => show_history(ctx, task, limit).await,
        CronSubcommand::Create { name, schedule, command, description } => {
            create_task(ctx, &name, &schedule, &command, description).await
        }
        CronSubcommand::Delete { task, force } => delete_task(ctx, &task, force).await,
    }
}

fn auth_header(ctx: &CliContext) -> CliResult<String> {
    let token = ctx.require_auth()?;
    Ok(format!("Bearer {}", token))
}

async fn list_tasks(ctx: &CliContext) -> CliResult<()> {
    print_header("Scheduled Tasks");

    let client = ctx.http_client();
    let url = format!("{}/api/v1/cron/tasks", ctx.server_url());

    let response = client
        .get(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to fetch tasks: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            // Fallback: Show built-in tasks info
            println!("{}", ctx.output_format.info("Cron API not available. Showing default tasks:"));
            println!();

            let default_tasks = vec![
                CronTaskRow {
                    id: "1".to_string(),
                    name: "cleanup_sessions".to_string(),
                    schedule: "0 0 * * *".to_string(),
                    status: "enabled".to_string(),
                    last_run: "-".to_string(),
                    next_run: "-".to_string(),
                },
                CronTaskRow {
                    id: "2".to_string(),
                    name: "generate_sitemap".to_string(),
                    schedule: "0 */6 * * *".to_string(),
                    status: "enabled".to_string(),
                    last_run: "-".to_string(),
                    next_run: "-".to_string(),
                },
                CronTaskRow {
                    id: "3".to_string(),
                    name: "optimize_media".to_string(),
                    schedule: "0 3 * * 0".to_string(),
                    status: "disabled".to_string(),
                    last_run: "-".to_string(),
                    next_run: "-".to_string(),
                },
            ];

            println!("{}", ctx.output_format.format(&default_tasks));
            return Ok(());
        }
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!("Failed to list tasks ({}): {}", status, body)));
    }

    let tasks: Vec<CronTaskRow> = response.json().await
        .map_err(|e| CliError::Serialization(format!("Failed to parse response: {}", e)))?;

    if tasks.is_empty() {
        println!("{}", ctx.output_format.info("No scheduled tasks found"));
    } else {
        println!("{}", ctx.output_format.format(&tasks));
        println!();
        println!("Total: {} task(s)", tasks.len());
    }

    Ok(())
}

async fn get_task(ctx: &CliContext, task: &str) -> CliResult<()> {
    let client = ctx.http_client();
    let url = format!("{}/api/v1/cron/tasks/{}", ctx.server_url(), task);

    let response = client
        .get(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to fetch task: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(CliError::NotFound(format!("Task not found: {}", task)));
        }
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!("Failed to get task ({}): {}", status, body)));
    }

    #[derive(Deserialize)]
    struct TaskDetails {
        id: String,
        name: String,
        schedule: String,
        command: String,
        description: Option<String>,
        status: String,
        last_run: Option<String>,
        next_run: Option<String>,
        run_count: i64,
    }

    let task_details: TaskDetails = response.json().await
        .map_err(|e| CliError::Serialization(format!("Failed to parse response: {}", e)))?;

    print_header("Task Details");
    print_kv("ID", &task_details.id);
    print_kv("Name", &task_details.name);
    print_kv("Schedule", &task_details.schedule);
    print_kv("Command", &task_details.command);
    if let Some(desc) = &task_details.description {
        print_kv("Description", desc);
    }
    print_kv("Status", &task_details.status);
    print_kv("Last Run", task_details.last_run.as_deref().unwrap_or("-"));
    print_kv("Next Run", task_details.next_run.as_deref().unwrap_or("-"));
    print_kv("Run Count", &task_details.run_count.to_string());

    Ok(())
}

async fn run_task(ctx: &CliContext, task: &str) -> CliResult<()> {
    print_header("Running Task");

    let spinner = ProgressBar::spinner(&format!("Running task '{}'...", task));

    let client = ctx.http_client();
    let url = format!("{}/api/v1/cron/tasks/{}/run", ctx.server_url(), task);

    let response = client
        .post(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to run task: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            println!("{}", ctx.output_format.info("Cron task execution via API is not available."));
            return Ok(());
        }
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!("Failed to run task ({}): {}", status, body)));
    }

    println!("{}", ctx.output_format.success(&format!("Task '{}' executed successfully", task)));
    Ok(())
}

async fn enable_task(ctx: &CliContext, task: &str) -> CliResult<()> {
    let client = ctx.http_client();
    let url = format!("{}/api/v1/cron/tasks/{}/enable", ctx.server_url(), task);

    let response = client
        .post(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to enable task: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!("Failed to enable task ({}): {}", status, body)));
    }

    println!("{}", ctx.output_format.success(&format!("Task '{}' enabled", task)));
    Ok(())
}

async fn disable_task(ctx: &CliContext, task: &str) -> CliResult<()> {
    let client = ctx.http_client();
    let url = format!("{}/api/v1/cron/tasks/{}/disable", ctx.server_url(), task);

    let response = client
        .post(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to disable task: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!("Failed to disable task ({}): {}", status, body)));
    }

    println!("{}", ctx.output_format.success(&format!("Task '{}' disabled", task)));
    Ok(())
}

async fn show_history(ctx: &CliContext, task: Option<String>, limit: u32) -> CliResult<()> {
    print_header("Task Execution History");

    let client = ctx.http_client();
    let url = match &task {
        Some(t) => format!("{}/api/v1/cron/tasks/{}/history?limit={}", ctx.server_url(), t, limit),
        None => format!("{}/api/v1/cron/history?limit={}", ctx.server_url(), limit),
    };

    let response = client
        .get(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to fetch history: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            println!("{}", ctx.output_format.info("No execution history available"));
            return Ok(());
        }
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!("Failed to get history ({}): {}", status, body)));
    }

    let history: Vec<CronHistoryRow> = response.json().await
        .map_err(|e| CliError::Serialization(format!("Failed to parse response: {}", e)))?;

    if history.is_empty() {
        println!("{}", ctx.output_format.info("No execution history found"));
    } else {
        println!("{}", ctx.output_format.format(&history));
    }

    Ok(())
}

async fn create_task(
    ctx: &CliContext,
    name: &str,
    schedule: &str,
    command: &str,
    description: Option<String>,
) -> CliResult<()> {
    print_header("Creating Scheduled Task");

    #[derive(Serialize)]
    struct CreateTaskRequest {
        name: String,
        schedule: String,
        command: String,
        description: Option<String>,
    }

    let client = ctx.http_client();
    let url = format!("{}/api/v1/cron/tasks", ctx.server_url());

    let response = client
        .post(&url)
        .header("Authorization", auth_header(ctx)?)
        .json(&CreateTaskRequest {
            name: name.to_string(),
            schedule: schedule.to_string(),
            command: command.to_string(),
            description,
        })
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to create task: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            println!("{}", ctx.output_format.info("Cron task creation via API is not available."));
            return Ok(());
        }
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!("Failed to create task ({}): {}", status, body)));
    }

    println!("{}", ctx.output_format.success(&format!("Task '{}' created successfully", name)));
    Ok(())
}

async fn delete_task(ctx: &CliContext, task: &str, force: bool) -> CliResult<()> {
    if !force {
        use dialoguer::Confirm;
        let confirmed = Confirm::new()
            .with_prompt(format!("Are you sure you want to delete task '{}'?", task))
            .default(false)
            .interact()
            .map_err(|e| CliError::InvalidInput(format!("Failed to get confirmation: {}", e)))?;

        if !confirmed {
            println!("Operation cancelled.");
            return Ok(());
        }
    }

    let client = ctx.http_client();
    let url = format!("{}/api/v1/cron/tasks/{}", ctx.server_url(), task);

    let response = client
        .delete(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to delete task: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!("Failed to delete task ({}): {}", status, body)));
    }

    println!("{}", ctx.output_format.success(&format!("Task '{}' deleted", task)));
    Ok(())
}
