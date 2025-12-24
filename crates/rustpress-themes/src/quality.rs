//! Theme Quality: AMP (216), Accessibility (217), Performance (218)
//!
//! AMP compatibility, accessibility checks, and performance scoring.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Quality check errors
#[derive(Debug, Error)]
pub enum QualityError {
    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

//=============================================================================
// AMP Compatibility (216)
//=============================================================================

/// AMP compatibility checker and transformer
pub struct AmpCompatibility {
    config: AmpConfig,
}

/// AMP configuration
#[derive(Debug, Clone)]
pub struct AmpConfig {
    /// Enable AMP for single posts
    pub enabled_posts: bool,
    /// Enable AMP for pages
    pub enabled_pages: bool,
    /// AMP mode (standard, paired, reader)
    pub mode: AmpMode,
    /// Allowed HTML tags
    pub allowed_tags: Vec<String>,
    /// Custom CSS limit (50KB for AMP)
    pub css_limit: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmpMode {
    /// Full AMP, no non-AMP version
    Standard,
    /// Both AMP and non-AMP versions
    Paired,
    /// AMP served via reader mode
    Reader,
}

impl Default for AmpConfig {
    fn default() -> Self {
        Self {
            enabled_posts: true,
            enabled_pages: true,
            mode: AmpMode::Paired,
            allowed_tags: vec![
                "amp-img".to_string(),
                "amp-video".to_string(),
                "amp-audio".to_string(),
                "amp-iframe".to_string(),
                "amp-carousel".to_string(),
                "amp-accordion".to_string(),
                "amp-sidebar".to_string(),
                "amp-analytics".to_string(),
            ],
            css_limit: 50 * 1024, // 50KB
        }
    }
}

impl AmpCompatibility {
    pub fn new(config: AmpConfig) -> Self {
        Self { config }
    }

    /// Transform HTML to AMP-compatible version
    pub fn transform(&self, html: &str) -> Result<String, QualityError> {
        let mut result = html.to_string();

        // Replace <img> with <amp-img>
        result = self.transform_images(&result);

        // Replace <video> with <amp-video>
        result = self.transform_videos(&result);

        // Replace <audio> with <amp-audio>
        result = self.transform_audio(&result);

        // Replace <iframe> with <amp-iframe>
        result = self.transform_iframes(&result);

        // Remove inline styles
        result = self.remove_inline_styles(&result);

        // Remove disallowed scripts
        result = self.remove_scripts(&result);

        Ok(result)
    }

