//! Comprehensive Tests for RustPress Editor (60+ Tests)
//!
//! This module contains thorough test coverage for:
//! - Block System (BlockType variants, BlockCategory, BlockAttributes)
//! - Block Operations (styles, visibility, alignment, ID generation)
//! - PostDocument (creation, serialization, content)
//! - PostMetadata and Terms
//! - PostSeo validation
//! - PostStats calculations
//! - PostRevision system
//! - PostPublishing workflow
//! - FeaturedMedia handling
//! - Content analysis and readability
//! - Collaboration sessions
//! - Block transformations
//! - Nested blocks and ordering

use rustpress_editor::blocks::{
    Block, BlockType, BlockCategory, BlockAttributes,
    BlockTransformer, BlockSerializer, ListType, SeparatorStyle, ButtonStyle,
    AlertType, TextAlign, Spacing,
};
use rustpress_editor::post::{
    PostDocument, PostContent, PostFormat, PublishStatus,
    FeaturedMedia, Author, Term, CommentStatus, RevisionType, HeadingCounts,
};
use rustpress_editor::collaboration::{
    CollaborationSession, CollaboratorInfo, CollaboratorRole, CursorInfo,
    Operation, OperationType, get_collaborator_color, CURSOR_COLORS,
};
use chrono::{Utc, Duration};
use std::collections::HashSet;

// ============================================================================
// BLOCK TYPE VARIANT TESTS (1-10)
// ============================================================================

#[test]
fn test_001_block_type_paragraph() {
    let block = Block::new(BlockType::Paragraph);
    assert_eq!(block.block_type, BlockType::Paragraph);
    assert!(block.attributes.content.is_none());
}

#[test]
fn test_002_block_type_heading_default_level() {
    let block = Block::new(BlockType::Heading);
    assert_eq!(block.block_type, BlockType::Heading);
    assert_eq!(block.attributes.level, Some(2)); // Default heading level
}

#[test]
fn test_003_block_type_list_default_type() {
    let block = Block::new(BlockType::List);
    assert_eq!(block.block_type, BlockType::List);
    assert_eq!(block.attributes.list_type, Some(ListType::Unordered));
}

#[test]
fn test_004_block_type_code_default_language() {
    let block = Block::new(BlockType::Code);
    assert_eq!(block.block_type, BlockType::Code);
    assert_eq!(block.attributes.language, Some("plaintext".to_string()));
    assert_eq!(block.attributes.show_line_numbers, Some(true));
}

#[test]
fn test_005_block_type_image() {
    let mut block = Block::new(BlockType::Image);
    block.attributes.url = Some("https://example.com/image.jpg".to_string());
    block.attributes.alt = Some("Test image".to_string());
    assert!(block.attributes.url.is_some());
    assert!(block.attributes.alt.is_some());
}

#[test]
fn test_006_block_type_video() {
    let block = Block::new(BlockType::Video);
    assert_eq!(block.block_type, BlockType::Video);
    assert_eq!(block.attributes.controls, Some(true)); // Default controls
}

#[test]
fn test_007_block_type_button() {
    let block = Block::new(BlockType::Button);
    assert_eq!(block.block_type, BlockType::Button);
    assert_eq!(block.attributes.button_style, Some(ButtonStyle::Primary));
}

#[test]
fn test_008_block_type_spacer() {
    let block = Block::new(BlockType::Spacer);
    assert_eq!(block.block_type, BlockType::Spacer);
    assert_eq!(block.attributes.spacer_height, Some("50px".to_string()));
}

#[test]
fn test_009_block_type_separator() {
    let block = Block::new(BlockType::Separator);
    assert_eq!(block.block_type, BlockType::Separator);
    assert_eq!(block.attributes.separator_style, Some(SeparatorStyle::Solid));
}

#[test]
fn test_010_block_type_custom() {
    let block = Block::new(BlockType::Custom(42));
    assert!(matches!(block.block_type, BlockType::Custom(42)));
}

// ============================================================================
// BLOCK CATEGORY TESTS (11-18)
// ============================================================================

#[test]
fn test_011_category_text_blocks() {
    assert_eq!(BlockType::Paragraph.category(), BlockCategory::Text);
    assert_eq!(BlockType::Heading.category(), BlockCategory::Text);
    assert_eq!(BlockType::List.category(), BlockCategory::Text);
    assert_eq!(BlockType::Quote.category(), BlockCategory::Text);
    assert_eq!(BlockType::Code.category(), BlockCategory::Text);
}

#[test]
fn test_012_category_media_blocks() {
    assert_eq!(BlockType::Image.category(), BlockCategory::Media);
    assert_eq!(BlockType::Gallery.category(), BlockCategory::Media);
    assert_eq!(BlockType::Video.category(), BlockCategory::Media);
    assert_eq!(BlockType::Audio.category(), BlockCategory::Media);
    assert_eq!(BlockType::File.category(), BlockCategory::Media);
}

#[test]
fn test_013_category_layout_blocks() {
    assert_eq!(BlockType::Group.category(), BlockCategory::Layout);
    assert_eq!(BlockType::Columns.category(), BlockCategory::Layout);
    assert_eq!(BlockType::Section.category(), BlockCategory::Layout);
    assert_eq!(BlockType::Spacer.category(), BlockCategory::Layout);
    assert_eq!(BlockType::Separator.category(), BlockCategory::Layout);
}

#[test]
fn test_014_category_design_blocks() {
    assert_eq!(BlockType::Button.category(), BlockCategory::Design);
    assert_eq!(BlockType::Buttons.category(), BlockCategory::Design);
    assert_eq!(BlockType::Icon.category(), BlockCategory::Design);
    assert_eq!(BlockType::SocialLinks.category(), BlockCategory::Design);
}

#[test]
fn test_015_category_interactive_blocks() {
    assert_eq!(BlockType::Accordion.category(), BlockCategory::Interactive);
    assert_eq!(BlockType::Tabs.category(), BlockCategory::Interactive);
    assert_eq!(BlockType::Form.category(), BlockCategory::Interactive);
    assert_eq!(BlockType::FormInput.category(), BlockCategory::Interactive);
}

#[test]
fn test_016_category_widget_blocks() {
    assert_eq!(BlockType::Shortcode.category(), BlockCategory::Widgets);
    assert_eq!(BlockType::Html.category(), BlockCategory::Widgets);
    assert_eq!(BlockType::QueryLoop.category(), BlockCategory::Widgets);
}

#[test]
fn test_017_block_type_display_name() {
    assert_eq!(BlockType::Paragraph.display_name(), "Paragraph");
    assert_eq!(BlockType::Heading.display_name(), "Heading");
    assert_eq!(BlockType::TableOfContents.display_name(), "Table of Contents");
    assert_eq!(BlockType::MediaText.display_name(), "Media & Text");
}

#[test]
fn test_018_block_is_container() {
    let group = Block::new(BlockType::Group);
    let columns = Block::new(BlockType::Columns);
    let paragraph = Block::new(BlockType::Paragraph);

    assert!(group.is_container());
    assert!(columns.is_container());
    assert!(!paragraph.is_container());
}

// ============================================================================
// BLOCK ATTRIBUTES AND DEFAULTS TESTS (19-25)
// ============================================================================

