//! # Custom Fields System
//!
//! Advanced custom fields with multiple field types, repeaters, and conditional logic.
//!
//! Features:
//! - Multiple field types (text, textarea, WYSIWYG, image, file, etc.)
//! - Field groups and layouts
//! - Repeater and flexible content fields
//! - Conditional display logic
//! - Field validation
//! - Field templates

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use thiserror::Error;

/// Field system errors
#[derive(Debug, Error)]
pub enum FieldError {
    #[error("Field not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Invalid field type: {0}")]
    InvalidType(String),

    #[error("Required field missing: {0}")]
    RequiredMissing(String),

    #[error("Field configuration error: {0}")]
    ConfigError(String),
}

/// Field types supported by the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldType {
    // Basic fields
    Text,
    Textarea,
    Number,
    Email,
    Url,
    Password,

    // Rich content
    Wysiwyg,
    Markdown,
    Code,

    // Selection
    Select,
    MultiSelect,
    Checkbox,
    Radio,
    ButtonGroup,
    TrueFalse,

    // Media
    Image,
    File,
    Gallery,
    Oembed,

    // Relational
    PostObject,
    PageLink,
    Relationship,
    Taxonomy,
    User,

    // jQuery UI
    DatePicker,
    DateTimePicker,
    TimePicker,
    ColorPicker,

    // Layout
    Message,
    Accordion,
    Tab,
    Group,
    Repeater,
    FlexibleContent,

    // Advanced
    Link,
    GoogleMap,
    Range,
    Clone,
}

impl FieldType {
    pub fn label(&self) -> &str {
        match self {
            Self::Text => "Text",
            Self::Textarea => "Text Area",
            Self::Number => "Number",
            Self::Email => "Email",
            Self::Url => "URL",
            Self::Password => "Password",
            Self::Wysiwyg => "WYSIWYG Editor",
            Self::Markdown => "Markdown Editor",
            Self::Code => "Code Editor",
            Self::Select => "Select",
            Self::MultiSelect => "Multi-Select",
            Self::Checkbox => "Checkbox",
            Self::Radio => "Radio Button",
            Self::ButtonGroup => "Button Group",
            Self::TrueFalse => "True / False",
            Self::Image => "Image",
            Self::File => "File",
            Self::Gallery => "Gallery",
            Self::Oembed => "oEmbed",
            Self::PostObject => "Post Object",
            Self::PageLink => "Page Link",
            Self::Relationship => "Relationship",
            Self::Taxonomy => "Taxonomy",
            Self::User => "User",
            Self::DatePicker => "Date Picker",
            Self::DateTimePicker => "Date Time Picker",
            Self::TimePicker => "Time Picker",
            Self::ColorPicker => "Color Picker",
            Self::Message => "Message",
            Self::Accordion => "Accordion",
            Self::Tab => "Tab",
            Self::Group => "Group",
            Self::Repeater => "Repeater",
            Self::FlexibleContent => "Flexible Content",
            Self::Link => "Link",
            Self::GoogleMap => "Google Map",
            Self::Range => "Range",
            Self::Clone => "Clone",
        }
    }

    pub fn is_layout(&self) -> bool {
        matches!(
            self,
            Self::Message | Self::Accordion | Self::Tab | Self::Group | Self::Repeater | Self::FlexibleContent
        )
    }

    pub fn supports_multiple(&self) -> bool {
        matches!(
            self,
            Self::MultiSelect | Self::Checkbox | Self::Gallery | Self::Relationship | Self::Repeater | Self::FlexibleContent
        )
    }
}

/// Field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    /// Unique field key
    pub key: String,

    /// Field name/slug
    pub name: String,

    /// Display label
    pub label: String,

    /// Field type
    pub field_type: FieldType,

    /// Instructions/help text
    pub instructions: String,

    /// Whether field is required
    pub required: bool,

    /// Default value
    pub default_value: Option<JsonValue>,

    /// Placeholder text
    pub placeholder: String,

    /// Prepend text/icon
    pub prepend: String,

    /// Append text/icon
    pub append: String,

    /// Maximum length (for text fields)
    pub maxlength: Option<usize>,

    /// Minimum value (for number fields)
    pub min: Option<f64>,

    /// Maximum value (for number fields)
    pub max: Option<f64>,

    /// Step value (for number fields)
    pub step: Option<f64>,

    /// Choices for select/radio/checkbox fields
    pub choices: Vec<FieldChoice>,

    /// Allow null/empty value
    pub allow_null: bool,

    /// Return format
    pub return_format: String,

    /// Preview size (for image fields)
    pub preview_size: String,

    /// Sub-fields (for group/repeater)
    pub sub_fields: Vec<Field>,

    /// Layouts (for flexible content)
    pub layouts: Vec<FlexibleLayout>,

    /// Conditional logic rules
    pub conditional_logic: Option<ConditionalLogic>,

    /// Wrapper attributes
    pub wrapper: FieldWrapper,

    /// Custom attributes
    pub attributes: HashMap<String, String>,

    /// Field order
    pub order: i32,

    /// Parent field key (for nested fields)
    pub parent: Option<String>,
}

