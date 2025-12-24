//! # RustPress Server
//!
//! HTTP server implementation using Axum framework.

pub mod app;
pub mod error;
pub mod extract;
pub mod middleware;
pub mod response;
pub mod state;
pub mod routes;
pub mod metrics;
pub mod shutdown;
pub mod services;
pub mod background;
pub mod security;

pub use app::App;
pub use state::AppState;
pub use services::{EmailService, EmailConfig, EmailTemplate};
pub use background::init_background_tasks;
