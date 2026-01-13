//! Block-based content editor (Gutenberg-style)
//!
//! Provides a WordPress Gutenberg-compatible block system for visual editing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Content block - the fundamental unit of block-based content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Unique block ID
    pub id: String,

    /// Block type (e.g., "core/paragraph", "core/heading", "core/image")
    #[serde(rename = "name")]
    pub block_type: String,

    /// Block attributes
    #[serde(default)]
    pub attributes: serde_json::Value,

    /// Inner blocks (for nested structures like columns)
    #[serde(default, rename = "innerBlocks")]
    pub inner_blocks: Vec<Block>,

    /// Inner HTML content
    #[serde(default, rename = "innerHTML")]
    pub inner_html: String,

    /// Inner content parts (alternating HTML and block markers)
    #[serde(default, rename = "innerContent")]
    pub inner_content: Vec<Option<String>>,
}

impl Block {
    /// Create a new block
    pub fn new(block_type: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            block_type: block_type.to_string(),
            attributes: serde_json::json!({}),
            inner_blocks: Vec::new(),
            inner_html: String::new(),
            inner_content: Vec::new(),
        }
    }

    /// Create a paragraph block
    pub fn paragraph(content: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            block_type: "core/paragraph".to_string(),
            attributes: serde_json::json!({
                "content": content
            }),
            inner_blocks: Vec::new(),
            inner_html: format!("<p>{}</p>", content),
            inner_content: vec![Some(format!("<p>{}</p>", content))],
        }
    }

    /// Create a heading block
    pub fn heading(content: &str, level: u8) -> Self {
        let level = level.clamp(1, 6);
        Self {
            id: Uuid::new_v4().to_string(),
            block_type: "core/heading".to_string(),
            attributes: serde_json::json!({
                "content": content,
                "level": level
            }),
            inner_blocks: Vec::new(),
            inner_html: format!("<h{level}>{content}</h{level}>"),
            inner_content: vec![Some(format!("<h{level}>{content}</h{level}>"))],
        }
    }

    /// Create an image block
    pub fn image(url: &str, alt: &str, caption: Option<&str>) -> Self {
        let mut attrs = serde_json::json!({
            "url": url,
            "alt": alt
        });

        if let Some(cap) = caption {
            attrs["caption"] = serde_json::Value::String(cap.to_string());
        }

        Self {
            id: Uuid::new_v4().to_string(),
            block_type: "core/image".to_string(),
            attributes: attrs,
            inner_blocks: Vec::new(),
            inner_html: format!(
                r#"<figure class="wp-block-image"><img src="{}" alt="{}"/>{}</figure>"#,
                url,
                alt,
                caption.map(|c| format!("<figcaption>{}</figcaption>", c)).unwrap_or_default()
            ),
            inner_content: vec![],
        }
    }

    /// Create a list block
    pub fn list(items: &[&str], ordered: bool) -> Self {
        let tag = if ordered { "ol" } else { "ul" };
        let list_html: String = items
            .iter()
            .map(|item| format!("<li>{}</li>", item))
            .collect();

        Self {
            id: Uuid::new_v4().to_string(),
            block_type: "core/list".to_string(),
            attributes: serde_json::json!({
                "ordered": ordered,
                "values": format!("<{tag}>{list_html}</{tag}>")
            }),
            inner_blocks: Vec::new(),
            inner_html: format!("<{tag}>{list_html}</{tag}>"),
            inner_content: vec![Some(format!("<{tag}>{list_html}</{tag}>"))],
        }
    }

    /// Create a quote block
    pub fn quote(content: &str, citation: Option<&str>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            block_type: "core/quote".to_string(),
            attributes: serde_json::json!({
                "value": format!("<p>{}</p>", content),
                "citation": citation.unwrap_or("")
            }),
            inner_blocks: Vec::new(),
            inner_html: format!(
                r#"<blockquote class="wp-block-quote"><p>{}</p>{}</blockquote>"#,
                content,
                citation.map(|c| format!("<cite>{}</cite>", c)).unwrap_or_default()
            ),
            inner_content: vec![],
        }
    }

    /// Create a code block
    pub fn code(content: &str, language: Option<&str>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            block_type: "core/code".to_string(),
            attributes: serde_json::json!({
                "content": content,
                "language": language.unwrap_or("")
            }),
            inner_blocks: Vec::new(),
            inner_html: format!(
                r#"<pre class="wp-block-code"><code{}>{}</code></pre>"#,
                language.map(|l| format!(r#" class="language-{}""#, l)).unwrap_or_default(),
                ammonia::clean(content)
            ),
            inner_content: vec![],
        }
    }

    /// Create a columns block
    pub fn columns(columns: Vec<Block>) -> Self {
        let column_count = columns.len();
        Self {
            id: Uuid::new_v4().to_string(),
            block_type: "core/columns".to_string(),
            attributes: serde_json::json!({
                "columns": column_count
            }),
            inner_blocks: columns,
            inner_html: String::new(),
            inner_content: vec![],
        }
    }

    /// Create a column block (for use inside columns)
    pub fn column(blocks: Vec<Block>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            block_type: "core/column".to_string(),
            attributes: serde_json::json!({}),
            inner_blocks: blocks,
            inner_html: String::new(),
            inner_content: vec![],
        }
    }

    /// Create a button block
    pub fn button(text: &str, url: &str, style: Option<&str>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            block_type: "core/button".to_string(),
            attributes: serde_json::json!({
                "text": text,
                "url": url,
                "className": style.unwrap_or("is-style-fill")
            }),
            inner_blocks: Vec::new(),
            inner_html: format!(
                r#"<div class="wp-block-button {}"><a class="wp-block-button__link" href="{}">{}</a></div>"#,
                style.unwrap_or("is-style-fill"),
                url,
                text
            ),
            inner_content: vec![],
        }
    }

    /// Create a separator/divider block
    pub fn separator(style: Option<&str>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            block_type: "core/separator".to_string(),
            attributes: serde_json::json!({
                "className": style.unwrap_or("is-style-default")
            }),
            inner_blocks: Vec::new(),
            inner_html: format!(
                r#"<hr class="wp-block-separator {}"/>"#,
                style.unwrap_or("is-style-default")
            ),
            inner_content: vec![],
        }
    }

    /// Create a spacer block
    pub fn spacer(height: u32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            block_type: "core/spacer".to_string(),
            attributes: serde_json::json!({
                "height": height
            }),
            inner_blocks: Vec::new(),
            inner_html: format!(
                r#"<div class="wp-block-spacer" style="height:{}px" aria-hidden="true"></div>"#,
                height
            ),
            inner_content: vec![],
        }
    }

    /// Create an embed block (YouTube, Vimeo, Twitter, etc.)
    pub fn embed(url: &str, provider: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            block_type: format!("core-embed/{}", provider),
            attributes: serde_json::json!({
                "url": url,
                "providerNameSlug": provider
            }),
            inner_blocks: Vec::new(),
            inner_html: format!(
                r#"<figure class="wp-block-embed is-type-video is-provider-{}"><div class="wp-block-embed__wrapper">{}</div></figure>"#,
                provider,
                url
            ),
            inner_content: vec![],
        }
    }

    /// Create a gallery block
    pub fn gallery(images: Vec<GalleryImage>, columns: u8) -> Self {
        let columns = columns.clamp(1, 8);
        Self {
            id: Uuid::new_v4().to_string(),
            block_type: "core/gallery".to_string(),
            attributes: serde_json::json!({
                "images": images,
                "columns": columns
            }),
            inner_blocks: Vec::new(),
            inner_html: String::new(),
            inner_content: vec![],
        }
    }

    /// Create a table block
    pub fn table(headers: Vec<String>, rows: Vec<Vec<String>>) -> Self {
        let header_cells: Vec<serde_json::Value> = headers
            .iter()
            .map(|h| serde_json::json!({"content": h}))
            .collect();

        let body_rows: Vec<serde_json::Value> = rows
            .iter()
            .map(|row| {
                let cells: Vec<serde_json::Value> = row
                    .iter()
                    .map(|cell| serde_json::json!({"content": cell}))
                    .collect();
                serde_json::json!({"cells": cells})
            })
            .collect();

        Self {
            id: Uuid::new_v4().to_string(),
            block_type: "core/table".to_string(),
            attributes: serde_json::json!({
                "hasFixedLayout": false,
                "head": [{"cells": header_cells}],
                "body": body_rows
            }),
            inner_blocks: Vec::new(),
            inner_html: String::new(),
            inner_content: vec![],
        }
    }

    /// Create a cover block (image with overlay)
    pub fn cover(image_url: &str, overlay_color: &str, inner_blocks: Vec<Block>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            block_type: "core/cover".to_string(),
            attributes: serde_json::json!({
                "url": image_url,
                "overlayColor": overlay_color,
                "dimRatio": 50
            }),
            inner_blocks,
            inner_html: String::new(),
            inner_content: vec![],
        }
    }

    /// Create a group block (container for other blocks)
    pub fn group(inner_blocks: Vec<Block>, layout: Option<&str>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            block_type: "core/group".to_string(),
            attributes: serde_json::json!({
                "layout": {
                    "type": layout.unwrap_or("constrained")
                }
            }),
            inner_blocks,
            inner_html: String::new(),
            inner_content: vec![],
        }
    }
}

