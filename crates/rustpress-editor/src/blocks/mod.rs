//! Block System for RustPress Editor
//!
//! Implements a comprehensive block-based content editing system with:
//! - 100+ block types (paragraphs, headings, images, galleries, etc.)
//! - Drag-and-drop reordering
//! - Nested blocks (groups, columns)
//! - Block patterns and templates
//! - Real-time validation
//! - Multiple serialization formats (HTML, Markdown, JSON)

pub mod types;
pub mod registry;
pub mod validation;
pub mod transform;
pub mod serialization;

pub use types::*;
pub use registry::{BlockRegistry, BlockDefinition, BlockSupports};
pub use validation::{BlockValidator, ValidationResult, ValidationError, ValidationConfig};
pub use transform::BlockTransformer;
pub use serialization::BlockSerializer;
