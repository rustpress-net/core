//! Output formatting for CLI results

use colored::Colorize;
use serde::Serialize;
use std::fmt::Display;
use tabled::{
    settings::{Style, object::Columns, Modify, Width},
    Table, Tabled,
};

/// Output format options
#[derive(Debug, Clone, Default, clap::ValueEnum)]
pub enum OutputFormat {
    /// Pretty-printed table (default)
    #[default]
    Table,
    /// JSON format
    Json,
    /// YAML format
    Yaml,
    /// Plain text (minimal formatting)
    Plain,
}

impl Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Yaml => write!(f, "yaml"),
            OutputFormat::Plain => write!(f, "plain"),
        }
    }
}

/// Output formatter trait
pub trait OutputFormatter {
    /// Format data for output
    fn format<T: Serialize + Tabled>(&self, data: &[T]) -> String;
    /// Format a single item
    fn format_one<T: Serialize>(&self, data: &T) -> String;
    /// Format a success message
    fn success(&self, msg: &str) -> String;
    /// Format an error message
    fn error(&self, msg: &str) -> String;
    /// Format a warning message
    fn warning(&self, msg: &str) -> String;
    /// Format an info message
    fn info(&self, msg: &str) -> String;
}

impl OutputFormatter for OutputFormat {
    fn format<T: Serialize + Tabled>(&self, data: &[T]) -> String {
        match self {
            OutputFormat::Table => {
                if data.is_empty() {
                    return "No results found.".dimmed().to_string();
                }
                Table::new(data)
                    .with(Style::rounded())
                    .with(Modify::new(Columns::first()).with(Width::truncate(40).suffix("...")))
                    .to_string()
            }
            OutputFormat::Json => {
                serde_json::to_string_pretty(data).unwrap_or_else(|e| format!("Error: {}", e))
            }
            OutputFormat::Yaml => {
                serde_yaml::to_string(data).unwrap_or_else(|e| format!("Error: {}", e))
            }
            OutputFormat::Plain => data
                .iter()
                .map(|item| {
                    serde_json::to_value(item)
                        .map(|v| format_plain_value(&v))
                        .unwrap_or_default()
                })
                .collect::<Vec<_>>()
                .join("\n---\n"),
        }
    }

    fn format_one<T: Serialize>(&self, data: &T) -> String {
        match self {
            OutputFormat::Table | OutputFormat::Plain => {
                serde_json::to_value(data)
                    .map(|v| format_key_value(&v))
                    .unwrap_or_else(|e| format!("Error: {}", e))
            }
            OutputFormat::Json => {
                serde_json::to_string_pretty(data).unwrap_or_else(|e| format!("Error: {}", e))
            }
            OutputFormat::Yaml => {
                serde_yaml::to_string(data).unwrap_or_else(|e| format!("Error: {}", e))
            }
        }
    }

    fn success(&self, msg: &str) -> String {
        match self {
            OutputFormat::Json => {
                serde_json::json!({"status": "success", "message": msg}).to_string()
            }
            OutputFormat::Yaml => format!("status: success\nmessage: {}", msg),
            _ => format!("{} {}", "✓".green().bold(), msg.green()),
        }
    }

    fn error(&self, msg: &str) -> String {
        match self {
            OutputFormat::Json => {
                serde_json::json!({"status": "error", "message": msg}).to_string()
            }
            OutputFormat::Yaml => format!("status: error\nmessage: {}", msg),
            _ => format!("{} {}", "✗".red().bold(), msg.red()),
        }
    }

    fn warning(&self, msg: &str) -> String {
        match self {
            OutputFormat::Json => {
                serde_json::json!({"status": "warning", "message": msg}).to_string()
            }
            OutputFormat::Yaml => format!("status: warning\nmessage: {}", msg),
            _ => format!("{} {}", "⚠".yellow().bold(), msg.yellow()),
        }
    }

    fn info(&self, msg: &str) -> String {
        match self {
            OutputFormat::Json => serde_json::json!({"status": "info", "message": msg}).to_string(),
            OutputFormat::Yaml => format!("status: info\nmessage: {}", msg),
            _ => format!("{} {}", "ℹ".blue().bold(), msg),
        }
    }
}

/// Format a JSON value as key-value pairs
fn format_key_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Object(map) => map
            .iter()
            .map(|(k, v)| {
                let formatted_value = match v {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Null => "null".dimmed().to_string(),
                    _ => v.to_string(),
                };
                format!("{}: {}", k.bold(), formatted_value)
            })
            .collect::<Vec<_>>()
            .join("\n"),
        _ => value.to_string(),
    }
}

/// Format a JSON value as plain text
fn format_plain_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Object(map) => map
            .iter()
            .map(|(k, v)| {
                let formatted_value = match v {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Null => String::new(),
                    _ => v.to_string(),
                };
                format!("{}={}", k, formatted_value)
            })
            .collect::<Vec<_>>()
            .join(" "),
        _ => value.to_string(),
    }
}

/// Progress bar helper for long-running operations
pub struct ProgressBar {
    bar: indicatif::ProgressBar,
}

impl ProgressBar {
    pub fn new(len: u64, message: &str) -> Self {
        let bar = indicatif::ProgressBar::new(len);
        bar.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        bar.set_message(message.to_string());
        Self { bar }
    }

    pub fn spinner(message: &str) -> Self {
        let bar = indicatif::ProgressBar::new_spinner();
        bar.set_style(
            indicatif::ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        bar.set_message(message.to_string());
        bar.enable_steady_tick(std::time::Duration::from_millis(100));
        Self { bar }
    }

    pub fn inc(&self, delta: u64) {
        self.bar.inc(delta);
    }

    pub fn set_message(&self, msg: &str) {
        self.bar.set_message(msg.to_string());
    }

    pub fn finish(&self, msg: &str) {
        self.bar.finish_with_message(msg.to_string());
    }

    pub fn finish_and_clear(&self) {
        self.bar.finish_and_clear();
    }
}

/// Helper to print a section header
pub fn print_header(title: &str) {
    println!("\n{}", title.bold().underline());
    println!();
}

/// Helper to print a key-value line
pub fn print_kv(key: &str, value: &str) {
    println!("  {}: {}", key.dimmed(), value);
}

/// Helper to print a bullet point
pub fn print_bullet(text: &str) {
    println!("  {} {}", "•".dimmed(), text);
}
