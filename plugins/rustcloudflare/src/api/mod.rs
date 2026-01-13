//! API handlers for RustCloudflare

pub mod cache;
pub mod dns;
pub mod security;
pub mod workers;
pub mod analytics;
pub mod settings;
pub mod oauth;
pub mod r2;
pub mod ssl;
pub mod stream;
pub mod rules;
pub mod d1;

use axum::{
    routing::{get, post, put, delete, patch},
    Router,
};
use std::sync::Arc;
use crate::services::CloudflareServices;

/// Create the API router with all routes
/// This returns a Router that can be nested under /api/plugins/rustcloudflare
pub fn create_router(services: Arc<CloudflareServices>) -> Router {
    Router::new()
        // Status & Connection
        .route("/status", get(get_status))
        .route("/connection", get(oauth::get_connection_status))

        // OAuth / Authentication routes
        .route("/auth/url", get(oauth::get_auth_url))
        .route("/auth/callback", get(oauth::oauth_callback))
        .route("/auth/sso-complete", post(oauth::sso_complete))
        .route("/auth/verify-token", post(oauth::verify_token))
        .route("/auth/save-credentials", post(oauth::save_credentials))
        .route("/auth/disconnect", post(oauth::disconnect))
        .route("/auth/accounts", post(oauth::list_accounts))
        .route("/auth/zones", post(oauth::list_zones))

        // Cache routes
        .route("/cache/purge", post(cache::purge_cache))
        .route("/cache/purge/all", post(cache::purge_all))
        .route("/cache/purge/tags", post(cache::purge_by_tags))
        .route("/cache/purge/prefix", post(cache::purge_by_prefix))
        .route("/cache/status", get(cache::get_cache_status))

        // DNS routes
        .route("/dns/records", get(dns::list_records))
        .route("/dns/records", post(dns::create_record))
        .route("/dns/records/:id", get(dns::get_record))
        .route("/dns/records/:id", put(dns::update_record))
        .route("/dns/records/:id", delete(dns::delete_record))
        .route("/dns/export", get(dns::export_zone))
        .route("/dns/sync", post(dns::sync_records))

        // SSL/TLS routes
        .route("/ssl/settings", get(ssl::get_ssl_settings))
        .route("/ssl/mode", put(ssl::update_ssl_mode))
        .route("/ssl/certificates", get(ssl::list_certificates))

        // Security routes
        .route("/security/level", get(security::get_security_level))
        .route("/security/level", put(security::set_security_level))
        .route("/security/under-attack", post(security::toggle_under_attack))
        .route("/security/waf/rules", get(security::list_waf_rules))
        .route("/security/firewall/rules", get(security::list_firewall_rules))
        .route("/security/firewall/rules", post(security::create_firewall_rule))
        .route("/security/firewall/rules/:id", delete(security::delete_firewall_rule))
        .route("/security/ip-access/rules", get(security::list_ip_access_rules))
        .route("/security/ip-access/block", post(security::block_ip))
        .route("/security/ip-access/allow", post(security::allow_ip))
        .route("/security/ip-access/rules/:id", delete(security::delete_ip_access_rule))

        // Page Rules routes
        .route("/rules/pages", get(rules::list_page_rules))
        .route("/rules/pages", post(rules::create_page_rule))
        .route("/rules/pages/:id", put(rules::update_page_rule))
        .route("/rules/pages/:id", delete(rules::delete_page_rule))

        // Workers routes
        .route("/workers", get(workers::list_workers))
        .route("/workers", post(workers::deploy_worker))
        .route("/workers/:name", get(workers::get_worker))
        .route("/workers/:name", delete(workers::delete_worker))
        .route("/workers/routes", get(workers::list_routes))
        .route("/workers/routes", post(workers::create_route))
        .route("/workers/routes/:id", delete(workers::delete_route))
        .route("/workers/kv/namespaces", get(workers::list_kv_namespaces))
        .route("/workers/kv/namespaces", post(workers::create_kv_namespace))
        .route("/workers/kv/namespaces/:id", delete(workers::delete_kv_namespace))
        .route("/workers/kv/:namespace/keys", get(workers::list_kv_keys))
        .route("/workers/kv/:namespace/values/:key", get(workers::get_kv_value))
        .route("/workers/kv/:namespace/values/:key", put(workers::set_kv_value))
        .route("/workers/kv/:namespace/values/:key", delete(workers::delete_kv_value))

        // R2 Storage routes
        .route("/r2/buckets", get(r2::list_buckets))
        .route("/r2/buckets", post(r2::create_bucket))
        .route("/r2/buckets/:name", delete(r2::delete_bucket))
        .route("/r2/buckets/:name/objects", get(r2::list_objects))
        .route("/r2/buckets/:name/objects", post(r2::upload_object))
        .route("/r2/buckets/:name/objects/*key", get(r2::get_object))
        .route("/r2/buckets/:name/objects/*key", delete(r2::delete_object))

        // Stream routes
        .route("/stream/videos", get(stream::list_videos))
        .route("/stream/videos/search", get(stream::search_videos))
        .route("/stream/videos/:id", get(stream::get_video))
        .route("/stream/videos/:id", delete(stream::delete_video))
        .route("/stream/videos/:id/urls", get(stream::get_video_urls))
        .route("/stream/videos/:id/embed", get(stream::get_embed_code))
        .route("/stream/stats", get(stream::get_stats))
        .route("/stream/live-inputs", get(stream::list_live_inputs))
        .route("/stream/live-inputs", post(stream::create_live_input))
        .route("/stream/live-inputs/:id", delete(stream::delete_live_input))
        .route("/stream/live-inputs/:id/urls", get(stream::get_live_input_urls))

        // D1 Database routes
        .route("/d1/databases", get(d1::list_databases))
        .route("/d1/databases", post(d1::create_database))
        .route("/d1/databases/:id", get(d1::get_database))
        .route("/d1/databases/:id/query", post(d1::execute_query))
        .route("/d1/databases/:id/batch", post(d1::execute_batch))
        .route("/d1/databases/:id/tables", get(d1::list_tables))
        .route("/d1/databases/:id/tables/:table/schema", get(d1::get_table_schema))

        // Analytics routes
        .route("/analytics", get(analytics::get_analytics))
        .route("/analytics/traffic", get(analytics::get_traffic_summary))
        .route("/analytics/security", get(analytics::get_security_summary))

        // Settings routes
        .route("/settings", get(settings::get_settings))
        .route("/settings", put(settings::update_settings))
        .route("/zone", get(settings::get_zone_info))
        .route("/zone/settings", get(settings::get_zone_settings))
        .route("/zone/settings", patch(settings::update_zone_settings))
        .route("/zone/development-mode", post(settings::toggle_dev_mode))

        // Add state to all routes
        .with_state(services)
}

/// Get Cloudflare status
async fn get_status() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "success": true,
        "data": {
            "connected": true,
            "plugin_version": crate::VERSION,
        }
    }))
}
