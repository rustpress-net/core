//! Theme Documentation Generator (219) and Screenshot Generation (220)
//!
//! Automatic documentation and visual preview generation for themes.

use crate::manifest::ThemeManifest;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;

/// Documentation errors
#[derive(Debug, Error)]
pub enum DocsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Template error: {0}")]
    Template(String),

    #[error("Screenshot generation failed: {0}")]
    Screenshot(String),
}

//=============================================================================
// Documentation Generator (219)
//=============================================================================

/// Documentation generator
pub struct DocGenerator {
    theme_path: PathBuf,
    manifest: ThemeManifest,
    output_dir: PathBuf,
}

/// Documentation configuration
#[derive(Debug, Clone)]
pub struct DocConfig {
    /// Include template documentation
    pub include_templates: bool,
    /// Include pattern documentation
    pub include_patterns: bool,
    /// Include customizer documentation
    pub include_customizer: bool,
    /// Include hook documentation
    pub include_hooks: bool,
    /// Output format
    pub format: DocFormat,
}

#[derive(Debug, Clone, Copy)]
pub enum DocFormat {
    Markdown,
    Html,
    Json,
}

impl Default for DocConfig {
    fn default() -> Self {
        Self {
            include_templates: true,
            include_patterns: true,
            include_customizer: true,
            include_hooks: true,
            format: DocFormat::Markdown,
        }
    }
}

impl DocGenerator {
    pub fn new(theme_path: PathBuf, manifest: ThemeManifest, output_dir: PathBuf) -> Self {
        Self {
            theme_path,
            manifest,
            output_dir,
        }
    }

    /// Generate complete documentation
    pub async fn generate(&self, config: &DocConfig) -> Result<PathBuf, DocsError> {
        fs::create_dir_all(&self.output_dir).await?;

        match config.format {
            DocFormat::Markdown => self.generate_markdown(config).await,
            DocFormat::Html => self.generate_html(config).await,
            DocFormat::Json => self.generate_json(config).await,
        }
    }

    async fn generate_markdown(&self, config: &DocConfig) -> Result<PathBuf, DocsError> {
        let mut content = String::new();

        // Header
        content.push_str(&format!("# {}\n\n", self.manifest.theme.name));
        content.push_str(&format!("{}\n\n", self.manifest.theme.description));

        // Metadata
        content.push_str("## Theme Information\n\n");
        content.push_str(&format!("- **Version:** {}\n", self.manifest.theme.version));
        content.push_str(&format!("- **Author:** {}\n", self.manifest.theme.author));
        content.push_str(&format!("- **License:** {}\n", self.manifest.theme.license));

        if let Some(requires) = &self.manifest.theme.requires_rustpress {
            content.push_str(&format!("- **Requires RustPress:** {}\n", requires));
        }

        content.push('\n');

        // Features
        content.push_str("## Features\n\n");
        self.document_features(&mut content);

        // Templates
        if config.include_templates {
            content.push_str("## Templates\n\n");
            self.document_templates(&mut content).await?;
        }

        // Patterns
        if config.include_patterns {
            content.push_str("## Block Patterns\n\n");
            self.document_patterns(&mut content).await?;
        }

        // Customizer
        if config.include_customizer {
            content.push_str("## Theme Customization\n\n");
            self.document_customizer(&mut content);
        }

        // Colors
        content.push_str("## Color Palette\n\n");
        self.document_colors(&mut content);

        // Typography
        content.push_str("## Typography\n\n");
        self.document_typography(&mut content);

        // Hooks
        if config.include_hooks {
            content.push_str("## Theme Hooks\n\n");
            self.document_hooks(&mut content);
        }

        // Write file
        let output_path = self.output_dir.join("README.md");
        fs::write(&output_path, content).await?;

        Ok(output_path)
    }

