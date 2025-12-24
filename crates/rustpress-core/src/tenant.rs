//! Multi-tenancy support for RustPress.
//!
//! Enables SaaS deployments with isolated tenant data.

use crate::id::TenantId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tenant status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TenantStatus {
    Active,
    Trial,
    Suspended,
    Cancelled,
    Pending,
}

impl Default for TenantStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Tenant subscription plan
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TenantPlan {
    Free,
    Starter,
    Professional,
    Enterprise,
    Custom(String),
}

impl Default for TenantPlan {
    fn default() -> Self {
        Self::Free
    }
}

/// Tenant entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    /// Unique tenant ID
    pub id: TenantId,
    /// Tenant slug (used in URLs)
    pub slug: String,
    /// Display name
    pub name: String,
    /// Tenant domain (for custom domain support)
    pub domain: Option<String>,
    /// Status
    pub status: TenantStatus,
    /// Subscription plan
    pub plan: TenantPlan,
    /// Settings/configuration
    pub settings: TenantSettings,
    /// Quota limits
    pub quotas: TenantQuotas,
    /// Owner user ID
    pub owner_id: Option<crate::id::UserId>,
    /// Trial end date
    pub trial_ends_at: Option<DateTime<Utc>>,
    /// Subscription end date
    pub subscription_ends_at: Option<DateTime<Utc>>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Tenant {
    pub fn new(slug: impl Into<String>, name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: TenantId::new(),
            slug: slug.into(),
            name: name.into(),
            domain: None,
            status: TenantStatus::default(),
            plan: TenantPlan::default(),
            settings: TenantSettings::default(),
            quotas: TenantQuotas::default(),
            owner_id: None,
            trial_ends_at: None,
            subscription_ends_at: None,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, TenantStatus::Active | TenantStatus::Trial)
    }

    pub fn is_trial(&self) -> bool {
        self.status == TenantStatus::Trial
    }

    pub fn is_trial_expired(&self) -> bool {
        if let Some(trial_ends) = self.trial_ends_at {
            self.is_trial() && Utc::now() > trial_ends
        } else {
            false
        }
    }

    pub fn is_subscription_expired(&self) -> bool {
        if let Some(sub_ends) = self.subscription_ends_at {
            Utc::now() > sub_ends
        } else {
            false
        }
    }

    pub fn can_access(&self) -> bool {
        self.is_active() && !self.is_trial_expired() && !self.is_subscription_expired()
    }
}

/// Tenant settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TenantSettings {
    /// Default locale
    pub default_locale: String,
    /// Timezone
    pub timezone: String,
    /// Date format
    pub date_format: String,
    /// Time format
    pub time_format: String,
    /// Enable user registration
    pub enable_registration: bool,
    /// Enable comments
    pub enable_comments: bool,
    /// Require comment moderation
    pub moderate_comments: bool,
    /// Custom branding
    pub branding: TenantBranding,
    /// Feature flags
    pub features: HashMap<String, bool>,
}

/// Tenant branding
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TenantBranding {
    pub logo_url: Option<String>,
    pub favicon_url: Option<String>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub custom_css: Option<String>,
}

/// Tenant quotas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantQuotas {
    /// Maximum number of users
    pub max_users: Option<u32>,
    /// Maximum storage in bytes
    pub max_storage_bytes: Option<u64>,
    /// Maximum posts
    pub max_posts: Option<u32>,
    /// Maximum pages
    pub max_pages: Option<u32>,
    /// Maximum media files
    pub max_media: Option<u32>,
    /// Maximum API requests per day
    pub max_api_requests_per_day: Option<u32>,
    /// Maximum plugins
    pub max_plugins: Option<u32>,
}

impl Default for TenantQuotas {
    fn default() -> Self {
        Self {
            max_users: Some(5),
            max_storage_bytes: Some(1024 * 1024 * 1024), // 1GB
            max_posts: Some(1000),
            max_pages: Some(100),
            max_media: Some(500),
            max_api_requests_per_day: Some(10000),
            max_plugins: Some(10),
        }
    }
}

impl TenantQuotas {
    pub fn unlimited() -> Self {
        Self {
            max_users: None,
            max_storage_bytes: None,
            max_posts: None,
            max_pages: None,
            max_media: None,
            max_api_requests_per_day: None,
            max_plugins: None,
        }
    }

    pub fn free_tier() -> Self {
        Self {
            max_users: Some(2),
            max_storage_bytes: Some(100 * 1024 * 1024), // 100MB
            max_posts: Some(50),
            max_pages: Some(10),
            max_media: Some(50),
            max_api_requests_per_day: Some(1000),
            max_plugins: Some(3),
        }
    }

    pub fn starter_tier() -> Self {
        Self {
            max_users: Some(10),
            max_storage_bytes: Some(5 * 1024 * 1024 * 1024), // 5GB
            max_posts: Some(1000),
            max_pages: Some(100),
            max_media: Some(1000),
            max_api_requests_per_day: Some(50000),
            max_plugins: Some(10),
        }
    }

    pub fn professional_tier() -> Self {
        Self {
            max_users: Some(50),
            max_storage_bytes: Some(50 * 1024 * 1024 * 1024), // 50GB
            max_posts: None,
            max_pages: None,
            max_media: Some(10000),
            max_api_requests_per_day: Some(500000),
            max_plugins: Some(50),
        }
    }

