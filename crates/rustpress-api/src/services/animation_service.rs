//! Animation Library service for handling animation-related business logic.
//!
//! This service manages animations, user preferences, custom animations,
//! presets, and usage analytics.

use chrono::{DateTime, Utc};
use rustpress_core::error::{Error, Result};
use sqlx::PgPool;
use uuid::Uuid;

use crate::handlers::animations::{
    AnimationAnalyticsResponse, AnimationCategoryInfo, AnimationDefinition,
    AnimationLibraryResponse, AnimationOutput, AnimationPreset, AnimationSettings,
    AnimationStep, AnimationUsageEntry, CategoryUsage, CreateCustomAnimationRequest,
    CreatePresetRequest, CustomAnimationListQuery, CustomAnimationListResponse,
    CustomAnimationResponse, GenerateOutputRequest, PresetListResponse,
    ToggleFavoriteAnimationResponse, TrackAnimationUsageRequest,
    UpdateCustomAnimationRequest, UpdateDefaultSettingsRequest,
    UpdatePresetRequest, UpdateRecentAnimationsRequest, UserAnimationPreferencesResponse,
};

/// Database row for animations
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AnimationRow {
    pub id: String,
    pub name: String,
    pub category: String,
    pub duration: i32,
    pub preview_description: Option<String>,
    pub css_keyframes: String,
    pub css_class: String,
    pub is_pro: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

/// Database row for user animations
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserAnimationRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub category: String,
    pub duration: i32,
    pub css_keyframes: String,
    pub css_class: String,
    pub settings: serde_json::Value,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database row for user animation preferences
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserAnimationPreferencesRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub favorite_animations: serde_json::Value,
    pub recent_animations: serde_json::Value,
    pub default_settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database row for animation presets
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AnimationPresetRow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub steps: serde_json::Value,
    pub is_system: bool,
    pub user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<AnimationRow> for AnimationDefinition {
    fn from(row: AnimationRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            category: row.category,
            duration: row.duration,
            preview_description: row.preview_description,
            css_keyframes: row.css_keyframes,
            css_class: row.css_class,
            is_pro: row.is_pro,
            sort_order: row.sort_order,
        }
    }
}

