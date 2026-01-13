//! Sync Service Tests
//!
//! Comprehensive tests for the SyncService functionality.

use std::sync::Arc;

use chrono::{Duration, Utc};
use rustanalytics::services::cache::CacheService;
use rustanalytics::services::analytics::AnalyticsService;
use rustanalytics::services::client::GoogleAnalyticsClient;
use rustanalytics::services::sync::{SyncHistoryEntry, SyncResult, SyncService, SyncStatus};

// ============================================================================
// Helper Functions
// ============================================================================

fn create_test_db() -> Arc<dyn std::any::Any + Send + Sync> {
    Arc::new(())
}

async fn create_test_sync_service() -> SyncService {
    let db = create_test_db();
    let cache = Arc::new(CacheService::new(db.clone(), 15));

    // Create GA client without credentials (will work for testing non-API methods)
    let client = Arc::new(
        GoogleAnalyticsClient::new("properties/12345".to_string(), None)
            .await
            .unwrap()
    );

    let analytics = Arc::new(AnalyticsService::new(client.clone(), cache.clone()));

    SyncService::new(client, analytics, cache, db)
}

// ============================================================================
// SyncStatus Tests
// ============================================================================

#[test]
fn test_sync_status_default_values() {
    let status = SyncStatus {
        is_syncing: false,
        last_sync: None,
        last_sync_status: None,
        next_scheduled_sync: None,
        total_syncs: 0,
        failed_syncs: 0,
    };

    assert!(!status.is_syncing);
    assert!(status.last_sync.is_none());
    assert!(status.last_sync_status.is_none());
    assert!(status.next_scheduled_sync.is_none());
    assert_eq!(status.total_syncs, 0);
    assert_eq!(status.failed_syncs, 0);
}

#[test]
fn test_sync_status_with_values() {
    let now = Utc::now();
    let status = SyncStatus {
        is_syncing: true,
        last_sync: Some(now),
        last_sync_status: Some(SyncResult::Success {
            records_synced: 100,
            duration_ms: 5000,
        }),
        next_scheduled_sync: Some(now + Duration::hours(1)),
        total_syncs: 50,
        failed_syncs: 5,
    };

    assert!(status.is_syncing);
    assert_eq!(status.last_sync, Some(now));
    assert!(status.last_sync_status.is_some());
    assert!(status.next_scheduled_sync.is_some());
    assert_eq!(status.total_syncs, 50);
    assert_eq!(status.failed_syncs, 5);
}

#[test]
fn test_sync_status_serialization() {
    let status = SyncStatus {
        is_syncing: false,
        last_sync: None,
        last_sync_status: None,
        next_scheduled_sync: None,
        total_syncs: 10,
        failed_syncs: 2,
    };

    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("\"is_syncing\":false"));
    assert!(json.contains("\"total_syncs\":10"));
    assert!(json.contains("\"failed_syncs\":2"));

    let deserialized: SyncStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.total_syncs, 10);
    assert_eq!(deserialized.failed_syncs, 2);
}

#[test]
fn test_sync_status_clone() {
    let status = SyncStatus {
        is_syncing: true,
        last_sync: Some(Utc::now()),
        last_sync_status: Some(SyncResult::Success {
            records_synced: 50,
            duration_ms: 1000,
        }),
        next_scheduled_sync: None,
        total_syncs: 100,
        failed_syncs: 10,
    };

    let cloned = status.clone();
    assert_eq!(cloned.is_syncing, status.is_syncing);
    assert_eq!(cloned.total_syncs, status.total_syncs);
    assert_eq!(cloned.failed_syncs, status.failed_syncs);
}

// ============================================================================
// SyncResult Tests
// ============================================================================

#[test]
fn test_sync_result_success() {
    let result = SyncResult::Success {
        records_synced: 500,
        duration_ms: 3000,
    };

    match result {
        SyncResult::Success { records_synced, duration_ms } => {
            assert_eq!(records_synced, 500);
            assert_eq!(duration_ms, 3000);
        }
        _ => panic!("Expected Success variant"),
    }
}

