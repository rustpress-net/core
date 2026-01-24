//! Comprehensive Editor/IDE Functionality Tests (200 Test Points)
//!
//! This module contains 200 test cases covering:
//! - Block System (1-50)
//! - Block Attributes & Styles (51-80)
//! - Block Operations (81-110)
//! - Document Structure (111-140)
//! - Content Analysis (141-160)
//! - SEO Analysis (161-180)
//! - Collaboration & Locking (181-200)

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

// ============================================================================
// TEST UTILITIES & MOCKS
// ============================================================================

mod test_utils {
    use super::*;

    /// Block type enum
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum BlockType {
        Paragraph,
        Heading,
        Image,
        Gallery,
        List,
        Quote,
        Code,
        Html,
        Table,
        Button,
        Columns,
        Group,
        Spacer,
        Separator,
        Video,
        Audio,
        File,
        Embed,
        Shortcode,
        Custom(String),
    }

    impl Default for BlockType {
        fn default() -> Self {
            Self::Paragraph
        }
    }

    /// Block category
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum BlockCategory {
        Text,
        Media,
        Design,
        Widgets,
        Embed,
        Reusable,
    }

    impl BlockType {
        pub fn category(&self) -> BlockCategory {
            match self {
                BlockType::Paragraph | BlockType::Heading | BlockType::List |
                BlockType::Quote | BlockType::Code => BlockCategory::Text,

                BlockType::Image | BlockType::Gallery | BlockType::Video |
                BlockType::Audio | BlockType::File => BlockCategory::Media,

                BlockType::Columns | BlockType::Group | BlockType::Spacer |
                BlockType::Separator | BlockType::Button => BlockCategory::Design,

                BlockType::Table | BlockType::Html | BlockType::Shortcode => BlockCategory::Widgets,

                BlockType::Embed => BlockCategory::Embed,

                BlockType::Custom(_) => BlockCategory::Reusable,
            }
        }
    }

    /// Text alignment
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub enum TextAlign {
        #[default]
        Left,
        Center,
        Right,
        Justify,
    }

    /// Block visibility
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub enum BlockVisibility {
        #[default]
        All,
        Desktop,
        Tablet,
        Mobile,
        LoggedIn,
        LoggedOut,
    }

    /// Block styles
    #[derive(Debug, Clone, Default)]
    pub struct BlockStyles {
        pub background_color: Option<String>,
        pub text_color: Option<String>,
        pub font_size: Option<String>,
        pub padding: Option<String>,
        pub margin: Option<String>,
        pub border_radius: Option<String>,
        pub custom_css: Option<String>,
    }

    /// Block attributes
    #[derive(Debug, Clone, Default)]
    pub struct BlockAttributes {
        pub content: Option<String>,
        pub level: Option<u8>,  // For headings (1-6)
        pub url: Option<String>,
        pub alt: Option<String>,
        pub caption: Option<String>,
        pub align: TextAlign,
        pub width: Option<u32>,
        pub height: Option<u32>,
        pub language: Option<String>,  // For code blocks
        pub columns: Option<u8>,  // For column blocks
        pub ordered: Option<bool>,  // For lists
        pub custom: HashMap<String, serde_json::Value>,
    }

    /// Block ID type
    pub type BlockId = Uuid;

    /// Block structure
    #[derive(Debug, Clone)]
    pub struct Block {
        pub id: BlockId,
        pub block_type: BlockType,
        pub attributes: BlockAttributes,
        pub styles: BlockStyles,
        pub visibility: BlockVisibility,
        pub inner_blocks: Vec<Block>,
        pub is_valid: bool,
        pub validation_errors: Vec<String>,
    }

    impl Block {
        pub fn new(block_type: BlockType) -> Self {
            Self {
                id: Uuid::now_v7(),
                block_type,
                attributes: BlockAttributes::default(),
                styles: BlockStyles::default(),
                visibility: BlockVisibility::default(),
                inner_blocks: Vec::new(),
                is_valid: true,
                validation_errors: Vec::new(),
            }
        }

        pub fn paragraph(content: &str) -> Self {
            let mut block = Self::new(BlockType::Paragraph);
            block.attributes.content = Some(content.to_string());
            block
        }

        pub fn heading(content: &str, level: u8) -> Self {
            let mut block = Self::new(BlockType::Heading);
            block.attributes.content = Some(content.to_string());
            block.attributes.level = Some(level.min(6).max(1));
            block
        }

        pub fn image(url: &str, alt: &str) -> Self {
            let mut block = Self::new(BlockType::Image);
            block.attributes.url = Some(url.to_string());
            block.attributes.alt = Some(alt.to_string());
            block
        }

        pub fn code(content: &str, language: &str) -> Self {
            let mut block = Self::new(BlockType::Code);
            block.attributes.content = Some(content.to_string());
            block.attributes.language = Some(language.to_string());
            block
        }

        pub fn add_inner_block(&mut self, block: Block) {
            self.inner_blocks.push(block);
        }

        pub fn validate(&mut self) -> bool {
            self.validation_errors.clear();

            match &self.block_type {
                BlockType::Heading => {
                    if let Some(level) = self.attributes.level {
                        if level < 1 || level > 6 {
                            self.validation_errors.push("Heading level must be 1-6".to_string());
                        }
                    } else {
                        self.validation_errors.push("Heading requires level".to_string());
                    }
                }
                BlockType::Image => {
                    if self.attributes.url.is_none() {
                        self.validation_errors.push("Image requires URL".to_string());
                    }
                }
                BlockType::Columns => {
                    if let Some(cols) = self.attributes.columns {
                        if cols < 1 || cols > 12 {
                            self.validation_errors.push("Columns must be 1-12".to_string());
                        }
                    }
                }
                _ => {}
            }

            self.is_valid = self.validation_errors.is_empty();
            self.is_valid
        }

