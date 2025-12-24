//! RustPress CLI Library
//!
//! This crate provides the CLI implementation for RustPress CMS.
//! It can be used as a library to programmatically execute CLI commands.

pub mod commands;
pub mod context;
pub mod error;
pub mod output;

pub use commands::{Cli, Commands};
pub use context::CliContext;
pub use error::{CliError, CliResult};
pub use output::{OutputFormat, OutputFormatter};
