//! Shell completion generation commands

use clap::{Args, CommandFactory};
use clap_complete::{generate, Shell};
use std::io;

use crate::error::CliResult;

use super::Cli;

#[derive(Args, Debug)]
pub struct CompletionCommand {
    /// Shell to generate completions for
    #[arg(value_enum)]
    pub shell: Shell,
}

pub async fn execute(cmd: CompletionCommand) -> CliResult<()> {
    let mut cli = Cli::command();
    let name = cli.get_name().to_string();
    generate(cmd.shell, &mut cli, name, &mut io::stdout());
    Ok(())
}

// Note: This module generates completions for the following shells:
// - Bash: rustpress completion bash > /etc/bash_completion.d/rustpress
// - Zsh: rustpress completion zsh > ~/.zsh/completions/_rustpress
// - Fish: rustpress completion fish > ~/.config/fish/completions/rustpress.fish
// - PowerShell: rustpress completion powershell > rustpress.ps1
