//! # Excerpt and Reading Time
//!
//! Smart excerpt generation and reading time calculation.
//!
//! Features:
//! - Automatic excerpt generation with smart truncation
//! - Reading time estimation
//! - Word/character counting
//! - Text summarization helpers

use regex::Regex;
use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

/// Excerpt generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcerptConfig {
    /// Maximum length in words
    pub max_words: usize,

    /// Maximum length in characters
    pub max_chars: usize,

    /// Ending to append when truncated
    pub more_text: String,

    /// Strip HTML tags
    pub strip_html: bool,

    /// Strip shortcodes
    pub strip_shortcodes: bool,

    /// Preserve paragraph breaks
    pub preserve_paragraphs: bool,

    /// Use first paragraph only
    pub first_paragraph_only: bool,

    /// Minimum excerpt length (don't truncate if shorter)
    pub min_length: usize,
}

impl Default for ExcerptConfig {
    fn default() -> Self {
        Self {
            max_words: 55,
            max_chars: 0, // 0 = use word count
            more_text: "…".to_string(),
            strip_html: true,
            strip_shortcodes: true,
            preserve_paragraphs: false,
            first_paragraph_only: false,
            min_length: 10,
        }
    }
}

/// Reading speed configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingTimeConfig {
    /// Words per minute (average reading speed)
    pub words_per_minute: u32,

    /// Seconds per image
    pub seconds_per_image: u32,

    /// Minimum reading time in minutes
    pub min_minutes: u32,

    /// Include images in calculation
    pub count_images: bool,

    /// Include code blocks (at reduced speed)
    pub count_code: bool,

    /// Code reading speed (words per minute, typically slower)
    pub code_wpm: u32,
}

impl Default for ReadingTimeConfig {
    fn default() -> Self {
        Self {
            words_per_minute: 200,
            seconds_per_image: 12,
            min_minutes: 1,
            count_images: true,
            count_code: true,
            code_wpm: 100,
        }
    }
}

/// Result of reading time calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingTime {
    /// Reading time in minutes
    pub minutes: u32,

    /// Reading time in seconds (for more precision)
    pub seconds: u32,

    /// Word count
    pub word_count: usize,

    /// Character count
    pub char_count: usize,

    /// Image count
    pub image_count: usize,

    /// Code block word count
    pub code_word_count: usize,

    /// Formatted display string
    pub display: String,
}

impl ReadingTime {
    /// Format for display
    pub fn format(&self) -> String {
        if self.minutes == 0 {
            "Less than a minute".to_string()
        } else if self.minutes == 1 {
            "1 min read".to_string()
        } else {
            format!("{} min read", self.minutes)
        }
    }

    /// Format with detailed breakdown
    pub fn format_detailed(&self) -> String {
        format!(
            "{} min read ({} words, {} images)",
            self.minutes, self.word_count, self.image_count
        )
    }
}

/// Content analyzer for excerpts and reading time
pub struct ContentAnalyzer {
    excerpt_config: ExcerptConfig,
    reading_config: ReadingTimeConfig,
}

impl Default for ContentAnalyzer {
    fn default() -> Self {
        Self {
            excerpt_config: ExcerptConfig::default(),
            reading_config: ReadingTimeConfig::default(),
        }
    }
}

