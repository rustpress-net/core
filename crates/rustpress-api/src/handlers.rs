//! API request handlers.
//!
//! This module contains the handler functions that process
//! incoming HTTP requests and return responses.

pub mod auth;
pub mod posts;
pub mod pages;
pub mod users;
pub mod media;
pub mod comments;
pub mod settings;
pub mod themes;

// Re-export handlers
pub use auth::*;
pub use posts::*;
pub use pages::*;
pub use users::*;
pub use media::*;
pub use comments::*;
pub use settings::*;
pub use themes::*;
