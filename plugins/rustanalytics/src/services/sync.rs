//! Sync Service
//!
//! Service for synchronizing analytics data from Google Analytics.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use tracing::{debug, error, info, warn};

use crate::models::DateRange;
use crate::services::analytics::AnalyticsService;
use crate::services::cache::CacheService;
use crate::services::client::{ClientError, GoogleAnalyticsClient};

/// Database pool type alias
type DbPool = Arc<dyn std::any::Any + Send + Sync>;

/// Sync status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncStatus {
    pub is_syncing: bool,
    pub last_sync: Option<DateTime<Utc>>,
    pub last_sync_status: Option<SyncResult>,
    pub next_scheduled_sync: Option<DateTime<Utc>>,
    pub total_syncs: u64,
    pub failed_syncs: u64,
}

/// Sync result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SyncResult {
    Success {
        records_synced: u64,
        duration_ms: u64,
    },
    PartialSuccess {
        records_synced: u64,
        errors: Vec<String>,
        duration_ms: u64,
    },
    Failed {
        error: String,
        duration_ms: u64,
    },
}

/// Sync Service for data synchronization
pub struct SyncService {
    /// GA API client (reserved for future direct API usage)
    #[allow(dead_code)]
    client: Arc<GoogleAnalyticsClient>,
    /// Analytics service
    analytics: Arc<AnalyticsService>,
    /// Cache service
    cache: Arc<CacheService>,
    /// Database pool (reserved for future database integration)
    #[allow(dead_code)]
    db: DbPool,
    /// Current sync status
    status: parking_lot::RwLock<SyncStatus>,
}

impl SyncService {
    /// Create a new sync service
    pub fn new(
        client: Arc<GoogleAnalyticsClient>,
        analytics: Arc<AnalyticsService>,
        cache: Arc<CacheService>,
        db: DbPool,
    ) -> Self {
        Self {
            client,
            analytics,
            cache,
            db,
            status: parking_lot::RwLock::new(SyncStatus {
                is_syncing: false,
                last_sync: None,
                last_sync_status: None,
                next_scheduled_sync: None,
                total_syncs: 0,
                failed_syncs: 0,
            }),
        }
    }

    /// Get current sync status
    pub fn status(&self) -> SyncStatus {
        self.status.read().clone()
    }

    /// Run a full data sync
    pub async fn sync_all(&self) -> Result<SyncResult, ClientError> {
        // Check if already syncing
        {
            let status = self.status.read();
            if status.is_syncing {
                return Err(ClientError::RequestFailed("Sync already in progress".to_string()));
            }
        }

        // Mark as syncing
        {
            let mut status = self.status.write();
            status.is_syncing = true;
        }

        let start = std::time::Instant::now();
        info!("Starting analytics data sync...");

        let result = self.perform_sync().await;

        let duration_ms = start.elapsed().as_millis() as u64;

        // Update status
        {
            let mut status = self.status.write();
            status.is_syncing = false;
            status.last_sync = Some(Utc::now());
            status.total_syncs += 1;

            match &result {
                Ok(r) => {
                    status.last_sync_status = Some(r.clone());
                }
                Err(e) => {
                    status.failed_syncs += 1;
                    status.last_sync_status = Some(SyncResult::Failed {
                        error: e.to_string(),
                        duration_ms,
                    });
                }
            }
        }

        result
    }

