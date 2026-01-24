//! Block Type Definitions
//!
//! Defines all available block types for the RustPress editor.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique block identifier
pub type BlockId = Uuid;

/// Block category for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockCategory {
    /// Text content blocks
    Text,
    /// Media blocks (images, videos, audio)
    Media,
    /// Layout blocks (columns, groups, sections)
    Layout,
    /// Design blocks (buttons, dividers, spacers)
    Design,
    /// Widget blocks (shortcodes, embeds)
    Widgets,
    /// Interactive blocks (forms, accordions)
    Interactive,
    /// Theme-specific blocks
    Theme,
    /// Reusable/pattern blocks
    Reusable,
}

/// Core block structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Unique block ID
    pub id: BlockId,
    /// Block type identifier
    #[serde(rename = "type")]
    pub block_type: BlockType,
    /// Block content/attributes
    pub attributes: BlockAttributes,
    /// Nested child blocks (for container blocks)
    #[serde(default)]
    pub children: Vec<Block>,
    /// Block-level styles
    #[serde(default)]
    pub styles: BlockStyles,
    /// Custom CSS classes
    #[serde(default)]
    pub css_classes: Vec<String>,
    /// Custom inline styles
    #[serde(default)]
    pub custom_css: Option<String>,
    /// Block visibility settings
    #[serde(default)]
    pub visibility: BlockVisibility,
    /// Animation settings
    #[serde(default)]
    pub animation: Option<BlockAnimation>,
    /// Block metadata
    #[serde(default)]
    pub meta: BlockMeta,
}

impl Block {
    /// Create a new block with the given type
    pub fn new(block_type: BlockType) -> Self {
        Self {
            id: Uuid::new_v4(),
            block_type,
            attributes: BlockAttributes::default_for_type(&block_type),
            children: Vec::new(),
            styles: BlockStyles::default(),
            css_classes: Vec::new(),
            custom_css: None,
            visibility: BlockVisibility::default(),
            animation: None,
            meta: BlockMeta::default(),
        }
    }

    /// Check if block can contain children
    pub fn is_container(&self) -> bool {
        matches!(
            self.block_type,
            BlockType::Group
                | BlockType::Columns
                | BlockType::Column
                | BlockType::Section
                | BlockType::Accordion
                | BlockType::AccordionItem
                | BlockType::Tabs
                | BlockType::Tab
                | BlockType::List
                | BlockType::ListItem
        )
    }

    /// Get the block's plain text content
    pub fn get_text_content(&self) -> String {
        self.attributes.get_text_content()
    }
}

/// All available block types (30+ types)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockType {
    // Text blocks (1-10)
    Paragraph,
    Heading,
    List,
    ListItem,
    Quote,
    Code,
    Preformatted,
    PullQuote,
    Verse,
    Footnotes,

    // Media blocks (11-20)
    Image,
    Gallery,
    Video,
    Audio,
    File,
    MediaText,
    Cover,
    Embed,
    VideoPlaylist,
    AudioPlaylist,

    // Layout blocks (21-30)
    Group,
    Columns,
    Column,
    Section,
    Row,
    Stack,
    Grid,
    Spacer,
    Separator,
    PageBreak,

    // Design blocks (31-40)
    Button,
    Buttons,
    Icon,
    SocialLinks,
    SocialLink,
    SiteTitle,
    SiteLogo,
    SiteTagline,
    Navigation,
    SearchBlock,

    // Interactive blocks (41-50)
    Accordion,
    AccordionItem,
    Tabs,
    Tab,
    ToggleContent,
    Modal,
    Tooltip,
    CountdownTimer,
    ProgressBar,
    Rating,

    // Widget blocks (51-60)
    Shortcode,
    Html,
    CustomHtml,
    ReusableBlock,
    TemplatePart,
    PostTitle,
    PostContent,
    PostExcerpt,
    PostFeaturedImage,
    PostMeta,

    // Query/Dynamic blocks (61-70)
    QueryLoop,
    PostTemplate,
    ArchiveTitle,
    TermDescription,
    PostNavigationLink,
    PostComments,
    CommentsQueryLoop,
    CommentTemplate,
    Pagination,
    ReadMore,

    // Form blocks (71-80)
    Form,
    FormInput,
    FormTextarea,
    FormSelect,
    FormCheckbox,
    FormRadio,
    FormSubmit,
    FormLabel,
    FormFieldset,
    FormHidden,

    // Advanced blocks (81-90)
    Table,
    TableOfContents,
    Footnote,
    Citation,
    Definition,
    Abbreviation,
    DetailsDisclosure,
    Map,
    Chart,
    Timeline,

    // Special blocks (91-100)
    Alert,
    Callout,
    Card,
    Testimonial,
    TeamMember,
    PricingTable,
    PricingColumn,
    Comparison,
    BeforeAfter,
    Counter,

    // Custom/Plugin blocks
    Custom(u32),
}