impl Field {
    pub fn new(key: &str, name: &str, field_type: FieldType) -> Self {
        Self {
            key: key.to_string(),
            name: name.to_string(),
            label: name.to_string(),
            field_type,
            instructions: String::new(),
            required: false,
            default_value: None,
            placeholder: String::new(),
            prepend: String::new(),
            append: String::new(),
            maxlength: None,
            min: None,
            max: None,
            step: None,
            choices: Vec::new(),
            allow_null: true,
            return_format: "value".to_string(),
            preview_size: "thumbnail".to_string(),
            sub_fields: Vec::new(),
            layouts: Vec::new(),
            conditional_logic: None,
            wrapper: FieldWrapper::default(),
            attributes: HashMap::new(),
            order: 0,
            parent: None,
        }
    }

    /// Builder methods
    pub fn label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }

    pub fn instructions(mut self, instructions: &str) -> Self {
        self.instructions = instructions.to_string();
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn default(mut self, value: JsonValue) -> Self {
        self.default_value = Some(value);
        self
    }

    pub fn placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = placeholder.to_string();
        self
    }

    pub fn choices(mut self, choices: Vec<FieldChoice>) -> Self {
        self.choices = choices;
        self
    }

    pub fn sub_fields(mut self, fields: Vec<Field>) -> Self {
        self.sub_fields = fields;
        self
    }

    pub fn conditional(mut self, logic: ConditionalLogic) -> Self {
        self.conditional_logic = Some(logic);
        self
    }

    pub fn width(mut self, width: &str) -> Self {
        self.wrapper.width = width.to_string();
        self
    }

    /// Create a text field
    pub fn text(key: &str, name: &str) -> Self {
        Self::new(key, name, FieldType::Text)
    }

    /// Create a textarea field
    pub fn textarea(key: &str, name: &str) -> Self {
        Self::new(key, name, FieldType::Textarea)
    }

    /// Create a WYSIWYG field
    pub fn wysiwyg(key: &str, name: &str) -> Self {
        Self::new(key, name, FieldType::Wysiwyg)
    }

    /// Create an image field
    pub fn image(key: &str, name: &str) -> Self {
        Self::new(key, name, FieldType::Image)
    }

    /// Create a select field
    pub fn select(key: &str, name: &str, choices: Vec<FieldChoice>) -> Self {
        Self::new(key, name, FieldType::Select).choices(choices)
    }

    /// Create a repeater field
    pub fn repeater(key: &str, name: &str, sub_fields: Vec<Field>) -> Self {
        Self::new(key, name, FieldType::Repeater).sub_fields(sub_fields)
    }

    /// Create a group field
    pub fn group(key: &str, name: &str, sub_fields: Vec<Field>) -> Self {
        Self::new(key, name, FieldType::Group).sub_fields(sub_fields)
    }

    /// Create a flexible content field
    pub fn flexible(key: &str, name: &str, layouts: Vec<FlexibleLayout>) -> Self {
        let mut field = Self::new(key, name, FieldType::FlexibleContent);
        field.layouts = layouts;
        field
    }

    /// Validate a value against this field
    pub fn validate(&self, value: &JsonValue) -> Result<(), FieldError> {
        // Check required
        if self.required && (value.is_null() || value == "") {
            return Err(FieldError::RequiredMissing(self.label.clone()));
        }

        // Type-specific validation
        match self.field_type {
            FieldType::Email => {
                if let Some(s) = value.as_str() {
                    if !s.is_empty() && !s.contains('@') {
                        return Err(FieldError::ValidationError(format!("{} must be a valid email", self.label)));
                    }
                }
            }
            FieldType::Url => {
                if let Some(s) = value.as_str() {
                    if !s.is_empty() && !s.starts_with("http://") && !s.starts_with("https://") {
                        return Err(FieldError::ValidationError(format!("{} must be a valid URL", self.label)));
                    }
                }
            }
            FieldType::Number | FieldType::Range => {
                if let Some(n) = value.as_f64() {
                    if let Some(min) = self.min {
                        if n < min {
                            return Err(FieldError::ValidationError(format!("{} must be at least {}", self.label, min)));
                        }
                    }
                    if let Some(max) = self.max {
                        if n > max {
                            return Err(FieldError::ValidationError(format!("{} must be at most {}", self.label, max)));
                        }
                    }
                }
            }
            FieldType::Text | FieldType::Textarea => {
                if let Some(s) = value.as_str() {
                    if let Some(max) = self.maxlength {
                        if s.len() > max {
                            return Err(FieldError::ValidationError(format!("{} must be {} characters or less", self.label, max)));
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }
}

/// Field choice option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldChoice {
    pub value: String,
    pub label: String,
    pub disabled: bool,
}

impl FieldChoice {
    pub fn new(value: &str, label: &str) -> Self {
        Self {
            value: value.to_string(),
            label: label.to_string(),
            disabled: false,
        }
    }
}

/// Wrapper attributes for field HTML
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FieldWrapper {
    pub width: String,
    pub class: String,
    pub id: String,
}

/// Flexible content layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlexibleLayout {
    /// Layout key
    pub key: String,

    /// Layout name
    pub name: String,

    /// Display label
    pub label: String,

    /// Display type (block, row, table)
    pub display: String,

    /// Sub-fields in this layout
    pub sub_fields: Vec<Field>,

    /// Minimum instances
    pub min: Option<usize>,

    /// Maximum instances
    pub max: Option<usize>,
}

impl FlexibleLayout {
    pub fn new(key: &str, name: &str, label: &str, sub_fields: Vec<Field>) -> Self {
        Self {
            key: key.to_string(),
            name: name.to_string(),
            label: label.to_string(),
            display: "block".to_string(),
            sub_fields,
            min: None,
            max: None,
        }
    }
}

// ============================================================================
// Conditional Logic
// ============================================================================

/// Conditional logic for showing/hiding fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalLogic {
    /// Whether all rules must match (AND) or any (OR)
    pub match_type: ConditionalMatch,

    /// Groups of rules (outer = OR, inner = AND typically)
    pub groups: Vec<ConditionalGroup>,
}

impl ConditionalLogic {
    pub fn new(match_type: ConditionalMatch) -> Self {
        Self {
            match_type,
            groups: Vec::new(),
        }
    }

    pub fn and() -> Self {
        Self::new(ConditionalMatch::All)
    }

    pub fn or() -> Self {
        Self::new(ConditionalMatch::Any)
    }

    pub fn add_group(mut self, group: ConditionalGroup) -> Self {
        self.groups.push(group);
        self
    }

    pub fn add_rule(mut self, rule: ConditionalRule) -> Self {
        if self.groups.is_empty() {
            self.groups.push(ConditionalGroup::new());
        }
        if let Some(group) = self.groups.last_mut() {
            group.rules.push(rule);
        }
        self
    }

    /// Evaluate conditional logic against field values
    pub fn evaluate(&self, values: &HashMap<String, JsonValue>) -> bool {
        if self.groups.is_empty() {
            return true;
        }

        match self.match_type {
            ConditionalMatch::All => self.groups.iter().all(|g| g.evaluate(values)),
            ConditionalMatch::Any => self.groups.iter().any(|g| g.evaluate(values)),
        }
    }
}

/// How to match conditional groups
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionalMatch {
    /// All groups must match
    All,
    /// Any group must match
    Any,
}

/// A group of conditional rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalGroup {
    pub rules: Vec<ConditionalRule>,
}