impl From<UserAnimationRow> for CustomAnimationResponse {
    fn from(row: UserAnimationRow) -> Self {
        let settings: AnimationSettings = serde_json::from_value(row.settings)
            .unwrap_or_default();

        Self {
            id: row.id,
            user_id: row.user_id,
            name: row.name,
            category: row.category,
            duration: row.duration,
            css_keyframes: row.css_keyframes,
            css_class: row.css_class,
            settings,
            is_public: row.is_public,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<UserAnimationPreferencesRow> for UserAnimationPreferencesResponse {
    fn from(row: UserAnimationPreferencesRow) -> Self {
        let favorite_animations: Vec<String> = serde_json::from_value(row.favorite_animations)
            .unwrap_or_default();
        let recent_animations: Vec<String> = serde_json::from_value(row.recent_animations)
            .unwrap_or_default();
        let default_settings: AnimationSettings = serde_json::from_value(row.default_settings)
            .unwrap_or_default();

        Self {
            user_id: row.user_id,
            favorite_animations,
            recent_animations,
            default_settings,
            updated_at: row.updated_at,
        }
    }
}

impl From<AnimationPresetRow> for AnimationPreset {
    fn from(row: AnimationPresetRow) -> Self {
        let steps: Vec<AnimationStep> = serde_json::from_value(row.steps)
            .unwrap_or_default();

        Self {
            id: row.id,
            name: row.name,
            description: row.description,
            steps,
            is_system: row.is_system,
            user_id: row.user_id,
            created_at: row.created_at,
        }
    }
}

/// Animation service for handling animation library operations
#[derive(Clone)]
pub struct AnimationService {
    pool: PgPool,
}

impl AnimationService {
    /// Create a new animation service
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get all system animations
    pub async fn list_animations(&self, category: Option<&str>) -> Result<Vec<AnimationDefinition>> {
        let query = match category {
            Some(cat) => {
                sqlx::query_as::<_, AnimationRow>(
                    "SELECT * FROM animations WHERE category = $1 ORDER BY sort_order ASC"
                )
                .bind(cat)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, AnimationRow>(
                    "SELECT * FROM animations ORDER BY category, sort_order ASC"
                )
                .fetch_all(&self.pool)
                .await
            }
        };

        let rows = query.map_err(|e| Error::database_with_source("Failed to fetch animations", e))?;
        Ok(rows.into_iter().map(AnimationDefinition::from).collect())
    }

    /// Get animation by ID
    pub async fn get_animation(&self, id: &str) -> Result<Option<AnimationDefinition>> {
        let row: Option<AnimationRow> = sqlx::query_as(
            "SELECT * FROM animations WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to fetch animation", e))?;

        Ok(row.map(AnimationDefinition::from))
    }

    /// Get animation categories with counts
    pub async fn get_categories(&self) -> Result<Vec<AnimationCategoryInfo>> {
        let rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT category, COUNT(*) as count FROM animations GROUP BY category ORDER BY category"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to fetch categories", e))?;

        Ok(rows.into_iter().map(|(cat, count)| {
            let (name, description, icon) = match cat.as_str() {
                "entrance" => ("Entrance", "Animations for elements appearing", "ArrowDownCircle"),
                "exit" => ("Exit", "Animations for elements disappearing", "ArrowUpCircle"),
                "emphasis" => ("Emphasis", "Attention-grabbing animations", "Sparkles"),
                "scroll" => ("Scroll", "Scroll-triggered animations", "MousePointer"),
                "rotation" => ("Rotation", "Rotating animations", "RotateCw"),
                "scale" => ("Scale", "Scaling animations", "Maximize2"),
                "motion" => ("Motion Path", "Continuous motion animations", "Move"),
                _ => ("Custom", "User-created animations", "Box"),
            };
            AnimationCategoryInfo {
                id: cat,
                name: name.to_string(),
                description: description.to_string(),
                icon: icon.to_string(),
                count: count as i32,
            }
        }).collect())
    }

    /// Get user's animation preferences
    pub async fn get_user_preferences(&self, user_id: Uuid) -> Result<UserAnimationPreferencesResponse> {
        let row: Option<UserAnimationPreferencesRow> = sqlx::query_as(
            "SELECT * FROM user_animation_preferences WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to fetch user preferences", e))?;

