//! Accessibility Checker
//!
//! WCAG compliance checking for content.

use serde::{Deserialize, Serialize};

/// Accessibility checker for WCAG compliance
#[derive(Debug, Clone)]
pub struct AccessibilityChecker {
    #[allow(dead_code)]
    config: AccessibilityConfig,
}

impl AccessibilityChecker {
    pub fn new() -> Self {
        Self {
            config: AccessibilityConfig::default(),
        }
    }

    /// Check content for accessibility issues
    pub fn check(&self, content: &AccessibilityContent) -> AccessibilityReport {
        let mut issues = Vec::new();
        let mut passed_checks = Vec::new();

        // Image checks
        self.check_images(&content.images, &mut issues, &mut passed_checks);

        // Link checks
        self.check_links(&content.links, &mut issues, &mut passed_checks);

        // Heading checks
        self.check_headings(&content.headings, &mut issues, &mut passed_checks);

        // Color contrast (if provided)
        self.check_color_contrast(&content.colors, &mut issues, &mut passed_checks);

        // Media checks
        self.check_media(&content.media, &mut issues, &mut passed_checks);

        // Table checks
        self.check_tables(&content.tables, &mut issues, &mut passed_checks);

        // Form checks
        self.check_forms(&content.forms, &mut issues, &mut passed_checks);

        // Calculate score
        let total_checks = issues.len() + passed_checks.len();
        let score = if total_checks > 0 {
            ((passed_checks.len() as f32 / total_checks as f32) * 100.0) as u32
        } else {
            100
        };

        let error_count = issues.iter().filter(|i| i.severity == Severity::Error).count();
        let warning_count = issues.iter().filter(|i| i.severity == Severity::Warning).count();

        AccessibilityReport {
            score,
            level: WcagLevel::from_score(score),
            issues,
            passed_checks,
            error_count,
            warning_count,
            summary: self.generate_summary(score, error_count, warning_count),
        }
    }

    fn check_images(
        &self,
        images: &[ImageElement],
        issues: &mut Vec<AccessibilityIssue>,
        passed: &mut Vec<String>,
    ) {
        let mut missing_alt = 0;
        let mut empty_alt = 0;
        let mut decorative = 0;

        for img in images {
            match &img.alt {
                None => missing_alt += 1,
                Some(alt) if alt.is_empty() && !img.is_decorative => empty_alt += 1,
                Some(alt) if alt.is_empty() && img.is_decorative => decorative += 1,
                Some(alt) => {
                    // Check alt text quality
                    if alt.len() > 125 {
                        issues.push(AccessibilityIssue {
                            id: "img-alt-long".to_string(),
                            criterion: "1.1.1".to_string(),
                            severity: Severity::Warning,
                            message: format!("Image alt text is too long ({} chars): \"{}...\"", alt.len(), &alt[..50]),
                            element: Some(img.src.clone()),
                            suggestion: "Keep alt text under 125 characters. Use longdesc for complex images.".to_string(),
                        });
                    }
                    if alt.starts_with("image of") || alt.starts_with("picture of") {
                        issues.push(AccessibilityIssue {
                            id: "img-alt-redundant".to_string(),
                            criterion: "1.1.1".to_string(),
                            severity: Severity::Warning,
                            message: "Alt text contains redundant phrases like 'image of'".to_string(),
                            element: Some(img.src.clone()),
                            suggestion: "Remove redundant phrases. Screen readers already announce images.".to_string(),
                        });
                    }
                }
            }
        }

        if missing_alt > 0 {
            issues.push(AccessibilityIssue {
                id: "img-alt-missing".to_string(),
                criterion: "1.1.1".to_string(),
                severity: Severity::Error,
                message: format!("{} image(s) missing alt attribute", missing_alt),
                element: None,
                suggestion: "Add descriptive alt text to all images, or empty alt=\"\" for decorative images.".to_string(),
            });
        }

        if empty_alt > 0 {
            issues.push(AccessibilityIssue {
                id: "img-alt-empty".to_string(),
                criterion: "1.1.1".to_string(),
                severity: Severity::Warning,
                message: format!("{} non-decorative image(s) have empty alt text", empty_alt),
                element: None,
                suggestion: "Add descriptive alt text to meaningful images.".to_string(),
            });
        }

        if missing_alt == 0 && empty_alt == 0 {
            passed.push(format!("All {} images have appropriate alt text", images.len()));
        }

        if decorative > 0 {
            passed.push(format!("{} decorative images correctly marked with empty alt", decorative));
        }
    }

