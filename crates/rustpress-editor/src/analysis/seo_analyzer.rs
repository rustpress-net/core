//! SEO Analyzer
//!
//! Analyzes content for search engine optimization.

use serde::{Deserialize, Serialize};

/// SEO Analyzer for content optimization
#[derive(Debug, Clone)]
pub struct SeoAnalyzer {
    /// Focus keyword for analysis
    focus_keyword: Option<String>,
    /// Secondary keywords
    secondary_keywords: Vec<String>,
    /// Analysis configuration
    config: SeoAnalyzerConfig,
}

impl SeoAnalyzer {
    pub fn new() -> Self {
        Self {
            focus_keyword: None,
            secondary_keywords: Vec::new(),
            config: SeoAnalyzerConfig::default(),
        }
    }

    /// Set the focus keyword
    pub fn with_focus_keyword(mut self, keyword: String) -> Self {
        self.focus_keyword = Some(keyword);
        self
    }

    /// Add secondary keywords
    pub fn with_secondary_keywords(mut self, keywords: Vec<String>) -> Self {
        self.secondary_keywords = keywords;
        self
    }

    /// Analyze content for SEO
    pub fn analyze(&self, content: &SeoContent) -> SeoAnalysisResult {
        let mut checks = Vec::new();
        let mut score = 0;
        let mut max_score = 0;

        // Title analysis
        checks.push(self.check_title(&content.title));
        max_score += 15;

        // Meta description analysis
        checks.push(self.check_meta_description(&content.meta_description));
        max_score += 15;

        // Content length
        checks.push(self.check_content_length(content.word_count));
        max_score += 10;

        // Heading structure
        checks.push(self.check_headings(&content.headings));
        max_score += 10;

        // Keyword usage
        if let Some(ref keyword) = self.focus_keyword {
            checks.push(self.check_keyword_in_title(keyword, &content.title));
            checks.push(self.check_keyword_in_first_paragraph(keyword, &content.first_paragraph));
            checks.push(self.check_keyword_density(keyword, &content.plain_text, content.word_count));
            checks.push(self.check_keyword_in_headings(keyword, &content.headings));
            checks.push(self.check_keyword_in_url(keyword, &content.slug));
            max_score += 50;
        }

        // Image optimization
        checks.push(self.check_images(&content.images));
        max_score += 10;

        // Internal/external links
        checks.push(self.check_links(&content.links));
        max_score += 10;

        // Calculate score
        for check in &checks {
            if check.passed {
                score += check.weight;
            }
        }

        let normalized_score = if max_score > 0 {
            ((score as f32 / max_score as f32) * 100.0) as u32
        } else {
            0
        };

        let suggestions = self.generate_suggestions(&checks);

        SeoAnalysisResult {
            score: normalized_score,
            checks,
            grade: SeoGrade::from_score(normalized_score),
            focus_keyword: self.focus_keyword.clone(),
            suggestions,
        }
    }

    fn check_title(&self, title: &str) -> SeoCheck {
        let length = title.chars().count();
        let ideal_min = self.config.title_min_length;
        let ideal_max = self.config.title_max_length;

        let (passed, message) = if length == 0 {
            (false, "Title is missing".to_string())
        } else if length < ideal_min {
            (false, format!("Title is too short ({} chars). Aim for {}-{} characters.", length, ideal_min, ideal_max))
        } else if length > ideal_max {
            (false, format!("Title is too long ({} chars). Keep it under {} characters.", length, ideal_max))
        } else {
            (true, format!("Title length is good ({} chars)", length))
        };

        SeoCheck {
            id: "title_length".to_string(),
            name: "Title Length".to_string(),
            passed,
            message,
            weight: 15,
            category: SeoCategory::Title,
        }
    }

    fn check_meta_description(&self, description: &Option<String>) -> SeoCheck {
        let (passed, message) = match description {
            None => (false, "Meta description is missing".to_string()),
            Some(desc) => {
                let length = desc.chars().count();
                let min = self.config.meta_desc_min_length;
                let max = self.config.meta_desc_max_length;

                if length < min {
                    (false, format!("Meta description is too short ({} chars). Aim for {}-{} characters.", length, min, max))
                } else if length > max {
                    (false, format!("Meta description is too long ({} chars). Keep it under {} characters.", length, max))
                } else {
                    (true, format!("Meta description length is good ({} chars)", length))
                }
            }
        };

        SeoCheck {
            id: "meta_description".to_string(),
            name: "Meta Description".to_string(),
            passed,
            message,
            weight: 15,
            category: SeoCategory::Meta,
        }
    }

