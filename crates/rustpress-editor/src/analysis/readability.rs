//! Readability Analyzer
//!
//! Calculates readability scores using various formulas.

use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

/// Readability analyzer
#[derive(Debug, Clone)]
pub struct ReadabilityAnalyzer {
    config: ReadabilityConfig,
}

impl ReadabilityAnalyzer {
    pub fn new() -> Self {
        Self {
            config: ReadabilityConfig::default(),
        }
    }

    /// Analyze text readability
    pub fn analyze(&self, text: &str) -> ReadabilityResult {
        let stats = self.calculate_stats(text);

        let flesch_reading_ease = self.flesch_reading_ease(&stats);
        let flesch_kincaid_grade = self.flesch_kincaid_grade(&stats);
        let gunning_fog = self.gunning_fog_index(&stats);
        let smog_index = self.smog_index(&stats);
        let coleman_liau = self.coleman_liau_index(&stats);
        let automated_readability = self.automated_readability_index(&stats);

        // Average grade level
        let avg_grade = (flesch_kincaid_grade + gunning_fog + smog_index + coleman_liau + automated_readability) / 5.0;

        let issues = self.find_issues(&stats);
        let suggestions = self.generate_suggestions(&stats, &issues);

        ReadabilityResult {
            flesch_reading_ease,
            flesch_kincaid_grade,
            gunning_fog_index: gunning_fog,
            smog_index,
            coleman_liau_index: coleman_liau,
            automated_readability_index: automated_readability,
            average_grade_level: avg_grade,
            reading_level: ReadingLevel::from_grade(avg_grade),
            stats,
            issues,
            suggestions,
        }
    }

    fn calculate_stats(&self, text: &str) -> TextStats {
        let words: Vec<&str> = text.unicode_words().collect();
        let word_count = words.len();

        let sentences: Vec<&str> = text
            .split(|c| c == '.' || c == '!' || c == '?')
            .filter(|s| !s.trim().is_empty())
            .collect();
        let sentence_count = sentences.len().max(1);

        let syllable_count: usize = words.iter().map(|w| count_syllables(w)).sum();
        let complex_words = words.iter().filter(|w| count_syllables(w) >= 3).count();

        let character_count: usize = words.iter().map(|w| w.chars().count()).sum();

        let paragraphs: Vec<&str> = text
            .split("\n\n")
            .filter(|p| !p.trim().is_empty())
            .collect();

        // Find long sentences
        let long_sentences: Vec<String> = sentences
            .iter()
            .filter(|s| s.unicode_words().count() > self.config.max_sentence_length)
            .map(|s| {
                let trimmed = s.trim();
                if trimmed.len() > 100 {
                    format!("{}...", &trimmed[..100])
                } else {
                    trimmed.to_string()
                }
            })
            .collect();

        // Find long paragraphs
        let long_paragraphs = paragraphs
            .iter()
            .filter(|p| p.unicode_words().count() > self.config.max_paragraph_length)
            .count();

        TextStats {
            word_count,
            sentence_count,
            syllable_count,
            complex_word_count: complex_words,
            character_count,
            paragraph_count: paragraphs.len(),
            avg_words_per_sentence: word_count as f32 / sentence_count as f32,
            avg_syllables_per_word: syllable_count as f32 / word_count.max(1) as f32,
            avg_chars_per_word: character_count as f32 / word_count.max(1) as f32,
            complex_word_percentage: (complex_words as f32 / word_count.max(1) as f32) * 100.0,
            long_sentences,
            long_paragraph_count: long_paragraphs,
        }
    }

    /// Flesch Reading Ease (0-100, higher = easier)
    fn flesch_reading_ease(&self, stats: &TextStats) -> f32 {
        if stats.word_count == 0 || stats.sentence_count == 0 {
            return 0.0;
        }

        let asl = stats.avg_words_per_sentence;
        let asw = stats.avg_syllables_per_word;

        (206.835 - (1.015 * asl) - (84.6 * asw)).clamp(0.0, 100.0)
    }

    /// Flesch-Kincaid Grade Level
    fn flesch_kincaid_grade(&self, stats: &TextStats) -> f32 {
        if stats.word_count == 0 || stats.sentence_count == 0 {
            return 0.0;
        }

        let asl = stats.avg_words_per_sentence;
        let asw = stats.avg_syllables_per_word;

        (0.39 * asl + 11.8 * asw - 15.59).max(0.0)
    }

    /// Gunning Fog Index
    fn gunning_fog_index(&self, stats: &TextStats) -> f32 {
        if stats.word_count == 0 || stats.sentence_count == 0 {
            return 0.0;
        }

        let asl = stats.avg_words_per_sentence;
        let complex_percentage = stats.complex_word_percentage;

        0.4 * (asl + complex_percentage)
    }

    /// SMOG Index
    fn smog_index(&self, stats: &TextStats) -> f32 {
        if stats.sentence_count < 30 {
            // SMOG requires at least 30 sentences for accuracy
            // Use approximation for shorter texts
            return self.gunning_fog_index(stats) * 0.9;
        }

        let polysyllables = stats.complex_word_count as f32;
        let sentences = stats.sentence_count as f32;

        1.0430 * (polysyllables * (30.0 / sentences)).sqrt() + 3.1291
    }