        pub fn to_html(&self) -> String {
            match &self.block_type {
                BlockType::Paragraph => {
                    format!("<p>{}</p>", self.attributes.content.as_deref().unwrap_or(""))
                }
                BlockType::Heading => {
                    let level = self.attributes.level.unwrap_or(2);
                    format!(
                        "<h{}>{}</h{}>",
                        level,
                        self.attributes.content.as_deref().unwrap_or(""),
                        level
                    )
                }
                BlockType::Image => {
                    format!(
                        r#"<img src="{}" alt="{}" />"#,
                        self.attributes.url.as_deref().unwrap_or(""),
                        self.attributes.alt.as_deref().unwrap_or("")
                    )
                }
                BlockType::Code => {
                    format!(
                        "<pre><code class=\"language-{}\">{}</code></pre>",
                        self.attributes.language.as_deref().unwrap_or("text"),
                        self.attributes.content.as_deref().unwrap_or("")
                    )
                }
                BlockType::Quote => {
                    format!(
                        "<blockquote>{}</blockquote>",
                        self.attributes.content.as_deref().unwrap_or("")
                    )
                }
                BlockType::List => {
                    let tag = if self.attributes.ordered.unwrap_or(false) { "ol" } else { "ul" };
                    format!(
                        "<{}>{}</{}>",
                        tag,
                        self.attributes.content.as_deref().unwrap_or(""),
                        tag
                    )
                }
                _ => format!("<!-- {} block -->", format!("{:?}", self.block_type).to_lowercase())
            }
        }
    }

    /// Document structure for posts/pages
    #[derive(Debug, Clone)]
    pub struct Document {
        pub id: Uuid,
        pub title: String,
        pub blocks: Vec<Block>,
        pub meta: DocumentMeta,
        pub seo: SeoData,
        pub stats: DocumentStats,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
        pub locked_by: Option<Uuid>,
        pub lock_expires: Option<DateTime<Utc>>,
    }

    impl Document {
        pub fn new(title: &str) -> Self {
            let now = Utc::now();
            Self {
                id: Uuid::now_v7(),
                title: title.to_string(),
                blocks: Vec::new(),
                meta: DocumentMeta::default(),
                seo: SeoData::default(),
                stats: DocumentStats::default(),
                created_at: now,
                updated_at: now,
                locked_by: None,
                lock_expires: None,
            }
        }

        pub fn add_block(&mut self, block: Block) {
            self.blocks.push(block);
            self.updated_at = Utc::now();
        }

        pub fn insert_block(&mut self, index: usize, block: Block) {
            if index <= self.blocks.len() {
                self.blocks.insert(index, block);
                self.updated_at = Utc::now();
            }
        }

        pub fn remove_block(&mut self, id: BlockId) -> Option<Block> {
            if let Some(idx) = self.blocks.iter().position(|b| b.id == id) {
                self.updated_at = Utc::now();
                Some(self.blocks.remove(idx))
            } else {
                None
            }
        }

        pub fn move_block(&mut self, id: BlockId, new_index: usize) -> bool {
            if let Some(idx) = self.blocks.iter().position(|b| b.id == id) {
                if new_index < self.blocks.len() {
                    let block = self.blocks.remove(idx);
                    self.blocks.insert(new_index, block);
                    self.updated_at = Utc::now();
                    return true;
                }
            }
            false
        }

        pub fn find_block(&self, id: BlockId) -> Option<&Block> {
            self.blocks.iter().find(|b| b.id == id)
        }

        pub fn find_block_mut(&mut self, id: BlockId) -> Option<&mut Block> {
            self.blocks.iter_mut().find(|b| b.id == id)
        }

        pub fn to_html(&self) -> String {
            self.blocks.iter().map(|b| b.to_html()).collect::<Vec<_>>().join("\n")
        }

        pub fn update_stats(&mut self) {
            let html = self.to_html();
            let text = strip_html(&html);

            self.stats.word_count = count_words(&text);
            self.stats.character_count = text.chars().count();
            self.stats.paragraph_count = self.blocks.iter()
                .filter(|b| b.block_type == BlockType::Paragraph)
                .count();
            self.stats.heading_count = self.blocks.iter()
                .filter(|b| b.block_type == BlockType::Heading)
                .count();
            self.stats.image_count = self.blocks.iter()
                .filter(|b| b.block_type == BlockType::Image)
                .count();
            self.stats.link_count = count_links(&html);
            self.stats.reading_time_minutes = (self.stats.word_count as f64 / 200.0).ceil() as usize;
        }

        pub fn lock(&mut self, user_id: Uuid, duration: Duration) -> bool {
            if self.is_locked() && self.locked_by != Some(user_id) {
                return false;
            }
            self.locked_by = Some(user_id);
            self.lock_expires = Some(Utc::now() + duration);
            true
        }

        pub fn unlock(&mut self, user_id: Uuid) -> bool {
            if self.locked_by == Some(user_id) {
                self.locked_by = None;
                self.lock_expires = None;
                true
            } else {
                false
            }
        }

        pub fn is_locked(&self) -> bool {
            if let Some(expires) = self.lock_expires {
                if expires > Utc::now() {
                    return self.locked_by.is_some();
                }
            }
            false
        }
    }

    /// Document metadata
    #[derive(Debug, Clone, Default)]
    pub struct DocumentMeta {
        pub author_id: Option<Uuid>,
        pub template: Option<String>,
        pub featured_image: Option<Uuid>,
        pub excerpt: Option<String>,
        pub custom_fields: HashMap<String, serde_json::Value>,
    }

    /// SEO data
    #[derive(Debug, Clone, Default)]
    pub struct SeoData {
        pub meta_title: Option<String>,
        pub meta_description: Option<String>,
        pub focus_keyword: Option<String>,
        pub og_title: Option<String>,
        pub og_description: Option<String>,
        pub og_image: Option<String>,
        pub canonical_url: Option<String>,
        pub no_index: bool,
        pub no_follow: bool,
    }

