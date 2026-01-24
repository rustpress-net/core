//! SEO Settings and Analysis
//!
//! Comprehensive SEO features including:
//! - Meta title and description
//! - Open Graph / Twitter Cards
//! - Schema.org structured data
//! - Readability analysis
//! - Keyword optimization

use serde::{Deserialize, Serialize};

/// Post SEO settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PostSeo {
    /// SEO title (different from post title)
    pub meta_title: Option<String>,

    /// Meta description
    pub meta_description: Option<String>,

    /// Focus keyword for analysis
    pub focus_keyword: Option<String>,

    /// Secondary keywords
    #[serde(default)]
    pub secondary_keywords: Vec<String>,

    /// Canonical URL
    pub canonical_url: Option<String>,

    /// Robots meta directives
    pub robots: RobotsMeta,

    /// Open Graph settings
    pub open_graph: OpenGraphSettings,

    /// Twitter Card settings
    pub twitter_card: TwitterCardSettings,

    /// Schema.org structured data
    pub schema: SchemaSettings,

    /// SEO analysis results (cached)
    pub analysis: Option<SeoAnalysis>,

    /// Readability analysis results (cached)
    pub readability: Option<ReadabilityAnalysis>,

    /// Breadcrumb settings
    pub breadcrumbs: BreadcrumbSettings,

    /// Sitemap settings
    pub sitemap: SitemapSettings,
}

impl PostSeo {
    /// Get effective title (SEO title or post title)
    pub fn get_effective_title(&self, post_title: &str) -> String {
        self.meta_title
            .clone()
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| post_title.to_string())
    }

    /// Check if SEO title length is optimal (50-60 chars)
    pub fn is_title_length_optimal(&self) -> bool {
        if let Some(title) = &self.meta_title {
            let len = title.chars().count();
            len >= 50 && len <= 60
        } else {
            false
        }
    }

    /// Check if meta description length is optimal (120-160 chars)
    pub fn is_description_length_optimal(&self) -> bool {
        if let Some(desc) = &self.meta_description {
            let len = desc.chars().count();
            len >= 120 && len <= 160
        } else {
            false
        }
    }

    /// Get SEO score (0-100)
    pub fn get_score(&self) -> u8 {
        self.analysis.as_ref().map(|a| a.score).unwrap_or(0)
    }

    /// Get readability score
    pub fn get_readability_score(&self) -> Option<f32> {
        self.readability.as_ref().map(|r| r.flesch_reading_ease)
    }
}

