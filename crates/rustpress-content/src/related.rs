//! # Related Posts Algorithm
//!
//! Intelligent related content discovery based on multiple signals.
//!
//! Features:
//! - Tag/category matching
//! - Content similarity scoring
//! - Keyword extraction
//! - Recency weighting
//! - Configurable scoring

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Related post scoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedConfig {
    /// Weight for shared categories
    pub category_weight: f64,

    /// Weight for shared tags
    pub tag_weight: f64,

    /// Weight for content similarity
    pub content_weight: f64,

    /// Weight for recency
    pub recency_weight: f64,

    /// Weight for author match
    pub author_weight: f64,

    /// Weight for custom taxonomy matches
    pub taxonomy_weight: f64,

    /// Maximum age in days for recency scoring
    pub max_age_days: u32,

    /// Minimum score threshold for inclusion
    pub min_score: f64,

    /// Maximum number of related posts to return
    pub max_results: usize,

    /// Whether to exclude same author posts
    pub exclude_same_author: bool,

    /// Boost for same post type
    pub same_type_boost: f64,
}

impl Default for RelatedConfig {
    fn default() -> Self {
        Self {
            category_weight: 0.3,
            tag_weight: 0.25,
            content_weight: 0.2,
            recency_weight: 0.15,
            author_weight: 0.05,
            taxonomy_weight: 0.05,
            max_age_days: 365,
            min_score: 0.1,
            max_results: 5,
            exclude_same_author: false,
            same_type_boost: 1.2,
        }
    }
}

/// Post data for related post calculation
#[derive(Debug, Clone)]
pub struct PostData {
    pub id: i64,
    pub post_type: String,
    pub title: String,
    pub content: String,
    pub excerpt: String,
    pub author_id: i64,
    pub categories: Vec<i64>,
    pub tags: Vec<i64>,
    pub taxonomies: HashMap<String, Vec<i64>>,
    pub published_at: DateTime<Utc>,
    pub keywords: Vec<String>,
}

impl PostData {
    pub fn new(id: i64, title: &str, content: &str) -> Self {
        Self {
            id,
            post_type: "post".to_string(),
            title: title.to_string(),
            content: content.to_string(),
            excerpt: String::new(),
            author_id: 0,
            categories: Vec::new(),
            tags: Vec::new(),
            taxonomies: HashMap::new(),
            published_at: Utc::now(),
            keywords: Vec::new(),
        }
    }

    pub fn with_categories(mut self, categories: Vec<i64>) -> Self {
        self.categories = categories;
        self
    }

    pub fn with_tags(mut self, tags: Vec<i64>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_author(mut self, author_id: i64) -> Self {
        self.author_id = author_id;
        self
    }

    pub fn with_published(mut self, date: DateTime<Utc>) -> Self {
        self.published_at = date;
        self
    }
}

/// Scored related post
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredPost {
    pub post_id: i64,
    pub score: f64,
    pub category_score: f64,
    pub tag_score: f64,
    pub content_score: f64,
    pub recency_score: f64,
    pub author_score: f64,
    pub shared_categories: Vec<i64>,
    pub shared_tags: Vec<i64>,
}

impl ScoredPost {
    pub fn new(post_id: i64) -> Self {
        Self {
            post_id,
            score: 0.0,
            category_score: 0.0,
            tag_score: 0.0,
            content_score: 0.0,
            recency_score: 0.0,
            author_score: 0.0,
            shared_categories: Vec::new(),
            shared_tags: Vec::new(),
        }
    }

    /// Calculate total score from components
    pub fn calculate_total(&mut self, config: &RelatedConfig) {
        self.score = self.category_score * config.category_weight
            + self.tag_score * config.tag_weight
            + self.content_score * config.content_weight
            + self.recency_score * config.recency_weight
            + self.author_score * config.author_weight;
    }
}

/// Related posts finder
pub struct RelatedFinder {
    config: RelatedConfig,
}

impl Default for RelatedFinder {
    fn default() -> Self {
        Self::new(RelatedConfig::default())
    }
}

impl RelatedFinder {
    pub fn new(config: RelatedConfig) -> Self {
        Self { config }
    }

    /// Find related posts
    pub fn find_related(&self, source: &PostData, candidates: &[PostData]) -> Vec<ScoredPost> {
        let mut scored: Vec<ScoredPost> = candidates
            .iter()
            .filter(|p| p.id != source.id)
            .filter(|p| !self.config.exclude_same_author || p.author_id != source.author_id)
            .map(|candidate| self.score_post(source, candidate))
            .filter(|s| s.score >= self.config.min_score)
            .collect();

        // Sort by score descending
        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Limit results
        scored.truncate(self.config.max_results);

        scored
    }