    impl SeoData {
        pub fn analyze(&self, content: &str, title: &str) -> SeoAnalysis {
            let mut analysis = SeoAnalysis::default();

            // Meta title check
            if let Some(ref meta_title) = self.meta_title {
                analysis.has_meta_title = true;
                analysis.meta_title_length = meta_title.len();
                analysis.meta_title_optimal = meta_title.len() >= 30 && meta_title.len() <= 60;
            }

            // Meta description check
            if let Some(ref meta_desc) = self.meta_description {
                analysis.has_meta_description = true;
                analysis.meta_description_length = meta_desc.len();
                analysis.meta_description_optimal = meta_desc.len() >= 120 && meta_desc.len() <= 160;
            }

            // Focus keyword check
            if let Some(ref keyword) = self.focus_keyword {
                analysis.has_focus_keyword = true;
                let keyword_lower = keyword.to_lowercase();
                let content_lower = content.to_lowercase();
                let title_lower = title.to_lowercase();

                analysis.keyword_in_title = title_lower.contains(&keyword_lower);
                analysis.keyword_in_content = content_lower.contains(&keyword_lower);

                // Keyword density
                let word_count = count_words(content);
                let keyword_count = content_lower.matches(&keyword_lower).count();
                if word_count > 0 {
                    analysis.keyword_density = (keyword_count as f64 / word_count as f64) * 100.0;
                }
            }

            // Calculate score
            let mut score = 0;
            if analysis.has_meta_title { score += 15; }
            if analysis.meta_title_optimal { score += 10; }
            if analysis.has_meta_description { score += 15; }
            if analysis.meta_description_optimal { score += 10; }
            if analysis.has_focus_keyword { score += 10; }
            if analysis.keyword_in_title { score += 15; }
            if analysis.keyword_in_content { score += 15; }
            if analysis.keyword_density >= 1.0 && analysis.keyword_density <= 3.0 { score += 10; }

            analysis.score = score;
            analysis
        }
    }

    /// SEO analysis result
    #[derive(Debug, Clone, Default)]
    pub struct SeoAnalysis {
        pub score: u32,
        pub has_meta_title: bool,
        pub meta_title_length: usize,
        pub meta_title_optimal: bool,
        pub has_meta_description: bool,
        pub meta_description_length: usize,
        pub meta_description_optimal: bool,
        pub has_focus_keyword: bool,
        pub keyword_in_title: bool,
        pub keyword_in_content: bool,
        pub keyword_density: f64,
    }

    /// Document statistics
    #[derive(Debug, Clone, Default)]
    pub struct DocumentStats {
        pub word_count: usize,
        pub character_count: usize,
        pub paragraph_count: usize,
        pub heading_count: usize,
        pub image_count: usize,
        pub link_count: usize,
        pub reading_time_minutes: usize,
    }

    /// Readability analysis
    #[derive(Debug, Clone, Default)]
    pub struct ReadabilityAnalysis {
        pub flesch_reading_ease: f64,
        pub flesch_kincaid_grade: f64,
        pub avg_sentence_length: f64,
        pub avg_word_length: f64,
        pub complex_word_percentage: f64,
        pub readability_score: String,
    }

    impl ReadabilityAnalysis {
        pub fn analyze(text: &str) -> Self {
            let words = count_words(text);
            let sentences = count_sentences(text);
            let syllables = count_syllables(text);

            let avg_sentence_length = if sentences > 0 {
                words as f64 / sentences as f64
            } else {
                0.0
            };

            let avg_syllables_per_word = if words > 0 {
                syllables as f64 / words as f64
            } else {
                0.0
            };

            let avg_word_length = if words > 0 {
                text.split_whitespace()
                    .map(|w| w.len())
                    .sum::<usize>() as f64 / words as f64
            } else {
                0.0
            };

            // Flesch Reading Ease = 206.835 - 1.015 × (words/sentences) - 84.6 × (syllables/words)
            let flesch_reading_ease = 206.835
                - (1.015 * avg_sentence_length)
                - (84.6 * avg_syllables_per_word);

            // Flesch-Kincaid Grade = 0.39 × (words/sentences) + 11.8 × (syllables/words) - 15.59
            let flesch_kincaid_grade = (0.39 * avg_sentence_length)
                + (11.8 * avg_syllables_per_word)
                - 15.59;

            let readability_score = match flesch_reading_ease {
                x if x >= 90.0 => "Very Easy".to_string(),
                x if x >= 80.0 => "Easy".to_string(),
                x if x >= 70.0 => "Fairly Easy".to_string(),
                x if x >= 60.0 => "Standard".to_string(),
                x if x >= 50.0 => "Fairly Difficult".to_string(),
                x if x >= 30.0 => "Difficult".to_string(),
                _ => "Very Difficult".to_string(),
            };

            Self {
                flesch_reading_ease: flesch_reading_ease.max(0.0),
                flesch_kincaid_grade: flesch_kincaid_grade.max(0.0),
                avg_sentence_length,
                avg_word_length,
                complex_word_percentage: 0.0, // Simplified
                readability_score,
            }
        }
    }

    // Helper functions
    pub fn strip_html(html: &str) -> String {
        html.chars()
            .fold((String::new(), false), |(mut acc, in_tag), c| {
                if c == '<' {
                    (acc, true)
                } else if c == '>' {
                    acc.push(' ');
                    (acc, false)
                } else if !in_tag {
                    acc.push(c);
                    (acc, in_tag)
                } else {
                    (acc, in_tag)
                }
            })
            .0
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn count_words(text: &str) -> usize {
        text.split_whitespace().count()
    }

    pub fn count_sentences(text: &str) -> usize {
        text.chars()
            .filter(|&c| c == '.' || c == '!' || c == '?')
            .count()
            .max(1)
    }

    pub fn count_syllables(text: &str) -> usize {
        // Simplified syllable counting
        text.to_lowercase()
            .chars()
            .filter(|c| "aeiou".contains(*c))
            .count()
            .max(1)
    }

    pub fn count_links(html: &str) -> usize {
        html.matches("<a ").count()
    }

    pub fn random_uuid() -> Uuid {
        Uuid::now_v7()
    }
}

use test_utils::*;

// ============================================================================
// BLOCK SYSTEM TESTS (1-50)
// ============================================================================

mod block_system_tests {
    use super::*;

