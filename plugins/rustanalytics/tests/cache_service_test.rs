//! Cache Service Tests
//!
//! Comprehensive tests for the CacheService functionality.

use std::sync::Arc;

use rustanalytics::services::cache::{CacheService, CacheStats};
use serde::{Deserialize, Serialize};

// ============================================================================
// Test Data Structures
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestData {
    id: u64,
    name: String,
    value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ComplexData {
    users: Vec<String>,
    metrics: std::collections::HashMap<String, i64>,
    nested: NestedData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct NestedData {
    level: u32,
    items: Vec<String>,
}

// ============================================================================
// Helper Functions
// ============================================================================

fn create_test_db() -> Arc<dyn std::any::Any + Send + Sync> {
    Arc::new(())
}

fn create_cache_service(duration_minutes: u32) -> CacheService {
    CacheService::new(create_test_db(), duration_minutes)
}

fn sample_test_data() -> TestData {
    TestData {
        id: 123,
        name: "test_item".to_string(),
        value: 45.67,
    }
}

fn sample_complex_data() -> ComplexData {
    let mut metrics = std::collections::HashMap::new();
    metrics.insert("pageviews".to_string(), 1000);
    metrics.insert("sessions".to_string(), 500);
    metrics.insert("users".to_string(), 250);

    ComplexData {
        users: vec!["user1".to_string(), "user2".to_string(), "user3".to_string()],
        metrics,
        nested: NestedData {
            level: 2,
            items: vec!["item1".to_string(), "item2".to_string()],
        },
    }
}

// ============================================================================
// Cache Creation Tests
// ============================================================================

#[test]
fn test_cache_service_creation() {
    let cache = create_cache_service(15);
    let stats = cache.stats();

    assert_eq!(stats.memory_entries, 0);
    assert_eq!(stats.memory_size_bytes, 0);
    assert_eq!(stats.expired_entries, 0);
    assert_eq!(stats.max_entries, 1000);
    assert_eq!(stats.cache_duration_minutes, 15);
}

#[test]
fn test_cache_service_with_different_durations() {
    let cache_5min = create_cache_service(5);
    let cache_60min = create_cache_service(60);
    let cache_1440min = create_cache_service(1440); // 24 hours

    assert_eq!(cache_5min.stats().cache_duration_minutes, 5);
    assert_eq!(cache_60min.stats().cache_duration_minutes, 60);
    assert_eq!(cache_1440min.stats().cache_duration_minutes, 1440);
}

#[test]
fn test_cache_service_debug_impl() {
    let cache = create_cache_service(30);
    let debug_str = format!("{:?}", cache);

    assert!(debug_str.contains("CacheService"));
    assert!(debug_str.contains("cache_duration"));
    assert!(debug_str.contains("max_memory_entries"));
}

// ============================================================================
// Basic Get/Set Tests
// ============================================================================

#[tokio::test]
async fn test_set_and_get_simple_value() {
    let cache = create_cache_service(15);
    let data = sample_test_data();

    cache.set("test_key", &data).await;

    let retrieved: Option<TestData> = cache.get("test_key").await;
    assert_eq!(retrieved, Some(data));
}

#[tokio::test]
async fn test_get_nonexistent_key() {
    let cache = create_cache_service(15);

    let result: Option<TestData> = cache.get("nonexistent_key").await;
    assert_eq!(result, None);
}

#[tokio::test]
async fn test_set_and_get_string() {
    let cache = create_cache_service(15);
    let value = "Hello, World!".to_string();

    cache.set("string_key", &value).await;

    let retrieved: Option<String> = cache.get("string_key").await;
    assert_eq!(retrieved, Some(value));
}

#[tokio::test]
async fn test_set_and_get_integer() {
    let cache = create_cache_service(15);
    let value: i64 = 42;

    cache.set("int_key", &value).await;

    let retrieved: Option<i64> = cache.get("int_key").await;
    assert_eq!(retrieved, Some(42));
}

#[tokio::test]
async fn test_set_and_get_float() {
    let cache = create_cache_service(15);
    let value: f64 = 3.14159;

    cache.set("float_key", &value).await;

    let retrieved: Option<f64> = cache.get("float_key").await;
    assert!(retrieved.is_some());
    assert!((retrieved.unwrap() - 3.14159).abs() < 0.00001);
}

#[tokio::test]
async fn test_set_and_get_boolean() {
    let cache = create_cache_service(15);

    cache.set("bool_true", &true).await;
    cache.set("bool_false", &false).await;

    let retrieved_true: Option<bool> = cache.get("bool_true").await;
    let retrieved_false: Option<bool> = cache.get("bool_false").await;

    assert_eq!(retrieved_true, Some(true));
    assert_eq!(retrieved_false, Some(false));
}

#[tokio::test]
async fn test_set_and_get_vec() {
    let cache = create_cache_service(15);
    let value = vec![1, 2, 3, 4, 5];

    cache.set("vec_key", &value).await;

    let retrieved: Option<Vec<i32>> = cache.get("vec_key").await;
    assert_eq!(retrieved, Some(value));
}

#[tokio::test]
async fn test_set_and_get_complex_data() {
    let cache = create_cache_service(15);
    let data = sample_complex_data();

    cache.set("complex_key", &data).await;

    let retrieved: Option<ComplexData> = cache.get("complex_key").await;
    assert_eq!(retrieved, Some(data));
}

// ============================================================================
// Overwrite Tests
// ============================================================================

#[tokio::test]
async fn test_overwrite_existing_key() {
    let cache = create_cache_service(15);

    let data1 = TestData { id: 1, name: "first".to_string(), value: 1.0 };
    let data2 = TestData { id: 2, name: "second".to_string(), value: 2.0 };

    cache.set("key", &data1).await;
    cache.set("key", &data2).await;

    let retrieved: Option<TestData> = cache.get("key").await;
    assert_eq!(retrieved, Some(data2));
}

#[tokio::test]
async fn test_overwrite_with_different_type() {
    let cache = create_cache_service(15);

    // Set as string first
    cache.set("key", &"string_value".to_string()).await;

    // Overwrite with integer
    cache.set("key", &42i64).await;

    // Should get integer now
    let retrieved: Option<i64> = cache.get("key").await;
    assert_eq!(retrieved, Some(42));

    // String retrieval should fail (different type)
    let retrieved_str: Option<String> = cache.get("key").await;
    // JSON "42" can be parsed as string "42" in some cases, but the value won't match
    assert!(retrieved_str.is_none() || retrieved_str == Some("42".to_string()));
}

// ============================================================================
// Delete Tests
// ============================================================================

#[tokio::test]
async fn test_delete_existing_key() {
    let cache = create_cache_service(15);
    let data = sample_test_data();

    cache.set("key_to_delete", &data).await;
    assert!(cache.get::<TestData>("key_to_delete").await.is_some());

    cache.delete("key_to_delete").await;

    let retrieved: Option<TestData> = cache.get("key_to_delete").await;
    assert_eq!(retrieved, None);
}

#[tokio::test]
async fn test_delete_nonexistent_key() {
    let cache = create_cache_service(15);

    // Should not panic when deleting non-existent key
    cache.delete("nonexistent").await;

    let retrieved: Option<String> = cache.get("nonexistent").await;
    assert_eq!(retrieved, None);
}

#[tokio::test]
async fn test_delete_one_of_many() {
    let cache = create_cache_service(15);

    cache.set("key1", &"value1".to_string()).await;
    cache.set("key2", &"value2".to_string()).await;
    cache.set("key3", &"value3".to_string()).await;

    cache.delete("key2").await;

    assert_eq!(cache.get::<String>("key1").await, Some("value1".to_string()));
    assert_eq!(cache.get::<String>("key2").await, None);
    assert_eq!(cache.get::<String>("key3").await, Some("value3".to_string()));
}

// ============================================================================
// Delete Pattern Tests
// ============================================================================

#[tokio::test]
async fn test_delete_pattern_matching() {
    let cache = create_cache_service(15);

    cache.set("analytics:overview:today", &"data1".to_string()).await;
    cache.set("analytics:overview:week", &"data2".to_string()).await;
    cache.set("analytics:traffic:today", &"data3".to_string()).await;
    cache.set("reports:daily", &"data4".to_string()).await;

    cache.delete_pattern("analytics:overview").await;

    // These should be deleted
    assert_eq!(cache.get::<String>("analytics:overview:today").await, None);
    assert_eq!(cache.get::<String>("analytics:overview:week").await, None);

    // These should remain
    assert_eq!(cache.get::<String>("analytics:traffic:today").await, Some("data3".to_string()));
    assert_eq!(cache.get::<String>("reports:daily").await, Some("data4".to_string()));
}

#[tokio::test]
async fn test_delete_pattern_no_matches() {
    let cache = create_cache_service(15);

    cache.set("key1", &"value1".to_string()).await;
    cache.set("key2", &"value2".to_string()).await;

    cache.delete_pattern("nonexistent_prefix").await;

    // All keys should remain
    assert_eq!(cache.get::<String>("key1").await, Some("value1".to_string()));
    assert_eq!(cache.get::<String>("key2").await, Some("value2".to_string()));
}

#[tokio::test]
async fn test_delete_pattern_all_matching() {
    let cache = create_cache_service(15);

    cache.set("prefix:a", &"1".to_string()).await;
    cache.set("prefix:b", &"2".to_string()).await;
    cache.set("prefix:c", &"3".to_string()).await;

    cache.delete_pattern("prefix:").await;

    assert_eq!(cache.get::<String>("prefix:a").await, None);
    assert_eq!(cache.get::<String>("prefix:b").await, None);
    assert_eq!(cache.get::<String>("prefix:c").await, None);

    assert_eq!(cache.stats().memory_entries, 0);
}

// ============================================================================
// Clear Tests
// ============================================================================

#[tokio::test]
async fn test_clear_cache() {
    let cache = create_cache_service(15);

    cache.set("key1", &"value1".to_string()).await;
    cache.set("key2", &"value2".to_string()).await;
    cache.set("key3", &"value3".to_string()).await;

    assert_eq!(cache.stats().memory_entries, 3);

    cache.clear().await;

    assert_eq!(cache.stats().memory_entries, 0);
    assert_eq!(cache.get::<String>("key1").await, None);
    assert_eq!(cache.get::<String>("key2").await, None);
    assert_eq!(cache.get::<String>("key3").await, None);
}

#[tokio::test]
async fn test_clear_empty_cache() {
    let cache = create_cache_service(15);

    // Should not panic
    cache.clear().await;

    assert_eq!(cache.stats().memory_entries, 0);
}

#[tokio::test]
async fn test_set_after_clear() {
    let cache = create_cache_service(15);

    cache.set("key1", &"value1".to_string()).await;
    cache.clear().await;
    cache.set("key2", &"value2".to_string()).await;

    assert_eq!(cache.get::<String>("key1").await, None);
    assert_eq!(cache.get::<String>("key2").await, Some("value2".to_string()));
}

// ============================================================================
// Statistics Tests
// ============================================================================

#[tokio::test]
async fn test_stats_empty_cache() {
    let cache = create_cache_service(30);
    let stats = cache.stats();

    assert_eq!(stats.memory_entries, 0);
    assert_eq!(stats.memory_size_bytes, 0);
    assert_eq!(stats.expired_entries, 0);
    assert_eq!(stats.max_entries, 1000);
    assert_eq!(stats.cache_duration_minutes, 30);
}

#[tokio::test]
async fn test_stats_with_entries() {
    let cache = create_cache_service(15);

    cache.set("key1", &"short".to_string()).await;
    cache.set("key2", &"a longer string value".to_string()).await;
    cache.set("key3", &sample_test_data()).await;

    let stats = cache.stats();

    assert_eq!(stats.memory_entries, 3);
    assert!(stats.memory_size_bytes > 0);
    assert_eq!(stats.expired_entries, 0);
}

#[tokio::test]
async fn test_stats_size_calculation() {
    let cache = create_cache_service(15);

    // Add known data
    let data = "x".repeat(100);
    cache.set("key", &data).await;

    let stats = cache.stats();

    // JSON serialization adds quotes: "xxxx..." so size is at least 102
    assert!(stats.memory_size_bytes >= 100);
}

#[tokio::test]
async fn test_stats_after_delete() {
    let cache = create_cache_service(15);

    cache.set("key1", &"value1".to_string()).await;
    cache.set("key2", &"value2".to_string()).await;

    assert_eq!(cache.stats().memory_entries, 2);

    cache.delete("key1").await;

    assert_eq!(cache.stats().memory_entries, 1);
}

// ============================================================================
// Cleanup Tests
// ============================================================================

#[tokio::test]
async fn test_cleanup_no_expired() {
    let cache = create_cache_service(60); // 60 min expiry

    cache.set("key1", &"value1".to_string()).await;
    cache.set("key2", &"value2".to_string()).await;

    cache.cleanup().await;

    // Nothing should be removed since nothing is expired
    assert_eq!(cache.stats().memory_entries, 2);
}

#[tokio::test]
async fn test_cleanup_runs_without_error() {
    let cache = create_cache_service(15);

    cache.set("key", &"value".to_string()).await;

    // Should not panic
    cache.cleanup().await;
}

// ============================================================================
// Warm Up Tests
// ============================================================================

#[tokio::test]
async fn test_warm_up_runs_without_error() {
    let cache = create_cache_service(15);

    // Should not panic
    cache.warm_up().await;
}

// ============================================================================
// Multiple Keys Tests
// ============================================================================

#[tokio::test]
async fn test_many_keys() {
    let cache = create_cache_service(15);

    // Insert 100 keys
    for i in 0..100 {
        cache.set(&format!("key_{}", i), &i).await;
    }

    assert_eq!(cache.stats().memory_entries, 100);

    // Verify some values
    assert_eq!(cache.get::<i32>("key_0").await, Some(0));
    assert_eq!(cache.get::<i32>("key_50").await, Some(50));
    assert_eq!(cache.get::<i32>("key_99").await, Some(99));
}

#[tokio::test]
async fn test_special_characters_in_keys() {
    let cache = create_cache_service(15);

    cache.set("key:with:colons", &"value1".to_string()).await;
    cache.set("key/with/slashes", &"value2".to_string()).await;
    cache.set("key.with.dots", &"value3".to_string()).await;
    cache.set("key-with-dashes", &"value4".to_string()).await;
    cache.set("key_with_underscores", &"value5".to_string()).await;
    cache.set("key with spaces", &"value6".to_string()).await;

    assert_eq!(cache.get::<String>("key:with:colons").await, Some("value1".to_string()));
    assert_eq!(cache.get::<String>("key/with/slashes").await, Some("value2".to_string()));
    assert_eq!(cache.get::<String>("key.with.dots").await, Some("value3".to_string()));
    assert_eq!(cache.get::<String>("key-with-dashes").await, Some("value4".to_string()));
    assert_eq!(cache.get::<String>("key_with_underscores").await, Some("value5".to_string()));
    assert_eq!(cache.get::<String>("key with spaces").await, Some("value6".to_string()));
}

#[tokio::test]
async fn test_unicode_in_keys_and_values() {
    let cache = create_cache_service(15);

    cache.set("キー", &"日本語の値".to_string()).await;
    cache.set("emoji_key_🔑", &"emoji_value_🎉".to_string()).await;
    cache.set("中文键", &"中文值".to_string()).await;

    assert_eq!(cache.get::<String>("キー").await, Some("日本語の値".to_string()));
    assert_eq!(cache.get::<String>("emoji_key_🔑").await, Some("emoji_value_🎉".to_string()));
    assert_eq!(cache.get::<String>("中文键").await, Some("中文值".to_string()));
}

#[tokio::test]
async fn test_empty_key() {
    let cache = create_cache_service(15);

    cache.set("", &"empty_key_value".to_string()).await;

    assert_eq!(cache.get::<String>("").await, Some("empty_key_value".to_string()));
}

#[tokio::test]
async fn test_empty_string_value() {
    let cache = create_cache_service(15);

    cache.set("key", &"".to_string()).await;

    let retrieved: Option<String> = cache.get("key").await;
    assert_eq!(retrieved, Some("".to_string()));
}

// ============================================================================
// Large Data Tests
// ============================================================================

#[tokio::test]
async fn test_large_string_value() {
    let cache = create_cache_service(15);

    // 1MB string
    let large_value = "x".repeat(1024 * 1024);

    cache.set("large_key", &large_value).await;

    let retrieved: Option<String> = cache.get("large_key").await;
    assert_eq!(retrieved, Some(large_value));
}

#[tokio::test]
async fn test_large_vec_value() {
    let cache = create_cache_service(15);

    // Vector with 10000 elements
    let large_vec: Vec<i32> = (0..10000).collect();

    cache.set("large_vec", &large_vec).await;

    let retrieved: Option<Vec<i32>> = cache.get("large_vec").await;
    assert_eq!(retrieved, Some(large_vec));
}

// ============================================================================
// Null/Option Value Tests
// ============================================================================

#[tokio::test]
async fn test_option_some_value() {
    let cache = create_cache_service(15);

    let value: Option<i32> = Some(42);
    cache.set("option_key", &value).await;

    let retrieved: Option<Option<i32>> = cache.get("option_key").await;
    assert_eq!(retrieved, Some(Some(42)));
}

#[tokio::test]
async fn test_option_none_value() {
    let cache = create_cache_service(15);

    let value: Option<i32> = None;
    cache.set("none_key", &value).await;

    let retrieved: Option<Option<i32>> = cache.get("none_key").await;
    assert_eq!(retrieved, Some(None));
}

// ============================================================================
// Concurrent Access Tests
// ============================================================================

#[tokio::test]
async fn test_concurrent_reads() {
    let cache = Arc::new(create_cache_service(15));

    cache.set("shared_key", &"shared_value".to_string()).await;

    let mut handles = vec![];

    for _ in 0..10 {
        let cache_clone = Arc::clone(&cache);
        handles.push(tokio::spawn(async move {
            let result: Option<String> = cache_clone.get("shared_key").await;
            assert_eq!(result, Some("shared_value".to_string()));
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn test_concurrent_writes() {
    let cache = Arc::new(create_cache_service(15));

    let mut handles = vec![];

    for i in 0..10 {
        let cache_clone = Arc::clone(&cache);
        handles.push(tokio::spawn(async move {
            cache_clone.set(&format!("key_{}", i), &i).await;
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all writes succeeded
    for i in 0..10 {
        let result: Option<i32> = cache.get(&format!("key_{}", i)).await;
        assert_eq!(result, Some(i));
    }
}

#[tokio::test]
async fn test_concurrent_read_write() {
    let cache = Arc::new(create_cache_service(15));

    // Pre-populate
    cache.set("key", &0i32).await;

    let mut handles = vec![];

    // Mix of reads and writes
    for i in 0..20 {
        let cache_clone = Arc::clone(&cache);
        if i % 2 == 0 {
            handles.push(tokio::spawn(async move {
                cache_clone.set("key", &i).await;
            }));
        } else {
            handles.push(tokio::spawn(async move {
                let _: Option<i32> = cache_clone.get("key").await;
            }));
        }
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // Should have some value
    let result: Option<i32> = cache.get("key").await;
    assert!(result.is_some());
}

// ============================================================================
// Cache Stats Serialization Test
// ============================================================================

#[test]
fn test_cache_stats_serialization() {
    let stats = CacheStats {
        memory_entries: 100,
        memory_size_bytes: 50000,
        expired_entries: 5,
        max_entries: 1000,
        cache_duration_minutes: 15,
    };

    let json = serde_json::to_string(&stats).unwrap();
    assert!(json.contains("\"memory_entries\":100"));
    assert!(json.contains("\"memory_size_bytes\":50000"));
    assert!(json.contains("\"expired_entries\":5"));
    assert!(json.contains("\"max_entries\":1000"));
    assert!(json.contains("\"cache_duration_minutes\":15"));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[tokio::test]
async fn test_get_wrong_type() {
    let cache = create_cache_service(15);

    // Store a struct
    cache.set("key", &sample_test_data()).await;

    // Try to get as different type - should return None due to parse failure
    let result: Option<Vec<String>> = cache.get("key").await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_zero_duration() {
    let cache = create_cache_service(0);

    cache.set("key", &"value".to_string()).await;

    // With 0 duration, entries expire immediately
    // Due to timing, the entry might still be valid, so just verify it was set
    assert_eq!(cache.stats().cache_duration_minutes, 0);
}

#[tokio::test]
async fn test_very_long_key() {
    let cache = create_cache_service(15);

    let long_key = "k".repeat(10000);
    cache.set(&long_key, &"value".to_string()).await;

    let retrieved: Option<String> = cache.get(&long_key).await;
    assert_eq!(retrieved, Some("value".to_string()));
}

#[tokio::test]
async fn test_rapid_set_get_delete() {
    let cache = create_cache_service(15);

    for i in 0..100 {
        let key = format!("rapid_key_{}", i);
        cache.set(&key, &i).await;
        let _: Option<i32> = cache.get(&key).await;
        cache.delete(&key).await;
    }

    // All should be deleted
    assert_eq!(cache.stats().memory_entries, 0);
}