    /// Score a single candidate post
    fn score_post(&self, source: &PostData, candidate: &PostData) -> ScoredPost {
        let mut scored = ScoredPost::new(candidate.id);

        // Category score
        scored.shared_categories = self.find_shared(&source.categories, &candidate.categories);
        scored.category_score = self.calculate_jaccard(
            &source.categories,
            &candidate.categories,
        );

        // Tag score
        scored.shared_tags = self.find_shared(&source.tags, &candidate.tags);
        scored.tag_score = self.calculate_jaccard(&source.tags, &candidate.tags);

        // Content similarity (keyword-based)
        scored.content_score = self.calculate_content_similarity(source, candidate);

        // Recency score
        scored.recency_score = self.calculate_recency_score(candidate.published_at);

        // Author score (bonus for same author)
        scored.author_score = if source.author_id == candidate.author_id { 1.0 } else { 0.0 };

        // Calculate total
        scored.calculate_total(&self.config);

        // Apply same-type boost
        if source.post_type == candidate.post_type {
            scored.score *= self.config.same_type_boost;
        }

        scored
    }

    /// Find shared items between two lists
    fn find_shared<T: Clone + Eq + std::hash::Hash>(&self, a: &[T], b: &[T]) -> Vec<T> {
        let set_a: HashSet<&T> = a.iter().collect();
        b.iter()
            .filter(|item| set_a.contains(item))
            .cloned()
            .collect()
    }

    /// Calculate Jaccard similarity coefficient
    fn calculate_jaccard<T: Eq + std::hash::Hash>(&self, a: &[T], b: &[T]) -> f64 {
        if a.is_empty() && b.is_empty() {
            return 0.0;
        }

        let set_a: HashSet<&T> = a.iter().collect();
        let set_b: HashSet<&T> = b.iter().collect();

        let intersection = set_a.intersection(&set_b).count();
        let union = set_a.union(&set_b).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Calculate content similarity using keywords
    fn calculate_content_similarity(&self, source: &PostData, candidate: &PostData) -> f64 {
        // Extract keywords if not pre-computed
        let source_keywords = if source.keywords.is_empty() {
            self.extract_keywords(&source.title, &source.content)
        } else {
            source.keywords.clone()
        };

        let candidate_keywords = if candidate.keywords.is_empty() {
            self.extract_keywords(&candidate.title, &candidate.content)
        } else {
            candidate.keywords.clone()
        };

        // Calculate similarity
        let source_set: HashSet<&str> = source_keywords.iter().map(|s| s.as_str()).collect();
        let candidate_set: HashSet<&str> = candidate_keywords.iter().map(|s| s.as_str()).collect();

        let intersection = source_set.intersection(&candidate_set).count();
        let union = source_set.union(&candidate_set).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Calculate recency score (newer = higher)
    fn calculate_recency_score(&self, published_at: DateTime<Utc>) -> f64 {
        let age = Utc::now().signed_duration_since(published_at);
        let age_days = age.num_days() as f64;

        if age_days <= 0.0 {
            return 1.0;
        }

        let max_days = self.config.max_age_days as f64;
        if age_days >= max_days {
            return 0.0;
        }

        // Exponential decay
        (-age_days / max_days * 3.0).exp()
    }

    /// Extract keywords from text
    fn extract_keywords(&self, title: &str, content: &str) -> Vec<String> {
        let mut keywords = Vec::new();

        // Combine title and content
        let text = format!("{} {}", title, content).to_lowercase();

        // Simple keyword extraction - split and filter
        let stopwords = self.get_stopwords();

        for word in text.split_whitespace() {
            // Clean word
            let clean: String = word.chars()
                .filter(|c| c.is_alphanumeric())
                .collect();

            if clean.len() >= 3 && !stopwords.contains(&clean.as_str()) {
                if !keywords.contains(&clean) {
                    keywords.push(clean);
                }
            }
        }

        // Limit to top keywords
        keywords.truncate(50);
        keywords
    }

    /// Get stopwords to filter out
    fn get_stopwords(&self) -> HashSet<&'static str> {
        [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
            "of", "with", "by", "from", "up", "about", "into", "through", "during",
            "before", "after", "above", "below", "between", "under", "again",
            "further", "then", "once", "here", "there", "when", "where", "why",
            "how", "all", "each", "few", "more", "most", "other", "some", "such",
            "no", "nor", "not", "only", "own", "same", "so", "than", "too", "very",
            "can", "will", "just", "should", "now", "this", "that", "these", "those",
            "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
            "do", "does", "did", "doing", "would", "could", "ought", "you", "your",
            "yours", "yourself", "yourselves", "he", "him", "his", "himself", "she",
            "her", "hers", "herself", "it", "its", "itself", "they", "them", "their",
            "theirs", "themselves", "what", "which", "who", "whom", "i", "me", "my",
            "myself", "we", "our", "ours", "ourselves",
        ].into_iter().collect()
    }
}

/// Term frequency-inverse document frequency calculator
pub struct TfIdf {
    /// Document frequency for each term
    doc_freq: HashMap<String, usize>,

    /// Total documents
    total_docs: usize,
}

impl TfIdf {
    pub fn new() -> Self {
        Self {
            doc_freq: HashMap::new(),
            total_docs: 0,
        }
    }

