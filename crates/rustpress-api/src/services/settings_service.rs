//! Settings service for managing site configuration.

use rustpress_core::error::{Error, Result};
use rustpress_database::repository::options::{OptionRow, OptionsRepository, SettingsGroup};
use sqlx::PgPool;
use uuid::Uuid;

/// Batch update request
#[derive(Debug, Clone, serde::Deserialize)]
pub struct BatchUpdateRequest {
    pub settings: Vec<SettingUpdate>,
}

/// Single setting update within a batch
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SettingUpdate {
    pub key: String,
    pub value: serde_json::Value,
}

/// Setting response for API
#[derive(Debug, Clone, serde::Serialize)]
pub struct SettingResponse {
    pub key: String,
    pub value: serde_json::Value,
    pub group: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_type: Option<String>,
    pub is_system: bool,
}

/// Grouped settings response for API
#[derive(Debug, Clone, serde::Serialize)]
pub struct SettingsGroupResponse {
    pub group: String,
    pub display_name: String,
    pub settings: Vec<SettingResponse>,
}

/// All settings response
#[derive(Debug, Clone, serde::Serialize)]
pub struct AllSettingsResponse {
    pub groups: Vec<SettingsGroupResponse>,
}

impl From<OptionRow> for SettingResponse {
    fn from(row: OptionRow) -> Self {
        Self {
            key: row.option_name,
            value: row.option_value.unwrap_or(serde_json::Value::Null),
            group: row.option_group,
            display_name: row.display_name,
            description: row.description,
            value_type: row.value_type,
            is_system: row.is_system,
        }
    }
}

/// Settings service for handling site configuration
#[derive(Clone)]
pub struct SettingsService {
    pool: PgPool,
    site_id: Option<Uuid>,
}

impl SettingsService {
    /// Create a new settings service
    pub fn new(pool: PgPool) -> Self {
        Self { pool, site_id: None }
    }

    /// Set the site ID for multi-site support
    pub fn with_site(mut self, site_id: Uuid) -> Self {
        self.site_id = Some(site_id);
        self
    }

    /// Get the repository instance
    fn repo(&self) -> OptionsRepository {
        let repo = OptionsRepository::new(self.pool.clone());
        match self.site_id {
            Some(id) => repo.with_site(id),
            None => repo,
        }
    }

    /// Get human-readable group display name
    fn group_display_name(group: &str) -> String {
        match group {
            "general" => "General Settings".to_string(),
            "reading" => "Reading Settings".to_string(),
            "discussion" => "Discussion Settings".to_string(),
            "media" => "Media Settings".to_string(),
            "permalinks" => "Permalink Settings".to_string(),
            "privacy" => "Privacy Settings".to_string(),
            "security" => "Security Settings".to_string(),
            other => {
                // Capitalize first letter
                let mut chars = other.chars();
                match chars.next() {
                    None => String::new(),
                    Some(c) => c.to_uppercase().collect::<String>() + chars.as_str() + " Settings",
                }
            }
        }
    }

    /// Get all settings grouped
    pub async fn get_all_grouped(&self) -> Result<AllSettingsResponse> {
        let groups = self.repo().get_all_grouped().await?;

        let response_groups = groups
            .into_iter()
            .map(|g| SettingsGroupResponse {
                display_name: Self::group_display_name(&g.group),
                group: g.group,
                settings: g.settings.into_iter().map(SettingResponse::from).collect(),
            })
            .collect();

        Ok(AllSettingsResponse { groups: response_groups })
    }

    /// Get all settings as a flat list
    pub async fn get_all(&self) -> Result<Vec<SettingResponse>> {
        let options = self.repo().get_all().await?;
        Ok(options.into_iter().map(SettingResponse::from).collect())
    }

    /// Get settings by group
    pub async fn get_by_group(&self, group: &str) -> Result<SettingsGroupResponse> {
        let options = self.repo().get_by_group(group).await?;

        Ok(SettingsGroupResponse {
            display_name: Self::group_display_name(group),
            group: group.to_string(),
            settings: options.into_iter().map(SettingResponse::from).collect(),
        })
    }

