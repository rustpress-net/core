//! Post Document Structure
//!
//! The main document structure for posts in RustPress.

use crate::blocks::{Block, BlockId, BlockSerializer};
use crate::post::{
    FeaturedMedia, PostMetadata, PostPublishing, PostRevision, PostSeo, PostStats,
    PublishStatus,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique post identifier
pub type PostId = i64;

/// Complete post document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostDocument {
    /// Post ID (database ID)
    pub id: PostId,

    /// Unique UUID for cross-reference
    pub uuid: Uuid,

    /// Post type (post, page, custom)
    pub post_type: String,

    /// Post title
    pub title: String,

    /// URL slug
    pub slug: String,

    /// Post content as blocks
    pub content: PostContent,

    /// Post excerpt/summary
    pub excerpt: Option<String>,

    /// Auto-generated excerpt
    pub auto_excerpt: Option<String>,

    /// Post metadata
    pub metadata: PostMetadata,

    /// Featured media (image/video)
    pub featured_media: Option<FeaturedMedia>,

    /// SEO settings
    pub seo: PostSeo,

    /// Publishing settings
    pub publishing: PostPublishing,

    /// Content statistics
    pub stats: PostStats,

    /// Custom fields
    pub custom_fields: HashMap<String, serde_json::Value>,

    /// Template to use
    pub template: Option<String>,

    /// Parent post ID (for hierarchical types)
    pub parent_id: Option<PostId>,

    /// Menu order (for sorting)
    pub menu_order: i32,

    /// Comment status
    pub comment_status: CommentStatus,

    /// Ping status
    pub ping_status: PingStatus,

    /// Post password (for protected posts)
    pub password: Option<String>,

    /// GUID (globally unique identifier URL)
    pub guid: String,

    /// Post format
    pub format: PostFormat,

    /// Sticky post
    pub sticky: bool,

    /// Timestamps
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,

    /// Version number (increments on save)
    pub version: u32,

    /// Revision history
    pub revisions: Vec<PostRevision>,

    /// Editor state (for resuming editing)
    pub editor_state: Option<EditorState>,
}

impl PostDocument {
    /// Create a new post document
    pub fn new(post_type: impl Into<String>, title: impl Into<String>) -> Self {
        let now = Utc::now();
        let title = title.into();
        let slug = slugify(&title);

        Self {
            id: 0,
            uuid: Uuid::new_v4(),
            post_type: post_type.into(),
            title: title.clone(),
            slug: slug.clone(),
            content: PostContent::default(),
            excerpt: None,
            auto_excerpt: None,
            metadata: PostMetadata::default(),
            featured_media: None,
            seo: PostSeo::default(),
            publishing: PostPublishing::default(),
            stats: PostStats::default(),
            custom_fields: HashMap::new(),
            template: None,
            parent_id: None,
            menu_order: 0,
            comment_status: CommentStatus::Open,
            ping_status: PingStatus::Open,
            password: None,
            guid: format!("/?p={}", Uuid::new_v4()),
            format: PostFormat::Standard,
            sticky: false,
            created_at: now,
            modified_at: now,
            version: 1,
            revisions: Vec::new(),
            editor_state: None,
        }
    }

    /// Create a new post
    pub fn new_post(title: impl Into<String>) -> Self {
        Self::new("post", title)
    }

    /// Create a new page
    pub fn new_page(title: impl Into<String>) -> Self {
        let mut doc = Self::new("page", title);
        doc.comment_status = CommentStatus::Closed;
        doc.ping_status = PingStatus::Closed;
        doc
    }

    /// Add a block to the content
    pub fn add_block(&mut self, block: Block) {
        self.content.blocks.push(block);
        self.update_stats();
    }

    /// Insert a block at a specific position
    pub fn insert_block(&mut self, index: usize, block: Block) {
        self.content.blocks.insert(index, block);
        self.update_stats();
    }

    /// Remove a block by ID
    pub fn remove_block(&mut self, block_id: BlockId) -> Option<Block> {
        if let Some(index) = self.content.blocks.iter().position(|b| b.id == block_id) {
            let block = self.content.blocks.remove(index);
            self.update_stats();
            Some(block)
        } else {
            None
        }
    }

    /// Get a block by ID
    pub fn get_block(&self, block_id: BlockId) -> Option<&Block> {
        self.find_block_recursive(&self.content.blocks, block_id)
    }