/// Robots meta directives
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RobotsMeta {
    /// Allow indexing
    #[serde(default = "default_true")]
    pub index: bool,

    /// Allow following links
    #[serde(default = "default_true")]
    pub follow: bool,

    /// Show in search results
    #[serde(default = "default_true")]
    pub archive: bool,

    /// Show snippet
    #[serde(default = "default_true")]
    pub snippet: bool,

    /// Show image preview
    #[serde(default = "default_true")]
    pub image_preview: bool,

    /// Max snippet length (-1 for unlimited)
    pub max_snippet: Option<i32>,

    /// Max image preview size
    pub max_image_preview: Option<ImagePreviewSize>,

    /// Max video preview seconds (-1 for unlimited)
    pub max_video_preview: Option<i32>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImagePreviewSize {
    None,
    Standard,
    Large,
}

impl RobotsMeta {
    /// Generate robots meta content
    pub fn to_meta_content(&self) -> String {
        let mut directives = Vec::new();

        if !self.index {
            directives.push("noindex");
        }
        if !self.follow {
            directives.push("nofollow");
        }
        if !self.archive {
            directives.push("noarchive");
        }
        if !self.snippet {
            directives.push("nosnippet");
        }
        if !self.image_preview {
            directives.push("noimageindex");
        }

        if directives.is_empty() {
            "index, follow".to_string()
        } else {
            directives.join(", ")
        }
    }
}

/// Open Graph settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenGraphSettings {
    /// OG title (defaults to SEO title)
    pub title: Option<String>,

    /// OG description (defaults to meta description)
    pub description: Option<String>,

    /// OG image URL
    pub image: Option<String>,

    /// OG image alt text
    pub image_alt: Option<String>,

    /// OG type (article, website, etc.)
    #[serde(default)]
    pub og_type: OgType,

    /// Article-specific settings
    pub article: Option<OgArticle>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OgType {
    #[default]
    Article,
    Website,
    Blog,
    Product,
    Video,
    Music,
    Profile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OgArticle {
    /// Author name or profile URL
    pub author: Option<String>,

    /// Publisher page URL
    pub publisher: Option<String>,

    /// Section/category
    pub section: Option<String>,

    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,

    /// Published time (ISO 8601)
    pub published_time: Option<String>,

    /// Modified time (ISO 8601)
    pub modified_time: Option<String>,

    /// Expiration time (ISO 8601)
    pub expiration_time: Option<String>,
}

/// Twitter Card settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TwitterCardSettings {
    /// Card type
    #[serde(default)]
    pub card_type: TwitterCardType,

    /// Twitter title
    pub title: Option<String>,

    /// Twitter description
    pub description: Option<String>,

    /// Twitter image URL
    pub image: Option<String>,

    /// Twitter image alt text
    pub image_alt: Option<String>,

    /// Twitter creator handle
    pub creator: Option<String>,

    /// Twitter site handle
    pub site: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TwitterCardType {
    #[default]
    SummaryLargeImage,
    Summary,
    App,
    Player,
}

/// Schema.org structured data settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchemaSettings {
    /// Enable schema output
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Schema type
    #[serde(default)]
    pub schema_type: SchemaType,

    /// Article-specific schema
    pub article: Option<ArticleSchema>,

    /// FAQ schema (if post contains FAQ)
    pub faq: Option<FaqSchema>,

    /// HowTo schema (if post is a tutorial)
    pub how_to: Option<HowToSchema>,

    /// Custom JSON-LD (advanced)
    pub custom_json_ld: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum SchemaType {
    #[default]
    Article,
    NewsArticle,
    BlogPosting,
    TechArticle,
    ScholarlyArticle,
    WebPage,
    ItemPage,
    FAQPage,
    HowTo,
    Recipe,
    Product,
    Review,
    Event,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleSchema {
    pub headline: Option<String>,
    pub description: Option<String>,
    pub author_name: Option<String>,
    pub author_url: Option<String>,
    pub publisher_name: Option<String>,
    pub publisher_logo: Option<String>,
    pub date_published: Option<String>,
    pub date_modified: Option<String>,
    pub image: Option<String>,
    pub word_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaqSchema {
    pub questions: Vec<FaqItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaqItem {
    pub question: String,
    pub answer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HowToSchema {
    pub name: String,
    pub description: Option<String>,
    pub total_time: Option<String>,
    pub estimated_cost: Option<String>,
    pub supplies: Vec<String>,
    pub tools: Vec<String>,
    pub steps: Vec<HowToStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HowToStep {
    pub name: String,
    pub text: String,
    pub image: Option<String>,
    pub url: Option<String>,
}

/// Breadcrumb settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BreadcrumbSettings {
    /// Show breadcrumbs
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Custom breadcrumb path
    pub custom_path: Option<Vec<BreadcrumbItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreadcrumbItem {
    pub name: String,
    pub url: String,
}

/// Sitemap settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SitemapSettings {
    /// Include in sitemap
    #[serde(default = "default_true")]
    pub include: bool,

    /// Priority (0.0 - 1.0)
    pub priority: Option<f32>,

    /// Change frequency
    pub changefreq: Option<ChangeFrequency>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeFrequency {
    Always,
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Yearly,
    Never,
}

/// SEO Analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeoAnalysis {
    /// Overall SEO score (0-100)
    pub score: u8,

    /// Analysis results
    pub results: Vec<SeoAnalysisItem>,

    /// Problems found
    pub problems: Vec<SeoProblem>,

    /// Improvements suggested
    pub improvements: Vec<SeoImprovement>,

    /// Analysis timestamp
    pub analyzed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeoAnalysisItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: SeoStatus,
    pub score_impact: i8, // positive or negative
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SeoStatus {
    Good,
    Ok,
    Warning,
    Bad,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeoProblem {
    pub severity: ProblemSeverity,
    pub title: String,
    pub description: String,
    pub fix_suggestion: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProblemSeverity {
    Critical,
    Major,
    Minor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeoImprovement {
    pub priority: ImprovementPriority,
    pub title: String,
    pub description: String,
    pub action: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImprovementPriority {
    High,
    Medium,
    Low,
}

/// Readability Analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadabilityAnalysis {
    /// Flesch Reading Ease score (0-100, higher = easier)
    pub flesch_reading_ease: f32,

    /// Flesch-Kincaid Grade Level
    pub flesch_kincaid_grade: f32,

    /// Gunning Fog Index
    pub gunning_fog: f32,

    /// SMOG Index
    pub smog_index: f32,

    /// Coleman-Liau Index
    pub coleman_liau: f32,

    /// Automated Readability Index
    pub automated_readability: f32,

    /// Average sentence length
    pub avg_sentence_length: f32,

    /// Average word length
    pub avg_word_length: f32,

    /// Percentage of complex words
    pub complex_word_percentage: f32,

    /// Reading time in minutes
    pub reading_time_minutes: u32,

    /// Speaking time in minutes
    pub speaking_time_minutes: u32,

    /// Issues found
    pub issues: Vec<ReadabilityIssue>,

    /// Overall rating
    pub rating: ReadabilityRating,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadabilityIssue {
    pub issue_type: ReadabilityIssueType,
    pub message: String,
    pub location: Option<String>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadabilityIssueType {
    LongSentence,
    LongParagraph,
    PassiveVoice,
    ComplexWord,
    ConsecutiveSentenceStarts,
    AdverbOveruse,
    HiddenVerb,
    Nominalization,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadabilityRating {
    VeryEasy,     // 90-100
    Easy,         // 80-89
    FairlyEasy,   // 70-79
    Standard,     // 60-69
    FairlyDifficult, // 50-59
    Difficult,    // 30-49
    VeryDifficult,   // 0-29
}

impl ReadabilityRating {
    pub fn from_flesch_score(score: f32) -> Self {
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

    pub fn description(&self) -> &'static str {
        match self {
            Self::VeryEasy => "Very easy to read. Easily understood by an average 11-year-old student.",
            Self::Easy => "Easy to read. Conversational English for consumers.",
            Self::FairlyEasy => "Fairly easy to read.",
            Self::Standard => "Plain English. Easily understood by 13- to 15-year-old students.",
            Self::FairlyDifficult => "Fairly difficult to read.",
            Self::Difficult => "Difficult to read. Best understood by college graduates.",
            Self::VeryDifficult => "Very difficult to read. Best understood by university graduates.",
        }
    }
}