/// Gallery image for gallery blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryImage {
    pub url: String,
    pub alt: String,
    pub id: Option<Uuid>,
    pub caption: Option<String>,
}

/// Block renderer for converting blocks to HTML
pub struct BlockRenderer {
    /// Custom block renderers
    renderers: HashMap<String, Box<dyn Fn(&Block) -> String + Send + Sync>>,
}

impl BlockRenderer {
    /// Create a new block renderer
    pub fn new() -> Self {
        Self {
            renderers: HashMap::new(),
        }
    }

    /// Register a custom block renderer
    pub fn register<F>(&mut self, block_type: &str, renderer: F)
    where
        F: Fn(&Block) -> String + Send + Sync + 'static,
    {
        self.renderers.insert(block_type.to_string(), Box::new(renderer));
    }

    /// Render a single block to HTML
    pub fn render_block(&self, block: &Block) -> String {
        // Check for custom renderer
        if let Some(renderer) = self.renderers.get(&block.block_type) {
            return renderer(block);
        }

        // Default rendering based on block type
        match block.block_type.as_str() {
            "core/paragraph" => self.render_paragraph(block),
            "core/heading" => self.render_heading(block),
            "core/image" => self.render_image(block),
            "core/list" => self.render_list(block),
            "core/quote" => self.render_quote(block),
            "core/code" => self.render_code(block),
            "core/columns" => self.render_columns(block),
            "core/column" => self.render_column(block),
            "core/button" | "core/buttons" => self.render_button(block),
            "core/separator" => self.render_separator(block),
            "core/spacer" => self.render_spacer(block),
            "core/gallery" => self.render_gallery(block),
            "core/table" => self.render_table(block),
            "core/cover" => self.render_cover(block),
            "core/group" => self.render_group(block),
            "core/html" => self.render_html(block),
            _ if block.block_type.starts_with("core-embed/") => self.render_embed(block),
            _ => self.render_generic(block),
        }
    }

