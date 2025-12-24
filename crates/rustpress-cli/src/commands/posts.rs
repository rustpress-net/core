//! Post management commands

use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use tabled::Tabled;

use crate::context::CliContext;
use crate::error::{CliError, CliResult};
use crate::output::{print_header, print_kv, OutputFormatter, ProgressBar};

#[derive(Args, Debug)]
pub struct PostsCommand {
    #[command(subcommand)]
    pub command: PostsSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum PostsSubcommand {
    /// List posts
    List {
        /// Filter by status (draft, publish, pending, trash)
        #[arg(short, long)]
        status: Option<String>,

        /// Filter by author ID
        #[arg(short, long)]
        author: Option<String>,

        /// Filter by category
        #[arg(long)]
        category: Option<String>,

        /// Search in title and content
        #[arg(long)]
        search: Option<String>,

        /// Maximum number of results
        #[arg(short, long, default_value = "20")]
        limit: u32,
    },

    /// Create a new post
    Create {
        /// Post title
        #[arg(short, long)]
        title: String,

        /// Post content
        #[arg(long)]
        content: Option<String>,

        /// Read content from file
        #[arg(short, long)]
        file: Option<String>,

        /// Post status (draft, publish)
        #[arg(short, long, default_value = "draft")]
        status: String,

        /// Post slug
        #[arg(long)]
        slug: Option<String>,

        /// Author ID
        #[arg(long)]
        author: Option<String>,
    },

    /// Get post details
    Get {
        /// Post ID or slug
        post: String,
    },

    /// Update a post
    Update {
        /// Post ID or slug
        post: String,

        /// New title
        #[arg(short, long)]
        title: Option<String>,

        /// New content
        #[arg(long)]
        content: Option<String>,

        /// New status
        #[arg(long)]
        status: Option<String>,
    },

    /// Delete a post
    Delete {
        /// Post ID or slug
        post: String,

        /// Permanently delete (skip trash)
        #[arg(short, long)]
        force: bool,
    },

    /// Publish a post
    Publish {
        /// Post ID or slug
        post: String,
    },

    /// Unpublish a post
    Unpublish {
        /// Post ID or slug
        post: String,
    },

    /// Duplicate a post
    Duplicate {
        /// Post ID or slug
        post: String,
    },

    /// Bulk delete posts
    BulkDelete {
        /// Post IDs to delete
        posts: Vec<String>,

        /// Permanently delete
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Debug, Serialize, Deserialize, Tabled)]
pub struct PostRow {
    #[tabled(rename = "ID")]
    pub id: String,
    #[tabled(rename = "Title")]
    pub title: String,
    #[tabled(rename = "Status")]
    pub status: String,
    #[tabled(rename = "Author")]
    pub author: String,
    #[tabled(rename = "Date")]
    pub date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostDetails {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub status: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub author: String,
    pub author_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub published_at: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreatePostRequest {
    title: String,
    content: String,
    status: String,
    slug: Option<String>,
    author_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct UpdatePostRequest {
    title: Option<String>,
    content: Option<String>,
    status: Option<String>,
}

pub async fn execute(ctx: &CliContext, cmd: PostsCommand) -> CliResult<()> {
    match cmd.command {
        PostsSubcommand::List {
            status,
            author,
            category,
            search,
            limit,
        } => list_posts(ctx, status, author, category, search, limit).await,
        PostsSubcommand::Create {
            title,
            content,
            file,
            status,
            slug,
            author,
        } => create_post(ctx, &title, content, file, &status, slug, author).await,
        PostsSubcommand::Get { post } => get_post(ctx, &post).await,
        PostsSubcommand::Update {
            post,
            title,
            content,
            status,
        } => update_post(ctx, &post, title, content, status).await,
        PostsSubcommand::Delete { post, force } => delete_post(ctx, &post, force).await,
        PostsSubcommand::Publish { post } => publish_post(ctx, &post).await,
        PostsSubcommand::Unpublish { post } => unpublish_post(ctx, &post).await,
        PostsSubcommand::Duplicate { post } => duplicate_post(ctx, &post).await,
        PostsSubcommand::BulkDelete { posts, force } => bulk_delete(ctx, posts, force).await,
    }
}

fn auth_header(ctx: &CliContext) -> CliResult<String> {
    let token = ctx.require_auth()?;
    Ok(format!("Bearer {}", token))
}

async fn list_posts(
    ctx: &CliContext,
    status: Option<String>,
    author: Option<String>,
    category: Option<String>,
    search: Option<String>,
    limit: u32,
) -> CliResult<()> {
    print_header("Posts");

    let client = ctx.http_client();
    let mut url = format!("{}/api/v1/posts?limit={}&type=post", ctx.server_url(), limit);

    if let Some(ref s) = status {
        url.push_str(&format!("&status={}", s));
    }
    if let Some(ref a) = author {
        url.push_str(&format!("&author={}", a));
    }
    if let Some(ref c) = category {
        url.push_str(&format!("&category={}", c));
    }
    if let Some(ref s) = search {
        url.push_str(&format!("&search={}", s));
    }

    let response = client
        .get(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to fetch posts: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to list posts ({}): {}",
            status, body
        )));
    }

