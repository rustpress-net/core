//! User management commands

use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use tabled::Tabled;

use crate::context::CliContext;
use crate::error::{CliError, CliResult};
use crate::output::{print_header, print_kv, OutputFormatter, ProgressBar};

#[derive(Args, Debug)]
pub struct UsersCommand {
    #[command(subcommand)]
    pub command: UsersSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum UsersSubcommand {
    /// List users
    List {
        /// Filter by role
        #[arg(short, long)]
        role: Option<String>,

        /// Filter by status
        #[arg(short, long)]
        status: Option<String>,

        /// Maximum number of results
        #[arg(short, long, default_value = "50")]
        limit: u32,
    },

    /// Create a new user
    Create {
        /// User email address
        #[arg(short, long)]
        email: String,

        /// User password
        #[arg(short, long)]
        password: String,

        /// User display name
        #[arg(short, long)]
        name: Option<String>,

        /// User role
        #[arg(short, long, default_value = "subscriber")]
        role: String,
    },

    /// Create an admin user
    CreateAdmin {
        /// Admin email address
        #[arg(short, long)]
        email: String,

        /// Admin password
        #[arg(short, long)]
        password: String,

        /// Admin display name
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Get user details
    Get {
        /// User ID or email
        user: String,
    },

    /// Update user
    Update {
        /// User ID or email
        user: String,

        /// New email address
        #[arg(short, long)]
        email: Option<String>,

        /// New display name
        #[arg(short, long)]
        name: Option<String>,

        /// New role
        #[arg(short, long)]
        role: Option<String>,

        /// New status
        #[arg(long)]
        status: Option<String>,
    },

    /// Delete user
    Delete {
        /// User ID or email
        user: String,

        /// Force deletion without confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Reset user password
    ResetPassword {
        /// User ID or email
        user: String,

        /// New password (will prompt if not provided)
        #[arg(short, long)]
        password: Option<String>,

        /// Generate a random password
        #[arg(long)]
        generate: bool,
    },
}

#[derive(Debug, Serialize, Deserialize, Tabled)]
pub struct UserRow {
    #[tabled(rename = "ID")]
    pub id: String,
    #[tabled(rename = "Email")]
    pub email: String,
    #[tabled(rename = "Name")]
    pub display_name: String,
    #[tabled(rename = "Role")]
    pub role: String,
    #[tabled(rename = "Status")]
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateUserRequest {
    email: String,
    password: String,
    display_name: Option<String>,
    role: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateUserRequest {
    email: Option<String>,
    display_name: Option<String>,
    role: Option<String>,
    status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResetPasswordRequest {
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse<T> {
    data: Option<T>,
    error: Option<String>,
}

pub async fn execute(ctx: &CliContext, cmd: UsersCommand) -> CliResult<()> {
    match cmd.command {
        UsersSubcommand::List { role, status, limit } => list_users(ctx, role, status, limit).await,
        UsersSubcommand::Create { email, password, name, role } => {
            create_user(ctx, email, password, name, role).await
        }
        UsersSubcommand::CreateAdmin { email, password, name } => {
            create_user(ctx, email, password, name, "administrator".to_string()).await
        }
        UsersSubcommand::Get { user } => get_user(ctx, &user).await,
        UsersSubcommand::Update { user, email, name, role, status } => {
            update_user(ctx, &user, email, name, role, status).await
        }
        UsersSubcommand::Delete { user, force } => delete_user(ctx, &user, force).await,
        UsersSubcommand::ResetPassword { user, password, generate } => {
            reset_password(ctx, &user, password, generate).await
        }
    }
}

fn api_client(ctx: &CliContext) -> CliResult<reqwest::Client> {
    Ok(reqwest::Client::new())
}

fn auth_header(ctx: &CliContext) -> CliResult<String> {
    let token = ctx.require_auth()?;
    Ok(format!("Bearer {}", token))
}

async fn list_users(
    ctx: &CliContext,
    role: Option<String>,
    status: Option<String>,
    limit: u32,
) -> CliResult<()> {
    print_header("Users");

    let client = api_client(ctx)?;
    let mut url = format!("{}/api/v1/users?limit={}", ctx.server_url(), limit);

    if let Some(r) = role {
        url.push_str(&format!("&role={}", r));
    }
    if let Some(s) = status {
        url.push_str(&format!("&status={}", s));
    }

    let response = client
        .get(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to fetch users: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to list users ({}): {}",
            status, body
        )));
    }

    let users: Vec<UserRow> = response
        .json()
        .await
        .map_err(|e| CliError::Serialization(format!("Failed to parse response: {}", e)))?;

    if users.is_empty() {
        println!("{}", ctx.output_format.info("No users found"));
    } else {
        println!("{}", ctx.output_format.format(&users));
    }

    Ok(())
}

async fn create_user(
    ctx: &CliContext,
    email: String,
    password: String,
    name: Option<String>,
    role: String,
) -> CliResult<()> {
    print_header("Creating User");

    let spinner = ProgressBar::spinner("Creating user...");

    let client = api_client(ctx)?;
    let url = format!("{}/api/v1/users", ctx.server_url());

    let request = CreateUserRequest {
        email: email.clone(),
        password,
        display_name: name,
        role: role.clone(),
    };

    let response = client
        .post(&url)
        .header("Authorization", auth_header(ctx)?)
        .json(&request)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to create user: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to create user ({}): {}",
            status, body
        )));
    }