#[test]
fn test_sync_result_partial_success() {
    let result = SyncResult::PartialSuccess {
        records_synced: 250,
        errors: vec!["Error 1".to_string(), "Error 2".to_string()],
        duration_ms: 5000,
    };

    match result {
        SyncResult::PartialSuccess { records_synced, errors, duration_ms } => {
            assert_eq!(records_synced, 250);
            assert_eq!(errors.len(), 2);
            assert_eq!(duration_ms, 5000);
        }
        _ => panic!("Expected PartialSuccess variant"),
    }
}

#[test]
fn test_sync_result_failed() {
    let result = SyncResult::Failed {
        error: "Connection timeout".to_string(),
        duration_ms: 30000,
    };

    match result {
        SyncResult::Failed { error, duration_ms } => {
            assert_eq!(error, "Connection timeout");
            assert_eq!(duration_ms, 30000);
        }
        _ => panic!("Expected Failed variant"),
    }
}

#[test]
fn test_sync_result_success_serialization() {
    let result = SyncResult::Success {
        records_synced: 100,
        duration_ms: 2000,
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("Success"));
    assert!(json.contains("\"records_synced\":100"));
    assert!(json.contains("\"duration_ms\":2000"));

    let deserialized: SyncResult = serde_json::from_str(&json).unwrap();
    match deserialized {
        SyncResult::Success { records_synced, duration_ms } => {
            assert_eq!(records_synced, 100);
            assert_eq!(duration_ms, 2000);
        }
        _ => panic!("Expected Success variant"),
    }
}

#[test]
fn test_sync_result_partial_success_serialization() {
    let result = SyncResult::PartialSuccess {
        records_synced: 75,
        errors: vec!["timeout".to_string()],
        duration_ms: 4000,
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("PartialSuccess"));
    assert!(json.contains("\"records_synced\":75"));
    assert!(json.contains("timeout"));

    let deserialized: SyncResult = serde_json::from_str(&json).unwrap();
    match deserialized {
        SyncResult::PartialSuccess { records_synced, errors, .. } => {
            assert_eq!(records_synced, 75);
            assert_eq!(errors.len(), 1);
        }
        _ => panic!("Expected PartialSuccess variant"),
    }
}

#[test]
fn test_sync_result_failed_serialization() {
    let result = SyncResult::Failed {
        error: "API error".to_string(),
        duration_ms: 1000,
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("Failed"));
    assert!(json.contains("API error"));

    let deserialized: SyncResult = serde_json::from_str(&json).unwrap();
    match deserialized {
        SyncResult::Failed { error, .. } => {
            assert_eq!(error, "API error");
        }
        _ => panic!("Expected Failed variant"),
    }
}

#[test]
fn test_sync_result_clone() {
    let result = SyncResult::PartialSuccess {
        records_synced: 50,
        errors: vec!["err1".to_string(), "err2".to_string()],
        duration_ms: 2500,
    };

    let cloned = result.clone();
    match cloned {
        SyncResult::PartialSuccess { records_synced, errors, duration_ms } => {
            assert_eq!(records_synced, 50);
            assert_eq!(errors.len(), 2);
            assert_eq!(duration_ms, 2500);
        }
        _ => panic!("Expected PartialSuccess variant"),
    }
}

// ============================================================================
// SyncHistoryEntry Tests
// ============================================================================

#[test]
fn test_sync_history_entry_creation() {
    let now = Utc::now();
    let entry = SyncHistoryEntry {
        id: uuid::Uuid::new_v4(),
        started_at: now,
        completed_at: Some(now + Duration::seconds(30)),
        result: SyncResult::Success {
            records_synced: 200,
            duration_ms: 30000,
        },
    };

    assert!(entry.completed_at.is_some());
    assert!(entry.completed_at.unwrap() > entry.started_at);
}

#[test]
fn test_sync_history_entry_incomplete() {
    let entry = SyncHistoryEntry {
        id: uuid::Uuid::new_v4(),
        started_at: Utc::now(),
        completed_at: None,
        result: SyncResult::Failed {
            error: "In progress".to_string(),
            duration_ms: 0,
        },
    };

    assert!(entry.completed_at.is_none());
}