    fn check_links(
        &self,
        links: &[LinkElement],
        issues: &mut Vec<AccessibilityIssue>,
        passed: &mut Vec<String>,
    ) {
        let mut generic_text = 0;
        let mut empty_links = 0;
        let mut new_window_no_warning = 0;

        let generic_phrases = ["click here", "read more", "learn more", "here", "link", "this"];

        for link in links {
            if link.text.is_empty() && link.aria_label.is_none() {
                empty_links += 1;
            } else {
                let text_lower = link.text.to_lowercase();
                if generic_phrases.iter().any(|&phrase| text_lower == phrase) {
                    generic_text += 1;
                }
            }

            if link.opens_new_window && !link.has_new_window_warning {
                new_window_no_warning += 1;
            }
        }

        if empty_links > 0 {
            issues.push(AccessibilityIssue {
                id: "link-empty".to_string(),
                criterion: "2.4.4".to_string(),
                severity: Severity::Error,
                message: format!("{} link(s) have no accessible text", empty_links),
                element: None,
                suggestion: "Add descriptive link text or aria-label.".to_string(),
            });
        }

        if generic_text > 0 {
            issues.push(AccessibilityIssue {
                id: "link-generic".to_string(),
                criterion: "2.4.4".to_string(),
                severity: Severity::Warning,
                message: format!("{} link(s) use generic text like 'click here'", generic_text),
                element: None,
                suggestion: "Use descriptive link text that explains the destination.".to_string(),
            });
        }

        if new_window_no_warning > 0 {
            issues.push(AccessibilityIssue {
                id: "link-new-window".to_string(),
                criterion: "3.2.5".to_string(),
                severity: Severity::Warning,
                message: format!("{} link(s) open in new window without warning", new_window_no_warning),
                element: None,
                suggestion: "Add '(opens in new window)' to link text or use aria-label.".to_string(),
            });
        }

        if empty_links == 0 && generic_text == 0 {
            passed.push(format!("All {} links have descriptive text", links.len()));
        }
    }

