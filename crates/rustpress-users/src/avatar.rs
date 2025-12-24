//! # User Avatar System
//!
//! Avatar management with Gravatar fallback support.
//!
//! Features:
//! - Custom avatar uploads
//! - Gravatar integration
//! - Multiple default avatar styles
//! - Avatar size variations
//! - Avatar caching

use md5::{Md5, Digest as Md5Digest};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Avatar data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Avatar {
    /// User ID
    pub user_id: i64,

    /// Avatar type
    pub avatar_type: AvatarType,

    /// Custom avatar URL (if uploaded)
    pub custom_url: Option<String>,

    /// Custom avatar attachment ID
    pub attachment_id: Option<i64>,

    /// Gravatar email hash
    pub email_hash: String,

    /// Default avatar style for Gravatar
    pub default_style: GravatarDefault,

    /// Gravatar rating
    pub rating: GravatarRating,

    /// Avatar variations (size -> url)
    pub variations: HashMap<u32, String>,
}

/// Avatar type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AvatarType {
    /// Use Gravatar
    Gravatar,
    /// Custom uploaded avatar
    Custom,
    /// Generated avatar (initials, patterns, etc.)
    Generated,
    /// Default placeholder
    Default,
}

/// Gravatar default styles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GravatarDefault {
    /// Mystery person silhouette
    Mp,
    /// Geometric pattern based on email hash
    Identicon,
    /// 8-bit arcade style face
    Retro,
    /// Monster face
    MonsterId,
    /// Wavatar generated face
    Wavatar,
    /// Robot face
    RoboHash,
    /// Blank/transparent
    Blank,
    /// 404 error if no gravatar
    NotFound,
}

impl GravatarDefault {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Mp => "mp",
            Self::Identicon => "identicon",
            Self::Retro => "retro",
            Self::MonsterId => "monsterid",
            Self::Wavatar => "wavatar",
            Self::RoboHash => "robohash",
            Self::Blank => "blank",
            Self::NotFound => "404",
        }
    }
}

/// Gravatar rating levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GravatarRating {
    /// Suitable for all audiences
    G,
    /// May contain mild content
    Pg,
    /// May contain adult content
    R,
    /// May contain explicit content
    X,
}

impl GravatarRating {
    pub fn as_str(&self) -> &str {
        match self {
            Self::G => "g",
            Self::Pg => "pg",
            Self::R => "r",
            Self::X => "x",
        }
    }
}

impl Avatar {
    pub fn new(user_id: i64, email: &str) -> Self {
        Self {
            user_id,
            avatar_type: AvatarType::Gravatar,
            custom_url: None,
            attachment_id: None,
            email_hash: Self::hash_email(email),
            default_style: GravatarDefault::Mp,
            rating: GravatarRating::G,
            variations: HashMap::new(),
        }
    }

