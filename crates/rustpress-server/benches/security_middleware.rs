//! Performance benchmarks for security middleware.
//!
//! Run with: cargo bench --package rustpress-server

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rustpress_server::security::{
    BotDetectionConfig, BotDetectionMiddleware,
    ContentSecurityConfig, ContentSecurityMiddleware,
    FingerprintConfig, FingerprintMiddleware, RequestFingerprint,
    SecurityConfig, SecurityMiddleware,
};
use axum::{body::Body, http::Request};
use serde_json::json;

// ============================================================================
// Request Validation Benchmarks
// ============================================================================

fn bench_request_validation(c: &mut Criterion) {
    let config = SecurityConfig::default();
    let middleware = SecurityMiddleware::new(config);

    let mut group = c.benchmark_group("request_validation");
    group.throughput(Throughput::Elements(1));

    // Benchmark clean input validation
    group.bench_function("clean_input", |b| {
        b.iter(|| {
            middleware.validate(black_box("Hello, this is a normal user input"), "test")
        })
    });

    // Benchmark SQL injection detection
    group.bench_function("sql_injection_detection", |b| {
        b.iter(|| {
            middleware.validate(black_box("1' OR '1'='1' --"), "test")
        })
    });

    // Benchmark XSS detection
    group.bench_function("xss_detection", |b| {
        b.iter(|| {
            middleware.validate(black_box("<script>alert('xss')</script>"), "test")
        })
    });

    // Benchmark path traversal detection
    group.bench_function("path_traversal_detection", |b| {
        b.iter(|| {
            middleware.validate(black_box("../../../etc/passwd"), "test")
        })
    });

    // Benchmark command injection detection
    group.bench_function("command_injection_detection", |b| {
        b.iter(|| {
            middleware.validate(black_box("; rm -rf /"), "test")
        })
    });

    // Benchmark URL-encoded input
    group.bench_function("url_encoded_input", |b| {
        b.iter(|| {
            middleware.validate(black_box("%3Cscript%3Ealert%28%27xss%27%29%3C%2Fscript%3E"), "test")
        })
    });

    // Benchmark long input
    let long_input = "a".repeat(10000);
    group.bench_function("long_input_10k", |b| {
        b.iter(|| {
            middleware.validate(black_box(&long_input), "test")
        })
    });

    group.finish();
}

fn bench_validation_throughput(c: &mut Criterion) {
    let config = SecurityConfig::default();
    let middleware = SecurityMiddleware::new(config);

    let inputs = vec![
        "normal user input",
        "SELECT * FROM users WHERE id=1",
        "<b>Bold text</b>",
        "path/to/file.txt",
        "Hello, World!",
        "user@example.com",
        "https://example.com/page?q=search",
        r#"{"name": "John", "age": 30}"#,
    ];

    let mut group = c.benchmark_group("validation_throughput");
    group.throughput(Throughput::Elements(inputs.len() as u64));

    group.bench_function("batch_validation", |b| {
        b.iter(|| {
            for input in &inputs {
                middleware.validate(black_box(input), "test");
            }
        })
    });

    group.finish();
}

// ============================================================================
// Bot Detection Benchmarks
// ============================================================================

fn bench_bot_detection(c: &mut Criterion) {
    let config = BotDetectionConfig::default();
    let detector = BotDetectionMiddleware::new(config);

    let mut group = c.benchmark_group("bot_detection");
    group.throughput(Throughput::Elements(1));

    // Normal browser request
    let normal_request = Request::builder()
        .uri("/test")
        .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .header("accept", "text/html,application/xhtml+xml,application/xml")
        .header("accept-language", "en-US,en;q=0.9")
        .header("accept-encoding", "gzip, deflate, br")
        .body(Body::empty())
        .unwrap();

    group.bench_function("normal_browser", |b| {
        b.iter(|| {
            detector.analyze(black_box(&normal_request), "192.168.1.1")
        })
    });

    // Bot-like request (missing headers)
    let bot_request = Request::builder()
        .uri("/test")
        .body(Body::empty())
        .unwrap();

    group.bench_function("bot_missing_headers", |b| {
        b.iter(|| {
            detector.analyze(black_box(&bot_request), "192.168.1.1")
        })
    });

    // Suspicious user-agent
    let suspicious_request = Request::builder()
        .uri("/test")
        .header("user-agent", "python-requests/2.28.0")
        .body(Body::empty())
        .unwrap();

    group.bench_function("suspicious_user_agent", |b| {
        b.iter(|| {
            detector.analyze(black_box(&suspicious_request), "192.168.1.1")
        })
    });

    // Search engine bot (allowed)
    let googlebot_request = Request::builder()
        .uri("/test")
        .header("user-agent", "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)")
        .header("accept", "*/*")
        .body(Body::empty())
        .unwrap();

    group.bench_function("allowed_bot_googlebot", |b| {
        b.iter(|| {
            detector.analyze(black_box(&googlebot_request), "192.168.1.1")
        })
    });

    group.finish();
}