    fn check_headings(
        &self,
        headings: &[HeadingElement],
        issues: &mut Vec<AccessibilityIssue>,
        passed: &mut Vec<String>,
    ) {
        if headings.is_empty() {
            issues.push(AccessibilityIssue {
                id: "heading-none".to_string(),
                criterion: "1.3.1".to_string(),
                severity: Severity::Warning,
                message: "No headings found in content".to_string(),
                element: None,
                suggestion: "Add headings to structure your content.".to_string(),
            });
            return;
        }

        // Check for skipped levels
        let mut prev_level = 0;
        let mut skipped_levels = Vec::new();

        for heading in headings {
            if prev_level > 0 && heading.level > prev_level + 1 {
                skipped_levels.push((prev_level, heading.level));
            }
            prev_level = heading.level;
        }

        if !skipped_levels.is_empty() {
            let skips: Vec<String> = skipped_levels
                .iter()
                .map(|(from, to)| format!("H{} to H{}", from, to))
                .collect();
            issues.push(AccessibilityIssue {
                id: "heading-skip".to_string(),
                criterion: "1.3.1".to_string(),
                severity: Severity::Warning,
                message: format!("Heading levels are skipped: {}", skips.join(", ")),
                element: None,
                suggestion: "Use heading levels in sequential order (H1, H2, H3...).".to_string(),
            });
        }

        // Check for multiple H1s
        let h1_count = headings.iter().filter(|h| h.level == 1).count();
        if h1_count > 1 {
            issues.push(AccessibilityIssue {
                id: "heading-multiple-h1".to_string(),
                criterion: "1.3.1".to_string(),
                severity: Severity::Warning,
                message: format!("Multiple H1 headings found ({})", h1_count),
                element: None,
                suggestion: "Use only one H1 per page for the main title.".to_string(),
            });
        }

        // Check for empty headings
        let empty_headings = headings.iter().filter(|h| h.text.trim().is_empty()).count();
        if empty_headings > 0 {
            issues.push(AccessibilityIssue {
                id: "heading-empty".to_string(),
                criterion: "1.3.1".to_string(),
                severity: Severity::Error,
                message: format!("{} empty heading(s) found", empty_headings),
                element: None,
                suggestion: "Remove empty headings or add content.".to_string(),
            });
        }

        if skipped_levels.is_empty() && h1_count <= 1 && empty_headings == 0 {
            passed.push(format!("Heading structure is correct ({} headings)", headings.len()));
        }
    }

    fn check_color_contrast(
        &self,
        colors: &[ColorPair],
        issues: &mut Vec<AccessibilityIssue>,
        passed: &mut Vec<String>,
    ) {
        let mut low_contrast = 0;

        for color in colors {
            let ratio = calculate_contrast_ratio(&color.foreground, &color.background);

            let (min_ratio, requirement) = if color.is_large_text {
                (3.0, "large text")
            } else {
                (4.5, "normal text")
            };

            if ratio < min_ratio {
                low_contrast += 1;
                issues.push(AccessibilityIssue {
                    id: "color-contrast".to_string(),
                    criterion: "1.4.3".to_string(),
                    severity: Severity::Error,
                    message: format!(
                        "Insufficient color contrast ({:.2}:1) for {}. Minimum is {:.1}:1",
                        ratio, requirement, min_ratio
                    ),
                    element: Some(format!("{} on {}", color.foreground, color.background)),
                    suggestion: "Increase contrast between text and background colors.".to_string(),
                });
            }
        }

        if !colors.is_empty() && low_contrast == 0 {
            passed.push("All color combinations meet contrast requirements".to_string());
        }
    }

    fn check_media(
        &self,
        media: &[MediaElement],
        issues: &mut Vec<AccessibilityIssue>,
        passed: &mut Vec<String>,
    ) {
        let mut videos_no_captions = 0;
        let mut audio_no_transcript = 0;

        for item in media {
            match item.media_type {
                MediaType::Video => {
                    if !item.has_captions {
                        videos_no_captions += 1;
                    }
                }
                MediaType::Audio => {
                    if !item.has_transcript {
                        audio_no_transcript += 1;
                    }
                }
            }
        }

        if videos_no_captions > 0 {
            issues.push(AccessibilityIssue {
                id: "video-captions".to_string(),
                criterion: "1.2.2".to_string(),
                severity: Severity::Error,
                message: format!("{} video(s) missing captions", videos_no_captions),
                element: None,
                suggestion: "Add captions to all video content.".to_string(),
            });
        }

        if audio_no_transcript > 0 {
            issues.push(AccessibilityIssue {
                id: "audio-transcript".to_string(),
                criterion: "1.2.1".to_string(),
                severity: Severity::Warning,
                message: format!("{} audio file(s) missing transcript", audio_no_transcript),
                element: None,
                suggestion: "Provide text transcripts for audio content.".to_string(),
            });
        }

        let video_count = media.iter().filter(|m| m.media_type == MediaType::Video).count();
        let audio_count = media.iter().filter(|m| m.media_type == MediaType::Audio).count();

        if video_count > 0 && videos_no_captions == 0 {
            passed.push(format!("All {} videos have captions", video_count));
        }
        if audio_count > 0 && audio_no_transcript == 0 {
            passed.push(format!("All {} audio files have transcripts", audio_count));
        }
    }

