//! Block Transformations
//!
//! Transform blocks between different types while preserving content.

use crate::blocks::{Block, BlockType, GalleryImage, ListType};
use uuid::Uuid;

/// Block transformer
#[derive(Debug, Clone, Default)]
pub struct BlockTransformer;

impl BlockTransformer {
    /// Create a new block transformer
    pub fn new() -> Self {
        Self
    }

    /// Transform a block to a different type
    pub fn transform(&self, block: &Block, target_type: BlockType) -> Option<Block> {
        // Check if transformation is valid
        if !self.can_transform(&block.block_type, &target_type) {
            return None;
        }

        let mut new_block = Block::new(target_type);
        new_block.id = Uuid::new_v4();

        // Transfer common attributes
        self.transfer_attributes(block, &mut new_block);

        // Apply type-specific transformations
        self.apply_type_transformation(block, &mut new_block);

        Some(new_block)
    }

    /// Check if a transformation is valid
    pub fn can_transform(&self, from: &BlockType, to: &BlockType) -> bool {
        let valid_transforms = self.get_valid_transforms(from);
        valid_transforms.contains(to)
    }

    /// Get valid transformation targets for a block type
    pub fn get_valid_transforms(&self, block_type: &BlockType) -> Vec<BlockType> {
        match block_type {
            BlockType::Paragraph => vec![
                BlockType::Heading,
                BlockType::List,
                BlockType::Quote,
                BlockType::Code,
                BlockType::Preformatted,
                BlockType::PullQuote,
            ],
            BlockType::Heading => vec![
                BlockType::Paragraph,
                BlockType::Quote,
            ],
            BlockType::List => vec![
                BlockType::Paragraph,
                BlockType::Quote,
            ],
            BlockType::Quote => vec![
                BlockType::Paragraph,
                BlockType::Heading,
                BlockType::PullQuote,
            ],
            BlockType::Code => vec![
                BlockType::Paragraph,
                BlockType::Preformatted,
            ],
            BlockType::Preformatted => vec![
                BlockType::Paragraph,
                BlockType::Code,
            ],
            BlockType::PullQuote => vec![
                BlockType::Quote,
                BlockType::Paragraph,
            ],
            BlockType::Image => vec![
                BlockType::Cover,
                BlockType::MediaText,
                BlockType::Gallery,
            ],
            BlockType::Video => vec![
                BlockType::Cover,
                BlockType::Embed,
            ],
            BlockType::Cover => vec![
                BlockType::Image,
                BlockType::Video,
                BlockType::Group,
            ],
            BlockType::MediaText => vec![
                BlockType::Image,
                BlockType::Group,
            ],
            BlockType::Group => vec![
                BlockType::Columns,
                BlockType::Cover,
                BlockType::Section,
            ],
            BlockType::Columns => vec![
                BlockType::Group,
            ],
            BlockType::Spacer => vec![
                BlockType::Separator,
            ],
            BlockType::Separator => vec![
                BlockType::Spacer,
            ],
            _ => vec![],
        }
    }

    /// Transfer common attributes between blocks
    fn transfer_attributes(&self, source: &Block, target: &mut Block) {
        // Transfer content if both blocks support it
        if source.attributes.content.is_some() {
            target.attributes.content = source.attributes.content.clone();
        }

        // Transfer common styles
        target.styles = source.styles.clone();

        // Transfer CSS classes
        target.css_classes = source.css_classes.clone();

        // Transfer anchor from meta
        target.meta.anchor = source.meta.anchor.clone();
    }

