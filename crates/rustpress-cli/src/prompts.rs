//! Interactive prompts and confirmations
//!
//! Provides user-friendly prompts for confirmations, selections, and input.

use colored::Colorize;
use dialoguer::{Confirm, Input, Password, Select};

use crate::error::{CliError, CliResult};

/// Ask for confirmation before a destructive action
pub fn confirm_action(message: &str) -> CliResult<bool> {
    let result = Confirm::new()
        .with_prompt(message)
        .default(false)
        .interact()
        .map_err(|e| CliError::InvalidInput(format!("Failed to get confirmation: {}", e)))?;

    Ok(result)
}

/// Ask for confirmation with a warning about destructive action
pub fn confirm_destructive(action: &str, target: &str) -> CliResult<bool> {
    println!();
    println!("{} This action cannot be undone!", "⚠ Warning:".yellow().bold());
    println!();

    let message = format!("Are you sure you want to {} '{}'?", action, target);
    confirm_action(&message)
}

/// Ask for confirmation with item count
pub fn confirm_bulk_action(action: &str, count: usize, item_type: &str) -> CliResult<bool> {
    println!();
    println!("{} This will {} {} {}.",
        "⚠ Warning:".yellow().bold(),
        action,
        count.to_string().cyan(),
        if count == 1 { item_type } else { &format!("{}s", item_type) }
    );
    println!();

    let message = format!("Are you sure you want to continue?");
    confirm_action(&message)
}

/// Prompt for text input
pub fn prompt_input(message: &str, default: Option<&str>) -> CliResult<String> {
    let mut input = Input::<String>::new().with_prompt(message);

    if let Some(default_val) = default {
        input = input.default(default_val.to_string());
    }

    input
        .interact_text()
        .map_err(|e| CliError::InvalidInput(format!("Failed to get input: {}", e)))
}

/// Prompt for password input (hidden)
pub fn prompt_password(message: &str) -> CliResult<String> {
    Password::new()
        .with_prompt(message)
        .interact()
        .map_err(|e| CliError::InvalidInput(format!("Failed to get password: {}", e)))
}

/// Prompt for password with confirmation
pub fn prompt_password_confirm(message: &str) -> CliResult<String> {
    Password::new()
        .with_prompt(message)
        .with_confirmation("Confirm password", "Passwords do not match")
        .interact()
        .map_err(|e| CliError::InvalidInput(format!("Failed to get password: {}", e)))
}

/// Prompt for selection from a list
pub fn prompt_select(message: &str, items: &[&str]) -> CliResult<usize> {
    Select::new()
        .with_prompt(message)
        .items(items)
        .default(0)
        .interact()
        .map_err(|e| CliError::InvalidInput(format!("Failed to get selection: {}", e)))
}

/// Show a success message
pub fn show_success(message: &str) {
    println!("{} {}", "✓".green(), message.green());
}

/// Show an error message
pub fn show_error(message: &str) {
    println!("{} {}", "✗".red(), message.red());
}

/// Show an info message
pub fn show_info(message: &str) {
    println!("{} {}", "ℹ".cyan(), message);
}

/// Show a warning message
pub fn show_warning(message: &str) {
    println!("{} {}", "⚠".yellow(), message.yellow());
}