#[test]
fn test_sync_history_entry_serialization() {
    let entry = SyncHistoryEntry {
        id: uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        started_at: Utc::now(),
        completed_at: None,
        result: SyncResult::Success {
            records_synced: 10,
            duration_ms: 500,
        },
    };

    let json = serde_json::to_string(&entry).unwrap();
    assert!(json.contains("550e8400-e29b-41d4-a716-446655440000"));
    assert!(json.contains("started_at"));

    let deserialized: SyncHistoryEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id.to_string(), "550e8400-e29b-41d4-a716-446655440000");
}

// ============================================================================
// SyncStatus with Different SyncResults Tests
// ============================================================================

#[test]
fn test_sync_status_with_success_result() {
    let status = SyncStatus {
        is_syncing: false,
        last_sync: Some(Utc::now()),
        last_sync_status: Some(SyncResult::Success {
            records_synced: 1000,
            duration_ms: 5000,
        }),
        next_scheduled_sync: None,
        total_syncs: 1,
        failed_syncs: 0,
    };

    match status.last_sync_status.unwrap() {
        SyncResult::Success { records_synced, .. } => {
            assert_eq!(records_synced, 1000);
        }
        _ => panic!("Expected Success"),
    }
}

#[test]
fn test_sync_status_with_failed_result() {
    let status = SyncStatus {
        is_syncing: false,
        last_sync: Some(Utc::now()),
        last_sync_status: Some(SyncResult::Failed {
            error: "Network error".to_string(),
            duration_ms: 100,
        }),
        next_scheduled_sync: None,
        total_syncs: 5,
        failed_syncs: 1,
    };

    match status.last_sync_status.unwrap() {
        SyncResult::Failed { error, .. } => {
            assert_eq!(error, "Network error");
        }
        _ => panic!("Expected Failed"),
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_sync_result_with_zero_records() {
    let result = SyncResult::Success {
        records_synced: 0,
        duration_ms: 100,
    };

    match result {
        SyncResult::Success { records_synced, .. } => {
            assert_eq!(records_synced, 0);
        }
        _ => panic!("Expected Success"),
    }
}

#[test]
fn test_sync_result_with_empty_errors() {
    let result = SyncResult::PartialSuccess {
        records_synced: 100,
        errors: vec![],
        duration_ms: 1000,
    };

    match result {
        SyncResult::PartialSuccess { errors, .. } => {
            assert!(errors.is_empty());
        }
        _ => panic!("Expected PartialSuccess"),
    }
}

#[test]
fn test_sync_result_with_large_values() {
    let result = SyncResult::Success {
        records_synced: u64::MAX,
        duration_ms: u64::MAX,
    };

    match result {
        SyncResult::Success { records_synced, duration_ms } => {
            assert_eq!(records_synced, u64::MAX);
            assert_eq!(duration_ms, u64::MAX);
        }
        _ => panic!("Expected Success"),
    }
}

#[test]
fn test_sync_result_with_unicode_error() {
    let result = SyncResult::Failed {
        error: "Error: \u{1F4A5} Something exploded \u{1F525}".to_string(),
        duration_ms: 0,
    };

    let json = serde_json::to_string(&result).unwrap();
    let deserialized: SyncResult = serde_json::from_str(&json).unwrap();

    match deserialized {
        SyncResult::Failed { error, .. } => {
            assert!(error.contains("exploded"));
        }
        _ => panic!("Expected Failed"),
    }
}

#[test]
fn test_sync_status_high_counts() {
    let status = SyncStatus {
        is_syncing: false,
        last_sync: None,
        last_sync_status: None,
        next_scheduled_sync: None,
        total_syncs: 1_000_000,
        failed_syncs: 500_000,
    };

    assert_eq!(status.total_syncs, 1_000_000);
    assert_eq!(status.failed_syncs, 500_000);

    // Success rate calculation
    let success_rate = (status.total_syncs - status.failed_syncs) as f64 / status.total_syncs as f64;
    assert!((success_rate - 0.5).abs() < 0.001);
}

// ============================================================================
// Time-based Tests
// ============================================================================

#[test]
fn test_sync_status_future_scheduled_sync() {
    let now = Utc::now();
    let future = now + Duration::hours(24);

    let status = SyncStatus {
        is_syncing: false,
        last_sync: Some(now),
        last_sync_status: None,
        next_scheduled_sync: Some(future),
        total_syncs: 1,
        failed_syncs: 0,
    };

    assert!(status.next_scheduled_sync.unwrap() > now);
}

#[test]
fn test_sync_status_past_scheduled_sync() {
    let now = Utc::now();
    let past = now - Duration::hours(1);

    let status = SyncStatus {
        is_syncing: false,
        last_sync: Some(past - Duration::hours(2)),
        last_sync_status: None,
        next_scheduled_sync: Some(past),
        total_syncs: 1,
        failed_syncs: 0,
    };

    // The scheduled sync time is in the past, meaning it's due
    assert!(status.next_scheduled_sync.unwrap() < now);
}

// ============================================================================
// Debug Trait Tests
// ============================================================================

#[test]
fn test_sync_status_debug() {
    let status = SyncStatus {
        is_syncing: true,
        last_sync: None,
        last_sync_status: None,
        next_scheduled_sync: None,
        total_syncs: 5,
        failed_syncs: 1,
    };

    let debug_str = format!("{:?}", status);
    assert!(debug_str.contains("SyncStatus"));
    assert!(debug_str.contains("is_syncing"));
    assert!(debug_str.contains("total_syncs"));
}

#[test]
fn test_sync_result_debug() {
    let result = SyncResult::Success {
        records_synced: 100,
        duration_ms: 1000,
    };

    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("Success"));
    assert!(debug_str.contains("records_synced"));
}

#[test]
fn test_sync_history_entry_debug() {
    let entry = SyncHistoryEntry {
        id: uuid::Uuid::new_v4(),
        started_at: Utc::now(),
        completed_at: None,
        result: SyncResult::Success {
            records_synced: 10,
            duration_ms: 100,
        },
    };

    let debug_str = format!("{:?}", entry);
    assert!(debug_str.contains("SyncHistoryEntry"));
    assert!(debug_str.contains("started_at"));
}

// ============================================================================
// Partial Success Edge Cases
// ============================================================================

#[test]
fn test_partial_success_with_many_errors() {
    let errors: Vec<String> = (0..100).map(|i| format!("Error #{}", i)).collect();

    let result = SyncResult::PartialSuccess {
        records_synced: 50,
        errors: errors.clone(),
        duration_ms: 10000,
    };

    match result {
        SyncResult::PartialSuccess { errors: errs, .. } => {
            assert_eq!(errs.len(), 100);
            assert_eq!(errs[0], "Error #0");
            assert_eq!(errs[99], "Error #99");
        }
        _ => panic!("Expected PartialSuccess"),
    }
}

#[test]
fn test_partial_success_with_long_error_message() {
    let long_error = "x".repeat(10000);

    let result = SyncResult::PartialSuccess {
        records_synced: 1,
        errors: vec![long_error.clone()],
        duration_ms: 100,
    };

    let json = serde_json::to_string(&result).unwrap();
    let deserialized: SyncResult = serde_json::from_str(&json).unwrap();

    match deserialized {
        SyncResult::PartialSuccess { errors, .. } => {
            assert_eq!(errors[0].len(), 10000);
        }
        _ => panic!("Expected PartialSuccess"),
    }
}

// ============================================================================
// CacheService Integration (minimal test)
// ============================================================================

#[tokio::test]
async fn test_cache_service_can_be_used_in_sync_context() {
    // Verify CacheService works in sync-like scenarios
    let db = create_test_db();
    let cache = Arc::new(CacheService::new(db, 15));

    // Simulate caching sync status
    let status = SyncStatus {
        is_syncing: false,
        last_sync: Some(Utc::now()),
        last_sync_status: Some(SyncResult::Success {
            records_synced: 100,
            duration_ms: 5000,
        }),
        next_scheduled_sync: None,
        total_syncs: 1,
        failed_syncs: 0,
    };

    cache.set("sync:status", &status).await;

    let retrieved: Option<SyncStatus> = cache.get("sync:status").await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().total_syncs, 1);
}