    /// Coleman-Liau Index
    fn coleman_liau_index(&self, stats: &TextStats) -> f32 {
        if stats.word_count == 0 {
            return 0.0;
        }

        let l = (stats.character_count as f32 / stats.word_count as f32) * 100.0;
        let s = (stats.sentence_count as f32 / stats.word_count as f32) * 100.0;

        (0.0588 * l - 0.296 * s - 15.8).max(0.0)
    }

    /// Automated Readability Index
    fn automated_readability_index(&self, stats: &TextStats) -> f32 {
        if stats.word_count == 0 || stats.sentence_count == 0 {
            return 0.0;
        }

        let chars_per_word = stats.avg_chars_per_word;
        let words_per_sentence = stats.avg_words_per_sentence;

        (4.71 * chars_per_word + 0.5 * words_per_sentence - 21.43).max(0.0)
    }

    fn find_issues(&self, stats: &TextStats) -> Vec<ReadabilityIssue> {
        let mut issues = Vec::new();

        // Long sentences
        if !stats.long_sentences.is_empty() {
            issues.push(ReadabilityIssue {
                issue_type: IssueType::LongSentences,
                count: stats.long_sentences.len(),
                severity: IssueSeverity::Warning,
                description: format!(
                    "{} sentence(s) exceed {} words",
                    stats.long_sentences.len(),
                    self.config.max_sentence_length
                ),
                examples: stats.long_sentences.clone(),
            });
        }

        // Long paragraphs
        if stats.long_paragraph_count > 0 {
            issues.push(ReadabilityIssue {
                issue_type: IssueType::LongParagraphs,
                count: stats.long_paragraph_count,
                severity: IssueSeverity::Warning,
                description: format!(
                    "{} paragraph(s) exceed {} words",
                    stats.long_paragraph_count,
                    self.config.max_paragraph_length
                ),
                examples: Vec::new(),
            });
        }

        // Too many complex words
        if stats.complex_word_percentage > self.config.max_complex_word_percentage {
            issues.push(ReadabilityIssue {
                issue_type: IssueType::ComplexWords,
                count: stats.complex_word_count,
                severity: IssueSeverity::Info,
                description: format!(
                    "{:.1}% of words have 3+ syllables (aim for <{:.0}%)",
                    stats.complex_word_percentage,
                    self.config.max_complex_word_percentage
                ),
                examples: Vec::new(),
            });
        }

        // High average sentence length
        if stats.avg_words_per_sentence > self.config.target_sentence_length as f32 {
            issues.push(ReadabilityIssue {
                issue_type: IssueType::HighAverageSentenceLength,
                count: 1,
                severity: IssueSeverity::Info,
                description: format!(
                    "Average sentence length is {:.1} words (aim for ~{} words)",
                    stats.avg_words_per_sentence,
                    self.config.target_sentence_length
                ),
                examples: Vec::new(),
            });
        }

        issues
    }

    fn generate_suggestions(&self, stats: &TextStats, issues: &[ReadabilityIssue]) -> Vec<String> {
        let mut suggestions = Vec::new();

        for issue in issues {
            match issue.issue_type {
                IssueType::LongSentences => {
                    suggestions.push("Break long sentences into shorter ones for better readability.".to_string());
                }
                IssueType::LongParagraphs => {
                    suggestions.push("Split long paragraphs to make content easier to scan.".to_string());
                }
                IssueType::ComplexWords => {
                    suggestions.push("Consider using simpler words where possible.".to_string());
                }
                IssueType::HighAverageSentenceLength => {
                    suggestions.push("Vary sentence length and include more short sentences.".to_string());
                }
                IssueType::PassiveVoice => {
                    suggestions.push("Use active voice more often for clearer writing.".to_string());
                }
            }
        }

        // Add general suggestions based on stats
        if stats.word_count < 300 {
            suggestions.push("Consider adding more content for better depth.".to_string());
        }

        if stats.paragraph_count < 3 && stats.word_count > 200 {
            suggestions.push("Add paragraph breaks to improve visual structure.".to_string());
        }

        suggestions
    }
}

impl Default for ReadabilityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for readability analysis
#[derive(Debug, Clone)]
pub struct ReadabilityConfig {
    pub max_sentence_length: usize,
    pub max_paragraph_length: usize,
    pub target_sentence_length: usize,
    pub max_complex_word_percentage: f32,
}

impl Default for ReadabilityConfig {
    fn default() -> Self {
        Self {
            max_sentence_length: 25,
            max_paragraph_length: 150,
            target_sentence_length: 15,
            max_complex_word_percentage: 15.0,
        }
    }
}

/// Text statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStats {
    pub word_count: usize,
    pub sentence_count: usize,
    pub syllable_count: usize,
    pub complex_word_count: usize,
    pub character_count: usize,
    pub paragraph_count: usize,
    pub avg_words_per_sentence: f32,
    pub avg_syllables_per_word: f32,
    pub avg_chars_per_word: f32,
    pub complex_word_percentage: f32,
    pub long_sentences: Vec<String>,
    pub long_paragraph_count: usize,
}