fn bench_bot_detection_with_timing(c: &mut Criterion) {
    let config = BotDetectionConfig {
        enable_timing_analysis: true,
        min_request_interval_ms: 50,
        max_requests_per_minute: 120,
        ..Default::default()
    };
    let detector = BotDetectionMiddleware::new(config);

    let request = Request::builder()
        .uri("/test")
        .header("user-agent", "Mozilla/5.0")
        .header("accept", "*/*")
        .body(Body::empty())
        .unwrap();

    let mut group = c.benchmark_group("bot_detection_timing");
    group.throughput(Throughput::Elements(1));

    // Simulate rapid requests from same IP
    group.bench_function("rapid_requests_same_ip", |b| {
        b.iter(|| {
            detector.analyze(black_box(&request), "192.168.1.100")
        })
    });

    // Different IPs (no timing penalty)
    let mut counter = 0u32;
    group.bench_function("different_ips", |b| {
        b.iter(|| {
            counter = counter.wrapping_add(1);
            let ip = format!("192.168.{}.{}", (counter >> 8) & 255, counter & 255);
            detector.analyze(black_box(&request), &ip)
        })
    });

    group.finish();
}

// ============================================================================
// Content Security Benchmarks
// ============================================================================

fn bench_content_security(c: &mut Criterion) {
    let config = ContentSecurityConfig::default();
    let middleware = ContentSecurityMiddleware::new(config);

    let mut group = c.benchmark_group("content_security");

    // JSON depth validation
    let shallow_json = json!({"a": {"b": "c"}});
    group.bench_function("json_depth_shallow", |b| {
        b.iter(|| {
            middleware.validate_json_depth(black_box(&shallow_json))
        })
    });

    // Deep nested JSON
    let deep_json = create_deep_json(20);
    group.bench_function("json_depth_deep_20", |b| {
        b.iter(|| {
            middleware.validate_json_depth(black_box(&deep_json))
        })
    });

    // JSON key count validation
    let few_keys = json!({"a": 1, "b": 2, "c": 3, "d": 4, "e": 5});
    group.bench_function("json_keys_5", |b| {
        b.iter(|| {
            middleware.validate_json_keys(black_box(&few_keys))
        })
    });

    let many_keys = create_wide_json(100);
    group.bench_function("json_keys_100", |b| {
        b.iter(|| {
            middleware.validate_json_keys(black_box(&many_keys))
        })
    });

    // JSON string length validation
    let normal_strings = json!({
        "name": "John Doe",
        "email": "john@example.com",
        "bio": "A short bio about the user."
    });
    group.bench_function("json_strings_normal", |b| {
        b.iter(|| {
            middleware.validate_json_strings(black_box(&normal_strings))
        })
    });

    // Magic bytes validation
    let jpeg_bytes = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46];
    group.bench_function("magic_bytes_jpeg", |b| {
        b.iter(|| {
            middleware.validate_magic_bytes(black_box(&jpeg_bytes), "image/jpeg")
        })
    });

    let png_bytes = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    group.bench_function("magic_bytes_png", |b| {
        b.iter(|| {
            middleware.validate_magic_bytes(black_box(&png_bytes), "image/png")
        })
    });

    // Content-Type validation
    group.bench_function("content_type_valid", |b| {
        b.iter(|| {
            middleware.validate_content_type(black_box(Some("application/json")))
        })
    });

    group.bench_function("content_type_invalid", |b| {
        b.iter(|| {
            middleware.validate_content_type(black_box(Some("text/html")))
        })
    });

    // Body limit lookup
    group.bench_function("body_limit_lookup", |b| {
        b.iter(|| {
            middleware.get_body_limit(black_box("/api/users/123"))
        })
    });

    group.finish();
}

fn bench_json_validation_sizes(c: &mut Criterion) {
    let config = ContentSecurityConfig::default();
    let middleware = ContentSecurityMiddleware::new(config);

    let mut group = c.benchmark_group("json_validation_sizes");

    for depth in [5, 10, 15, 20, 25, 30] {
        let json = create_deep_json(depth);
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::new("depth", depth), &json, |b, json| {
            b.iter(|| middleware.validate_json_depth(black_box(json)))
        });
    }

    for keys in [10, 50, 100, 250, 500, 750, 1000] {
        let json = create_wide_json(keys);
        group.bench_with_input(BenchmarkId::new("keys", keys), &json, |b, json| {
            b.iter(|| middleware.validate_json_keys(black_box(json)))
        });
    }

    group.finish();
}

// ============================================================================
// Fingerprint Benchmarks
// ============================================================================