    fn check_content_length(&self, word_count: u32) -> SeoCheck {
        let min = self.config.min_word_count;
        let (passed, message) = if word_count < min {
            (false, format!("Content is too short ({} words). Aim for at least {} words.", word_count, min))
        } else if word_count >= 1500 {
            (true, format!("Excellent content length ({} words)", word_count))
        } else {
            (true, format!("Good content length ({} words)", word_count))
        };

        SeoCheck {
            id: "content_length".to_string(),
            name: "Content Length".to_string(),
            passed,
            message,
            weight: 10,
            category: SeoCategory::Content,
        }
    }

    fn check_headings(&self, headings: &[HeadingInfo]) -> SeoCheck {
        let h1_count = headings.iter().filter(|h| h.level == 1).count();
        let h2_count = headings.iter().filter(|h| h.level == 2).count();

        let (passed, message) = if h1_count == 0 {
            (false, "No H1 heading found. Add a main heading.".to_string())
        } else if h1_count > 1 {
            (false, format!("Multiple H1 headings found ({}). Use only one H1.", h1_count))
        } else if h2_count == 0 {
            (false, "No H2 subheadings found. Add subheadings to structure content.".to_string())
        } else {
            (true, format!("Good heading structure (1 H1, {} H2s)", h2_count))
        };

        SeoCheck {
            id: "heading_structure".to_string(),
            name: "Heading Structure".to_string(),
            passed,
            message,
            weight: 10,
            category: SeoCategory::Structure,
        }
    }

    fn check_keyword_in_title(&self, keyword: &str, title: &str) -> SeoCheck {
        let title_lower = title.to_lowercase();
        let keyword_lower = keyword.to_lowercase();
        let passed = title_lower.contains(&keyword_lower);

        let message = if passed {
            "Focus keyword appears in title".to_string()
        } else {
            "Focus keyword not found in title".to_string()
        };

        SeoCheck {
            id: "keyword_in_title".to_string(),
            name: "Keyword in Title".to_string(),
            passed,
            message,
            weight: 10,
            category: SeoCategory::Keyword,
        }
    }

    fn check_keyword_in_first_paragraph(&self, keyword: &str, first_paragraph: &str) -> SeoCheck {
        let para_lower = first_paragraph.to_lowercase();
        let keyword_lower = keyword.to_lowercase();
        let passed = para_lower.contains(&keyword_lower);

        let message = if passed {
            "Focus keyword appears in first paragraph".to_string()
        } else {
            "Add focus keyword to the first paragraph".to_string()
        };

        SeoCheck {
            id: "keyword_in_intro".to_string(),
            name: "Keyword in Introduction".to_string(),
            passed,
            message,
            weight: 10,
            category: SeoCategory::Keyword,
        }
    }

    fn check_keyword_density(&self, keyword: &str, text: &str, word_count: u32) -> SeoCheck {
        let text_lower = text.to_lowercase();
        let keyword_lower = keyword.to_lowercase();
        let keyword_count = text_lower.matches(&keyword_lower).count() as f32;
        let density = if word_count > 0 {
            (keyword_count / word_count as f32) * 100.0
        } else {
            0.0
        };

        let min = self.config.keyword_density_min;
        let max = self.config.keyword_density_max;

        let (passed, message) = if density < min {
            (false, format!("Keyword density is too low ({:.1}%). Aim for {:.1}%-{:.1}%.", density, min, max))
        } else if density > max {
            (false, format!("Keyword density is too high ({:.1}%). Keep it between {:.1}%-{:.1}%.", density, min, max))
        } else {
            (true, format!("Good keyword density ({:.1}%)", density))
        };

        SeoCheck {
            id: "keyword_density".to_string(),
            name: "Keyword Density".to_string(),
            passed,
            message,
            weight: 10,
            category: SeoCategory::Keyword,
        }
    }

    fn check_keyword_in_headings(&self, keyword: &str, headings: &[HeadingInfo]) -> SeoCheck {
        let keyword_lower = keyword.to_lowercase();
        let found = headings.iter().any(|h| h.text.to_lowercase().contains(&keyword_lower));

        let message = if found {
            "Focus keyword appears in subheadings".to_string()
        } else {
            "Add focus keyword to at least one subheading".to_string()
        };

        SeoCheck {
            id: "keyword_in_headings".to_string(),
            name: "Keyword in Headings".to_string(),
            passed: found,
            message,
            weight: 10,
            category: SeoCategory::Keyword,
        }
    }

    fn check_keyword_in_url(&self, keyword: &str, slug: &str) -> SeoCheck {
        let slug_lower = slug.to_lowercase().replace('-', " ");
        let keyword_lower = keyword.to_lowercase();
        let passed = slug_lower.contains(&keyword_lower) ||
                     slug.to_lowercase().contains(&keyword_lower.replace(' ', "-"));

        let message = if passed {
            "Focus keyword appears in URL".to_string()
        } else {
            "Consider adding focus keyword to URL slug".to_string()
        };

        SeoCheck {
            id: "keyword_in_url".to_string(),
            name: "Keyword in URL".to_string(),
            passed,
            message,
            weight: 10,
            category: SeoCategory::Keyword,
        }
    }