        match row {
            Some(r) => Ok(UserAnimationPreferencesResponse::from(r)),
            None => {
                // Create default preferences
                let created = self.create_user_preferences(user_id).await?;
                Ok(UserAnimationPreferencesResponse::from(created))
            }
        }
    }

    /// Create default user preferences
    async fn create_user_preferences(&self, user_id: Uuid) -> Result<UserAnimationPreferencesRow> {
        let default_settings = serde_json::to_value(AnimationSettings::default())
            .unwrap_or(serde_json::json!({}));

        let row: UserAnimationPreferencesRow = sqlx::query_as(
            r#"
            INSERT INTO user_animation_preferences (user_id, favorite_animations, recent_animations, default_settings)
            VALUES ($1, '[]'::jsonb, '[]'::jsonb, $2)
            ON CONFLICT (user_id) DO UPDATE SET updated_at = NOW()
            RETURNING *
            "#
        )
        .bind(user_id)
        .bind(&default_settings)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to create user preferences", e))?;

        Ok(row)
    }

    /// Update user's recent animations
    pub async fn update_recent_animations(
        &self,
        user_id: Uuid,
        request: UpdateRecentAnimationsRequest,
    ) -> Result<UserAnimationPreferencesResponse> {
        let prefs = self.get_user_preferences(user_id).await?;

        // Add animation to recent list (max 10, most recent first)
        let mut recent = prefs.recent_animations;
        recent.retain(|a| a != &request.animation_id);
        recent.insert(0, request.animation_id);
        recent.truncate(10);

        let row: UserAnimationPreferencesRow = sqlx::query_as(
            r#"
            UPDATE user_animation_preferences
            SET recent_animations = $2, updated_at = NOW()
            WHERE user_id = $1
            RETURNING *
            "#
        )
        .bind(user_id)
        .bind(serde_json::to_value(&recent).unwrap_or_default())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to update recent animations", e))?;

        Ok(UserAnimationPreferencesResponse::from(row))
    }

    /// Toggle animation as favorite
    pub async fn toggle_favorite_animation(
        &self,
        user_id: Uuid,
        animation_id: String,
    ) -> Result<ToggleFavoriteAnimationResponse> {
        let prefs = self.get_user_preferences(user_id).await?;

        let mut favorites = prefs.favorite_animations;
        let is_favorite = if favorites.contains(&animation_id) {
            favorites.retain(|a| a != &animation_id);
            false
        } else {
            favorites.push(animation_id.clone());
            true
        };

        sqlx::query(
            r#"
            UPDATE user_animation_preferences
            SET favorite_animations = $2, updated_at = NOW()
            WHERE user_id = $1
            "#
        )
        .bind(user_id)
        .bind(serde_json::to_value(&favorites).unwrap_or_default())
        .execute(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to update favorite animations", e))?;

        Ok(ToggleFavoriteAnimationResponse {
            animation_id,
            is_favorite,
        })
    }

    /// Update user's default animation settings
    pub async fn update_default_settings(
        &self,
        user_id: Uuid,
        request: UpdateDefaultSettingsRequest,
    ) -> Result<UserAnimationPreferencesResponse> {
        let row: UserAnimationPreferencesRow = sqlx::query_as(
            r#"
            UPDATE user_animation_preferences
            SET default_settings = $2, updated_at = NOW()
            WHERE user_id = $1
            RETURNING *
            "#
        )
        .bind(user_id)
        .bind(serde_json::to_value(&request.settings).unwrap_or_default())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to update default settings", e))?;

        Ok(UserAnimationPreferencesResponse::from(row))
    }

    /// List custom animations
    pub async fn list_custom_animations(
        &self,
        user_id: Uuid,
        params: CustomAnimationListQuery,
    ) -> Result<CustomAnimationListResponse> {
        let page = params.page.unwrap_or(1).max(1);
        let per_page = params.per_page.unwrap_or(20).min(100).max(1);
        let offset = (page - 1) * per_page;

        let mut conditions = vec!["(user_id = $1 OR is_public = TRUE)".to_string()];

        if let Some(ref category) = params.category {
            conditions.push(format!("category = '{}'", category.replace('\'', "''")));
        }

        if let Some(ref search) = params.search {
            let escaped = search.replace('\'', "''");
            conditions.push(format!("name ILIKE '%{}%'", escaped));
        }

        if let Some(is_public) = params.is_public {
            conditions.push(format!("is_public = {}", is_public));
        }

        let where_clause = conditions.join(" AND ");

        // Count query
        let count_query = format!(
            "SELECT COUNT(*) FROM user_animations WHERE {}",
            where_clause
        );
        let total: (i64,) = sqlx::query_as(&count_query)
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to count custom animations", e))?;

        // Data query
        let data_query = format!(
            "SELECT * FROM user_animations WHERE {} ORDER BY created_at DESC LIMIT {} OFFSET {}",
            where_clause, per_page, offset
        );
        let rows: Vec<UserAnimationRow> = sqlx::query_as(&data_query)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to list custom animations", e))?;

        let items: Vec<CustomAnimationResponse> = rows.into_iter().map(CustomAnimationResponse::from).collect();
        let total_pages = ((total.0 as f64) / (per_page as f64)).ceil() as u64;

        Ok(CustomAnimationListResponse {
            items,
            total: total.0 as u64,
            page: page as u64,
            per_page: per_page as u64,
            total_pages,
        })
    }

    /// Create custom animation
    pub async fn create_custom_animation(
        &self,
        user_id: Uuid,
        request: CreateCustomAnimationRequest,
    ) -> Result<CustomAnimationResponse> {
        let settings = request.settings.unwrap_or_default();

        let row: UserAnimationRow = sqlx::query_as(
            r#"
            INSERT INTO user_animations (user_id, name, category, duration, css_keyframes, css_class, settings, is_public)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#
        )
        .bind(user_id)
        .bind(&request.name)
        .bind(request.category.as_deref().unwrap_or("custom"))
        .bind(request.duration.unwrap_or(500))
        .bind(&request.css_keyframes)
        .bind(&request.css_class)
        .bind(serde_json::to_value(&settings).unwrap_or_default())
        .bind(request.is_public.unwrap_or(false))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to create custom animation", e))?;

        Ok(CustomAnimationResponse::from(row))
    }

    /// Update custom animation
    pub async fn update_custom_animation(
        &self,
        id: Uuid,
        user_id: Uuid,
        request: UpdateCustomAnimationRequest,
    ) -> Result<CustomAnimationResponse> {
        // Verify ownership
        let existing: Option<UserAnimationRow> = sqlx::query_as(
            "SELECT * FROM user_animations WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to fetch animation", e))?;

        let existing = existing.ok_or_else(|| Error::not_found("Animation", id.to_string()))?;

        if existing.user_id != user_id {
            return Err(Error::forbidden("You can only update your own animations"));
        }

        let row: UserAnimationRow = sqlx::query_as(
            r#"
            UPDATE user_animations
            SET
                name = COALESCE($3, name),
                category = COALESCE($4, category),
                duration = COALESCE($5, duration),
                css_keyframes = COALESCE($6, css_keyframes),
                css_class = COALESCE($7, css_class),
                settings = COALESCE($8, settings),
                is_public = COALESCE($9, is_public),
                updated_at = NOW()
            WHERE id = $1 AND user_id = $2
            RETURNING *
            "#
        )
        .bind(id)
        .bind(user_id)
        .bind(&request.name)
        .bind(&request.category)
        .bind(request.duration)
        .bind(&request.css_keyframes)
        .bind(&request.css_class)
        .bind(request.settings.as_ref().map(|s| serde_json::to_value(s).ok()).flatten())
        .bind(request.is_public)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to update animation", e))?;

        Ok(CustomAnimationResponse::from(row))
    }

    /// Delete custom animation
    pub async fn delete_custom_animation(&self, id: Uuid, user_id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM user_animations WHERE id = $1 AND user_id = $2"
        )
        .bind(id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to delete animation", e))?;

        if result.rows_affected() == 0 {
            return Err(Error::not_found("Animation", id.to_string()));
        }

        Ok(true)
    }

    /// List animation presets
    pub async fn list_presets(&self, user_id: Uuid, page: u32, per_page: u32) -> Result<PresetListResponse> {
        let page = page.max(1);
        let per_page = per_page.min(100).max(1);
        let offset = (page - 1) * per_page;

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM animation_presets WHERE is_system = TRUE OR user_id = $1"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to count presets", e))?;

        let rows: Vec<AnimationPresetRow> = sqlx::query_as(
            r#"
            SELECT * FROM animation_presets
            WHERE is_system = TRUE OR user_id = $1
            ORDER BY is_system DESC, created_at DESC
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(user_id)
        .bind(per_page as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to list presets", e))?;

        let items: Vec<AnimationPreset> = rows.into_iter().map(AnimationPreset::from).collect();
        let total_pages = ((total.0 as f64) / (per_page as f64)).ceil() as u64;

        Ok(PresetListResponse {
            items,
            total: total.0 as u64,
            page: page as u64,
            per_page: per_page as u64,
            total_pages,
        })
    }

    /// Create animation preset
    pub async fn create_preset(
        &self,
        user_id: Uuid,
        request: CreatePresetRequest,
    ) -> Result<AnimationPreset> {
        let row: AnimationPresetRow = sqlx::query_as(
            r#"
            INSERT INTO animation_presets (name, description, steps, user_id, is_system)
            VALUES ($1, $2, $3, $4, FALSE)
            RETURNING *
            "#
        )
        .bind(&request.name)
        .bind(&request.description)
        .bind(serde_json::to_value(&request.steps).unwrap_or_default())
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to create preset", e))?;

        Ok(AnimationPreset::from(row))
    }

    /// Delete animation preset
    pub async fn delete_preset(&self, id: Uuid, user_id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM animation_presets WHERE id = $1 AND user_id = $2 AND is_system = FALSE"
        )
        .bind(id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to delete preset", e))?;

        if result.rows_affected() == 0 {
            return Err(Error::not_found("Preset", id.to_string()));
        }

        Ok(true)
    }

    /// Track animation usage for analytics
    pub async fn track_usage(
        &self,
        user_id: Option<Uuid>,
        request: TrackAnimationUsageRequest,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO animation_usage_analytics (user_id, animation_id, animation_type, post_id, action, settings)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(user_id)
        .bind(&request.animation_id)
        .bind(request.animation_type.as_deref().unwrap_or("builtin"))
        .bind(request.post_id)
        .bind(request.action.as_deref().unwrap_or("apply"))
        .bind(request.settings.as_ref().map(|s| serde_json::to_value(s).ok()).flatten().unwrap_or(serde_json::json!({})))
        .execute(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to track usage", e))?;

        Ok(())
    }

    /// Get animation usage analytics
    pub async fn get_analytics(&self, user_id: Uuid) -> Result<AnimationAnalyticsResponse> {
        // Total applications
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM animation_usage_analytics WHERE user_id = $1 AND action = 'apply'"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to count applications", e))?;

        // Unique animations
        let unique: (i64,) = sqlx::query_as(
            "SELECT COUNT(DISTINCT animation_id) FROM animation_usage_analytics WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to count unique animations", e))?;

        // Most used
        let most_used: Vec<(String, String, i64, DateTime<Utc>)> = sqlx::query_as(
            r#"
            SELECT animation_id, animation_type, COUNT(*) as count, MAX(created_at) as last_used
            FROM animation_usage_analytics
            WHERE user_id = $1
            GROUP BY animation_id, animation_type
            ORDER BY count DESC
            LIMIT 10
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to get most used", e))?;

        // Recent activity
        let recent: Vec<(String, String, i64, DateTime<Utc>)> = sqlx::query_as(
            r#"
            SELECT animation_id, animation_type, COUNT(*) as count, MAX(created_at) as last_used
            FROM animation_usage_analytics
            WHERE user_id = $1 AND created_at > NOW() - INTERVAL '7 days'
            GROUP BY animation_id, animation_type
            ORDER BY last_used DESC
            LIMIT 10
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to get recent activity", e))?;

        // Category breakdown
        let category_breakdown: Vec<(String, i64)> = sqlx::query_as(
            r#"
            SELECT a.category, COUNT(*) as count
            FROM animation_usage_analytics au
            JOIN animations a ON au.animation_id = a.id
            WHERE au.user_id = $1
            GROUP BY a.category
            ORDER BY count DESC
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        Ok(AnimationAnalyticsResponse {
            total_applications: total.0,
            unique_animations_used: unique.0,
            most_used_animations: most_used.into_iter().map(|(id, t, c, l)| AnimationUsageEntry {
                animation_id: id,
                animation_type: t,
                usage_count: c,
                last_used: l,
            }).collect(),
            recent_activity: recent.into_iter().map(|(id, t, c, l)| AnimationUsageEntry {
                animation_id: id,
                animation_type: t,
                usage_count: c,
                last_used: l,
            }).collect(),
            category_breakdown: category_breakdown.into_iter().map(|(cat, count)| CategoryUsage {
                category: cat,
                count,
            }).collect(),
        })
    }

    /// Generate animation output code
    pub fn generate_output(&self, request: GenerateOutputRequest) -> AnimationOutput {
        let settings = request.settings;
        let animation_name = request.animation_id.replace('-', "");

        // Generate inline style
        let inline_style = format!(
            "animation: {} {}ms {} {}ms {} {} {}",
            animation_name,
            settings.duration,
            settings.easing,
            settings.delay,
            if settings.repeat == -1 { "infinite".to_string() } else { settings.repeat.to_string() },
            settings.direction,
            settings.fill_mode
        );

        // Generate CSS class
        let css_class = format!("animate-{}", request.animation_id);

        // Generate data attributes
        let data_attributes = format!(
            r#"data-animation="{}" data-duration="{}" data-delay="{}" data-trigger="{}" data-offset="{}""#,
            request.animation_id,
            settings.duration,
            settings.delay,
            settings.trigger,
            settings.scroll_offset
        );

        // JS trigger code for scroll/click triggers
        let js_trigger_code = if settings.trigger != "load" {
            Some(format!(
                r#"
// Animation trigger code for {}
document.querySelectorAll('[data-animation="{}"]').forEach(el => {{
    const observer = new IntersectionObserver((entries) => {{
        entries.forEach(entry => {{
            if (entry.isIntersecting) {{
                entry.target.classList.add('{}');
                observer.unobserve(entry.target);
            }}
        }});
    }}, {{ threshold: 0.1, rootMargin: '0px 0px -{}px 0px' }});
    observer.observe(el);
}});
"#,
                request.animation_id,
                request.animation_id,
                css_class,
                settings.scroll_offset
            ))
        } else {
            None
        };

        AnimationOutput {
            inline_style,
            css_class,
            data_attributes,
            css_keyframes: String::new(), // Will be filled from animation definition
            js_trigger_code,
        }
    }

    /// Get complete animation library
    pub async fn get_animation_library(&self, user_id: Uuid) -> Result<AnimationLibraryResponse> {
        let animations = self.list_animations(None).await?;
        let categories = self.get_categories().await?;
        let prefs = self.get_user_preferences(user_id).await?;
        let custom_list = self.list_custom_animations(user_id, CustomAnimationListQuery::default()).await?;
        let preset_list = self.list_presets(user_id, 1, 50).await?;

        Ok(AnimationLibraryResponse {
            animations,
            categories,
            custom_animations: custom_list.items,
            presets: preset_list.items,
            recent_animations: prefs.recent_animations,
            favorite_animations: prefs.favorite_animations,
            default_settings: prefs.default_settings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_animation_settings() {
        let settings = AnimationSettings::default();
        assert_eq!(settings.duration, 500);
        assert_eq!(settings.delay, 0);
        assert_eq!(settings.easing, "ease");
        assert_eq!(settings.repeat, 1);
        assert_eq!(settings.direction, "normal");
        assert_eq!(settings.fill_mode, "forwards");
        assert_eq!(settings.trigger, "load");
    }

    #[tokio::test]
    async fn test_generate_inline_style() {
        let service = AnimationService::new(PgPool::connect_lazy("postgres://").unwrap());
        let request = GenerateOutputRequest {
            animation_id: "fade-in".to_string(),
            settings: AnimationSettings {
                duration: 1000,
                delay: 200,
                easing: "ease-out".to_string(),
                repeat: 1,
                ..Default::default()
            },
            output_format: "inline-style".to_string(),
            include_keyframes: None,
        };

        let output = service.generate_output(request);
        assert!(output.inline_style.contains("1000ms"));
        assert!(output.inline_style.contains("ease-out"));
        assert!(output.inline_style.contains("200ms"));
    }
}