    /// Perform the actual sync
    async fn perform_sync(&self) -> Result<SyncResult, ClientError> {
        let start = std::time::Instant::now();
        let mut records_synced = 0u64;
        let mut errors = Vec::new();

        // Sync different date ranges
        let date_ranges = vec![
            ("today", DateRange::today()),
            ("yesterday", DateRange::yesterday()),
            ("last7days", DateRange::last_n_days(7)),
            ("last30days", DateRange::last_n_days(30)),
        ];

        for (name, date_range) in date_ranges {
            match self.sync_date_range(&date_range).await {
                Ok(count) => {
                    records_synced += count;
                    debug!("Synced {} records for {}", count, name);
                }
                Err(e) => {
                    errors.push(format!("{}: {}", name, e));
                    warn!("Failed to sync {}: {}", name, e);
                }
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        if errors.is_empty() {
            info!("Sync completed successfully: {} records in {}ms", records_synced, duration_ms);
            Ok(SyncResult::Success {
                records_synced,
                duration_ms,
            })
        } else if records_synced > 0 {
            warn!("Sync completed with errors: {} records, {} errors", records_synced, errors.len());
            Ok(SyncResult::PartialSuccess {
                records_synced,
                errors,
                duration_ms,
            })
        } else {
            error!("Sync failed completely: {:?}", errors);
            Err(ClientError::RequestFailed(errors.join("; ")))
        }
    }

    /// Sync data for a specific date range
    async fn sync_date_range(&self, date_range: &DateRange) -> Result<u64, ClientError> {
        let mut count = 0u64;

        // Sync overview data
        if let Ok(_) = self.analytics.get_overview(date_range.clone(), false).await {
            count += 1;
        }

        // Sync traffic sources
        if let Ok(sources) = self.analytics.get_traffic_sources(date_range.clone(), Some(100)).await {
            count += sources.len() as u64;
        }

        // Sync channels
        if let Ok(channels) = self.analytics.get_channels(date_range.clone()).await {
            count += channels.len() as u64;
        }

        // Sync top pages
        if let Ok(pages) = self.analytics.get_top_pages(date_range.clone(), Some(100)).await {
            count += pages.len() as u64;
        }

        // Sync referrers
        if let Ok(referrers) = self.analytics.get_referrers(date_range.clone(), Some(100)).await {
            count += referrers.len() as u64;
        }

        Ok(count)
    }

    /// Sync only essential data (for quick updates)
    pub async fn sync_essential(&self) -> Result<SyncResult, ClientError> {
        let start = std::time::Instant::now();
        let mut records_synced = 0u64;

        // Only sync today's data
        let date_range = DateRange::today();

        if let Ok(_) = self.analytics.get_overview(date_range.clone(), false).await {
            records_synced += 1;
        }

        if let Ok(pages) = self.analytics.get_top_pages(date_range, Some(10)).await {
            records_synced += pages.len() as u64;
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(SyncResult::Success {
            records_synced,
            duration_ms,
        })
    }

    /// Clear cache and force re-sync
    pub async fn force_sync(&self) -> Result<SyncResult, ClientError> {
        info!("Force syncing: clearing cache first");

        // Clear all cache
        self.cache.clear().await;

        // Run full sync
        self.sync_all().await
    }

    /// Schedule next sync
    pub fn schedule_next_sync(&self, hours_from_now: u32) {
        let mut status = self.status.write();
        status.next_scheduled_sync = Some(
            Utc::now() + chrono::Duration::hours(hours_from_now as i64)
        );
    }

    /// Cancel scheduled sync
    pub fn cancel_scheduled_sync(&self) {
        let mut status = self.status.write();
        status.next_scheduled_sync = None;
    }

    /// Check if a sync is due
    pub fn is_sync_due(&self) -> bool {
        let status = self.status.read();

        if let Some(next_sync) = status.next_scheduled_sync {
            return Utc::now() >= next_sync;
        }

        // If no next sync scheduled, check last sync
        if let Some(last_sync) = status.last_sync {
            // Sync if more than 1 hour has passed
            return Utc::now() - last_sync > chrono::Duration::hours(1);
        }

        // No sync ever run, should sync
        true
    }

    /// Get sync history (would be stored in database)
    pub async fn get_sync_history(&self, _limit: u32) -> Vec<SyncHistoryEntry> {
        // In a real implementation, this would query the database
        Vec::new()
    }
}

/// Sync history entry
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncHistoryEntry {
    pub id: uuid::Uuid,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: SyncResult,
}

impl std::fmt::Debug for SyncService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncService")
            .field("status", &self.status())
            .finish()
    }
}