    pub fn enterprise_tier() -> Self {
        Self::unlimited()
    }
}

/// Tenant usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TenantUsage {
    pub user_count: u32,
    pub storage_bytes: u64,
    pub post_count: u32,
    pub page_count: u32,
    pub media_count: u32,
    pub api_requests_today: u32,
    pub plugin_count: u32,
}

impl TenantUsage {
    pub fn is_within_quota(&self, quotas: &TenantQuotas) -> bool {
        let within = |usage: u32, quota: Option<u32>| quota.map_or(true, |q| usage < q);
        let within_storage = |usage: u64, quota: Option<u64>| quota.map_or(true, |q| usage < q);

        within(self.user_count, quotas.max_users)
            && within_storage(self.storage_bytes, quotas.max_storage_bytes)
            && within(self.post_count, quotas.max_posts)
            && within(self.page_count, quotas.max_pages)
            && within(self.media_count, quotas.max_media)
            && within(self.api_requests_today, quotas.max_api_requests_per_day)
            && within(self.plugin_count, quotas.max_plugins)
    }

    pub fn quota_violations(&self, quotas: &TenantQuotas) -> Vec<QuotaViolation> {
        let mut violations = Vec::new();

        if let Some(max) = quotas.max_users {
            if self.user_count >= max {
                violations.push(QuotaViolation::Users {
                    current: self.user_count,
                    max,
                });
            }
        }

        if let Some(max) = quotas.max_storage_bytes {
            if self.storage_bytes >= max {
                violations.push(QuotaViolation::Storage {
                    current: self.storage_bytes,
                    max,
                });
            }
        }

        if let Some(max) = quotas.max_posts {
            if self.post_count >= max {
                violations.push(QuotaViolation::Posts {
                    current: self.post_count,
                    max,
                });
            }
        }

        if let Some(max) = quotas.max_api_requests_per_day {
            if self.api_requests_today >= max {
                violations.push(QuotaViolation::ApiRequests {
                    current: self.api_requests_today,
                    max,
                });
            }
        }

        violations
    }
}

/// Quota violation types
#[derive(Debug, Clone)]
pub enum QuotaViolation {
    Users { current: u32, max: u32 },
    Storage { current: u64, max: u64 },
    Posts { current: u32, max: u32 },
    Pages { current: u32, max: u32 },
    Media { current: u32, max: u32 },
    ApiRequests { current: u32, max: u32 },
    Plugins { current: u32, max: u32 },
}

impl std::fmt::Display for QuotaViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Users { current, max } => {
                write!(f, "User quota exceeded: {} of {} users", current, max)
            }
            Self::Storage { current, max } => {
                write!(
                    f,
                    "Storage quota exceeded: {} of {} bytes",
                    current, max
                )
            }
            Self::Posts { current, max } => {
                write!(f, "Post quota exceeded: {} of {} posts", current, max)
            }
            Self::Pages { current, max } => {
                write!(f, "Page quota exceeded: {} of {} pages", current, max)
            }
            Self::Media { current, max } => {
                write!(f, "Media quota exceeded: {} of {} files", current, max)
            }
            Self::ApiRequests { current, max } => {
                write!(
                    f,
                    "API request quota exceeded: {} of {} requests",
                    current, max
                )
            }
            Self::Plugins { current, max } => {
                write!(f, "Plugin quota exceeded: {} of {} plugins", current, max)
            }
        }
    }
}

/// Tenant resolver for identifying tenants from requests
pub trait TenantResolver: Send + Sync {
    /// Resolve tenant from subdomain
    fn from_subdomain(&self, subdomain: &str) -> Option<TenantId>;

    /// Resolve tenant from custom domain
    fn from_domain(&self, domain: &str) -> Option<TenantId>;

    /// Resolve tenant from header
    fn from_header(&self, value: &str) -> Option<TenantId>;

    /// Resolve tenant from path prefix
    fn from_path(&self, path: &str) -> Option<TenantId>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_creation() {
        let tenant = Tenant::new("acme", "Acme Corporation");
        assert_eq!(tenant.slug, "acme");
        assert_eq!(tenant.name, "Acme Corporation");
        assert_eq!(tenant.status, TenantStatus::Pending);
    }

    #[test]
    fn test_tenant_access() {
        let mut tenant = Tenant::new("test", "Test");
        assert!(!tenant.can_access()); // Pending status

        tenant.status = TenantStatus::Active;
        assert!(tenant.can_access());

        tenant.status = TenantStatus::Suspended;
        assert!(!tenant.can_access());
    }

    #[test]
    fn test_quota_check() {
        let quotas = TenantQuotas::free_tier();
        let mut usage = TenantUsage::default();

        assert!(usage.is_within_quota(&quotas));

        usage.user_count = 100;
        assert!(!usage.is_within_quota(&quotas));

        let violations = usage.quota_violations(&quotas);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_unlimited_quotas() {
        let quotas = TenantQuotas::unlimited();
        let usage = TenantUsage {
            user_count: 1000000,
            storage_bytes: u64::MAX,
            post_count: 1000000,
            page_count: 1000000,
            media_count: 1000000,
            api_requests_today: 1000000,
            plugin_count: 1000000,
        };

        assert!(usage.is_within_quota(&quotas));
    }
}