    /// Apply type-specific transformation logic
    fn apply_type_transformation(&self, source: &Block, target: &mut Block) {
        match (&source.block_type, &target.block_type) {
            // Paragraph to Heading
            (BlockType::Paragraph, BlockType::Heading) => {
                target.attributes.level = Some(2); // Default to H2
            }

            // Heading to Paragraph
            (BlockType::Heading, BlockType::Paragraph) => {
                target.attributes.level = None;
            }

            // Paragraph to List
            (BlockType::Paragraph, BlockType::List) => {
                // Split content into list items by newlines
                if let Some(content) = &source.attributes.content {
                    target.attributes.list_type = Some(ListType::Unordered);
                    // Create child ListItem blocks
                    target.children = content
                        .lines()
                        .map(|line| {
                            let mut item = Block::new(BlockType::ListItem);
                            item.attributes.content = Some(line.to_string());
                            item
                        })
                        .collect();
                }
            }

            // List to Paragraph
            (BlockType::List, BlockType::Paragraph) => {
                // Combine all list item content
                let content: String = source.children
                    .iter()
                    .filter_map(|child| child.attributes.content.clone())
                    .collect::<Vec<_>>()
                    .join("\n");
                target.attributes.content = Some(content);
            }

            // Paragraph/Heading to Quote
            (BlockType::Paragraph | BlockType::Heading, BlockType::Quote) => {
                // Content transfers automatically
            }

            // Quote to PullQuote
            (BlockType::Quote, BlockType::PullQuote) => {
                target.attributes.citation = source.attributes.citation.clone();
            }

            // PullQuote to Quote
            (BlockType::PullQuote, BlockType::Quote) => {
                target.attributes.citation = source.attributes.citation.clone();
            }

            // Image to Cover
            (BlockType::Image, BlockType::Cover) => {
                target.attributes.url = source.attributes.url.clone();
                target.attributes.media_id = source.attributes.media_id;
                target.attributes.alt = source.attributes.alt.clone();
            }

            // Cover to Image
            (BlockType::Cover, BlockType::Image) => {
                target.attributes.url = source.attributes.url.clone();
                target.attributes.media_id = source.attributes.media_id;
                target.attributes.alt = source.attributes.alt.clone();
            }

            // Image to Gallery
            (BlockType::Image, BlockType::Gallery) => {
                if let Some(url) = &source.attributes.url {
                    target.attributes.images = vec![GalleryImage {
                        id: source.attributes.media_id.unwrap_or(0),
                        url: url.clone(),
                        alt: source.attributes.alt.clone(),
                        caption: source.attributes.caption.clone(),
                        link: None,
                    }];
                }
                target.attributes.columns = Some(3);
            }

            // Image to MediaText
            (BlockType::Image, BlockType::MediaText) => {
                target.attributes.url = source.attributes.url.clone();
                target.attributes.media_id = source.attributes.media_id;
                target.attributes.alt = source.attributes.alt.clone();
            }

            // Video to Cover
            (BlockType::Video, BlockType::Cover) => {
                target.attributes.url = source.attributes.url.clone();
            }

            // Code to Preformatted
            (BlockType::Code, BlockType::Preformatted) => {
                // Content transfers automatically
                // Remove language-specific settings
            }

            // Preformatted to Code
            (BlockType::Preformatted, BlockType::Code) => {
                // Content transfers automatically
                target.attributes.language = None;
            }

            // Group to Columns
            (BlockType::Group, BlockType::Columns) => {
                // Wrap existing children in a single column
                if !source.children.is_empty() {
                    let mut column = Block::new(BlockType::Column);
                    column.children = source.children.clone();
                    target.children = vec![column];
                }
            }

            // Columns to Group
            (BlockType::Columns, BlockType::Group) => {
                // Flatten columns into group
                let mut all_children = Vec::new();
                for column in &source.children {
                    all_children.extend(column.children.clone());
                }
                target.children = all_children;
            }

            // Spacer to Separator
            (BlockType::Spacer, BlockType::Separator) => {
                // No content to transfer
            }

            // Separator to Spacer
            (BlockType::Separator, BlockType::Spacer) => {
                target.attributes.spacer_height = Some("100px".to_string());
            }

            _ => {
                // No specific transformation needed
            }
        }
    }
}

/// Transform multiple paragraphs into a single list
pub fn paragraphs_to_list(paragraphs: &[Block], ordered: bool) -> Block {
    let mut list = Block::new(BlockType::List);
    list.attributes.list_type = Some(if ordered { ListType::Ordered } else { ListType::Unordered });

    // Create child ListItem blocks
    list.children = paragraphs
        .iter()
        .filter_map(|p| p.attributes.content.clone())
        .map(|content| {
            let mut item = Block::new(BlockType::ListItem);
            item.attributes.content = Some(content);
            item
        })
        .collect();

    list
}

/// Transform a list into multiple paragraphs
pub fn list_to_paragraphs(list: &Block) -> Vec<Block> {
    list.children
        .iter()
        .map(|item| {
            let mut para = Block::new(BlockType::Paragraph);
            para.attributes.content = item.attributes.content.clone();
            para
        })
        .collect()
}

/// Merge multiple blocks of the same type
pub fn merge_blocks(blocks: &[Block]) -> Option<Block> {
    if blocks.is_empty() {
        return None;
    }

    let first = &blocks[0];
    let block_type = &first.block_type;

    // Only merge compatible types
    if !blocks.iter().all(|b| &b.block_type == block_type) {
        return None;
    }

    match block_type {
        BlockType::Paragraph => {
            let mut merged = Block::new(BlockType::Paragraph);
            let contents: Vec<String> = blocks
                .iter()
                .filter_map(|b| b.attributes.content.clone())
                .collect();
            merged.attributes.content = Some(contents.join("\n\n"));
            Some(merged)
        }
        BlockType::List => {
            let mut merged = Block::new(BlockType::List);
            merged.attributes.list_type = first.attributes.list_type;
            // Combine all children
            merged.children = blocks
                .iter()
                .flat_map(|b| b.children.clone())
                .collect();
            Some(merged)
        }
        _ => None,
    }
}

/// Split a block into multiple blocks
pub fn split_block(block: &Block, at_offset: usize) -> Option<(Block, Block)> {
    match &block.block_type {
        BlockType::Paragraph | BlockType::Heading | BlockType::Quote => {
            if let Some(content) = &block.attributes.content {
                if at_offset < content.len() {
                    let (before, after) = content.split_at(at_offset);

                    let mut first = block.clone();
                    first.id = Uuid::new_v4();
                    first.attributes.content = Some(before.to_string());

                    let mut second = Block::new(block.block_type);
                    second.attributes.content = Some(after.to_string());
                    second.styles = block.styles.clone();

                    return Some((first, second));
                }
            }
        }
        _ => {}
    }
    None
}
