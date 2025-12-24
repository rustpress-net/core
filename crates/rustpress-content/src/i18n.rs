//! # Multi-Language Content
//!
//! Translation linking and multi-language content management.
//!
//! Features:
//! - Translation linking between posts
//! - Language detection
//! - hreflang tag generation
//! - Content synchronization
//! - Language switching

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported language with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Language {
    /// Language code (e.g., "en", "en-US")
    pub code: String,

    /// Locale code (e.g., "en_US")
    pub locale: String,

    /// Native name (e.g., "English")
    pub native_name: String,

    /// English name
    pub english_name: String,

    /// RTL language
    pub rtl: bool,

    /// Flag icon code
    pub flag: String,

    /// Whether this is the default language
    pub is_default: bool,

    /// Language active status
    pub active: bool,

    /// Display order
    pub order: i32,
}

impl Language {
    pub fn new(code: &str, native_name: &str, english_name: &str) -> Self {
        Self {
            code: code.to_string(),
            locale: code.replace('-', "_"),
            native_name: native_name.to_string(),
            english_name: english_name.to_string(),
            rtl: false,
            flag: code.split('-').next().unwrap_or(code).to_string(),
            is_default: false,
            active: true,
            order: 0,
        }
    }

    pub fn rtl(mut self) -> Self {
        self.rtl = true;
        self
    }

    pub fn as_default(mut self) -> Self {
        self.is_default = true;
        self
    }

    /// Common languages
    pub fn english() -> Self {
        Self::new("en-US", "English", "English").as_default()
    }

    pub fn spanish() -> Self {
        Self::new("es", "Español", "Spanish")
    }

    pub fn french() -> Self {
        Self::new("fr", "Français", "French")
    }

    pub fn german() -> Self {
        Self::new("de", "Deutsch", "German")
    }

    pub fn arabic() -> Self {
        Self::new("ar", "العربية", "Arabic").rtl()
    }

    pub fn chinese_simplified() -> Self {
        Self::new("zh-CN", "简体中文", "Chinese (Simplified)")
    }

    pub fn japanese() -> Self {
        Self::new("ja", "日本語", "Japanese")
    }
}

/// Translation link between content in different languages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationLink {
    /// Unique translation group ID
    pub translation_id: String,

    /// Post ID
    pub post_id: i64,

    /// Language code
    pub language: String,

    /// Post type
    pub post_type: String,

    /// Source/original language
    pub source_language: Option<String>,

    /// Translation status
    pub status: TranslationStatus,

    /// Last sync timestamp
    pub last_synced: Option<DateTime<Utc>>,
}

/// Translation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TranslationStatus {
    /// Not translated
    NotTranslated,

    /// Translation in progress
    InProgress,

    /// Translated and up to date
    Translated,

    /// Source content changed, needs update
    NeedsUpdate,

    /// Machine translated
    MachineTranslated,
}

impl TranslationStatus {
    pub fn label(&self) -> &str {
        match self {
            Self::NotTranslated => "Not Translated",
            Self::InProgress => "In Progress",
            Self::Translated => "Translated",
            Self::NeedsUpdate => "Needs Update",
            Self::MachineTranslated => "Machine Translated",
        }
    }

    pub fn color(&self) -> &str {
        match self {
            Self::NotTranslated => "#999",
            Self::InProgress => "#f0ad4e",
            Self::Translated => "#5cb85c",
            Self::NeedsUpdate => "#d9534f",
            Self::MachineTranslated => "#5bc0de",
        }
    }
}

/// Translation manager
pub struct TranslationManager {
    /// Available languages
    languages: HashMap<String, Language>,

    /// Translation links (translation_id -> links)
    translations: HashMap<String, Vec<TranslationLink>>,

    /// Default language code
    default_language: String,
}

impl Default for TranslationManager {
    fn default() -> Self {
        let mut manager = Self {
            languages: HashMap::new(),
            translations: HashMap::new(),
            default_language: "en-US".to_string(),
        };

        // Register default language
        manager.register_language(Language::english());

        manager
    }
}

