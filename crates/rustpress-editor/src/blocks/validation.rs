//! Block Validation
//!
//! Validates block structure, content, and constraints.

use crate::blocks::{Block, BlockType};
use serde::{Deserialize, Serialize};

/// Block validator
#[derive(Debug, Clone)]
pub struct BlockValidator {
    config: ValidationConfig,
}

impl BlockValidator {
    pub fn new() -> Self {
        Self {
            config: ValidationConfig::default(),
        }
    }

    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Validate a single block
    pub fn validate(&self, block: &Block) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Validate based on block type
        self.validate_block_type(block, &mut errors, &mut warnings);

        // Validate content
        self.validate_content(block, &mut errors, &mut warnings);

        // Validate children
        self.validate_children(block, &mut errors, &mut warnings);

        // Validate styles
        self.validate_styles(block, &mut warnings);

        // Recursively validate children
        for (i, child) in block.children.iter().enumerate() {
            let child_result = self.validate(child);
            for mut error in child_result.errors {
                error.path.insert(0, format!("children[{}]", i));
                errors.push(error);
            }
            for mut warning in child_result.warnings {
                warning.path.insert(0, format!("children[{}]", i));
                warnings.push(warning);
            }
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    /// Validate multiple blocks
    pub fn validate_blocks(&self, blocks: &[Block]) -> ValidationResult {
        let mut all_errors = Vec::new();
        let mut all_warnings = Vec::new();

        for (i, block) in blocks.iter().enumerate() {
            let result = self.validate(block);
            for mut error in result.errors {
                error.path.insert(0, format!("[{}]", i));
                all_errors.push(error);
            }
            for mut warning in result.warnings {
                warning.path.insert(0, format!("[{}]", i));
                all_warnings.push(warning);
            }
        }

        ValidationResult {
            is_valid: all_errors.is_empty(),
            errors: all_errors,
            warnings: all_warnings,
        }
    }

    fn validate_block_type(&self, block: &Block, errors: &mut Vec<ValidationError>, _warnings: &mut Vec<ValidationWarning>) {
        // Check if block type is valid
        match &block.block_type {
            BlockType::Custom(id) if *id == 0 => {
                errors.push(ValidationError {
                    code: "INVALID_CUSTOM_BLOCK_ID".to_string(),
                    message: "Custom block ID cannot be 0".to_string(),
                    path: vec!["block_type".to_string()],
                    severity: ErrorSeverity::Error,
                });
            }
            _ => {}
        }
    }

    fn validate_content(&self, block: &Block, errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationWarning>) {
        let attrs = &block.attributes;

        match &block.block_type {
            // Text blocks require content
            BlockType::Paragraph | BlockType::Quote | BlockType::PullQuote | BlockType::Verse => {
                // Content can be empty for these blocks (valid empty paragraph)
            }

            // Headings should have content
            BlockType::Heading => {
                if attrs.content.as_ref().map(|c| c.trim().is_empty()).unwrap_or(true) {
                    warnings.push(ValidationWarning {
                        code: "EMPTY_HEADING".to_string(),
                        message: "Heading block has no content".to_string(),
                        path: vec!["attributes.content".to_string()],
                    });
                }

                // Validate heading level
                if let Some(level) = attrs.level {
                    if level == 0 || level > 6 {
                        errors.push(ValidationError {
                            code: "INVALID_HEADING_LEVEL".to_string(),
                            message: format!("Heading level must be 1-6, got {}", level),
                            path: vec!["attributes.level".to_string()],
                            severity: ErrorSeverity::Error,
                        });
                    }
                }
            }

            // Media blocks require URL or media_id
            BlockType::Image | BlockType::Video | BlockType::Audio | BlockType::File => {
                if attrs.url.is_none() && attrs.media_id.is_none() {
                    errors.push(ValidationError {
                        code: "MISSING_MEDIA_SOURCE".to_string(),
                        message: "Media block requires url or media_id".to_string(),
                        path: vec!["attributes".to_string()],
                        severity: ErrorSeverity::Error,
                    });
                }

                // Warn if image has no alt text
                if block.block_type == BlockType::Image {
                    if attrs.alt.as_ref().map(|a| a.is_empty()).unwrap_or(true) {
                        warnings.push(ValidationWarning {
                            code: "MISSING_ALT_TEXT".to_string(),
                            message: "Image is missing alt text for accessibility".to_string(),
                            path: vec!["attributes.alt".to_string()],
                        });
                    }
                }
            }

            // Embed blocks require URL
            BlockType::Embed => {
                if attrs.url.is_none() {
                    errors.push(ValidationError {
                        code: "MISSING_EMBED_URL".to_string(),
                        message: "Embed block requires a URL".to_string(),
                        path: vec!["attributes.url".to_string()],
                        severity: ErrorSeverity::Error,
                    });
                }
            }

            // Button requires text
            BlockType::Button => {
                let has_text = attrs.button_text.as_ref().map(|t| !t.is_empty()).unwrap_or(false)
                    || attrs.content.as_ref().map(|c| !c.is_empty()).unwrap_or(false);

                if !has_text {
                    warnings.push(ValidationWarning {
                        code: "EMPTY_BUTTON".to_string(),
                        message: "Button has no visible text".to_string(),
                        path: vec!["attributes".to_string()],
                    });
                }
            }

            // Code block language check
            BlockType::Code => {
                // Just informational, no error
            }

            // Progress bar value check
            BlockType::ProgressBar => {
                if let Some(progress) = attrs.progress {
                    if progress > 100 {
                        errors.push(ValidationError {
                            code: "INVALID_PROGRESS_VALUE".to_string(),
                            message: format!("Progress value must be 0-100, got {}", progress),
                            path: vec!["attributes.progress".to_string()],
                            severity: ErrorSeverity::Error,
                        });
                    }
                }
            }

            // Rating value check
            BlockType::Rating => {
                if let (Some(rating), Some(max)) = (attrs.rating, attrs.rating_max) {
                    if rating > max as f32 {
                        errors.push(ValidationError {
                            code: "INVALID_RATING_VALUE".to_string(),
                            message: format!("Rating {} exceeds maximum {}", rating, max),
                            path: vec!["attributes.rating".to_string()],
                            severity: ErrorSeverity::Error,
                        });
                    }
                }
            }

            _ => {}
        }

        // Check content length limits
        if let Some(content) = &attrs.content {
            if content.len() > self.config.max_content_length {
                errors.push(ValidationError {
                    code: "CONTENT_TOO_LONG".to_string(),
                    message: format!(
                        "Content exceeds maximum length of {} characters",
                        self.config.max_content_length
                    ),
                    path: vec!["attributes.content".to_string()],
                    severity: ErrorSeverity::Error,
                });
            }
        }
    }

    fn validate_children(&self, block: &Block, errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationWarning>) {
        // Check if block can have children
        let can_have_children = matches!(
            block.block_type,
            BlockType::Group
                | BlockType::Columns
                | BlockType::Column
                | BlockType::Section
                | BlockType::Row
                | BlockType::Stack
                | BlockType::Grid
                | BlockType::Accordion
                | BlockType::AccordionItem
                | BlockType::Tabs
                | BlockType::Tab
                | BlockType::List
                | BlockType::ListItem
                | BlockType::MediaText
                | BlockType::Cover
                | BlockType::Buttons
                | BlockType::Form
        );

        if !can_have_children && !block.children.is_empty() {
            errors.push(ValidationError {
                code: "UNEXPECTED_CHILDREN".to_string(),
                message: format!(
                    "{:?} blocks cannot contain children",
                    block.block_type
                ),
                path: vec!["children".to_string()],
                severity: ErrorSeverity::Error,
            });
        }

        // Validate specific parent-child relationships
        match &block.block_type {
            BlockType::Columns => {
                for child in &block.children {
                    if child.block_type != BlockType::Column {
                        errors.push(ValidationError {
                            code: "INVALID_COLUMN_CHILD".to_string(),
                            message: "Columns block can only contain Column blocks".to_string(),
                            path: vec!["children".to_string()],
                            severity: ErrorSeverity::Error,
                        });
                        break;
                    }
                }
            }

            BlockType::Tabs => {
                for child in &block.children {
                    if child.block_type != BlockType::Tab {
                        errors.push(ValidationError {
                            code: "INVALID_TAB_CHILD".to_string(),
                            message: "Tabs block can only contain Tab blocks".to_string(),
                            path: vec!["children".to_string()],
                            severity: ErrorSeverity::Error,
                        });
                        break;
                    }
                }
            }

            BlockType::Accordion => {
                for child in &block.children {
                    if child.block_type != BlockType::AccordionItem {
                        errors.push(ValidationError {
                            code: "INVALID_ACCORDION_CHILD".to_string(),
                            message: "Accordion block can only contain AccordionItem blocks".to_string(),
                            path: vec!["children".to_string()],
                            severity: ErrorSeverity::Error,
                        });
                        break;
                    }
                }
            }

            BlockType::List => {
                for child in &block.children {
                    if child.block_type != BlockType::ListItem {
                        warnings.push(ValidationWarning {
                            code: "NON_LIST_ITEM_IN_LIST".to_string(),
                            message: "List block should only contain ListItem blocks".to_string(),
                            path: vec!["children".to_string()],
                        });
                        break;
                    }
                }
            }

            BlockType::Buttons => {
                for child in &block.children {
                    if child.block_type != BlockType::Button {
                        warnings.push(ValidationWarning {
                            code: "NON_BUTTON_IN_BUTTONS".to_string(),
                            message: "Buttons block should only contain Button blocks".to_string(),
                            path: vec!["children".to_string()],
                        });
                        break;
                    }
                }
            }

            _ => {}
        }

        // Check nesting depth
        let depth = self.calculate_depth(block);
        if depth > self.config.max_nesting_depth {
            warnings.push(ValidationWarning {
                code: "DEEP_NESTING".to_string(),
                message: format!(
                    "Block nesting depth ({}) exceeds recommended maximum ({})",
                    depth, self.config.max_nesting_depth
                ),
                path: vec![],
            });
        }
    }

    fn validate_styles(&self, block: &Block, warnings: &mut Vec<ValidationWarning>) {
        let styles = &block.styles;

        // Check opacity range
        if let Some(opacity) = styles.opacity {
            if !(0.0..=1.0).contains(&opacity) {
                warnings.push(ValidationWarning {
                    code: "INVALID_OPACITY".to_string(),
                    message: format!("Opacity should be between 0 and 1, got {}", opacity),
                    path: vec!["styles.opacity".to_string()],
                });
            }
        }
    }

    fn calculate_depth(&self, block: &Block) -> usize {
        if block.children.is_empty() {
            1
        } else {
            1 + block.children.iter().map(|c| self.calculate_depth(c)).max().unwrap_or(0)
        }
    }
}

impl Default for BlockValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation configuration
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Maximum content length in characters
    pub max_content_length: usize,
    /// Maximum nesting depth
    pub max_nesting_depth: usize,
    /// Strict mode (treat warnings as errors)
    pub strict: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_content_length: 100_000,
            max_nesting_depth: 10,
            strict: false,
        }
    }
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the content is valid
    pub is_valid: bool,
    /// Validation errors
    pub errors: Vec<ValidationError>,
    /// Validation warnings
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    /// Create a valid result with no issues
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Get all messages (errors and warnings)
    pub fn all_messages(&self) -> Vec<String> {
        let mut messages = Vec::new();
        for error in &self.errors {
            messages.push(format!("[ERROR] {}: {}", error.code, error.message));
        }
        for warning in &self.warnings {
            messages.push(format!("[WARN] {}: {}", warning.code, warning.message));
        }
        messages
    }
}

/// Validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error code
    pub code: String,
    /// Human-readable message
    pub message: String,
    /// Path to the error location
    pub path: Vec<String>,
    /// Error severity
    pub severity: ErrorSeverity,
}

/// Error severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorSeverity {
    Error,
    Warning,
}

/// Validation warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Warning code
    pub code: String,
    /// Human-readable message
    pub message: String,
    /// Path to the warning location
    pub path: Vec<String>,
}
