//! Authentication commands for CLI login

use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};

use crate::context::{CliContext, CliCredentials};
use crate::error::{CliError, CliResult};
use crate::output::{print_header, print_kv, OutputFormatter};

#[derive(Args, Debug)]
pub struct AuthCommand {
    #[command(subcommand)]
    pub command: AuthSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum AuthSubcommand {
    /// Login to RustPress and store credentials
    Login {
        /// Email address
        #[arg(short, long)]
        email: Option<String>,
        /// Password (will prompt if not provided)
        #[arg(short, long)]
        password: Option<String>,
        /// Server URL (default: http://localhost:3080)
        #[arg(short, long)]
        server: Option<String>,
    },
    /// Logout and clear stored credentials
    Logout,
    /// Show current logged-in user
    Whoami,
    /// Display or manage tokens
    Token {
        /// Show the current token
        #[arg(long)]
        show: bool,
        /// Refresh the token
        #[arg(long)]
        refresh: bool,
    },
    /// Configure authentication settings
    Config {
        /// Set the server URL
        #[arg(long)]
        server: Option<String>,
        /// Show current configuration
        #[arg(long)]
        show: bool,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    access_token: String,
    refresh_token: Option<String>,
    token_type: String,
    expires_in: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserInfo {
    id: String,
    email: String,
    display_name: Option<String>,
    role: String,
}

pub async fn execute(ctx: &CliContext, cmd: AuthCommand) -> CliResult<()> {
    match cmd.command {
        AuthSubcommand::Login { email, password, server } => login(ctx, email, password, server).await,
        AuthSubcommand::Logout => logout(ctx).await,
        AuthSubcommand::Whoami => whoami(ctx).await,
        AuthSubcommand::Token { show, refresh } => manage_token(ctx, show, refresh).await,
        AuthSubcommand::Config { server, show } => configure(ctx, server, show).await,
    }
}

async fn login(ctx: &CliContext, email: Option<String>, password: Option<String>, server: Option<String>) -> CliResult<()> {
    print_header("Login to RustPress");

    let server_url = server.unwrap_or_else(|| {
        std::env::var("RUSTPRESS_SERVER_URL").unwrap_or_else(|_| "http://localhost:3080".to_string())
    });

    // Get email - from arg, env, or prompt
    let email = match email {
        Some(e) => e,
        None => {
            print!("Email: ");
            std::io::Write::flush(&mut std::io::stdout())?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            input.trim().to_string()
        }
    };

    // Get password - from arg, env, or prompt
    let password = match password {
        Some(p) => p,
        None => {
            // Use rpassword for hidden input if available, otherwise plain input
            print!("Password: ");
            std::io::Write::flush(&mut std::io::stdout())?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            input.trim().to_string()
        }
    };

    if email.is_empty() || password.is_empty() {
        return Err(CliError::InvalidInput("Email and password are required".to_string()));
    }

    // Make login request
    let client = reqwest::Client::new();
    let login_url = format!("{}/api/v1/auth/login", server_url);

    let response = client
        .post(&login_url)
        .json(&LoginRequest { email: email.clone(), password })
        .send()
        .await
        .map_err(|e| CliError::Network(format!("Failed to connect to server: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::Auth(format!("Login failed ({}): {}", status, body)));
    }

    let login_response: LoginResponse = response
        .json()
        .await
        .map_err(|e| CliError::Serialization(format!("Failed to parse login response: {}", e)))?;

    // Save credentials
    let creds = CliCredentials {
        server_url: server_url.clone(),
        access_token: Some(login_response.access_token),
        refresh_token: login_response.refresh_token,
        email: Some(email.clone()),
    };
    creds.save()?;

    println!("{}", ctx.output_format.success(&format!("Logged in as {} on {}", email, server_url)));
    Ok(())
}

async fn logout(ctx: &CliContext) -> CliResult<()> {
    print_header("Logout from RustPress");

    let creds = CliCredentials::load();
    if creds.access_token.is_none() {
        println!("{}", ctx.output_format.info("Not currently logged in"));
        return Ok(());
    }

    CliCredentials::clear()?;
    println!("{}", ctx.output_format.success("Successfully logged out"));
    Ok(())
}

async fn whoami(ctx: &CliContext) -> CliResult<()> {
    let creds = CliCredentials::load();

    match (&creds.access_token, &creds.email) {
        (Some(token), Some(email)) => {
            print_header("Current User");
            print_kv("Email", email);
            print_kv("Server", &creds.server_url);

            // Try to get more user info from the server
            let client = reqwest::Client::new();
            let url = format!("{}/api/v1/users/me", creds.server_url);

            match client
                .get(&url)
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    if let Ok(user_info) = response.json::<UserInfo>().await {
                        if let Some(name) = user_info.display_name {
                            print_kv("Name", &name);
                        }
                        print_kv("Role", &user_info.role);
                        print_kv("ID", &user_info.id);
                    }
                }
                Ok(response) if response.status() == reqwest::StatusCode::UNAUTHORIZED => {
                    println!();
                    println!("{}", ctx.output_format.warning("Token may be expired. Run 'rustpress auth login' to re-authenticate."));
                }
                _ => {
                    println!();
                    println!("{}", ctx.output_format.warning("Could not fetch user details from server"));
                }
            }
        }
        _ => {
            println!("{}", ctx.output_format.info("Not logged in. Run 'rustpress auth login' to authenticate."));
        }
    }

    Ok(())
}

async fn manage_token(ctx: &CliContext, show: bool, refresh: bool) -> CliResult<()> {
    let mut creds = CliCredentials::load();

    // Default to showing token if neither show nor refresh is specified
    let should_show = show || !refresh;

    if refresh {
        print_header("Refreshing Token");

        let refresh_token = creds.refresh_token.clone().ok_or_else(|| {
            CliError::Auth("No refresh token available. Please login again.".to_string())
        })?;

        let client = reqwest::Client::new();
        let url = format!("{}/api/v1/auth/refresh", creds.server_url);

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", refresh_token))
            .send()
            .await
            .map_err(|e| CliError::Network(format!("Failed to refresh token: {}", e)))?;

        if !response.status().is_success() {
            return Err(CliError::Auth("Token refresh failed. Please login again.".to_string()));
        }

        let login_response: LoginResponse = response
            .json()
            .await
            .map_err(|e| CliError::Serialization(format!("Failed to parse refresh response: {}", e)))?;

        creds.access_token = Some(login_response.access_token);
        if let Some(new_refresh) = login_response.refresh_token {
            creds.refresh_token = Some(new_refresh);
        }
        creds.save()?;

        println!("{}", ctx.output_format.success("Token refreshed successfully"));
        return Ok(());
    }

    if should_show {
        print_header("Current Token");
        match &creds.access_token {
            Some(token) => {
                // Show truncated token for security
                let display_token = if token.len() > 20 {
                    format!("{}...{}", &token[..10], &token[token.len()-10..])
                } else {
                    token.clone()
                };
                print_kv("Access Token", &display_token);
                print_kv("Server", &creds.server_url);
            }
            None => {
                println!("{}", ctx.output_format.info("No token stored. Run 'rustpress auth login' to authenticate."));
            }
        }
    }

    Ok(())
}

async fn configure(ctx: &CliContext, server: Option<String>, _show: bool) -> CliResult<()> {
    let mut creds = CliCredentials::load();

    if let Some(server_url) = server {
        creds.server_url = server_url.clone();
        creds.save()?;
        println!("{}", ctx.output_format.success(&format!("Server URL set to: {}", server_url)));
        return Ok(());
    }

    // Default to showing config if no server option is provided
    // This also handles the explicit --show flag
    print_header("Authentication Configuration");
    print_kv("Server URL", if creds.server_url.is_empty() { "(not set)" } else { &creds.server_url });
    print_kv("Logged In", if creds.access_token.is_some() { "Yes" } else { "No" });
    if let Some(email) = &creds.email {
        print_kv("Email", email);
    }

    Ok(())
}