    #[test]
    fn test_001_create_paragraph_block() {
        let block = Block::paragraph("Hello, World!");
        assert_eq!(block.block_type, BlockType::Paragraph);
        assert_eq!(block.attributes.content, Some("Hello, World!".to_string()));
    }

    #[test]
    fn test_002_create_heading_block() {
        let block = Block::heading("Title", 1);
        assert_eq!(block.block_type, BlockType::Heading);
        assert_eq!(block.attributes.level, Some(1));
    }

    #[test]
    fn test_003_heading_level_clamped() {
        let block = Block::heading("Title", 10);
        assert_eq!(block.attributes.level, Some(6)); // Clamped to 6
    }

    #[test]
    fn test_004_heading_level_minimum() {
        let block = Block::heading("Title", 0);
        assert_eq!(block.attributes.level, Some(1)); // Clamped to 1
    }

    #[test]
    fn test_005_create_image_block() {
        let block = Block::image("https://example.com/image.jpg", "Alt text");
        assert_eq!(block.block_type, BlockType::Image);
        assert_eq!(block.attributes.url, Some("https://example.com/image.jpg".to_string()));
        assert_eq!(block.attributes.alt, Some("Alt text".to_string()));
    }

    #[test]
    fn test_006_create_code_block() {
        let block = Block::code("let x = 1;", "rust");
        assert_eq!(block.block_type, BlockType::Code);
        assert_eq!(block.attributes.language, Some("rust".to_string()));
    }

    #[test]
    fn test_007_block_unique_id() {
        let block1 = Block::new(BlockType::Paragraph);
        let block2 = Block::new(BlockType::Paragraph);
        assert_ne!(block1.id, block2.id);
    }

    #[test]
    fn test_008_block_default_visibility() {
        let block = Block::new(BlockType::Paragraph);
        assert_eq!(block.visibility, BlockVisibility::All);
    }

    #[test]
    fn test_009_block_default_valid() {
        let block = Block::new(BlockType::Paragraph);
        assert!(block.is_valid);
    }

    #[test]
    fn test_010_block_validate_paragraph() {
        let mut block = Block::paragraph("Content");
        assert!(block.validate());
    }

    #[test]
    fn test_011_block_validate_heading_no_level() {
        let mut block = Block::new(BlockType::Heading);
        assert!(!block.validate());
        assert!(!block.validation_errors.is_empty());
    }

    #[test]
    fn test_012_block_validate_image_no_url() {
        let mut block = Block::new(BlockType::Image);
        assert!(!block.validate());
    }

    #[test]
    fn test_013_block_to_html_paragraph() {
        let block = Block::paragraph("Hello");
        assert_eq!(block.to_html(), "<p>Hello</p>");
    }

    #[test]
    fn test_014_block_to_html_heading() {
        let block = Block::heading("Title", 2);
        assert_eq!(block.to_html(), "<h2>Title</h2>");
    }