    /// Hash email for Gravatar
    pub fn hash_email(email: &str) -> String {
        let email_lower = email.trim().to_lowercase();
        let mut hasher = Md5::new();
        hasher.update(email_lower.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Update email hash
    pub fn update_email(&mut self, email: &str) {
        self.email_hash = Self::hash_email(email);
        self.variations.clear();
    }

    /// Set custom avatar
    pub fn set_custom(&mut self, url: &str, attachment_id: Option<i64>) {
        self.avatar_type = AvatarType::Custom;
        self.custom_url = Some(url.to_string());
        self.attachment_id = attachment_id;
        self.variations.clear();
    }

    /// Reset to Gravatar
    pub fn use_gravatar(&mut self) {
        self.avatar_type = AvatarType::Gravatar;
        self.custom_url = None;
        self.attachment_id = None;
        self.variations.clear();
    }

    /// Get avatar URL for specific size
    pub fn get_url(&self, size: u32) -> String {
        // Check cached variations
        if let Some(url) = self.variations.get(&size) {
            return url.clone();
        }

        match self.avatar_type {
            AvatarType::Custom => {
                self.custom_url.clone().unwrap_or_else(|| self.gravatar_url(size))
            }
            AvatarType::Gravatar => self.gravatar_url(size),
            AvatarType::Generated => self.generated_url(size),
            AvatarType::Default => self.default_url(size),
        }
    }

    /// Build Gravatar URL
    pub fn gravatar_url(&self, size: u32) -> String {
        format!(
            "https://www.gravatar.com/avatar/{}?s={}&d={}&r={}",
            self.email_hash,
            size,
            self.default_style.as_str(),
            self.rating.as_str()
        )
    }

    /// Build generated avatar URL (using DiceBear or similar)
    pub fn generated_url(&self, size: u32) -> String {
        format!(
            "https://api.dicebear.com/7.x/initials/svg?seed={}&size={}",
            urlencoding::encode(&self.email_hash),
            size
        )
    }

    /// Default placeholder URL
    pub fn default_url(&self, size: u32) -> String {
        format!(
            "https://www.gravatar.com/avatar/00000000000000000000000000000000?s={}&d=mp",
            size
        )
    }

    /// Get srcset for responsive images
    pub fn get_srcset(&self, sizes: &[u32]) -> String {
        sizes.iter()
            .map(|&size| format!("{} {}w", self.get_url(size), size))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Generate HTML img tag
    pub fn get_html(&self, size: u32, alt: &str, class: Option<&str>) -> String {
        let sizes = [size, size * 2];
        let srcset = self.get_srcset(&sizes);
        let class_attr = class.map(|c| format!(r#" class="{}""#, c)).unwrap_or_default();

        format!(
            r#"<img src="{}" srcset="{}" width="{}" height="{}" alt="{}"{} loading="lazy">"#,
            self.get_url(size),
            srcset,
            size,
            size,
            alt,
            class_attr
        )
    }
}

/// Avatar settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarSettings {
    /// Enable avatars
    pub enabled: bool,

    /// Default avatar type
    pub default_type: AvatarType,

    /// Default Gravatar style
    pub default_gravatar_style: GravatarDefault,

    /// Gravatar rating limit
    pub max_rating: GravatarRating,

    /// Allow custom uploads
    pub allow_custom_upload: bool,

    /// Max upload size in bytes
    pub max_upload_size: u64,

    /// Allowed mime types
    pub allowed_types: Vec<String>,

    /// Standard sizes to generate
    pub sizes: Vec<u32>,

    /// Crop mode
    pub crop_mode: CropMode,
}

/// Image crop mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CropMode {
    /// Crop to exact square
    Square,
    /// Fit within bounds
    Fit,
    /// Fill bounds (may crop)
    Fill,
}

impl Default for AvatarSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            default_type: AvatarType::Gravatar,
            default_gravatar_style: GravatarDefault::Mp,
            max_rating: GravatarRating::G,
            allow_custom_upload: true,
            max_upload_size: 2 * 1024 * 1024, // 2MB
            allowed_types: vec![
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "image/gif".to_string(),
                "image/webp".to_string(),
            ],
            sizes: vec![24, 32, 48, 64, 96, 128, 256],
            crop_mode: CropMode::Square,
        }
    }
}

/// Avatar manager
pub struct AvatarManager {
    avatars: HashMap<i64, Avatar>,
    settings: AvatarSettings,
    upload_dir: String,
}

impl AvatarManager {
    pub fn new(upload_dir: &str) -> Self {
        Self {
            avatars: HashMap::new(),
            settings: AvatarSettings::default(),
            upload_dir: upload_dir.to_string(),
        }
    }

    pub fn with_settings(mut self, settings: AvatarSettings) -> Self {
        self.settings = settings;
        self
    }

    /// Get avatar for user
    pub fn get_avatar(&self, user_id: i64) -> Option<&Avatar> {
        self.avatars.get(&user_id)
    }

    /// Get or create avatar for user
    pub fn get_or_create_avatar(&mut self, user_id: i64, email: &str) -> &mut Avatar {
        self.avatars.entry(user_id)
            .or_insert_with(|| Avatar::new(user_id, email))
    }

