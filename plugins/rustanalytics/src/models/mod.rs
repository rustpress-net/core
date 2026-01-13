//! Models for RustAnalytics - Google Analytics Integration
//!
//! This module contains all data models for interacting with Google Analytics API
//! and storing analytics data locally.

#![allow(ambiguous_glob_reexports)]

pub mod settings;
pub mod analytics;
pub mod reports;
pub mod realtime;
pub mod audience;
pub mod acquisition;
pub mod behavior;
pub mod conversions;
pub mod ecommerce;
pub mod api;

// Re-export all types from submodules
pub use settings::*;
pub use analytics::*;
pub use reports::*;
pub use realtime::*;
pub use audience::*;
pub use acquisition::*;
pub use behavior::*;
pub use conversions::*;
pub use ecommerce::*;
pub use api::*;
