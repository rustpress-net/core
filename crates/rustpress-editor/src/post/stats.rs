//! Post Statistics
//!
//! Word count, reading time, and content analysis.

use crate::post::PostContent;
use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

/// Post content statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PostStats {
    /// Total word count
    pub word_count: u32,

    /// Total character count (with spaces)
    pub character_count: u32,

    /// Character count without spaces
    pub character_count_no_spaces: u32,

    /// Sentence count
    pub sentence_count: u32,

    /// Paragraph count
    pub paragraph_count: u32,

    /// Total block count
    pub block_count: u32,

    /// Image count
    pub image_count: u32,

    /// Video count
    pub video_count: u32,

    /// Link count
    pub link_count: u32,

    /// Heading count by level
    pub heading_counts: HeadingCounts,

    /// Estimated reading time in minutes
    pub reading_time_minutes: u32,

    /// Estimated speaking time in minutes
    pub speaking_time_minutes: u32,

    /// Average word length
    pub avg_word_length: f32,

    /// Average sentence length (words per sentence)
    pub avg_sentence_length: f32,

    /// Average paragraph length (words per paragraph)
    pub avg_paragraph_length: f32,

    /// Most used words (top 10)
    #[serde(default)]
    pub top_words: Vec<WordFrequency>,

    /// Content density metrics
    pub density: ContentDensity,
}