impl ContentAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_excerpt_config(mut self, config: ExcerptConfig) -> Self {
        self.excerpt_config = config;
        self
    }

    pub fn with_reading_config(mut self, config: ReadingTimeConfig) -> Self {
        self.reading_config = config;
        self
    }

    /// Generate excerpt from content
    pub fn generate_excerpt(&self, content: &str) -> String {
        self.generate_excerpt_with_config(content, &self.excerpt_config)
    }

    /// Generate excerpt with custom config
    pub fn generate_excerpt_with_config(&self, content: &str, config: &ExcerptConfig) -> String {
        let mut text = content.to_string();

        // Strip shortcodes if configured
        if config.strip_shortcodes {
            text = self.strip_shortcodes(&text);
        }

        // Strip HTML if configured
        if config.strip_html {
            text = self.strip_html(&text);
        }

        // Get first paragraph if configured
        if config.first_paragraph_only {
            text = self.get_first_paragraph(&text);
        }

        // Normalize whitespace
        text = self.normalize_whitespace(&text);

        // Check if already short enough
        let word_count = text.split_whitespace().count();
        if word_count <= config.min_length {
            return text;
        }

        // Truncate by words or characters
        let truncated = if config.max_chars > 0 {
            self.truncate_by_chars(&text, config.max_chars)
        } else {
            self.truncate_by_words(&text, config.max_words)
        };

        // Add more text indicator if truncated
        if truncated.len() < text.len() {
            format!("{}{}", truncated.trim_end_matches(&['.', ',', ' ', '\n'][..]), &config.more_text)
        } else {
            truncated
        }
    }

    /// Calculate reading time
    pub fn calculate_reading_time(&self, content: &str) -> ReadingTime {
        self.calculate_reading_time_with_config(content, &self.reading_config)
    }

    /// Calculate reading time with custom config
    pub fn calculate_reading_time_with_config(&self, content: &str, config: &ReadingTimeConfig) -> ReadingTime {
        // Count images
        let image_count = self.count_images(content);

        // Extract code blocks
        let (regular_text, code_text) = self.separate_code_blocks(content);

        // Strip HTML for word counting
        let clean_text = self.strip_html(&self.strip_shortcodes(&regular_text));
        let clean_code = self.strip_html(&code_text);

        // Count words
        let word_count = clean_text.split_whitespace().count();
        let code_word_count = if config.count_code {
            clean_code.split_whitespace().count()
        } else {
            0
        };

        // Count characters
        let char_count = clean_text.chars().count();

        // Calculate time
        let text_seconds = (word_count as f64 / config.words_per_minute as f64 * 60.0) as u32;
        let code_seconds = if config.count_code {
            (code_word_count as f64 / config.code_wpm as f64 * 60.0) as u32
        } else {
            0
        };
        let image_seconds = if config.count_images {
            image_count as u32 * config.seconds_per_image
        } else {
            0
        };

        let total_seconds = text_seconds + code_seconds + image_seconds;
        let minutes = (total_seconds / 60).max(config.min_minutes);

        let reading_time = ReadingTime {
            minutes,
            seconds: total_seconds,
            word_count,
            char_count,
            image_count,
            code_word_count,
            display: String::new(),
        };

        ReadingTime {
            display: reading_time.format(),
            ..reading_time
        }
    }

    /// Strip HTML tags
    fn strip_html(&self, content: &str) -> String {
        let re = Regex::new(r"<[^>]+>").unwrap();
        let text = re.replace_all(content, " ");

        // Decode common HTML entities
        text.replace("&nbsp;", " ")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
    }

    /// Strip shortcodes
    fn strip_shortcodes(&self, content: &str) -> String {
        let re = Regex::new(r"\[/?[a-zA-Z_][a-zA-Z0-9_-]*[^\]]*\]").unwrap();
        re.replace_all(content, "").to_string()
    }

    /// Get first paragraph
    fn get_first_paragraph(&self, content: &str) -> String {
        // Try to find paragraph tags
        let p_re = Regex::new(r"<p[^>]*>(.*?)</p>").unwrap();
        if let Some(caps) = p_re.captures(content) {
            return caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
        }

        // Fall back to first block of text
        content.split("\n\n")
            .next()
            .unwrap_or(content)
            .to_string()
    }

    /// Normalize whitespace
    fn normalize_whitespace(&self, content: &str) -> String {
        let re = Regex::new(r"\s+").unwrap();
        re.replace_all(content, " ").trim().to_string()
    }

    /// Truncate by word count
    fn truncate_by_words(&self, content: &str, max_words: usize) -> String {
        let words: Vec<&str> = content.split_whitespace().collect();
        if words.len() <= max_words {
            return content.to_string();
        }

        words[..max_words].join(" ")
    }

    /// Truncate by character count (word-boundary aware)
    fn truncate_by_chars(&self, content: &str, max_chars: usize) -> String {
        if content.chars().count() <= max_chars {
            return content.to_string();
        }

        // Find word boundary near max_chars
        let graphemes: Vec<&str> = content.graphemes(true).collect();
        let mut char_count = 0;
        let mut last_word_boundary = 0;

        for (i, g) in graphemes.iter().enumerate() {
            char_count += 1;
            if g.chars().all(|c| c.is_whitespace()) {
                last_word_boundary = i;
            }
            if char_count >= max_chars {
                break;
            }
        }

        if last_word_boundary > 0 {
            graphemes[..last_word_boundary].join("")
        } else {
            graphemes[..max_chars.min(graphemes.len())].join("")
        }
    }

    /// Count images in content
    fn count_images(&self, content: &str) -> usize {
        let img_re = Regex::new(r"<img[^>]*>").unwrap();
        img_re.find_iter(content).count()
    }

    /// Separate code blocks from regular content
    fn separate_code_blocks(&self, content: &str) -> (String, String) {
        // Use separate patterns for pre and code tags since Rust regex doesn't support backreferences
        let pre_re = Regex::new(r"<pre[^>]*>([\s\S]*?)</pre>").unwrap();
        let code_re = Regex::new(r"<code[^>]*>([\s\S]*?)</code>").unwrap();
        let mut code_content = String::new();

        // Extract pre blocks
        let content = pre_re.replace_all(content, |caps: &regex::Captures| {
            code_content.push_str(caps.get(1).map(|m| m.as_str()).unwrap_or(""));
            code_content.push(' ');
            "" // Remove from regular content
        }).to_string();

        // Extract code blocks
        let regular = code_re.replace_all(&content, |caps: &regex::Captures| {
            code_content.push_str(caps.get(1).map(|m| m.as_str()).unwrap_or(""));
            code_content.push(' ');
            "" // Remove from regular content
        }).to_string();

        (regular, code_content)
    }

    /// Count words in content
    pub fn word_count(&self, content: &str) -> usize {
        let clean = self.strip_html(&self.strip_shortcodes(content));
        clean.split_whitespace().count()
    }

    /// Count characters in content
    pub fn char_count(&self, content: &str) -> usize {
        let clean = self.strip_html(&self.strip_shortcodes(content));
        clean.chars().count()
    }

    /// Count sentences
    pub fn sentence_count(&self, content: &str) -> usize {
        let clean = self.strip_html(&self.strip_shortcodes(content));
        let re = Regex::new(r"[.!?]+\s+").unwrap();
        re.find_iter(&clean).count() + 1
    }

    /// Count paragraphs
    pub fn paragraph_count(&self, content: &str) -> usize {
        let p_re = Regex::new(r"<p[^>]*>").unwrap();
        let p_count = p_re.find_iter(content).count();

        if p_count > 0 {
            p_count
        } else {
            // Count by double newlines
            content.split("\n\n").filter(|s| !s.trim().is_empty()).count()
        }
    }

    /// Get content statistics
    pub fn get_stats(&self, content: &str) -> ContentStats {
        let reading_time = self.calculate_reading_time(content);

        ContentStats {
            word_count: reading_time.word_count,
            char_count: reading_time.char_count,
            sentence_count: self.sentence_count(content),
            paragraph_count: self.paragraph_count(content),
            image_count: reading_time.image_count,
            reading_time_minutes: reading_time.minutes,
            reading_time_display: reading_time.display,
        }
    }
}