/// Readability analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadabilityResult {
    /// Flesch Reading Ease (0-100)
    pub flesch_reading_ease: f32,
    /// Flesch-Kincaid Grade Level
    pub flesch_kincaid_grade: f32,
    /// Gunning Fog Index
    pub gunning_fog_index: f32,
    /// SMOG Index
    pub smog_index: f32,
    /// Coleman-Liau Index
    pub coleman_liau_index: f32,
    /// Automated Readability Index
    pub automated_readability_index: f32,
    /// Average grade level across all metrics
    pub average_grade_level: f32,
    /// Human-readable reading level
    pub reading_level: ReadingLevel,
    /// Text statistics
    pub stats: TextStats,
    /// Issues found
    pub issues: Vec<ReadabilityIssue>,
    /// Improvement suggestions
    pub suggestions: Vec<String>,
}

impl ReadabilityResult {
    /// Get a score out of 100 (higher = better readability)
    pub fn score(&self) -> u32 {
        self.flesch_reading_ease as u32
    }

    /// Get grade for display
    pub fn grade(&self) -> ReadabilityGrade {
        ReadabilityGrade::from_flesch(self.flesch_reading_ease)
    }
}

/// Reading level descriptions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadingLevel {
    Elementary,      // Grade 1-5
    MiddleSchool,    // Grade 6-8
    HighSchool,      // Grade 9-12
    College,         // Grade 13-16
    Graduate,        // Grade 17+
}

impl ReadingLevel {
    pub fn from_grade(grade: f32) -> Self {
        match grade as u32 {
            0..=5 => Self::Elementary,
            6..=8 => Self::MiddleSchool,
            9..=12 => Self::HighSchool,
            13..=16 => Self::College,
            _ => Self::Graduate,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Elementary => "Elementary School",
            Self::MiddleSchool => "Middle School",
            Self::HighSchool => "High School",
            Self::College => "College",
            Self::Graduate => "Graduate",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Elementary => "Very easy to read. Easily understood by an average 11-year-old student.",
            Self::MiddleSchool => "Fairly easy to read. Conversational English for consumers.",
            Self::HighSchool => "Standard difficulty. Suitable for most readers.",
            Self::College => "Fairly difficult to read. Best understood by college graduates.",
            Self::Graduate => "Difficult to read. Best understood by university graduates.",
        }
    }
}

/// Readability grade
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReadabilityGrade {
    VeryEasy,
    Easy,
    FairlyEasy,
    Standard,
    FairlyDifficult,
    Difficult,
    VeryDifficult,
}

impl ReadabilityGrade {
    pub fn from_flesch(score: f32) -> Self {
        match score as u32 {
            90..=100 => Self::VeryEasy,
            80..=89 => Self::Easy,
            70..=79 => Self::FairlyEasy,
            60..=69 => Self::Standard,
            50..=59 => Self::FairlyDifficult,
            30..=49 => Self::Difficult,
            _ => Self::VeryDifficult,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::VeryEasy => "Very Easy",
            Self::Easy => "Easy",
            Self::FairlyEasy => "Fairly Easy",
            Self::Standard => "Standard",
            Self::FairlyDifficult => "Fairly Difficult",
            Self::Difficult => "Difficult",
            Self::VeryDifficult => "Very Difficult",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            Self::VeryEasy | Self::Easy => "#10b981",
            Self::FairlyEasy | Self::Standard => "#22c55e",
            Self::FairlyDifficult => "#f59e0b",
            Self::Difficult => "#f97316",
            Self::VeryDifficult => "#ef4444",
        }
    }
}

/// Readability issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadabilityIssue {
    pub issue_type: IssueType,
    pub count: usize,
    pub severity: IssueSeverity,
    pub description: String,
    pub examples: Vec<String>,
}

/// Issue types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueType {
    LongSentences,
    LongParagraphs,
    ComplexWords,
    HighAverageSentenceLength,
    PassiveVoice,
}

/// Issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

/// Count syllables in a word (English approximation)
fn count_syllables(word: &str) -> usize {
    let word = word.to_lowercase();
    let word = word.trim_matches(|c: char| !c.is_alphabetic());

    if word.is_empty() {
        return 0;
    }

    if word.len() <= 3 {
        return 1;
    }

    let vowels = ['a', 'e', 'i', 'o', 'u', 'y'];
    let mut count = 0;
    let mut prev_was_vowel = false;
    let chars: Vec<char> = word.chars().collect();

    for (i, &c) in chars.iter().enumerate() {
        let is_vowel = vowels.contains(&c);

        if is_vowel && !prev_was_vowel {
            count += 1;
        }
        prev_was_vowel = is_vowel;

        // Silent 'e' at end
        if i == chars.len() - 1 && c == 'e' && count > 1 {
            count -= 1;
        }
    }

    count.max(1)
}
