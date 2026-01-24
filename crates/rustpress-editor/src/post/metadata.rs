//! Post Metadata
//!
//! Author, categories, tags, and organizational metadata.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Post metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PostMetadata {
    /// Primary author
    pub author: Option<Author>,

    /// Co-authors
    #[serde(default)]
    pub co_authors: Vec<Author>,

    /// Categories
    #[serde(default)]
    pub categories: Vec<Term>,

    /// Tags
    #[serde(default)]
    pub tags: Vec<Term>,

    /// Custom taxonomies
    #[serde(default)]
    pub taxonomies: Vec<TaxonomyTerms>,

    /// Related posts
    #[serde(default)]
    pub related_posts: Vec<i64>,

    /// Publication date (can differ from created_at)
    pub publish_date: Option<DateTime<Utc>>,

    /// Last modified by
    pub last_modified_by: Option<Author>,

    /// Content source/origin
    pub source: Option<ContentSource>,

    /// Copyright info
    pub copyright: Option<String>,

    /// Language
    pub language: Option<String>,

    /// Translations
    #[serde(default)]
    pub translations: Vec<Translation>,
}

impl PostMetadata {
    /// Set the author
    pub fn set_author(&mut self, author: Author) {
        self.author = Some(author);
    }

    /// Add a category
    pub fn add_category(&mut self, category: Term) {
        if !self.categories.iter().any(|c| c.id == category.id) {
            self.categories.push(category);
        }
    }

    /// Remove a category
    pub fn remove_category(&mut self, category_id: i64) {
        self.categories.retain(|c| c.id != category_id);
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: Term) {
        if !self.tags.iter().any(|t| t.id == tag.id) {
            self.tags.push(tag);
        }
    }

    /// Remove a tag
    pub fn remove_tag(&mut self, tag_id: i64) {
        self.tags.retain(|t| t.id != tag_id);
    }

    /// Get all term IDs (categories + tags)
    pub fn get_all_term_ids(&self) -> Vec<i64> {
        let mut ids: Vec<i64> = self.categories.iter().map(|c| c.id).collect();
        ids.extend(self.tags.iter().map(|t| t.id));
        ids
    }

    /// Check if post has a specific category
    pub fn has_category(&self, slug: &str) -> bool {
        self.categories.iter().any(|c| c.slug == slug)
    }

    /// Check if post has a specific tag
    pub fn has_tag(&self, slug: &str) -> bool {
        self.tags.iter().any(|t| t.slug == slug)
    }
}

/// Author information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    /// User ID
    pub id: i64,

    /// Display name
    pub name: String,

    /// Username/login
    pub username: String,

    /// Email (optional, for privacy)
    pub email: Option<String>,

    /// Avatar URL
    pub avatar_url: Option<String>,

    /// Author bio
    pub bio: Option<String>,

    /// Website URL
    pub url: Option<String>,

    /// Social links
    #[serde(default)]
    pub social_links: Vec<SocialLink>,
}

impl Author {
    pub fn new(id: i64, name: impl Into<String>, username: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            username: username.into(),
            email: None,
            avatar_url: None,
            bio: None,
            url: None,
            social_links: Vec::new(),
        }
    }

    /// Get avatar URL with fallback to Gravatar
    pub fn get_avatar(&self, size: u32) -> String {
        if let Some(url) = &self.avatar_url {
            url.clone()
        } else if let Some(email) = &self.email {
            let hash = md5_hash(email.trim().to_lowercase().as_str());
            format!(
                "https://www.gravatar.com/avatar/{}?s={}&d=identicon",
                hash, size
            )
        } else {
            format!(
                "https://www.gravatar.com/avatar/00000000000000000000000000000000?s={}&d=identicon",
                size
            )
        }
    }
}

/// Social link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialLink {
    pub service: String,
    pub url: String,
    pub username: Option<String>,
}

