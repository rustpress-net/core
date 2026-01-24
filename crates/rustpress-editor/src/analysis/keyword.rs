//! Keyword Analyzer
//!
//! Analyzes keyword usage and density in content.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;

/// Keyword analyzer for content optimization
#[derive(Debug, Clone)]
pub struct KeywordAnalyzer {
    stop_words: Vec<String>,
    config: KeywordConfig,
}

impl KeywordAnalyzer {
    pub fn new() -> Self {
        Self {
            stop_words: default_stop_words(),
            config: KeywordConfig::default(),
        }
    }

    /// Analyze keywords in text
    pub fn analyze(&self, text: &str, focus_keyword: Option<&str>) -> KeywordAnalysis {
        let words: Vec<&str> = text.unicode_words().collect();
        let word_count = words.len();

        // Extract and count all words
        let word_frequencies = self.extract_word_frequencies(&words);

        // Extract n-grams
        let bigrams = self.extract_ngrams(&words, 2);
        let trigrams = self.extract_ngrams(&words, 3);

        // Get top keywords
        let top_keywords = self.get_top_keywords(&word_frequencies, 20);
        let top_phrases = self.get_top_phrases(&bigrams, &trigrams, 10);

        // Analyze focus keyword if provided
        let focus_analysis = focus_keyword.map(|kw| {
            self.analyze_focus_keyword(kw, text, &words, word_count)
        });

        // Generate suggestions
        let suggestions = self.generate_suggestions(&top_keywords, focus_analysis.as_ref());

        KeywordAnalysis {
            word_count,
            unique_word_count: word_frequencies.len(),
            top_keywords,
            top_phrases,
            focus_keyword_analysis: focus_analysis,
            keyword_density_map: self.calculate_density_map(&word_frequencies, word_count),
            suggestions,
        }
    }

    fn extract_word_frequencies(&self, words: &[&str]) -> HashMap<String, u32> {
        let mut frequencies: HashMap<String, u32> = HashMap::new();

        for word in words {
            let normalized = word.to_lowercase();

            // Skip stop words and short words
            if normalized.len() < self.config.min_word_length
                || self.stop_words.contains(&normalized)
            {
                continue;
            }

            *frequencies.entry(normalized).or_insert(0) += 1;
        }

        frequencies
    }

    fn extract_ngrams(&self, words: &[&str], n: usize) -> HashMap<String, u32> {
        let mut ngrams: HashMap<String, u32> = HashMap::new();

        if words.len() < n {
            return ngrams;
        }

        for window in words.windows(n) {
            let phrase: Vec<String> = window.iter()
                .map(|w| w.to_lowercase())
                .collect();

            // Skip if any word is a stop word at start/end
            if self.stop_words.contains(&phrase[0]) || self.stop_words.contains(&phrase[n - 1]) {
                continue;
            }

            let phrase_str = phrase.join(" ");
            *ngrams.entry(phrase_str).or_insert(0) += 1;
        }

        ngrams
    }