    /// Update avatar
    pub fn set_avatar(&mut self, avatar: Avatar) {
        self.avatars.insert(avatar.user_id, avatar);
    }

    /// Get avatar URL
    pub fn get_avatar_url(&self, user_id: i64, size: u32) -> String {
        self.avatars.get(&user_id)
            .map(|a| a.get_url(size))
            .unwrap_or_else(|| Avatar::new(0, "").default_url(size))
    }

    /// Delete custom avatar
    pub fn delete_custom(&mut self, user_id: i64) {
        if let Some(avatar) = self.avatars.get_mut(&user_id) {
            avatar.use_gravatar();
        }
    }

    /// Get settings
    pub fn get_settings(&self) -> &AvatarSettings {
        &self.settings
    }

    /// Update settings
    pub fn update_settings(&mut self, settings: AvatarSettings) {
        self.settings = settings;
    }

    /// Validate upload
    pub fn validate_upload(&self, size: u64, mime_type: &str) -> Result<(), String> {
        if !self.settings.allow_custom_upload {
            return Err("Custom avatar uploads are disabled".to_string());
        }

        if size > self.settings.max_upload_size {
            return Err(format!(
                "File too large. Maximum size is {} bytes",
                self.settings.max_upload_size
            ));
        }

        if !self.settings.allowed_types.contains(&mime_type.to_string()) {
            return Err(format!(
                "Invalid file type. Allowed types: {:?}",
                self.settings.allowed_types
            ));
        }

        Ok(())
    }

    /// Get upload path for user
    pub fn get_upload_path(&self, user_id: i64, extension: &str) -> String {
        format!("{}/avatars/{}.{}", self.upload_dir, user_id, extension)
    }
}

/// Avatar helper functions
pub mod helpers {
    use super::*;

    /// Generate initials from name
    pub fn get_initials(name: &str) -> String {
        name.split_whitespace()
            .filter_map(|word| word.chars().next())
            .take(2)
            .map(|c| c.to_uppercase().to_string())
            .collect()
    }

    /// Generate color from string (for backgrounds)
    pub fn string_to_color(s: &str) -> String {
        let hash: u32 = s.chars()
            .enumerate()
            .fold(0u32, |acc, (i, c)| {
                acc.wrapping_add((c as u32).wrapping_mul(i as u32 + 1))
            });

        // Generate HSL color with good saturation and lightness
        let hue = hash % 360;
        format!("hsl({}, 65%, 55%)", hue)
    }

    /// Generate SVG avatar with initials
    pub fn generate_initials_svg(name: &str, size: u32) -> String {
        let initials = get_initials(name);
        let bg_color = string_to_color(name);
        let font_size = size as f32 * 0.4;

        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 {size} {size}">
<rect width="100%" height="100%" fill="{bg_color}"/>
<text x="50%" y="50%" dominant-baseline="central" text-anchor="middle" fill="white" font-family="system-ui, sans-serif" font-size="{font_size}" font-weight="600">{initials}</text>
</svg>"#,
            size = size,
            bg_color = bg_color,
            font_size = font_size,
            initials = initials
        )
    }

    /// Get data URI for SVG
    pub fn svg_to_data_uri(svg: &str) -> String {
        format!(
            "data:image/svg+xml,{}",
            urlencoding::encode(svg)
        )
    }
}

/// Multiple avatar sources for fallback chain
#[derive(Debug, Clone)]
pub struct AvatarFallbackChain {
    sources: Vec<AvatarSource>,
}

/// Avatar source in fallback chain
#[derive(Debug, Clone)]
pub enum AvatarSource {
    Custom(String),
    Gravatar { email_hash: String, default: GravatarDefault },
    Generated { name: String },
    Static(String),
}

impl AvatarFallbackChain {
    pub fn new() -> Self {
        Self { sources: Vec::new() }
    }

    pub fn add_custom(mut self, url: &str) -> Self {
        self.sources.push(AvatarSource::Custom(url.to_string()));
        self
    }

    pub fn add_gravatar(mut self, email: &str, default: GravatarDefault) -> Self {
        self.sources.push(AvatarSource::Gravatar {
            email_hash: Avatar::hash_email(email),
            default,
        });
        self
    }