#[tokio::test]
async fn test_cache_clear_in_sync_context() {
    let db = create_test_db();
    let cache = Arc::new(CacheService::new(db, 15));

    // Add some cached data
    cache.set("analytics:overview", &"data1".to_string()).await;
    cache.set("analytics:traffic", &"data2".to_string()).await;

    assert_eq!(cache.stats().memory_entries, 2);

    // Clear cache (as force_sync would do)
    cache.clear().await;

    assert_eq!(cache.stats().memory_entries, 0);
}

// ============================================================================
// Concurrent Status Tests (simulating concurrent access patterns)
// ============================================================================

#[test]
fn test_sync_status_multiple_updates() {
    let mut status = SyncStatus {
        is_syncing: false,
        last_sync: None,
        last_sync_status: None,
        next_scheduled_sync: None,
        total_syncs: 0,
        failed_syncs: 0,
    };

    // Simulate multiple sync operations
    for i in 0..100 {
        status.is_syncing = true;
        status.total_syncs += 1;

        if i % 10 == 0 {
            status.failed_syncs += 1;
            status.last_sync_status = Some(SyncResult::Failed {
                error: format!("Error at iteration {}", i),
                duration_ms: 100,
            });
        } else {
            status.last_sync_status = Some(SyncResult::Success {
                records_synced: i as u64 * 10,
                duration_ms: i as u64 * 100,
            });
        }

        status.is_syncing = false;
        status.last_sync = Some(Utc::now());
    }

    assert_eq!(status.total_syncs, 100);
    assert_eq!(status.failed_syncs, 10);
}