impl ConditionalGroup {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add(mut self, rule: ConditionalRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Evaluate all rules in the group (AND)
    pub fn evaluate(&self, values: &HashMap<String, JsonValue>) -> bool {
        self.rules.iter().all(|rule| rule.evaluate(values))
    }
}

impl Default for ConditionalGroup {
    fn default() -> Self {
        Self::new()
    }
}

/// A single conditional rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalRule {
    /// Field key to check
    pub field: String,

    /// Comparison operator
    pub operator: ConditionalOperator,

    /// Value to compare against
    pub value: JsonValue,
}

impl ConditionalRule {
    pub fn new(field: &str, operator: ConditionalOperator, value: JsonValue) -> Self {
        Self {
            field: field.to_string(),
            operator,
            value,
        }
    }

    /// Create equals rule
    pub fn equals(field: &str, value: impl Into<JsonValue>) -> Self {
        Self::new(field, ConditionalOperator::Equals, value.into())
    }

    /// Create not equals rule
    pub fn not_equals(field: &str, value: impl Into<JsonValue>) -> Self {
        Self::new(field, ConditionalOperator::NotEquals, value.into())
    }

    /// Create empty rule
    pub fn empty(field: &str) -> Self {
        Self::new(field, ConditionalOperator::Empty, JsonValue::Null)
    }