    fn transform_images(&self, html: &str) -> String {
        // Simple regex-based transformation
        let re = regex::Regex::new(
            r#"<img\s+([^>]*src="[^"]+")([^>]*)>"#
        ).unwrap();

        re.replace_all(html, |caps: &regex::Captures| {
            let attrs = &caps[1];
            let rest = &caps[2];

            // Extract width and height if present
            let width_re = regex::Regex::new(r#"width="(\d+)""#).unwrap();
            let height_re = regex::Regex::new(r#"height="(\d+)""#).unwrap();

            let width = width_re.captures(rest)
                .map(|c| c[1].to_string())
                .unwrap_or_else(|| "auto".to_string());

            let height = height_re.captures(rest)
                .map(|c| c[1].to_string())
                .unwrap_or_else(|| "auto".to_string());

            format!(
                r#"<amp-img {} layout="responsive" width="{}" height="{}"></amp-img>"#,
                attrs, width, height
            )
        }).to_string()
    }

    fn transform_videos(&self, html: &str) -> String {
        let re = regex::Regex::new(
            r#"<video\s+([^>]*)>([\s\S]*?)</video>"#
        ).unwrap();

        re.replace_all(html, |caps: &regex::Captures| {
            let attrs = &caps[1];
            let inner = &caps[2];
            format!(r#"<amp-video {} layout="responsive">{}</amp-video>"#, attrs, inner)
        }).to_string()
    }

    fn transform_audio(&self, html: &str) -> String {
        let re = regex::Regex::new(
            r#"<audio\s+([^>]*)>([\s\S]*?)</audio>"#
        ).unwrap();

        re.replace_all(html, |caps: &regex::Captures| {
            let attrs = &caps[1];
            let inner = &caps[2];
            format!(r#"<amp-audio {}>{}</amp-audio>"#, attrs, inner)
        }).to_string()
    }

    fn transform_iframes(&self, html: &str) -> String {
        let re = regex::Regex::new(
            r#"<iframe\s+([^>]*)></iframe>"#
        ).unwrap();

        re.replace_all(html, |caps: &regex::Captures| {
            let attrs = &caps[1];
            format!(r#"<amp-iframe {} layout="responsive" sandbox="allow-scripts allow-same-origin"></amp-iframe>"#, attrs)
        }).to_string()
    }

    fn remove_inline_styles(&self, html: &str) -> String {
        let re = regex::Regex::new(r#"\s*style="[^"]*""#).unwrap();
        re.replace_all(html, "").to_string()
    }

    fn remove_scripts(&self, html: &str) -> String {
        // Keep only AMP scripts
        let re = regex::Regex::new(
            r#"<script(?![^>]*ampproject)[^>]*>[\s\S]*?</script>"#
        ).unwrap();

        re.replace_all(html, "").to_string()
    }

    /// Generate AMP boilerplate
    pub fn generate_boilerplate(&self) -> String {
        r#"<style amp-boilerplate>body{-webkit-animation:-amp-start 8s steps(1,end) 0s 1 normal both;-moz-animation:-amp-start 8s steps(1,end) 0s 1 normal both;-ms-animation:-amp-start 8s steps(1,end) 0s 1 normal both;animation:-amp-start 8s steps(1,end) 0s 1 normal both}@-webkit-keyframes -amp-start{from{visibility:hidden}to{visibility:visible}}@-moz-keyframes -amp-start{from{visibility:hidden}to{visibility:visible}}@-ms-keyframes -amp-start{from{visibility:hidden}to{visibility:visible}}@-o-keyframes -amp-start{from{visibility:hidden}to{visibility:visible}}@keyframes -amp-start{from{visibility:hidden}to{visibility:visible}}</style>
<noscript><style amp-boilerplate>body{-webkit-animation:none;-moz-animation:none;-ms-animation:none;animation:none}</style></noscript>
<script async src="https://cdn.ampproject.org/v0.js"></script>"#.to_string()
    }

    /// Validate CSS size for AMP
    pub fn validate_css_size(&self, css: &str) -> Result<(), QualityError> {
        if css.len() > self.config.css_limit {
            return Err(QualityError::Validation(format!(
                "CSS exceeds AMP limit ({} > {} bytes)",
                css.len(),
                self.config.css_limit
            )));
        }
        Ok(())
    }
}

//=============================================================================
// Accessibility Checks (217)
//=============================================================================

/// Accessibility checker
pub struct AccessibilityChecker {
    rules: Vec<AccessibilityRule>,
}

/// Accessibility rule
#[derive(Debug, Clone)]
pub struct AccessibilityRule {
    pub id: String,
    pub description: String,
    pub severity: Severity,
    pub wcag_criteria: String,
    pub check: AccessibilityCheck,
}

#[derive(Debug, Clone)]
pub enum AccessibilityCheck {
    ImagesHaveAlt,
    LinksHaveText,
    HeadingsHaveHierarchy,
    FormLabels,
    ColorContrast,
    SkipLinks,
    LandmarkRoles,
    FocusVisible,
    LanguageDeclared,
    TabOrder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Notice,
}

/// Accessibility issue
#[derive(Debug, Clone, Serialize)]
pub struct AccessibilityIssue {
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
    pub element: Option<String>,
    pub suggestion: String,
    pub wcag_criteria: String,
}

/// Accessibility report
#[derive(Debug, Clone, Serialize)]
pub struct AccessibilityReport {
    pub issues: Vec<AccessibilityIssue>,
    pub errors: usize,
    pub warnings: usize,
    pub notices: usize,
    pub passed: bool,
}

impl AccessibilityChecker {
    pub fn new() -> Self {
        Self {
            rules: Self::default_rules(),
        }
    }

    fn default_rules() -> Vec<AccessibilityRule> {
        vec![
            AccessibilityRule {
                id: "img-alt".to_string(),
                description: "Images must have alt attributes".to_string(),
                severity: Severity::Error,
                wcag_criteria: "WCAG 1.1.1".to_string(),
                check: AccessibilityCheck::ImagesHaveAlt,
            },
            AccessibilityRule {
                id: "link-text".to_string(),
                description: "Links must have discernible text".to_string(),
                severity: Severity::Error,
                wcag_criteria: "WCAG 2.4.4".to_string(),
                check: AccessibilityCheck::LinksHaveText,
            },
            AccessibilityRule {
                id: "heading-hierarchy".to_string(),
                description: "Headings must be in proper hierarchy".to_string(),
                severity: Severity::Warning,
                wcag_criteria: "WCAG 1.3.1".to_string(),
                check: AccessibilityCheck::HeadingsHaveHierarchy,
            },
            AccessibilityRule {
                id: "form-labels".to_string(),
                description: "Form inputs must have labels".to_string(),
                severity: Severity::Error,
                wcag_criteria: "WCAG 1.3.1".to_string(),
                check: AccessibilityCheck::FormLabels,
            },
            AccessibilityRule {
                id: "color-contrast".to_string(),
                description: "Text must have sufficient color contrast".to_string(),
                severity: Severity::Error,
                wcag_criteria: "WCAG 1.4.3".to_string(),
                check: AccessibilityCheck::ColorContrast,
            },
            AccessibilityRule {
                id: "skip-link".to_string(),
                description: "Page should have a skip link".to_string(),
                severity: Severity::Warning,
                wcag_criteria: "WCAG 2.4.1".to_string(),
                check: AccessibilityCheck::SkipLinks,
            },
            AccessibilityRule {
                id: "landmarks".to_string(),
                description: "Page should have landmark roles".to_string(),
                severity: Severity::Warning,
                wcag_criteria: "WCAG 1.3.1".to_string(),
                check: AccessibilityCheck::LandmarkRoles,
            },
            AccessibilityRule {
                id: "focus-visible".to_string(),
                description: "Interactive elements must have visible focus".to_string(),
                severity: Severity::Error,
                wcag_criteria: "WCAG 2.4.7".to_string(),
                check: AccessibilityCheck::FocusVisible,
            },
            AccessibilityRule {
                id: "lang-attr".to_string(),
                description: "HTML element must have lang attribute".to_string(),
                severity: Severity::Error,
                wcag_criteria: "WCAG 3.1.1".to_string(),
                check: AccessibilityCheck::LanguageDeclared,
            },
        ]
    }

    /// Check HTML for accessibility issues
    pub fn check(&self, html: &str) -> AccessibilityReport {
        let mut issues = Vec::new();

        for rule in &self.rules {
            match &rule.check {
                AccessibilityCheck::ImagesHaveAlt => {
                    issues.extend(self.check_images_alt(html, rule));
                }
                AccessibilityCheck::LinksHaveText => {
                    issues.extend(self.check_links_text(html, rule));
                }
                AccessibilityCheck::HeadingsHaveHierarchy => {
                    issues.extend(self.check_headings(html, rule));
                }
                AccessibilityCheck::FormLabels => {
                    issues.extend(self.check_form_labels(html, rule));
                }
                AccessibilityCheck::SkipLinks => {
                    issues.extend(self.check_skip_links(html, rule));
                }
                AccessibilityCheck::LandmarkRoles => {
                    issues.extend(self.check_landmarks(html, rule));
                }
                AccessibilityCheck::LanguageDeclared => {
                    issues.extend(self.check_language(html, rule));
                }
                _ => {}
            }
        }

        let errors = issues.iter().filter(|i| i.severity == Severity::Error).count();
        let warnings = issues.iter().filter(|i| i.severity == Severity::Warning).count();
        let notices = issues.iter().filter(|i| i.severity == Severity::Notice).count();

        AccessibilityReport {
            issues,
            errors,
            warnings,
            notices,
            passed: errors == 0,
        }
    }

    fn check_images_alt(&self, html: &str, rule: &AccessibilityRule) -> Vec<AccessibilityIssue> {
        let mut issues = Vec::new();
        let re = regex::Regex::new(r#"<img\s+([^>]*)>"#).unwrap();

        for cap in re.captures_iter(html) {
            let attrs = &cap[1];
            if !attrs.contains("alt=") {
                issues.push(AccessibilityIssue {
                    rule_id: rule.id.clone(),
                    severity: rule.severity,
                    message: "Image missing alt attribute".to_string(),
                    element: Some(cap[0].to_string()),
                    suggestion: "Add alt attribute describing the image content".to_string(),
                    wcag_criteria: rule.wcag_criteria.clone(),
                });
            } else if attrs.contains(r#"alt="""#) {
                // Empty alt is okay for decorative images
            }
        }

        issues
    }

    fn check_links_text(&self, html: &str, rule: &AccessibilityRule) -> Vec<AccessibilityIssue> {
        let mut issues = Vec::new();
        let re = regex::Regex::new(r#"<a\s+([^>]*)>([^<]*)</a>"#).unwrap();

        for cap in re.captures_iter(html) {
            let attrs = &cap[1];
            let text = cap[2].trim();

            if text.is_empty() && !attrs.contains("aria-label") && !attrs.contains("aria-labelledby") {
                issues.push(AccessibilityIssue {
                    rule_id: rule.id.clone(),
                    severity: rule.severity,
                    message: "Link has no discernible text".to_string(),
                    element: Some(cap[0].to_string()),
                    suggestion: "Add link text or aria-label attribute".to_string(),
                    wcag_criteria: rule.wcag_criteria.clone(),
                });
            }
        }

        issues
    }

    fn check_headings(&self, html: &str, rule: &AccessibilityRule) -> Vec<AccessibilityIssue> {
        let mut issues = Vec::new();
        let re = regex::Regex::new(r#"<h([1-6])[^>]*>"#).unwrap();

        let mut last_level = 0;
        for cap in re.captures_iter(html) {
            let level: u8 = cap[1].parse().unwrap_or(1);

            if last_level > 0 && level > last_level + 1 {
                issues.push(AccessibilityIssue {
                    rule_id: rule.id.clone(),
                    severity: rule.severity,
                    message: format!("Heading level skipped: h{} to h{}", last_level, level),
                    element: Some(cap[0].to_string()),
                    suggestion: "Use headings in sequential order".to_string(),
                    wcag_criteria: rule.wcag_criteria.clone(),
                });
            }

            last_level = level;
        }

        issues
    }

    fn check_form_labels(&self, html: &str, rule: &AccessibilityRule) -> Vec<AccessibilityIssue> {
        let mut issues = Vec::new();
        let input_re = regex::Regex::new(r#"<input\s+([^>]*id="([^"]+)"[^>]*)>"#).unwrap();

        for cap in input_re.captures_iter(html) {
            let input_id = &cap[2];
            let label_pattern = format!(r#"<label[^>]*for="{}""#, regex::escape(input_id));

            if regex::Regex::new(&label_pattern).unwrap().find(html).is_none() {
                // Check for aria-label
                if !cap[1].contains("aria-label") {
                    issues.push(AccessibilityIssue {
                        rule_id: rule.id.clone(),
                        severity: rule.severity,
                        message: format!("Input '{}' has no associated label", input_id),
                        element: Some(cap[0].to_string()),
                        suggestion: "Add a <label> element with matching 'for' attribute".to_string(),
                        wcag_criteria: rule.wcag_criteria.clone(),
                    });
                }
            }
        }

        issues
    }

    fn check_skip_links(&self, html: &str, rule: &AccessibilityRule) -> Vec<AccessibilityIssue> {
        let mut issues = Vec::new();

        // Check for skip link pattern
        let has_skip_link = html.contains(r##"href="#content""##) ||
            html.contains(r##"href="#main""##) ||
            html.contains("skip-link") ||
            html.contains("skip-to-content");

        if !has_skip_link {
            issues.push(AccessibilityIssue {
                rule_id: rule.id.clone(),
                severity: rule.severity,
                message: "Page has no skip link".to_string(),
                element: None,
                suggestion: "Add a skip link at the beginning of the page".to_string(),
                wcag_criteria: rule.wcag_criteria.clone(),
            });
        }

        issues
    }

    fn check_landmarks(&self, html: &str, rule: &AccessibilityRule) -> Vec<AccessibilityIssue> {
        let mut issues = Vec::new();

        let has_main = html.contains("<main") || html.contains(r#"role="main""#);
        let has_nav = html.contains("<nav") || html.contains(r#"role="navigation""#);
        let has_header = html.contains("<header") || html.contains(r#"role="banner""#);

        if !has_main {
            issues.push(AccessibilityIssue {
                rule_id: rule.id.clone(),
                severity: rule.severity,
                message: "Page has no main landmark".to_string(),
                element: None,
                suggestion: "Add <main> element or role=\"main\"".to_string(),
                wcag_criteria: rule.wcag_criteria.clone(),
            });
        }

        if !has_nav {
            issues.push(AccessibilityIssue {
                rule_id: rule.id.clone(),
                severity: Severity::Notice,
                message: "Page has no navigation landmark".to_string(),
                element: None,
                suggestion: "Consider adding <nav> element".to_string(),
                wcag_criteria: rule.wcag_criteria.clone(),
            });
        }

        issues
    }

    fn check_language(&self, html: &str, rule: &AccessibilityRule) -> Vec<AccessibilityIssue> {
        let mut issues = Vec::new();

        if !html.contains(r#"<html"#) || !regex::Regex::new(r#"<html[^>]+lang="#).unwrap().is_match(html) {
            issues.push(AccessibilityIssue {
                rule_id: rule.id.clone(),
                severity: rule.severity,
                message: "HTML element missing lang attribute".to_string(),
                element: None,
                suggestion: "Add lang attribute to <html> element".to_string(),
                wcag_criteria: rule.wcag_criteria.clone(),
            });
        }

        issues
    }
}

impl Default for AccessibilityChecker {
    fn default() -> Self {
        Self::new()
    }
}

//=============================================================================
// Performance Scoring (218)
//=============================================================================

/// Performance scorer
pub struct PerformanceScorer {
    weights: PerformanceWeights,
}

/// Scoring weights
#[derive(Debug, Clone)]
pub struct PerformanceWeights {
    pub css_size: f32,
    pub js_size: f32,
    pub image_optimization: f32,
    pub critical_css: f32,
    pub lazy_loading: f32,
    pub caching: f32,
    pub compression: f32,
}

impl Default for PerformanceWeights {
    fn default() -> Self {
        Self {
            css_size: 0.15,
            js_size: 0.15,
            image_optimization: 0.20,
            critical_css: 0.15,
            lazy_loading: 0.10,
            caching: 0.10,
            compression: 0.15,
        }
    }
}

/// Performance metrics
#[derive(Debug, Clone, Serialize)]
pub struct PerformanceMetrics {
    pub css_size: u64,
    pub js_size: u64,
    pub total_images: u32,
    pub optimized_images: u32,
    pub has_critical_css: bool,
    pub lazy_loading_enabled: bool,
    pub cache_headers: bool,
    pub compression_enabled: bool,
}

/// Performance score
#[derive(Debug, Clone, Serialize)]
pub struct PerformanceScore {
    pub overall: f32,
    pub grade: char,
    pub metrics: PerformanceMetrics,
    pub recommendations: Vec<String>,
}

impl PerformanceScorer {
    pub fn new() -> Self {
        Self {
            weights: PerformanceWeights::default(),
        }
    }

    /// Score theme performance
    pub fn score(&self, metrics: PerformanceMetrics) -> PerformanceScore {
        let mut score = 0.0;
        let mut recommendations = Vec::new();

        // CSS score (lower is better)
        let css_score = if metrics.css_size < 50_000 {
            1.0
        } else if metrics.css_size < 100_000 {
            0.7
        } else if metrics.css_size < 200_000 {
            0.4
        } else {
            recommendations.push("Consider reducing CSS file size".to_string());
            0.1
        };
        score += css_score * self.weights.css_size;

        // JS score (lower is better)
        let js_score = if metrics.js_size < 100_000 {
            1.0
        } else if metrics.js_size < 200_000 {
            0.7
        } else if metrics.js_size < 500_000 {
            0.4
        } else {
            recommendations.push("Consider reducing JavaScript size".to_string());
            0.1
        };
        score += js_score * self.weights.js_size;

        // Image optimization
        let image_score = if metrics.total_images == 0 {
            1.0
        } else {
            let ratio = metrics.optimized_images as f32 / metrics.total_images as f32;
            if ratio < 0.5 {
                recommendations.push("Optimize more images for web".to_string());
            }
            ratio
        };
        score += image_score * self.weights.image_optimization;

        // Critical CSS
        let critical_score = if metrics.has_critical_css { 1.0 } else {
            recommendations.push("Implement critical CSS for faster rendering".to_string());
            0.0
        };
        score += critical_score * self.weights.critical_css;

        // Lazy loading
        let lazy_score = if metrics.lazy_loading_enabled { 1.0 } else {
            recommendations.push("Enable lazy loading for images".to_string());
            0.0
        };
        score += lazy_score * self.weights.lazy_loading;

        // Caching
        let cache_score = if metrics.cache_headers { 1.0 } else {
            recommendations.push("Configure proper cache headers".to_string());
            0.0
        };
        score += cache_score * self.weights.caching;

        // Compression
        let compression_score = if metrics.compression_enabled { 1.0 } else {
            recommendations.push("Enable gzip/brotli compression".to_string());
            0.0
        };
        score += compression_score * self.weights.compression;

        let overall = (score * 100.0).round() / 100.0;
        let grade = if overall >= 0.9 {
            'A'
        } else if overall >= 0.8 {
            'B'
        } else if overall >= 0.7 {
            'C'
        } else if overall >= 0.6 {
            'D'
        } else {
            'F'
        };

        PerformanceScore {
            overall,
            grade,
            metrics,
            recommendations,
        }
    }
}

impl Default for PerformanceScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amp_transform_images() {
        let amp = AmpCompatibility::new(AmpConfig::default());
        let html = r#"<img src="test.jpg" width="100" height="100">"#;
        let result = amp.transform(html).unwrap();

        assert!(result.contains("amp-img"));
        assert!(result.contains("layout=\"responsive\""));
    }

    #[test]
    fn test_accessibility_checker() {
        let checker = AccessibilityChecker::new();
        let html = r#"<html><body><img src="test.jpg"></body></html>"#;

        let report = checker.check(html);
        assert!(!report.passed);
        assert!(report.errors > 0);
    }

    #[test]
    fn test_performance_scorer() {
        let scorer = PerformanceScorer::new();
        let metrics = PerformanceMetrics {
            css_size: 50000,
            js_size: 100000,
            total_images: 10,
            optimized_images: 10,
            has_critical_css: true,
            lazy_loading_enabled: true,
            cache_headers: true,
            compression_enabled: true,
        };

        let score = scorer.score(metrics);
        assert!(score.overall > 0.8);
        assert_eq!(score.grade, 'A');
    }
}