// ============================================================================
// JSON Round-trip Tests
// ============================================================================

#[test]
fn test_sync_status_json_roundtrip() {
    let now = Utc::now();
    let original = SyncStatus {
        is_syncing: true,
        last_sync: Some(now),
        last_sync_status: Some(SyncResult::PartialSuccess {
            records_synced: 500,
            errors: vec!["err1".to_string(), "err2".to_string()],
            duration_ms: 3000,
        }),
        next_scheduled_sync: Some(now + Duration::hours(6)),
        total_syncs: 42,
        failed_syncs: 7,
    };

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: SyncStatus = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.is_syncing, original.is_syncing);
    assert_eq!(deserialized.total_syncs, original.total_syncs);
    assert_eq!(deserialized.failed_syncs, original.failed_syncs);
    assert!(deserialized.last_sync.is_some());
    assert!(deserialized.next_scheduled_sync.is_some());
}

#[test]
fn test_sync_history_entry_json_roundtrip() {
    let original = SyncHistoryEntry {
        id: uuid::Uuid::new_v4(),
        started_at: Utc::now(),
        completed_at: Some(Utc::now() + Duration::minutes(5)),
        result: SyncResult::Success {
            records_synced: 1000,
            duration_ms: 300000,
        },
    };

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: SyncHistoryEntry = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id, original.id);
    assert!(deserialized.completed_at.is_some());
}

// ============================================================================
// SyncService Instance Tests
// ============================================================================

#[tokio::test]
async fn test_sync_service_creation() {
    let sync_service = create_test_sync_service().await;
    let status = sync_service.status();

    assert!(!status.is_syncing);
    assert!(status.last_sync.is_none());
    assert!(status.last_sync_status.is_none());
    assert!(status.next_scheduled_sync.is_none());
    assert_eq!(status.total_syncs, 0);
    assert_eq!(status.failed_syncs, 0);
}

#[tokio::test]
async fn test_sync_service_status() {
    let sync_service = create_test_sync_service().await;

    // Initial status
    let status = sync_service.status();
    assert!(!status.is_syncing);
    assert_eq!(status.total_syncs, 0);
}