    /// Create not empty rule
    pub fn not_empty(field: &str) -> Self {
        Self::new(field, ConditionalOperator::NotEmpty, JsonValue::Null)
    }

    /// Evaluate the rule against values
    pub fn evaluate(&self, values: &HashMap<String, JsonValue>) -> bool {
        let field_value = values.get(&self.field);

        match self.operator {
            ConditionalOperator::Equals => {
                field_value.map(|v| v == &self.value).unwrap_or(false)
            }
            ConditionalOperator::NotEquals => {
                field_value.map(|v| v != &self.value).unwrap_or(true)
            }
            ConditionalOperator::Contains => {
                field_value
                    .and_then(|v| v.as_str())
                    .and_then(|s| self.value.as_str().map(|v| s.contains(v)))
                    .unwrap_or(false)
            }
            ConditionalOperator::Empty => {
                field_value.map(|v| v.is_null() || v == "" || v == &JsonValue::Array(vec![])).unwrap_or(true)
            }
            ConditionalOperator::NotEmpty => {
                field_value.map(|v| !v.is_null() && v != "" && v != &JsonValue::Array(vec![])).unwrap_or(false)
            }
            ConditionalOperator::GreaterThan => {
                field_value
                    .and_then(|v| v.as_f64())
                    .and_then(|fv| self.value.as_f64().map(|cv| fv > cv))
                    .unwrap_or(false)
            }
            ConditionalOperator::LessThan => {
                field_value
                    .and_then(|v| v.as_f64())
                    .and_then(|fv| self.value.as_f64().map(|cv| fv < cv))
                    .unwrap_or(false)
            }
            ConditionalOperator::Pattern => {
                field_value
                    .and_then(|v| v.as_str())
                    .and_then(|s| {
                        self.value.as_str()
                            .and_then(|p| regex::Regex::new(p).ok())
                            .map(|re| re.is_match(s))
                    })
                    .unwrap_or(false)
            }
        }
    }
}

/// Conditional operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionalOperator {
    Equals,
    NotEquals,
    Contains,
    Empty,
    NotEmpty,
    GreaterThan,
    LessThan,
    Pattern,
}

// ============================================================================
// Field Group
// ============================================================================

/// A group of fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldGroup {
    /// Unique key
    pub key: String,

    /// Group title
    pub title: String,

    /// Fields in this group
    pub fields: Vec<Field>,

    /// Where to show this group
    pub location: Vec<LocationRule>,

    /// Menu order
    pub menu_order: i32,

    /// Position (normal, side, acf_after_title)
    pub position: String,

    /// Style (default, seamless)
    pub style: String,

    /// Label placement (top, left)
    pub label_placement: String,

    /// Instruction placement (label, field)
    pub instruction_placement: String,

    /// Hide on screen options
    pub hide_on_screen: Vec<String>,

    /// Active state
    pub active: bool,

    /// Description
    pub description: String,
}

impl FieldGroup {
    pub fn new(key: &str, title: &str) -> Self {
        Self {
            key: key.to_string(),
            title: title.to_string(),
            fields: Vec::new(),
            location: Vec::new(),
            menu_order: 0,
            position: "normal".to_string(),
            style: "default".to_string(),
            label_placement: "top".to_string(),
            instruction_placement: "label".to_string(),
            hide_on_screen: Vec::new(),
            active: true,
            description: String::new(),
        }
    }

    pub fn add_field(mut self, field: Field) -> Self {
        self.fields.push(field);
        self
    }

    pub fn add_location(mut self, rule: LocationRule) -> Self {
        self.location.push(rule);
        self
    }

    pub fn position(mut self, position: &str) -> Self {
        self.position = position.to_string();
        self
    }