    fn check_tables(
        &self,
        tables: &[TableElement],
        issues: &mut Vec<AccessibilityIssue>,
        passed: &mut Vec<String>,
    ) {
        let mut missing_headers = 0;
        let mut missing_caption = 0;

        for table in tables {
            if !table.has_headers {
                missing_headers += 1;
            }
            if table.is_data_table && table.caption.is_none() {
                missing_caption += 1;
            }
        }

        if missing_headers > 0 {
            issues.push(AccessibilityIssue {
                id: "table-headers".to_string(),
                criterion: "1.3.1".to_string(),
                severity: Severity::Error,
                message: format!("{} table(s) missing header cells", missing_headers),
                element: None,
                suggestion: "Use <th> elements for table headers.".to_string(),
            });
        }

        if missing_caption > 0 {
            issues.push(AccessibilityIssue {
                id: "table-caption".to_string(),
                criterion: "1.3.1".to_string(),
                severity: Severity::Warning,
                message: format!("{} data table(s) missing caption", missing_caption),
                element: None,
                suggestion: "Add a <caption> to describe the table.".to_string(),
            });
        }

        if !tables.is_empty() && missing_headers == 0 {
            passed.push(format!("All {} tables have proper headers", tables.len()));
        }
    }

    fn check_forms(
        &self,
        forms: &[FormElement],
        issues: &mut Vec<AccessibilityIssue>,
        passed: &mut Vec<String>,
    ) {
        let mut missing_labels = 0;

        for form in forms {
            for field in &form.fields {
                if field.label.is_none() && field.aria_label.is_none() {
                    missing_labels += 1;
                }
            }
        }

        if missing_labels > 0 {
            issues.push(AccessibilityIssue {
                id: "form-labels".to_string(),
                criterion: "1.3.1".to_string(),
                severity: Severity::Error,
                message: format!("{} form field(s) missing labels", missing_labels),
                element: None,
                suggestion: "Add <label> elements or aria-label to all form inputs.".to_string(),
            });
        }

        let total_fields: usize = forms.iter().map(|f| f.fields.len()).sum();
        if total_fields > 0 && missing_labels == 0 {
            passed.push(format!("All {} form fields have labels", total_fields));
        }
    }

    fn generate_summary(&self, _score: u32, errors: usize, warnings: usize) -> String {
        if errors == 0 && warnings == 0 {
            "Excellent! No accessibility issues found.".to_string()
        } else if errors == 0 {
            format!("Good accessibility with {} minor issue(s) to review.", warnings)
        } else {
            format!("{} critical issue(s) and {} warning(s) need attention.", errors, warnings)
        }
    }
}

impl Default for AccessibilityChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Accessibility checker configuration
#[derive(Debug, Clone)]
pub struct AccessibilityConfig {
    pub wcag_level: WcagLevel,
}

impl Default for AccessibilityConfig {
    fn default() -> Self {
        Self {
            wcag_level: WcagLevel::AA,
        }
    }
}

/// Content to check for accessibility
#[derive(Debug, Clone, Default)]
pub struct AccessibilityContent {
    pub images: Vec<ImageElement>,
    pub links: Vec<LinkElement>,
    pub headings: Vec<HeadingElement>,
    pub colors: Vec<ColorPair>,
    pub media: Vec<MediaElement>,
    pub tables: Vec<TableElement>,
    pub forms: Vec<FormElement>,
}

/// Image element for checking
#[derive(Debug, Clone)]
pub struct ImageElement {
    pub src: String,
    pub alt: Option<String>,
    pub is_decorative: bool,
}