impl BlockType {
    /// Get the category for this block type
    pub fn category(&self) -> BlockCategory {
        match self {
            // Text
            BlockType::Paragraph
            | BlockType::Heading
            | BlockType::List
            | BlockType::ListItem
            | BlockType::Quote
            | BlockType::Code
            | BlockType::Preformatted
            | BlockType::PullQuote
            | BlockType::Verse
            | BlockType::Footnotes => BlockCategory::Text,

            // Media
            BlockType::Image
            | BlockType::Gallery
            | BlockType::Video
            | BlockType::Audio
            | BlockType::File
            | BlockType::MediaText
            | BlockType::Cover
            | BlockType::Embed
            | BlockType::VideoPlaylist
            | BlockType::AudioPlaylist => BlockCategory::Media,

            // Layout
            BlockType::Group
            | BlockType::Columns
            | BlockType::Column
            | BlockType::Section
            | BlockType::Row
            | BlockType::Stack
            | BlockType::Grid
            | BlockType::Spacer
            | BlockType::Separator
            | BlockType::PageBreak => BlockCategory::Layout,

            // Design
            BlockType::Button
            | BlockType::Buttons
            | BlockType::Icon
            | BlockType::SocialLinks
            | BlockType::SocialLink
            | BlockType::SiteTitle
            | BlockType::SiteLogo
            | BlockType::SiteTagline
            | BlockType::Navigation
            | BlockType::SearchBlock => BlockCategory::Design,

            // Interactive
            BlockType::Accordion
            | BlockType::AccordionItem
            | BlockType::Tabs
            | BlockType::Tab
            | BlockType::ToggleContent
            | BlockType::Modal
            | BlockType::Tooltip
            | BlockType::CountdownTimer
            | BlockType::ProgressBar
            | BlockType::Rating
            | BlockType::Form
            | BlockType::FormInput
            | BlockType::FormTextarea
            | BlockType::FormSelect
            | BlockType::FormCheckbox
            | BlockType::FormRadio
            | BlockType::FormSubmit
            | BlockType::FormLabel
            | BlockType::FormFieldset
            | BlockType::FormHidden => BlockCategory::Interactive,

            // Widgets
            BlockType::Shortcode
            | BlockType::Html
            | BlockType::CustomHtml
            | BlockType::ReusableBlock
            | BlockType::TemplatePart
            | BlockType::PostTitle
            | BlockType::PostContent
            | BlockType::PostExcerpt
            | BlockType::PostFeaturedImage
            | BlockType::PostMeta
            | BlockType::QueryLoop
            | BlockType::PostTemplate
            | BlockType::ArchiveTitle
            | BlockType::TermDescription
            | BlockType::PostNavigationLink
            | BlockType::PostComments
            | BlockType::CommentsQueryLoop
            | BlockType::CommentTemplate
            | BlockType::Pagination
            | BlockType::ReadMore => BlockCategory::Widgets,

            // Custom
            _ => BlockCategory::Widgets,
        }
    }