#[test]
fn test_019_block_attributes_default() {
    let attrs = BlockAttributes::default_for_type(&BlockType::Paragraph);
    assert!(attrs.content.is_none());
    assert!(attrs.level.is_none());
}

#[test]
fn test_020_block_attributes_heading_level() {
    let attrs = BlockAttributes::default_for_type(&BlockType::Heading);
    assert_eq!(attrs.level, Some(2));
}

#[test]
fn test_021_block_attributes_counter() {
    let attrs = BlockAttributes::default_for_type(&BlockType::Counter);
    assert_eq!(attrs.counter_start, Some(0));
    assert_eq!(attrs.counter_end, Some(100));
}

#[test]
fn test_022_block_attributes_progress_bar() {
    let attrs = BlockAttributes::default_for_type(&BlockType::ProgressBar);
    assert_eq!(attrs.progress, Some(50));
}

#[test]
fn test_023_block_attributes_rating() {
    let attrs = BlockAttributes::default_for_type(&BlockType::Rating);
    assert_eq!(attrs.rating_max, Some(5));
}

#[test]
fn test_024_block_attributes_toc() {
    let attrs = BlockAttributes::default_for_type(&BlockType::TableOfContents);
    assert_eq!(attrs.toc_max_depth, Some(3));
    assert_eq!(attrs.toc_numbered, Some(true));
}

#[test]
fn test_025_block_attributes_get_text_content() {
    let mut block = Block::new(BlockType::Paragraph);
    block.attributes.content = Some("Test content".to_string());
    assert_eq!(block.get_text_content(), "Test content");
}

// ============================================================================
// BLOCK STYLES TESTS (26-32)
// ============================================================================

#[test]
fn test_026_block_styles_background_color() {
    let mut block = Block::new(BlockType::Paragraph);
    block.styles.background_color = Some("#ff0000".to_string());
    assert_eq!(block.styles.background_color, Some("#ff0000".to_string()));
}

#[test]
fn test_027_block_styles_text_color() {
    let mut block = Block::new(BlockType::Paragraph);
    block.styles.text_color = Some("#000000".to_string());
    assert_eq!(block.styles.text_color, Some("#000000".to_string()));
}

#[test]
fn test_028_block_styles_padding_all() {
    let mut block = Block::new(BlockType::Group);
    block.styles.padding = Some(Spacing::All("20px".to_string()));
    assert!(matches!(block.styles.padding, Some(Spacing::All(_))));
}

#[test]
fn test_029_block_styles_padding_individual() {
    let mut block = Block::new(BlockType::Group);
    block.styles.padding = Some(Spacing::Individual {
        top: Some("10px".to_string()),
        right: Some("20px".to_string()),
        bottom: Some("10px".to_string()),
        left: Some("20px".to_string()),
    });
    assert!(matches!(block.styles.padding, Some(Spacing::Individual { .. })));
}

#[test]
fn test_030_block_styles_border() {
    let mut block = Block::new(BlockType::Button);
    block.styles.border_width = Some("1px".to_string());
    block.styles.border_color = Some("#333".to_string());
    block.styles.border_style = Some("solid".to_string());
    block.styles.border_radius = Some("4px".to_string());

    assert!(block.styles.border_width.is_some());
    assert!(block.styles.border_color.is_some());
    assert!(block.styles.border_style.is_some());
    assert!(block.styles.border_radius.is_some());
}

#[test]
fn test_031_block_styles_typography() {
    let mut block = Block::new(BlockType::Paragraph);
    block.styles.font_size = Some("16px".to_string());
    block.styles.font_family = Some("Arial, sans-serif".to_string());
    block.styles.font_weight = Some("bold".to_string());
    block.styles.line_height = Some("1.5".to_string());

    assert_eq!(block.styles.font_size, Some("16px".to_string()));
    assert!(block.styles.font_family.is_some());
}

#[test]
fn test_032_block_styles_box_shadow() {
    let mut block = Block::new(BlockType::Group);
    block.styles.box_shadow = Some("0 4px 6px rgba(0,0,0,0.1)".to_string());
    assert!(block.styles.box_shadow.is_some());
}

// ============================================================================
// BLOCK VISIBILITY TESTS (33-37)
// ============================================================================

#[test]
fn test_033_block_visibility_default() {
    let block = Block::new(BlockType::Paragraph);
    assert!(block.visibility.desktop);
    assert!(block.visibility.tablet);
    assert!(block.visibility.mobile);
    assert!(!block.visibility.logged_in_only);
}

#[test]
fn test_034_block_visibility_desktop_only() {
    let mut block = Block::new(BlockType::Paragraph);
    block.visibility.tablet = false;
    block.visibility.mobile = false;

    assert!(block.visibility.desktop);
    assert!(!block.visibility.tablet);
    assert!(!block.visibility.mobile);
}

#[test]
fn test_035_block_visibility_logged_in() {
    let mut block = Block::new(BlockType::Paragraph);
    block.visibility.logged_in_only = true;
    assert!(block.visibility.logged_in_only);
}

#[test]
fn test_036_block_visibility_roles() {
    let mut block = Block::new(BlockType::Paragraph);
    block.visibility.roles = vec!["admin".to_string(), "editor".to_string()];
    assert_eq!(block.visibility.roles.len(), 2);
}

#[test]
fn test_037_block_visibility_schedule() {
    use rustpress_editor::blocks::VisibilitySchedule;

    let mut block = Block::new(BlockType::Paragraph);
    block.visibility.schedule = Some(VisibilitySchedule {
        start: Some(Utc::now()),
        end: Some(Utc::now() + Duration::days(7)),
    });
    assert!(block.visibility.schedule.is_some());
}

// ============================================================================
// TEXT ALIGN TESTS (38-40)
// ============================================================================

#[test]
fn test_038_text_align_values() {
    let mut block = Block::new(BlockType::Paragraph);

    block.attributes.align = Some(TextAlign::Left);
    assert_eq!(block.attributes.align, Some(TextAlign::Left));

    block.attributes.align = Some(TextAlign::Center);
    assert_eq!(block.attributes.align, Some(TextAlign::Center));

    block.attributes.align = Some(TextAlign::Right);
    assert_eq!(block.attributes.align, Some(TextAlign::Right));
}

#[test]
fn test_039_text_align_justify() {
    let mut block = Block::new(BlockType::Paragraph);
    block.attributes.align = Some(TextAlign::Justify);
    assert_eq!(block.attributes.align, Some(TextAlign::Justify));
}

#[test]
fn test_040_text_align_default_none() {
    let block = Block::new(BlockType::Paragraph);
    assert!(block.attributes.align.is_none());
}

// ============================================================================
// BLOCK ID GENERATION TESTS (41-43)
// ============================================================================

#[test]
fn test_041_block_id_unique() {
    let block1 = Block::new(BlockType::Paragraph);
    let block2 = Block::new(BlockType::Paragraph);
    assert_ne!(block1.id, block2.id);
}

#[test]
fn test_042_block_id_uniqueness_many() {
    let mut ids = HashSet::new();
    for _ in 0..100 {
        let block = Block::new(BlockType::Paragraph);
        ids.insert(block.id);
    }
    assert_eq!(ids.len(), 100);
}

