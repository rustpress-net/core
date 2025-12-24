//! # RustPress Database
//!
//! Database layer with SQLx for type-safe database interactions.
//!
//! ## Database Design Points Implemented
//!
//! - Point 31: Posts table with UUID primary keys
//! - Point 32: Flexible meta system using EAV pattern
//! - Point 33: Terms/taxonomy system for categories and tags
//! - Point 34: Users table with Argon2 password hashing
//! - Point 35: Permissions/capabilities for fine-grained access control
//! - Point 36: Comments table with nested reply support
//! - Point 37: Media/attachments table with metadata
//! - Point 38: Options/settings table for site configuration
//! - Point 39: Revisions table for content versioning
//! - Point 40: Sessions table for server-side session management
//! - Point 41: Audit log tables for tracking changes
//! - Point 42: Soft deletes across all content tables
//! - Point 43: Indexes for common query patterns
//! - Point 44: Full-text search schema using PostgreSQL tsvector
//! - Point 45: Database partitioning strategy
//! - Point 46: Links/menu items table for navigation
//! - Point 47: Widgets/blocks table for layout components
//! - Point 48: Scheduled posts for future publishing
//! - Point 49: User_meta table for extended profile data
//! - Point 50: Relationship tables for post-to-post connections
//! - Point 51: Notifications table for user alerts
//! - Point 52: Queue jobs table for persistent background tasks
//! - Point 53: Cache invalidation tracking table
//! - Point 54: Redirects table for URL management
//! - Point 55: Multi-site tables structure for network installations

pub mod pool;
pub mod migration;
pub mod repository;
pub mod transaction;
pub mod models;
pub mod schema;

pub use pool::{DatabasePool, PoolConfig};
pub use migration::Migrator;
pub use transaction::Transaction;
pub use schema::*;