    /// Get display name for block type
    pub fn display_name(&self) -> &'static str {
        match self {
            BlockType::Paragraph => "Paragraph",
            BlockType::Heading => "Heading",
            BlockType::List => "List",
            BlockType::ListItem => "List Item",
            BlockType::Quote => "Quote",
            BlockType::Code => "Code",
            BlockType::Preformatted => "Preformatted",
            BlockType::PullQuote => "Pull Quote",
            BlockType::Verse => "Verse",
            BlockType::Footnotes => "Footnotes",
            BlockType::Image => "Image",
            BlockType::Gallery => "Gallery",
            BlockType::Video => "Video",
            BlockType::Audio => "Audio",
            BlockType::File => "File",
            BlockType::MediaText => "Media & Text",
            BlockType::Cover => "Cover",
            BlockType::Embed => "Embed",
            BlockType::VideoPlaylist => "Video Playlist",
            BlockType::AudioPlaylist => "Audio Playlist",
            BlockType::Group => "Group",
            BlockType::Columns => "Columns",
            BlockType::Column => "Column",
            BlockType::Section => "Section",
            BlockType::Row => "Row",
            BlockType::Stack => "Stack",
            BlockType::Grid => "Grid",
            BlockType::Spacer => "Spacer",
            BlockType::Separator => "Separator",
            BlockType::PageBreak => "Page Break",
            BlockType::Button => "Button",
            BlockType::Buttons => "Buttons",
            BlockType::Icon => "Icon",
            BlockType::SocialLinks => "Social Links",
            BlockType::SocialLink => "Social Link",
            BlockType::SiteTitle => "Site Title",
            BlockType::SiteLogo => "Site Logo",
            BlockType::SiteTagline => "Site Tagline",
            BlockType::Navigation => "Navigation",
            BlockType::SearchBlock => "Search",
            BlockType::Accordion => "Accordion",
            BlockType::AccordionItem => "Accordion Item",
            BlockType::Tabs => "Tabs",
            BlockType::Tab => "Tab",
            BlockType::ToggleContent => "Toggle Content",
            BlockType::Modal => "Modal",
            BlockType::Tooltip => "Tooltip",
            BlockType::CountdownTimer => "Countdown Timer",
            BlockType::ProgressBar => "Progress Bar",
            BlockType::Rating => "Rating",
            BlockType::Form => "Form",
            BlockType::FormInput => "Input Field",
            BlockType::FormTextarea => "Text Area",
            BlockType::FormSelect => "Select",
            BlockType::FormCheckbox => "Checkbox",
            BlockType::FormRadio => "Radio Button",
            BlockType::FormSubmit => "Submit Button",
            BlockType::FormLabel => "Label",
            BlockType::FormFieldset => "Fieldset",
            BlockType::FormHidden => "Hidden Field",
            BlockType::Shortcode => "Shortcode",
            BlockType::Html => "HTML",
            BlockType::CustomHtml => "Custom HTML",
            BlockType::ReusableBlock => "Reusable Block",
            BlockType::TemplatePart => "Template Part",
            BlockType::PostTitle => "Post Title",
            BlockType::PostContent => "Post Content",
            BlockType::PostExcerpt => "Post Excerpt",
            BlockType::PostFeaturedImage => "Featured Image",
            BlockType::PostMeta => "Post Meta",
            BlockType::QueryLoop => "Query Loop",
            BlockType::PostTemplate => "Post Template",
            BlockType::ArchiveTitle => "Archive Title",
            BlockType::TermDescription => "Term Description",
            BlockType::PostNavigationLink => "Post Navigation",
            BlockType::PostComments => "Post Comments",
            BlockType::CommentsQueryLoop => "Comments Query",
            BlockType::CommentTemplate => "Comment Template",
            BlockType::Pagination => "Pagination",
            BlockType::ReadMore => "Read More",
            BlockType::Table => "Table",
            BlockType::TableOfContents => "Table of Contents",
            BlockType::Footnote => "Footnote",
            BlockType::Citation => "Citation",
            BlockType::Definition => "Definition",
            BlockType::Abbreviation => "Abbreviation",
            BlockType::DetailsDisclosure => "Details",
            BlockType::Map => "Map",
            BlockType::Chart => "Chart",
            BlockType::Timeline => "Timeline",
            BlockType::Alert => "Alert",
            BlockType::Callout => "Callout",
            BlockType::Card => "Card",
            BlockType::Testimonial => "Testimonial",
            BlockType::TeamMember => "Team Member",
            BlockType::PricingTable => "Pricing Table",
            BlockType::PricingColumn => "Pricing Column",
            BlockType::Comparison => "Comparison",
            BlockType::BeforeAfter => "Before/After",
            BlockType::Counter => "Counter",
            BlockType::Custom(_) => "Custom Block",
        }
    }
}

