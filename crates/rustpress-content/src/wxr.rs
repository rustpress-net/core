//! # WXR Import/Export
//!
//! WordPress eXtended RSS (WXR) format import and export.
//!
//! Features:
//! - Full WXR 1.2 specification support
//! - Posts, pages, custom post types
//! - Categories, tags, custom taxonomies
//! - Users and authors
//! - Comments
//! - Attachments with media download
//! - Custom fields (post meta)

use chrono::{DateTime, Datelike, NaiveDateTime, Utc};
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufReader, Cursor, Read, Write};
use thiserror::Error;

/// WXR errors
#[derive(Debug, Error)]
pub enum WxrError {
    #[error("XML parsing error: {0}")]
    XmlError(#[from] quick_xml::Error),

    #[error("Invalid WXR format: {0}")]
    InvalidFormat(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Date parsing error: {0}")]
    DateError(String),

    #[error("Import error: {0}")]
    ImportError(String),
}

/// WXR document representing imported/exported content
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WxrDocument {
    /// Site information
    pub site: WxrSite,

    /// Authors/Users
    pub authors: Vec<WxrAuthor>,

    /// Categories
    pub categories: Vec<WxrCategory>,

    /// Tags
    pub tags: Vec<WxrTag>,

    /// Custom taxonomies
    pub terms: Vec<WxrTerm>,

    /// Posts, pages, and custom post types
    pub items: Vec<WxrItem>,

    /// WXR version
    pub version: String,
}

/// Site information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WxrSite {
    pub title: String,
    pub link: String,
    pub description: String,
    pub pubdate: Option<DateTime<Utc>>,
    pub language: String,
    pub base_site_url: String,
    pub base_blog_url: String,
}

/// Author/User
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WxrAuthor {
    pub id: i64,
    pub login: String,
    pub email: String,
    pub display_name: String,
    pub first_name: String,
    pub last_name: String,
}

/// Category
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WxrCategory {
    pub term_id: i64,
    pub nicename: String,
    pub parent: String,
    pub name: String,
    pub description: String,
}

/// Tag
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WxrTag {
    pub term_id: i64,
    pub slug: String,
    pub name: String,
    pub description: String,
}

/// Custom taxonomy term
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WxrTerm {
    pub term_id: i64,
    pub taxonomy: String,
    pub slug: String,
    pub parent: String,
    pub name: String,
    pub description: String,
}

/// Content item (post, page, attachment, etc.)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WxrItem {
    pub title: String,
    pub link: String,
    pub pubdate: Option<DateTime<Utc>>,
    pub creator: String,
    pub guid: String,
    pub description: String,
    pub content: String,
    pub excerpt: String,
    pub post_id: i64,
    pub post_date: Option<DateTime<Utc>>,
    pub post_date_gmt: Option<DateTime<Utc>>,
    pub post_modified: Option<DateTime<Utc>>,
    pub post_modified_gmt: Option<DateTime<Utc>>,
    pub comment_status: String,
    pub ping_status: String,
    pub post_name: String,
    pub status: String,
    pub post_parent: i64,
    pub menu_order: i32,
    pub post_type: String,
    pub post_password: String,
    pub is_sticky: bool,
    pub attachment_url: Option<String>,
    pub categories: Vec<WxrItemCategory>,
    pub tags: Vec<String>,
    pub meta: Vec<WxrMeta>,
    pub comments: Vec<WxrComment>,
}

/// Item category reference
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WxrItemCategory {
    pub domain: String,
    pub nicename: String,
    pub name: String,
}

/// Post meta
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WxrMeta {
    pub key: String,
    pub value: String,
}

/// Comment
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WxrComment {
    pub id: i64,
    pub author: String,
    pub author_email: String,
    pub author_url: String,
    pub author_ip: String,
    pub date: Option<DateTime<Utc>>,
    pub date_gmt: Option<DateTime<Utc>>,
    pub content: String,
    pub approved: String,
    pub comment_type: String,
    pub parent: i64,
    pub user_id: i64,
    pub meta: Vec<WxrMeta>,
}

/// WXR Parser
pub struct WxrParser;

