//! Interactive REPL (Read-Eval-Print Loop) for RustPress CLI
//!
//! Provides an interactive shell with command history, auto-completion,
//! and persistent session management.

use clap::Parser;
use colored::Colorize;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Config, Editor, Helper};
use std::borrow::Cow;

use crate::commands::{Cli, Commands};
use crate::context::{CliContext, CliCredentials};
use crate::error::{CliError, CliResult};
use crate::output::print_header;

/// REPL command completer
#[derive(Default)]
struct ReplCompleter {
    commands: Vec<String>,
}

impl ReplCompleter {
    fn new() -> Self {
        Self {
            commands: vec![
                // Main commands
                "auth".into(),
                "server".into(),
                "db".into(),
                "users".into(),
                "posts".into(),
                "pages".into(),
                "media".into(),
                "themes".into(),
                "plugins".into(),
                "cache".into(),
                "settings".into(),
                "backup".into(),
                "seo".into(),
                "config".into(),
                // Auth subcommands
                "auth login".into(),
                "auth logout".into(),
                "auth whoami".into(),
                "auth token".into(),
                // Server subcommands
                "server start".into(),
                "server stop".into(),
                "server status".into(),
                "server health".into(),
                // DB subcommands
                "db migrate".into(),
                "db status".into(),
                "db backup".into(),
                "db restore".into(),
                "db tables".into(),
                "db optimize".into(),
                // User subcommands
                "users list".into(),
                "users create".into(),
                "users get".into(),
                "users delete".into(),
                "users update".into(),
                "users import".into(),
                "users export".into(),
                // Posts subcommands
                "posts list".into(),
                "posts create".into(),
                "posts get".into(),
                "posts update".into(),
                "posts delete".into(),
                "posts publish".into(),
                "posts unpublish".into(),
                // Pages subcommands
                "pages list".into(),
                "pages create".into(),
                "pages get".into(),
                "pages update".into(),
                "pages delete".into(),
                // Media subcommands
                "media list".into(),
                "media upload".into(),
                "media get".into(),
                "media delete".into(),
                "media optimize".into(),
                // Theme subcommands
                "themes list".into(),
                "themes get".into(),
                "themes activate".into(),
                "themes install".into(),
                "themes delete".into(),
                // Plugin subcommands
                "plugins list".into(),
                "plugins get".into(),
                "plugins activate".into(),
                "plugins deactivate".into(),
                "plugins install".into(),
                // Cache subcommands
                "cache stats".into(),
                "cache clear".into(),
                "cache warm".into(),
                // Settings subcommands
                "settings list".into(),
                "settings get".into(),
                "settings set".into(),
                // Backup subcommands
                "backup create".into(),
                "backup list".into(),
                "backup restore".into(),
                "backup delete".into(),
                // SEO subcommands
                "seo sitemap".into(),
                "seo analyze".into(),
                "seo robots".into(),
                // Config subcommands
                "config show".into(),
                "config validate".into(),
                "config init".into(),
                // Health check
                "health".into(),
                "health check".into(),
                // System info
                "info".into(),
                "system".into(),
                // REPL commands
                "help".into(),
                "exit".into(),
                "quit".into(),
                "clear".into(),
                "history".into(),
                "status".into(),
            ],
        }
    }
}

impl Completer for ReplCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let line_up_to_cursor = &line[..pos];
        let start = line_up_to_cursor
            .rfind(char::is_whitespace)
            .map(|i| i + 1)
            .unwrap_or(0);
        let word = &line_up_to_cursor[start..];

        let matches: Vec<Pair> = self
            .commands
            .iter()
            .filter(|cmd| cmd.starts_with(line_up_to_cursor) || cmd.starts_with(word))
            .map(|cmd| {
                let display = if cmd.starts_with(line_up_to_cursor) {
                    cmd[line_up_to_cursor.len()..].to_string()
                } else {
                    cmd[word.len()..].to_string()
                };
                Pair {
                    display: cmd.clone(),
                    replacement: display,
                }
            })
            .collect();

        Ok((pos, matches))
    }
}

impl Hinter for ReplCompleter {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &rustyline::Context<'_>) -> Option<String> {
        if pos < line.len() {
            return None;
        }

        self.commands
            .iter()
            .find(|cmd| cmd.starts_with(line) && cmd.len() > line.len())
            .map(|cmd| cmd[line.len()..].to_string())
    }
}

impl Highlighter for ReplCompleter {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(format!("\x1b[90m{}\x1b[0m", hint))
    }
}

impl Validator for ReplCompleter {}

impl Helper for ReplCompleter {}

