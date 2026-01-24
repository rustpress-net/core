//! Page Builder Module

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Builder block types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockType {
    Text,
    Image,
    Button,
    Container,
    Row,
    Column,
}

/// A builder block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuilderBlock {
    pub id: Uuid,
    pub block_type: BlockType,
    pub content: serde_json::Value,
    pub children: Vec<BuilderBlock>,
}

impl BuilderBlock {
    pub fn new(block_type: BlockType) -> Self {
        Self {
            id: Uuid::new_v4(),
            block_type,
            content: serde_json::Value::Null,
            children: Vec::new(),
        }
    }
}

/// Page layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageLayout {
    pub id: Uuid,
    pub name: String,
    pub blocks: Vec<BuilderBlock>,
}

impl PageLayout {
    pub fn new(name: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            blocks: Vec::new(),
        }
    }
}
