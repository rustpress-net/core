//! Critical CSS Extraction
//!
//! Extracts above-the-fold CSS for optimal page load performance.

use scraper::Html;
use std::collections::HashSet;
use std::path::Path;
use thiserror::Error;
use tokio::fs;
use regex::Regex;

/// Critical CSS extraction errors
#[derive(Debug, Error)]
pub enum CriticalCssError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSS parsing error: {0}")]
    CssParsing(String),

    #[error("HTML parsing error: {0}")]
    HtmlParsing(String),

    #[error("Extraction error: {0}")]
    Extraction(String),
}

/// Configuration for critical CSS extraction
#[derive(Debug, Clone)]
pub struct CriticalCssConfig {
    /// Viewport width for above-the-fold detection
    pub viewport_width: u32,
    /// Viewport height for above-the-fold detection
    pub viewport_height: u32,
    /// Maximum size of critical CSS (bytes)
    pub max_size: usize,
    /// Include all font-face rules
    pub include_fonts: bool,
    /// Include all keyframe animations
    pub include_animations: bool,
    /// CSS selectors to always include
    pub force_include: Vec<String>,
    /// CSS selectors to always exclude
    pub force_exclude: Vec<String>,
    /// Minify output
    pub minify: bool,
}

impl Default for CriticalCssConfig {
    fn default() -> Self {
        Self {
            viewport_width: 1300,
            viewport_height: 900,
            max_size: 32 * 1024, // 32KB
            include_fonts: true,
            include_animations: true,
            force_include: Vec::new(),
            force_exclude: Vec::new(),
            minify: true,
        }
    }
}

/// Critical CSS extraction result
#[derive(Debug, Clone)]
pub struct CriticalCssResult {
    /// The critical CSS
    pub critical: String,
    /// Remaining non-critical CSS
    pub remaining: String,
    /// Size of critical CSS in bytes
    pub size: usize,
    /// Number of selectors found in HTML
    pub selectors_found: usize,
}

/// Critical CSS extractor
pub struct CriticalCssExtractor {
    config: CriticalCssConfig,
}

impl CriticalCssExtractor {
    pub fn new(config: CriticalCssConfig) -> Self {
        Self { config }
    }

    /// Extract critical CSS from HTML and CSS
    pub async fn extract(&self, html: &str, css: &str) -> Result<CriticalCssResult, CriticalCssError> {
        // Parse HTML to find used selectors
        let used_selectors = self.extract_selectors_from_html(html)?;

        // Extract critical rules using regex-based approach
        let critical = self.extract_critical_rules(css, &used_selectors)?;

        // Minify if configured
        let final_css = if self.config.minify {
            self.minify_css(&critical)
        } else {
            critical
        };

        // Calculate remaining CSS
        let remaining = self.calculate_remaining(css, &final_css);

        Ok(CriticalCssResult {
            critical: final_css.clone(),
            remaining,
            size: final_css.len(),
            selectors_found: used_selectors.len(),
        })
    }

    /// Extract from file paths
    pub async fn extract_from_files(
        &self,
        html_path: &Path,
        css_path: &Path,
    ) -> Result<CriticalCssResult, CriticalCssError> {
        let html = fs::read_to_string(html_path).await?;
        let css = fs::read_to_string(css_path).await?;

        self.extract(&html, &css).await
    }

    fn extract_selectors_from_html(&self, html: &str) -> Result<HashSet<String>, CriticalCssError> {
        let document = Html::parse_document(html);
        let mut selectors = HashSet::new();

        // Collect all element types
        for element in document.tree.nodes() {
            if let Some(el) = element.value().as_element() {
                // Add element name
                selectors.insert(el.name().to_string());

                // Add classes
                if let Some(classes) = el.attr("class") {
                    for class in classes.split_whitespace() {
                        selectors.insert(format!(".{}", class));
                    }
                }

                // Add IDs
                if let Some(id) = el.attr("id") {
                    selectors.insert(format!("#{}", id));
                }

                // Add data attributes
                for attr in el.attrs() {
                    if attr.0.starts_with("data-") {
                        selectors.insert(format!("[{}]", attr.0));
                    }
                }
            }
        }

        // Add forced includes
        for selector in &self.config.force_include {
            selectors.insert(selector.clone());
        }

        Ok(selectors)
    }