/// Content statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentStats {
    pub word_count: usize,
    pub char_count: usize,
    pub sentence_count: usize,
    pub paragraph_count: usize,
    pub image_count: usize,
    pub reading_time_minutes: u32,
    pub reading_time_display: String,
}

/// Quick excerpt generation with defaults
pub fn generate_excerpt(content: &str, max_words: usize) -> String {
    let config = ExcerptConfig {
        max_words,
        ..Default::default()
    };
    ContentAnalyzer::new()
        .with_excerpt_config(config)
        .generate_excerpt(content)
}

/// Quick reading time calculation with defaults
pub fn calculate_reading_time(content: &str) -> ReadingTime {
    ContentAnalyzer::new().calculate_reading_time(content)
}

/// Quick word count
pub fn word_count(content: &str) -> usize {
    ContentAnalyzer::new().word_count(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_excerpt_generation() {
        let content = "This is a test paragraph with some words. It has multiple sentences and should be truncated properly.";
        let excerpt = generate_excerpt(content, 10);

        assert!(excerpt.len() < content.len());
        assert!(excerpt.ends_with("…"));
    }

    #[test]
    fn test_html_stripping() {
        let content = "<p>Hello <strong>World</strong>!</p>";
        let analyzer = ContentAnalyzer::new();
        let stripped = analyzer.strip_html(content);

        assert!(!stripped.contains('<'));
        assert!(stripped.contains("Hello"));
        assert!(stripped.contains("World"));
    }

    #[test]
    fn test_reading_time() {
        // ~200 words should be ~1 minute
        let content = "word ".repeat(200);
        let reading_time = calculate_reading_time(&content);

        assert_eq!(reading_time.minutes, 1);
        assert_eq!(reading_time.word_count, 200);
    }

    #[test]
    fn test_word_count() {
        let content = "One two three four five";
        assert_eq!(word_count(content), 5);
    }

    #[test]
    fn test_shortcode_stripping() {
        let content = "Hello [gallery ids=\"1,2,3\"/] World";
        let analyzer = ContentAnalyzer::new();
        let stripped = analyzer.strip_shortcodes(content);

        assert!(!stripped.contains("[gallery"));
        assert!(stripped.contains("Hello"));
        assert!(stripped.contains("World"));
    }
}
