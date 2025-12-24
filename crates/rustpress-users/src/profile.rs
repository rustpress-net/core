//! # User Profile System
//!
//! User profile editing and management.
//!
//! Features:
//! - Profile field management
//! - Profile validation
//! - Profile history
//! - Social links

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// User profile data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// User ID
    pub user_id: i64,

    /// Username (login name)
    pub username: String,

    /// Email address
    pub email: String,

    /// Display name
    pub display_name: String,

    /// First name
    pub first_name: String,

    /// Last name
    pub last_name: String,

    /// Nickname
    pub nickname: String,

    /// Website URL
    pub url: String,

    /// Biographical info
    pub description: String,

    /// Rich text bio
    pub rich_bio: Option<String>,

    /// Social links
    pub social_links: SocialLinks,

    /// Profile visibility
    pub visibility: ProfileVisibility,

    /// Language preference
    pub locale: String,

    /// Timezone
    pub timezone: String,

    /// Date format preference
    pub date_format: String,

    /// Time format preference
    pub time_format: String,

    /// Custom meta fields
    pub meta: HashMap<String, serde_json::Value>,

    /// Profile created at
    pub created_at: DateTime<Utc>,

    /// Last updated
    pub updated_at: DateTime<Utc>,
}

/// Profile visibility level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileVisibility {
    Public,
    RegisteredOnly,
    Private,
}

/// Social media links
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SocialLinks {
    pub twitter: Option<String>,
    pub facebook: Option<String>,
    pub instagram: Option<String>,
    pub linkedin: Option<String>,
    pub youtube: Option<String>,
    pub github: Option<String>,
    pub mastodon: Option<String>,
    pub tiktok: Option<String>,
    pub pinterest: Option<String>,
    pub custom: HashMap<String, String>,
}

