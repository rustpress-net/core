//! Services for RustAnalytics plugin
//!
//! This module contains all service implementations for interacting with
//! Google Analytics API and managing analytics data.

pub mod client;
pub mod analytics;
pub mod realtime;
pub mod reports;
pub mod cache;
pub mod sync;

pub use client::GoogleAnalyticsClient;
pub use analytics::AnalyticsService;
pub use realtime::RealtimeService;
pub use reports::ReportService;
pub use cache::CacheService;
pub use sync::SyncService;
