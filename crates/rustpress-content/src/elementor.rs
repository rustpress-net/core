//! Elementor Page Builder Compatibility Layer
//!
//! Provides compatibility with Elementor's JSON-based page structure,
//! allowing import/export of Elementor templates and rendering of Elementor content.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::blocks::Block;

/// Elementor document structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementorDocument {
    /// Document version
    #[serde(default)]
    pub version: String,

    /// Document title
    #[serde(default)]
    pub title: String,

    /// Document type (page, section, etc.)
    #[serde(rename = "type", default)]
    pub doc_type: String,

    /// Page settings
    #[serde(default)]
    pub page_settings: serde_json::Value,

    /// Root elements
    #[serde(default)]
    pub content: Vec<ElementorElement>,
}

/// Elementor element (widget, section, column)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementorElement {
    /// Element ID
    pub id: String,

    /// Element type (section, column, widget)
    #[serde(rename = "elType")]
    pub el_type: String,

    /// Widget type (for widgets only)
    #[serde(rename = "widgetType", default)]
    pub widget_type: Option<String>,

    /// Element settings
    #[serde(default)]
    pub settings: serde_json::Value,

    /// Child elements
    #[serde(default)]
    pub elements: Vec<ElementorElement>,

    /// Custom ID
    #[serde(rename = "customId", default)]
    pub custom_id: Option<String>,

    /// CSS classes
    #[serde(rename = "customClasses", default)]
    pub custom_classes: Option<String>,
}

impl ElementorElement {
    /// Create a new section
    pub fn section(elements: Vec<ElementorElement>) -> Self {
        Self {
            id: generate_elementor_id(),
            el_type: "section".to_string(),
            widget_type: None,
            settings: serde_json::json!({}),
            elements,
            custom_id: None,
            custom_classes: None,
        }
    }

    /// Create a new column
    pub fn column(elements: Vec<ElementorElement>, width: u8) -> Self {
        Self {
            id: generate_elementor_id(),
            el_type: "column".to_string(),
            widget_type: None,
            settings: serde_json::json!({
                "_column_size": width
            }),
            elements,
            custom_id: None,
            custom_classes: None,
        }
    }

    /// Create a widget
    pub fn widget(widget_type: &str, settings: serde_json::Value) -> Self {
        Self {
            id: generate_elementor_id(),
            el_type: "widget".to_string(),
            widget_type: Some(widget_type.to_string()),
            settings,
            elements: Vec::new(),
            custom_id: None,
            custom_classes: None,
        }
    }

    /// Create a heading widget
    pub fn heading(text: &str, tag: &str, align: &str) -> Self {
        Self::widget("heading", serde_json::json!({
            "title": text,
            "header_size": tag,
            "align": align
        }))
    }

    /// Create a text editor widget
    pub fn text_editor(content: &str) -> Self {
        Self::widget("text-editor", serde_json::json!({
            "editor": content
        }))
    }

    /// Create an image widget
    pub fn image(url: &str, alt: &str, link: Option<&str>) -> Self {
        let mut settings = serde_json::json!({
            "image": {
                "url": url,
                "alt": alt
            }
        });

        if let Some(link_url) = link {
            settings["link"] = serde_json::json!({
                "url": link_url
            });
        }

        Self::widget("image", settings)
    }

    /// Create a button widget
    pub fn button(text: &str, url: &str, button_type: &str) -> Self {
        Self::widget("button", serde_json::json!({
            "text": text,
            "link": {
                "url": url
            },
            "button_type": button_type
        }))
    }

    /// Create a video widget
    pub fn video(url: &str, provider: &str) -> Self {
        Self::widget("video", serde_json::json!({
            "video_type": provider,
            "youtube_url": if provider == "youtube" { url } else { "" },
            "vimeo_url": if provider == "vimeo" { url } else { "" },
            "hosted_url": if provider == "hosted" { serde_json::json!({"url": url}) } else { serde_json::json!({}) }
        }))
    }

    /// Create a divider widget
    pub fn divider(style: &str) -> Self {
        Self::widget("divider", serde_json::json!({
            "style": style
        }))
    }

    /// Create a spacer widget
    pub fn spacer(space: u32) -> Self {
        Self::widget("spacer", serde_json::json!({
            "space": {
                "size": space,
                "unit": "px"
            }
        }))
    }