    let posts: Vec<PostRow> = response
        .json()
        .await
        .map_err(|e| CliError::Serialization(format!("Failed to parse response: {}", e)))?;

    if posts.is_empty() {
        println!("{}", ctx.output_format.info("No posts found"));
    } else {
        println!("{}", ctx.output_format.format(&posts));
        println!();
        println!("Total: {} post(s)", posts.len());
    }

    Ok(())
}

async fn create_post(
    ctx: &CliContext,
    title: &str,
    content: Option<String>,
    file: Option<String>,
    status: &str,
    slug: Option<String>,
    author: Option<String>,
) -> CliResult<()> {
    print_header("Creating Post");

    let content = if let Some(path) = file {
        std::fs::read_to_string(&path)?
    } else {
        content.unwrap_or_default()
    };

    let slug_display = slug.clone().unwrap_or_else(|| slugify(title));
    print_kv("Title", title);
    print_kv("Slug", &slug_display);
    print_kv("Status", status);

    let spinner = ProgressBar::spinner("Creating post...");

    let client = ctx.http_client();
    let url = format!("{}/api/v1/posts", ctx.server_url());

    let request = CreatePostRequest {
        title: title.to_string(),
        content,
        status: status.to_string(),
        slug,
        author_id: author,
    };

    let response = client
        .post(&url)
        .header("Authorization", auth_header(ctx)?)
        .json(&request)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to create post: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to create post ({}): {}",
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
        ctx.output_format.success(&format!("Post created with ID: {}", id))
    );

    Ok(())
}

async fn get_post(ctx: &CliContext, post: &str) -> CliResult<()> {
    let client = ctx.http_client();
    let url = format!("{}/api/v1/posts/{}", ctx.server_url(), post);

    let response = client
        .get(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to fetch post: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(CliError::NotFound(format!("Post not found: {}", post)));
        }
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to get post ({}): {}",
            status, body
        )));
    }

    let details: PostDetails = response
        .json()
        .await
        .map_err(|e| CliError::Serialization(format!("Failed to parse response: {}", e)))?;

    print_header("Post Details");
    print_kv("ID", &details.id);
    print_kv("Title", &details.title);
    print_kv("Slug", &details.slug);
    print_kv("Status", &details.status);
    print_kv("Author", &details.author);
    print_kv("Created", &details.created_at);
    print_kv("Updated", &details.updated_at);
    if let Some(ref published) = details.published_at {
        print_kv("Published", published);
    }
    if !details.content.is_empty() {
        println!();
        println!("Content:");
        let preview = if details.content.len() > 500 {
            format!("{}...", &details.content[..500])
        } else {
            details.content.clone()
        };
        println!("{}", preview);
    }

    Ok(())
}