    /// Get a mutable block by ID
    pub fn get_block_mut(&mut self, block_id: BlockId) -> Option<&mut Block> {
        Self::find_block_recursive_mut(&mut self.content.blocks, block_id)
    }

    fn find_block_recursive<'a>(&self, blocks: &'a [Block], id: BlockId) -> Option<&'a Block> {
        for block in blocks {
            if block.id == id {
                return Some(block);
            }
            if let Some(found) = self.find_block_recursive(&block.children, id) {
                return Some(found);
            }
        }
        None
    }

    fn find_block_recursive_mut<'a>(
        blocks: &'a mut [Block],
        id: BlockId,
    ) -> Option<&'a mut Block> {
        for block in blocks {
            if block.id == id {
                return Some(block);
            }
            if let Some(found) = Self::find_block_recursive_mut(&mut block.children, id) {
                return Some(found);
            }
        }
        None
    }

    /// Move a block to a new position
    pub fn move_block(&mut self, block_id: BlockId, new_index: usize) -> bool {
        if let Some(current_index) = self.content.blocks.iter().position(|b| b.id == block_id) {
            let block = self.content.blocks.remove(current_index);
            let insert_index = if new_index > current_index {
                new_index.saturating_sub(1)
            } else {
                new_index
            };
            self.content
                .blocks
                .insert(insert_index.min(self.content.blocks.len()), block);
            true
        } else {
            false
        }
    }

    /// Update content statistics
    pub fn update_stats(&mut self) {
        self.stats = PostStats::calculate(&self.content);
        self.auto_excerpt = Some(self.generate_excerpt(160));
    }

    /// Generate excerpt from content
    pub fn generate_excerpt(&self, max_chars: usize) -> String {
        let text = self.content.get_plain_text();
        if text.len() <= max_chars {
            text
        } else {
            let truncated = &text[..max_chars];
            if let Some(last_space) = truncated.rfind(' ') {
                format!("{}...", &truncated[..last_space])
            } else {
                format!("{}...", truncated)
            }
        }
    }

    /// Get the excerpt (manual or auto-generated)
    pub fn get_excerpt(&self) -> String {
        self.excerpt
            .clone()
            .or_else(|| self.auto_excerpt.clone())
            .unwrap_or_default()
    }

    /// Create a revision of current state
    pub fn create_revision(&mut self, author_id: i64, message: Option<String>) {
        let revision = PostRevision::new(
            self.version,
            author_id,
            self.title.clone(),
            self.content.clone(),
            self.excerpt.clone(),
            message,
        );
        self.revisions.push(revision);
        self.version += 1;
        self.modified_at = Utc::now();
    }

    /// Restore from a revision
    pub fn restore_revision(&mut self, revision_index: usize) -> bool {
        if let Some(revision) = self.revisions.get(revision_index) {
            self.title = revision.title.clone();
            self.content = revision.content.clone();
            self.excerpt = revision.excerpt.clone();
            self.version += 1;
            self.modified_at = Utc::now();
            self.update_stats();
            true
        } else {
            false
        }
    }

    /// Check if post is published
    pub fn is_published(&self) -> bool {
        self.publishing.status == PublishStatus::Published
    }

    /// Check if post is draft
    pub fn is_draft(&self) -> bool {
        self.publishing.status == PublishStatus::Draft
    }

    /// Check if post is scheduled
    pub fn is_scheduled(&self) -> bool {
        self.publishing.status == PublishStatus::Scheduled
    }

    /// Get the full permalink
    pub fn get_permalink(&self, base_url: &str) -> String {
        let post_type_slug = match self.post_type.as_str() {
            "post" => "",
            "page" => "",
            other => other,
        };

        if post_type_slug.is_empty() {
            format!("{}/{}/", base_url.trim_end_matches('/'), self.slug)
        } else {
            format!(
                "{}/{}/{}/",
                base_url.trim_end_matches('/'),
                post_type_slug,
                self.slug
            )
        }
    }

    /// Validate the document
    pub fn validate(&self) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        if self.title.trim().is_empty() {
            errors.push(ValidationError {
                field: "title".to_string(),
                message: "Title is required".to_string(),
                severity: ValidationSeverity::Error,
            });
        }

        if self.slug.trim().is_empty() {
            errors.push(ValidationError {
                field: "slug".to_string(),
                message: "Slug is required".to_string(),
                severity: ValidationSeverity::Error,
            });
        }

        if self.content.blocks.is_empty() {
            errors.push(ValidationError {
                field: "content".to_string(),
                message: "Post has no content".to_string(),
                severity: ValidationSeverity::Warning,
            });
        }

        // SEO validation
        if self.seo.meta_title.is_none() || self.seo.meta_title.as_ref().map_or(true, |t| t.is_empty()) {
            errors.push(ValidationError {
                field: "seo.meta_title".to_string(),
                message: "SEO title is recommended".to_string(),
                severity: ValidationSeverity::Info,
            });
        }

        if self.seo.meta_description.is_none() {
            errors.push(ValidationError {
                field: "seo.meta_description".to_string(),
                message: "Meta description is recommended for SEO".to_string(),
                severity: ValidationSeverity::Info,
            });
        }

        if self.featured_media.is_none() && self.post_type == "post" {
            errors.push(ValidationError {
                field: "featured_media".to_string(),
                message: "Featured image is recommended".to_string(),
                severity: ValidationSeverity::Info,
            });
        }

        errors
    }
}