    /// Render multiple blocks to HTML
    pub fn render_blocks(&self, blocks: &[Block]) -> String {
        blocks.iter().map(|b| self.render_block(b)).collect()
    }

    fn render_paragraph(&self, block: &Block) -> String {
        let content = block.attributes.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let align = block.attributes.get("align")
            .and_then(|v| v.as_str())
            .map(|a| format!(r#" class="has-text-align-{}""#, a))
            .unwrap_or_default();

        format!("<p{}>{}</p>\n", align, content)
    }

    fn render_heading(&self, block: &Block) -> String {
        let content = block.attributes.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let level = block.attributes.get("level")
            .and_then(|v| v.as_u64())
            .unwrap_or(2)
            .clamp(1, 6);

        format!("<h{level}>{content}</h{level}>\n")
    }

    fn render_image(&self, block: &Block) -> String {
        let url = block.attributes.get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let alt = block.attributes.get("alt")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let caption = block.attributes.get("caption")
            .and_then(|v| v.as_str());
        let align = block.attributes.get("align")
            .and_then(|v| v.as_str())
            .map(|a| format!(" align{}", a))
            .unwrap_or_default();

        let mut html = format!(
            r#"<figure class="wp-block-image{}"><img src="{}" alt="{}"/>"#,
            align, url, alt
        );

        if let Some(cap) = caption {
            html.push_str(&format!("<figcaption>{}</figcaption>", cap));
        }

        html.push_str("</figure>\n");
        html
    }

    fn render_list(&self, block: &Block) -> String {
        let _ordered = block.attributes.get("ordered")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let values = block.attributes.get("values")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if values.is_empty() && !block.inner_html.is_empty() {
            return block.inner_html.clone();
        }

        format!("{}\n", values)
    }

    fn render_quote(&self, block: &Block) -> String {
        let value = block.attributes.get("value")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let citation = block.attributes.get("citation")
            .and_then(|v| v.as_str())
            .filter(|c| !c.is_empty());

        let mut html = format!(r#"<blockquote class="wp-block-quote">{}"#, value);

        if let Some(cite) = citation {
            html.push_str(&format!("<cite>{}</cite>", cite));
        }

        html.push_str("</blockquote>\n");
        html
    }

    fn render_code(&self, block: &Block) -> String {
        let content = block.attributes.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let language = block.attributes.get("language")
            .and_then(|v| v.as_str())
            .filter(|l| !l.is_empty());

        let lang_class = language
            .map(|l| format!(r#" class="language-{}""#, l))
            .unwrap_or_default();

        format!(
            r#"<pre class="wp-block-code"><code{}>{}</code></pre>"#,
            lang_class,
            ammonia::clean(content)
        )
    }

    fn render_columns(&self, block: &Block) -> String {
        let inner = self.render_blocks(&block.inner_blocks);
        format!(r#"<div class="wp-block-columns">{}</div>"#, inner)
    }

    fn render_column(&self, block: &Block) -> String {
        let inner = self.render_blocks(&block.inner_blocks);
        format!(r#"<div class="wp-block-column">{}</div>"#, inner)
    }

    fn render_button(&self, block: &Block) -> String {
        let text = block.attributes.get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let url = block.attributes.get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("#");
        let class = block.attributes.get("className")
            .and_then(|v| v.as_str())
            .unwrap_or("is-style-fill");

        format!(
            r#"<div class="wp-block-button {}"><a class="wp-block-button__link" href="{}">{}</a></div>"#,
            class, url, text
        )
    }

    fn render_separator(&self, block: &Block) -> String {
        let class = block.attributes.get("className")
            .and_then(|v| v.as_str())
            .unwrap_or("is-style-default");

        format!(r#"<hr class="wp-block-separator {}"/>"#, class)
    }

    fn render_spacer(&self, block: &Block) -> String {
        let height = block.attributes.get("height")
            .and_then(|v| v.as_u64())
            .unwrap_or(100);

        format!(
            r#"<div class="wp-block-spacer" style="height:{}px" aria-hidden="true"></div>"#,
            height
        )
    }

    fn render_gallery(&self, block: &Block) -> String {
        let columns = block.attributes.get("columns")
            .and_then(|v| v.as_u64())
            .unwrap_or(3);

        let images = block.attributes.get("images")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|img| {
                        let url = img.get("url")?.as_str()?;
                        let alt = img.get("alt").and_then(|v| v.as_str()).unwrap_or("");
                        Some(format!(
                            r#"<figure class="wp-block-image"><img src="{}" alt="{}"/></figure>"#,
                            url, alt
                        ))
                    })
                    .collect::<String>()
            })
            .unwrap_or_default();

        format!(
            r#"<figure class="wp-block-gallery has-nested-images columns-{}">{}</figure>"#,
            columns, images
        )
    }

    fn render_table(&self, block: &Block) -> String {
        let mut html = String::from(r#"<figure class="wp-block-table"><table>"#);

        // Render header
        if let Some(head) = block.attributes.get("head").and_then(|v| v.as_array()) {
            html.push_str("<thead>");
            for row in head {
                html.push_str("<tr>");
                if let Some(cells) = row.get("cells").and_then(|v| v.as_array()) {
                    for cell in cells {
                        let content = cell.get("content").and_then(|v| v.as_str()).unwrap_or("");
                        html.push_str(&format!("<th>{}</th>", content));
                    }
                }
                html.push_str("</tr>");
            }
            html.push_str("</thead>");
        }

        // Render body
        if let Some(body) = block.attributes.get("body").and_then(|v| v.as_array()) {
            html.push_str("<tbody>");
            for row in body {
                html.push_str("<tr>");
                if let Some(cells) = row.get("cells").and_then(|v| v.as_array()) {
                    for cell in cells {
                        let content = cell.get("content").and_then(|v| v.as_str()).unwrap_or("");
                        html.push_str(&format!("<td>{}</td>", content));
                    }
                }
                html.push_str("</tr>");
            }
            html.push_str("</tbody>");
        }

        html.push_str("</table></figure>");
        html
    }

    fn render_cover(&self, block: &Block) -> String {
        let url = block.attributes.get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let dim_ratio = block.attributes.get("dimRatio")
            .and_then(|v| v.as_u64())
            .unwrap_or(50);

        let inner = self.render_blocks(&block.inner_blocks);

        format!(
            r#"<div class="wp-block-cover" style="background-image:url({})">
                <span class="wp-block-cover__gradient-background has-background-dim-{}" aria-hidden="true"></span>
                <div class="wp-block-cover__inner-container">{}</div>
            </div>"#,
            url, dim_ratio, inner
        )
    }

    fn render_group(&self, block: &Block) -> String {
        let inner = self.render_blocks(&block.inner_blocks);
        format!(r#"<div class="wp-block-group">{}</div>"#, inner)
    }

    fn render_html(&self, block: &Block) -> String {
        // Raw HTML block - be careful with sanitization
        block.attributes.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    }

    fn render_embed(&self, block: &Block) -> String {
        let url = block.attributes.get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let provider = block.attributes.get("providerNameSlug")
            .and_then(|v| v.as_str())
            .unwrap_or("video");

        // Generate embed HTML based on provider
        let embed_html = match provider {
            "youtube" => generate_youtube_embed(url),
            "vimeo" => generate_vimeo_embed(url),
            "twitter" => generate_twitter_embed(url),
            _ => format!(r#"<a href="{}">{}</a>"#, url, url),
        };

        format!(
            r#"<figure class="wp-block-embed is-type-video is-provider-{}">
                <div class="wp-block-embed__wrapper">{}</div>
            </figure>"#,
            provider, embed_html
        )
    }

    fn render_generic(&self, block: &Block) -> String {
        // Fallback: use inner_html or render inner blocks
        if !block.inner_html.is_empty() {
            block.inner_html.clone()
        } else if !block.inner_blocks.is_empty() {
            self.render_blocks(&block.inner_blocks)
        } else {
            String::new()
        }
    }
}

impl Default for BlockRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate YouTube embed HTML
fn generate_youtube_embed(url: &str) -> String {
    // Extract video ID from YouTube URL
    let video_id = extract_youtube_id(url);
    if let Some(id) = video_id {
        format!(
            r#"<iframe width="560" height="315" src="https://www.youtube.com/embed/{}" frameborder="0" allowfullscreen></iframe>"#,
            id
        )
    } else {
        format!(r#"<a href="{}">{}</a>"#, url, url)
    }
}

/// Generate Vimeo embed HTML
fn generate_vimeo_embed(url: &str) -> String {
    // Extract video ID from Vimeo URL
    let video_id = extract_vimeo_id(url);
    if let Some(id) = video_id {
        format!(
            r#"<iframe src="https://player.vimeo.com/video/{}" width="640" height="360" frameborder="0" allowfullscreen></iframe>"#,
            id
        )
    } else {
        format!(r#"<a href="{}">{}</a>"#, url, url)
    }
}

/// Generate Twitter embed HTML
fn generate_twitter_embed(url: &str) -> String {
    format!(
        r#"<blockquote class="twitter-tweet"><a href="{}"></a></blockquote>
        <script async src="https://platform.twitter.com/widgets.js" charset="utf-8"></script>"#,
        url
    )
}

/// Extract YouTube video ID from URL
fn extract_youtube_id(url: &str) -> Option<String> {
    let re = regex::Regex::new(
        r"(?:youtube\.com/watch\?v=|youtu\.be/|youtube\.com/embed/)([a-zA-Z0-9_-]{11})"
    ).ok()?;
    re.captures(url).and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

/// Extract Vimeo video ID from URL
fn extract_vimeo_id(url: &str) -> Option<String> {
    let re = regex::Regex::new(r"vimeo\.com/(\d+)").ok()?;
    re.captures(url).and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

/// Block parser for converting block JSON to Block structs
pub struct BlockParser;

impl BlockParser {
    /// Parse JSON string to blocks
    pub fn parse(json: &str) -> Result<Vec<Block>, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Parse WordPress block comments from HTML
    pub fn parse_block_html(html: &str) -> Vec<Block> {
        let mut blocks = Vec::new();
        // Note: Rust's regex crate doesn't support backreferences, so we use a simpler pattern
        // and accept any closing tag format
        let block_regex = regex::Regex::new(
            r#"<!-- wp:([a-z-]+/[a-z-]+|[a-z-]+)(\s+(\{.*?\}))?\s*(/)?-->(.*?)(?:<!-- /wp:[a-z-/]+ -->)?"#
        ).unwrap();

        for cap in block_regex.captures_iter(html) {
            let block_type = cap.get(1).map(|m| m.as_str()).unwrap_or("core/paragraph");
            let attrs_json = cap.get(3).map(|m| m.as_str()).unwrap_or("{}");
            let inner_html = cap.get(5).map(|m| m.as_str()).unwrap_or("");

            let attributes: serde_json::Value = serde_json::from_str(attrs_json)
                .unwrap_or(serde_json::json!({}));

            blocks.push(Block {
                id: Uuid::new_v4().to_string(),
                block_type: if block_type.contains('/') {
                    block_type.to_string()
                } else {
                    format!("core/{}", block_type)
                },
                attributes,
                inner_blocks: Vec::new(),
                inner_html: inner_html.trim().to_string(),
                inner_content: vec![Some(inner_html.trim().to_string())],
            });
        }

        blocks
    }

    /// Serialize blocks to JSON
    pub fn serialize(blocks: &[Block]) -> Result<String, serde_json::Error> {
        serde_json::to_string(blocks)
    }

    /// Serialize blocks to WordPress block HTML format
    pub fn to_block_html(blocks: &[Block]) -> String {
        blocks.iter().map(|block| {
            let attrs = if block.attributes.as_object().map(|o| o.is_empty()).unwrap_or(true) {
                String::new()
            } else {
                format!(" {}", serde_json::to_string(&block.attributes).unwrap_or_default())
            };

            if block.inner_blocks.is_empty() && block.inner_html.is_empty() {
                format!("<!-- wp:{}{} /-->\n", block.block_type.replace("core/", ""), attrs)
            } else {
                let inner = if !block.inner_blocks.is_empty() {
                    Self::to_block_html(&block.inner_blocks)
                } else {
                    block.inner_html.clone()
                };

                format!(
                    "<!-- wp:{}{} -->\n{}\n<!-- /wp:{} -->\n",
                    block.block_type.replace("core/", ""),
                    attrs,
                    inner,
                    block.block_type.replace("core/", "")
                )
            }
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_paragraph() {
        let block = Block::paragraph("Hello, world!");
        assert_eq!(block.block_type, "core/paragraph");
        assert!(block.inner_html.contains("Hello, world!"));
    }

    #[test]
    fn test_render_blocks() {
        let blocks = vec![
            Block::heading("Welcome", 1),
            Block::paragraph("This is a test."),
        ];

        let renderer = BlockRenderer::new();
        let html = renderer.render_blocks(&blocks);

        assert!(html.contains("<h1>Welcome</h1>"));
        assert!(html.contains("<p>This is a test.</p>"));
    }

    #[test]
    fn test_parse_block_html() {
        let html = r#"<!-- wp:paragraph -->
        <p>Hello, world!</p>
        <!-- /wp:paragraph -->"#;

        let blocks = BlockParser::parse_block_html(html);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, "core/paragraph");
    }
}