    /// Create an icon widget
    pub fn icon(icon: &str, link: Option<&str>) -> Self {
        let mut settings = serde_json::json!({
            "selected_icon": {
                "value": icon,
                "library": "fa-solid"
            }
        });

        if let Some(link_url) = link {
            settings["link"] = serde_json::json!({
                "url": link_url
            });
        }

        Self::widget("icon", settings)
    }

    /// Create an icon list widget
    pub fn icon_list(items: Vec<(&str, &str, Option<&str>)>) -> Self {
        let icon_list: Vec<serde_json::Value> = items
            .iter()
            .map(|(icon, text, link)| {
                let mut item = serde_json::json!({
                    "text": text,
                    "selected_icon": {
                        "value": icon,
                        "library": "fa-solid"
                    }
                });
                if let Some(url) = link {
                    item["link"] = serde_json::json!({"url": url});
                }
                item
            })
            .collect();

        Self::widget("icon-list", serde_json::json!({
            "icon_list": icon_list
        }))
    }

    /// Create a counter widget
    pub fn counter(starting: u64, ending: u64, title: &str) -> Self {
        Self::widget("counter", serde_json::json!({
            "starting_number": starting,
            "ending_number": ending,
            "title": title
        }))
    }

    /// Create a progress bar widget
    pub fn progress_bar(title: &str, percent: u8) -> Self {
        Self::widget("progress", serde_json::json!({
            "title": title,
            "percent": percent
        }))
    }

    /// Create a testimonial widget
    pub fn testimonial(content: &str, name: &str, title: &str, image_url: Option<&str>) -> Self {
        let mut settings = serde_json::json!({
            "testimonial_content": content,
            "testimonial_name": name,
            "testimonial_job": title
        });

        if let Some(url) = image_url {
            settings["testimonial_image"] = serde_json::json!({
                "url": url
            });
        }

        Self::widget("testimonial", settings)
    }

    /// Create a tabs widget
    pub fn tabs(tabs: Vec<(&str, &str)>) -> Self {
        let tab_items: Vec<serde_json::Value> = tabs
            .iter()
            .map(|(title, content)| {
                serde_json::json!({
                    "tab_title": title,
                    "tab_content": content
                })
            })
            .collect();

        Self::widget("tabs", serde_json::json!({
            "tabs": tab_items
        }))
    }

    /// Create an accordion widget
    pub fn accordion(items: Vec<(&str, &str)>) -> Self {
        let accordion_items: Vec<serde_json::Value> = items
            .iter()
            .map(|(title, content)| {
                serde_json::json!({
                    "tab_title": title,
                    "tab_content": content
                })
            })
            .collect();

        Self::widget("accordion", serde_json::json!({
            "tabs": accordion_items
        }))
    }

    /// Create a toggle widget
    pub fn toggle(items: Vec<(&str, &str)>) -> Self {
        let toggle_items: Vec<serde_json::Value> = items
            .iter()
            .map(|(title, content)| {
                serde_json::json!({
                    "tab_title": title,
                    "tab_content": content
                })
            })
            .collect();

        Self::widget("toggle", serde_json::json!({
            "tabs": toggle_items
        }))
    }

    /// Create a social icons widget
    pub fn social_icons(icons: Vec<(&str, &str)>) -> Self {
        let social_items: Vec<serde_json::Value> = icons
            .iter()
            .map(|(platform, url)| {
                serde_json::json!({
                    "social_icon": {
                        "value": format!("fab fa-{}", platform),
                        "library": "fa-brands"
                    },
                    "link": {
                        "url": url
                    }
                })
            })
            .collect();

        Self::widget("social-icons", serde_json::json!({
            "social_icon_list": social_items
        }))
    }

    /// Create an alert widget
    pub fn alert(title: &str, description: &str, alert_type: &str) -> Self {
        Self::widget("alert", serde_json::json!({
            "alert_title": title,
            "alert_description": description,
            "alert_type": alert_type
        }))
    }

    /// Create a google maps widget
    pub fn google_maps(address: &str, zoom: u8, height: u32) -> Self {
        Self::widget("google_maps", serde_json::json!({
            "address": address,
            "zoom": {
                "size": zoom
            },
            "height": {
                "size": height,
                "unit": "px"
            }
        }))
    }