    println!(
        "{}",
        ctx.output_format
            .success(&format!("Created user: {} ({})", email, role))
    );
    Ok(())
}

async fn get_user(ctx: &CliContext, user: &str) -> CliResult<()> {
    print_header("User Details");

    let client = api_client(ctx)?;
    let url = format!("{}/api/v1/users/{}", ctx.server_url(), user);

    let response = client
        .get(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to fetch user: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(CliError::NotFound(format!("User not found: {}", user)));
        }
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to get user ({}): {}",
            status, body
        )));
    }

    let user_data: UserRow = response
        .json()
        .await
        .map_err(|e| CliError::Serialization(format!("Failed to parse response: {}", e)))?;

    print_kv("ID", &user_data.id);
    print_kv("Email", &user_data.email);
    print_kv("Name", &user_data.display_name);
    print_kv("Role", &user_data.role);
    print_kv("Status", &user_data.status);

    Ok(())
}

async fn update_user(
    ctx: &CliContext,
    user: &str,
    email: Option<String>,
    name: Option<String>,
    role: Option<String>,
    status: Option<String>,
) -> CliResult<()> {
    print_header("Updating User");

    if email.is_none() && name.is_none() && role.is_none() && status.is_none() {
        return Err(CliError::InvalidInput(
            "No update fields provided. Use --email, --name, --role, or --status".to_string(),
        ));
    }

    let spinner = ProgressBar::spinner("Updating user...");

    let client = api_client(ctx)?;
    let url = format!("{}/api/v1/users/{}", ctx.server_url(), user);

    let request = UpdateUserRequest {
        email,
        display_name: name,
        role,
        status,
    };

    let response = client
        .patch(&url)
        .header("Authorization", auth_header(ctx)?)
        .json(&request)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to update user: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to update user ({}): {}",
            status, body
        )));
    }

    println!(
        "{}",
        ctx.output_format
            .success(&format!("Updated user: {}", user))
    );
    Ok(())
}

async fn delete_user(ctx: &CliContext, user: &str, force: bool) -> CliResult<()> {
    if !force {
        println!("This will permanently delete user: {}", user);
        println!("Run with --force to confirm deletion.");
        return Ok(());
    }

    print_header("Deleting User");

    let spinner = ProgressBar::spinner("Deleting user...");

    let client = api_client(ctx)?;
    let url = format!("{}/api/v1/users/{}", ctx.server_url(), user);

    let response = client
        .delete(&url)
        .header("Authorization", auth_header(ctx)?)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to delete user: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to delete user ({}): {}",
            status, body
        )));
    }

    println!(
        "{}",
        ctx.output_format
            .success(&format!("Deleted user: {}", user))
    );
    Ok(())
}

async fn reset_password(
    ctx: &CliContext,
    user: &str,
    password: Option<String>,
    generate: bool,
) -> CliResult<()> {
    print_header("Reset Password");

    let new_password = if generate {
        // Generate a random password
        use rand::Rng;
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*";
        let mut rng = rand::thread_rng();
        let password: String = (0..16)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        println!("Generated password: {}", password);
        password
    } else {
        password.ok_or_else(|| {
            CliError::InvalidInput(
                "Password required. Use --password or --generate".to_string(),
            )
        })?
    };

    let spinner = ProgressBar::spinner("Resetting password...");

    let client = api_client(ctx)?;
    let url = format!("{}/api/v1/users/{}/password", ctx.server_url(), user);

    let request = ResetPasswordRequest {
        password: new_password,
    };

    let response = client
        .put(&url)
        .header("Authorization", auth_header(ctx)?)
        .json(&request)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to reset password: {}", e)))?;

    spinner.finish_and_clear();

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::OperationFailed(format!(
            "Failed to reset password ({}): {}",
            status, body
        )));
    }

    println!(
        "{}",
        ctx.output_format
            .success(&format!("Password reset for user: {}", user))
    );
    Ok(())
}
