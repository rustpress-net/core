//! # Table of Contents and Content Structure
//!
//! TOC generation, anchor management, and footnotes/endnotes.
//!
//! Features:
//! - Automatic TOC generation from headings
//! - Anchor link management
//! - Footnotes and endnotes system
//! - Content structure analysis

use regex::Regex;
use serde::{Deserialize, Serialize};
use slug::slugify;
use std::collections::HashMap;

// ============================================================================
// Table of Contents
// ============================================================================

/// Table of contents entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocEntry {
    /// Entry ID
    pub id: String,

    /// Heading level (1-6)
    pub level: u8,

    /// Heading text
    pub text: String,

    /// Anchor link
    pub anchor: String,

    /// Child entries
    pub children: Vec<TocEntry>,
}

impl TocEntry {
    pub fn new(level: u8, text: &str, anchor: &str) -> Self {
        Self {
            id: anchor.to_string(),
            level,
            text: text.to_string(),
            anchor: anchor.to_string(),
            children: Vec::new(),
        }
    }

    /// Render as HTML list item
    pub fn to_html(&self) -> String {
        let mut html = format!(r#"<li><a href="#{}">{}</a>"#, self.anchor, self.text);

        if !self.children.is_empty() {
            html.push_str("<ul>");
            for child in &self.children {
                html.push_str(&child.to_html());
            }
            html.push_str("</ul>");
        }

        html.push_str("</li>");
        html
    }
}

/// TOC configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocConfig {
    /// Minimum heading level to include (1-6)
    pub min_level: u8,

    /// Maximum heading level to include (1-6)
    pub max_level: u8,

    /// Minimum number of headings to show TOC
    pub min_headings: usize,

    /// TOC title
    pub title: String,

    /// CSS class for TOC container
    pub class: String,

    /// Whether to show toggle
    pub collapsible: bool,

    /// Collapsed by default
    pub collapsed: bool,

    /// Ordered list (numbered)
    pub ordered: bool,

    /// Add smooth scrolling
    pub smooth_scroll: bool,

    /// Highlight current section
    pub highlight_current: bool,

    /// Anchor prefix
    pub anchor_prefix: String,
}

impl Default for TocConfig {
    fn default() -> Self {
        Self {
            min_level: 2,
            max_level: 4,
            min_headings: 3,
            title: "Table of Contents".to_string(),
            class: "toc".to_string(),
            collapsible: true,
            collapsed: false,
            ordered: false,
            smooth_scroll: true,
            highlight_current: true,
            anchor_prefix: String::new(),
        }
    }
}

/// TOC generator
pub struct TocGenerator {
    config: TocConfig,
    anchor_counts: HashMap<String, usize>,
}

impl Default for TocGenerator {
    fn default() -> Self {
        Self::new(TocConfig::default())
    }
}

impl TocGenerator {
    pub fn new(config: TocConfig) -> Self {
        Self {
            config,
            anchor_counts: HashMap::new(),
        }
    }