    /// Create a form widget
    pub fn form(form_name: &str, fields: Vec<ElementorFormField>) -> Self {
        let form_fields: Vec<serde_json::Value> = fields
            .iter()
            .map(|f| f.to_json())
            .collect();

        Self::widget("form", serde_json::json!({
            "form_name": form_name,
            "form_fields": form_fields,
            "submit_actions": ["email"]
        }))
    }
}

/// Elementor form field
#[derive(Debug, Clone)]
pub struct ElementorFormField {
    pub field_type: String,
    pub label: String,
    pub placeholder: Option<String>,
    pub required: bool,
    pub width: u8,
}

impl ElementorFormField {
    pub fn new(field_type: &str, label: &str) -> Self {
        Self {
            field_type: field_type.to_string(),
            label: label.to_string(),
            placeholder: None,
            required: false,
            width: 100,
        }
    }

    pub fn text(label: &str) -> Self {
        Self::new("text", label)
    }

    pub fn email(label: &str) -> Self {
        Self::new("email", label)
    }

    pub fn textarea(label: &str) -> Self {
        Self::new("textarea", label)
    }

    pub fn with_placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = Some(placeholder.to_string());
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn with_width(mut self, width: u8) -> Self {
        self.width = width;
        self
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "field_type": self.field_type,
            "field_label": self.label,
            "placeholder": self.placeholder.as_deref().unwrap_or(""),
            "required": if self.required { "true" } else { "" },
            "width": self.width.to_string()
        })
    }
}

/// Generate Elementor-style ID
fn generate_elementor_id() -> String {
    let uuid = Uuid::new_v4();
    let bytes = uuid.as_bytes();
    // Elementor uses 7-character alphanumeric IDs
    let chars: String = bytes
        .iter()
        .take(7)
        .map(|b| {
            let idx = (b % 36) as usize;
            if idx < 10 {
                (b'0' + idx as u8) as char
            } else {
                (b'a' + (idx - 10) as u8) as char
            }
        })
        .collect();
    chars
}

/// Elementor renderer for converting Elementor elements to HTML
pub struct ElementorRenderer {
    /// Widget renderers
    widget_renderers: HashMap<String, Box<dyn Fn(&ElementorElement) -> String + Send + Sync>>,
}

impl ElementorRenderer {
    /// Create a new renderer
    pub fn new() -> Self {
        Self {
            widget_renderers: HashMap::new(),
        }
    }

    /// Register a custom widget renderer
    pub fn register_widget<F>(&mut self, widget_type: &str, renderer: F)
    where
        F: Fn(&ElementorElement) -> String + Send + Sync + 'static,
    {
        self.widget_renderers.insert(widget_type.to_string(), Box::new(renderer));
    }

    /// Render Elementor JSON content to HTML
    pub fn render(&self, content: &str) -> String {
        // Parse the Elementor JSON
        let doc: Result<ElementorDocument, _> = serde_json::from_str(content);

        match doc {
            Ok(document) => {
                self.render_elements(&document.content)
            }
            Err(_) => {
                // Try parsing as array of elements directly
                let elements: Result<Vec<ElementorElement>, _> = serde_json::from_str(content);
                match elements {
                    Ok(elems) => self.render_elements(&elems),
                    Err(_) => content.to_string(), // Return as-is if not valid JSON
                }
            }
        }
    }

    /// Render elements to HTML
    pub fn render_elements(&self, elements: &[ElementorElement]) -> String {
        elements
            .iter()
            .map(|el| self.render_element(el))
            .collect()
    }

    /// Render a single element
    pub fn render_element(&self, element: &ElementorElement) -> String {
        match element.el_type.as_str() {
            "section" => self.render_section(element),
            "column" => self.render_column(element),
            "widget" => self.render_widget(element),
            "container" => self.render_container(element),
            _ => self.render_elements(&element.elements),
        }
    }

    fn render_section(&self, element: &ElementorElement) -> String {
        let inner = self.render_elements(&element.elements);
        let classes = self.get_element_classes(element, "elementor-section");
        let id = element.custom_id.as_deref().unwrap_or(&element.id);

        format!(
            r#"<section class="{}" id="elementor-{}" data-id="{}">
                <div class="elementor-container">{}</div>
            </section>"#,
            classes, id, element.id, inner
        )
    }