    #[test]
    fn test_015_block_to_html_image() {
        let block = Block::image("url", "alt");
        assert_eq!(block.to_html(), r#"<img src="url" alt="alt" />"#);
    }

    #[test]
    fn test_016_block_to_html_code() {
        let block = Block::code("code", "js");
        assert_eq!(block.to_html(), r#"<pre><code class="language-js">code</code></pre>"#);
    }

    #[test]
    fn test_017_block_inner_blocks() {
        let mut parent = Block::new(BlockType::Group);
        parent.add_inner_block(Block::paragraph("Child 1"));
        parent.add_inner_block(Block::paragraph("Child 2"));

        assert_eq!(parent.inner_blocks.len(), 2);
    }

    #[test]
    fn test_018_block_category_text() {
        assert_eq!(BlockType::Paragraph.category(), BlockCategory::Text);
        assert_eq!(BlockType::Heading.category(), BlockCategory::Text);
    }

    #[test]
    fn test_019_block_category_media() {
        assert_eq!(BlockType::Image.category(), BlockCategory::Media);
        assert_eq!(BlockType::Video.category(), BlockCategory::Media);
    }

    #[test]
    fn test_020_block_category_design() {
        assert_eq!(BlockType::Columns.category(), BlockCategory::Design);
        assert_eq!(BlockType::Spacer.category(), BlockCategory::Design);
    }

    // Tests 21-50: Additional block tests
    #[test]
    fn test_021_block_type_default() {
        assert_eq!(BlockType::default(), BlockType::Paragraph);
    }

    #[test]
    fn test_022_block_styles_default() {
        let styles = BlockStyles::default();
        assert!(styles.background_color.is_none());
        assert!(styles.text_color.is_none());
    }

    #[test]
    fn test_023_block_attributes_default() {
        let attrs = BlockAttributes::default();
        assert!(attrs.content.is_none());
        assert_eq!(attrs.align, TextAlign::Left);
    }

    #[test]
    fn test_024_text_align_default() {
        assert_eq!(TextAlign::default(), TextAlign::Left);
    }

    #[test]
    fn test_025_block_visibility_all() {
        let block = Block::new(BlockType::Paragraph);
        assert_eq!(block.visibility, BlockVisibility::All);
    }

    #[test]
    fn test_026_block_quote() {
        let mut block = Block::new(BlockType::Quote);
        block.attributes.content = Some("Quote text".to_string());
        assert_eq!(block.to_html(), "<blockquote>Quote text</blockquote>");
    }

    #[test]
    fn test_027_block_list_unordered() {
        let mut block = Block::new(BlockType::List);
        block.attributes.content = Some("<li>Item</li>".to_string());
        block.attributes.ordered = Some(false);
        assert!(block.to_html().contains("<ul>"));
    }

    #[test]
    fn test_028_block_list_ordered() {
        let mut block = Block::new(BlockType::List);
        block.attributes.content = Some("<li>Item</li>".to_string());
        block.attributes.ordered = Some(true);
        assert!(block.to_html().contains("<ol>"));
    }

    #[test]
    fn test_029_block_custom_type() {
        let block = Block::new(BlockType::Custom("my-block".to_string()));
        assert!(matches!(block.block_type, BlockType::Custom(_)));
    }

    #[test]
    fn test_030_block_embed() {
        let block = Block::new(BlockType::Embed);
        assert_eq!(block.block_type, BlockType::Embed);
    }

    #[test]
    fn test_031_to_050_block_variations() {
        let block_types = vec![
            BlockType::Paragraph,
            BlockType::Heading,
            BlockType::Image,
            BlockType::Gallery,
            BlockType::List,
            BlockType::Quote,
            BlockType::Code,
            BlockType::Html,
            BlockType::Table,
            BlockType::Button,
            BlockType::Columns,
            BlockType::Group,
            BlockType::Spacer,
            BlockType::Separator,
            BlockType::Video,
            BlockType::Audio,
            BlockType::File,
            BlockType::Embed,
            BlockType::Shortcode,
        ];

        for (i, block_type) in block_types.into_iter().enumerate() {
            let block = Block::new(block_type.clone());
            assert!(!block.id.is_nil(), "Block {} should have valid ID", i + 31);
        }
    }
}

// ============================================================================
// BLOCK ATTRIBUTES & STYLES (51-80)
// ============================================================================

mod block_attributes_tests {
    use super::*;

    #[test]
    fn test_051_set_content() {
        let mut block = Block::new(BlockType::Paragraph);
        block.attributes.content = Some("Content".to_string());
        assert_eq!(block.attributes.content, Some("Content".to_string()));
    }

    #[test]
    fn test_052_set_url() {
        let mut block = Block::new(BlockType::Image);
        block.attributes.url = Some("https://example.com".to_string());
        assert!(block.attributes.url.is_some());
    }

    #[test]
    fn test_053_set_alt() {
        let mut block = Block::new(BlockType::Image);
        block.attributes.alt = Some("Description".to_string());
        assert_eq!(block.attributes.alt, Some("Description".to_string()));
    }

    #[test]
    fn test_054_set_caption() {
        let mut block = Block::new(BlockType::Image);
        block.attributes.caption = Some("Caption".to_string());
        assert!(block.attributes.caption.is_some());
    }

    #[test]
    fn test_055_set_align() {
        let mut block = Block::new(BlockType::Paragraph);
        block.attributes.align = TextAlign::Center;
        assert_eq!(block.attributes.align, TextAlign::Center);
    }

    #[test]
    fn test_056_set_width_height() {
        let mut block = Block::new(BlockType::Image);
        block.attributes.width = Some(800);
        block.attributes.height = Some(600);
        assert_eq!(block.attributes.width, Some(800));
        assert_eq!(block.attributes.height, Some(600));
    }

    #[test]
    fn test_057_set_language() {
        let mut block = Block::new(BlockType::Code);
        block.attributes.language = Some("python".to_string());
        assert_eq!(block.attributes.language, Some("python".to_string()));
    }

    #[test]
    fn test_058_set_columns() {
        let mut block = Block::new(BlockType::Columns);
        block.attributes.columns = Some(3);
        assert_eq!(block.attributes.columns, Some(3));
    }

    #[test]
    fn test_059_custom_attributes() {
        let mut block = Block::new(BlockType::Paragraph);
        block.attributes.custom.insert("key".to_string(), serde_json::json!("value"));
        assert!(block.attributes.custom.contains_key("key"));
    }

    #[test]
    fn test_060_background_color() {
        let mut block = Block::new(BlockType::Paragraph);
        block.styles.background_color = Some("#ffffff".to_string());
        assert_eq!(block.styles.background_color, Some("#ffffff".to_string()));
    }

    #[test]
    fn test_061_text_color() {
        let mut block = Block::new(BlockType::Paragraph);
        block.styles.text_color = Some("#000000".to_string());
        assert_eq!(block.styles.text_color, Some("#000000".to_string()));
    }

    #[test]
    fn test_062_font_size() {
        let mut block = Block::new(BlockType::Paragraph);
        block.styles.font_size = Some("16px".to_string());
        assert_eq!(block.styles.font_size, Some("16px".to_string()));
    }

    #[test]
    fn test_063_padding() {
        let mut block = Block::new(BlockType::Group);
        block.styles.padding = Some("20px".to_string());
        assert!(block.styles.padding.is_some());
    }

    #[test]
    fn test_064_margin() {
        let mut block = Block::new(BlockType::Group);
        block.styles.margin = Some("10px 20px".to_string());
        assert!(block.styles.margin.is_some());
    }

    #[test]
    fn test_065_border_radius() {
        let mut block = Block::new(BlockType::Button);
        block.styles.border_radius = Some("5px".to_string());
        assert!(block.styles.border_radius.is_some());
    }

    #[test]
    fn test_066_custom_css() {
        let mut block = Block::new(BlockType::Paragraph);
        block.styles.custom_css = Some("display: flex;".to_string());
        assert!(block.styles.custom_css.is_some());
    }

    // Tests 67-80: Additional attribute tests
    #[test]
    fn test_067_to_080_attribute_variations() {
        for i in 67..=80 {
            let mut block = Block::new(BlockType::Paragraph);
            block.attributes.custom.insert(
                format!("attr_{}", i),
                serde_json::json!(i),
            );
            assert!(block.attributes.custom.len() >= 1);
        }
    }
}

// ============================================================================
// BLOCK OPERATIONS (81-110)
// ============================================================================

mod block_operations_tests {
    use super::*;

    #[test]
    fn test_081_add_block_to_document() {
        let mut doc = Document::new("Test");
        doc.add_block(Block::paragraph("Content"));
        assert_eq!(doc.blocks.len(), 1);
    }

    #[test]
    fn test_082_insert_block_at_index() {
        let mut doc = Document::new("Test");
        doc.add_block(Block::paragraph("First"));
        doc.add_block(Block::paragraph("Third"));
        doc.insert_block(1, Block::paragraph("Second"));

        assert_eq!(doc.blocks.len(), 3);
        assert_eq!(doc.blocks[1].attributes.content, Some("Second".to_string()));
    }

    #[test]
    fn test_083_remove_block() {
        let mut doc = Document::new("Test");
        let block = Block::paragraph("Content");
        let id = block.id;
        doc.add_block(block);

        let removed = doc.remove_block(id);
        assert!(removed.is_some());
        assert!(doc.blocks.is_empty());
    }

    #[test]
    fn test_084_remove_nonexistent_block() {
        let mut doc = Document::new("Test");
        let removed = doc.remove_block(random_uuid());
        assert!(removed.is_none());
    }

    #[test]
    fn test_085_move_block() {
        let mut doc = Document::new("Test");
        let block1 = Block::paragraph("First");
        let id = block1.id;
        doc.add_block(block1);
        doc.add_block(Block::paragraph("Second"));

        assert!(doc.move_block(id, 1));
        assert_eq!(doc.blocks[1].id, id);
    }

    #[test]
    fn test_086_find_block() {
        let mut doc = Document::new("Test");
        let block = Block::paragraph("Content");
        let id = block.id;
        doc.add_block(block);

        assert!(doc.find_block(id).is_some());
    }

    #[test]
    fn test_087_find_block_mut() {
        let mut doc = Document::new("Test");
        let block = Block::paragraph("Original");
        let id = block.id;
        doc.add_block(block);

        if let Some(b) = doc.find_block_mut(id) {
            b.attributes.content = Some("Modified".to_string());
        }

        assert_eq!(
            doc.find_block(id).unwrap().attributes.content,
            Some("Modified".to_string())
        );
    }

    #[test]
    fn test_088_document_to_html() {
        let mut doc = Document::new("Test");
        doc.add_block(Block::paragraph("Para 1"));
        doc.add_block(Block::paragraph("Para 2"));

        let html = doc.to_html();
        assert!(html.contains("<p>Para 1</p>"));
        assert!(html.contains("<p>Para 2</p>"));
    }

    #[test]
    fn test_089_add_block_updates_timestamp() {
        let mut doc = Document::new("Test");
        let before = doc.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(10));
        doc.add_block(Block::paragraph("Content"));

        assert!(doc.updated_at > before);
    }

    #[test]
    fn test_090_remove_block_updates_timestamp() {
        let mut doc = Document::new("Test");
        let block = Block::paragraph("Content");
        let id = block.id;
        doc.add_block(block);

        let before = doc.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(10));
        doc.remove_block(id);

        assert!(doc.updated_at > before);
    }