    async fn generate_html(&self, config: &DocConfig) -> Result<PathBuf, DocsError> {
        let markdown = self.generate_markdown(config).await?;
        let md_content = fs::read_to_string(&markdown).await?;

        // Simple markdown to HTML conversion
        let html_body = self.markdown_to_html(&md_content);

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - Documentation</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; max-width: 800px; margin: 0 auto; padding: 2rem; line-height: 1.6; }}
        h1, h2, h3 {{ margin-top: 2rem; }}
        code {{ background: #f4f4f4; padding: 0.2rem 0.4rem; border-radius: 3px; }}
        pre {{ background: #f4f4f4; padding: 1rem; overflow-x: auto; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 0.5rem; text-align: left; }}
        th {{ background: #f4f4f4; }}
    </style>
</head>
<body>
{}
</body>
</html>"#,
            self.manifest.theme.name,
            html_body
        );

        let output_path = self.output_dir.join("index.html");
        fs::write(&output_path, html).await?;

        Ok(output_path)
    }

    async fn generate_json(&self, _config: &DocConfig) -> Result<PathBuf, DocsError> {
        let doc = ThemeDocumentation {
            name: self.manifest.theme.name.clone(),
            version: self.manifest.theme.version.clone(),
            description: Some(self.manifest.theme.description.clone()),
            author: Some(self.manifest.theme.author.clone()),
            license: Some(self.manifest.theme.license.clone()),
            features: self.collect_features(),
            colors: self.manifest.colors.palette.iter()
                .map(|c| ColorDoc {
                    name: c.name.clone(),
                    slug: c.slug.clone(),
                    color: c.color.clone(),
                })
                .collect(),
            typography: TypographyDoc {
                font_families: self.manifest.typography.font_families.iter()
                    .map(|f| f.name.clone())
                    .collect(),
                font_sizes: self.manifest.typography.font_sizes.iter()
                    .map(|s| s.name.clone())
                    .collect(),
            },
        };

        let json = serde_json::to_string_pretty(&doc)
            .map_err(|e| DocsError::Template(e.to_string()))?;

        let output_path = self.output_dir.join("theme-docs.json");
        fs::write(&output_path, json).await?;

        Ok(output_path)
    }

    fn document_features(&self, content: &mut String) {
        let supports = &self.manifest.supports;

        if supports.block_editor {
            content.push_str("- Block Editor (Gutenberg)\n");
        }
        if supports.custom_logo.is_some() {
            content.push_str("- Custom Logo\n");
        }
        if supports.post_thumbnails {
            content.push_str("- Post Thumbnails\n");
        }
        if supports.custom_header.is_some() {
            content.push_str("- Custom Header\n");
        }
        if supports.custom_background.is_some() {
            content.push_str("- Custom Background\n");
        }
        if supports.full_site_editing {
            content.push_str("- Full Site Editing\n");
        }

        content.push('\n');
    }

    async fn document_templates(&self, content: &mut String) -> Result<(), DocsError> {
        let templates_dir = self.theme_path.join("templates");

        if !templates_dir.exists() {
            content.push_str("No custom templates defined.\n\n");
            return Ok(());
        }

        content.push_str("| Template | Description |\n");
        content.push_str("|----------|-------------|\n");

        let mut entries = fs::read_dir(&templates_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "html") {
                let name = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");

                let description = self.get_template_description(name);
                content.push_str(&format!("| `{}` | {} |\n", name, description));
            }
        }

        content.push('\n');
        Ok(())
    }

    fn get_template_description(&self, name: &str) -> &'static str {
        match name {
            "index" => "Default template for all pages",
            "home" => "Homepage template",
            "front-page" => "Static front page template",
            "single" => "Single post template",
            "page" => "Page template",
            "archive" => "Archive listing template",
            "category" => "Category archive template",
            "tag" => "Tag archive template",
            "author" => "Author archive template",
            "search" => "Search results template",
            "404" => "Not found error template",
            _ => "Custom template",
        }
    }

    async fn document_patterns(&self, content: &mut String) -> Result<(), DocsError> {
        if self.manifest.patterns.is_empty() {
            content.push_str("No block patterns defined.\n\n");
            return Ok(());
        }

        for pattern in &self.manifest.patterns {
            content.push_str(&format!("### {}\n\n", pattern.title));

            if let Some(desc) = &pattern.description {
                content.push_str(&format!("{}\n\n", desc));
            }

            content.push_str(&format!("- **Categories:** {}\n", pattern.categories.join(", ")));
            content.push('\n');
        }

        Ok(())
    }

    fn document_customizer(&self, content: &mut String) {
        content.push_str("The theme provides the following customization options:\n\n");

        content.push_str("### Colors\n\n");
        content.push_str("Customize the theme color palette in Appearance > Customize > Colors.\n\n");

        content.push_str("### Typography\n\n");
        content.push_str("Customize fonts and text sizes in Appearance > Customize > Typography.\n\n");

        content.push_str("### Layout\n\n");
        content.push_str(&format!(
            "- **Content Width:** {}\n",
            self.manifest.layout.content_width.as_deref().unwrap_or("Not set")
        ));
        content.push_str(&format!(
            "- **Wide Width:** {}\n",
            self.manifest.layout.wide_width.as_deref().unwrap_or("Not set")
        ));
        content.push('\n');
    }

    fn document_colors(&self, content: &mut String) {
        if self.manifest.colors.palette.is_empty() {
            content.push_str("Default color palette.\n\n");
            return;
        }

        content.push_str("| Name | Slug | Color |\n");
        content.push_str("|------|------|-------|\n");

        for color in &self.manifest.colors.palette {
            content.push_str(&format!(
                "| {} | `{}` | {} |\n",
                color.name, color.slug, color.color
            ));
        }

        content.push('\n');
    }

    fn document_typography(&self, content: &mut String) {
        content.push_str("### Font Families\n\n");

        if self.manifest.typography.font_families.is_empty() {
            content.push_str("Default system fonts.\n\n");
        } else {
            for family in &self.manifest.typography.font_families {
                content.push_str(&format!("- **{}:** `{}`\n", family.name, family.font_family));
            }
            content.push('\n');
        }

        content.push_str("### Font Sizes\n\n");

        if self.manifest.typography.font_sizes.is_empty() {
            content.push_str("Default font sizes.\n\n");
        } else {
            content.push_str("| Name | Size |\n");
            content.push_str("|------|------|\n");

            for size in &self.manifest.typography.font_sizes {
                content.push_str(&format!("| {} | {} |\n", size.name, size.size));
            }
            content.push('\n');
        }
    }

    fn document_hooks(&self, content: &mut String) {
        content.push_str("### Available Hooks\n\n");
        content.push_str("| Hook | Description |\n");
        content.push_str("|------|-------------|\n");
        content.push_str("| `theme_setup` | Runs when theme is initialized |\n");
        content.push_str("| `enqueue_scripts` | Runs when scripts are enqueued |\n");
        content.push_str("| `customize_register` | Runs when customizer is set up |\n");
        content.push_str("| `widget_init` | Runs when widgets are initialized |\n");
        content.push('\n');
    }

    fn collect_features(&self) -> Vec<String> {
        let mut features = Vec::new();
        let supports = &self.manifest.supports;

        if supports.block_editor { features.push("Block Editor".to_string()); }
        if supports.custom_logo.is_some() { features.push("Custom Logo".to_string()); }
        if supports.post_thumbnails { features.push("Post Thumbnails".to_string()); }
        if supports.custom_header.is_some() { features.push("Custom Header".to_string()); }
        if supports.custom_background.is_some() { features.push("Custom Background".to_string()); }
        if supports.full_site_editing { features.push("Full Site Editing".to_string()); }

        features
    }

    fn markdown_to_html(&self, markdown: &str) -> String {
        let mut html = String::new();
        let mut in_code_block = false;
        let mut in_table = false;

        for line in markdown.lines() {
            // Code blocks
            if line.starts_with("```") {
                if in_code_block {
                    html.push_str("</code></pre>\n");
                } else {
                    html.push_str("<pre><code>");
                }
                in_code_block = !in_code_block;
                continue;
            }

            if in_code_block {
                html.push_str(&html_escape(line));
                html.push('\n');
                continue;
            }

            // Tables
            if line.starts_with('|') {
                if !in_table {
                    html.push_str("<table>\n");
                    in_table = true;
                }

                if line.contains("---") {
                    continue; // Skip separator row
                }

                let cells: Vec<&str> = line.trim_matches('|').split('|').collect();
                let tag = if html.contains("<table>") && !html.contains("</tr>") { "th" } else { "td" };

                html.push_str("<tr>");
                for cell in cells {
                    html.push_str(&format!("<{}>{}</{}>", tag, cell.trim(), tag));
                }
                html.push_str("</tr>\n");
                continue;
            } else if in_table {
                html.push_str("</table>\n");
                in_table = false;
            }

            // Headings
            if line.starts_with("### ") {
                html.push_str(&format!("<h3>{}</h3>\n", &line[4..]));
            } else if line.starts_with("## ") {
                html.push_str(&format!("<h2>{}</h2>\n", &line[3..]));
            } else if line.starts_with("# ") {
                html.push_str(&format!("<h1>{}</h1>\n", &line[2..]));
            } else if line.starts_with("- ") {
                html.push_str(&format!("<li>{}</li>\n", &line[2..]));
            } else if line.is_empty() {
                html.push_str("<br>\n");
            } else {
                html.push_str(&format!("<p>{}</p>\n", line));
            }
        }

        if in_table {
            html.push_str("</table>\n");
        }

        html
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[derive(Debug, Serialize, Deserialize)]
struct ThemeDocumentation {
    name: String,
    version: String,
    description: Option<String>,
    author: Option<String>,
    license: Option<String>,
    features: Vec<String>,
    colors: Vec<ColorDoc>,
    typography: TypographyDoc,
}

#[derive(Debug, Serialize, Deserialize)]
struct ColorDoc {
    name: String,
    slug: String,
    color: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TypographyDoc {
    font_families: Vec<String>,
    font_sizes: Vec<String>,
}

//=============================================================================
// Screenshot Generation (220)
//=============================================================================

/// Screenshot generator configuration
#[derive(Debug, Clone)]
pub struct ScreenshotConfig {
    /// Output width
    pub width: u32,
    /// Output height
    pub height: u32,
    /// Viewport width for capture
    pub viewport_width: u32,
    /// Viewport height for capture
    pub viewport_height: u32,
    /// Format (png, jpeg, webp)
    pub format: ImageFormat,
    /// Quality (for jpeg/webp)
    pub quality: u8,
}

#[derive(Debug, Clone, Copy)]
pub enum ImageFormat {
    Png,
    Jpeg,
    WebP,
}

impl Default for ScreenshotConfig {
    fn default() -> Self {
        Self {
            width: 1200,
            height: 900,
            viewport_width: 1200,
            viewport_height: 900,
            format: ImageFormat::Png,
            quality: 85,
        }
    }
}

/// Screenshot generator
pub struct ScreenshotGenerator {
    config: ScreenshotConfig,
}

impl ScreenshotGenerator {
    pub fn new(config: ScreenshotConfig) -> Self {
        Self { config }
    }

    /// Generate screenshot from HTML content
    pub async fn generate_from_html(
        &self,
        html: &str,
        output_path: &Path,
    ) -> Result<(), DocsError> {
        // In a real implementation, this would use a headless browser
        // like chromium via headless_chrome or similar crate

        // For now, generate a placeholder image
        self.generate_placeholder(output_path).await
    }

    /// Generate placeholder screenshot
    async fn generate_placeholder(&self, output_path: &Path) -> Result<(), DocsError> {
        use image::{ImageBuffer, Rgb};

        // Create a simple gradient image
        let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(
            self.config.width,
            self.config.height,
        );

        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let r = (x as f32 / self.config.width as f32 * 255.0) as u8;
            let g = (y as f32 / self.config.height as f32 * 255.0) as u8;
            let b = 200u8;
            *pixel = Rgb([r, g, b]);
        }

        // Add some visual elements to indicate it's a placeholder
        // (In a real implementation, you'd render actual content)

        img.save(output_path)
            .map_err(|e| DocsError::Screenshot(e.to_string()))?;

        Ok(())
    }

    /// Generate multiple screenshots for different viewports
    pub async fn generate_responsive(
        &self,
        html: &str,
        output_dir: &Path,
    ) -> Result<Vec<PathBuf>, DocsError> {
        fs::create_dir_all(output_dir).await?;

        let viewports = [
            ("desktop", 1920, 1080),
            ("laptop", 1366, 768),
            ("tablet", 768, 1024),
            ("mobile", 375, 812),
        ];

        let mut paths = Vec::new();

        for (name, width, height) in viewports {
            let config = ScreenshotConfig {
                width,
                height,
                viewport_width: width,
                viewport_height: height,
                ..self.config
            };

            let generator = ScreenshotGenerator::new(config);
            let path = output_dir.join(format!("screenshot-{}.png", name));

            generator.generate_from_html(html, &path).await?;
            paths.push(path);
        }

        Ok(paths)
    }

    /// Generate theme screenshot (1200x900 standard)
    pub async fn generate_theme_screenshot(
        &self,
        theme_path: &Path,
    ) -> Result<PathBuf, DocsError> {
        let output_path = theme_path.join("screenshot.png");

        // Try to find an index.html or front-page.html
        let templates_dir = theme_path.join("templates");
        let html_path = if templates_dir.join("front-page.html").exists() {
            templates_dir.join("front-page.html")
        } else if templates_dir.join("index.html").exists() {
            templates_dir.join("index.html")
        } else {
            // Generate placeholder
            self.generate_placeholder(&output_path).await?;
            return Ok(output_path);
        };

        let html = fs::read_to_string(&html_path).await?;
        self.generate_from_html(&html, &output_path).await?;

        Ok(output_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_screenshot_placeholder() {
        let dir = tempdir().unwrap();
        let generator = ScreenshotGenerator::new(ScreenshotConfig::default());
        let output_path = dir.path().join("test.png");

        generator.generate_placeholder(&output_path).await.unwrap();
        assert!(output_path.exists());
    }

    #[test]
    fn test_screenshot_config_default() {
        let config = ScreenshotConfig::default();
        assert_eq!(config.width, 1200);
        assert_eq!(config.height, 900);
    }
}
