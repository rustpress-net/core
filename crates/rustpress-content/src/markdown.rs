//! Markdown processing with live preview support
//!
//! Provides markdown to HTML conversion with syntax highlighting,
//! table support, and GFM (GitHub Flavored Markdown) extensions.

use pulldown_cmark::{html, Event, Options, Parser, Tag, HeadingLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Markdown processor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownConfig {
    /// Enable tables
    pub tables: bool,

    /// Enable footnotes
    pub footnotes: bool,

    /// Enable strikethrough
    pub strikethrough: bool,

    /// Enable task lists
    pub tasklists: bool,

    /// Enable smart punctuation
    pub smart_punctuation: bool,

    /// Enable heading anchors
    pub heading_anchors: bool,

    /// Enable syntax highlighting
    pub syntax_highlighting: bool,

    /// Syntax highlighting theme
    pub highlight_theme: String,

    /// Enable table of contents generation
    pub toc: bool,

    /// Maximum heading depth for TOC
    pub toc_max_depth: u8,

    /// Enable auto-linking URLs
    pub autolink: bool,
}

impl Default for MarkdownConfig {
    fn default() -> Self {
        Self {
            tables: true,
            footnotes: true,
            strikethrough: true,
            tasklists: true,
            smart_punctuation: true,
            heading_anchors: true,
            syntax_highlighting: true,
            highlight_theme: "github".to_string(),
            toc: false,
            toc_max_depth: 3,
            autolink: true,
        }
    }
}

/// Markdown processor
pub struct MarkdownProcessor {
    config: MarkdownConfig,
}

impl MarkdownProcessor {
    /// Create new processor with default config
    pub fn new() -> Self {
        Self {
            config: MarkdownConfig::default(),
        }
    }

    /// Create processor with custom config
    pub fn with_config(config: MarkdownConfig) -> Self {
        Self { config }
    }

    /// Convert markdown to HTML
    pub fn to_html(&self, markdown: &str) -> String {
        let options = self.build_options();
        let parser = Parser::new_ext(markdown, options);

        // Process events for custom handling
        let events: Vec<Event> = if self.config.heading_anchors {
            self.add_heading_anchors(parser)
        } else {
            parser.collect()
        };

        // Render to HTML
        let mut html_output = String::new();
        html::push_html(&mut html_output, events.into_iter());

        // Apply syntax highlighting if enabled
        if self.config.syntax_highlighting {
            html_output = self.apply_syntax_highlighting(&html_output);
        }

        // Sanitize output
        ammonia::clean(&html_output)
    }

    /// Convert markdown to plain text (strip formatting)
    pub fn to_plain_text(&self, markdown: &str) -> String {
        let options = self.build_options();
        let parser = Parser::new_ext(markdown, options);

        let mut text = String::new();
        for event in parser {
            if let Event::Text(t) = event {
                text.push_str(&t);
            } else if let Event::SoftBreak | Event::HardBreak = event {
                text.push(' ');
            }
        }

        text
    }

    /// Extract headings for TOC
    pub fn extract_headings(&self, markdown: &str) -> Vec<Heading> {
        let options = self.build_options();
        let parser = Parser::new_ext(markdown, options);

        let mut headings = Vec::new();
        let mut current_heading: Option<(u8, String)> = None;

        for event in parser {
            match event {
                Event::Start(Tag::Heading(level, _, _)) => {
                    let level_num = heading_level_to_u8(level);
                    current_heading = Some((level_num, String::new()));
                }
                Event::Text(text) => {
                    if let Some((_, ref mut content)) = current_heading {
                        content.push_str(&text);
                    }
                }
                Event::End(Tag::Heading(_, _, _)) => {
                    if let Some((level, content)) = current_heading.take() {
                        if level <= self.config.toc_max_depth {
                            let slug = slug::slugify(&content);
                            headings.push(Heading {
                                level,
                                text: content,
                                slug,
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        headings
    }

    /// Generate table of contents HTML
    pub fn generate_toc(&self, markdown: &str) -> String {
        let headings = self.extract_headings(markdown);

        if headings.is_empty() {
            return String::new();
        }

        let mut html = String::from(r#"<nav class="toc"><ul>"#);
        let mut prev_level = 0u8;

        for heading in &headings {
            // Handle nesting
            if heading.level > prev_level {
                for _ in prev_level..heading.level {
                    html.push_str("<ul>");
                }
            } else if heading.level < prev_level {
                for _ in heading.level..prev_level {
                    html.push_str("</ul>");
                }
            }

            html.push_str(&format!(
                "<li><a href=\"#{}\">{}</a></li>",
                heading.slug, heading.text
            ));

            prev_level = heading.level;
        }

        // Close remaining lists
        for _ in 0..prev_level {
            html.push_str("</ul>");
        }

        html.push_str("</nav>");
        html
    }

    /// Convert HTML back to markdown (best effort)
    pub fn from_html(&self, html: &str) -> String {
        // Simple HTML to markdown conversion
        let mut markdown = html.to_string();

        // Replace common HTML tags with markdown equivalents
        let replacements = [
            (r"<h1[^>]*>(.*?)</h1>", "\n# $1\n"),
            (r"<h2[^>]*>(.*?)</h2>", "\n## $1\n"),
            (r"<h3[^>]*>(.*?)</h3>", "\n### $1\n"),
            (r"<h4[^>]*>(.*?)</h4>", "\n#### $1\n"),
            (r"<h5[^>]*>(.*?)</h5>", "\n##### $1\n"),
            (r"<h6[^>]*>(.*?)</h6>", "\n###### $1\n"),
            (r"<p[^>]*>(.*?)</p>", "\n$1\n"),
            (r"<strong[^>]*>(.*?)</strong>", "**$1**"),
            (r"<b[^>]*>(.*?)</b>", "**$1**"),
            (r"<em[^>]*>(.*?)</em>", "*$1*"),
            (r"<i[^>]*>(.*?)</i>", "*$1*"),
            (r"<code[^>]*>(.*?)</code>", "`$1`"),
            (r#"<a[^>]*href=["']([^"']*)["'][^>]*>(.*?)</a>"#, "[$2]($1)"),
            (r#"<img[^>]*src=["']([^"']*)["'][^>]*/?>"#, "![]($1)"),
            (r"<br\s*/?>", "\n"),
            (r"<hr\s*/?>", "\n---\n"),
            (r"<li[^>]*>(.*?)</li>", "- $1"),
            (r"<blockquote[^>]*>(.*?)</blockquote>", "> $1"),
        ];

        for (pattern, replacement) in replacements {
            if let Ok(re) = regex::Regex::new(&format!("(?si){}", pattern)) {
                markdown = re.replace_all(&markdown, replacement).to_string();
            }
        }

        // Remove remaining HTML tags
        if let Ok(re) = regex::Regex::new(r"<[^>]+>") {
            markdown = re.replace_all(&markdown, "").to_string();
        }

        // Clean up whitespace
        if let Ok(re) = regex::Regex::new(r"\n{3,}") {
            markdown = re.replace_all(&markdown, "\n\n").to_string();
        }

        markdown.trim().to_string()
    }

    /// Get word count from markdown
    pub fn word_count(&self, markdown: &str) -> usize {
        let text = self.to_plain_text(markdown);
        text.split_whitespace().count()
    }

    /// Get reading time estimate (words per minute)
    pub fn reading_time(&self, markdown: &str, wpm: usize) -> usize {
        let words = self.word_count(markdown);
        let wpm = if wpm == 0 { 200 } else { wpm }; // Default 200 WPM
        (words + wpm - 1) / wpm // Round up
    }

    /// Extract first image URL from markdown
    pub fn extract_first_image(&self, markdown: &str) -> Option<String> {
        let re = regex::Regex::new(r"!\[.*?\]\(([^)]+)\)").ok()?;
        re.captures(markdown)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Extract all links from markdown
    pub fn extract_links(&self, markdown: &str) -> Vec<MarkdownLink> {
        let mut links = Vec::new();

        if let Ok(re) = regex::Regex::new(r"\[([^\]]+)\]\(([^)]+)\)") {
            for cap in re.captures_iter(markdown) {
                if let (Some(text), Some(url)) = (cap.get(1), cap.get(2)) {
                    links.push(MarkdownLink {
                        text: text.as_str().to_string(),
                        url: url.as_str().to_string(),
                    });
                }
            }
        }

        links
    }

    fn build_options(&self) -> Options {
        let mut options = Options::empty();

        if self.config.tables {
            options.insert(Options::ENABLE_TABLES);
        }
        if self.config.footnotes {
            options.insert(Options::ENABLE_FOOTNOTES);
        }
        if self.config.strikethrough {
            options.insert(Options::ENABLE_STRIKETHROUGH);
        }
        if self.config.tasklists {
            options.insert(Options::ENABLE_TASKLISTS);
        }
        if self.config.smart_punctuation {
            options.insert(Options::ENABLE_SMART_PUNCTUATION);
        }

        options
    }

    fn add_heading_anchors<'a>(&self, parser: Parser<'a, 'a>) -> Vec<Event<'a>> {
        let mut events = Vec::new();
        let mut in_heading = false;
        let mut heading_text = String::new();
        let mut heading_level: u8 = 0;

        for event in parser {
            match &event {
                Event::Start(Tag::Heading(level, _, _)) => {
                    in_heading = true;
                    heading_level = heading_level_to_u8(*level);
                    heading_text.clear();
                    events.push(event);
                }
                Event::Text(text) if in_heading => {
                    heading_text.push_str(text);
                    events.push(event);
                }
                Event::End(Tag::Heading(_, _, _)) => {
                    in_heading = false;
                    let slug = slug::slugify(&heading_text);

                    // Insert anchor before closing tag
                    let anchor_html = format!(
                        " <a class=\"anchor\" href=\"#{}\" aria-label=\"Permalink\">&#x1f517;</a>",
                        slug
                    );
                    events.push(Event::Html(anchor_html.into()));
                    events.push(event);

                    // Update the heading start tag to include ID
                    if let Some(start_idx) = events.iter().rposition(|e| {
                        matches!(e, Event::Start(Tag::Heading(_, _, _)))
                    }) {
                        let new_tag = format!("<h{} id=\"{}\">", heading_level, slug);
                        events[start_idx] = Event::Html(new_tag.into());
                    }
                }
                _ => events.push(event),
            }
        }

        events
    }

    fn apply_syntax_highlighting(&self, html: &str) -> String {
        // Simple syntax highlighting using regex
        // In production, use a proper syntax highlighter like syntect
        let mut result = html.to_string();

        // Find code blocks with language
        if let Ok(re) = regex::Regex::new(r#"<pre><code class="language-(\w+)">([\s\S]*?)</code></pre>"#) {
            result = re.replace_all(&result, |caps: &regex::Captures| {
                let lang = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let code = caps.get(2).map(|m| m.as_str()).unwrap_or("");

                format!(
                    r#"<pre class="language-{lang}"><code class="language-{lang}">{code}</code></pre>"#,
                    lang = lang,
                    code = highlight_code(code, lang)
                )
            }).to_string();
        }

        result
    }
}

impl Default for MarkdownProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert HeadingLevel to u8
fn heading_level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

/// Heading information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heading {
    pub level: u8,
    pub text: String,
    pub slug: String,
}

/// Link information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownLink {
    pub text: String,
    pub url: String,
}

/// Simple syntax highlighting (basic implementation)
fn highlight_code(code: &str, language: &str) -> String {
    // This is a basic implementation. For production, use syntect.
    let keywords: HashMap<&str, Vec<&str>> = [
        ("rust", vec!["fn", "let", "mut", "const", "static", "pub", "mod", "use", "impl", "trait", "struct", "enum", "type", "where", "async", "await", "match", "if", "else", "for", "while", "loop", "return", "break", "continue", "self", "Self", "super", "crate"]),
        ("javascript", vec!["function", "const", "let", "var", "return", "if", "else", "for", "while", "do", "switch", "case", "break", "continue", "class", "extends", "import", "export", "default", "async", "await", "try", "catch", "finally", "throw", "new", "this"]),
        ("python", vec!["def", "class", "return", "if", "elif", "else", "for", "while", "break", "continue", "import", "from", "as", "try", "except", "finally", "raise", "with", "lambda", "yield", "global", "nonlocal", "pass", "True", "False", "None", "and", "or", "not", "in", "is"]),
        ("go", vec!["func", "package", "import", "type", "struct", "interface", "const", "var", "return", "if", "else", "for", "switch", "case", "default", "break", "continue", "go", "chan", "select", "defer", "range", "map", "make", "new"]),
        ("sql", vec!["SELECT", "FROM", "WHERE", "JOIN", "LEFT", "RIGHT", "INNER", "OUTER", "ON", "AND", "OR", "NOT", "IN", "IS", "NULL", "AS", "ORDER", "BY", "GROUP", "HAVING", "LIMIT", "OFFSET", "INSERT", "INTO", "VALUES", "UPDATE", "SET", "DELETE", "CREATE", "TABLE", "ALTER", "DROP", "INDEX"]),
    ].into_iter().collect();

    let lang_keywords = keywords.get(language).cloned().unwrap_or_default();

    if lang_keywords.is_empty() {
        return code.to_string();
    }

    let mut result = code.to_string();

    // Highlight strings
    if let Ok(re) = regex::Regex::new(r#"("[^"]*"|'[^']*')"#) {
        result = re.replace_all(&result, r#"<span class="string">$1</span>"#).to_string();
    }

    // Highlight comments (simple // and # style)
    if let Ok(re) = regex::Regex::new(r"(//.*|#.*)$") {
        result = re.replace_all(&result, r#"<span class="comment">$1</span>"#).to_string();
    }

    // Highlight keywords
    for keyword in lang_keywords {
        let pattern = format!(r"\b({})\b", regex::escape(keyword));
        if let Ok(re) = regex::Regex::new(&pattern) {
            result = re.replace_all(&result, r#"<span class="keyword">$1</span>"#).to_string();
        }
    }

    // Highlight numbers
    if let Ok(re) = regex::Regex::new(r"\b(\d+(?:\.\d+)?)\b") {
        result = re.replace_all(&result, r#"<span class="number">$1</span>"#).to_string();
    }

    result
}

/// Live preview data for editor synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivePreviewData {
    pub html: String,
    pub toc: Vec<Heading>,
    pub word_count: usize,
    pub reading_time: usize,
    pub links: Vec<MarkdownLink>,
}

impl LivePreviewData {
    /// Generate live preview data from markdown
    pub fn from_markdown(markdown: &str) -> Self {
        let processor = MarkdownProcessor::new();

        Self {
            html: processor.to_html(markdown),
            toc: processor.extract_headings(markdown),
            word_count: processor.word_count(markdown),
            reading_time: processor.reading_time(markdown, 200),
            links: processor.extract_links(markdown),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_to_html() {
        let processor = MarkdownProcessor::new();
        let html = processor.to_html("# Hello\n\nThis is **bold** text.");

        assert!(html.contains("<h1"));
        assert!(html.contains("Hello"));
        assert!(html.contains("<strong>bold</strong>"));
    }

    #[test]
    fn test_extract_headings() {
        let processor = MarkdownProcessor::new();
        let headings = processor.extract_headings("# Title\n## Section 1\n## Section 2");

        assert_eq!(headings.len(), 3);
        assert_eq!(headings[0].text, "Title");
        assert_eq!(headings[0].level, 1);
    }

    #[test]
    fn test_word_count() {
        let processor = MarkdownProcessor::new();
        let count = processor.word_count("Hello world, this is a test.");

        assert_eq!(count, 6);
    }

    #[test]
    fn test_extract_links() {
        let processor = MarkdownProcessor::new();
        let links = processor.extract_links("Check out [Google](https://google.com) and [GitHub](https://github.com)");

        assert_eq!(links.len(), 2);
        assert_eq!(links[0].text, "Google");
        assert_eq!(links[0].url, "https://google.com");
    }
}