/// Post content container
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PostContent {
    /// Content blocks
    pub blocks: Vec<Block>,

    /// Raw HTML (for legacy content)
    pub raw_html: Option<String>,

    /// Markdown source (if using markdown mode)
    pub markdown: Option<String>,

    /// Content format version
    pub format_version: u32,
}

impl PostContent {
    /// Get plain text from all blocks
    pub fn get_plain_text(&self) -> String {
        self.blocks
            .iter()
            .map(|b| b.get_text_content())
            .collect::<Vec<_>>()
            .join(" ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Get HTML from all blocks
    pub fn get_html(&self) -> String {
        let serializer = BlockSerializer::new();
        serializer.to_html(&self.blocks)
    }

    /// Get all block IDs
    pub fn get_all_block_ids(&self) -> Vec<BlockId> {
        fn collect_ids(blocks: &[Block], ids: &mut Vec<BlockId>) {
            for block in blocks {
                ids.push(block.id);
                collect_ids(&block.children, ids);
            }
        }

        let mut ids = Vec::new();
        collect_ids(&self.blocks, &mut ids);
        ids
    }

    /// Count total blocks (including nested)
    pub fn count_blocks(&self) -> usize {
        fn count_recursive(blocks: &[Block]) -> usize {
            blocks.iter().fold(0, |acc, b| {
                acc + 1 + count_recursive(&b.children)
            })
        }
        count_recursive(&self.blocks)
    }
}

/// Comment status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommentStatus {
    Open,
    Closed,
}

/// Ping status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PingStatus {
    Open,
    Closed,
}

/// Post format (like WordPress post formats)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PostFormat {
    Standard,
    Aside,
    Gallery,
    Link,
    Image,
    Quote,
    Status,
    Video,
    Audio,
    Chat,
}


/// Validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub severity: ValidationSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

/// Editor state for resuming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorState {
    /// Selected block IDs
    pub selected_blocks: Vec<BlockId>,

    /// Scroll position
    pub scroll_position: f64,

    /// Active panel
    pub active_panel: Option<String>,

    /// Expanded sections
    pub expanded_sections: Vec<String>,

    /// Last cursor position
    pub cursor_position: Option<CursorPosition>,

    /// Saved at timestamp
    pub saved_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub block_id: BlockId,
    pub offset: usize,
}

/// Simple slugify function
fn slugify(text: &str) -> String {
    let mut result = String::new();
    let mut last_was_dash = true; // Start true to avoid leading dashes

    for c in text.to_lowercase().chars() {
        if c.is_alphanumeric() {
            result.push(c);
            last_was_dash = false;
        } else if c.is_whitespace() || c == '-' || c == '_' {
            // Only add a dash if the last character wasn't a dash
            if !last_was_dash {
                result.push('-');
                last_was_dash = true;
            }
        }
        // Other characters are simply removed
    }

    // Remove trailing dash if present
    result.trim_end_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::BlockType;

    #[test]
    fn test_new_post() {
        let post = PostDocument::new_post("Hello World");
        assert_eq!(post.title, "Hello World");
        assert_eq!(post.slug, "hello-world");
        assert_eq!(post.post_type, "post");
        assert!(post.is_draft());
    }

    #[test]
    fn test_add_block() {
        let mut post = PostDocument::new_post("Test");
        let block = Block::new(BlockType::Paragraph);
        post.add_block(block);
        assert_eq!(post.content.blocks.len(), 1);
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("  Multiple   Spaces  "), "multiple-spaces");
        assert_eq!(slugify("Special! Characters@Here"), "special-charactershere");
    }
}
