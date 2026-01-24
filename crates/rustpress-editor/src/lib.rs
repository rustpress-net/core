//! RustPress Editor
//!
//! A comprehensive block-based content editor for RustPress CMS.
//!
//! ## Features
//!
//! - **Block System**: 100+ block types with drag-and-drop editing
//! - **Real-time Preview**: Live WYSIWYG editing with split-screen preview
//! - **Media Management**: Featured images, galleries, video embedding
//! - **SEO Optimization**: Meta tags, Open Graph, Schema.org structured data
//! - **Publishing Workflow**: Draft, review, schedule, publish
//! - **Version History**: Full revision tracking with diff comparison
//! - **Collaboration**: Real-time co-editing (optional)
//! - **Accessibility**: WCAG-compliant editing experience
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use rustpress_editor::prelude::*;
//!
//! // Create a new post
//! let mut post = PostDocument::new_post("My First Post");
//!
//! // Add blocks
//! let mut heading = Block::new(BlockType::Heading);
//! heading.attributes.content = Some("Introduction".to_string());
//! heading.attributes.level = Some(2);
//! post.add_block(heading);
//!
//! let mut paragraph = Block::new(BlockType::Paragraph);
//! paragraph.attributes.content = Some("This is my content.".to_string());
//! post.add_block(paragraph);
//!
//! // Update stats
//! post.update_stats();
//!
//! // Publish
//! post.publishing.publish();
//! ```

pub mod blocks;
pub mod post;
pub mod collaboration;
pub mod analysis;
pub mod api;

/// Prelude for common imports
pub mod prelude {
    pub use crate::blocks::{
        Block, BlockAttributes, BlockCategory, BlockId, BlockStyles, BlockType,
        BlockVisibility, TextAlign,
    };
    pub use crate::post::{
        Author, CommentStatus, FeaturedMedia, PostContent, PostDocument, PostFormat,
        PostId, PostMetadata, PostPublishing, PostRevision, PostSeo, PostStats,
        PublishStatus, Term,
    };
}

// Re-exports
pub use blocks::Block;
pub use post::PostDocument;