    fn check_images(&self, images: &[ImageInfo]) -> SeoCheck {
        if images.is_empty() {
            return SeoCheck {
                id: "images".to_string(),
                name: "Image Optimization".to_string(),
                passed: false,
                message: "No images found. Add at least one image.".to_string(),
                weight: 10,
                category: SeoCategory::Media,
            };
        }

        let missing_alt = images.iter().filter(|i| i.alt.is_none() || i.alt.as_ref().map(|a| a.is_empty()).unwrap_or(true)).count();

        let (passed, message) = if missing_alt > 0 {
            (false, format!("{} image(s) missing alt text", missing_alt))
        } else {
            (true, format!("All {} images have alt text", images.len()))
        };

        SeoCheck {
            id: "images".to_string(),
            name: "Image Optimization".to_string(),
            passed,
            message,
            weight: 10,
            category: SeoCategory::Media,
        }
    }

    fn check_links(&self, links: &[LinkInfo]) -> SeoCheck {
        let internal_count = links.iter().filter(|l| l.is_internal).count();
        let external_count = links.iter().filter(|l| !l.is_internal).count();

        let (passed, message) = if internal_count == 0 && external_count == 0 {
            (false, "No links found. Add internal and external links.".to_string())
        } else if internal_count == 0 {
            (false, "No internal links found. Add links to other pages on your site.".to_string())
        } else if external_count == 0 {
            (true, format!("{} internal links found. Consider adding external links to authoritative sources.", internal_count))
        } else {
            (true, format!("{} internal and {} external links found", internal_count, external_count))
        };

        SeoCheck {
            id: "links".to_string(),
            name: "Internal/External Links".to_string(),
            passed,
            message,
            weight: 10,
            category: SeoCategory::Links,
        }
    }

    fn generate_suggestions(&self, checks: &[SeoCheck]) -> Vec<String> {
        checks
            .iter()
            .filter(|c| !c.passed)
            .map(|c| c.message.clone())
            .collect()
    }
}

impl Default for SeoAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// SEO analyzer configuration
#[derive(Debug, Clone)]
pub struct SeoAnalyzerConfig {
    pub title_min_length: usize,
    pub title_max_length: usize,
    pub meta_desc_min_length: usize,
    pub meta_desc_max_length: usize,
    pub min_word_count: u32,
    pub keyword_density_min: f32,
    pub keyword_density_max: f32,
}

impl Default for SeoAnalyzerConfig {
    fn default() -> Self {
        Self {
            title_min_length: 30,
            title_max_length: 60,
            meta_desc_min_length: 120,
            meta_desc_max_length: 160,
            min_word_count: 300,
            keyword_density_min: 0.5,
            keyword_density_max: 2.5,
        }
    }
}

/// Content to analyze for SEO
#[derive(Debug, Clone)]
pub struct SeoContent {
    pub title: String,
    pub slug: String,
    pub meta_description: Option<String>,
    pub plain_text: String,
    pub first_paragraph: String,
    pub word_count: u32,
    pub headings: Vec<HeadingInfo>,
    pub images: Vec<ImageInfo>,
    pub links: Vec<LinkInfo>,
}

/// Heading information
#[derive(Debug, Clone)]
pub struct HeadingInfo {
    pub level: u8,
    pub text: String,
}

/// Image information
#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub src: String,
    pub alt: Option<String>,
    pub title: Option<String>,
}

/// Link information
#[derive(Debug, Clone)]
pub struct LinkInfo {
    pub href: String,
    pub text: String,
    pub is_internal: bool,
    pub is_nofollow: bool,
}

/// SEO analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeoAnalysisResult {
    pub score: u32,
    pub checks: Vec<SeoCheck>,
    pub grade: SeoGrade,
    pub focus_keyword: Option<String>,
    pub suggestions: Vec<String>,
}

/// Individual SEO check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeoCheck {
    pub id: String,
    pub name: String,
    pub passed: bool,
    pub message: String,
    pub weight: u32,
    pub category: SeoCategory,
}

/// SEO check categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SeoCategory {
    Title,
    Meta,
    Content,
    Structure,
    Keyword,
    Media,
    Links,
}

/// SEO grade
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SeoGrade {
    Excellent,
    Good,
    Fair,
    NeedsWork,
    Poor,
}

impl SeoGrade {
    pub fn from_score(score: u32) -> Self {
        match score {
            90..=100 => Self::Excellent,
            75..=89 => Self::Good,
            60..=74 => Self::Fair,
            40..=59 => Self::NeedsWork,
            _ => Self::Poor,
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            Self::Excellent => "#10b981",
            Self::Good => "#22c55e",
            Self::Fair => "#f59e0b",
            Self::NeedsWork => "#f97316",
            Self::Poor => "#ef4444",
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