fn bench_fingerprint(c: &mut Criterion) {
    let config = FingerprintConfig::default();
    let middleware = FingerprintMiddleware::new(config);

    let mut group = c.benchmark_group("fingerprint");
    group.throughput(Throughput::Elements(1));

    // Create fingerprint from request
    let request = Request::builder()
        .uri("/api/users")
        .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .header("accept", "text/html,application/xhtml+xml,application/xml")
        .header("accept-language", "en-US,en;q=0.9")
        .header("accept-encoding", "gzip, deflate, br")
        .header("connection", "keep-alive")
        .header("cache-control", "max-age=0")
        .body(Body::empty())
        .unwrap();

    group.bench_function("create_fingerprint", |b| {
        b.iter(|| {
            RequestFingerprint::from_request(black_box(&request))
        })
    });

    // Process request (fingerprint + profile update)
    group.bench_function("process_request", |b| {
        b.iter(|| {
            middleware.process(black_box(&request), "192.168.1.1")
        })
    });

    // Fingerprint similarity comparison
    let fp1 = RequestFingerprint::from_request(&request);
    let fp2 = RequestFingerprint::from_request(&request);
    group.bench_function("fingerprint_similarity", |b| {
        b.iter(|| {
            fp1.similarity(black_box(&fp2))
        })
    });

    // Get profile
    middleware.process(&request, "192.168.1.50");
    group.bench_function("get_profile", |b| {
        b.iter(|| {
            middleware.get_profile(black_box("192.168.1.50"))
        })
    });

    // Get stats
    group.bench_function("get_stats", |b| {
        b.iter(|| {
            middleware.get_stats()
        })
    });

    group.finish();
}

fn bench_fingerprint_scaling(c: &mut Criterion) {
    let config = FingerprintConfig {
        max_profiles: 100000,
        ..Default::default()
    };
    let middleware = FingerprintMiddleware::new(config);

    let request = Request::builder()
        .uri("/test")
        .header("user-agent", "Mozilla/5.0")
        .header("accept", "*/*")
        .body(Body::empty())
        .unwrap();

    // Pre-populate with profiles
    for i in 0..1000 {
        let ip = format!("10.0.{}.{}", i / 256, i % 256);
        middleware.process(&request, &ip);
    }

    let mut group = c.benchmark_group("fingerprint_scaling");
    group.throughput(Throughput::Elements(1));

    // New profile creation
    let mut counter = 2000u32;
    group.bench_function("new_profile_with_1k_existing", |b| {
        b.iter(|| {
            counter = counter.wrapping_add(1);
            let ip = format!("172.16.{}.{}", (counter >> 8) & 255, counter & 255);
            middleware.process(black_box(&request), &ip)
        })
    });

    // Existing profile update
    group.bench_function("existing_profile_update", |b| {
        b.iter(|| {
            middleware.process(black_box(&request), "10.0.1.100")
        })
    });

    // Get suspicious profiles (scan all)
    group.bench_function("get_suspicious_profiles", |b| {
        b.iter(|| {
            middleware.get_suspicious_profiles()
        })
    });

    group.finish();
}

// ============================================================================
// Combined Middleware Stack Benchmarks
// ============================================================================

fn bench_full_stack(c: &mut Criterion) {
    let security = SecurityMiddleware::new(SecurityConfig::default());
    let content = ContentSecurityMiddleware::new(ContentSecurityConfig::default());
    let bot = BotDetectionMiddleware::new(BotDetectionConfig::default());
    let fingerprint = FingerprintMiddleware::new(FingerprintConfig::default());

    let request = Request::builder()
        .uri("/api/posts/123?q=search&page=1")
        .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .header("accept", "application/json")
        .header("accept-language", "en-US,en;q=0.9")
        .header("accept-encoding", "gzip, deflate, br")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let path = "/api/posts/123";
    let query = "q=search&page=1";

    let mut group = c.benchmark_group("full_stack");
    group.throughput(Throughput::Elements(1));

    // Simulate full middleware chain
    group.bench_function("all_security_checks", |b| {
        b.iter(|| {
            // Request validation
            let _ = security.validate(black_box(path), "path");
            let _ = security.validate(black_box(query), "query");

            // Content security
            let _ = content.validate_content_type(Some("application/json"));
            let _ = content.get_body_limit(path);

            // Bot detection
            let _ = bot.analyze(black_box(&request), "192.168.1.1");

            // Fingerprinting
            let _ = fingerprint.process(black_box(&request), "192.168.1.1");
        })
    });

    group.finish();
}

// ============================================================================
// Helper Functions
// ============================================================================

fn create_deep_json(depth: usize) -> serde_json::Value {
    let mut value = json!("leaf");
    for i in 0..depth {
        value = json!({ format!("level_{}", i): value });
    }
    value
}

fn create_wide_json(keys: usize) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for i in 0..keys {
        map.insert(format!("key_{}", i), json!(i));
    }
    serde_json::Value::Object(map)
}

// ============================================================================
// Criterion Configuration
// ============================================================================

criterion_group!(
    benches,
    bench_request_validation,
    bench_validation_throughput,
    bench_bot_detection,
    bench_bot_detection_with_timing,
    bench_content_security,
    bench_json_validation_sizes,
    bench_fingerprint,
    bench_fingerprint_scaling,
    bench_full_stack,
);

criterion_main!(benches);