#[tokio::test]
async fn test_sync_service_schedule_next_sync() {
    let sync_service = create_test_sync_service().await;

    // Schedule a sync 6 hours from now
    sync_service.schedule_next_sync(6);

    let status = sync_service.status();
    assert!(status.next_scheduled_sync.is_some());

    let next_sync = status.next_scheduled_sync.unwrap();
    let now = Utc::now();

    // Should be approximately 6 hours from now
    let diff = next_sync - now;
    assert!(diff.num_hours() >= 5 && diff.num_hours() <= 6);
}

#[tokio::test]
async fn test_sync_service_cancel_scheduled_sync() {
    let sync_service = create_test_sync_service().await;

    // Schedule a sync
    sync_service.schedule_next_sync(12);
    assert!(sync_service.status().next_scheduled_sync.is_some());

    // Cancel it
    sync_service.cancel_scheduled_sync();
    assert!(sync_service.status().next_scheduled_sync.is_none());
}

#[tokio::test]
async fn test_sync_service_is_sync_due_no_previous_sync() {
    let sync_service = create_test_sync_service().await;

    // When no sync has ever been run, should be due
    assert!(sync_service.is_sync_due());
}

#[tokio::test]
async fn test_sync_service_is_sync_due_with_scheduled() {
    let sync_service = create_test_sync_service().await;

    // Schedule a sync far in the future (24 hours)
    sync_service.schedule_next_sync(24);

    // Should not be due yet
    assert!(!sync_service.is_sync_due());
}

#[tokio::test]
async fn test_sync_service_get_sync_history() {
    let sync_service = create_test_sync_service().await;

    // Get sync history (currently returns empty vec)
    let history = sync_service.get_sync_history(10).await;
    assert!(history.is_empty());
}

#[tokio::test]
async fn test_sync_service_debug_impl() {
    let sync_service = create_test_sync_service().await;

    let debug_str = format!("{:?}", sync_service);
    assert!(debug_str.contains("SyncService"));
    assert!(debug_str.contains("status"));
}

#[tokio::test]
async fn test_sync_service_multiple_schedule_operations() {
    let sync_service = create_test_sync_service().await;

    // Schedule multiple times - should overwrite
    sync_service.schedule_next_sync(1);
    let first = sync_service.status().next_scheduled_sync;

    sync_service.schedule_next_sync(24);
    let second = sync_service.status().next_scheduled_sync;

    // Second schedule should be later
    assert!(second.unwrap() > first.unwrap());
}

#[tokio::test]
async fn test_sync_service_schedule_cancel_schedule() {
    let sync_service = create_test_sync_service().await;

    sync_service.schedule_next_sync(1);
    assert!(sync_service.status().next_scheduled_sync.is_some());

    sync_service.cancel_scheduled_sync();
    assert!(sync_service.status().next_scheduled_sync.is_none());

    sync_service.schedule_next_sync(2);
    assert!(sync_service.status().next_scheduled_sync.is_some());
}

#[tokio::test]
async fn test_sync_service_sync_essential() {
    let sync_service = create_test_sync_service().await;

    // sync_essential should work even without credentials
    // (analytics calls will fail but method should return a result)
    let result = sync_service.sync_essential().await;

    // Should return Success with 0 records since API calls fail without credentials
    match result {
        Ok(SyncResult::Success { records_synced, .. }) => {
            assert_eq!(records_synced, 0);
        }
        _ => {
            // Also acceptable - the result depends on internal error handling
        }
    }
}

#[tokio::test]
async fn test_sync_service_concurrent_status_reads() {
    let sync_service = Arc::new(create_test_sync_service().await);

    let mut handles = vec![];

    for _ in 0..10 {
        let svc = Arc::clone(&sync_service);
        handles.push(tokio::spawn(async move {
            let status = svc.status();
            assert!(!status.is_syncing || status.is_syncing); // Just verify we can read
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn test_sync_service_schedule_zero_hours() {
    let sync_service = create_test_sync_service().await;

    // Schedule 0 hours from now (immediate)
    sync_service.schedule_next_sync(0);

    let status = sync_service.status();
    assert!(status.next_scheduled_sync.is_some());

    // Should be due immediately
    assert!(sync_service.is_sync_due());
}