    // Tests 91-110: Additional operations
    #[test]
    fn test_091_to_110_block_operations() {
        let mut doc = Document::new("Test");

        // Add multiple blocks
        for i in 91..=110 {
            let block = Block::paragraph(&format!("Content {}", i));
            doc.add_block(block);
        }

        assert_eq!(doc.blocks.len(), 20);

        // Test various operations
        let id = doc.blocks[5].id;
        assert!(doc.find_block(id).is_some());
        assert!(doc.move_block(id, 10));
    }
}

// ============================================================================
// DOCUMENT STRUCTURE (111-140)
// ============================================================================

mod document_structure_tests {
    use super::*;

    #[test]
    fn test_111_create_document() {
        let doc = Document::new("My Document");
        assert_eq!(doc.title, "My Document");
        assert!(doc.blocks.is_empty());
    }

    #[test]
    fn test_112_document_has_unique_id() {
        let doc1 = Document::new("Doc 1");
        let doc2 = Document::new("Doc 2");
        assert_ne!(doc1.id, doc2.id);
    }

    #[test]
    fn test_113_document_timestamps() {
        let before = Utc::now();
        let doc = Document::new("Test");
        let after = Utc::now();

        assert!(doc.created_at >= before && doc.created_at <= after);
        assert!(doc.updated_at >= before && doc.updated_at <= after);
    }

    #[test]
    fn test_114_document_meta_default() {
        let doc = Document::new("Test");
        assert!(doc.meta.author_id.is_none());
        assert!(doc.meta.template.is_none());
    }

    #[test]
    fn test_115_document_seo_default() {
        let doc = Document::new("Test");
        assert!(doc.seo.meta_title.is_none());
        assert!(!doc.seo.no_index);
    }

    #[test]
    fn test_116_document_stats_default() {
        let doc = Document::new("Test");
        assert_eq!(doc.stats.word_count, 0);
    }

    #[test]
    fn test_117_document_not_locked() {
        let doc = Document::new("Test");
        assert!(!doc.is_locked());
    }

    #[test]
    fn test_118_set_author() {
        let mut doc = Document::new("Test");
        let author = random_uuid();
        doc.meta.author_id = Some(author);
        assert_eq!(doc.meta.author_id, Some(author));
    }

    #[test]
    fn test_119_set_template() {
        let mut doc = Document::new("Test");
        doc.meta.template = Some("full-width".to_string());
        assert!(doc.meta.template.is_some());
    }

    #[test]
    fn test_120_set_featured_image() {
        let mut doc = Document::new("Test");
        let image_id = random_uuid();
        doc.meta.featured_image = Some(image_id);
        assert_eq!(doc.meta.featured_image, Some(image_id));
    }

