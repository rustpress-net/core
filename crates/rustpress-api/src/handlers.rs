//! API request handlers.
//!
//! This module contains the handler functions that process
//! incoming HTTP requests and return responses.

pub mod animations;
pub mod auth;
pub mod blocks;
pub mod comments;
pub mod media;
pub mod pages;
pub mod posts;
pub mod settings;
pub mod templates;
pub mod themes;
pub mod users;

// Re-export handlers
pub use animations::*;
pub use auth::*;
pub use blocks::*;
pub use comments::*;
pub use media::*;
pub use pages::*;
pub use posts::*;
pub use settings::*;
pub use templates::*;
pub use themes::*;
pub use users::*;
