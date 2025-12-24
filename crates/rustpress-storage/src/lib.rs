//! # RustPress Storage
//!
//! File storage abstraction supporting local and cloud storage backends.

pub mod backend;
pub mod storage;
pub mod file;

pub use backend::{StorageBackend, LocalBackend};
pub use storage::{Storage, StorageConfig};
pub use file::{StoredFile, FileMetadata};

#[cfg(feature = "s3")]
pub use backend::S3Backend;