/// Block attributes container
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlockAttributes {
    /// Rich text content (for text blocks)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Heading level (1-6)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u8>,

    /// Text alignment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align: Option<TextAlign>,

    /// Media URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Media ID (for attachments)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_id: Option<i64>,

    /// Alt text for images
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt: Option<String>,

    /// Caption text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,

    /// Link URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,

    /// Link target (_blank, _self, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_target: Option<String>,

    /// Link rel attribute
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_rel: Option<String>,

    /// Width (pixels or percentage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,

    /// Height (pixels or percentage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<String>,

    /// Size preset (thumbnail, medium, large, full)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,

    /// Gallery images
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub images: Vec<GalleryImage>,

    /// Columns count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<u8>,

    /// Column widths (for columns block)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub column_widths: Vec<f32>,

    /// List type (ordered/unordered)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_type: Option<ListType>,

    /// List start number (for ordered lists)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<u32>,

    /// Reversed order (for ordered lists)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reversed: Option<bool>,

    /// Programming language (for code blocks)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Show line numbers (for code blocks)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_line_numbers: Option<bool>,

    /// Embed provider (youtube, vimeo, twitter, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// Video autoplay
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autoplay: Option<bool>,

    /// Video loop
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loop_video: Option<bool>,

    /// Video muted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muted: Option<bool>,

    /// Video controls
    #[serde(skip_serializing_if = "Option::is_none")]
    pub controls: Option<bool>,

    /// Poster image (for video)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster: Option<String>,

    /// Citation text (for quotes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation: Option<String>,

    /// Table data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_data: Option<TableData>,

    /// Spacer height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacer_height: Option<String>,

    /// Separator style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub separator_style: Option<SeparatorStyle>,

    /// Button text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button_text: Option<String>,

    /// Button style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button_style: Option<ButtonStyle>,

    /// Icon name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,

    /// Icon size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_size: Option<String>,

    /// Social service (for social links)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,

    /// Accordion default open
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_open: Option<bool>,

    /// Alert type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert_type: Option<AlertType>,

    /// Countdown target date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_date: Option<DateTime<Utc>>,

    /// Progress value (0-100)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<u8>,

    /// Rating value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<f32>,

    /// Rating max value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating_max: Option<u8>,

    /// Map coordinates
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coordinates: Option<MapCoordinates>,

    /// Map zoom level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zoom: Option<u8>,

    /// Chart data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chart_data: Option<ChartData>,

    /// Form action URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,

    /// Form method
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,

    /// Input type (text, email, number, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_type: Option<String>,

    /// Placeholder text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,

    /// Required field
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,

    /// Field name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Field value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// Select options
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<SelectOption>,

    /// Counter start value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counter_start: Option<i64>,

    /// Counter end value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counter_end: Option<i64>,

    /// Counter prefix
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counter_prefix: Option<String>,

    /// Counter suffix
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counter_suffix: Option<String>,

    /// TOC max depth
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toc_max_depth: Option<u8>,

    /// TOC show numbers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toc_numbered: Option<bool>,

    /// Reusable block reference ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_id: Option<i64>,

    /// Query parameters (for query loop)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<QueryParams>,

    /// Custom attributes map for extensibility
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom: HashMap<String, serde_json::Value>,
}

impl BlockAttributes {
    /// Create default attributes for a block type
    pub fn default_for_type(block_type: &BlockType) -> Self {
        let mut attrs = Self::default();
        match block_type {
            BlockType::Heading => {
                attrs.level = Some(2);
            }
            BlockType::List => {
                attrs.list_type = Some(ListType::Unordered);
            }
            BlockType::Code => {
                attrs.language = Some("plaintext".to_string());
                attrs.show_line_numbers = Some(true);
            }
            BlockType::Video => {
                attrs.controls = Some(true);
            }
            BlockType::Spacer => {
                attrs.spacer_height = Some("50px".to_string());
            }
            BlockType::Separator => {
                attrs.separator_style = Some(SeparatorStyle::Solid);
            }
            BlockType::Button => {
                attrs.button_style = Some(ButtonStyle::Primary);
            }
            BlockType::ProgressBar => {
                attrs.progress = Some(50);
            }
            BlockType::Rating => {
                attrs.rating_max = Some(5);
            }
            BlockType::Counter => {
                attrs.counter_start = Some(0);
                attrs.counter_end = Some(100);
            }
            BlockType::TableOfContents => {
                attrs.toc_max_depth = Some(3);
                attrs.toc_numbered = Some(true);
            }
            _ => {}
        }
        attrs
    }

    /// Get plain text content from attributes
    pub fn get_text_content(&self) -> String {
        self.content.clone().unwrap_or_default()
    }
}

/// Gallery image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryImage {
    pub id: i64,
    pub url: String,
    pub alt: Option<String>,
    pub caption: Option<String>,
    pub link: Option<String>,
}

/// Text alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

/// List type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ListType {
    Ordered,
    Unordered,
    Checklist,
}

/// Separator styles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SeparatorStyle {
    Solid,
    Dashed,
    Dotted,
    Double,
    Gradient,
}

/// Button styles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Outline,
    Ghost,
    Link,
}

/// Alert types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertType {
    Info,
    Success,
    Warning,
    Error,
}

/// Table data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub has_fixed_layout: bool,
    pub has_header_row: bool,
    pub has_footer_row: bool,
}