    /// Get a single setting by key
    pub async fn get(&self, key: &str) -> Result<Option<SettingResponse>> {
        let option = self.repo().get_full(key).await?;
        Ok(option.map(SettingResponse::from))
    }

    /// Get a single setting value by key
    pub async fn get_value(&self, key: &str) -> Result<Option<serde_json::Value>> {
        self.repo().get(key).await
    }

    /// Update a single setting
    pub async fn update(&self, key: &str, value: serde_json::Value) -> Result<SettingResponse> {
        // First check if this setting exists
        let existing = self.repo().get_full(key).await?;

        match existing {
            Some(option) => {
                // Update existing option
                self.repo().set(key, value.clone()).await?;

                // Return updated setting
                Ok(SettingResponse {
                    key: key.to_string(),
                    value,
                    group: option.option_group,
                    display_name: option.display_name,
                    description: option.description,
                    value_type: option.value_type,
                    is_system: option.is_system,
                })
            }
            None => {
                // Create new custom setting in "general" group
                self.repo().set(key, value.clone()).await?;

                Ok(SettingResponse {
                    key: key.to_string(),
                    value,
                    group: "general".to_string(),
                    display_name: None,
                    description: None,
                    value_type: None,
                    is_system: false,
                })
            }
        }
    }

    /// Batch update multiple settings
    pub async fn batch_update(&self, updates: Vec<SettingUpdate>) -> Result<Vec<SettingResponse>> {
        let updates_vec: Vec<(String, serde_json::Value)> = updates
            .iter()
            .map(|u| (u.key.clone(), u.value.clone()))
            .collect();

        self.repo().batch_update(updates_vec).await?;

        // Return updated settings
        let mut results = Vec::new();
        for update in updates {
            if let Some(setting) = self.get(&update.key).await? {
                results.push(setting);
            }
        }

        Ok(results)
    }

    /// Delete a setting (only non-system settings)
    pub async fn delete(&self, key: &str) -> Result<bool> {
        self.repo().delete(key).await
    }

    /// Get all available groups
    pub async fn get_groups(&self) -> Result<Vec<String>> {
        self.repo().get_groups().await
    }

    /// Get autoload settings (for caching on startup)
    pub async fn get_autoload(&self) -> Result<Vec<SettingResponse>> {
        let options = self.repo().get_autoload().await?;
        Ok(options.into_iter().map(SettingResponse::from).collect())
    }

    // ==========================================================================
    // Convenience methods for common settings
    // ==========================================================================

    /// Get site title
    pub async fn get_site_title(&self) -> Result<Option<String>> {
        Ok(self.get_value("site_title")
            .await?
            .and_then(|v| v.as_str().map(String::from)))
    }

    /// Get posts per page
    pub async fn get_posts_per_page(&self) -> Result<i64> {
        self.get_value("posts_per_page")
            .await?
            .and_then(|v| v.as_i64())
            .ok_or_else(|| Error::validation("posts_per_page not configured"))
    }

    /// Check if comments are enabled globally
    pub async fn are_comments_enabled(&self) -> Result<bool> {
        Ok(self
            .get_value("comments_enabled")
            .await?
            .and_then(|v| v.as_bool())
            .unwrap_or(true))
    }

    /// Get timezone setting
    pub async fn get_timezone(&self) -> Result<String> {
        self.get_value("timezone")
            .await?
            .and_then(|v| v.as_str().map(String::from))
            .ok_or_else(|| Error::validation("timezone not configured"))
    }

    /// Get permalink structure
    pub async fn get_permalink_structure(&self) -> Result<String> {
        Ok(self.get_value("permalink_structure")
            .await?
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| "/%postname%/".to_string()))
    }
}

impl Default for SettingsService {
    fn default() -> Self {
        panic!("SettingsService requires a database pool")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_display_name() {
        assert_eq!(SettingsService::group_display_name("general"), "General Settings");
        assert_eq!(SettingsService::group_display_name("reading"), "Reading Settings");
        assert_eq!(SettingsService::group_display_name("custom"), "Custom Settings");
    }
}