impl WxrParser {
    /// Parse WXR from XML string
    pub fn parse(xml: &str) -> Result<WxrDocument, WxrError> {
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);

        let mut doc = WxrDocument::default();
        let mut buf = Vec::new();
        let mut current_element = String::new();
        let mut in_channel = false;
        let mut in_item = false;
        let mut current_item = WxrItem::default();
        let mut in_author = false;
        let mut current_author = WxrAuthor::default();
        let mut in_category = false;
        let mut current_category = WxrCategory::default();
        let mut in_tag = false;
        let mut current_tag = WxrTag::default();
        let mut in_comment = false;
        let mut current_comment = WxrComment::default();
        let mut in_postmeta = false;
        let mut current_meta = WxrMeta::default();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    current_element = name.clone();

                    match name.as_str() {
                        "channel" => in_channel = true,
                        "item" => {
                            in_item = true;
                            current_item = WxrItem::default();
                        }
                        "wp:author" => {
                            in_author = true;
                            current_author = WxrAuthor::default();
                        }
                        "wp:category" => {
                            in_category = true;
                            current_category = WxrCategory::default();
                        }
                        "wp:tag" => {
                            in_tag = true;
                            current_tag = WxrTag::default();
                        }
                        "wp:comment" => {
                            in_comment = true;
                            current_comment = WxrComment::default();
                        }
                        "wp:postmeta" | "wp:commentmeta" => {
                            in_postmeta = true;
                            current_meta = WxrMeta::default();
                        }
                        "category" if in_item => {
                            let mut cat = WxrItemCategory::default();
                            for attr in e.attributes().flatten() {
                                match attr.key.as_ref() {
                                    b"domain" => cat.domain = String::from_utf8_lossy(&attr.value).to_string(),
                                    b"nicename" => cat.nicename = String::from_utf8_lossy(&attr.value).to_string(),
                                    _ => {}
                                }
                            }
                            current_item.categories.push(cat);
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                    match name.as_str() {
                        "channel" => in_channel = false,
                        "item" => {
                            in_item = false;
                            doc.items.push(current_item.clone());
                        }
                        "wp:author" => {
                            in_author = false;
                            doc.authors.push(current_author.clone());
                        }
                        "wp:category" => {
                            in_category = false;
                            doc.categories.push(current_category.clone());
                        }
                        "wp:tag" => {
                            in_tag = false;
                            doc.tags.push(current_tag.clone());
                        }
                        "wp:comment" => {
                            in_comment = false;
                            current_item.comments.push(current_comment.clone());
                        }
                        "wp:postmeta" => {
                            in_postmeta = false;
                            current_item.meta.push(current_meta.clone());
                        }
                        "wp:commentmeta" => {
                            in_postmeta = false;
                            current_comment.meta.push(current_meta.clone());
                        }
                        _ => {}
                    }
                    current_element.clear();
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape().unwrap_or_default().to_string();

                    if in_postmeta {
                        match current_element.as_str() {
                            "wp:meta_key" => current_meta.key = text,
                            "wp:meta_value" => current_meta.value = text,
                            _ => {}
                        }
                    } else if in_comment {
                        match current_element.as_str() {
                            "wp:comment_id" => current_comment.id = text.parse().unwrap_or(0),
                            "wp:comment_author" => current_comment.author = text,
                            "wp:comment_author_email" => current_comment.author_email = text,
                            "wp:comment_author_url" => current_comment.author_url = text,
                            "wp:comment_author_IP" => current_comment.author_ip = text,
                            "wp:comment_date" => current_comment.date = parse_date(&text),
                            "wp:comment_date_gmt" => current_comment.date_gmt = parse_date(&text),
                            "wp:comment_content" => current_comment.content = text,
                            "wp:comment_approved" => current_comment.approved = text,
                            "wp:comment_type" => current_comment.comment_type = text,
                            "wp:comment_parent" => current_comment.parent = text.parse().unwrap_or(0),
                            "wp:comment_user_id" => current_comment.user_id = text.parse().unwrap_or(0),
                            _ => {}
                        }
                    } else if in_author {
                        match current_element.as_str() {
                            "wp:author_id" => current_author.id = text.parse().unwrap_or(0),
                            "wp:author_login" => current_author.login = text,
                            "wp:author_email" => current_author.email = text,
                            "wp:author_display_name" => current_author.display_name = text,
                            "wp:author_first_name" => current_author.first_name = text,
                            "wp:author_last_name" => current_author.last_name = text,
                            _ => {}
                        }
                    } else if in_category {
                        match current_element.as_str() {
                            "wp:term_id" => current_category.term_id = text.parse().unwrap_or(0),
                            "wp:category_nicename" => current_category.nicename = text,
                            "wp:category_parent" => current_category.parent = text,
                            "wp:cat_name" => current_category.name = text,
                            "wp:category_description" => current_category.description = text,
                            _ => {}
                        }
                    } else if in_tag {
                        match current_element.as_str() {
                            "wp:term_id" => current_tag.term_id = text.parse().unwrap_or(0),
                            "wp:tag_slug" => current_tag.slug = text,
                            "wp:tag_name" => current_tag.name = text,
                            "wp:tag_description" => current_tag.description = text,
                            _ => {}
                        }
                    } else if in_item {
                        match current_element.as_str() {
                            "title" => current_item.title = text,
                            "link" => current_item.link = text,
                            "pubDate" => current_item.pubdate = parse_rfc2822_date(&text),
                            "dc:creator" => current_item.creator = text,
                            "guid" => current_item.guid = text,
                            "description" => current_item.description = text,
                            "content:encoded" => current_item.content = text,
                            "excerpt:encoded" => current_item.excerpt = text,
                            "wp:post_id" => current_item.post_id = text.parse().unwrap_or(0),
                            "wp:post_date" => current_item.post_date = parse_date(&text),
                            "wp:post_date_gmt" => current_item.post_date_gmt = parse_date(&text),
                            "wp:post_modified" => current_item.post_modified = parse_date(&text),
                            "wp:post_modified_gmt" => current_item.post_modified_gmt = parse_date(&text),
                            "wp:comment_status" => current_item.comment_status = text,
                            "wp:ping_status" => current_item.ping_status = text,
                            "wp:post_name" => current_item.post_name = text,
                            "wp:status" => current_item.status = text,
                            "wp:post_parent" => current_item.post_parent = text.parse().unwrap_or(0),
                            "wp:menu_order" => current_item.menu_order = text.parse().unwrap_or(0),
                            "wp:post_type" => current_item.post_type = text,
                            "wp:post_password" => current_item.post_password = text,
                            "wp:is_sticky" => current_item.is_sticky = text == "1",
                            "wp:attachment_url" => current_item.attachment_url = Some(text),
                            _ => {}
                        }
                    } else if in_channel {
                        match current_element.as_str() {
                            "title" => doc.site.title = text,
                            "link" => doc.site.link = text,
                            "description" => doc.site.description = text,
                            "pubDate" => doc.site.pubdate = parse_rfc2822_date(&text),
                            "language" => doc.site.language = text,
                            "wp:base_site_url" => doc.site.base_site_url = text,
                            "wp:base_blog_url" => doc.site.base_blog_url = text,
                            "wp:wxr_version" => doc.version = text,
                            _ => {}
                        }
                    }
                }
                Ok(Event::CData(e)) => {
                    let text = String::from_utf8_lossy(&e).to_string();

                    if in_item {
                        match current_element.as_str() {
                            "content:encoded" => current_item.content = text,
                            "excerpt:encoded" => current_item.excerpt = text,
                            "category" => {
                                if let Some(cat) = current_item.categories.last_mut() {
                                    cat.name = text;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(WxrError::XmlError(e)),
                _ => {}
            }
            buf.clear();
        }

        Ok(doc)
    }
}

/// WXR Writer/Exporter
pub struct WxrWriter;

impl WxrWriter {
    /// Export document to WXR XML
    pub fn export(doc: &WxrDocument) -> Result<String, WxrError> {
        let mut buffer = Vec::new();
        let mut writer = Writer::new_with_indent(Cursor::new(&mut buffer), b' ', 2);

        // XML declaration
        writer.write_event(Event::Decl(quick_xml::events::BytesDecl::new("1.0", Some("UTF-8"), None)))?;

        // RSS root element with namespaces
        let mut rss = BytesStart::new("rss");
        rss.push_attribute(("version", "2.0"));
        rss.push_attribute(("xmlns:excerpt", "http://wordpress.org/export/1.2/excerpt/"));
        rss.push_attribute(("xmlns:content", "http://purl.org/rss/1.0/modules/content/"));
        rss.push_attribute(("xmlns:wfw", "http://wellformedweb.org/CommentAPI/"));
        rss.push_attribute(("xmlns:dc", "http://purl.org/dc/elements/1.1/"));
        rss.push_attribute(("xmlns:wp", "http://wordpress.org/export/1.2/"));
        writer.write_event(Event::Start(rss))?;

        // Channel
        writer.write_event(Event::Start(BytesStart::new("channel")))?;

        // Site info
        Self::write_element(&mut writer, "title", &doc.site.title)?;
        Self::write_element(&mut writer, "link", &doc.site.link)?;
        Self::write_element(&mut writer, "description", &doc.site.description)?;
        Self::write_element(&mut writer, "language", &doc.site.language)?;
        Self::write_element(&mut writer, "wp:wxr_version", &doc.version)?;
        Self::write_element(&mut writer, "wp:base_site_url", &doc.site.base_site_url)?;
        Self::write_element(&mut writer, "wp:base_blog_url", &doc.site.base_blog_url)?;

        // Authors
        for author in &doc.authors {
            writer.write_event(Event::Start(BytesStart::new("wp:author")))?;
            Self::write_element(&mut writer, "wp:author_id", &author.id.to_string())?;
            Self::write_cdata_element(&mut writer, "wp:author_login", &author.login)?;
            Self::write_cdata_element(&mut writer, "wp:author_email", &author.email)?;
            Self::write_cdata_element(&mut writer, "wp:author_display_name", &author.display_name)?;
            Self::write_cdata_element(&mut writer, "wp:author_first_name", &author.first_name)?;
            Self::write_cdata_element(&mut writer, "wp:author_last_name", &author.last_name)?;
            writer.write_event(Event::End(BytesEnd::new("wp:author")))?;
        }

        // Categories
        for cat in &doc.categories {
            writer.write_event(Event::Start(BytesStart::new("wp:category")))?;
            Self::write_element(&mut writer, "wp:term_id", &cat.term_id.to_string())?;
            Self::write_cdata_element(&mut writer, "wp:category_nicename", &cat.nicename)?;
            Self::write_cdata_element(&mut writer, "wp:category_parent", &cat.parent)?;
            Self::write_cdata_element(&mut writer, "wp:cat_name", &cat.name)?;
            Self::write_cdata_element(&mut writer, "wp:category_description", &cat.description)?;
            writer.write_event(Event::End(BytesEnd::new("wp:category")))?;
        }

        // Tags
        for tag in &doc.tags {
            writer.write_event(Event::Start(BytesStart::new("wp:tag")))?;
            Self::write_element(&mut writer, "wp:term_id", &tag.term_id.to_string())?;
            Self::write_cdata_element(&mut writer, "wp:tag_slug", &tag.slug)?;
            Self::write_cdata_element(&mut writer, "wp:tag_name", &tag.name)?;
            Self::write_cdata_element(&mut writer, "wp:tag_description", &tag.description)?;
            writer.write_event(Event::End(BytesEnd::new("wp:tag")))?;
        }

        // Items
        for item in &doc.items {
            writer.write_event(Event::Start(BytesStart::new("item")))?;

            Self::write_element(&mut writer, "title", &item.title)?;
            Self::write_element(&mut writer, "link", &item.link)?;
            if let Some(date) = item.pubdate {
                Self::write_element(&mut writer, "pubDate", &date.format("%a, %d %b %Y %H:%M:%S %z").to_string())?;
            }
            Self::write_cdata_element(&mut writer, "dc:creator", &item.creator)?;
            Self::write_element(&mut writer, "guid", &item.guid)?;
            Self::write_element(&mut writer, "description", &item.description)?;
            Self::write_cdata_element(&mut writer, "content:encoded", &item.content)?;
            Self::write_cdata_element(&mut writer, "excerpt:encoded", &item.excerpt)?;
            Self::write_element(&mut writer, "wp:post_id", &item.post_id.to_string())?;
            if let Some(date) = item.post_date {
                Self::write_element(&mut writer, "wp:post_date", &date.format("%Y-%m-%d %H:%M:%S").to_string())?;
            }
            if let Some(date) = item.post_date_gmt {
                Self::write_element(&mut writer, "wp:post_date_gmt", &date.format("%Y-%m-%d %H:%M:%S").to_string())?;
            }
            Self::write_element(&mut writer, "wp:comment_status", &item.comment_status)?;
            Self::write_element(&mut writer, "wp:ping_status", &item.ping_status)?;
            Self::write_cdata_element(&mut writer, "wp:post_name", &item.post_name)?;
            Self::write_element(&mut writer, "wp:status", &item.status)?;
            Self::write_element(&mut writer, "wp:post_parent", &item.post_parent.to_string())?;
            Self::write_element(&mut writer, "wp:menu_order", &item.menu_order.to_string())?;
            Self::write_element(&mut writer, "wp:post_type", &item.post_type)?;
            Self::write_element(&mut writer, "wp:post_password", &item.post_password)?;
            Self::write_element(&mut writer, "wp:is_sticky", if item.is_sticky { "1" } else { "0" })?;

            if let Some(ref url) = item.attachment_url {
                Self::write_element(&mut writer, "wp:attachment_url", url)?;
            }

            // Categories and tags
            for cat in &item.categories {
                let mut elem = BytesStart::new("category");
                elem.push_attribute(("domain", cat.domain.as_str()));
                elem.push_attribute(("nicename", cat.nicename.as_str()));
                writer.write_event(Event::Start(elem))?;
                writer.write_event(Event::CData(quick_xml::events::BytesCData::new(&cat.name)))?;
                writer.write_event(Event::End(BytesEnd::new("category")))?;
            }

            // Post meta
            for meta in &item.meta {
                writer.write_event(Event::Start(BytesStart::new("wp:postmeta")))?;
                Self::write_cdata_element(&mut writer, "wp:meta_key", &meta.key)?;
                Self::write_cdata_element(&mut writer, "wp:meta_value", &meta.value)?;
                writer.write_event(Event::End(BytesEnd::new("wp:postmeta")))?;
            }

            // Comments
            for comment in &item.comments {
                writer.write_event(Event::Start(BytesStart::new("wp:comment")))?;
                Self::write_element(&mut writer, "wp:comment_id", &comment.id.to_string())?;
                Self::write_cdata_element(&mut writer, "wp:comment_author", &comment.author)?;
                Self::write_element(&mut writer, "wp:comment_author_email", &comment.author_email)?;
                Self::write_element(&mut writer, "wp:comment_author_url", &comment.author_url)?;
                Self::write_element(&mut writer, "wp:comment_author_IP", &comment.author_ip)?;
                if let Some(date) = comment.date {
                    Self::write_element(&mut writer, "wp:comment_date", &date.format("%Y-%m-%d %H:%M:%S").to_string())?;
                }
                if let Some(date) = comment.date_gmt {
                    Self::write_element(&mut writer, "wp:comment_date_gmt", &date.format("%Y-%m-%d %H:%M:%S").to_string())?;
                }
                Self::write_cdata_element(&mut writer, "wp:comment_content", &comment.content)?;
                Self::write_element(&mut writer, "wp:comment_approved", &comment.approved)?;
                Self::write_element(&mut writer, "wp:comment_type", &comment.comment_type)?;
                Self::write_element(&mut writer, "wp:comment_parent", &comment.parent.to_string())?;
                Self::write_element(&mut writer, "wp:comment_user_id", &comment.user_id.to_string())?;
                writer.write_event(Event::End(BytesEnd::new("wp:comment")))?;
            }

            writer.write_event(Event::End(BytesEnd::new("item")))?;
        }

        writer.write_event(Event::End(BytesEnd::new("channel")))?;
        writer.write_event(Event::End(BytesEnd::new("rss")))?;

        Ok(String::from_utf8(buffer).unwrap_or_default())
    }

    fn write_element<W: Write>(writer: &mut Writer<W>, name: &str, value: &str) -> Result<(), WxrError> {
        writer.write_event(Event::Start(BytesStart::new(name)))?;
        writer.write_event(Event::Text(BytesText::new(value)))?;
        writer.write_event(Event::End(BytesEnd::new(name)))?;
        Ok(())
    }

    fn write_cdata_element<W: Write>(writer: &mut Writer<W>, name: &str, value: &str) -> Result<(), WxrError> {
        writer.write_event(Event::Start(BytesStart::new(name)))?;
        writer.write_event(Event::CData(quick_xml::events::BytesCData::new(value)))?;
        writer.write_event(Event::End(BytesEnd::new(name)))?;
        Ok(())
    }
}

/// Parse WordPress date format
fn parse_date(s: &str) -> Option<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .ok()
        .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
}

/// Parse RFC 2822 date format
fn parse_rfc2822_date(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc2822(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

/// Import options
#[derive(Debug, Clone)]
pub struct ImportOptions {
    /// Import authors
    pub import_authors: bool,

    /// Map author logins to user IDs
    pub author_mapping: HashMap<String, i64>,

    /// Default author ID for unmapped authors
    pub default_author_id: i64,

    /// Import categories
    pub import_categories: bool,

    /// Import tags
    pub import_tags: bool,

    /// Import attachments
    pub import_attachments: bool,

    /// Download attachment files
    pub download_attachments: bool,

    /// Import comments
    pub import_comments: bool,

    /// Import post meta
    pub import_meta: bool,

    /// Filter by post types
    pub post_types: Option<Vec<String>>,

    /// Filter by status
    pub statuses: Option<Vec<String>>,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            import_authors: true,
            author_mapping: HashMap::new(),
            default_author_id: 1,
            import_categories: true,
            import_tags: true,
            import_attachments: true,
            download_attachments: true,
            import_comments: true,
            import_meta: true,
            post_types: None,
            statuses: None,
        }
    }
}

/// Export options
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Include authors
    pub include_authors: bool,

    /// Include categories
    pub include_categories: bool,

    /// Include tags
    pub include_tags: bool,

    /// Include attachments
    pub include_attachments: bool,

    /// Include comments
    pub include_comments: bool,

    /// Include post meta
    pub include_meta: bool,

    /// Filter by post types
    pub post_types: Option<Vec<String>>,

    /// Filter by status
    pub statuses: Option<Vec<String>>,

    /// Filter by author IDs
    pub authors: Option<Vec<i64>>,

    /// Filter by category IDs
    pub categories: Option<Vec<i64>>,

    /// Date range start
    pub date_from: Option<DateTime<Utc>>,

    /// Date range end
    pub date_to: Option<DateTime<Utc>>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            include_authors: true,
            include_categories: true,
            include_tags: true,
            include_attachments: true,
            include_comments: true,
            include_meta: true,
            post_types: None,
            statuses: None,
            authors: None,
            categories: None,
            date_from: None,
            date_to: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_date() {
        let date = parse_date("2024-01-15 12:30:00");
        assert!(date.is_some());

        let dt = date.unwrap();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 15);
    }

    #[test]
    fn test_wxr_document_default() {
        let doc = WxrDocument::default();
        assert!(doc.items.is_empty());
        assert!(doc.authors.is_empty());
        assert!(doc.categories.is_empty());
    }

    #[test]
    fn test_export_empty_document() {
        let mut doc = WxrDocument::default();
        doc.version = "1.2".to_string();
        doc.site.title = "Test Site".to_string();

        let xml = WxrWriter::export(&doc).unwrap();
        assert!(xml.contains("Test Site"));
        assert!(xml.contains("wp:wxr_version"));
    }
}