    fn render_column(&self, element: &ElementorElement) -> String {
        let inner = self.render_elements(&element.elements);
        let classes = self.get_element_classes(element, "elementor-column");
        let width = element.settings.get("_column_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(100);

        format!(
            r#"<div class="{}" style="width: {}%">
                <div class="elementor-widget-wrap">{}</div>
            </div>"#,
            classes, width, inner
        )
    }

    fn render_container(&self, element: &ElementorElement) -> String {
        let inner = self.render_elements(&element.elements);
        let classes = self.get_element_classes(element, "elementor-container");

        format!(
            r#"<div class="{}">{}</div>"#,
            classes, inner
        )
    }

    fn render_widget(&self, element: &ElementorElement) -> String {
        let widget_type = element.widget_type.as_deref().unwrap_or("unknown");

        // Check for custom renderer
        if let Some(renderer) = self.widget_renderers.get(widget_type) {
            return renderer(element);
        }

        // Default widget rendering
        let content = match widget_type {
            "heading" => self.render_heading_widget(element),
            "text-editor" => self.render_text_editor_widget(element),
            "image" => self.render_image_widget(element),
            "button" => self.render_button_widget(element),
            "video" => self.render_video_widget(element),
            "divider" => self.render_divider_widget(element),
            "spacer" => self.render_spacer_widget(element),
            "icon" => self.render_icon_widget(element),
            "icon-list" => self.render_icon_list_widget(element),
            "counter" => self.render_counter_widget(element),
            "progress" => self.render_progress_widget(element),
            "testimonial" => self.render_testimonial_widget(element),
            "tabs" => self.render_tabs_widget(element),
            "accordion" | "toggle" => self.render_accordion_widget(element),
            "social-icons" => self.render_social_icons_widget(element),
            "alert" => self.render_alert_widget(element),
            "google_maps" => self.render_maps_widget(element),
            "form" => self.render_form_widget(element),
            "html" => self.render_html_widget(element),
            "shortcode" => self.render_shortcode_widget(element),
            _ => format!("<!-- Widget: {} -->", widget_type),
        };

        let classes = self.get_element_classes(element, &format!("elementor-widget-{}", widget_type));

        format!(
            r#"<div class="elementor-widget {}">
                <div class="elementor-widget-container">{}</div>
            </div>"#,
            classes, content
        )
    }

    fn get_element_classes(&self, element: &ElementorElement, base_class: &str) -> String {
        let mut classes = vec![base_class.to_string()];

        if let Some(ref custom) = element.custom_classes {
            classes.push(custom.clone());
        }

        classes.join(" ")
    }

    fn render_heading_widget(&self, element: &ElementorElement) -> String {
        let title = element.settings.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let tag = element.settings.get("header_size")
            .and_then(|v| v.as_str())
            .unwrap_or("h2");
        let align = element.settings.get("align")
            .and_then(|v| v.as_str())
            .unwrap_or("left");

        format!(
            r#"<{} class="elementor-heading-title" style="text-align: {}">{}</{}>"#,
            tag, align, title, tag
        )
    }

    fn render_text_editor_widget(&self, element: &ElementorElement) -> String {
        element.settings.get("editor")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    }

    fn render_image_widget(&self, element: &ElementorElement) -> String {
        let image = element.settings.get("image");
        let url = image
            .and_then(|i| i.get("url"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let alt = image
            .and_then(|i| i.get("alt"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let link = element.settings.get("link")
            .and_then(|l| l.get("url"))
            .and_then(|v| v.as_str());

        let img_html = format!(r#"<img src="{}" alt="{}"/>"#, url, alt);

        if let Some(link_url) = link {
            format!(r#"<a href="{}">{}</a>"#, link_url, img_html)
        } else {
            img_html
        }
    }

    fn render_button_widget(&self, element: &ElementorElement) -> String {
        let text = element.settings.get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("Button");
        let url = element.settings.get("link")
            .and_then(|l| l.get("url"))
            .and_then(|v| v.as_str())
            .unwrap_or("#");
        let btn_type = element.settings.get("button_type")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        format!(
            r#"<a class="elementor-button elementor-button-{}" href="{}">{}</a>"#,
            btn_type, url, text
        )
    }

    fn render_video_widget(&self, element: &ElementorElement) -> String {
        let video_type = element.settings.get("video_type")
            .and_then(|v| v.as_str())
            .unwrap_or("youtube");

        let url = match video_type {
            "youtube" => element.settings.get("youtube_url").and_then(|v| v.as_str()),
            "vimeo" => element.settings.get("vimeo_url").and_then(|v| v.as_str()),
            _ => element.settings.get("hosted_url")
                .and_then(|v| v.get("url"))
                .and_then(|v| v.as_str()),
        };

        let url = url.unwrap_or("");

        match video_type {
            "youtube" => {
                let video_id = extract_youtube_id(url);
                if let Some(id) = video_id {
                    format!(
                        r#"<div class="elementor-video"><iframe src="https://www.youtube.com/embed/{}" allowfullscreen></iframe></div>"#,
                        id
                    )
                } else {
                    format!(r#"<a href="{}">{}</a>"#, url, url)
                }
            }
            "vimeo" => {
                let video_id = extract_vimeo_id(url);
                if let Some(id) = video_id {
                    format!(
                        r#"<div class="elementor-video"><iframe src="https://player.vimeo.com/video/{}" allowfullscreen></iframe></div>"#,
                        id
                    )
                } else {
                    format!(r#"<a href="{}">{}</a>"#, url, url)
                }
            }
            _ => format!(r#"<video src="{}" controls></video>"#, url),
        }
    }

    fn render_divider_widget(&self, element: &ElementorElement) -> String {
        let style = element.settings.get("style")
            .and_then(|v| v.as_str())
            .unwrap_or("solid");

        format!(r#"<div class="elementor-divider"><span class="elementor-divider-separator" style="border-style: {}"></span></div>"#, style)
    }

    fn render_spacer_widget(&self, element: &ElementorElement) -> String {
        let space = element.settings.get("space")
            .and_then(|v| v.get("size"))
            .and_then(|v| v.as_u64())
            .unwrap_or(50);

        format!(r#"<div class="elementor-spacer" style="height: {}px"></div>"#, space)
    }

    fn render_icon_widget(&self, element: &ElementorElement) -> String {
        let icon = element.settings.get("selected_icon")
            .and_then(|v| v.get("value"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let link = element.settings.get("link")
            .and_then(|l| l.get("url"))
            .and_then(|v| v.as_str());

        let icon_html = format!(r#"<i class="{}"></i>"#, icon);

        if let Some(url) = link {
            format!(r#"<a href="{}">{}</a>"#, url, icon_html)
        } else {
            icon_html
        }
    }

    fn render_icon_list_widget(&self, element: &ElementorElement) -> String {
        let items = element.settings.get("icon_list")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|item| {
                        let icon = item.get("selected_icon")
                            .and_then(|v| v.get("value"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let text = item.get("text")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let link = item.get("link")
                            .and_then(|l| l.get("url"))
                            .and_then(|v| v.as_str());

                        let content = format!(r#"<i class="{}"></i><span>{}</span>"#, icon, text);

                        if let Some(url) = link {
                            format!(r#"<li><a href="{}">{}</a></li>"#, url, content)
                        } else {
                            format!(r#"<li>{}</li>"#, content)
                        }
                    })
                    .collect::<String>()
            })
            .unwrap_or_default();

        format!(r#"<ul class="elementor-icon-list">{}</ul>"#, items)
    }

    fn render_counter_widget(&self, element: &ElementorElement) -> String {
        let ending = element.settings.get("ending_number")
            .and_then(|v| v.as_u64())
            .unwrap_or(100);
        let title = element.settings.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        format!(
            r#"<div class="elementor-counter">
                <div class="elementor-counter-number">{}</div>
                <div class="elementor-counter-title">{}</div>
            </div>"#,
            ending, title
        )
    }

    fn render_progress_widget(&self, element: &ElementorElement) -> String {
        let title = element.settings.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let percent = element.settings.get("percent")
            .and_then(|v| v.as_u64())
            .unwrap_or(50);

        format!(
            r#"<div class="elementor-progress">
                <div class="elementor-progress-title">{}</div>
                <div class="elementor-progress-wrapper">
                    <div class="elementor-progress-bar" style="width: {}%"></div>
                </div>
                <span class="elementor-progress-percentage">{}%</span>
            </div>"#,
            title, percent, percent
        )
    }

    fn render_testimonial_widget(&self, element: &ElementorElement) -> String {
        let content = element.settings.get("testimonial_content")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let name = element.settings.get("testimonial_name")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let job = element.settings.get("testimonial_job")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let image = element.settings.get("testimonial_image")
            .and_then(|v| v.get("url"))
            .and_then(|v| v.as_str());

        let img_html = image
            .map(|url| format!(r#"<img src="{}" alt="{}"/>"#, url, name))
            .unwrap_or_default();

        format!(
            r#"<div class="elementor-testimonial">
                <div class="elementor-testimonial-content">{}</div>
                {}
                <div class="elementor-testimonial-name">{}</div>
                <div class="elementor-testimonial-job">{}</div>
            </div>"#,
            content, img_html, name, job
        )
    }

    fn render_tabs_widget(&self, element: &ElementorElement) -> String {
        let tabs = element.settings.get("tabs")
            .and_then(|v| v.as_array());

        if let Some(tabs) = tabs {
            let nav: String = tabs.iter().enumerate()
                .map(|(i, tab)| {
                    let title = tab.get("tab_title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    format!(
                        r#"<li class="elementor-tab-title{}" data-tab="{}">{}</li>"#,
                        if i == 0 { " elementor-active" } else { "" },
                        i + 1,
                        title
                    )
                })
                .collect();

            let content: String = tabs.iter().enumerate()
                .map(|(i, tab)| {
                    let content = tab.get("tab_content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    format!(
                        r#"<div class="elementor-tab-content{}" data-tab="{}">{}</div>"#,
                        if i == 0 { " elementor-active" } else { "" },
                        i + 1,
                        content
                    )
                })
                .collect();

            format!(
                r#"<div class="elementor-tabs">
                    <ul class="elementor-tabs-wrapper">{}</ul>
                    <div class="elementor-tabs-content-wrapper">{}</div>
                </div>"#,
                nav, content
            )
        } else {
            String::new()
        }
    }

    fn render_accordion_widget(&self, element: &ElementorElement) -> String {
        let items = element.settings.get("tabs")
            .and_then(|v| v.as_array());

        if let Some(items) = items {
            let content: String = items.iter().enumerate()
                .map(|(i, item)| {
                    let title = item.get("tab_title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let content = item.get("tab_content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    format!(
                        r#"<div class="elementor-accordion-item">
                            <div class="elementor-accordion-title{}">{}</div>
                            <div class="elementor-accordion-content{}">{}</div>
                        </div>"#,
                        if i == 0 { " elementor-active" } else { "" },
                        title,
                        if i == 0 { " elementor-active" } else { "" },
                        content
                    )
                })
                .collect();

            format!(r#"<div class="elementor-accordion">{}</div>"#, content)
        } else {
            String::new()
        }
    }

    fn render_social_icons_widget(&self, element: &ElementorElement) -> String {
        let icons = element.settings.get("social_icon_list")
            .and_then(|v| v.as_array());

        if let Some(icons) = icons {
            let content: String = icons.iter()
                .map(|icon| {
                    let icon_class = icon.get("social_icon")
                        .and_then(|v| v.get("value"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let url = icon.get("link")
                        .and_then(|l| l.get("url"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("#");

                    format!(
                        r#"<a class="elementor-social-icon" href="{}"><i class="{}"></i></a>"#,
                        url, icon_class
                    )
                })
                .collect();

            format!(r#"<div class="elementor-social-icons">{}</div>"#, content)
        } else {
            String::new()
        }
    }

    fn render_alert_widget(&self, element: &ElementorElement) -> String {
        let title = element.settings.get("alert_title")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let description = element.settings.get("alert_description")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let alert_type = element.settings.get("alert_type")
            .and_then(|v| v.as_str())
            .unwrap_or("info");

        format!(
            r#"<div class="elementor-alert elementor-alert-{}">
                <strong class="elementor-alert-title">{}</strong>
                <span class="elementor-alert-description">{}</span>
            </div>"#,
            alert_type, title, description
        )
    }

    fn render_maps_widget(&self, element: &ElementorElement) -> String {
        let address = element.settings.get("address")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let zoom = element.settings.get("zoom")
            .and_then(|v| v.get("size"))
            .and_then(|v| v.as_u64())
            .unwrap_or(10);
        let height = element.settings.get("height")
            .and_then(|v| v.get("size"))
            .and_then(|v| v.as_u64())
            .unwrap_or(300);

        format!(
            r#"<div class="elementor-custom-embed" style="height: {}px">
                <iframe src="https://maps.google.com/maps?q={}&z={}&output=embed" frameborder="0" allowfullscreen></iframe>
            </div>"#,
            height, urlencoding::encode(address), zoom
        )
    }

    fn render_form_widget(&self, element: &ElementorElement) -> String {
        let form_name = element.settings.get("form_name")
            .and_then(|v| v.as_str())
            .unwrap_or("Contact Form");
        let fields = element.settings.get("form_fields")
            .and_then(|v| v.as_array());

        let fields_html: String = fields
            .map(|fields| {
                fields.iter()
                    .map(|field| {
                        let field_type = field.get("field_type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("text");
                        let label = field.get("field_label")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let placeholder = field.get("placeholder")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let required = field.get("required")
                            .and_then(|v| v.as_str())
                            .map(|v| v == "true")
                            .unwrap_or(false);
                        let width = field.get("width")
                            .and_then(|v| v.as_str())
                            .unwrap_or("100");

                        let req_attr = if required { "required" } else { "" };

                        let input = match field_type {
                            "textarea" => format!(
                                r#"<textarea name="{}" placeholder="{}" {}></textarea>"#,
                                label, placeholder, req_attr
                            ),
                            "email" => format!(
                                r#"<input type="email" name="{}" placeholder="{}" {}/>"#,
                                label, placeholder, req_attr
                            ),
                            _ => format!(
                                r#"<input type="text" name="{}" placeholder="{}" {}/>"#,
                                label, placeholder, req_attr
                            ),
                        };

                        format!(
                            r#"<div class="elementor-field-group" style="width: {}%">
                                <label>{}</label>
                                {}
                            </div>"#,
                            width, label, input
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();

        format!(
            r#"<form class="elementor-form" name="{}">
                {}
                <button type="submit" class="elementor-button">Submit</button>
            </form>"#,
            form_name, fields_html
        )
    }

    fn render_html_widget(&self, element: &ElementorElement) -> String {
        element.settings.get("html")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    }

    fn render_shortcode_widget(&self, element: &ElementorElement) -> String {
        let shortcode = element.settings.get("shortcode")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Return shortcode as-is for server-side processing
        format!("<!-- shortcode: {} -->", shortcode)
    }
}

impl Default for ElementorRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert Elementor document to Gutenberg blocks
pub struct ElementorToBlocksConverter;

impl ElementorToBlocksConverter {
    /// Convert Elementor elements to blocks
    pub fn convert(elements: &[ElementorElement]) -> Vec<Block> {
        elements.iter().flat_map(|el| Self::convert_element(el)).collect()
    }

    fn convert_element(element: &ElementorElement) -> Vec<Block> {
        match element.el_type.as_str() {
            "section" => {
                let inner_blocks: Vec<Block> = element.elements.iter()
                    .flat_map(|el| Self::convert_element(el))
                    .collect();

                vec![Block::group(inner_blocks, Some("constrained"))]
            }
            "column" => {
                element.elements.iter()
                    .flat_map(|el| Self::convert_element(el))
                    .collect()
            }
            "widget" => Self::convert_widget(element),
            _ => element.elements.iter()
                .flat_map(|el| Self::convert_element(el))
                .collect(),
        }
    }

    fn convert_widget(element: &ElementorElement) -> Vec<Block> {
        let widget_type = element.widget_type.as_deref().unwrap_or("");

        match widget_type {
            "heading" => {
                let text = element.settings.get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let level = element.settings.get("header_size")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.chars().last())
                    .and_then(|c| c.to_digit(10))
                    .unwrap_or(2) as u8;

                vec![Block::heading(text, level)]
            }
            "text-editor" => {
                let content = element.settings.get("editor")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                vec![Block::paragraph(content)]
            }
            "image" => {
                let url = element.settings.get("image")
                    .and_then(|i| i.get("url"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let alt = element.settings.get("image")
                    .and_then(|i| i.get("alt"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                vec![Block::image(url, alt, None)]
            }
            "button" => {
                let text = element.settings.get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let url = element.settings.get("link")
                    .and_then(|l| l.get("url"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("#");

                vec![Block::button(text, url, None)]
            }
            "divider" => vec![Block::separator(None)],
            "spacer" => {
                let height = element.settings.get("space")
                    .and_then(|v| v.get("size"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(50) as u32;

                vec![Block::spacer(height)]
            }
            "video" => {
                let video_type = element.settings.get("video_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("youtube");
                let url = match video_type {
                    "youtube" => element.settings.get("youtube_url"),
                    "vimeo" => element.settings.get("vimeo_url"),
                    _ => None,
                }
                .and_then(|v| v.as_str())
                .unwrap_or("");

                vec![Block::embed(url, video_type)]
            }
            _ => Vec::new(),
        }
    }
}

/// Convert Gutenberg blocks to Elementor elements
pub struct BlocksToElementorConverter;

impl BlocksToElementorConverter {
    /// Convert blocks to Elementor elements
    pub fn convert(blocks: &[Block]) -> Vec<ElementorElement> {
        // Wrap blocks in a section with column
        let widgets: Vec<ElementorElement> = blocks.iter()
            .flat_map(|block| Self::convert_block(block))
            .collect();

        vec![
            ElementorElement::section(vec![
                ElementorElement::column(widgets, 100)
            ])
        ]
    }

    fn convert_block(block: &Block) -> Vec<ElementorElement> {
        match block.block_type.as_str() {
            "core/paragraph" => {
                let content = block.attributes.get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                vec![ElementorElement::text_editor(content)]
            }
            "core/heading" => {
                let content = block.attributes.get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let level = block.attributes.get("level")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(2);
                vec![ElementorElement::heading(content, &format!("h{}", level), "left")]
            }
            "core/image" => {
                let url = block.attributes.get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let alt = block.attributes.get("alt")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                vec![ElementorElement::image(url, alt, None)]
            }
            "core/button" => {
                let text = block.attributes.get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let url = block.attributes.get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("#");
                vec![ElementorElement::button(text, url, "default")]
            }
            "core/separator" => vec![ElementorElement::divider("solid")],
            "core/spacer" => {
                let height = block.attributes.get("height")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(50) as u32;
                vec![ElementorElement::spacer(height)]
            }
            "core/columns" => {
                let columns: Vec<ElementorElement> = block.inner_blocks.iter()
                    .map(|col| {
                        let col_widgets: Vec<ElementorElement> = col.inner_blocks.iter()
                            .flat_map(|b| Self::convert_block(b))
                            .collect();
                        ElementorElement::column(col_widgets, 50)
                    })
                    .collect();
                vec![ElementorElement::section(columns)]
            }
            "core/group" => {
                let inner: Vec<ElementorElement> = block.inner_blocks.iter()
                    .flat_map(|b| Self::convert_block(b))
                    .collect();
                vec![ElementorElement::section(vec![
                    ElementorElement::column(inner, 100)
                ])]
            }
            _ if block.block_type.starts_with("core-embed/") => {
                let url = block.attributes.get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let provider = block.block_type.replace("core-embed/", "");
                vec![ElementorElement::video(url, &provider)]
            }
            _ => Vec::new(),
        }
    }
}

// Helper functions for video URL extraction
fn extract_youtube_id(url: &str) -> Option<String> {
    let re = regex::Regex::new(
        r"(?:youtube\.com/watch\?v=|youtu\.be/|youtube\.com/embed/)([a-zA-Z0-9_-]{11})"
    ).ok()?;
    re.captures(url).and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

fn extract_vimeo_id(url: &str) -> Option<String> {
    let re = regex::Regex::new(r"vimeo\.com/(\d+)").ok()?;
    re.captures(url).and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_heading_widget() {
        let heading = ElementorElement::heading("Test Heading", "h2", "center");
        assert_eq!(heading.widget_type, Some("heading".to_string()));
        assert_eq!(
            heading.settings.get("title").and_then(|v| v.as_str()),
            Some("Test Heading")
        );
    }

    #[test]
    fn test_render_section() {
        let section = ElementorElement::section(vec![
            ElementorElement::column(vec![
                ElementorElement::heading("Hello", "h1", "left")
            ], 100)
        ]);

        let renderer = ElementorRenderer::new();
        let html = renderer.render_element(&section);

        assert!(html.contains("elementor-section"));
        assert!(html.contains("Hello"));
    }

    #[test]
    fn test_elementor_to_blocks() {
        let elements = vec![
            ElementorElement::section(vec![
                ElementorElement::column(vec![
                    ElementorElement::heading("Test", "h1", "left"),
                    ElementorElement::text_editor("<p>Content</p>")
                ], 100)
            ])
        ];

        let blocks = ElementorToBlocksConverter::convert(&elements);
        assert!(!blocks.is_empty());
    }
}
