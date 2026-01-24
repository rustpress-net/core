//! Block Registry
//!
//! Central registry for block type definitions and metadata.

use crate::blocks::{BlockCategory, BlockType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Block registry containing all available block types
#[derive(Debug, Clone)]
pub struct BlockRegistry {
    blocks: HashMap<BlockType, BlockDefinition>,
}

impl BlockRegistry {
    /// Create a new block registry with all default blocks
    pub fn new() -> Self {
        let mut registry = Self {
            blocks: HashMap::new(),
        };
        registry.register_default_blocks();
        registry
    }

    /// Register a block definition
    pub fn register(&mut self, definition: BlockDefinition) {
        self.blocks.insert(definition.block_type, definition);
    }

    /// Get a block definition by type
    pub fn get(&self, block_type: BlockType) -> Option<&BlockDefinition> {
        self.blocks.get(&block_type)
    }

    /// Get all blocks in a category
    pub fn get_by_category(&self, category: BlockCategory) -> Vec<&BlockDefinition> {
        self.blocks
            .values()
            .filter(|b| b.category == category)
            .collect()
    }

    /// Get all block types
    pub fn all(&self) -> Vec<&BlockDefinition> {
        self.blocks.values().collect()
    }

    /// Search blocks by name or keywords
    pub fn search(&self, query: &str) -> Vec<&BlockDefinition> {
        let query_lower = query.to_lowercase();
        self.blocks
            .values()
            .filter(|b| {
                b.name.to_lowercase().contains(&query_lower)
                    || b.description.to_lowercase().contains(&query_lower)
                    || b.keywords.iter().any(|k| k.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    /// Check if a block type is registered
    pub fn is_registered(&self, block_type: BlockType) -> bool {
        self.blocks.contains_key(&block_type)
    }

    /// Get transforms available for a block type
    pub fn get_transforms(&self, block_type: BlockType) -> Vec<BlockType> {
        self.blocks
            .get(&block_type)
            .map(|b| b.transforms_to.clone())
            .unwrap_or_default()
    }

    fn register_default_blocks(&mut self) {
        // Text blocks
        self.register(BlockDefinition {
            block_type: BlockType::Paragraph,
            name: "Paragraph".to_string(),
            description: "Start with the building block of all narrative.".to_string(),
            category: BlockCategory::Text,
            icon: "paragraph".to_string(),
            keywords: vec!["text".to_string(), "paragraph".to_string()],
            supports: BlockSupports {
                align: true,
                anchor: true,
                color: true,
                typography: true,
                spacing: true,
                ..Default::default()
            },
            transforms_to: vec![BlockType::Heading, BlockType::List, BlockType::Quote],
            parent: None,
            example: None,
        });

        self.register(BlockDefinition {
            block_type: BlockType::Heading,
            name: "Heading".to_string(),
            description: "Introduce new sections and organize content.".to_string(),
            category: BlockCategory::Text,
            icon: "heading".to_string(),
            keywords: vec!["title".to_string(), "subtitle".to_string(), "heading".to_string()],
            supports: BlockSupports {
                align: true,
                anchor: true,
                color: true,
                typography: true,
                level: true,
                ..Default::default()
            },
            transforms_to: vec![BlockType::Paragraph, BlockType::Quote],
            parent: None,
            example: None,
        });

        self.register(BlockDefinition {
            block_type: BlockType::List,
            name: "List".to_string(),
            description: "Create bulleted or numbered lists.".to_string(),
            category: BlockCategory::Text,
            icon: "list".to_string(),
            keywords: vec!["bullet".to_string(), "numbered".to_string(), "ordered".to_string()],
            supports: BlockSupports {
                anchor: true,
                color: true,
                typography: true,
                spacing: true,
                ..Default::default()
            },
            transforms_to: vec![BlockType::Paragraph, BlockType::Quote],
            parent: None,
            example: None,
        });

        self.register(BlockDefinition {
            block_type: BlockType::Quote,
            name: "Quote".to_string(),
            description: "Give quoted text visual emphasis.".to_string(),
            category: BlockCategory::Text,
            icon: "quote".to_string(),
            keywords: vec!["blockquote".to_string(), "citation".to_string()],
            supports: BlockSupports {
                align: true,
                anchor: true,
                color: true,
                typography: true,
                ..Default::default()
            },
            transforms_to: vec![BlockType::Paragraph, BlockType::PullQuote],
            parent: None,
            example: None,
        });

        self.register(BlockDefinition {
            block_type: BlockType::Code,
            name: "Code".to_string(),
            description: "Display code snippets with syntax highlighting.".to_string(),
            category: BlockCategory::Text,
            icon: "code".to_string(),
            keywords: vec!["source".to_string(), "snippet".to_string(), "programming".to_string()],
            supports: BlockSupports {
                anchor: true,
                typography: true,
                ..Default::default()
            },
            transforms_to: vec![BlockType::Paragraph, BlockType::Preformatted],
            parent: None,
            example: None,
        });

        // Media blocks
        self.register(BlockDefinition {
            block_type: BlockType::Image,
            name: "Image".to_string(),
            description: "Insert an image to make a visual statement.".to_string(),
            category: BlockCategory::Media,
            icon: "image".to_string(),
            keywords: vec!["photo".to_string(), "picture".to_string(), "img".to_string()],
            supports: BlockSupports {
                align: true,
                anchor: true,
                filter: true,
                ..Default::default()
            },
            transforms_to: vec![BlockType::Cover, BlockType::MediaText],
            parent: None,
            example: None,
        });

        self.register(BlockDefinition {
            block_type: BlockType::Gallery,
            name: "Gallery".to_string(),
            description: "Display multiple images in a rich gallery.".to_string(),
            category: BlockCategory::Media,
            icon: "gallery".to_string(),
            keywords: vec!["images".to_string(), "photos".to_string()],
            supports: BlockSupports {
                align: true,
                anchor: true,
                ..Default::default()
            },
            transforms_to: vec![],
            parent: None,
            example: None,
        });

        self.register(BlockDefinition {
            block_type: BlockType::Video,
            name: "Video".to_string(),
            description: "Embed a video from your media library or URL.".to_string(),
            category: BlockCategory::Media,
            icon: "video".to_string(),
            keywords: vec!["movie".to_string(), "film".to_string()],
            supports: BlockSupports {
                align: true,
                anchor: true,
                ..Default::default()
            },
            transforms_to: vec![BlockType::Cover],
            parent: None,
            example: None,
        });

        self.register(BlockDefinition {
            block_type: BlockType::Audio,
            name: "Audio".to_string(),
            description: "Embed a simple audio player.".to_string(),
            category: BlockCategory::Media,
            icon: "audio".to_string(),
            keywords: vec!["music".to_string(), "sound".to_string(), "podcast".to_string()],
            supports: BlockSupports {
                align: true,
                anchor: true,
                ..Default::default()
            },
            transforms_to: vec![],
            parent: None,
            example: None,
        });

        self.register(BlockDefinition {
            block_type: BlockType::Cover,
            name: "Cover".to_string(),
            description: "Add an image or video with a text overlay.".to_string(),
            category: BlockCategory::Media,
            icon: "cover".to_string(),
            keywords: vec!["banner".to_string(), "header".to_string(), "hero".to_string()],
            supports: BlockSupports {
                align: true,
                anchor: true,
                color: true,
                full_height: true,
                spacing: true,
                ..Default::default()
            },
            transforms_to: vec![BlockType::Image, BlockType::Video],
            parent: None,
            example: None,
        });

        self.register(BlockDefinition {
            block_type: BlockType::Embed,
            name: "Embed".to_string(),
            description: "Embed videos, images, tweets, audio, and other content.".to_string(),
            category: BlockCategory::Media,
            icon: "embed".to_string(),
            keywords: vec!["youtube".to_string(), "twitter".to_string(), "instagram".to_string()],
            supports: BlockSupports {
                align: true,
                ..Default::default()
            },
            transforms_to: vec![],
            parent: None,
            example: None,
        });

        // Layout blocks
        self.register(BlockDefinition {
            block_type: BlockType::Group,
            name: "Group".to_string(),
            description: "Gather blocks in a layout container.".to_string(),
            category: BlockCategory::Layout,
            icon: "group".to_string(),
            keywords: vec!["container".to_string(), "wrapper".to_string(), "section".to_string()],
            supports: BlockSupports {
                align: true,
                anchor: true,
                color: true,
                spacing: true,
                ..Default::default()
            },
            transforms_to: vec![BlockType::Columns],
            parent: None,
            example: None,
        });

        self.register(BlockDefinition {
            block_type: BlockType::Columns,
            name: "Columns".to_string(),
            description: "Display content in multiple columns.".to_string(),
            category: BlockCategory::Layout,
            icon: "columns".to_string(),
            keywords: vec!["layout".to_string(), "grid".to_string()],
            supports: BlockSupports {
                align: true,
                anchor: true,
                color: true,
                spacing: true,
                ..Default::default()
            },
            transforms_to: vec![BlockType::Group],
            parent: None,
            example: None,
        });

        self.register(BlockDefinition {
            block_type: BlockType::Column,
            name: "Column".to_string(),
            description: "A single column within a columns block.".to_string(),
            category: BlockCategory::Layout,
            icon: "column".to_string(),
            keywords: vec![],
            supports: BlockSupports {
                anchor: true,
                color: true,
                spacing: true,
                ..Default::default()
            },
            transforms_to: vec![],
            parent: Some(BlockType::Columns),
            example: None,
        });

        self.register(BlockDefinition {
            block_type: BlockType::Spacer,
            name: "Spacer".to_string(),
            description: "Add white space between blocks.".to_string(),
            category: BlockCategory::Layout,
            icon: "spacer".to_string(),
            keywords: vec!["whitespace".to_string(), "gap".to_string()],
            supports: BlockSupports::default(),
            transforms_to: vec![BlockType::Separator],
            parent: None,
            example: None,
        });

        self.register(BlockDefinition {
            block_type: BlockType::Separator,
            name: "Separator".to_string(),
            description: "Create a break between ideas or sections.".to_string(),
            category: BlockCategory::Layout,
            icon: "separator".to_string(),
            keywords: vec!["horizontal".to_string(), "line".to_string(), "divider".to_string()],
            supports: BlockSupports {
                align: true,
                color: true,
                ..Default::default()
            },
            transforms_to: vec![BlockType::Spacer],
            parent: None,
            example: None,
        });

        // Design blocks
        self.register(BlockDefinition {
            block_type: BlockType::Button,
            name: "Button".to_string(),
            description: "Prompt visitors to take action with a button.".to_string(),
            category: BlockCategory::Design,
            icon: "button".to_string(),
            keywords: vec!["link".to_string(), "cta".to_string()],
            supports: BlockSupports {
                align: true,
                anchor: true,
                color: true,
                typography: true,
                ..Default::default()
            },
            transforms_to: vec![],
            parent: None,
            example: None,
        });

        self.register(BlockDefinition {
            block_type: BlockType::Buttons,
            name: "Buttons".to_string(),
            description: "Prompt visitors to take action with a group of buttons.".to_string(),
            category: BlockCategory::Design,
            icon: "buttons".to_string(),
            keywords: vec!["links".to_string(), "ctas".to_string()],
            supports: BlockSupports {
                align: true,
                anchor: true,
                spacing: true,
                ..Default::default()
            },
            transforms_to: vec![],
            parent: None,
            example: None,
        });

        // Widget blocks
        self.register(BlockDefinition {
            block_type: BlockType::Table,
            name: "Table".to_string(),
            description: "Create structured content in rows and columns.".to_string(),
            category: BlockCategory::Widgets,
            icon: "table".to_string(),
            keywords: vec!["data".to_string(), "grid".to_string()],
            supports: BlockSupports {
                anchor: true,
                color: true,
                typography: true,
                ..Default::default()
            },
            transforms_to: vec![],
            parent: None,
            example: None,
        });

        self.register(BlockDefinition {
            block_type: BlockType::TableOfContents,
            name: "Table of Contents".to_string(),
            description: "Summarize your post with a list of headings.".to_string(),
            category: BlockCategory::Widgets,
            icon: "toc".to_string(),
            keywords: vec!["navigation".to_string(), "summary".to_string()],
            supports: BlockSupports {
                color: true,
                typography: true,
                ..Default::default()
            },
            transforms_to: vec![],
            parent: None,
            example: None,
        });

        // Add more blocks as needed...
    }
}

impl Default for BlockRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Block definition metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDefinition {
    /// Block type identifier
    pub block_type: BlockType,

    /// Human-readable name
    pub name: String,

    /// Description for the block inserter
    pub description: String,

    /// Block category
    pub category: BlockCategory,

    /// Icon identifier
    pub icon: String,

    /// Search keywords
    pub keywords: Vec<String>,

    /// Supported features
    pub supports: BlockSupports,

    /// Block types this block can transform to
    pub transforms_to: Vec<BlockType>,

    /// Required parent block type (for nested blocks)
    pub parent: Option<BlockType>,

    /// Example block configuration
    pub example: Option<serde_json::Value>,
}

/// Block support flags
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlockSupports {
    /// Supports alignment (left, center, right, wide, full)
    pub align: bool,

    /// Supports HTML anchor
    pub anchor: bool,

    /// Supports custom class name
    pub custom_class_name: bool,

    /// Supports color settings
    pub color: bool,

    /// Supports typography settings
    pub typography: bool,

    /// Supports spacing settings
    pub spacing: bool,

    /// Supports heading level (for Heading block)
    pub level: bool,

    /// Supports full height (for Cover block)
    pub full_height: bool,

    /// Supports duotone filter (for Image block)
    pub filter: bool,

    /// Can be converted to reusable block
    pub reusable: bool,

    /// Supports lock settings
    pub lock: bool,

    /// Supports position settings (sticky, fixed)
    pub position: bool,
}