async fn update_post(
    ctx: &CliContext,
    post: &str,
    title: Option<String>,
    content: Option<String>,
    status: Option<String>,
) -> CliResult<()> {
    print_header("Updating Post");

    if title.is_none() && content.is_none() && status.is_none() {
        return Err(CliError::InvalidInput("No updates specified".to_string()));
    }

    let spinner = ProgressBar::spinner("Updating post...");

    let client = ctx.http_client();
    let url = format!("{}/api/v1/posts/{}", ctx.server_url(), post);

    let request = UpdatePostRequest {
        title,
        content,
        status,
    };

    let response = client
        .patch(&url)
        .header("Authorization", auth_header(ctx)?)
        .json(&request)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to update post: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to update post ({}): {}",
            status, body
        )));
    }

    println!("{}", ctx.output_format.success(&format!("Post {} updated", post)));
    Ok(())
}

async fn delete_post(ctx: &CliContext, post: &str, force: bool) -> CliResult<()> {
    if !force {
        println!("This will move the post to trash.");
        println!("Run with --force to permanently delete.");
    }

    let spinner = ProgressBar::spinner(if force { "Deleting post..." } else { "Moving to trash..." });

    let client = ctx.http_client();
    let url = if force {
        format!("{}/api/v1/posts/{}?permanent=true", ctx.server_url(), post)
    } else {
        format!("{}/api/v1/posts/{}", ctx.server_url(), post)
    };

    let response = client
        .delete(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to delete post: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to delete post ({}): {}",
            status, body
        )));
    }

    println!(
        "{}",
        ctx.output_format.success(if force { "Post deleted" } else { "Post moved to trash" })
    );
    Ok(())
}

async fn publish_post(ctx: &CliContext, post: &str) -> CliResult<()> {
    print_header("Publishing Post");

    let spinner = ProgressBar::spinner("Publishing post...");

    let client = ctx.http_client();
    let url = format!("{}/api/v1/posts/{}/publish", ctx.server_url(), post);

    let response = client
        .post(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to publish post: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to publish post ({}): {}",
            status, body
        )));
    }

    println!("{}", ctx.output_format.success("Post published"));
    Ok(())
}

async fn unpublish_post(ctx: &CliContext, post: &str) -> CliResult<()> {
    print_header("Unpublishing Post");

    let spinner = ProgressBar::spinner("Unpublishing post...");

    let client = ctx.http_client();
    let url = format!("{}/api/v1/posts/{}/unpublish", ctx.server_url(), post);

    let response = client
        .post(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to unpublish post: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to unpublish post ({}): {}",
            status, body
        )));
    }

    println!("{}", ctx.output_format.success("Post unpublished"));
    Ok(())
}

async fn duplicate_post(ctx: &CliContext, post: &str) -> CliResult<()> {
    print_header("Duplicating Post");

    let spinner = ProgressBar::spinner("Duplicating post...");

    let client = ctx.http_client();
    let url = format!("{}/api/v1/posts/{}/duplicate", ctx.server_url(), post);

    let response = client
        .post(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to duplicate post: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to duplicate post ({}): {}",
            status, body
        )));
    }

    let created: serde_json::Value = response
        .json()
        .await
        .map_err(|e| CliError::Serialization(format!("Failed to parse response: {}", e)))?;

    let id = created.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");

    println!(
        "{}",
        ctx.output_format.success(&format!("Post duplicated with ID: {}", id))
    );
    Ok(())
}

async fn bulk_delete(ctx: &CliContext, posts: Vec<String>, force: bool) -> CliResult<()> {
    print_header(&format!("Bulk Delete {} Posts", posts.len()));

    let mut deleted = 0;
    let mut errors = 0;

    for post in &posts {
        let client = ctx.http_client();
        let url = if force {
            format!("{}/api/v1/posts/{}?permanent=true", ctx.server_url(), post)
        } else {
            format!("{}/api/v1/posts/{}", ctx.server_url(), post)
        };

        let response = client
            .delete(&url)
            .header("Authorization", auth_header(ctx)?)
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => deleted += 1,
            _ => errors += 1,
        }
    }

    println!(
        "{}",
        ctx.output_format.success(&format!(
            "{} {} post(s){}",
            if force { "Deleted" } else { "Trashed" },
            deleted,
            if errors > 0 { format!(", {} failed", errors) } else { String::new() }
        ))
    );

    Ok(())
}

/// Convert a title to a URL-friendly slug
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