#[test]
fn test_043_block_id_not_nil() {
    let block = Block::new(BlockType::Paragraph);
    assert!(!block.id.is_nil());
}

// ============================================================================
// POST DOCUMENT CREATION TESTS (44-50)
// ============================================================================

#[test]
fn test_044_post_document_new_post() {
    let post = PostDocument::new_post("My Test Post");
    assert_eq!(post.title, "My Test Post");
    assert_eq!(post.post_type, "post");
    assert_eq!(post.slug, "my-test-post");
    assert!(post.is_draft());
}

#[test]
fn test_045_post_document_new_page() {
    let page = PostDocument::new_page("About Us");
    assert_eq!(page.title, "About Us");
    assert_eq!(page.post_type, "page");
    assert_eq!(page.comment_status, CommentStatus::Closed);
}

#[test]
fn test_046_post_document_unique_uuid() {
    let post1 = PostDocument::new_post("Post 1");
    let post2 = PostDocument::new_post("Post 2");
    assert_ne!(post1.uuid, post2.uuid);
}

#[test]
fn test_047_post_document_default_format() {
    let post = PostDocument::new_post("Test");
    assert_eq!(post.format, PostFormat::Standard);
}

#[test]
fn test_048_post_document_timestamps() {
    let before = Utc::now();
    let post = PostDocument::new_post("Test");
    let after = Utc::now();

    assert!(post.created_at >= before && post.created_at <= after);
    assert!(post.modified_at >= before && post.modified_at <= after);
}

#[test]
fn test_049_post_document_version_starts_at_one() {
    let post = PostDocument::new_post("Test");
    assert_eq!(post.version, 1);
}

#[test]
fn test_050_post_document_add_block() {
    let mut post = PostDocument::new_post("Test");
    let mut block = Block::new(BlockType::Paragraph);
    block.attributes.content = Some("Test content".to_string());
    post.add_block(block);

    assert_eq!(post.content.blocks.len(), 1);
}

// ============================================================================
// POST DOCUMENT SERIALIZATION TESTS (51-55)
// ============================================================================

#[test]
fn test_051_post_content_to_json() {
    let mut post = PostDocument::new_post("Test");
    let mut block = Block::new(BlockType::Paragraph);
    block.attributes.content = Some("Hello world".to_string());
    post.add_block(block);

    let serializer = BlockSerializer::new();
    let json = serializer.to_json(&post.content.blocks);
    assert!(json.is_ok());
    assert!(json.unwrap().contains("paragraph"));
}

#[test]
fn test_052_post_content_from_json() {
    let json = r#"[{"id":"00000000-0000-0000-0000-000000000001","type":"paragraph","attributes":{"content":"Test"},"children":[],"styles":{},"css_classes":[],"visibility":{"desktop":true,"tablet":true,"mobile":true,"logged_in_only":false,"roles":[]},"meta":{}}]"#;

    let serializer = BlockSerializer::new();
    let result = serializer.from_json(json);
    assert!(result.is_ok());
}

#[test]
fn test_053_post_content_to_html() {
    let mut post = PostDocument::new_post("Test");
    let mut block = Block::new(BlockType::Paragraph);
    block.attributes.content = Some("Hello world".to_string());
    post.add_block(block);

    let html = post.content.get_html();
    assert!(html.contains("<p>"));
    assert!(html.contains("Hello world"));
}

#[test]
fn test_054_post_content_to_plain_text() {
    let mut post = PostDocument::new_post("Test");
    let mut block = Block::new(BlockType::Paragraph);
    block.attributes.content = Some("Hello world".to_string());
    post.add_block(block);

    let text = post.content.get_plain_text();
    assert_eq!(text, "Hello world");
}

#[test]
fn test_055_post_content_block_ids() {
    let mut post = PostDocument::new_post("Test");
    post.add_block(Block::new(BlockType::Paragraph));
    post.add_block(Block::new(BlockType::Paragraph));

    let ids = post.content.get_all_block_ids();
    assert_eq!(ids.len(), 2);
}

// ============================================================================
// POST CONTENT FORMAT TESTS (56-58)
// ============================================================================

#[test]
fn test_056_post_content_default() {
    let content = PostContent::default();
    assert!(content.blocks.is_empty());
    assert!(content.raw_html.is_none());
    assert!(content.markdown.is_none());
}

#[test]
fn test_057_post_content_count_blocks() {
    let mut content = PostContent::default();

    let mut group = Block::new(BlockType::Group);
    group.children.push(Block::new(BlockType::Paragraph));
    group.children.push(Block::new(BlockType::Paragraph));
    content.blocks.push(group);

    // Should count parent + children = 3
    assert_eq!(content.count_blocks(), 3);
}

#[test]
fn test_058_post_format_variants() {
    let formats = vec![
        PostFormat::Standard,
        PostFormat::Aside,
        PostFormat::Gallery,
        PostFormat::Link,
        PostFormat::Image,
        PostFormat::Quote,
        PostFormat::Video,
        PostFormat::Audio,
    ];

    for format in formats {
        let mut post = PostDocument::new_post("Test");
        post.format = format;
        assert!(matches!(post.format, PostFormat::Standard | PostFormat::Aside |
            PostFormat::Gallery | PostFormat::Link | PostFormat::Image |
            PostFormat::Quote | PostFormat::Video | PostFormat::Audio | _));
    }
}

// ============================================================================
// POST METADATA TESTS (59-65)
// ============================================================================

#[test]
fn test_059_post_metadata_set_author() {
    let mut post = PostDocument::new_post("Test");
    let author = Author::new(1, "John Doe", "johndoe");
    post.metadata.set_author(author);

    assert!(post.metadata.author.is_some());
    assert_eq!(post.metadata.author.as_ref().unwrap().name, "John Doe");
}

#[test]
fn test_060_post_metadata_add_category() {
    let mut post = PostDocument::new_post("Test");
    let category = Term::new(1, "Technology", "technology");
    post.metadata.add_category(category);

    assert_eq!(post.metadata.categories.len(), 1);
}

#[test]
fn test_061_post_metadata_add_duplicate_category() {
    let mut post = PostDocument::new_post("Test");
    let category1 = Term::new(1, "Technology", "technology");
    let category2 = Term::new(1, "Technology", "technology");
    post.metadata.add_category(category1);
    post.metadata.add_category(category2);

    // Should not add duplicate
    assert_eq!(post.metadata.categories.len(), 1);
}

#[test]
fn test_062_post_metadata_add_tag() {
    let mut post = PostDocument::new_post("Test");
    let tag = Term::new(1, "Rust", "rust");
    post.metadata.add_tag(tag);

    assert_eq!(post.metadata.tags.len(), 1);
}

#[test]
fn test_063_post_metadata_has_category() {
    let mut post = PostDocument::new_post("Test");
    let category = Term::new(1, "Technology", "technology");
    post.metadata.add_category(category);

    assert!(post.metadata.has_category("technology"));
    assert!(!post.metadata.has_category("science"));
}

#[test]
fn test_064_post_metadata_has_tag() {
    let mut post = PostDocument::new_post("Test");
    let tag = Term::new(1, "Rust", "rust");
    post.metadata.add_tag(tag);

    assert!(post.metadata.has_tag("rust"));
    assert!(!post.metadata.has_tag("python"));
}