    fn get_top_keywords(&self, frequencies: &HashMap<String, u32>, limit: usize) -> Vec<KeywordEntry> {
        let mut sorted: Vec<_> = frequencies.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));

        sorted
            .into_iter()
            .take(limit)
            .map(|(word, count)| KeywordEntry {
                keyword: word.clone(),
                count: *count,
                keyword_type: KeywordType::SingleWord,
            })
            .collect()
    }

    fn get_top_phrases(
        &self,
        bigrams: &HashMap<String, u32>,
        trigrams: &HashMap<String, u32>,
        limit: usize,
    ) -> Vec<KeywordEntry> {
        let mut all_phrases: Vec<_> = bigrams
            .iter()
            .filter(|(_, &count)| count >= self.config.min_phrase_frequency)
            .map(|(phrase, &count)| KeywordEntry {
                keyword: phrase.clone(),
                count,
                keyword_type: KeywordType::Bigram,
            })
            .chain(
                trigrams
                    .iter()
                    .filter(|(_, &count)| count >= self.config.min_phrase_frequency)
                    .map(|(phrase, &count)| KeywordEntry {
                        keyword: phrase.clone(),
                        count,
                        keyword_type: KeywordType::Trigram,
                    })
            )
            .collect();

        all_phrases.sort_by(|a, b| b.count.cmp(&a.count));
        all_phrases.into_iter().take(limit).collect()
    }

    fn analyze_focus_keyword(
        &self,
        keyword: &str,
        text: &str,
        _words: &[&str],
        word_count: usize,
    ) -> FocusKeywordAnalysis {
        let keyword_lower = keyword.to_lowercase();
        let text_lower = text.to_lowercase();

        // Count occurrences
        let occurrences = text_lower.matches(&keyword_lower).count() as u32;

        // Calculate density
        let keyword_words: Vec<&str> = keyword.unicode_words().collect();
        let keyword_word_count = keyword_words.len();
        let density = if word_count > 0 {
            (occurrences as f32 * keyword_word_count as f32 / word_count as f32) * 100.0
        } else {
            0.0
        };

        // Find positions
        let positions: Vec<usize> = text_lower
            .match_indices(&keyword_lower)
            .map(|(pos, _)| pos)
            .collect();

        // Check if in first paragraph
        let first_para_end = text.find("\n\n").unwrap_or(text.len().min(500));
        let in_first_paragraph = positions.first().map(|&p| p < first_para_end).unwrap_or(false);

        // Check distribution (divided into quarters)
        let text_len = text.len();
        let quarter = text_len / 4;
        let mut distribution = [0u32; 4];
        for pos in &positions {
            let section = (*pos / quarter.max(1)).min(3);
            distribution[section] += 1;
        }

        // Assess density
        let density_status = if density < 0.5 {
            DensityStatus::TooLow
        } else if density > 2.5 {
            DensityStatus::TooHigh
        } else {
            DensityStatus::Good
        };

        // Check prominence (appears early and often)
        let prominence_score = if in_first_paragraph && occurrences >= 3 {
            100
        } else if in_first_paragraph || occurrences >= 3 {
            70
        } else if occurrences >= 1 {
            40
        } else {
            0
        };

        FocusKeywordAnalysis {
            keyword: keyword.to_string(),
            occurrences,
            density,
            density_status,
            positions,
            in_first_paragraph,
            distribution,
            prominence_score,
            suggestions: self.focus_keyword_suggestions(&keyword_lower, occurrences, density, in_first_paragraph),
        }
    }

    fn focus_keyword_suggestions(
        &self,
        keyword: &str,
        occurrences: u32,
        density: f32,
        in_first_paragraph: bool,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        if occurrences == 0 {
            suggestions.push(format!("Add the focus keyword \"{}\" to your content.", keyword));
        }

        if !in_first_paragraph && occurrences > 0 {
            suggestions.push("Add the focus keyword to your first paragraph.".to_string());
        }

        if density < 0.5 {
            suggestions.push("Increase keyword usage slightly for better SEO.".to_string());
        } else if density > 2.5 {
            suggestions.push("Reduce keyword repetition to avoid keyword stuffing.".to_string());
        }

        suggestions
    }

    fn calculate_density_map(&self, frequencies: &HashMap<String, u32>, word_count: usize) -> HashMap<String, f32> {
        frequencies
            .iter()
            .map(|(word, count)| {
                let density = (*count as f32 / word_count.max(1) as f32) * 100.0;
                (word.clone(), density)
            })
            .collect()
    }

    fn generate_suggestions(
        &self,
        top_keywords: &[KeywordEntry],
        focus_analysis: Option<&FocusKeywordAnalysis>,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Check for focus keyword in top keywords
        if let Some(focus) = focus_analysis {
            let focus_lower = focus.keyword.to_lowercase();
            let is_top_keyword = top_keywords
                .iter()
                .take(5)
                .any(|k| k.keyword == focus_lower);

            if !is_top_keyword && focus.occurrences > 0 {
                suggestions.push(
                    "Your focus keyword is not among the most frequent words. Consider using it more naturally.".to_string()
                );
            }

            suggestions.extend(focus.suggestions.clone());
        }

        // General keyword suggestions
        if top_keywords.len() < 5 {
            suggestions.push("Content may lack clear topic focus. Consider using related keywords.".to_string());
        }

        suggestions
    }
}

impl Default for KeywordAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Keyword analyzer configuration
#[derive(Debug, Clone)]
pub struct KeywordConfig {
    pub min_word_length: usize,
    pub min_phrase_frequency: u32,
}

impl Default for KeywordConfig {
    fn default() -> Self {
        Self {
            min_word_length: 3,
            min_phrase_frequency: 2,
        }
    }
}

/// Keyword analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordAnalysis {
    pub word_count: usize,
    pub unique_word_count: usize,
    pub top_keywords: Vec<KeywordEntry>,
    pub top_phrases: Vec<KeywordEntry>,
    pub focus_keyword_analysis: Option<FocusKeywordAnalysis>,
    pub keyword_density_map: HashMap<String, f32>,
    pub suggestions: Vec<String>,
}

/// Keyword entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordEntry {
    pub keyword: String,
    pub count: u32,
    pub keyword_type: KeywordType,
}

/// Keyword type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeywordType {
    SingleWord,
    Bigram,
    Trigram,
}

/// Focus keyword analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusKeywordAnalysis {
    pub keyword: String,
    pub occurrences: u32,
    pub density: f32,
    pub density_status: DensityStatus,
    pub positions: Vec<usize>,
    pub in_first_paragraph: bool,
    pub distribution: [u32; 4],
    pub prominence_score: u32,
    pub suggestions: Vec<String>,
}

/// Keyword density status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DensityStatus {
    TooLow,
    Good,
    TooHigh,
}

impl DensityStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::TooLow => "Too Low",
            Self::Good => "Good",
            Self::TooHigh => "Too High",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            Self::TooLow => "#f97316",
            Self::Good => "#10b981",
            Self::TooHigh => "#ef4444",
        }
    }
}

/// Default English stop words
fn default_stop_words() -> Vec<String> {
    vec![
        "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
        "by", "from", "as", "is", "was", "are", "were", "been", "be", "have", "has", "had",
        "do", "does", "did", "will", "would", "could", "should", "may", "might", "must",
        "can", "this", "that", "these", "those", "it", "its", "they", "them", "their",
        "we", "us", "our", "you", "your", "he", "she", "him", "her", "his", "i", "my",
        "me", "not", "no", "if", "then", "else", "when", "where", "what", "which", "who",
        "how", "why", "all", "any", "both", "each", "few", "more", "most", "other", "some",
        "such", "than", "too", "very", "just", "only", "so", "also", "about", "into",
        "through", "during", "before", "after", "above", "below", "between", "under",
        "again", "further", "once", "here", "there", "own", "same", "being", "having",
        "doing", "while", "until", "because", "although", "though", "since", "unless",
    ].into_iter().map(String::from).collect()
}
