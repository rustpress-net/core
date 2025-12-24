//! Page management commands

use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use tabled::Tabled;

use crate::context::CliContext;
use crate::error::{CliError, CliResult};
use crate::output::{print_header, print_kv, OutputFormatter, ProgressBar};

#[derive(Args, Debug)]
pub struct PagesCommand {
    #[command(subcommand)]
    pub command: PagesSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum PagesSubcommand {
    /// List pages
    List {
        /// Filter by status
        #[arg(short, long)]
        status: Option<String>,

        /// Filter by parent page ID
        #[arg(short, long)]
        parent: Option<String>,

        /// Maximum number of results
        #[arg(short, long, default_value = "50")]
        limit: u32,
    },

    /// Create a new page
    Create {
        /// Page title
        #[arg(short, long)]
        title: String,

        /// Page content
        #[arg(long)]
        content: Option<String>,

        /// Page template
        #[arg(long)]
        template: Option<String>,

        /// Parent page ID
        #[arg(long)]
        parent: Option<String>,

        /// Page status
        #[arg(long, default_value = "draft")]
        status: String,
    },

    /// Get page details
    Get {
        /// Page ID or slug
        page: String,
    },

    /// Update a page
    Update {
        /// Page ID or slug
        page: String,

        /// New title
        #[arg(long)]
        title: Option<String>,

        /// New content
        #[arg(long)]
        content: Option<String>,

        /// New status
        #[arg(long)]
        status: Option<String>,

        /// New template
        #[arg(long)]
        template: Option<String>,
    },

    /// Delete a page
    Delete {
        /// Page ID or slug
        page: String,

        /// Permanently delete
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Debug, Serialize, Deserialize, Tabled)]
pub struct PageRow {
    #[tabled(rename = "ID")]
    pub id: String,
    #[tabled(rename = "Title")]
    pub title: String,
    #[tabled(rename = "Status")]
    pub status: String,
    #[tabled(rename = "Template")]
    pub template: String,
    #[tabled(rename = "Order")]
    pub menu_order: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageDetails {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub status: String,
    pub content: String,
    pub template: Option<String>,
    pub parent_id: Option<String>,
    pub menu_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
struct CreatePageRequest {
    title: String,
    content: String,
    status: String,
    template: Option<String>,
    parent_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct UpdatePageRequest {
    title: Option<String>,
    content: Option<String>,
    status: Option<String>,
    template: Option<String>,
}

pub async fn execute(ctx: &CliContext, cmd: PagesCommand) -> CliResult<()> {
    match cmd.command {
        PagesSubcommand::List {
            status,
            parent,
            limit,
        } => list_pages(ctx, status, parent, limit).await,
        PagesSubcommand::Create {
            title,
            content,
            template,
            parent,
            status,
        } => create_page(ctx, &title, content, template, parent, &status).await,
        PagesSubcommand::Get { page } => get_page(ctx, &page).await,
        PagesSubcommand::Update {
            page,
            title,
            content,
            status,
            template,
        } => update_page(ctx, &page, title, content, status, template).await,
        PagesSubcommand::Delete { page, force } => delete_page(ctx, &page, force).await,
    }
}

fn auth_header(ctx: &CliContext) -> CliResult<String> {
    let token = ctx.require_auth()?;
    Ok(format!("Bearer {}", token))
}

async fn list_pages(
    ctx: &CliContext,
    status: Option<String>,
    parent: Option<String>,
    limit: u32,
) -> CliResult<()> {
    print_header("Pages");

    let client = ctx.http_client();
    let mut url = format!("{}/api/v1/pages?limit={}", ctx.server_url(), limit);

    if let Some(ref s) = status {
        url.push_str(&format!("&status={}", s));
    }
    if let Some(ref p) = parent {
        url.push_str(&format!("&parent={}", p));
    }

    let response = client
        .get(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to fetch pages: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to list pages ({}): {}",
            status, body
        )));
    }

    let pages: Vec<PageRow> = response
        .json()
        .await
        .map_err(|e| CliError::Serialization(format!("Failed to parse response: {}", e)))?;