impl TranslationManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a language
    pub fn register_language(&mut self, language: Language) {
        if language.is_default {
            self.default_language = language.code.clone();
        }
        self.languages.insert(language.code.clone(), language);
    }

    /// Get language by code
    pub fn get_language(&self, code: &str) -> Option<&Language> {
        self.languages.get(code)
    }

    /// Get all active languages
    pub fn get_active_languages(&self) -> Vec<&Language> {
        let mut langs: Vec<&Language> = self.languages.values()
            .filter(|l| l.active)
            .collect();
        langs.sort_by_key(|l| l.order);
        langs
    }

    /// Get default language
    pub fn get_default_language(&self) -> Option<&Language> {
        self.languages.get(&self.default_language)
    }

    /// Link posts as translations of each other
    pub fn link_translations(&mut self, links: Vec<(i64, String, String)>) -> String {
        let translation_id = uuid::Uuid::new_v4().to_string();

        let translation_links: Vec<TranslationLink> = links
            .into_iter()
            .map(|(post_id, language, post_type)| TranslationLink {
                translation_id: translation_id.clone(),
                post_id,
                language,
                post_type,
                source_language: Some(self.default_language.clone()),
                status: TranslationStatus::Translated,
                last_synced: Some(Utc::now()),
            })
            .collect();

        self.translations.insert(translation_id.clone(), translation_links);
        translation_id
    }

    /// Get all translations for a post
    pub fn get_translations(&self, post_id: i64) -> Vec<&TranslationLink> {
        self.translations.values()
            .flat_map(|links| links.iter())
            .filter(|link| {
                self.translations.values()
                    .flat_map(|l| l.iter())
                    .any(|l| l.post_id == post_id && l.translation_id == link.translation_id)
            })
            .collect()
    }

    /// Get translation for specific language
    pub fn get_translation_for_language(&self, post_id: i64, language: &str) -> Option<&TranslationLink> {
        // Find the translation group containing this post
        for links in self.translations.values() {
            if links.iter().any(|l| l.post_id == post_id) {
                return links.iter().find(|l| l.language == language);
            }
        }
        None
    }

    /// Get post's language
    pub fn get_post_language(&self, post_id: i64) -> Option<&str> {
        for links in self.translations.values() {
            if let Some(link) = links.iter().find(|l| l.post_id == post_id) {
                return Some(&link.language);
            }
        }
        None
    }

    /// Update translation status
    pub fn update_status(&mut self, post_id: i64, status: TranslationStatus) {
        for links in self.translations.values_mut() {
            if let Some(link) = links.iter_mut().find(|l| l.post_id == post_id) {
                link.status = status;
                if status == TranslationStatus::Translated {
                    link.last_synced = Some(Utc::now());
                }
            }
        }
    }

    /// Mark translations as needing update when source changes
    pub fn mark_needs_update(&mut self, source_post_id: i64) {
        for links in self.translations.values_mut() {
            if links.iter().any(|l| l.post_id == source_post_id) {
                for link in links.iter_mut() {
                    if link.post_id != source_post_id {
                        link.status = TranslationStatus::NeedsUpdate;
                    }
                }
            }
        }
    }

    /// Generate hreflang tags for a post
    pub fn generate_hreflang_tags(&self, post_id: i64, get_url: impl Fn(i64) -> String) -> String {
        let translations = self.get_translations(post_id);
        if translations.is_empty() {
            return String::new();
        }

        let mut tags = Vec::new();

        for link in translations {
            let url = get_url(link.post_id);
            let lang = &link.language;

            tags.push(format!(
                r#"<link rel="alternate" hreflang="{}" href="{}">"#,
                lang, url
            ));

            // Add x-default for default language
            if self.languages.get(lang).map(|l| l.is_default).unwrap_or(false) {
                tags.push(format!(
                    r#"<link rel="alternate" hreflang="x-default" href="{}">"#,
                    url
                ));
            }
        }

        tags.join("\n")
    }

    /// Generate language switcher data
    pub fn get_language_switcher_data(&self, current_post_id: i64, get_url: impl Fn(i64) -> String) -> Vec<LanguageSwitcherItem> {
        let translations = self.get_translations(current_post_id);
        let current_lang = self.get_post_language(current_post_id);

        self.get_active_languages()
            .into_iter()
            .map(|lang| {
                let translation = translations.iter()
                    .find(|t| t.language == lang.code);

                LanguageSwitcherItem {
                    code: lang.code.clone(),
                    name: lang.native_name.clone(),
                    flag: lang.flag.clone(),
                    url: translation.map(|t| get_url(t.post_id)),
                    is_current: current_lang == Some(&lang.code),
                    is_available: translation.is_some(),
                }
            })
            .collect()
    }
}

/// Language switcher item for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageSwitcherItem {
    pub code: String,
    pub name: String,
    pub flag: String,
    pub url: Option<String>,
    pub is_current: bool,
    pub is_available: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_creation() {
        let lang = Language::english();
        assert_eq!(lang.code, "en-US");
        assert!(lang.is_default);
        assert!(!lang.rtl);
    }

    #[test]
    fn test_rtl_language() {
        let lang = Language::arabic();
        assert!(lang.rtl);
    }

    #[test]
    fn test_translation_manager() {
        let mut manager = TranslationManager::new();
        manager.register_language(Language::spanish());
        manager.register_language(Language::french());

        let langs = manager.get_active_languages();
        assert!(langs.len() >= 3); // en, es, fr
    }

    #[test]
    fn test_link_translations() {
        let mut manager = TranslationManager::new();
        manager.register_language(Language::spanish());

        let translation_id = manager.link_translations(vec![
            (1, "en-US".to_string(), "post".to_string()),
            (2, "es".to_string(), "post".to_string()),
        ]);

        assert!(!translation_id.is_empty());

        let translations = manager.get_translations(1);
        assert_eq!(translations.len(), 2);
    }
}