#[test]
fn test_065_post_metadata_get_all_term_ids() {
    let mut post = PostDocument::new_post("Test");
    post.metadata.add_category(Term::new(1, "Tech", "tech"));
    post.metadata.add_tag(Term::new(2, "Rust", "rust"));

    let ids = post.metadata.get_all_term_ids();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&1));
    assert!(ids.contains(&2));
}

// ============================================================================
// POST SEO VALIDATION TESTS (66-72)
// ============================================================================

#[test]
fn test_066_post_seo_title_length_optimal() {
    let mut post = PostDocument::new_post("Test");
    // 55 characters - optimal length
    post.seo.meta_title = Some("This is an optimal SEO title that is around 55 chars.".to_string());

    assert!(post.seo.is_title_length_optimal());
}

#[test]
fn test_067_post_seo_title_too_short() {
    let mut post = PostDocument::new_post("Test");
    post.seo.meta_title = Some("Short".to_string());

    assert!(!post.seo.is_title_length_optimal());
}

#[test]
fn test_068_post_seo_title_too_long() {
    let mut post = PostDocument::new_post("Test");
    post.seo.meta_title = Some("A".repeat(70));

    assert!(!post.seo.is_title_length_optimal());
}

#[test]
fn test_069_post_seo_description_optimal() {
    let mut post = PostDocument::new_post("Test");
    // 140 characters - optimal
    post.seo.meta_description = Some("A".repeat(140));

    assert!(post.seo.is_description_length_optimal());
}

#[test]
fn test_070_post_seo_description_too_short() {
    let mut post = PostDocument::new_post("Test");
    post.seo.meta_description = Some("Too short".to_string());

    assert!(!post.seo.is_description_length_optimal());
}

#[test]
fn test_071_post_seo_effective_title() {
    let mut post = PostDocument::new_post("Post Title");

    // Without SEO title, should use post title
    assert_eq!(post.seo.get_effective_title(&post.title), "Post Title");

    // With SEO title, should use it
    post.seo.meta_title = Some("SEO Title".to_string());
    assert_eq!(post.seo.get_effective_title(&post.title), "SEO Title");
}

#[test]
fn test_072_post_seo_robots_meta() {
    let mut post = PostDocument::new_post("Test");
    post.seo.robots.index = false;

    let content = post.seo.robots.to_meta_content();
    assert!(content.contains("noindex"));
}

// ============================================================================
// POST STATS CALCULATION TESTS (73-80)
// ============================================================================

#[test]
fn test_073_post_stats_word_count() {
    let mut post = PostDocument::new_post("Test");
    let mut block = Block::new(BlockType::Paragraph);
    block.attributes.content = Some("One two three four five six seven eight nine ten".to_string());
    post.add_block(block);
    post.update_stats();

    assert_eq!(post.stats.word_count, 10);
}

#[test]
fn test_074_post_stats_reading_time() {
    let mut post = PostDocument::new_post("Test");
    let mut block = Block::new(BlockType::Paragraph);
    // 225 words = 1 minute at 225 wpm
    block.attributes.content = Some(vec!["word"; 225].join(" "));
    post.add_block(block);
    post.update_stats();

    assert_eq!(post.stats.reading_time_minutes, 1);
}

#[test]
fn test_075_post_stats_paragraph_count() {
    let mut post = PostDocument::new_post("Test");
    post.add_block(Block::new(BlockType::Paragraph));
    post.add_block(Block::new(BlockType::Paragraph));
    post.add_block(Block::new(BlockType::Paragraph));
    post.update_stats();

    // Block count includes all blocks
    assert_eq!(post.stats.block_count, 3);
}

#[test]
fn test_076_post_stats_heading_counts() {
    let mut post = PostDocument::new_post("Test");

    let mut h1 = Block::new(BlockType::Heading);
    h1.attributes.level = Some(1);
    post.add_block(h1);

    let mut h2 = Block::new(BlockType::Heading);
    h2.attributes.level = Some(2);
    post.add_block(h2);

    post.update_stats();

    assert_eq!(post.stats.heading_counts.h1, 1);
    assert_eq!(post.stats.heading_counts.h2, 1);
}

#[test]
fn test_077_post_stats_image_count() {
    let mut post = PostDocument::new_post("Test");
    post.add_block(Block::new(BlockType::Image));
    post.add_block(Block::new(BlockType::Image));
    post.update_stats();

    assert_eq!(post.stats.image_count, 2);
}

#[test]
fn test_078_post_stats_format_reading_time() {
    let mut post = PostDocument::new_post("Test");
    post.stats.reading_time_minutes = 5;

    assert_eq!(post.stats.format_reading_time(), "5 min read");
}

#[test]
fn test_079_post_stats_format_reading_time_one_minute() {
    let mut post = PostDocument::new_post("Test");
    post.stats.reading_time_minutes = 1;

    assert_eq!(post.stats.format_reading_time(), "1 min read");
}

#[test]
fn test_080_post_stats_format_reading_time_less_than_minute() {
    let mut post = PostDocument::new_post("Test");
    post.stats.reading_time_minutes = 0;

    assert_eq!(post.stats.format_reading_time(), "< 1 min read");
}

// ============================================================================
// POST REVISION TESTS (81-87)
// ============================================================================

#[test]
fn test_081_post_revision_creation() {
    let mut post = PostDocument::new_post("Test");
    post.add_block(Block::new(BlockType::Paragraph));
    post.create_revision(1, Some("Initial save".to_string()));

    assert_eq!(post.revisions.len(), 1);
    assert_eq!(post.version, 2);
}

#[test]
fn test_082_post_revision_multiple() {
    let mut post = PostDocument::new_post("Test");
    post.create_revision(1, None);
    post.create_revision(1, None);
    post.create_revision(1, None);

    assert_eq!(post.revisions.len(), 3);
    assert_eq!(post.version, 4);
}

#[test]
fn test_083_post_revision_stores_content() {
    let mut post = PostDocument::new_post("Test");
    let mut block = Block::new(BlockType::Paragraph);
    block.attributes.content = Some("Original".to_string());
    post.add_block(block);
    post.create_revision(1, None);

    assert_eq!(post.revisions[0].content.blocks.len(), 1);
}

#[test]
fn test_084_post_revision_restore() {
    let mut post = PostDocument::new_post("Original Title");
    post.create_revision(1, None);

    post.title = "Modified Title".to_string();
    post.create_revision(1, None);

    assert!(post.restore_revision(0));
    assert_eq!(post.title, "Original Title");
}

#[test]
fn test_085_post_revision_invalid_restore() {
    let mut post = PostDocument::new_post("Test");
    assert!(!post.restore_revision(999));
}

#[test]
fn test_086_post_revision_type() {
    use rustpress_editor::post::PostRevision;

    let revision = PostRevision::autosave(
        1,
        1,
        "Test".to_string(),
        PostContent::default(),
        None,
    );

    assert_eq!(revision.revision_type, RevisionType::Autosave);
}

#[test]
fn test_087_post_revision_changes_calculation() {
    use rustpress_editor::post::RevisionChanges;
    use rustpress_editor::post::PostRevision;

    let old = PostRevision::new(1, 1, "Old".to_string(), PostContent::default(), None, None);
    let new = PostRevision::new(2, 1, "New".to_string(), PostContent::default(), None, None);

    let changes = RevisionChanges::calculate(&old, &new);
    assert!(changes.title_changed);
}