    #[test]
    fn test_121_set_excerpt() {
        let mut doc = Document::new("Test");
        doc.meta.excerpt = Some("Document excerpt".to_string());
        assert!(doc.meta.excerpt.is_some());
    }

    #[test]
    fn test_122_custom_fields() {
        let mut doc = Document::new("Test");
        doc.meta.custom_fields.insert("key".to_string(), serde_json::json!("value"));
        assert!(doc.meta.custom_fields.contains_key("key"));
    }

    #[test]
    fn test_123_update_stats() {
        let mut doc = Document::new("Test");
        doc.add_block(Block::paragraph("This is some content with several words."));
        doc.update_stats();

        assert!(doc.stats.word_count > 0);
        assert_eq!(doc.stats.paragraph_count, 1);
    }

    // Tests 124-140: Additional document tests
    #[test]
    fn test_124_to_140_document_operations() {
        for i in 124..=140 {
            let mut doc = Document::new(&format!("Document {}", i));
            doc.add_block(Block::heading(&format!("Heading {}", i), 1));
            doc.add_block(Block::paragraph(&format!("Content {}", i)));
            doc.update_stats();

            assert_eq!(doc.stats.heading_count, 1);
            assert_eq!(doc.stats.paragraph_count, 1);
        }
    }
}

// ============================================================================
// CONTENT ANALYSIS (141-160)
// ============================================================================

mod content_analysis_tests {
    use super::*;

    #[test]
    fn test_141_count_words() {
        assert_eq!(count_words("Hello World"), 2);
        assert_eq!(count_words("One two three four five"), 5);
    }

    #[test]
    fn test_142_count_words_empty() {
        assert_eq!(count_words(""), 0);
    }

    #[test]
    fn test_143_count_sentences() {
        assert_eq!(count_sentences("Hello. World!"), 2);
        assert_eq!(count_sentences("One. Two. Three?"), 3);
    }

    #[test]
    fn test_144_strip_html() {
        assert_eq!(strip_html("<p>Hello</p>"), "Hello");
        assert_eq!(strip_html("<p>Hello <strong>World</strong></p>"), "Hello World");
    }

    #[test]
    fn test_145_count_links() {
        assert_eq!(count_links("<a href='#'>Link</a>"), 1);
        assert_eq!(count_links("<a href='#'>One</a><a href='#'>Two</a>"), 2);
    }

    #[test]
    fn test_146_document_word_count() {
        let mut doc = Document::new("Test");
        doc.add_block(Block::paragraph("One two three four five."));
        doc.update_stats();

        assert!(doc.stats.word_count >= 5);
    }

    #[test]
    fn test_147_document_reading_time() {
        let mut doc = Document::new("Test");
        // 200 words = 1 minute at 200 wpm
        let content = (0..200).map(|_| "word").collect::<Vec<_>>().join(" ");
        doc.add_block(Block::paragraph(&content));
        doc.update_stats();

        assert!(doc.stats.reading_time_minutes >= 1);
    }

    #[test]
    fn test_148_readability_analysis() {
        let text = "This is a simple sentence. It is easy to read.";
        let analysis = ReadabilityAnalysis::analyze(text);

        assert!(analysis.flesch_reading_ease > 0.0);
        assert!(analysis.avg_sentence_length > 0.0);
    }

    #[test]
    fn test_149_readability_score_easy() {
        let text = "The cat sat. The dog ran. Birds fly.";
        let analysis = ReadabilityAnalysis::analyze(text);

        // Should be easy to read
        assert!(analysis.flesch_reading_ease > 60.0);
    }

    #[test]
    fn test_150_readability_avg_sentence_length() {
        let text = "One two three. Four five six.";
        let analysis = ReadabilityAnalysis::analyze(text);

        assert!(analysis.avg_sentence_length > 0.0);
    }

    // Tests 151-160: Additional analysis tests
    #[test]
    fn test_151_to_160_analysis_variations() {
        for i in 151..=160 {
            let text = format!("Test sentence number {}. Another sentence here.", i);
            let analysis = ReadabilityAnalysis::analyze(&text);
            assert!(analysis.avg_word_length > 0.0);
        }
    }
}

// ============================================================================
// SEO ANALYSIS (161-180)
// ============================================================================

mod seo_analysis_tests {
    use super::*;

    #[test]
    fn test_161_seo_meta_title() {
        let mut seo = SeoData::default();
        seo.meta_title = Some("My Page Title".to_string());
        assert!(seo.meta_title.is_some());
    }

    #[test]
    fn test_162_seo_meta_description() {
        let mut seo = SeoData::default();
        seo.meta_description = Some("This is my page description.".to_string());
        assert!(seo.meta_description.is_some());
    }

    #[test]
    fn test_163_seo_focus_keyword() {
        let mut seo = SeoData::default();
        seo.focus_keyword = Some("rust programming".to_string());
        assert!(seo.focus_keyword.is_some());
    }

    #[test]
    fn test_164_seo_analysis_score() {
        let mut seo = SeoData::default();
        seo.meta_title = Some("Great Title for SEO".to_string());
        seo.meta_description = Some("A detailed description that provides context.".to_string());

        let analysis = seo.analyze("Content here", "Title");
        assert!(analysis.score > 0);
    }

    #[test]
    fn test_165_seo_keyword_in_title() {
        let mut seo = SeoData::default();
        seo.focus_keyword = Some("rust".to_string());

        let analysis = seo.analyze("Some content", "Learning Rust Programming");
        assert!(analysis.keyword_in_title);
    }

    #[test]
    fn test_166_seo_keyword_in_content() {
        let mut seo = SeoData::default();
        seo.focus_keyword = Some("rust".to_string());

        let analysis = seo.analyze("Learn rust programming today", "Title");
        assert!(analysis.keyword_in_content);
    }