    pub fn add_generated(mut self, name: &str) -> Self {
        self.sources.push(AvatarSource::Generated { name: name.to_string() });
        self
    }

    pub fn add_static(mut self, url: &str) -> Self {
        self.sources.push(AvatarSource::Static(url.to_string()));
        self
    }

    /// Get URL for first available source
    pub fn get_url(&self, size: u32) -> String {
        for source in &self.sources {
            match source {
                AvatarSource::Custom(url) => return url.clone(),
                AvatarSource::Gravatar { email_hash, default } => {
                    return format!(
                        "https://www.gravatar.com/avatar/{}?s={}&d={}",
                        email_hash,
                        size,
                        default.as_str()
                    );
                }
                AvatarSource::Generated { name } => {
                    return helpers::svg_to_data_uri(&helpers::generate_initials_svg(name, size));
                }
                AvatarSource::Static(url) => return url.clone(),
            }
        }

        // Final fallback
        format!("https://www.gravatar.com/avatar/00000000000000000000000000000000?s={}&d=mp", size)
    }

    /// Generate onerror fallback chain JavaScript
    pub fn get_onerror_chain(&self, size: u32) -> String {
        let urls: Vec<String> = self.sources.iter()
            .skip(1) // Skip first since it's in src
            .map(|source| match source {
                AvatarSource::Custom(url) => url.clone(),
                AvatarSource::Gravatar { email_hash, default } => {
                    format!(
                        "https://www.gravatar.com/avatar/{}?s={}&d={}",
                        email_hash, size, default.as_str()
                    )
                }
                AvatarSource::Generated { name } => {
                    helpers::svg_to_data_uri(&helpers::generate_initials_svg(name, size))
                }
                AvatarSource::Static(url) => url.clone(),
            })
            .collect();

        if urls.is_empty() {
            return String::new();
        }

        let fallbacks: String = urls.iter()
            .enumerate()
            .map(|(i, url)| {
                if i == urls.len() - 1 {
                    format!("this.src='{}'", url)
                } else {
                    format!("this.src='{}'; this.onerror=function(){{", url)
                }
            })
            .collect::<Vec<_>>()
            .join("");

        let closing_braces = "}".repeat(urls.len().saturating_sub(1));

        format!("{}{}", fallbacks, closing_braces)
    }
}

impl Default for AvatarFallbackChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_hash() {
        // Test case from Gravatar docs
        let hash = Avatar::hash_email(" MyEmailAddress@example.com ");
        assert_eq!(hash, "0bc83cb571cd1c50ba6f3e8a78ef1346");
    }

    #[test]
    fn test_avatar_url() {
        let avatar = Avatar::new(1, "test@example.com");
        let url = avatar.gravatar_url(80);
        assert!(url.contains("gravatar.com"));
        assert!(url.contains("s=80"));
    }

    #[test]
    fn test_initials() {
        assert_eq!(helpers::get_initials("John Doe"), "JD");
        assert_eq!(helpers::get_initials("Alice"), "A");
        assert_eq!(helpers::get_initials("Bob Smith Jones"), "BS");
    }

    #[test]
    fn test_avatar_manager() {
        let mut manager = AvatarManager::new("/uploads");
        let avatar = manager.get_or_create_avatar(1, "test@example.com");
        assert_eq!(avatar.user_id, 1);
    }

    #[test]
    fn test_upload_validation() {
        let manager = AvatarManager::new("/uploads");

        assert!(manager.validate_upload(1024, "image/jpeg").is_ok());
        assert!(manager.validate_upload(10 * 1024 * 1024, "image/jpeg").is_err());
        assert!(manager.validate_upload(1024, "application/exe").is_err());
    }

    #[test]
    fn test_fallback_chain() {
        let chain = AvatarFallbackChain::new()
            .add_gravatar("test@example.com", GravatarDefault::Identicon)
            .add_generated("Test User");

        let url = chain.get_url(64);
        assert!(url.contains("gravatar.com"));
    }
}