    /// Add a document to the corpus
    pub fn add_document(&mut self, terms: &[String]) {
        self.total_docs += 1;

        // Count unique terms in this document
        let unique: HashSet<&String> = terms.iter().collect();
        for term in unique {
            *self.doc_freq.entry(term.clone()).or_insert(0) += 1;
        }
    }

    /// Calculate TF-IDF for a term in a document
    pub fn calculate(&self, term: &str, term_freq: usize, doc_length: usize) -> f64 {
        if doc_length == 0 || self.total_docs == 0 {
            return 0.0;
        }

        // Term frequency (normalized)
        let tf = term_freq as f64 / doc_length as f64;

        // Inverse document frequency
        let df = self.doc_freq.get(term).copied().unwrap_or(0);
        let idf = if df > 0 {
            (self.total_docs as f64 / df as f64).ln()
        } else {
            0.0
        };

        tf * idf
    }

    /// Get top terms by TF-IDF score
    pub fn get_top_terms(&self, document: &[String], n: usize) -> Vec<(String, f64)> {
        let mut term_counts: HashMap<&String, usize> = HashMap::new();
        for term in document {
            *term_counts.entry(term).or_insert(0) += 1;
        }

        let doc_length = document.len();
        let mut scores: Vec<(String, f64)> = term_counts
            .iter()
            .map(|(term, &count)| {
                let score = self.calculate(term, count, doc_length);
                ((*term).clone(), score)
            })
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(n);
        scores
    }
}

impl Default for TfIdf {
    fn default() -> Self {
        Self::new()
    }
}

/// Category-based related posts finder (simpler, faster)
pub fn find_by_categories(
    post_id: i64,
    categories: &[i64],
    all_posts: &[(i64, Vec<i64>)], // (post_id, categories)
    limit: usize,
) -> Vec<i64> {
    let cat_set: HashSet<i64> = categories.iter().copied().collect();

    let mut scored: Vec<(i64, usize)> = all_posts
        .iter()
        .filter(|(id, _)| *id != post_id)
        .map(|(id, cats)| {
            let shared = cats.iter().filter(|c| cat_set.contains(c)).count();
            (*id, shared)
        })
        .filter(|(_, shared)| *shared > 0)
        .collect();

    scored.sort_by(|a, b| b.1.cmp(&a.1));
    scored.truncate(limit);

    scored.into_iter().map(|(id, _)| id).collect()
}

/// Tag-based related posts finder
pub fn find_by_tags(
    post_id: i64,
    tags: &[i64],
    all_posts: &[(i64, Vec<i64>)], // (post_id, tags)
    limit: usize,
) -> Vec<i64> {
    let tag_set: HashSet<i64> = tags.iter().copied().collect();

    let mut scored: Vec<(i64, usize)> = all_posts
        .iter()
        .filter(|(id, _)| *id != post_id)
        .map(|(id, post_tags)| {
            let shared = post_tags.iter().filter(|t| tag_set.contains(t)).count();
            (*id, shared)
        })
        .filter(|(_, shared)| *shared > 0)
        .collect();

    scored.sort_by(|a, b| b.1.cmp(&a.1));
    scored.truncate(limit);

    scored.into_iter().map(|(id, _)| id).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jaccard_similarity() {
        let finder = RelatedFinder::default();

        let a = vec![1, 2, 3];
        let b = vec![2, 3, 4];

        let score = finder.calculate_jaccard(&a, &b);
        assert!((score - 0.5).abs() < 0.01); // 2 shared / 4 total
    }

    #[test]
    fn test_find_related() {
        let finder = RelatedFinder::default();

        let source = PostData::new(1, "Rust Programming", "Learn Rust")
            .with_categories(vec![1, 2])
            .with_tags(vec![10, 11]);

        let candidates = vec![
            PostData::new(2, "More Rust", "Advanced Rust")
                .with_categories(vec![1, 2])
                .with_tags(vec![10]),
            PostData::new(3, "Python Guide", "Learn Python")
                .with_categories(vec![3])
                .with_tags(vec![20]),
        ];

        let related = finder.find_related(&source, &candidates);

        assert!(!related.is_empty());
        assert_eq!(related[0].post_id, 2); // Should match Rust post
    }

    #[test]
    fn test_keyword_extraction() {
        let finder = RelatedFinder::default();
        let keywords = finder.extract_keywords("Rust Programming Guide", "Learn how to program in Rust");

        assert!(keywords.contains(&"rust".to_string()));
        assert!(keywords.contains(&"programming".to_string()));
        assert!(!keywords.contains(&"the".to_string())); // Stopword
    }

    #[test]
    fn test_find_by_categories() {
        let posts = vec![
            (1, vec![1, 2]),
            (2, vec![1, 3]),
            (3, vec![4, 5]),
        ];

        let related = find_by_categories(1, &[1, 2], &posts, 5);

        assert_eq!(related.len(), 1);
        assert_eq!(related[0], 2);
    }
}