    if pages.is_empty() {
        println!("{}", ctx.output_format.info("No pages found"));
    } else {
        println!("{}", ctx.output_format.format(&pages));
    }

    Ok(())
}

async fn create_page(
    ctx: &CliContext,
    title: &str,
    content: Option<String>,
    template: Option<String>,
    parent: Option<String>,
    status: &str,
) -> CliResult<()> {
    print_header("Creating Page");

    let slug = slugify(title);

    print_kv("Title", title);
    print_kv("Slug", &slug);
    print_kv("Status", status);

    let spinner = ProgressBar::spinner("Creating page...");

    let client = ctx.http_client();
    let url = format!("{}/api/v1/pages", ctx.server_url());

    let request = CreatePageRequest {
        title: title.to_string(),
        content: content.unwrap_or_default(),
        status: status.to_string(),
        template,
        parent_id: parent,
    };

    let response = client
        .post(&url)
        .header("Authorization", auth_header(ctx)?)
        .json(&request)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to create page: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to create page ({}): {}",
            status, body
        )));
    }

    let created: serde_json::Value = response
        .json()
        .await
        .map_err(|e| CliError::Serialization(format!("Failed to parse response: {}", e)))?;

    let id = created.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");

    println!();
    println!(
        "{}",
        ctx.output_format.success(&format!("Page created with ID: {}", id))
    );

    Ok(())
}

async fn get_page(ctx: &CliContext, page: &str) -> CliResult<()> {
    let client = ctx.http_client();
    let url = format!("{}/api/v1/pages/{}", ctx.server_url(), page);

    let response = client
        .get(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to fetch page: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(CliError::NotFound(format!("Page not found: {}", page)));
        }
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to get page ({}): {}",
            status, body
        )));
    }

    let details: PageDetails = response
        .json()
        .await
        .map_err(|e| CliError::Serialization(format!("Failed to parse response: {}", e)))?;

    print_header("Page Details");
    print_kv("ID", &details.id);
    print_kv("Title", &details.title);
    print_kv("Slug", &details.slug);
    print_kv("Status", &details.status);
    if let Some(ref template) = details.template {
        print_kv("Template", template);
    }
    if let Some(ref parent) = details.parent_id {
        print_kv("Parent", parent);
    }
    print_kv("Menu Order", &details.menu_order.to_string());
    print_kv("Created", &details.created_at);
    print_kv("Updated", &details.updated_at);

    Ok(())
}

async fn update_page(
    ctx: &CliContext,
    page: &str,
    title: Option<String>,
    content: Option<String>,
    status: Option<String>,
    template: Option<String>,
) -> CliResult<()> {
    print_header("Updating Page");

    if title.is_none() && content.is_none() && status.is_none() && template.is_none() {
        return Err(CliError::InvalidInput("No updates specified".to_string()));
    }

    let spinner = ProgressBar::spinner("Updating page...");

    let client = ctx.http_client();
    let url = format!("{}/api/v1/pages/{}", ctx.server_url(), page);

    let request = UpdatePageRequest {
        title,
        content,
        status,
        template,
    };

    let response = client
        .patch(&url)
        .header("Authorization", auth_header(ctx)?)
        .json(&request)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to update page: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to update page ({}): {}",
            status, body
        )));
    }

    println!("{}", ctx.output_format.success(&format!("Page {} updated", page)));
    Ok(())
}

async fn delete_page(ctx: &CliContext, page: &str, force: bool) -> CliResult<()> {
    if !force {
        println!("This will move the page to trash.");
        println!("Run with --force to permanently delete.");
    }

    let spinner = ProgressBar::spinner(if force { "Deleting page..." } else { "Moving to trash..." });

    let client = ctx.http_client();
    let url = if force {
        format!("{}/api/v1/pages/{}?permanent=true", ctx.server_url(), page)
    } else {
        format!("{}/api/v1/pages/{}", ctx.server_url(), page)
    };

    let response = client
        .delete(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to delete page: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to delete page ({}): {}",
            status, body
        )));
    }

    println!(
        "{}",
        ctx.output_format.success(if force { "Page deleted" } else { "Page moved to trash" })
    );
    Ok(())
}

fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