/// Run the interactive REPL
pub async fn run_repl() -> CliResult<()> {
    print_header("RustPress Interactive Shell");
    println!("{}", "Type 'help' for available commands, 'exit' to quit.".dimmed());
    println!();

    // Check authentication status
    let creds = CliCredentials::load();
    if creds.access_token.is_some() {
        if let Some(email) = &creds.email {
            println!("{} {} on {}",
                "Logged in as".green(),
                email.cyan().bold(),
                creds.server_url.cyan()
            );
        }
    } else {
        println!("{}", "Not logged in. Use 'auth login' to authenticate.".yellow());
    }
    println!();

    // Configure rustyline
    let config = Config::builder()
        .history_ignore_space(true)
        .history_ignore_dups(true)
        .map_err(|e| CliError::InvalidInput(format!("Config error: {}", e)))?
        .max_history_size(1000)
        .map_err(|e| CliError::InvalidInput(format!("Config error: {}", e)))?
        .build();

    let mut rl: Editor<ReplCompleter, _> = Editor::with_config(config)
        .map_err(|e| CliError::InvalidInput(format!("Failed to create readline: {}", e)))?;
    rl.set_helper(Some(ReplCompleter::new()));

    // Load history
    let history_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".rustpress")
        .join("history");
    let _ = rl.load_history(&history_path);

    loop {
        // Build prompt based on auth status
        let creds = CliCredentials::load();
        let prompt = if creds.access_token.is_some() {
            format!("{} ", "rustpress>".green().bold())
        } else {
            format!("{} ", "rustpress>".yellow().bold())
        };

        match rl.readline(&prompt) {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                let _ = rl.add_history_entry(line);

                // Handle REPL-specific commands
                match line {
                    "exit" | "quit" | "q" => {
                        println!("{}", "Goodbye!".cyan());
                        break;
                    }
                    "clear" | "cls" => {
                        print!("\x1B[2J\x1B[1;1H");
                        continue;
                    }
                    "help" | "?" => {
                        print_repl_help();
                        continue;
                    }
                    "history" => {
                        for (i, entry) in rl.history().iter().enumerate() {
                            println!("{:4}  {}", i + 1, entry);
                        }
                        continue;
                    }
                    "status" => {
                        print_status();
                        continue;
                    }
                    "info" | "system" => {
                        print_system_info().await;
                        continue;
                    }
                    _ => {}
                }

                // Parse and execute the command
                if let Err(e) = execute_command(line).await {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                }
                println!();
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}", "Use 'exit' or 'quit' to leave the shell.".dimmed());
            }
            Err(ReadlineError::Eof) => {
                println!("{}", "Goodbye!".cyan());
                break;
            }
            Err(err) => {
                eprintln!("{} {:?}", "Readline error:".red(), err);
                break;
            }
        }
    }

    // Save history
    if let Some(parent) = history_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = rl.save_history(&history_path);

    Ok(())
}

/// Execute a command from the REPL
async fn execute_command(line: &str) -> CliResult<()> {
    // Parse the command line
    let args = match shellwords::split(line) {
        Ok(args) => args,
        Err(e) => return Err(CliError::InvalidInput(format!("Failed to parse command: {}", e))),
    };

    if args.is_empty() {
        return Ok(());
    }

    // Build full args with "rustpress" as argv[0]
    let mut full_args = vec!["rustpress".to_string()];
    full_args.extend(args);

    // Parse using clap
    let cli = match Cli::try_parse_from(&full_args) {
        Ok(cli) => cli,
        Err(e) => {
            // Print clap's error message (help, version, or actual error)
            println!("{}", e);
            return Ok(());
        }
    };

    // Create context
    let ctx = CliContext::new(&cli)?;

    // Check authentication for non-exempt commands
    match &cli.command {
        Commands::Artifacts { .. } | Commands::Auth(_) | Commands::Completion(_) | Commands::Config(_)
        | Commands::Interactive | Commands::Health { .. } | Commands::Info => {}
        Commands::ImportExport(ref cmd) => {
            if !matches!(cmd.command, crate::commands::import_export::ImportExportSubcommand::Analyze { .. }) {
                ctx.require_auth()?;
            }
        }
        _ => {
            ctx.require_auth()?;
        }
    }

    // Execute the command
    match cli.command {
        Commands::Artifacts { command } => {
            crate::commands::artifacts::execute(command).map_err(|e| anyhow::anyhow!(e.to_string()).into())
        }
        Commands::Auth(cmd) => crate::commands::auth::execute(&ctx, cmd).await,
        Commands::Server(cmd) => crate::commands::server::execute(&ctx, cmd).await,
        Commands::Db(cmd) => crate::commands::db::execute(&ctx, cmd).await,
        Commands::Users(cmd) => crate::commands::users::execute(&ctx, cmd).await,
        Commands::Posts(cmd) => crate::commands::posts::execute(&ctx, cmd).await,
        Commands::Pages(cmd) => crate::commands::pages::execute(&ctx, cmd).await,
        Commands::Media(cmd) => crate::commands::media::execute(&ctx, cmd).await,
        Commands::Themes(cmd) => crate::commands::themes::execute(&ctx, cmd).await,
        Commands::Plugins(cmd) => crate::commands::plugins::execute(&ctx, cmd).await,
        Commands::Cache(cmd) => crate::commands::cache::execute(&ctx, cmd).await,
        Commands::Settings(cmd) => crate::commands::settings::execute(&ctx, cmd).await,
        Commands::Backup(cmd) => crate::commands::backup::execute(&ctx, cmd).await,
        Commands::Seo(cmd) => crate::commands::seo::execute(&ctx, cmd).await,
        Commands::Config(cmd) => crate::commands::config::execute(&ctx, cmd).await,
        Commands::Completion(cmd) => crate::commands::completion::execute(cmd).await,
        Commands::ImportExport(cmd) => crate::commands::import_export::execute(&ctx, cmd).await,
        Commands::Cron(cmd) => crate::commands::cron::execute(&ctx, cmd).await,
        Commands::Interactive => {
            println!("Already in interactive mode!");
            Ok(())
        }
        Commands::Health { detailed: _ } => {
            print_system_info().await;
            Ok(())
        }
        Commands::Info => {
            print_system_info().await;
            Ok(())
        }
    }
}