// ============================================================================
// POST PUBLISHING STATUS TESTS (88-95)
// ============================================================================

#[test]
fn test_088_post_publishing_default_draft() {
    let post = PostDocument::new_post("Test");
    assert!(post.is_draft());
    assert!(!post.is_published());
}

#[test]
fn test_089_post_publishing_publish() {
    let mut post = PostDocument::new_post("Test");
    post.publishing.publish();

    assert!(post.is_published());
    assert!(post.publishing.published_at.is_some());
}

#[test]
fn test_090_post_publishing_schedule() {
    let mut post = PostDocument::new_post("Test");
    let future = Utc::now() + Duration::days(7);
    post.publishing.schedule(future);

    assert!(post.is_scheduled());
    assert_eq!(post.publishing.scheduled_at, Some(future));
}

#[test]
fn test_091_post_publishing_unpublish() {
    let mut post = PostDocument::new_post("Test");
    post.publishing.publish();
    post.publishing.unpublish();

    assert!(post.is_draft());
}

#[test]
fn test_092_post_publishing_trash() {
    let mut post = PostDocument::new_post("Test");
    post.publishing.trash();

    assert_eq!(post.publishing.status, PublishStatus::Trash);
}

#[test]
fn test_093_post_publishing_restore() {
    let mut post = PostDocument::new_post("Test");
    post.publishing.trash();
    post.publishing.restore();

    assert!(post.is_draft());
}

#[test]
fn test_094_post_publishing_submit_for_review() {
    let mut post = PostDocument::new_post("Test");
    post.publishing.submit_for_review();

    assert_eq!(post.publishing.status, PublishStatus::Pending);
    assert!(post.publishing.review.is_some());
}

#[test]
fn test_095_post_publishing_history() {
    let mut post = PostDocument::new_post("Test");
    post.publishing.publish();
    post.publishing.unpublish();

    assert!(post.publishing.history.len() >= 2);
}

// ============================================================================
// FEATURED MEDIA TESTS (96-100)
// ============================================================================

#[test]
fn test_096_featured_media_from_url() {
    let media = FeaturedMedia::from_url(1, "https://example.com/image.jpg".to_string());

    assert_eq!(media.id, 1);
    assert_eq!(media.url, "https://example.com/image.jpg");
    assert_eq!(media.media_type, rustpress_editor::post::MediaType::Image);
}

#[test]
fn test_097_featured_media_alt_caption() {
    let mut media = FeaturedMedia::from_url(1, "https://example.com/image.jpg".to_string());
    media.alt = Some("Alt text".to_string());
    media.caption = Some("Image caption".to_string());

    assert_eq!(media.alt, Some("Alt text".to_string()));
    assert_eq!(media.caption, Some("Image caption".to_string()));
}

#[test]
fn test_098_featured_media_sizes() {
    let mut media = FeaturedMedia::from_url(1, "https://example.com/image.jpg".to_string());
    media.sizes.thumbnail = Some("https://example.com/image-150.jpg".to_string());
    media.sizes.medium = Some("https://example.com/image-300.jpg".to_string());

    assert!(media.sizes.thumbnail.is_some());
}

#[test]
fn test_099_featured_media_get_url_for_size() {
    use rustpress_editor::post::ImageSize;

    let mut media = FeaturedMedia::from_url(1, "https://example.com/image.jpg".to_string());
    media.sizes.thumbnail = Some("https://example.com/thumb.jpg".to_string());

    assert_eq!(media.get_url_for_size(ImageSize::Thumbnail), "https://example.com/thumb.jpg");
    assert_eq!(media.get_url_for_size(ImageSize::Full), "https://example.com/image.jpg");
}

#[test]
fn test_100_featured_media_focal_point() {
    use rustpress_editor::post::FocalPoint;

    let mut media = FeaturedMedia::from_url(1, "https://example.com/image.jpg".to_string());
    media.focal_point = Some(FocalPoint::new(0.3, 0.7));

    let focal = media.focal_point.unwrap();
    assert!((focal.x - 0.3).abs() < 0.001);
    assert!((focal.y - 0.7).abs() < 0.001);
}

// ============================================================================
// AUTHOR INFORMATION TESTS (101-103)
// ============================================================================

#[test]
fn test_101_author_creation() {
    let author = Author::new(1, "John Doe", "johndoe");

    assert_eq!(author.id, 1);
    assert_eq!(author.name, "John Doe");
    assert_eq!(author.username, "johndoe");
}

#[test]
fn test_102_author_optional_fields() {
    let mut author = Author::new(1, "John Doe", "johndoe");
    author.email = Some("john@example.com".to_string());
    author.bio = Some("A software developer".to_string());
    author.url = Some("https://johndoe.com".to_string());

    assert!(author.email.is_some());
    assert!(author.bio.is_some());
}

#[test]
fn test_103_author_get_avatar() {
    let author = Author::new(1, "John Doe", "johndoe");
    let avatar = author.get_avatar(64);

    assert!(avatar.contains("gravatar.com"));
}

// ============================================================================
// COLLABORATION SESSION TESTS (104-110)
// ============================================================================

#[test]
fn test_104_collaboration_session_new() {
    let session = CollaborationSession::new(1);

    assert_eq!(session.post_id, 1);
    assert!(session.users.is_empty());
    assert!(session.is_empty());
}

#[test]
fn test_105_collaboration_session_join() {
    let mut session = CollaborationSession::new(1);
    let user = CollaboratorInfo::new(1, "John".to_string(), "#ff0000".to_string());
    session.join(user);

    assert_eq!(session.active_user_count(), 1);
    assert!(!session.is_empty());
}

#[test]
fn test_106_collaboration_session_leave() {
    let mut session = CollaborationSession::new(1);
    let user = CollaboratorInfo::new(1, "John".to_string(), "#ff0000".to_string());
    session.join(user);
    session.leave(1);

    assert!(session.is_empty());
}

#[test]
fn test_107_collaboration_cursor_update() {
    let mut session = CollaborationSession::new(1);
    let user = CollaboratorInfo::new(1, "John".to_string(), "#ff0000".to_string());
    session.join(user);

    let cursor = CursorInfo {
        block_id: uuid::Uuid::new_v4(),
        offset: 10,
        selection_start: None,
        selection_end: None,
    };
    session.update_cursor(1, cursor);

    assert!(session.users.get(&1).unwrap().cursor.is_some());
}

#[test]
fn test_108_collaborator_colors() {
    assert_eq!(CURSOR_COLORS.len(), 10);
    assert_eq!(get_collaborator_color(0), "#ef4444");
    assert_eq!(get_collaborator_color(10), "#ef4444"); // Wraps around
}

#[test]
fn test_109_collaborator_roles() {
    let mut user = CollaboratorInfo::new(1, "John".to_string(), "#ff0000".to_string());
    assert_eq!(user.role, CollaboratorRole::Editor);

    user.role = CollaboratorRole::Viewer;
    assert_eq!(user.role, CollaboratorRole::Viewer);
}