/// Map coordinates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapCoordinates {
    pub lat: f64,
    pub lng: f64,
}

/// Chart data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    pub chart_type: ChartType,
    pub labels: Vec<String>,
    pub datasets: Vec<ChartDataset>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChartType {
    Bar,
    Line,
    Pie,
    Doughnut,
    Area,
    Radar,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDataset {
    pub label: String,
    pub data: Vec<f64>,
    pub color: Option<String>,
}

/// Select option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    pub label: String,
    pub value: String,
    pub selected: bool,
    pub disabled: bool,
}

/// Query parameters for dynamic blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryParams {
    pub post_type: String,
    pub posts_per_page: u32,
    pub order_by: String,
    pub order: String,
    pub categories: Vec<i64>,
    pub tags: Vec<i64>,
    pub author: Option<i64>,
    pub search: Option<String>,
    pub date_query: Option<DateQuery>,
    pub meta_query: Option<Vec<MetaQuery>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateQuery {
    pub after: Option<String>,
    pub before: Option<String>,
    pub inclusive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaQuery {
    pub key: String,
    pub value: Option<String>,
    pub compare: String,
    pub type_: String,
}

/// Block styles
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlockStyles {
    /// Background color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,

    /// Background gradient
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_gradient: Option<String>,

    /// Background image
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_image: Option<String>,

    /// Text color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_color: Option<String>,

    /// Link color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_color: Option<String>,

    /// Font size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,

    /// Font family
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,

    /// Font weight
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<String>,

    /// Line height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_height: Option<String>,

    /// Letter spacing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub letter_spacing: Option<String>,

    /// Text transform
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_transform: Option<String>,

    /// Text decoration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_decoration: Option<String>,

    /// Border radius
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_radius: Option<String>,

    /// Border width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_width: Option<String>,

    /// Border color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_color: Option<String>,

    /// Border style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_style: Option<String>,

    /// Box shadow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub box_shadow: Option<String>,

    /// Padding (all sides or individual)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding: Option<Spacing>,

    /// Margin (all sides or individual)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin: Option<Spacing>,

    /// Min height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_height: Option<String>,

    /// Max width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_width: Option<String>,

    /// Opacity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,

    /// Z-index
    #[serde(skip_serializing_if = "Option::is_none")]
    pub z_index: Option<i32>,

    /// Display
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,

    /// Flex direction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_direction: Option<String>,

    /// Justify content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub justify_content: Option<String>,

    /// Align items
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align_items: Option<String>,

    /// Gap
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<String>,

    /// Responsive overrides
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub responsive: HashMap<String, Box<BlockStyles>>,
}

/// Spacing (padding/margin)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Spacing {
    All(String),
    Individual {
        top: Option<String>,
        right: Option<String>,
        bottom: Option<String>,
        left: Option<String>,
    },
}

/// Block visibility settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlockVisibility {
    /// Show on desktop
    #[serde(default = "default_true")]
    pub desktop: bool,

    /// Show on tablet
    #[serde(default = "default_true")]
    pub tablet: bool,

    /// Show on mobile
    #[serde(default = "default_true")]
    pub mobile: bool,

    /// Only show to logged-in users
    #[serde(default)]
    pub logged_in_only: bool,

    /// Only show to specific roles
    #[serde(default)]
    pub roles: Vec<String>,

    /// Schedule visibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<VisibilitySchedule>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibilitySchedule {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
}

/// Block animation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockAnimation {
    pub animation_type: AnimationType,
    pub duration: u32,      // milliseconds
    pub delay: u32,         // milliseconds
    pub easing: String,
    pub trigger: AnimationTrigger,
    pub repeat: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationType {
    FadeIn,
    FadeInUp,
    FadeInDown,
    FadeInLeft,
    FadeInRight,
    SlideInUp,
    SlideInDown,
    SlideInLeft,
    SlideInRight,
    ZoomIn,
    ZoomOut,
    Bounce,
    Pulse,
    Shake,
    Flip,
    RotateIn,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationTrigger {
    OnLoad,
    OnScroll,
    OnHover,
    OnClick,
}

/// Block metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlockMeta {
    /// Block label (for layers panel)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// Locked (cannot edit)
    #[serde(default)]
    pub locked: bool,

    /// Anchor/ID for linking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor: Option<String>,

    /// Additional HTML attributes
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub html_attributes: HashMap<String, String>,

    /// Creation timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,

    /// Last modified timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<DateTime<Utc>>,
}