/// Print REPL help
fn print_repl_help() {
    println!("{}", "RustPress Interactive Shell Commands".cyan().bold());
    println!();
    println!("{}", "REPL Commands:".yellow());
    println!("  {}       Show this help message", "help".green());
    println!("  {}       Exit the interactive shell", "exit".green());
    println!("  {}      Clear the screen", "clear".green());
    println!("  {}    Show command history", "history".green());
    println!("  {}     Show current status", "status".green());
    println!("  {}   Show system information", "info".green());
    println!();
    println!("{}", "Main Commands:".yellow());
    println!("  {}       Authentication (login, logout, whoami)", "auth".green());
    println!("  {}     Server management", "server".green());
    println!("  {}         Database operations", "db".green());
    println!("  {}      User management", "users".green());
    println!("  {}      Post management", "posts".green());
    println!("  {}      Page management", "pages".green());
    println!("  {}      Media management", "media".green());
    println!("  {}     Theme management", "themes".green());
    println!("  {}    Plugin management", "plugins".green());
    println!("  {}      Cache operations", "cache".green());
    println!("  {}   Settings management", "settings".green());
    println!("  {}     Backup/restore", "backup".green());
    println!("  {}        SEO tools", "seo".green());
    println!("  {}     Configuration", "config".green());
    println!();
    println!("{}", "Examples:".yellow());
    println!("  {} {}", "auth login".cyan(), "--email admin@example.com".dimmed());
    println!("  {}", "posts list --limit 10".cyan());
    println!("  {} {}", "users create".cyan(), "--email user@example.com --password secret123".dimmed());
    println!();
    println!("{}", "Tips:".yellow());
    println!("  - Use {} to auto-complete commands", "Tab".cyan());
    println!("  - Use {} to navigate history", "Up/Down arrows".cyan());
    println!("  - Add {} to any command for help", "--help".cyan());
}

/// Print current status
fn print_status() {
    let creds = CliCredentials::load();

    println!("{}", "Current Status".cyan().bold());
    println!();

    if let Some(email) = &creds.email {
        println!("  {} {}", "User:".green(), email);
        println!("  {} {}", "Server:".green(), creds.server_url);
        println!("  {} {}", "Authenticated:".green(), "Yes".green());
    } else {
        println!("  {} {}", "Authenticated:".yellow(), "No".red());
        println!("  {} Use 'auth login' to authenticate", "Tip:".dimmed());
    }
}

/// Print system information
async fn print_system_info() {
    println!("{}", "System Information".cyan().bold());
    println!();
    println!("  {} {}", "CLI Version:".green(), env!("CARGO_PKG_VERSION"));
    println!("  {} {}", "OS:".green(), std::env::consts::OS);
    println!("  {} {}", "Architecture:".green(), std::env::consts::ARCH);

    // Try to get server info
    let creds = CliCredentials::load();
    if creds.access_token.is_some() {
        let client = reqwest::Client::new();
        let url = format!("{}/api/v1/health", creds.server_url);

        match client.get(&url).send().await {
            Ok(response) if response.status().is_success() => {
                println!("  {} {}", "Server Status:".green(), "Online".green());
            }
            _ => {
                println!("  {} {}", "Server Status:".green(), "Unreachable".red());
            }
        }
    }
}