impl PostStats {
    /// Calculate stats from post content
    pub fn calculate(content: &PostContent) -> Self {
        let text = content.get_plain_text();
        let words: Vec<&str> = text.unicode_words().collect();
        let word_count = words.len() as u32;

        let sentences: Vec<&str> = text
            .split(|c| c == '.' || c == '!' || c == '?')
            .filter(|s| !s.trim().is_empty())
            .collect();
        let sentence_count = sentences.len() as u32;

        let paragraphs: Vec<&str> = text
            .split("\n\n")
            .filter(|p| !p.trim().is_empty())
            .collect();
        let paragraph_count = paragraphs.len().max(1) as u32;

        let character_count = text.chars().count() as u32;
        let character_count_no_spaces = text.chars().filter(|c| !c.is_whitespace()).count() as u32;

        let block_count = content.count_blocks() as u32;

        // Count by block type
        let mut image_count = 0u32;
        let mut video_count = 0u32;
        let mut heading_counts = HeadingCounts::default();

        fn count_blocks(
            blocks: &[crate::blocks::Block],
            images: &mut u32,
            videos: &mut u32,
            headings: &mut HeadingCounts,
        ) {
            use crate::blocks::BlockType;

            for block in blocks {
                match block.block_type {
                    BlockType::Image | BlockType::Gallery | BlockType::Cover => *images += 1,
                    BlockType::Video | BlockType::Embed => *videos += 1,
                    BlockType::Heading => {
                        if let Some(level) = block.attributes.level {
                            match level {
                                1 => headings.h1 += 1,
                                2 => headings.h2 += 1,
                                3 => headings.h3 += 1,
                                4 => headings.h4 += 1,
                                5 => headings.h5 += 1,
                                6 => headings.h6 += 1,
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
                count_blocks(&block.children, images, videos, headings);
            }
        }

        count_blocks(
            &content.blocks,
            &mut image_count,
            &mut video_count,
            &mut heading_counts,
        );

        // Count links in text (simplified)
        let link_count = text.matches("http").count() as u32;

        // Reading time calculation
        // Average reading speed: 200-250 words per minute
        let reading_time_minutes = ((word_count as f32) / 225.0).ceil() as u32;

        // Speaking time: ~150 words per minute
        let speaking_time_minutes = ((word_count as f32) / 150.0).ceil() as u32;

        // Average calculations
        let avg_word_length = if word_count > 0 {
            words.iter().map(|w| w.chars().count()).sum::<usize>() as f32 / word_count as f32
        } else {
            0.0
        };

        let avg_sentence_length = if sentence_count > 0 {
            word_count as f32 / sentence_count as f32
        } else {
            0.0
        };

        let avg_paragraph_length = if paragraph_count > 0 {
            word_count as f32 / paragraph_count as f32
        } else {
            0.0
        };

        // Word frequency analysis
        let top_words = Self::calculate_word_frequency(&words);

        // Content density
        let density = ContentDensity {
            text_percentage: if block_count > 0 {
                let text_blocks = block_count - image_count - video_count;
                (text_blocks as f32 / block_count as f32) * 100.0
            } else {
                0.0
            },
            media_percentage: if block_count > 0 {
                ((image_count + video_count) as f32 / block_count as f32) * 100.0
            } else {
                0.0
            },
            headings_per_1000_words: if word_count > 0 {
                (heading_counts.total() as f32 / word_count as f32) * 1000.0
            } else {
                0.0
            },
            images_per_1000_words: if word_count > 0 {
                (image_count as f32 / word_count as f32) * 1000.0
            } else {
                0.0
            },
            links_per_1000_words: if word_count > 0 {
                (link_count as f32 / word_count as f32) * 1000.0
            } else {
                0.0
            },
        };

        Self {
            word_count,
            character_count,
            character_count_no_spaces,
            sentence_count,
            paragraph_count,
            block_count,
            image_count,
            video_count,
            link_count,
            heading_counts,
            reading_time_minutes,
            speaking_time_minutes,
            avg_word_length,
            avg_sentence_length,
            avg_paragraph_length,
            top_words,
            density,
        }
    }

    /// Calculate word frequency
    fn calculate_word_frequency(words: &[&str]) -> Vec<WordFrequency> {
        use std::collections::HashMap;

        // Common stop words to exclude
        const STOP_WORDS: &[&str] = &[
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "by", "from", "as", "is", "was", "are", "were", "been", "be", "have", "has", "had",
            "do", "does", "did", "will", "would", "could", "should", "may", "might", "must",
            "can", "this", "that", "these", "those", "it", "its", "they", "them", "their",
            "we", "us", "our", "you", "your", "he", "she", "him", "her", "his", "i", "my",
            "me", "not", "no", "if", "then", "else", "when", "where", "what", "which", "who",
            "how", "why", "all", "any", "both", "each", "few", "more", "most", "other", "some",
            "such", "than", "too", "very", "just", "only", "so", "also", "about", "into",
        ];

        let mut frequency: HashMap<String, u32> = HashMap::new();

        for word in words {
            let lower = word.to_lowercase();
            if lower.len() >= 3 && !STOP_WORDS.contains(&lower.as_str()) {
                *frequency.entry(lower).or_insert(0) += 1;
            }
        }

        let mut sorted: Vec<_> = frequency.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        sorted
            .into_iter()
            .take(10)
            .map(|(word, count)| WordFrequency { word, count })
            .collect()
    }

    /// Format reading time for display
    pub fn format_reading_time(&self) -> String {
        if self.reading_time_minutes == 0 {
            "< 1 min read".to_string()
        } else if self.reading_time_minutes == 1 {
            "1 min read".to_string()
        } else {
            format!("{} min read", self.reading_time_minutes)
        }
    }

    /// Get content score (basic quality metric)
    pub fn get_content_score(&self) -> ContentScore {
        let mut score = 0;
        let mut issues = Vec::new();

        // Word count scoring
        if self.word_count >= 300 {
            score += 20;
        } else {
            issues.push("Content is too short (< 300 words)".to_string());
        }

        if self.word_count >= 1000 {
            score += 10;
        }

        // Heading structure
        if self.heading_counts.h1 <= 1 && self.heading_counts.total() >= 2 {
            score += 15;
        } else if self.heading_counts.h1 > 1 {
            issues.push("Multiple H1 headings found".to_string());
        } else if self.heading_counts.total() < 2 {
            issues.push("Add more headings to structure content".to_string());
        }

        // Images
        if self.image_count >= 1 {
            score += 15;
        } else {
            issues.push("Add at least one image".to_string());
        }

        // Sentence length (ideal: 15-20 words)
        if self.avg_sentence_length >= 10.0 && self.avg_sentence_length <= 25.0 {
            score += 15;
        } else if self.avg_sentence_length > 25.0 {
            issues.push("Sentences are too long on average".to_string());
        }

        // Paragraph length
        if self.avg_paragraph_length <= 150.0 {
            score += 15;
        } else {
            issues.push("Paragraphs are too long".to_string());
        }

        // Links
        if self.link_count >= 2 {
            score += 10;
        }

        ContentScore {
            score: score.min(100),
            issues,
            grade: match score {
                90..=100 => ContentGrade::Excellent,
                75..=89 => ContentGrade::Good,
                60..=74 => ContentGrade::Fair,
                40..=59 => ContentGrade::NeedsWork,
                _ => ContentGrade::Poor,
            },
        }
    }
}

/// Heading counts by level
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HeadingCounts {
    pub h1: u32,
    pub h2: u32,
    pub h3: u32,
    pub h4: u32,
    pub h5: u32,
    pub h6: u32,
}

impl HeadingCounts {
    pub fn total(&self) -> u32 {
        self.h1 + self.h2 + self.h3 + self.h4 + self.h5 + self.h6
    }
}

/// Word frequency entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordFrequency {
    pub word: String,
    pub count: u32,
}

/// Content density metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContentDensity {
    /// Percentage of text blocks
    pub text_percentage: f32,
    /// Percentage of media blocks
    pub media_percentage: f32,
    /// Headings per 1000 words
    pub headings_per_1000_words: f32,
    /// Images per 1000 words
    pub images_per_1000_words: f32,
    /// Links per 1000 words
    pub links_per_1000_words: f32,
}

/// Content quality score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentScore {
    pub score: u32,
    pub issues: Vec<String>,
    pub grade: ContentGrade,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentGrade {
    Excellent,
    Good,
    Fair,
    NeedsWork,
    Poor,
}

impl ContentGrade {
    pub fn color(&self) -> &'static str {
        match self {
            Self::Excellent => "#10b981", // green
            Self::Good => "#22c55e",      // lime
            Self::Fair => "#f59e0b",      // amber
            Self::NeedsWork => "#f97316", // orange
            Self::Poor => "#ef4444",      // red
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Excellent => "Excellent",
            Self::Good => "Good",
            Self::Fair => "Fair",
            Self::NeedsWork => "Needs Work",
            Self::Poor => "Poor",
        }
    }
}