/// Link element for checking
#[derive(Debug, Clone)]
pub struct LinkElement {
    pub href: String,
    pub text: String,
    pub aria_label: Option<String>,
    pub opens_new_window: bool,
    pub has_new_window_warning: bool,
}

/// Heading element for checking
#[derive(Debug, Clone)]
pub struct HeadingElement {
    pub level: u8,
    pub text: String,
}

/// Color pair for contrast checking
#[derive(Debug, Clone)]
pub struct ColorPair {
    pub foreground: String,
    pub background: String,
    pub is_large_text: bool,
}

/// Media element for checking
#[derive(Debug, Clone)]
pub struct MediaElement {
    pub media_type: MediaType,
    pub has_captions: bool,
    pub has_transcript: bool,
    pub has_audio_description: bool,
}

/// Media type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaType {
    Video,
    Audio,
}

/// Table element for checking
#[derive(Debug, Clone)]
pub struct TableElement {
    pub has_headers: bool,
    pub caption: Option<String>,
    pub is_data_table: bool,
}

/// Form element for checking
#[derive(Debug, Clone)]
pub struct FormElement {
    pub fields: Vec<FormField>,
}

/// Form field
#[derive(Debug, Clone)]
pub struct FormField {
    pub field_type: String,
    pub label: Option<String>,
    pub aria_label: Option<String>,
}

/// Accessibility report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityReport {
    pub score: u32,
    pub level: WcagLevel,
    pub issues: Vec<AccessibilityIssue>,
    pub passed_checks: Vec<String>,
    pub error_count: usize,
    pub warning_count: usize,
    pub summary: String,
}

/// Accessibility issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityIssue {
    pub id: String,
    pub criterion: String,
    pub severity: Severity,
    pub message: String,
    pub element: Option<String>,
    pub suggestion: String,
}

/// Issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// WCAG conformance level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WcagLevel {
    A,
    AA,
    AAA,
}

impl WcagLevel {
    pub fn from_score(score: u32) -> Self {
        match score {
            95..=100 => Self::AAA,
            80..=94 => Self::AA,
            _ => Self::A,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::A => "Level A",
            Self::AA => "Level AA",
            Self::AAA => "Level AAA",
        }
    }
}

/// Calculate contrast ratio between two colors (simplified)
fn calculate_contrast_ratio(fg: &str, bg: &str) -> f32 {
    let fg_lum = get_relative_luminance(fg);
    let bg_lum = get_relative_luminance(bg);

    let lighter = fg_lum.max(bg_lum);
    let darker = fg_lum.min(bg_lum);

    (lighter + 0.05) / (darker + 0.05)
}

/// Get relative luminance of a color
fn get_relative_luminance(color: &str) -> f32 {
    let (r, g, b) = parse_color(color);

    let r = srgb_to_linear(r);
    let g = srgb_to_linear(g);
    let b = srgb_to_linear(b);

    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn srgb_to_linear(value: f32) -> f32 {
    if value <= 0.03928 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn parse_color(color: &str) -> (f32, f32, f32) {
    let color = color.trim_start_matches('#');

    if color.len() == 6 {
        let r = u8::from_str_radix(&color[0..2], 16).unwrap_or(0) as f32 / 255.0;
        let g = u8::from_str_radix(&color[2..4], 16).unwrap_or(0) as f32 / 255.0;
        let b = u8::from_str_radix(&color[4..6], 16).unwrap_or(0) as f32 / 255.0;
        (r, g, b)
    } else if color.len() == 3 {
        let r = u8::from_str_radix(&color[0..1].repeat(2), 16).unwrap_or(0) as f32 / 255.0;
        let g = u8::from_str_radix(&color[1..2].repeat(2), 16).unwrap_or(0) as f32 / 255.0;
        let b = u8::from_str_radix(&color[2..3].repeat(2), 16).unwrap_or(0) as f32 / 255.0;
        (r, g, b)
    } else {
        (0.0, 0.0, 0.0)
    }
}
