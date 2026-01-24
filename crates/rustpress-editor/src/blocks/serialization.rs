//! Block Serialization
//!
//! Convert blocks to/from various formats (HTML, Markdown, JSON).

use crate::blocks::{Block, BlockStyles, BlockType, ListType, Spacing};
use pulldown_cmark::{html, Options, Parser};

/// Block serializer for multiple formats
#[derive(Debug, Clone, Default)]
pub struct BlockSerializer;

impl BlockSerializer {
    /// Create a new block serializer
    pub fn new() -> Self {
        Self
    }

    /// Convert blocks to HTML
    pub fn to_html(&self, blocks: &[Block]) -> String {
        let mut html = String::new();
        for block in blocks {
            html.push_str(&self.block_to_html(block));
        }
        html
    }

    /// Convert a single block to HTML
    pub fn block_to_html(&self, block: &Block) -> String {
        let inner_html = Self::block_inner_html(block);
        let children_html = self.to_html(&block.children);

        let class_attr = if !block.css_classes.is_empty() {
            format!(r#" class="{}""#, block.css_classes.join(" "))
        } else {
            String::new()
        };

        let id_attr = block.meta.anchor.as_ref()
            .map(|a| format!(r#" id="{}""#, a))
            .unwrap_or_default();

        let style_attr = Self::styles_to_css(&block.styles);
        let style_str = if style_attr.is_empty() {
            String::new()
        } else {
            format!(r#" style="{}""#, style_attr)
        };

        match &block.block_type {
            BlockType::Paragraph => {
                format!("<p{}{}{}>{}</p>\n", id_attr, class_attr, style_str, inner_html)
            }
            BlockType::Heading => {
                let level = block.attributes.level.unwrap_or(2);
                format!(
                    "<h{level}{}{}{}>{}{}</h{level}>\n",
                    id_attr, class_attr, style_str, inner_html, children_html
                )
            }
            BlockType::List => {
                let is_ordered = matches!(block.attributes.list_type, Some(ListType::Ordered));
                let tag = if is_ordered { "ol" } else { "ul" };
                format!("<{tag}{}{}{}>\n{}</{tag}>\n", id_attr, class_attr, style_str, children_html)
            }
            BlockType::ListItem => {
                format!("<li{}{}{}>{}{}</li>\n", id_attr, class_attr, style_str, inner_html, children_html)
            }
            BlockType::Quote => {
                let cite = block.attributes.citation.as_ref()
                    .map(|c| format!("<cite>{}</cite>", c))
                    .unwrap_or_default();
                format!(
                    "<blockquote{}{}{}><p>{}</p>{}{}</blockquote>\n",
                    id_attr, class_attr, style_str, inner_html, cite, children_html
                )
            }
            BlockType::Code => {
                let lang_class = block.attributes.language.as_ref()
                    .map(|l| format!(r#" class="language-{}""#, l))
                    .unwrap_or_default();
                format!(
                    "<pre{}{}{}><code{}>{}</code></pre>\n",
                    id_attr, class_attr, style_str, lang_class,
                    html_escape(&inner_html)
                )
            }
            BlockType::Preformatted => {
                format!("<pre{}{}{}>{}</pre>\n", id_attr, class_attr, style_str, inner_html)
            }
            BlockType::Image => {
                let src = block.attributes.url.as_deref().unwrap_or("");
                let alt = block.attributes.alt.as_deref().unwrap_or("");
                let width = block.attributes.width.as_ref()
                    .map(|w| format!(r#" width="{}""#, w))
                    .unwrap_or_default();
                let height = block.attributes.height.as_ref()
                    .map(|h| format!(r#" height="{}""#, h))
                    .unwrap_or_default();

                let caption = block.attributes.caption.as_ref()
                    .map(|c| format!("<figcaption>{}</figcaption>", c))
                    .unwrap_or_default();

                format!(
                    r#"<figure{}{}{}><img src="{}" alt="{}"{}{} />{}</figure>"#,
                    id_attr, class_attr, style_str, src, alt, width, height, caption
                ) + "\n"
            }
            BlockType::Video => {
                let src = block.attributes.url.as_deref().unwrap_or("");
                let poster = block.attributes.poster.as_ref()
                    .map(|p| format!(r#" poster="{}""#, p))
                    .unwrap_or_default();
                let controls = if block.attributes.controls.unwrap_or(true) { " controls" } else { "" };
                let autoplay = if block.attributes.autoplay.unwrap_or(false) { " autoplay" } else { "" };
                let muted = if block.attributes.muted.unwrap_or(false) { " muted" } else { "" };
                let loop_attr = if block.attributes.loop_video.unwrap_or(false) { " loop" } else { "" };

                format!(
                    r#"<video src="{}"{}{}{}{}{}{}{}{}></video>"#,
                    src, poster, controls, autoplay, muted, loop_attr,
                    id_attr, class_attr, style_str
                ) + "\n"
            }
            BlockType::Audio => {
                let src = block.attributes.url.as_deref().unwrap_or("");
                format!(r#"<audio src="{}" controls{}{}{}></audio>"#, src, id_attr, class_attr, style_str) + "\n"
            }
            BlockType::Embed => {
                let url = block.attributes.url.as_deref().unwrap_or("");
                format!(
                    r#"<figure class="embed"{}{}"{}><iframe src="{}" frameborder="0" allowfullscreen></iframe></figure>"#,
                    class_attr, id_attr, style_str, url
                ) + "\n"
            }
            BlockType::Cover => {
                let bg_url = block.attributes.url.as_deref()
                    .map(|u| format!("background-image: url({});", u))
                    .unwrap_or_default();
                format!(
                    r#"<div class="cover"{}{}{} style="{}">{}</div>"#,
                    class_attr, id_attr, style_str, bg_url, children_html
                ) + "\n"
            }
            BlockType::Group => {
                format!("<div{}{}{}>{}</div>\n", id_attr, class_attr, style_str, children_html)
            }
            BlockType::Section => {
                format!("<section{}{}{}>{}</section>\n", id_attr, class_attr, style_str, children_html)
            }
            BlockType::Columns => {
                let cols = block.children.len();
                format!(
                    r#"<div class="columns cols-{}"{}{}"{}>{}</div>"#,
                    cols, class_attr, id_attr, style_str, children_html
                ) + "\n"
            }
            BlockType::Column => {
                let width = block.attributes.width.as_ref()
                    .map(|w| format!("width: {};", w))
                    .unwrap_or_default();
                format!(
                    r#"<div class="column"{}{} style="{}{}">{}</div>"#,
                    class_attr, id_attr, width, style_attr, children_html
                )
            }
            BlockType::Spacer => {
                let height = block.attributes.spacer_height.as_deref().unwrap_or("100px");
                format!(r#"<div class="spacer" style="height: {};"></div>"#, height) + "\n"
            }
            BlockType::Separator => {
                format!("<hr{}{}{}>\n", id_attr, class_attr, style_str)
            }
            BlockType::Button => {
                let url = block.attributes.href.as_deref().unwrap_or("#");
                let text = block.attributes.button_text.as_deref()
                    .or(block.attributes.content.as_deref())
                    .unwrap_or("Button");
                format!(
                    r#"<a href="{}" class="button"{}{}{}>{}</a>"#,
                    url, class_attr, id_attr, style_str, text
                ) + "\n"
            }
            BlockType::Buttons => {
                format!(
                    r#"<div class="buttons"{}{}{}>{}</div>"#,
                    class_attr, id_attr, style_str, children_html
                ) + "\n"
            }
            BlockType::Table => {
                if let Some(table_data) = &block.attributes.table_data {
                    let header_row = if table_data.has_header_row && !table_data.headers.is_empty() {
                        let headers = table_data.headers.iter()
                            .map(|h| format!("<th>{}</th>", h))
                            .collect::<Vec<_>>()
                            .join("");
                        format!("<thead><tr>{}</tr></thead>", headers)
                    } else {
                        String::new()
                    };

                    let body_rows = table_data.rows.iter()
                        .map(|row| {
                            let cells = row.iter()
                                .map(|c| format!("<td>{}</td>", c))
                                .collect::<Vec<_>>()
                                .join("");
                            format!("<tr>{}</tr>", cells)
                        })
                        .collect::<Vec<_>>()
                        .join("");

                    format!("<table{}{}{}>{}<tbody>{}</tbody></table>\n", id_attr, class_attr, style_str, header_row, body_rows)
                } else {
                    format!("<table{}{}{}>{}</table>\n", id_attr, class_attr, style_str, children_html)
                }
            }
            BlockType::Html | BlockType::CustomHtml => {
                block.attributes.content.clone().unwrap_or_default()
            }
            BlockType::PullQuote => {
                let cite = block.attributes.citation.as_ref()
                    .map(|c| format!("<cite>{}</cite>", c))
                    .unwrap_or_default();
                format!(
                    r#"<figure class="pullquote"{}{}{}><blockquote>{}</blockquote>{}</figure>"#,
                    class_attr, id_attr, style_str, inner_html, cite
                ) + "\n"
            }
            BlockType::Accordion => {
                format!("<div class=\"accordion\"{}{}{}>{}</div>\n", id_attr, class_attr, style_str, children_html)
            }
            BlockType::AccordionItem => {
                let title = block.attributes.content.as_deref().unwrap_or("Accordion Item");
                let open = if block.attributes.default_open.unwrap_or(false) { " open" } else { "" };
                format!(
                    r#"<details{}{}{}{}><summary>{}</summary><div class="accordion-content">{}</div></details>"#,
                    open, id_attr, class_attr, style_str, title, children_html
                ) + "\n"
            }
            BlockType::Tabs => {
                format!("<div class=\"tabs\"{}{}{}>{}</div>\n", id_attr, class_attr, style_str, children_html)
            }
            BlockType::Tab => {
                let title = block.attributes.content.as_deref().unwrap_or("Tab");
                format!(
                    r#"<div class="tab-panel" data-title="{}"{}{}{}>{}</div>"#,
                    title, id_attr, class_attr, style_str, children_html
                ) + "\n"
            }
            BlockType::Alert => {
                let alert_type = block.attributes.alert_type.as_ref()
                    .map(|t| format!("{:?}", t).to_lowercase())
                    .unwrap_or_else(|| "info".to_string());
                format!(
                    r#"<div class="alert alert-{}"{}{}{}>{}</div>"#,
                    alert_type, id_attr, class_attr, style_str, inner_html
                ) + "\n"
            }
            _ => {
                // Default rendering
                format!("<div{}{}{}>{}{}</div>\n", id_attr, class_attr, style_str, inner_html, children_html)
            }
        }
    }

    fn block_inner_html(block: &Block) -> String {
        block.attributes.content.clone().unwrap_or_default()
    }

    fn styles_to_css(styles: &BlockStyles) -> String {
        let mut css = Vec::new();

        if let Some(bg) = &styles.background_color {
            css.push(format!("background-color: {}", bg));
        }
        if let Some(color) = &styles.text_color {
            css.push(format!("color: {}", color));
        }
        if let Some(size) = &styles.font_size {
            css.push(format!("font-size: {}", size));
        }
        if let Some(lh) = &styles.line_height {
            css.push(format!("line-height: {}", lh));
        }

        if let Some(padding) = &styles.padding {
            match padding {
                Spacing::All(value) => css.push(format!("padding: {}", value)),
                Spacing::Individual { top, right, bottom, left } => {
                    css.push(format!("padding: {} {} {} {}",
                        top.as_deref().unwrap_or("0"),
                        right.as_deref().unwrap_or("0"),
                        bottom.as_deref().unwrap_or("0"),
                        left.as_deref().unwrap_or("0"),
                    ));
                }
            }
        }
        if let Some(margin) = &styles.margin {
            match margin {
                Spacing::All(value) => css.push(format!("margin: {}", value)),
                Spacing::Individual { top, right, bottom, left } => {
                    css.push(format!("margin: {} {} {} {}",
                        top.as_deref().unwrap_or("0"),
                        right.as_deref().unwrap_or("0"),
                        bottom.as_deref().unwrap_or("0"),
                        left.as_deref().unwrap_or("0"),
                    ));
                }
            }
        }

        if let Some(br) = &styles.border_radius {
            css.push(format!("border-radius: {}", br));
        }
        if let Some(bw) = &styles.border_width {
            css.push(format!("border-width: {}", bw));
        }
        if let Some(bc) = &styles.border_color {
            css.push(format!("border-color: {}", bc));
        }
        if let Some(bs) = &styles.border_style {
            css.push(format!("border-style: {}", bs));
        }
        if let Some(shadow) = &styles.box_shadow {
            css.push(format!("box-shadow: {}", shadow));
        }

        css.join("; ")
    }

    /// Convert blocks to plain text
    pub fn to_plain_text(&self, blocks: &[Block]) -> String {
        let mut text = String::new();
        for block in blocks {
            text.push_str(&self.block_to_plain_text(block));
            text.push('\n');
        }
        text.trim().to_string()
    }

    fn block_to_plain_text(&self, block: &Block) -> String {
        let mut text = String::new();

        if let Some(content) = &block.attributes.content {
            text.push_str(&strip_html(content));
        }

        for child in &block.children {
            text.push_str(&self.block_to_plain_text(child));
            text.push('\n');
        }

        text
    }

    /// Convert blocks to Markdown
    pub fn to_markdown(&self, blocks: &[Block]) -> String {
        let mut md = String::new();
        for block in blocks {
            md.push_str(&self.block_to_markdown(block));
            md.push_str("\n\n");
        }
        md.trim().to_string()
    }

    fn block_to_markdown(&self, block: &Block) -> String {
        let content = block.attributes.content.as_deref().unwrap_or("");

        match &block.block_type {
            BlockType::Paragraph => content.to_string(),
            BlockType::Heading => {
                let level = block.attributes.level.unwrap_or(2);
                let prefix = "#".repeat(level as usize);
                format!("{} {}", prefix, content)
            }
            BlockType::List => {
                let is_ordered = matches!(block.attributes.list_type, Some(ListType::Ordered));
                block.children.iter().enumerate()
                    .map(|(i, child)| {
                        let child_content = child.attributes.content.as_deref().unwrap_or("");
                        if is_ordered {
                            format!("{}. {}", i + 1, child_content)
                        } else {
                            format!("- {}", child_content)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            BlockType::ListItem => content.to_string(),
            BlockType::Quote => {
                let quote = content.lines()
                    .map(|line| format!("> {}", line))
                    .collect::<Vec<_>>()
                    .join("\n");
                if let Some(cite) = &block.attributes.citation {
                    format!("{}\n> — {}", quote, cite)
                } else {
                    quote
                }
            }
            BlockType::Code => {
                let lang = block.attributes.language.as_deref().unwrap_or("");
                format!("```{}\n{}\n```", lang, content)
            }
            BlockType::Image => {
                let alt = block.attributes.alt.as_deref().unwrap_or("");
                let url = block.attributes.url.as_deref().unwrap_or("");
                format!("![{}]({})", alt, url)
            }
            BlockType::Separator => "---".to_string(),
            BlockType::Html | BlockType::CustomHtml => content.to_string(),
            _ => {
                // For unsupported blocks, try to extract text content
                let children_md: String = block.children.iter()
                    .map(|c| self.block_to_markdown(c))
                    .collect::<Vec<_>>()
                    .join("\n\n");

                if !content.is_empty() && !children_md.is_empty() {
                    format!("{}\n\n{}", content, children_md)
                } else if !content.is_empty() {
                    content.to_string()
                } else {
                    children_md
                }
            }
        }
    }

    /// Parse Markdown into blocks
    pub fn from_markdown(&self, markdown: &str) -> Vec<Block> {
        // Convert markdown to HTML first
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);

        let parser = Parser::new_ext(markdown, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        // For now, return a single HTML block
        // A more sophisticated parser would convert to proper blocks
        let mut block = Block::new(BlockType::Html);
        block.attributes.content = Some(html_output);
        vec![block]
    }

    /// Convert blocks to JSON
    pub fn to_json(&self, blocks: &[Block]) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(blocks)
    }

    /// Parse JSON into blocks
    pub fn from_json(&self, json: &str) -> Result<Vec<Block>, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Escape HTML special characters
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Strip HTML tags from text
fn strip_html(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for c in html.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(c);
        }
    }

    result
}
