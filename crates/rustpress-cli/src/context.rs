//! CLI Context - Holds configuration and state for CLI operations

use crate::commands::Cli;
use crate::error::{CliError, CliResult};
use crate::output::OutputFormat;
use serde::{Deserialize, Serialize};

/// Stored CLI credentials for authentication
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CliCredentials {
    pub server_url: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub email: Option<String>,
}

impl CliCredentials {
    pub fn config_path() -> std::path::PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        home.join(".rustpress").join("credentials.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(creds) = serde_json::from_str(&content) {
                    return creds;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> CliResult<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn clear() -> CliResult<()> {
        let path = Self::config_path();
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    pub fn is_authenticated(&self) -> bool {
        self.access_token.is_some()
    }
}

/// CLI context containing configuration and shared state
pub struct CliContext {
    /// Output format (table, json, yaml)
    pub output_format: OutputFormat,
    /// Quiet mode - suppress non-essential output
    pub quiet: bool,
    /// Verbose level (0-3)
    pub verbose: u8,
    /// Disable colored output
    pub no_color: bool,
    /// Stored credentials
    credentials: CliCredentials,
}

impl CliContext {
    /// Create a new CLI context from parsed arguments
    pub fn new(cli: &Cli) -> CliResult<Self> {
        // Check for color support
        let no_color = cli.no_color || std::env::var("NO_COLOR").is_ok();

        if no_color {
            colored::control::set_override(false);
        }

        // Load stored credentials
        let credentials = CliCredentials::load();

        Ok(Self {
            output_format: cli.output.clone(),
            quiet: cli.quiet,
            verbose: cli.verbose,
            no_color,
            credentials,
        })
    }

    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.credentials.is_authenticated()
    }

    /// Get the current credentials
    pub fn credentials(&self) -> &CliCredentials {
        &self.credentials
    }

    /// Get the access token if authenticated
    pub fn access_token(&self) -> Option<&str> {
        self.credentials.access_token.as_deref()
    }

    /// Get the server URL
    pub fn server_url(&self) -> &str {
        if self.credentials.server_url.is_empty() {
            "http://localhost:3080"
        } else {
            &self.credentials.server_url
        }
    }

    /// Require authentication - returns error if not logged in
    pub fn require_auth(&self) -> CliResult<&str> {
        self.access_token().ok_or_else(|| {
            CliError::Auth(
                "Not authenticated. Please run 'rustpress auth login' first.".to_string(),
            )
        })
    }

    /// Print a message if not in quiet mode
    pub fn print(&self, msg: &str) {
        if !self.quiet {
            println!("{}", msg);
        }
    }

    /// Print a message at verbose level 1+
    pub fn print_verbose(&self, msg: &str) {
        if self.verbose >= 1 && !self.quiet {
            println!("{}", msg);
        }
    }

    /// Print a message at verbose level 2+
    pub fn print_debug(&self, msg: &str) {
        if self.verbose >= 2 && !self.quiet {
            println!("{}", msg);
        }
    }

    /// Print a message at verbose level 3
    pub fn print_trace(&self, msg: &str) {
        if self.verbose >= 3 && !self.quiet {
            println!("{}", msg);
        }
    }

    /// Check if verbose output is enabled
    pub fn is_verbose(&self) -> bool {
        self.verbose > 0
    }

    /// Create an HTTP client for API calls
    pub fn http_client(&self) -> reqwest::Client {
        reqwest::Client::new()
    }

    /// Create authorization header value
    pub fn auth_header(&self) -> CliResult<String> {
        let token = self.require_auth()?;
        Ok(format!("Bearer {}", token))
    }
}