impl SocialLinks {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, platform: &str, url: &str) {
        match platform.to_lowercase().as_str() {
            "twitter" | "x" => self.twitter = Some(url.to_string()),
            "facebook" => self.facebook = Some(url.to_string()),
            "instagram" => self.instagram = Some(url.to_string()),
            "linkedin" => self.linkedin = Some(url.to_string()),
            "youtube" => self.youtube = Some(url.to_string()),
            "github" => self.github = Some(url.to_string()),
            "mastodon" => self.mastodon = Some(url.to_string()),
            "tiktok" => self.tiktok = Some(url.to_string()),
            "pinterest" => self.pinterest = Some(url.to_string()),
            _ => { self.custom.insert(platform.to_string(), url.to_string()); }
        }
    }

    pub fn get(&self, platform: &str) -> Option<&String> {
        match platform.to_lowercase().as_str() {
            "twitter" | "x" => self.twitter.as_ref(),
            "facebook" => self.facebook.as_ref(),
            "instagram" => self.instagram.as_ref(),
            "linkedin" => self.linkedin.as_ref(),
            "youtube" => self.youtube.as_ref(),
            "github" => self.github.as_ref(),
            "mastodon" => self.mastodon.as_ref(),
            "tiktok" => self.tiktok.as_ref(),
            "pinterest" => self.pinterest.as_ref(),
            _ => self.custom.get(platform),
        }
    }

    pub fn all(&self) -> Vec<(&str, &str)> {
        let mut links = Vec::new();

        if let Some(ref url) = self.twitter { links.push(("twitter", url.as_str())); }
        if let Some(ref url) = self.facebook { links.push(("facebook", url.as_str())); }
        if let Some(ref url) = self.instagram { links.push(("instagram", url.as_str())); }
        if let Some(ref url) = self.linkedin { links.push(("linkedin", url.as_str())); }
        if let Some(ref url) = self.youtube { links.push(("youtube", url.as_str())); }
        if let Some(ref url) = self.github { links.push(("github", url.as_str())); }
        if let Some(ref url) = self.mastodon { links.push(("mastodon", url.as_str())); }
        if let Some(ref url) = self.tiktok { links.push(("tiktok", url.as_str())); }
        if let Some(ref url) = self.pinterest { links.push(("pinterest", url.as_str())); }

        for (platform, url) in &self.custom {
            links.push((platform.as_str(), url.as_str()));
        }

        links
    }
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            user_id: 0,
            username: String::new(),
            email: String::new(),
            display_name: String::new(),
            first_name: String::new(),
            last_name: String::new(),
            nickname: String::new(),
            url: String::new(),
            description: String::new(),
            rich_bio: None,
            social_links: SocialLinks::default(),
            visibility: ProfileVisibility::Public,
            locale: "en_US".to_string(),
            timezone: "UTC".to_string(),
            date_format: "F j, Y".to_string(),
            time_format: "g:i a".to_string(),
            meta: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl UserProfile {
    pub fn new(user_id: i64, username: &str, email: &str) -> Self {
        let mut profile = Self::default();
        profile.user_id = user_id;
        profile.username = username.to_string();
        profile.email = email.to_string();
        profile.display_name = username.to_string();
        profile.nickname = username.to_string();
        profile
    }

    /// Get full name
    pub fn full_name(&self) -> String {
        if !self.first_name.is_empty() && !self.last_name.is_empty() {
            format!("{} {}", self.first_name, self.last_name)
        } else if !self.first_name.is_empty() {
            self.first_name.clone()
        } else if !self.last_name.is_empty() {
            self.last_name.clone()
        } else {
            self.display_name.clone()
        }
    }

    /// Get preferred display name
    pub fn get_display_name(&self) -> &str {
        if !self.display_name.is_empty() {
            &self.display_name
        } else if !self.nickname.is_empty() {
            &self.nickname
        } else {
            &self.username
        }
    }

    /// Set meta value
    pub fn set_meta(&mut self, key: &str, value: serde_json::Value) {
        self.meta.insert(key.to_string(), value);
        self.updated_at = Utc::now();
    }

    /// Get meta value
    pub fn get_meta(&self, key: &str) -> Option<&serde_json::Value> {
        self.meta.get(key)
    }

    /// Get meta as string
    pub fn get_meta_string(&self, key: &str) -> Option<String> {
        self.meta.get(key).and_then(|v| v.as_str().map(String::from))
    }
}

/// Profile update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileUpdate {
    pub display_name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub nickname: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub locale: Option<String>,
    pub timezone: Option<String>,
    pub social_links: Option<SocialLinks>,
    pub visibility: Option<ProfileVisibility>,
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

impl ProfileUpdate {
    pub fn new() -> Self {
        Self {
            display_name: None,
            first_name: None,
            last_name: None,
            nickname: None,
            url: None,
            description: None,
            locale: None,
            timezone: None,
            social_links: None,
            visibility: None,
            meta: None,
        }
    }

    /// Apply update to profile
    pub fn apply_to(&self, profile: &mut UserProfile) {
        if let Some(ref v) = self.display_name { profile.display_name = v.clone(); }
        if let Some(ref v) = self.first_name { profile.first_name = v.clone(); }
        if let Some(ref v) = self.last_name { profile.last_name = v.clone(); }
        if let Some(ref v) = self.nickname { profile.nickname = v.clone(); }
        if let Some(ref v) = self.url { profile.url = v.clone(); }
        if let Some(ref v) = self.description { profile.description = v.clone(); }
        if let Some(ref v) = self.locale { profile.locale = v.clone(); }
        if let Some(ref v) = self.timezone { profile.timezone = v.clone(); }
        if let Some(ref v) = self.social_links { profile.social_links = v.clone(); }
        if let Some(v) = self.visibility { profile.visibility = v; }
        if let Some(ref meta) = self.meta {
            for (key, value) in meta {
                profile.meta.insert(key.clone(), value.clone());
            }
        }
        profile.updated_at = Utc::now();
    }
}

impl Default for ProfileUpdate {
    fn default() -> Self {
        Self::new()
    }
}