#[test]
fn test_110_collaboration_settings_default() {
    let session = CollaborationSession::new(1);

    assert!(!session.settings.allow_anonymous);
    assert!(session.settings.show_cursors);
    assert_eq!(session.settings.max_collaborators, 10);
}

// ============================================================================
// BLOCK TRANSFORM TESTS (111-118)
// ============================================================================

#[test]
fn test_111_transform_paragraph_to_heading() {
    let transformer = BlockTransformer::new();
    let mut para = Block::new(BlockType::Paragraph);
    para.attributes.content = Some("Test heading".to_string());

    let result = transformer.transform(&para, BlockType::Heading);
    assert!(result.is_some());
    let heading = result.unwrap();
    assert_eq!(heading.block_type, BlockType::Heading);
    assert_eq!(heading.attributes.content, Some("Test heading".to_string()));
}

#[test]
fn test_112_transform_heading_to_paragraph() {
    let transformer = BlockTransformer::new();
    let mut heading = Block::new(BlockType::Heading);
    heading.attributes.content = Some("Test paragraph".to_string());
    heading.attributes.level = Some(2);

    let result = transformer.transform(&heading, BlockType::Paragraph);
    assert!(result.is_some());
    let para = result.unwrap();
    assert_eq!(para.block_type, BlockType::Paragraph);
    assert!(para.attributes.level.is_none());
}

#[test]
fn test_113_transform_paragraph_to_list() {
    let transformer = BlockTransformer::new();
    let mut para = Block::new(BlockType::Paragraph);
    para.attributes.content = Some("Item 1\nItem 2\nItem 3".to_string());

    let result = transformer.transform(&para, BlockType::List);
    assert!(result.is_some());
    let list = result.unwrap();
    assert_eq!(list.children.len(), 3);
}

#[test]
fn test_114_transform_image_to_gallery() {
    let transformer = BlockTransformer::new();
    let mut image = Block::new(BlockType::Image);
    image.attributes.url = Some("https://example.com/image.jpg".to_string());

    let result = transformer.transform(&image, BlockType::Gallery);
    assert!(result.is_some());
    let gallery = result.unwrap();
    assert_eq!(gallery.attributes.images.len(), 1);
}

#[test]
fn test_115_transform_invalid() {
    let transformer = BlockTransformer::new();
    let para = Block::new(BlockType::Paragraph);

    // Paragraph can't directly transform to Image
    let result = transformer.transform(&para, BlockType::Image);
    assert!(result.is_none());
}

#[test]
fn test_116_transform_get_valid_transforms() {
    let transformer = BlockTransformer::new();
    let valid = transformer.get_valid_transforms(&BlockType::Paragraph);

    assert!(valid.contains(&BlockType::Heading));
    assert!(valid.contains(&BlockType::List));
    assert!(valid.contains(&BlockType::Quote));
}

#[test]
fn test_117_transform_can_transform() {
    let transformer = BlockTransformer::new();

    assert!(transformer.can_transform(&BlockType::Paragraph, &BlockType::Heading));
    assert!(!transformer.can_transform(&BlockType::Paragraph, &BlockType::Image));
}

#[test]
fn test_118_transform_preserves_styles() {
    let transformer = BlockTransformer::new();
    let mut para = Block::new(BlockType::Paragraph);
    para.attributes.content = Some("Test".to_string());
    para.styles.text_color = Some("#ff0000".to_string());

    let result = transformer.transform(&para, BlockType::Heading);
    assert!(result.is_some());
    let heading = result.unwrap();
    assert_eq!(heading.styles.text_color, Some("#ff0000".to_string()));
}

// ============================================================================
// BLOCK DRAG-AND-DROP / ORDERING TESTS (119-123)
// ============================================================================

#[test]
fn test_119_post_move_block() {
    let mut post = PostDocument::new_post("Test");

    let mut block1 = Block::new(BlockType::Paragraph);
    block1.attributes.content = Some("First".to_string());
    let id1 = block1.id;
    post.add_block(block1);

    let mut block2 = Block::new(BlockType::Paragraph);
    block2.attributes.content = Some("Second".to_string());
    post.add_block(block2);

    assert!(post.move_block(id1, 1));
    assert_eq!(post.content.blocks[1].id, id1);
}

#[test]
fn test_120_post_insert_block() {
    let mut post = PostDocument::new_post("Test");
    post.add_block(Block::new(BlockType::Paragraph));
    post.add_block(Block::new(BlockType::Paragraph));

    let mut middle = Block::new(BlockType::Heading);
    middle.attributes.content = Some("Middle".to_string());
    post.insert_block(1, middle);

    assert_eq!(post.content.blocks.len(), 3);
    assert_eq!(post.content.blocks[1].block_type, BlockType::Heading);
}

#[test]
fn test_121_post_remove_block() {
    let mut post = PostDocument::new_post("Test");
    let block = Block::new(BlockType::Paragraph);
    let id = block.id;
    post.add_block(block);

    let removed = post.remove_block(id);
    assert!(removed.is_some());
    assert!(post.content.blocks.is_empty());
}

#[test]
fn test_122_post_get_block() {
    let mut post = PostDocument::new_post("Test");
    let mut block = Block::new(BlockType::Paragraph);
    block.attributes.content = Some("Test content".to_string());
    let id = block.id;
    post.add_block(block);

    let found = post.get_block(id);
    assert!(found.is_some());
    assert_eq!(found.unwrap().attributes.content, Some("Test content".to_string()));
}

#[test]
fn test_123_post_get_block_mut() {
    let mut post = PostDocument::new_post("Test");
    let block = Block::new(BlockType::Paragraph);
    let id = block.id;
    post.add_block(block);

    if let Some(b) = post.get_block_mut(id) {
        b.attributes.content = Some("Modified".to_string());
    }

    assert_eq!(post.get_block(id).unwrap().attributes.content, Some("Modified".to_string()));
}

// ============================================================================
// NESTED BLOCKS TESTS (124-128)
// ============================================================================

#[test]
fn test_124_nested_blocks_group() {
    let mut group = Block::new(BlockType::Group);
    group.children.push(Block::new(BlockType::Paragraph));
    group.children.push(Block::new(BlockType::Paragraph));

    assert_eq!(group.children.len(), 2);
}

#[test]
fn test_125_nested_blocks_columns() {
    let mut columns = Block::new(BlockType::Columns);

    let mut col1 = Block::new(BlockType::Column);
    col1.children.push(Block::new(BlockType::Paragraph));

    let mut col2 = Block::new(BlockType::Column);
    col2.children.push(Block::new(BlockType::Paragraph));

    columns.children.push(col1);
    columns.children.push(col2);

    assert_eq!(columns.children.len(), 2);
    assert_eq!(columns.children[0].children.len(), 1);
}

#[test]
fn test_126_nested_blocks_accordion() {
    let mut accordion = Block::new(BlockType::Accordion);

    let mut item1 = Block::new(BlockType::AccordionItem);
    item1.attributes.content = Some("Section 1".to_string());
    item1.children.push(Block::new(BlockType::Paragraph));

    accordion.children.push(item1);

    assert_eq!(accordion.children.len(), 1);
}