    pub fn style(mut self, style: &str) -> Self {
        self.style = style.to_string();
        self
    }
}

/// Location rule for where to show field group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationRule {
    pub param: String,
    pub operator: String,
    pub value: String,
}

impl LocationRule {
    pub fn new(param: &str, operator: &str, value: &str) -> Self {
        Self {
            param: param.to_string(),
            operator: operator.to_string(),
            value: value.to_string(),
        }
    }

    /// Post type equals
    pub fn post_type(post_type: &str) -> Self {
        Self::new("post_type", "==", post_type)
    }

    /// Page template equals
    pub fn page_template(template: &str) -> Self {
        Self::new("page_template", "==", template)
    }

    /// Page parent equals
    pub fn page_parent(parent_id: i64) -> Self {
        Self::new("page_parent", "==", &parent_id.to_string())
    }

    /// Taxonomy term equals
    pub fn taxonomy(taxonomy: &str, term: &str) -> Self {
        Self::new(&format!("taxonomy:{}", taxonomy), "==", term)
    }
}

// ============================================================================
// Field Registry
// ============================================================================

/// Registry for field groups
pub struct FieldRegistry {
    groups: HashMap<String, FieldGroup>,
}

impl Default for FieldRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FieldRegistry {
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }

    /// Register a field group
    pub fn register(&mut self, group: FieldGroup) {
        self.groups.insert(group.key.clone(), group);
    }

    /// Get a field group
    pub fn get(&self, key: &str) -> Option<&FieldGroup> {
        self.groups.get(key)
    }

    /// Get field groups for a location
    pub fn get_for_location(&self, post_type: &str, template: Option<&str>) -> Vec<&FieldGroup> {
        self.groups.values()
            .filter(|group| {
                group.active && group.location.iter().any(|rule| {
                    match rule.param.as_str() {
                        "post_type" => {
                            let matches = rule.value == post_type;
                            if rule.operator == "==" { matches } else { !matches }
                        }
                        "page_template" => {
                            if let Some(tpl) = template {
                                let matches = rule.value == tpl;
                                if rule.operator == "==" { matches } else { !matches }
                            } else {
                                false
                            }
                        }
                        _ => false
                    }
                })
            })
            .collect()
    }

    /// Get all field groups
    pub fn get_all(&self) -> Vec<&FieldGroup> {
        self.groups.values().collect()
    }

    /// Remove a field group
    pub fn remove(&mut self, key: &str) {
        self.groups.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_creation() {
        let field = Field::text("field_title", "title")
            .label("Post Title")
            .required(true)
            .placeholder("Enter title...");

        assert_eq!(field.key, "field_title");
        assert!(field.required);
    }

    #[test]
    fn test_field_validation() {
        let field = Field::new("email", "email", FieldType::Email).required(true);

        // Valid email
        assert!(field.validate(&JsonValue::String("test@example.com".to_string())).is_ok());

        // Invalid email
        assert!(field.validate(&JsonValue::String("invalid".to_string())).is_err());

        // Empty required
        assert!(field.validate(&JsonValue::Null).is_err());
    }

    #[test]
    fn test_conditional_logic() {
        let logic = ConditionalLogic::and()
            .add_rule(ConditionalRule::equals("show_subtitle", true));

        let mut values = HashMap::new();
        values.insert("show_subtitle".to_string(), JsonValue::Bool(true));
        assert!(logic.evaluate(&values));

        values.insert("show_subtitle".to_string(), JsonValue::Bool(false));
        assert!(!logic.evaluate(&values));
    }

    #[test]
    fn test_repeater_field() {
        let repeater = Field::repeater("slides", "slides", vec![
            Field::image("slide_image", "image"),
            Field::text("slide_title", "title"),
        ]);

        assert_eq!(repeater.field_type, FieldType::Repeater);
        assert_eq!(repeater.sub_fields.len(), 2);
    }

    #[test]
    fn test_field_group_location() {
        let mut registry = FieldRegistry::new();

        let group = FieldGroup::new("group_1", "Page Fields")
            .add_location(LocationRule::post_type("page"))
            .add_field(Field::text("subtitle", "Subtitle"));

        registry.register(group);

        let groups = registry.get_for_location("page", None);
        assert_eq!(groups.len(), 1);

        let groups = registry.get_for_location("post", None);
        assert_eq!(groups.len(), 0);
    }
}