    fn extract_critical_rules(
        &self,
        css: &str,
        used_selectors: &HashSet<String>,
    ) -> Result<String, CriticalCssError> {
        let mut critical_rules = Vec::new();
        let mut critical_size = 0;

        // Regex to match CSS rules
        let rule_regex = Regex::new(r"([^{}]+)\{([^{}]*)\}")
            .map_err(|e| CriticalCssError::CssParsing(e.to_string()))?;

        // Also match @font-face and @keyframes
        let fontface_regex = Regex::new(r"@font-face\s*\{[^{}]*\}")
            .map_err(|e| CriticalCssError::CssParsing(e.to_string()))?;
        let keyframes_regex = Regex::new(r"@keyframes\s+[^\{]+\{(?:[^{}]*\{[^{}]*\})*[^{}]*\}")
            .map_err(|e| CriticalCssError::CssParsing(e.to_string()))?;

        // Include @font-face rules if configured
        if self.config.include_fonts {
            for cap in fontface_regex.captures_iter(css) {
                let rule = cap.get(0).unwrap().as_str();
                if critical_size + rule.len() <= self.config.max_size {
                    critical_rules.push(rule.to_string());
                    critical_size += rule.len();
                }
            }
        }

        // Include @keyframes rules if configured
        if self.config.include_animations {
            for cap in keyframes_regex.captures_iter(css) {
                let rule = cap.get(0).unwrap().as_str();
                if critical_size + rule.len() <= self.config.max_size {
                    critical_rules.push(rule.to_string());
                    critical_size += rule.len();
                }
            }
        }

        // Process style rules
        for cap in rule_regex.captures_iter(css) {
            let selector = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("");
            let declarations = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            // Skip at-rules we already handled
            if selector.starts_with('@') {
                continue;
            }

            // Check if selector matches used elements
            if self.selector_matches_used(selector, used_selectors) {
                let rule = format!("{} {{ {} }}", selector, declarations);

                // Check force exclude
                let exclude = self.config.force_exclude.iter().any(|ex| {
                    selector.contains(ex)
                });

                let rule_len = rule.len();
                if !exclude && critical_size + rule_len <= self.config.max_size {
                    critical_rules.push(rule);
                    critical_size += rule_len;
                }
            }
        }

        Ok(critical_rules.join("\n"))
    }

    fn selector_matches_used(&self, selector: &str, used: &HashSet<String>) -> bool {
        // Split compound selectors
        let selectors: Vec<&str> = selector.split(',').map(|s| s.trim()).collect();

        for sel in selectors {
            // Check direct matches
            for used_sel in used {
                if sel.contains(used_sel.as_str()) {
                    return true;
                }
            }

            // Check element selectors
            let parts: Vec<&str> = sel
                .split(|c: char| c.is_whitespace() || c == '>' || c == '+' || c == '~')
                .filter(|s| !s.is_empty())
                .collect();

            for part in parts {
                // Extract base selector (element, class, or id)
                let base = part
                    .split(|c: char| c == ':' || c == '[')
                    .next()
                    .unwrap_or(part);

                if used.contains(base) {
                    return true;
                }
            }
        }

        false
    }

    fn minify_css(&self, css: &str) -> String {
        // Simple minification
        css.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("")
            .replace(": ", ":")
            .replace(" {", "{")
            .replace("{ ", "{")
            .replace(" }", "}")
            .replace("; ", ";")
            .replace("  ", " ")
    }

    fn calculate_remaining(&self, original: &str, critical: &str) -> String {
        // Simple approach: return rules not in critical CSS
        // In production, use proper CSS parsing
        let critical_rules: HashSet<&str> = critical.split('}')
            .filter(|s| !s.trim().is_empty())
            .collect();

        original.split('}')
            .filter(|rule| {
                let trimmed = rule.trim();
                !trimmed.is_empty() && !critical_rules.contains(trimmed)
            })
            .map(|rule| format!("{}}}", rule))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_extract_selectors() {
        let html = r##"
            <html>
            <body class="home page">
                <div id="main" class="container">
                    <h1>Hello</h1>
                </div>
            </body>
            </html>
        "##;

        let extractor = CriticalCssExtractor::new(CriticalCssConfig::default());
        let selectors = extractor.extract_selectors_from_html(html).unwrap();

        assert!(selectors.contains("html"));
        assert!(selectors.contains("body"));
        assert!(selectors.contains(".home"));
        assert!(selectors.contains(".page"));
        assert!(selectors.contains("#main"));
        assert!(selectors.contains(".container"));
        assert!(selectors.contains("h1"));
    }

    #[tokio::test]
    async fn test_critical_css_extraction() {
        let html = r##"
            <div class="header">
                <h1>Title</h1>
            </div>
        "##;

        let css = r##"
            .header { background: blue; }
            .footer { background: gray; }
            h1 { color: white; }
            h2 { color: black; }
        "##;

        let extractor = CriticalCssExtractor::new(CriticalCssConfig::default());
        let result = extractor.extract(html, css).await.unwrap();

        assert!(result.critical.contains(".header"));
        assert!(result.critical.contains("h1"));
        assert!(!result.critical.contains(".footer"));
    }
}