#[test]
fn test_127_nested_blocks_deep() {
    let mut section = Block::new(BlockType::Section);
    let mut group = Block::new(BlockType::Group);
    let mut columns = Block::new(BlockType::Columns);
    let mut column = Block::new(BlockType::Column);
    column.children.push(Block::new(BlockType::Paragraph));

    columns.children.push(column);
    group.children.push(columns);
    section.children.push(group);

    // Verify 4 levels of nesting
    assert_eq!(section.children[0].children[0].children[0].children.len(), 1);
}

#[test]
fn test_128_find_nested_block() {
    let mut post = PostDocument::new_post("Test");

    let mut group = Block::new(BlockType::Group);
    let para = Block::new(BlockType::Paragraph);
    let nested_id = para.id;
    group.children.push(para);
    post.add_block(group);

    let found = post.get_block(nested_id);
    assert!(found.is_some());
}

// ============================================================================
// COPY/PASTE BLOCK HANDLING (129-132)
// ============================================================================

#[test]
fn test_129_block_clone() {
    let mut original = Block::new(BlockType::Paragraph);
    original.attributes.content = Some("Original content".to_string());
    original.styles.text_color = Some("#000000".to_string());

    let cloned = original.clone();

    assert_eq!(cloned.block_type, original.block_type);
    assert_eq!(cloned.attributes.content, original.attributes.content);
    assert_eq!(cloned.styles.text_color, original.styles.text_color);
    // ID is the same in clone (would need new ID for paste)
    assert_eq!(cloned.id, original.id);
}

#[test]
fn test_130_block_serialization_roundtrip() {
    let mut block = Block::new(BlockType::Paragraph);
    block.attributes.content = Some("Test content".to_string());

    let serializer = BlockSerializer::new();
    let json = serializer.to_json(&[block.clone()]).unwrap();
    let restored: Vec<Block> = serializer.from_json(&json).unwrap();

    assert_eq!(restored.len(), 1);
    assert_eq!(restored[0].attributes.content, block.attributes.content);
}

#[test]
fn test_131_merge_blocks() {
    use rustpress_editor::blocks::transform::merge_blocks;

    let mut block1 = Block::new(BlockType::Paragraph);
    block1.attributes.content = Some("First paragraph.".to_string());

    let mut block2 = Block::new(BlockType::Paragraph);
    block2.attributes.content = Some("Second paragraph.".to_string());

    let merged = merge_blocks(&[block1, block2]);
    assert!(merged.is_some());

    let content = merged.unwrap().attributes.content.unwrap();
    assert!(content.contains("First paragraph."));
    assert!(content.contains("Second paragraph."));
}

#[test]
fn test_132_split_block() {
    use rustpress_editor::blocks::transform::split_block;

    let mut block = Block::new(BlockType::Paragraph);
    block.attributes.content = Some("Hello World".to_string());

    let result = split_block(&block, 5);
    assert!(result.is_some());

    let (first, second) = result.unwrap();
    assert_eq!(first.attributes.content, Some("Hello".to_string()));
    assert_eq!(second.attributes.content, Some(" World".to_string()));
}

// ============================================================================
// DOCUMENT VALIDATION TESTS (133-135)
// ============================================================================

#[test]
fn test_133_document_validate_empty_title() {
    let mut post = PostDocument::new_post("");
    let errors = post.validate();

    assert!(errors.iter().any(|e| e.field == "title"));
}

#[test]
fn test_134_document_validate_empty_content() {
    let post = PostDocument::new_post("Test");
    let errors = post.validate();

    assert!(errors.iter().any(|e| e.field == "content"));
}

#[test]
fn test_135_document_validate_seo() {
    let post = PostDocument::new_post("Test");
    let errors = post.validate();

    // Should have SEO recommendations
    assert!(errors.iter().any(|e| e.field.starts_with("seo")));
}

// ============================================================================
// PERMALINK AND SLUG TESTS (136-138)
// ============================================================================

#[test]
fn test_136_post_slug_generation() {
    let post = PostDocument::new_post("Hello World Test");
    assert_eq!(post.slug, "hello-world-test");
}

#[test]
fn test_137_post_slug_special_characters() {
    let post = PostDocument::new_post("Test! @Special# Characters");
    // Slug should only contain alphanumeric and dashes
    assert!(!post.slug.contains("!"));
    assert!(!post.slug.contains("@"));
}

#[test]
fn test_138_post_permalink() {
    let post = PostDocument::new_post("My Post");
    let permalink = post.get_permalink("https://example.com");

    assert!(permalink.starts_with("https://example.com/"));
    assert!(permalink.contains("my-post"));
}

// ============================================================================
// CONTENT ANALYSIS TESTS (139-142)
// ============================================================================

#[test]
fn test_139_content_score_calculation() {
    let mut post = PostDocument::new_post("Test");

    // Add substantial content
    for _ in 0..5 {
        let mut para = Block::new(BlockType::Paragraph);
        para.attributes.content = Some("This is a test paragraph with some content. ".repeat(10));
        post.add_block(para);
    }

    let mut heading = Block::new(BlockType::Heading);
    heading.attributes.level = Some(2);
    post.add_block(heading);

    post.add_block(Block::new(BlockType::Image));

    post.update_stats();
    let score = post.stats.get_content_score();

    assert!(score.score > 0);
}

#[test]
fn test_140_content_density() {
    let mut post = PostDocument::new_post("Test");
    post.add_block(Block::new(BlockType::Paragraph));
    post.add_block(Block::new(BlockType::Paragraph));
    post.add_block(Block::new(BlockType::Image));

    post.update_stats();

    // 2 text blocks, 1 media = 66.6% text, 33.3% media
    assert!(post.stats.density.text_percentage > 60.0);
}

#[test]
fn test_141_heading_counts_total() {
    let counts = HeadingCounts {
        h1: 1,
        h2: 2,
        h3: 3,
        h4: 0,
        h5: 0,
        h6: 0,
    };

    assert_eq!(counts.total(), 6);
}

#[test]
fn test_142_generate_excerpt() {
    let mut post = PostDocument::new_post("Test");
    let mut block = Block::new(BlockType::Paragraph);
    block.attributes.content = Some("This is a very long piece of content that should be truncated when generating an excerpt for the post.".to_string());
    post.add_block(block);

    let excerpt = post.generate_excerpt(50);
    assert!(excerpt.len() <= 53); // 50 + "..."
    assert!(excerpt.ends_with("..."));
}

// ============================================================================
// BLOCK META TESTS (143-145)
// ============================================================================

#[test]
fn test_143_block_meta_anchor() {
    let mut block = Block::new(BlockType::Heading);
    block.meta.anchor = Some("section-1".to_string());

    assert_eq!(block.meta.anchor, Some("section-1".to_string()));
}

#[test]
fn test_144_block_meta_locked() {
    let mut block = Block::new(BlockType::Paragraph);
    block.meta.locked = true;

    assert!(block.meta.locked);
}

#[test]
fn test_145_block_meta_html_attributes() {
    let mut block = Block::new(BlockType::Button);
    block.meta.html_attributes.insert("data-action".to_string(), "submit".to_string());

    assert!(block.meta.html_attributes.contains_key("data-action"));
}

// ============================================================================
// TERM TESTS (146-148)
// ============================================================================

#[test]
fn test_146_term_creation() {
    let term = Term::new(1, "Technology", "technology");

    assert_eq!(term.id, 1);
    assert_eq!(term.name, "Technology");
    assert_eq!(term.slug, "technology");
}