    /// Generate TOC from HTML content
    pub fn generate(&mut self, content: &str) -> (Vec<TocEntry>, String) {
        self.anchor_counts.clear();

        let heading_re = Regex::new(r"<h([1-6])([^>]*)>(.*?)</h\1>").unwrap();
        let id_re = Regex::new(r#"id="([^"]*)""#).unwrap();

        let mut entries = Vec::new();
        let mut modified_content = content.to_string();

        for caps in heading_re.captures_iter(content) {
            let level: u8 = caps[1].parse().unwrap_or(1);

            // Check level bounds
            if level < self.config.min_level || level > self.config.max_level {
                continue;
            }

            let attrs = &caps[2];
            let text = strip_html_tags(&caps[3]);

            // Get or generate anchor
            let anchor = if let Some(id_caps) = id_re.captures(attrs) {
                id_caps[1].to_string()
            } else {
                self.generate_anchor(&text)
            };

            entries.push(TocEntry::new(level, &text, &anchor));

            // Add ID to heading if not present
            if !attrs.contains("id=") {
                let old_tag = format!("<h{}{}>{}</h{}>", level, attrs, &caps[3], level);
                let new_tag = format!(
                    r#"<h{} id="{}"{}>{}</h{}>"#,
                    level, anchor, attrs, &caps[3], level
                );
                modified_content = modified_content.replace(&old_tag, &new_tag);
            }
        }

        // Build tree structure
        let tree = self.build_tree(entries);

        (tree, modified_content)
    }

    /// Generate unique anchor from text
    fn generate_anchor(&mut self, text: &str) -> String {
        let base = if self.config.anchor_prefix.is_empty() {
            slugify(text)
        } else {
            format!("{}-{}", self.config.anchor_prefix, slugify(text))
        };

        let count = self.anchor_counts.entry(base.clone()).or_insert(0);
        *count += 1;

        if *count == 1 {
            base
        } else {
            format!("{}-{}", base, *count - 1)
        }
    }

    /// Build tree structure from flat list
    fn build_tree(&self, entries: Vec<TocEntry>) -> Vec<TocEntry> {
        let mut result: Vec<TocEntry> = Vec::new();
        let mut stack: Vec<(u8, usize)> = Vec::new(); // (level, index)

        for entry in entries {
            // Pop stack until we find parent level
            while let Some(&(level, _)) = stack.last() {
                if level >= entry.level {
                    stack.pop();
                } else {
                    break;
                }
            }

            if stack.is_empty() {
                result.push(entry.clone());
                stack.push((entry.level, result.len() - 1));
            } else {
                // Add as child of last item on stack
                let path: Vec<usize> = stack.iter().map(|(_, idx)| *idx).collect();
                Self::add_to_tree(&mut result, &path, entry.clone());
                stack.push((entry.level, 0)); // Index doesn't matter for children
            }
        }

        result
    }

    fn add_to_tree(entries: &mut [TocEntry], path: &[usize], entry: TocEntry) {
        if path.len() == 1 {
            entries[path[0]].children.push(entry);
        } else {
            Self::add_to_tree(&mut entries[path[0]].children, &path[1..], entry);
        }
    }

    /// Render TOC as HTML
    pub fn render(&self, entries: &[TocEntry]) -> String {
        if entries.len() < self.config.min_headings {
            return String::new();
        }

        let list_tag = if self.config.ordered { "ol" } else { "ul" };
        let collapsed_class = if self.config.collapsed { " collapsed" } else { "" };

        let mut html = format!(
            r#"<nav class="{}{}""#,
            self.config.class, collapsed_class
        );

        if self.config.smooth_scroll {
            html.push_str(r#" data-smooth-scroll="true""#);
        }
        if self.config.highlight_current {
            html.push_str(r#" data-highlight-current="true""#);
        }

        html.push('>');

        if !self.config.title.is_empty() {
            html.push_str(&format!(r#"<div class="toc-title">{}</div>"#, self.config.title));
        }

        if self.config.collapsible {
            html.push_str(r#"<button class="toc-toggle" aria-expanded="true">Toggle</button>"#);
        }

        html.push_str(&format!("<{}>", list_tag));
        for entry in entries {
            html.push_str(&entry.to_html());
        }
        html.push_str(&format!("</{}>", list_tag));

        html.push_str("</nav>");
        html
    }
}

// ============================================================================
// Anchor Management
// ============================================================================

/// Anchor link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anchor {
    pub id: String,
    pub text: String,
    pub element_type: String,
    pub position: usize,
}

/// Anchor manager
pub struct AnchorManager {
    anchors: HashMap<String, Anchor>,
}

impl Default for AnchorManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AnchorManager {
    pub fn new() -> Self {
        Self {
            anchors: HashMap::new(),
        }
    }

    /// Extract all anchors from content
    pub fn extract(&mut self, content: &str) {
        self.anchors.clear();

        let id_re = Regex::new(r#"<(\w+)[^>]*id="([^"]*)"[^>]*>([^<]*)"#).unwrap();

        for (pos, caps) in id_re.captures_iter(content).enumerate() {
            let element = &caps[1];
            let id = &caps[2];
            let text = &caps[3];

            self.anchors.insert(id.to_string(), Anchor {
                id: id.to_string(),
                text: text.to_string(),
                element_type: element.to_string(),
                position: pos,
            });
        }
    }

    /// Get anchor by ID
    pub fn get(&self, id: &str) -> Option<&Anchor> {
        self.anchors.get(id)
    }

    /// Get all anchors
    pub fn get_all(&self) -> Vec<&Anchor> {
        self.anchors.values().collect()
    }

    /// Check if anchor exists
    pub fn exists(&self, id: &str) -> bool {
        self.anchors.contains_key(id)
    }

    /// Validate internal links in content
    pub fn validate_links(&self, content: &str) -> Vec<String> {
        let mut broken = Vec::new();
        let link_re = Regex::new(r#"href="#([^"]*)""#).unwrap();

        for caps in link_re.captures_iter(content) {
            let target = &caps[1];
            if !self.anchors.contains_key(target) {
                broken.push(target.to_string());
            }
        }

        broken
    }
}

// ============================================================================
// Footnotes and Endnotes
// ============================================================================

/// Footnote/endnote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Footnote {
    /// Unique ID
    pub id: String,

    /// Note number
    pub number: usize,

    /// Note content
    pub content: String,

    /// Reference text in body
    pub reference_text: String,

    /// Position in content
    pub position: usize,
}

/// Footnote configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootnoteConfig {
    /// Render as endnotes (at end) or footnotes (inline/bottom of section)
    pub endnotes: bool,

    /// Reference format in text
    pub reference_format: String,

    /// Backlink text
    pub backlink_text: String,

    /// CSS class for footnotes
    pub class: String,

    /// Title for notes section
    pub title: String,

    /// Number format (numeric, roman, alpha)
    pub number_format: NumberFormat,
}

impl Default for FootnoteConfig {
    fn default() -> Self {
        Self {
            endnotes: true,
            reference_format: "[%n]".to_string(),
            backlink_text: "↩".to_string(),
            class: "footnotes".to_string(),
            title: "Notes".to_string(),
            number_format: NumberFormat::Numeric,
        }
    }
}

/// Number format for footnotes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumberFormat {
    Numeric,
    Roman,
    RomanLower,
    Alpha,
    AlphaLower,
}

impl NumberFormat {
    pub fn format(&self, n: usize) -> String {
        match self {
            Self::Numeric => n.to_string(),
            Self::Roman => to_roman(n, false),
            Self::RomanLower => to_roman(n, true),
            Self::Alpha => to_alpha(n, false),
            Self::AlphaLower => to_alpha(n, true),
        }
    }
}

/// Convert number to Roman numerals
fn to_roman(n: usize, lowercase: bool) -> String {
    let numerals = [
        (1000, "M"), (900, "CM"), (500, "D"), (400, "CD"),
        (100, "C"), (90, "XC"), (50, "L"), (40, "XL"),
        (10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I"),
    ];

    let mut result = String::new();
    let mut remaining = n;

    for (value, numeral) in numerals {
        while remaining >= value {
            result.push_str(numeral);
            remaining -= value;
        }
    }

    if lowercase {
        result.to_lowercase()
    } else {
        result
    }
}

/// Convert number to alphabetic
fn to_alpha(n: usize, lowercase: bool) -> String {
    let base = if lowercase { b'a' } else { b'A' };
    let mut result = String::new();
    let mut remaining = n;

    while remaining > 0 {
        remaining -= 1;
        result.insert(0, (base + (remaining % 26) as u8) as char);
        remaining /= 26;
    }

    result
}

/// Footnote processor
pub struct FootnoteProcessor {
    config: FootnoteConfig,
}

impl Default for FootnoteProcessor {
    fn default() -> Self {
        Self::new(FootnoteConfig::default())
    }
}

impl FootnoteProcessor {
    pub fn new(config: FootnoteConfig) -> Self {
        Self { config }
    }

    /// Process content and extract footnotes
    /// Uses format: [^1] for reference and [^1]: content for definition
    pub fn process(&self, content: &str) -> (String, Vec<Footnote>) {
        // Find footnote definitions
        let def_re = Regex::new(r"\[\^(\d+)\]:\s*(.+?)(?:\n\n|\n\[\^|$)").unwrap();
        let ref_re = Regex::new(r"\[\^(\d+)\](?!:)").unwrap();

        let mut footnotes: HashMap<String, Footnote> = HashMap::new();

        // Extract definitions
        for caps in def_re.captures_iter(content) {
            let id = caps[1].to_string();
            let note_content = caps[2].trim().to_string();

            footnotes.insert(id.clone(), Footnote {
                id: id.clone(),
                number: id.parse().unwrap_or(0),
                content: note_content,
                reference_text: String::new(),
                position: 0,
            });
        }

        // Remove definitions from content
        let content = def_re.replace_all(content, "").to_string();

        // Replace references with links
        let mut processed = content.clone();
        let mut note_list: Vec<Footnote> = Vec::new();

        for (pos, caps) in ref_re.captures_iter(&content).enumerate() {
            let id = &caps[1];

            if let Some(footnote) = footnotes.get_mut(id) {
                footnote.position = pos;

                let number = self.config.number_format.format(footnote.number);
                let reference = self.config.reference_format.replace("%n", &number);

                let link = format!(
                    r#"<sup class="footnote-ref"><a href="#fn-{}" id="fnref-{}">{}</a></sup>"#,
                    id, id, reference
                );

                processed = processed.replacen(&caps[0], &link, 1);
                note_list.push(footnote.clone());
            }
        }

        // Sort by position
        note_list.sort_by_key(|f| f.position);

        (processed, note_list)
    }

    /// Render footnotes section
    pub fn render(&self, footnotes: &[Footnote]) -> String {
        if footnotes.is_empty() {
            return String::new();
        }

        let mut html = format!(
            r#"<section class="{}"><hr><h4>{}</h4><ol>"#,
            self.config.class, self.config.title
        );

        for footnote in footnotes {
            let number = self.config.number_format.format(footnote.number);
            html.push_str(&format!(
                r#"<li id="fn-{}" value="{}"><p>{} <a href="#fnref-{}" class="footnote-backref">{}</a></p></li>"#,
                footnote.id, footnote.number, footnote.content, footnote.id, self.config.backlink_text
            ));
        }

        html.push_str("</ol></section>");
        html
    }
}

/// Strip HTML tags from string
fn strip_html_tags(s: &str) -> String {
    let re = Regex::new(r"<[^>]+>").unwrap();
    re.replace_all(s, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toc_generation() {
        let mut generator = TocGenerator::default();
        let content = r#"<h2>Introduction</h2><p>Text</p><h2>Chapter 1</h2><h3>Section 1.1</h3>"#;

        let (entries, _) = generator.generate(content);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].text, "Introduction");
    }

    #[test]
    fn test_anchor_extraction() {
        let mut manager = AnchorManager::new();
        let content = r#"<h2 id="intro">Introduction</h2><p id="para1">Text</p>"#;

        manager.extract(content);
        assert!(manager.exists("intro"));
        assert!(manager.exists("para1"));
    }

    #[test]
    fn test_footnote_processing() {
        let processor = FootnoteProcessor::default();
        let content = "This is text[^1] with a footnote.\n\n[^1]: This is the footnote content.";

        let (processed, footnotes) = processor.process(content);
        assert_eq!(footnotes.len(), 1);
        assert!(processed.contains("fnref-1"));
    }

    #[test]
    fn test_number_formats() {
        assert_eq!(NumberFormat::Numeric.format(5), "5");
        assert_eq!(NumberFormat::Roman.format(4), "IV");
        assert_eq!(NumberFormat::RomanLower.format(4), "iv");
        assert_eq!(NumberFormat::Alpha.format(1), "A");
        assert_eq!(NumberFormat::Alpha.format(27), "AA");
    }
}