/// Profile change history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileChangeHistory {
    pub id: Uuid,
    pub user_id: i64,
    pub changed_by: i64,
    pub changed_at: DateTime<Utc>,
    pub changes: Vec<ProfileChange>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Individual field change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileChange {
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

impl ProfileChangeHistory {
    pub fn new(user_id: i64, changed_by: i64) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            changed_by,
            changed_at: Utc::now(),
            changes: Vec::new(),
            ip_address: None,
            user_agent: None,
        }
    }

    pub fn add_change(&mut self, field: &str, old: Option<&str>, new: Option<&str>) {
        self.changes.push(ProfileChange {
            field: field.to_string(),
            old_value: old.map(String::from),
            new_value: new.map(String::from),
        });
    }
}

/// Profile field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileField {
    pub id: String,
    pub label: String,
    pub field_type: ProfileFieldType,
    pub section: String,
    pub order: i32,
    pub required: bool,
    pub editable: bool,
    pub visible_in_admin: bool,
    pub visible_in_frontend: bool,
    pub validation_rules: Vec<String>,
    pub help_text: Option<String>,
}

/// Profile field types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProfileFieldType {
    Text,
    Textarea,
    RichText,
    Email,
    Url,
    Select(Vec<String>),
    Checkbox,
    Date,
    Image,
    Custom(String),
}

/// Profile field section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSection {
    pub id: String,
    pub label: String,
    pub order: i32,
    pub description: Option<String>,
}

/// Profile manager
pub struct ProfileManager {
    profiles: HashMap<i64, UserProfile>,
    history: HashMap<i64, Vec<ProfileChangeHistory>>,
    fields: Vec<ProfileField>,
    sections: Vec<ProfileSection>,
}

impl Default for ProfileManager {
    fn default() -> Self {
        let mut manager = Self {
            profiles: HashMap::new(),
            history: HashMap::new(),
            fields: Vec::new(),
            sections: Vec::new(),
        };

        manager.register_default_fields();
        manager
    }
}