#[test]
fn test_147_term_hierarchical() {
    let mut term = Term::new(2, "Web Development", "web-development");
    term.parent_id = Some(1);

    assert_eq!(term.parent_id, Some(1));
}

#[test]
fn test_148_term_meta() {
    let mut term = Term::new(1, "Featured", "featured");
    term.meta.color = Some("#ff0000".to_string());
    term.meta.featured = true;

    assert!(term.meta.featured);
    assert_eq!(term.meta.color, Some("#ff0000".to_string()));
}

// ============================================================================
// SERIALIZATION FORMAT TESTS (149-152)
// ============================================================================

#[test]
fn test_149_blocks_to_markdown() {
    let serializer = BlockSerializer::new();

    let mut heading = Block::new(BlockType::Heading);
    heading.attributes.content = Some("Title".to_string());
    heading.attributes.level = Some(1);

    let md = serializer.to_markdown(&[heading]);
    assert!(md.starts_with("# "));
}

#[test]
fn test_150_blocks_to_plain_text() {
    let serializer = BlockSerializer::new();

    let mut para = Block::new(BlockType::Paragraph);
    para.attributes.content = Some("<strong>Bold</strong> text".to_string());

    let text = serializer.to_plain_text(&[para]);
    assert!(!text.contains("<strong>"));
}

#[test]
fn test_151_blocks_to_html_with_styles() {
    let serializer = BlockSerializer::new();

    let mut para = Block::new(BlockType::Paragraph);
    para.attributes.content = Some("Styled text".to_string());
    para.styles.text_color = Some("red".to_string());

    let html = serializer.to_html(&[para]);
    assert!(html.contains("style="));
}

#[test]
fn test_152_blocks_code_html_escape() {
    let serializer = BlockSerializer::new();

    let mut code = Block::new(BlockType::Code);
    code.attributes.content = Some("<script>alert('xss')</script>".to_string());

    let html = serializer.to_html(&[code]);
    assert!(html.contains("&lt;script&gt;"));
}

// ============================================================================
// OPERATION TYPES TESTS (153-155)
// ============================================================================

#[test]
fn test_153_operation_insert_text() {
    let op = Operation {
        id: uuid::Uuid::new_v4(),
        op_type: OperationType::InsertText {
            block_id: uuid::Uuid::new_v4(),
            offset: 0,
            text: "Hello".to_string(),
        },
        user_id: 1,
        timestamp: Utc::now(),
        version: 1,
    };

    assert!(matches!(op.op_type, OperationType::InsertText { .. }));
}

#[test]
fn test_154_operation_delete_text() {
    let op = Operation {
        id: uuid::Uuid::new_v4(),
        op_type: OperationType::DeleteText {
            block_id: uuid::Uuid::new_v4(),
            offset: 0,
            length: 5,
        },
        user_id: 1,
        timestamp: Utc::now(),
        version: 1,
    };

    assert!(matches!(op.op_type, OperationType::DeleteText { .. }));
}

#[test]
fn test_155_operation_move_block() {
    let op = Operation {
        id: uuid::Uuid::new_v4(),
        op_type: OperationType::MoveBlock {
            block_id: uuid::Uuid::new_v4(),
            new_parent_id: None,
            new_index: 2,
        },
        user_id: 1,
        timestamp: Utc::now(),
        version: 1,
    };

    assert!(matches!(op.op_type, OperationType::MoveBlock { .. }));
}

// ============================================================================
// ALERT AND SPECIAL BLOCK TESTS (156-158)
// ============================================================================

#[test]
fn test_156_alert_types() {
    let mut alert = Block::new(BlockType::Alert);
    alert.attributes.alert_type = Some(AlertType::Warning);

    assert_eq!(alert.attributes.alert_type, Some(AlertType::Warning));
}

#[test]
fn test_157_button_styles() {
    let mut button = Block::new(BlockType::Button);
    button.attributes.button_style = Some(ButtonStyle::Secondary);

    assert_eq!(button.attributes.button_style, Some(ButtonStyle::Secondary));
}

#[test]
fn test_158_separator_styles() {
    let mut sep = Block::new(BlockType::Separator);
    sep.attributes.separator_style = Some(SeparatorStyle::Dashed);

    assert_eq!(sep.attributes.separator_style, Some(SeparatorStyle::Dashed));
}

// ============================================================================
// PUBLISHING STATUS DISPLAY TESTS (159-160)
// ============================================================================

#[test]
fn test_159_publish_status_display_name() {
    assert_eq!(PublishStatus::Draft.display_name(), "Draft");
    assert_eq!(PublishStatus::Published.display_name(), "Published");
    assert_eq!(PublishStatus::Pending.display_name(), "Pending Review");
}

#[test]
fn test_160_publish_status_color() {
    assert_eq!(PublishStatus::Published.color(), "#10b981");
    assert_eq!(PublishStatus::Draft.color(), "#6b7280");
    assert_eq!(PublishStatus::Trash.color(), "#ef4444");
}

// ============================================================================
// ADDITIONAL EDGE CASE TESTS (161+)
// ============================================================================

#[test]
fn test_161_empty_block_to_html() {
    let serializer = BlockSerializer::new();
    let para = Block::new(BlockType::Paragraph);

    let html = serializer.to_html(&[para]);
    assert!(html.contains("<p>"));
}

#[test]
fn test_162_list_ordered_html() {
    let serializer = BlockSerializer::new();

    let mut list = Block::new(BlockType::List);
    list.attributes.list_type = Some(ListType::Ordered);

    let mut item = Block::new(BlockType::ListItem);
    item.attributes.content = Some("Item 1".to_string());
    list.children.push(item);

    let html = serializer.to_html(&[list]);
    assert!(html.contains("<ol>"));
}

#[test]
fn test_163_post_visibility_is_viewable() {
    let mut post = PostDocument::new_post("Test");

    assert!(!post.publishing.is_viewable()); // Draft

    post.publishing.publish();
    assert!(post.publishing.is_viewable()); // Published
}

#[test]
fn test_164_revision_comparison() {
    use rustpress_editor::post::compare_revisions;
    use rustpress_editor::post::PostRevision;

    let mut content1 = PostContent::default();
    let mut block1 = Block::new(BlockType::Paragraph);
    block1.attributes.content = Some("Original".to_string());
    content1.blocks.push(block1);

    let mut content2 = PostContent::default();
    let mut block2 = Block::new(BlockType::Paragraph);
    block2.attributes.content = Some("Modified".to_string());
    content2.blocks.push(block2);

    let rev1 = PostRevision::new(1, 1, "Test".to_string(), content1, None, None);
    let rev2 = PostRevision::new(2, 1, "Test".to_string(), content2, None, None);

    let comparison = compare_revisions(&rev1, &rev2);
    assert!(comparison.changes.content_changed);
}

#[test]
fn test_165_css_classes() {
    let mut block = Block::new(BlockType::Paragraph);
    block.css_classes = vec!["custom-class".to_string(), "another-class".to_string()];

    let serializer = BlockSerializer::new();
    let html = serializer.to_html(&[block]);

    assert!(html.contains("custom-class"));
    assert!(html.contains("another-class"));
}