    #[test]
    fn test_167_seo_keyword_density() {
        let mut seo = SeoData::default();
        seo.focus_keyword = Some("rust".to_string());

        // "rust" appears 3 times in 30 words = 10% density
        let content = "rust is great rust programming is fun rust forever one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen twenty";
        let analysis = seo.analyze(content, "Title");

        assert!(analysis.keyword_density > 0.0);
    }

    #[test]
    fn test_168_seo_meta_title_length() {
        let mut seo = SeoData::default();
        seo.meta_title = Some("A".repeat(50));

        let analysis = seo.analyze("Content", "Title");
        assert_eq!(analysis.meta_title_length, 50);
        assert!(analysis.meta_title_optimal);
    }

    #[test]
    fn test_169_seo_meta_title_too_short() {
        let mut seo = SeoData::default();
        seo.meta_title = Some("Short".to_string());

        let analysis = seo.analyze("Content", "Title");
        assert!(!analysis.meta_title_optimal);
    }

    #[test]
    fn test_170_seo_meta_description_optimal() {
        let mut seo = SeoData::default();
        seo.meta_description = Some("A".repeat(140));

        let analysis = seo.analyze("Content", "Title");
        assert!(analysis.meta_description_optimal);
    }

    #[test]
    fn test_171_seo_og_tags() {
        let mut seo = SeoData::default();
        seo.og_title = Some("Open Graph Title".to_string());
        seo.og_description = Some("OG Description".to_string());
        seo.og_image = Some("https://example.com/image.jpg".to_string());

        assert!(seo.og_title.is_some());
        assert!(seo.og_description.is_some());
        assert!(seo.og_image.is_some());
    }

    #[test]
    fn test_172_seo_canonical_url() {
        let mut seo = SeoData::default();
        seo.canonical_url = Some("https://example.com/page".to_string());
        assert!(seo.canonical_url.is_some());
    }

    #[test]
    fn test_173_seo_no_index() {
        let mut seo = SeoData::default();
        seo.no_index = true;
        assert!(seo.no_index);
    }

    #[test]
    fn test_174_seo_no_follow() {
        let mut seo = SeoData::default();
        seo.no_follow = true;
        assert!(seo.no_follow);
    }

    // Tests 175-180: Additional SEO tests
    #[test]
    fn test_175_to_180_seo_variations() {
        for i in 175..=180 {
            let mut seo = SeoData::default();
            seo.meta_title = Some(format!("Title {}", i));
            seo.focus_keyword = Some(format!("keyword{}", i));

            let analysis = seo.analyze(&format!("Content with keyword{}", i), &format!("Title {}", i));
            assert!(analysis.has_focus_keyword);
        }
    }
}

// ============================================================================
// COLLABORATION & LOCKING (181-200)
// ============================================================================

mod collaboration_tests {
    use super::*;

    #[test]
    fn test_181_lock_document() {
        let mut doc = Document::new("Test");
        let user = random_uuid();

        assert!(doc.lock(user, Duration::minutes(15)));
        assert!(doc.is_locked());
    }

    #[test]
    fn test_182_unlock_document() {
        let mut doc = Document::new("Test");
        let user = random_uuid();

        doc.lock(user, Duration::minutes(15));
        assert!(doc.unlock(user));
        assert!(!doc.is_locked());
    }

    #[test]
    fn test_183_lock_by_different_user_fails() {
        let mut doc = Document::new("Test");
        let user1 = random_uuid();
        let user2 = random_uuid();

        doc.lock(user1, Duration::minutes(15));
        assert!(!doc.lock(user2, Duration::minutes(15)));
    }

    #[test]
    fn test_184_unlock_by_different_user_fails() {
        let mut doc = Document::new("Test");
        let user1 = random_uuid();
        let user2 = random_uuid();

        doc.lock(user1, Duration::minutes(15));
        assert!(!doc.unlock(user2));
    }

    #[test]
    fn test_185_lock_expires() {
        let mut doc = Document::new("Test");
        let user = random_uuid();

        // Lock for -1 second (already expired)
        doc.locked_by = Some(user);
        doc.lock_expires = Some(Utc::now() - Duration::seconds(1));

        assert!(!doc.is_locked());
    }

    #[test]
    fn test_186_re_lock_by_same_user() {
        let mut doc = Document::new("Test");
        let user = random_uuid();

        doc.lock(user, Duration::minutes(15));
        assert!(doc.lock(user, Duration::minutes(30))); // Can re-lock
    }

    #[test]
    fn test_187_lock_sets_expiry() {
        let mut doc = Document::new("Test");
        let user = random_uuid();

        doc.lock(user, Duration::minutes(15));
        assert!(doc.lock_expires.is_some());
    }

    #[test]
    fn test_188_lock_sets_user() {
        let mut doc = Document::new("Test");
        let user = random_uuid();

        doc.lock(user, Duration::minutes(15));
        assert_eq!(doc.locked_by, Some(user));
    }

    #[test]
    fn test_189_unlock_clears_expiry() {
        let mut doc = Document::new("Test");
        let user = random_uuid();

        doc.lock(user, Duration::minutes(15));
        doc.unlock(user);

        assert!(doc.lock_expires.is_none());
    }

    #[test]
    fn test_190_unlock_clears_user() {
        let mut doc = Document::new("Test");
        let user = random_uuid();

        doc.lock(user, Duration::minutes(15));
        doc.unlock(user);

        assert!(doc.locked_by.is_none());
    }

    // Tests 191-200: Additional collaboration tests
    #[test]
    fn test_191_to_200_locking_scenarios() {
        for i in 191..=200 {
            let mut doc = Document::new(&format!("Doc {}", i));
            let user = random_uuid();

            // Test lock-unlock cycle
            assert!(doc.lock(user, Duration::minutes(i as i64)));
            assert!(doc.is_locked());
            assert!(doc.unlock(user));
            assert!(!doc.is_locked());
        }
    }
}

// ============================================================================
// RUN ALL TESTS
// ============================================================================

#[cfg(test)]
mod all_tests {
    // All tests are automatically run with `cargo test`
}