impl ProfileManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register default profile fields
    fn register_default_fields(&mut self) {
        // Sections
        self.sections.push(ProfileSection {
            id: "name".to_string(),
            label: "Name".to_string(),
            order: 1,
            description: None,
        });

        self.sections.push(ProfileSection {
            id: "contact".to_string(),
            label: "Contact Info".to_string(),
            order: 2,
            description: None,
        });

        self.sections.push(ProfileSection {
            id: "about".to_string(),
            label: "About Yourself".to_string(),
            order: 3,
            description: None,
        });

        self.sections.push(ProfileSection {
            id: "social".to_string(),
            label: "Social Links".to_string(),
            order: 4,
            description: None,
        });

        // Fields
        self.fields.push(ProfileField {
            id: "first_name".to_string(),
            label: "First Name".to_string(),
            field_type: ProfileFieldType::Text,
            section: "name".to_string(),
            order: 1,
            required: false,
            editable: true,
            visible_in_admin: true,
            visible_in_frontend: true,
            validation_rules: Vec::new(),
            help_text: None,
        });

        self.fields.push(ProfileField {
            id: "last_name".to_string(),
            label: "Last Name".to_string(),
            field_type: ProfileFieldType::Text,
            section: "name".to_string(),
            order: 2,
            required: false,
            editable: true,
            visible_in_admin: true,
            visible_in_frontend: true,
            validation_rules: Vec::new(),
            help_text: None,
        });

        self.fields.push(ProfileField {
            id: "nickname".to_string(),
            label: "Nickname".to_string(),
            field_type: ProfileFieldType::Text,
            section: "name".to_string(),
            order: 3,
            required: true,
            editable: true,
            visible_in_admin: true,
            visible_in_frontend: false,
            validation_rules: Vec::new(),
            help_text: None,
        });

        self.fields.push(ProfileField {
            id: "display_name".to_string(),
            label: "Display Name".to_string(),
            field_type: ProfileFieldType::Select(vec![
                "username".to_string(),
                "first_last".to_string(),
                "last_first".to_string(),
                "nickname".to_string(),
            ]),
            section: "name".to_string(),
            order: 4,
            required: true,
            editable: true,
            visible_in_admin: true,
            visible_in_frontend: false,
            validation_rules: Vec::new(),
            help_text: Some("How your name will appear publicly".to_string()),
        });

        self.fields.push(ProfileField {
            id: "email".to_string(),
            label: "Email".to_string(),
            field_type: ProfileFieldType::Email,
            section: "contact".to_string(),
            order: 1,
            required: true,
            editable: true,
            visible_in_admin: true,
            visible_in_frontend: false,
            validation_rules: vec!["email".to_string()],
            help_text: Some("Used for notifications and account recovery".to_string()),
        });

        self.fields.push(ProfileField {
            id: "url".to_string(),
            label: "Website".to_string(),
            field_type: ProfileFieldType::Url,
            section: "contact".to_string(),
            order: 2,
            required: false,
            editable: true,
            visible_in_admin: true,
            visible_in_frontend: true,
            validation_rules: vec!["url".to_string()],
            help_text: None,
        });

        self.fields.push(ProfileField {
            id: "description".to_string(),
            label: "Biographical Info".to_string(),
            field_type: ProfileFieldType::Textarea,
            section: "about".to_string(),
            order: 1,
            required: false,
            editable: true,
            visible_in_admin: true,
            visible_in_frontend: true,
            validation_rules: Vec::new(),
            help_text: Some("Share a little about yourself".to_string()),
        });
    }

    /// Get profile by user ID
    pub fn get_profile(&self, user_id: i64) -> Option<&UserProfile> {
        self.profiles.get(&user_id)
    }

    /// Get profile mutable
    pub fn get_profile_mut(&mut self, user_id: i64) -> Option<&mut UserProfile> {
        self.profiles.get_mut(&user_id)
    }

    /// Store or update profile
    pub fn save_profile(&mut self, profile: UserProfile) {
        self.profiles.insert(profile.user_id, profile);
    }

    /// Update profile with change tracking
    pub fn update_profile(
        &mut self,
        user_id: i64,
        update: ProfileUpdate,
        changed_by: i64,
        ip: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<(), String> {
        let profile = self.profiles.get_mut(&user_id)
            .ok_or_else(|| "Profile not found".to_string())?;

        // Track changes
        let mut history = ProfileChangeHistory::new(user_id, changed_by);
        history.ip_address = ip.map(String::from);
        history.user_agent = user_agent.map(String::from);

        if let Some(ref new) = update.display_name {
            if new != &profile.display_name {
                history.add_change("display_name", Some(&profile.display_name), Some(new));
            }
        }
        if let Some(ref new) = update.first_name {
            if new != &profile.first_name {
                history.add_change("first_name", Some(&profile.first_name), Some(new));
            }
        }
        if let Some(ref new) = update.last_name {
            if new != &profile.last_name {
                history.add_change("last_name", Some(&profile.last_name), Some(new));
            }
        }
        if let Some(ref new) = update.description {
            if new != &profile.description {
                history.add_change("description", Some(&profile.description), Some(new));
            }
        }

        // Apply update
        update.apply_to(profile);

        // Store history if changes were made
        if !history.changes.is_empty() {
            self.history.entry(user_id)
                .or_insert_with(Vec::new)
                .push(history);
        }

        Ok(())
    }

    /// Get profile change history
    pub fn get_history(&self, user_id: i64) -> Option<&Vec<ProfileChangeHistory>> {
        self.history.get(&user_id)
    }

    /// Add custom profile field
    pub fn add_field(&mut self, field: ProfileField) {
        self.fields.push(field);
        self.fields.sort_by_key(|f| (f.section.clone(), f.order));
    }

    /// Get fields by section
    pub fn get_fields_by_section(&self, section: &str) -> Vec<&ProfileField> {
        self.fields.iter()
            .filter(|f| f.section == section)
            .collect()
    }

    /// Get all visible fields for frontend
    pub fn get_frontend_fields(&self) -> Vec<&ProfileField> {
        self.fields.iter()
            .filter(|f| f.visible_in_frontend)
            .collect()
    }

    /// Get all sections
    pub fn get_sections(&self) -> &[ProfileSection] {
        &self.sections
    }

    /// Add section
    pub fn add_section(&mut self, section: ProfileSection) {
        self.sections.push(section);
        self.sections.sort_by_key(|s| s.order);
    }

    /// Search profiles
    pub fn search(&self, query: &str) -> Vec<&UserProfile> {
        let query_lower = query.to_lowercase();
        self.profiles.values()
            .filter(|p| {
                p.username.to_lowercase().contains(&query_lower)
                    || p.display_name.to_lowercase().contains(&query_lower)
                    || p.email.to_lowercase().contains(&query_lower)
                    || p.first_name.to_lowercase().contains(&query_lower)
                    || p.last_name.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Get public profiles
    pub fn get_public_profiles(&self) -> Vec<&UserProfile> {
        self.profiles.values()
            .filter(|p| p.visibility == ProfileVisibility::Public)
            .collect()
    }
}

/// Email change request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailChangeRequest {
    pub id: Uuid,
    pub user_id: i64,
    pub current_email: String,
    pub new_email: String,
    pub verification_token: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub verified: bool,
}

impl EmailChangeRequest {
    pub fn new(user_id: i64, current_email: &str, new_email: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            current_email: current_email.to_string(),
            new_email: new_email.to_string(),
            verification_token: Uuid::new_v4().to_string(),
            created_at: now,
            expires_at: now + chrono::Duration::hours(24),
            verified: false,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// Password change request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordChangeRequest {
    pub user_id: i64,
    pub current_password_hash: String,
    pub new_password_hash: String,
    pub changed_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub require_logout_all: bool,
}

impl PasswordChangeRequest {
    pub fn new(user_id: i64, current_hash: &str, new_hash: &str) -> Self {
        Self {
            user_id,
            current_password_hash: current_hash.to_string(),
            new_password_hash: new_hash.to_string(),
            changed_at: Utc::now(),
            ip_address: None,
            require_logout_all: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_profile() {
        let profile = UserProfile::new(1, "testuser", "test@example.com");
        assert_eq!(profile.username, "testuser");
        assert_eq!(profile.get_display_name(), "testuser");
    }

    #[test]
    fn test_full_name() {
        let mut profile = UserProfile::new(1, "testuser", "test@example.com");
        profile.first_name = "John".to_string();
        profile.last_name = "Doe".to_string();
        assert_eq!(profile.full_name(), "John Doe");
    }

    #[test]
    fn test_social_links() {
        let mut links = SocialLinks::new();
        links.set("twitter", "https://twitter.com/user");
        links.set("custom_site", "https://custom.com");

        assert_eq!(links.get("twitter"), Some(&"https://twitter.com/user".to_string()));
        assert_eq!(links.get("custom_site"), Some(&"https://custom.com".to_string()));
    }

    #[test]
    fn test_profile_update() {
        let mut profile = UserProfile::new(1, "testuser", "test@example.com");
        let mut update = ProfileUpdate::new();
        update.first_name = Some("Jane".to_string());
        update.display_name = Some("Jane D".to_string());

        update.apply_to(&mut profile);

        assert_eq!(profile.first_name, "Jane");
        assert_eq!(profile.display_name, "Jane D");
    }

    #[test]
    fn test_profile_manager() {
        let mut manager = ProfileManager::new();
        let profile = UserProfile::new(1, "user1", "user1@example.com");
        manager.save_profile(profile);

        assert!(manager.get_profile(1).is_some());
        assert!(manager.get_profile(999).is_none());
    }
}