/// Taxonomy term (category, tag, or custom)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Term {
    /// Term ID
    pub id: i64,

    /// Term name
    pub name: String,

    /// URL slug
    pub slug: String,

    /// Description
    pub description: Option<String>,

    /// Parent term ID (for hierarchical taxonomies)
    pub parent_id: Option<i64>,

    /// Term count (posts using this term)
    pub count: u32,

    /// Term metadata
    #[serde(default)]
    pub meta: TermMeta,
}

impl Term {
    pub fn new(id: i64, name: impl Into<String>, slug: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            slug: slug.into(),
            description: None,
            parent_id: None,
            count: 0,
            meta: TermMeta::default(),
        }
    }
}

/// Term metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TermMeta {
    /// Color for visual display
    pub color: Option<String>,

    /// Icon
    pub icon: Option<String>,

    /// Image URL
    pub image_url: Option<String>,

    /// Featured term
    pub featured: bool,
}

/// Custom taxonomy with terms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxonomyTerms {
    /// Taxonomy slug
    pub taxonomy: String,

    /// Taxonomy name
    pub name: String,

    /// Terms in this taxonomy
    pub terms: Vec<Term>,
}

/// Content source information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSource {
    /// Source name
    pub name: String,

    /// Source URL
    pub url: Option<String>,

    /// Original author
    pub author: Option<String>,

    /// License
    pub license: Option<String>,

    /// Attribution text
    pub attribution: Option<String>,
}

/// Translation reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Translation {
    /// Language code (e.g., "en", "fr", "de")
    pub language: String,

    /// Language name
    pub language_name: String,

    /// Translated post ID
    pub post_id: i64,

    /// URL to translated post
    pub url: Option<String>,
}

/// Simple MD5 hash for Gravatar
fn md5_hash(input: &str) -> String {
    use std::fmt::Write;

    // Simple MD5 implementation for Gravatar
    // In production, use a proper crypto library
    let digest = md5_compute(input.as_bytes());
    let mut s = String::with_capacity(32);
    for byte in digest {
        write!(&mut s, "{:02x}", byte).unwrap();
    }
    s
}

/// Minimal MD5 computation
fn md5_compute(data: &[u8]) -> [u8; 16] {
    const S: [u32; 64] = [
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22,
        5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20,
        4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
        6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
    ];
    const K: [u32; 64] = [
        0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee,
        0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
        0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be,
        0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
        0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa,
        0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
        0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
        0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
        0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c,
        0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
        0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05,
        0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
        0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039,
        0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
        0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1,
        0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
    ];

    let mut h0: u32 = 0x67452301;
    let mut h1: u32 = 0xefcdab89;
    let mut h2: u32 = 0x98badcfe;
    let mut h3: u32 = 0x10325476;

    let bit_len = (data.len() as u64) * 8;
    let mut padded = data.to_vec();
    padded.push(0x80);
    while (padded.len() % 64) != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_le_bytes());

    for chunk in padded.chunks(64) {
        let mut m = [0u32; 16];
        for (i, word) in chunk.chunks(4).enumerate() {
            m[i] = u32::from_le_bytes([word[0], word[1], word[2], word[3]]);
        }

        let (mut a, mut b, mut c, mut d) = (h0, h1, h2, h3);

        for i in 0..64 {
            let (f, g) = match i {
                0..=15 => ((b & c) | ((!b) & d), i),
                16..=31 => ((d & b) | ((!d) & c), (5 * i + 1) % 16),
                32..=47 => (b ^ c ^ d, (3 * i + 5) % 16),
                _ => (c ^ (b | (!d)), (7 * i) % 16),
            };
            let temp = d;
            d = c;
            c = b;
            b = b.wrapping_add(
                (a.wrapping_add(f).wrapping_add(K[i]).wrapping_add(m[g]))
                    .rotate_left(S[i]),
            );
            a = temp;
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
    }

    let mut result = [0u8; 16];
    result[0..4].copy_from_slice(&h0.to_le_bytes());
    result[4..8].copy_from_slice(&h1.to_le_bytes());
    result[8..12].copy_from_slice(&h2.to_le_bytes());
    result[12..16].copy_from_slice(&h3.to_le_bytes());
    result
}
