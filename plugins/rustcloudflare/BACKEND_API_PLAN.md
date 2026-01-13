# RustCloudflare Plugin - Backend & API Integration Plan

## Overview

This document outlines the complete backend implementation required to make the RustCloudflare plugin fully functional. The plugin integrates with the Cloudflare API to provide CDN management, security features, DNS controls, and more directly from the RustPress admin panel.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        RustPress Admin UI                       │
│                    (React + TypeScript + Vite)                  │
└─────────────────────────────┬───────────────────────────────────┘
                              │ HTTP/REST
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    RustCloudflare Backend                       │
│                        (Rust + Actix-web)                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │   Handlers  │  │   Services  │  │   Cloudflare Client     │  │
│  │   (API)     │──│   (Logic)   │──│   (External API calls)  │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
└─────────────────────────────┬───────────────────────────────────┘
                              │ HTTPS
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Cloudflare API                              │
│              https://api.cloudflare.com/client/v4               │
└─────────────────────────────────────────────────────────────────┘
```

---

## 1. Database Schema

### Plugin Settings Table

```sql
CREATE TABLE cloudflare_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    site_id UUID REFERENCES sites(id) ON DELETE CASCADE,
    api_token TEXT ENCRYPTED,           -- Encrypted Cloudflare API token
    account_id VARCHAR(64),             -- Cloudflare Account ID
    zone_id VARCHAR(64),                -- Cloudflare Zone ID

    -- Auto-purge settings
    auto_purge_enabled BOOLEAN DEFAULT true,
    auto_purge_on_post_update BOOLEAN DEFAULT true,
    auto_purge_on_media_upload BOOLEAN DEFAULT true,
    auto_purge_on_theme_change BOOLEAN DEFAULT true,

    -- Cache warming
    cache_warming_enabled BOOLEAN DEFAULT false,
    cache_warming_schedule VARCHAR(32) DEFAULT 'immediate',

    -- Development mode
    development_mode_duration INTEGER DEFAULT 180,

    -- Notifications
    security_email_alerts BOOLEAN DEFAULT false,
    security_slack_webhook TEXT,

    -- R2 & Workers
    r2_default_bucket VARCHAR(128),
    workers_enabled BOOLEAN DEFAULT true,

    -- Analytics
    analytics_retention_days INTEGER DEFAULT 30,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_cloudflare_settings_site ON cloudflare_settings(site_id);
```

### Cache Purge Log Table

```sql
CREATE TABLE cloudflare_purge_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    site_id UUID REFERENCES sites(id) ON DELETE CASCADE,
    purge_type VARCHAR(32) NOT NULL,    -- 'all', 'urls', 'tags', 'prefix'
    targets JSONB,                       -- URLs, tags, or prefixes purged
    triggered_by VARCHAR(64),            -- 'user', 'post_update', 'media_upload', 'theme_change'
    user_id UUID REFERENCES users(id),
    success BOOLEAN,
    error_message TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_cloudflare_purge_log_site ON cloudflare_purge_log(site_id);
CREATE INDEX idx_cloudflare_purge_log_created ON cloudflare_purge_log(created_at);
```

### Analytics Cache Table

```sql
CREATE TABLE cloudflare_analytics_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    site_id UUID REFERENCES sites(id) ON DELETE CASCADE,
    time_range VARCHAR(16) NOT NULL,    -- '1h', '6h', '24h', '7d', '30d'
    data JSONB NOT NULL,
    fetched_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_cloudflare_analytics_site ON cloudflare_analytics_cache(site_id);
CREATE INDEX idx_cloudflare_analytics_expires ON cloudflare_analytics_cache(expires_at);
```

---

## 2. Rust Backend Structure

### Directory Structure

```
src/
├── api/
│   ├── mod.rs
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── status.rs           # Connection status endpoints
│   │   ├── cache.rs            # Cache purge endpoints
│   │   ├── dns.rs              # DNS record management
│   │   ├── security.rs         # Security settings & firewall
│   │   ├── ssl.rs              # SSL/TLS management
│   │   ├── workers.rs          # Workers deployment
│   │   ├── r2.rs               # R2 storage management
│   │   ├── analytics.rs        # Traffic analytics
│   │   ├── page_rules.rs       # Page rules management
│   │   ├── zone.rs             # Zone settings
│   │   └── settings.rs         # Plugin settings
│   └── routes.rs
├── services/
│   ├── mod.rs
│   ├── cloudflare_client.rs    # Cloudflare API client
│   ├── cache_service.rs        # Cache management logic
│   ├── dns_service.rs          # DNS record logic
│   ├── security_service.rs     # Security logic
│   ├── ssl_service.rs          # SSL/TLS logic
│   ├── workers_service.rs      # Workers logic
│   ├── r2_service.rs           # R2 storage logic
│   ├── analytics_service.rs    # Analytics logic
│   └── settings_service.rs     # Plugin settings logic
├── models/
│   ├── mod.rs
│   ├── settings.rs             # Plugin settings model
│   ├── dns.rs                  # DNS record models
│   ├── security.rs             # Security models
│   ├── workers.rs              # Workers models
│   ├── r2.rs                   # R2 models
│   └── analytics.rs            # Analytics models
├── middleware/
│   ├── mod.rs
│   └── cloudflare_auth.rs      # Auth middleware
└── workers/
    ├── mod.rs
    ├── auto_purge.rs           # Background auto-purge worker
    └── cache_warmer.rs         # Cache warming worker
```

---

## 3. API Endpoints

### Status & Connection

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/cloudflare/status` | Get connection status |
| GET | `/api/v1/cloudflare/zone` | Get zone details |
| POST | `/api/v1/cloudflare/connection/test` | Test API credentials |

### Cache Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/cloudflare/cache/purge/all` | Purge entire cache |
| POST | `/api/v1/cloudflare/cache/purge` | Purge by URLs |
| POST | `/api/v1/cloudflare/cache/purge/tags` | Purge by cache tags (Enterprise) |
| POST | `/api/v1/cloudflare/cache/purge/prefix` | Purge by URL prefix (Enterprise) |
| GET | `/api/v1/cloudflare/cache/status` | Get cache statistics |
| POST | `/api/v1/cloudflare/cache/warm` | Warm specific URLs |
| POST | `/api/v1/cloudflare/cache/local/clear` | Clear local analytics cache |

### DNS Records

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/cloudflare/dns/records` | List all DNS records |
| POST | `/api/v1/cloudflare/dns/records` | Create DNS record |
| PUT | `/api/v1/cloudflare/dns/records/:id` | Update DNS record |
| DELETE | `/api/v1/cloudflare/dns/records/:id` | Delete DNS record |
| GET | `/api/v1/cloudflare/dns/export` | Export zone file |
| POST | `/api/v1/cloudflare/dns/import` | Import zone file |
| POST | `/api/v1/cloudflare/dns/sync` | Sync DNS records |

### Security

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/cloudflare/security/level` | Get security level |
| PUT | `/api/v1/cloudflare/security/level` | Set security level |
| POST | `/api/v1/cloudflare/security/under-attack` | Toggle Under Attack mode |
| GET | `/api/v1/cloudflare/security/firewall/rules` | List firewall rules |
| POST | `/api/v1/cloudflare/security/firewall/rules` | Create firewall rule |
| PUT | `/api/v1/cloudflare/security/firewall/rules/:id` | Update firewall rule |
| DELETE | `/api/v1/cloudflare/security/firewall/rules/:id` | Delete firewall rule |
| GET | `/api/v1/cloudflare/security/ip-access/rules` | List IP access rules |
| POST | `/api/v1/cloudflare/security/ip-access/block` | Block IP |
| POST | `/api/v1/cloudflare/security/ip-access/allow` | Allow IP |
| POST | `/api/v1/cloudflare/security/ip-access/challenge` | Challenge IP |
| DELETE | `/api/v1/cloudflare/security/ip-access/rules/:id` | Delete IP rule |
| GET | `/api/v1/cloudflare/security/events` | Get security events |
| GET | `/api/v1/cloudflare/security/waf/rules` | Get WAF rules |
| PATCH | `/api/v1/cloudflare/security/waf/rules/:id` | Toggle WAF rule |

### SSL/TLS

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/cloudflare/ssl/status` | Get SSL status |
| PUT | `/api/v1/cloudflare/ssl/mode` | Set SSL mode (off, flexible, full, strict) |
| GET | `/api/v1/cloudflare/ssl/certificates` | List certificates |
| POST | `/api/v1/cloudflare/ssl/certificates` | Order new certificate |
| POST | `/api/v1/cloudflare/ssl/certificates/custom` | Upload custom certificate |

### Workers

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/cloudflare/workers` | List workers |
| GET | `/api/v1/cloudflare/workers/:name` | Get worker details |
| POST | `/api/v1/cloudflare/workers` | Deploy new worker |
| PUT | `/api/v1/cloudflare/workers/:name` | Update worker script |
| DELETE | `/api/v1/cloudflare/workers/:name` | Delete worker |
| GET | `/api/v1/cloudflare/workers/routes` | List worker routes |
| POST | `/api/v1/cloudflare/workers/routes` | Create route |
| DELETE | `/api/v1/cloudflare/workers/routes/:id` | Delete route |
| POST | `/api/v1/cloudflare/workers/templates/deploy` | Deploy from template |

### KV Storage

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/cloudflare/workers/kv/namespaces` | List namespaces |
| POST | `/api/v1/cloudflare/workers/kv/namespaces` | Create namespace |
| DELETE | `/api/v1/cloudflare/workers/kv/namespaces/:id` | Delete namespace |
| GET | `/api/v1/cloudflare/workers/kv/namespaces/:id/keys` | List keys |
| GET | `/api/v1/cloudflare/workers/kv/namespaces/:id/values/:key` | Get value |
| PUT | `/api/v1/cloudflare/workers/kv/namespaces/:id/values/:key` | Put value |
| DELETE | `/api/v1/cloudflare/workers/kv/namespaces/:id/values/:key` | Delete value |

### R2 Storage

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/cloudflare/r2/buckets` | List buckets |
| POST | `/api/v1/cloudflare/r2/buckets` | Create bucket |
| DELETE | `/api/v1/cloudflare/r2/buckets/:name` | Delete bucket |
| GET | `/api/v1/cloudflare/r2/buckets/:name/objects` | List objects |
| POST | `/api/v1/cloudflare/r2/buckets/:name/objects` | Upload object |
| DELETE | `/api/v1/cloudflare/r2/buckets/:name/objects/:key` | Delete object |
| GET | `/api/v1/cloudflare/r2/buckets/:name/objects/:key/url` | Get signed URL |

### Analytics

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/cloudflare/analytics` | Get analytics (with time_range param) |
| GET | `/api/v1/cloudflare/analytics/zone` | Get zone analytics |
| GET | `/api/v1/cloudflare/analytics/traffic` | Get traffic summary |
| GET | `/api/v1/cloudflare/analytics/bandwidth` | Get bandwidth stats |
| GET | `/api/v1/cloudflare/analytics/security` | Get security analytics |
| GET | `/api/v1/cloudflare/analytics/cache` | Get cache analytics |

### Page Rules

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/cloudflare/page-rules` | List page rules |
| POST | `/api/v1/cloudflare/page-rules` | Create page rule |
| PUT | `/api/v1/cloudflare/page-rules/:id` | Update page rule |
| DELETE | `/api/v1/cloudflare/page-rules/:id` | Delete page rule |

### Zone Settings

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/cloudflare/zone/settings` | Get all zone settings |
| PATCH | `/api/v1/cloudflare/zone/settings` | Update zone settings |
| POST | `/api/v1/cloudflare/zone/development-mode` | Toggle development mode |

### Plugin Settings

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/cloudflare/settings` | Get plugin settings |
| PUT | `/api/v1/cloudflare/settings` | Update plugin settings |
| POST | `/api/v1/cloudflare/settings/reset` | Reset to defaults |

---

## 4. Cloudflare Client Implementation

### Core Client Structure (Rust)

```rust
// src/services/cloudflare_client.rs

use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct CloudflareClient {
    client: Client,
    api_token: String,
    account_id: String,
    zone_id: String,
}

impl CloudflareClient {
    pub fn new(api_token: &str, account_id: &str, zone_id: &str) -> Self {
        let client = Client::builder()
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    "Authorization",
                    format!("Bearer {}", api_token).parse().unwrap(),
                );
                headers.insert("Content-Type", "application/json".parse().unwrap());
                headers
            })
            .build()
            .unwrap();

        Self {
            client,
            api_token: api_token.to_string(),
            account_id: account_id.to_string(),
            zone_id: zone_id.to_string(),
        }
    }

    const BASE_URL: &'static str = "https://api.cloudflare.com/client/v4";

    // Zone endpoints
    pub async fn get_zone(&self) -> Result<Zone, CloudflareError> { ... }

    // Cache endpoints
    pub async fn purge_all(&self) -> Result<(), CloudflareError> { ... }
    pub async fn purge_urls(&self, urls: &[String]) -> Result<(), CloudflareError> { ... }
    pub async fn purge_tags(&self, tags: &[String]) -> Result<(), CloudflareError> { ... }

    // DNS endpoints
    pub async fn list_dns_records(&self) -> Result<Vec<DnsRecord>, CloudflareError> { ... }
    pub async fn create_dns_record(&self, record: &CreateDnsRecord) -> Result<DnsRecord, CloudflareError> { ... }
    pub async fn update_dns_record(&self, id: &str, record: &UpdateDnsRecord) -> Result<DnsRecord, CloudflareError> { ... }
    pub async fn delete_dns_record(&self, id: &str) -> Result<(), CloudflareError> { ... }

    // Security endpoints
    pub async fn get_security_level(&self) -> Result<String, CloudflareError> { ... }
    pub async fn set_security_level(&self, level: &str) -> Result<(), CloudflareError> { ... }
    pub async fn set_under_attack_mode(&self, enabled: bool) -> Result<(), CloudflareError> { ... }
    pub async fn list_firewall_rules(&self) -> Result<Vec<FirewallRule>, CloudflareError> { ... }

    // SSL endpoints
    pub async fn get_ssl_setting(&self) -> Result<SslSetting, CloudflareError> { ... }
    pub async fn set_ssl_mode(&self, mode: &str) -> Result<(), CloudflareError> { ... }

    // Workers endpoints
    pub async fn list_workers(&self) -> Result<Vec<Worker>, CloudflareError> { ... }
    pub async fn deploy_worker(&self, name: &str, script: &str) -> Result<Worker, CloudflareError> { ... }

    // R2 endpoints
    pub async fn list_r2_buckets(&self) -> Result<Vec<R2Bucket>, CloudflareError> { ... }
    pub async fn create_r2_bucket(&self, name: &str) -> Result<R2Bucket, CloudflareError> { ... }

    // Analytics endpoints
    pub async fn get_analytics(&self, since: DateTime<Utc>, until: DateTime<Utc>) -> Result<Analytics, CloudflareError> { ... }
}
```

---

## 5. Background Workers

### Auto-Purge Worker

```rust
// src/workers/auto_purge.rs

use tokio::sync::mpsc;

pub enum PurgeEvent {
    PostUpdated { post_id: Uuid, slug: String },
    MediaUploaded { media_id: Uuid, url: String },
    ThemeChanged { theme_id: String },
}

pub struct AutoPurgeWorker {
    rx: mpsc::Receiver<PurgeEvent>,
    cloudflare_client: CloudflareClient,
    settings: CloudflareSettings,
}

impl AutoPurgeWorker {
    pub async fn run(mut self) {
        while let Some(event) = self.rx.recv().await {
            if !self.settings.auto_purge_enabled {
                continue;
            }

            match event {
                PurgeEvent::PostUpdated { post_id, slug } => {
                    if self.settings.auto_purge_on_post_update {
                        let urls = vec![
                            format!("/{}", slug),
                            "/".to_string(),
                            "/sitemap.xml".to_string(),
                            "/feed/".to_string(),
                        ];
                        self.cloudflare_client.purge_urls(&urls).await.ok();
                    }
                }
                PurgeEvent::MediaUploaded { media_id, url } => {
                    if self.settings.auto_purge_on_media_upload {
                        self.cloudflare_client.purge_urls(&[url]).await.ok();
                    }
                }
                PurgeEvent::ThemeChanged { theme_id } => {
                    if self.settings.auto_purge_on_theme_change {
                        // Purge all CSS/JS files
                        self.cloudflare_client.purge_all().await.ok();
                    }
                }
            }
        }
    }
}
```

### Cache Warmer Worker

```rust
// src/workers/cache_warmer.rs

pub struct CacheWarmerWorker {
    cloudflare_client: CloudflareClient,
    settings: CloudflareSettings,
    db_pool: PgPool,
}

impl CacheWarmerWorker {
    pub async fn warm_cache(&self) -> Result<(), Error> {
        // Get important URLs to warm
        let urls_to_warm = self.get_priority_urls().await?;

        for url in urls_to_warm {
            // Make GET request to warm the cache
            reqwest::get(&url).await.ok();
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    async fn get_priority_urls(&self) -> Result<Vec<String>, Error> {
        // Query database for:
        // - Homepage
        // - Recent posts
        // - Popular posts
        // - Important pages
        // - Sitemap
        // ...
    }
}
```

---

## 6. Event Hooks Integration

Integrate with RustPress hooks to trigger auto-purge:

```rust
// In RustPress plugin system

pub fn register_cloudflare_hooks(plugin_manager: &mut PluginManager) {
    // Post hooks
    plugin_manager.add_hook("post_published", |ctx| {
        let post = ctx.get::<Post>("post")?;
        cloudflare_service.on_post_updated(post.id, &post.slug).await
    });

    plugin_manager.add_hook("post_updated", |ctx| {
        let post = ctx.get::<Post>("post")?;
        cloudflare_service.on_post_updated(post.id, &post.slug).await
    });

    plugin_manager.add_hook("post_deleted", |ctx| {
        let post = ctx.get::<Post>("post")?;
        cloudflare_service.on_post_deleted(post.id, &post.slug).await
    });

    // Media hooks
    plugin_manager.add_hook("media_uploaded", |ctx| {
        let media = ctx.get::<Media>("media")?;
        cloudflare_service.on_media_uploaded(media.id, &media.url).await
    });

    // Theme hooks
    plugin_manager.add_hook("theme_activated", |ctx| {
        let theme_id = ctx.get::<String>("theme_id")?;
        cloudflare_service.on_theme_changed(&theme_id).await
    });
}
```

---

## 7. Required Cloudflare API Permissions

When users create their API token, they need these permissions:

### Zone Permissions
- **Zone Settings**: Read, Edit
- **Zone**: Read
- **Cache Purge**: Purge
- **DNS**: Read, Edit
- **Firewall Services**: Read, Edit
- **Page Rules**: Read, Edit
- **SSL and Certificates**: Read, Edit
- **Analytics**: Read

### Account Permissions
- **Workers Scripts**: Read, Edit
- **Workers KV Storage**: Read, Edit
- **R2 Storage**: Read, Edit (if using R2)
- **Cloudflare Stream**: Read, Edit (if using Stream)

---

## 8. Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum CloudflareError {
    #[error("Not authenticated - check your API token")]
    Unauthorized,

    #[error("Zone not found - check your Zone ID")]
    ZoneNotFound,

    #[error("Rate limited - please wait and try again")]
    RateLimited { retry_after: u32 },

    #[error("Feature not available on your plan")]
    PlanRestriction { feature: String },

    #[error("API error: {message}")]
    ApiError { code: u32, message: String },

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Invalid response: {0}")]
    ParseError(#[from] serde_json::Error),
}
```

---

## 9. Response Format

All API responses follow RustPress standard format:

```json
// Success response
{
  "success": true,
  "data": { ... },
  "meta": {
    "page": 1,
    "per_page": 20,
    "total": 100
  }
}

// Error response
{
  "success": false,
  "error": {
    "code": "CLOUDFLARE_API_ERROR",
    "message": "Failed to purge cache",
    "details": { ... }
  }
}
```

---

## 10. Implementation Checklist

### Phase 1: Core Setup
- [ ] Database migrations
- [ ] Cloudflare client implementation
- [ ] Settings endpoints (GET, PUT)
- [ ] Connection test endpoint
- [ ] Zone info endpoint

### Phase 2: Cache Management
- [ ] Purge all cache
- [ ] Purge by URLs
- [ ] Purge by tags (Enterprise)
- [ ] Purge by prefix (Enterprise)
- [ ] Cache statistics endpoint
- [ ] Auto-purge worker

### Phase 3: DNS Management
- [ ] List DNS records
- [ ] Create DNS record
- [ ] Update DNS record
- [ ] Delete DNS record
- [ ] Export/Import zone file

### Phase 4: Security Features
- [ ] Security level endpoints
- [ ] Under Attack mode toggle
- [ ] Firewall rules CRUD
- [ ] IP access rules
- [ ] Security events log
- [ ] WAF rules management

### Phase 5: SSL/TLS
- [ ] SSL status endpoint
- [ ] SSL mode management
- [ ] Certificate listing
- [ ] Certificate ordering
- [ ] Custom certificate upload

### Phase 6: Workers & KV
- [ ] Workers list/deploy/update/delete
- [ ] Worker routes management
- [ ] Worker templates
- [ ] KV namespace CRUD
- [ ] KV key-value operations

### Phase 7: R2 Storage
- [ ] Bucket CRUD
- [ ] Object listing
- [ ] Object upload (multipart)
- [ ] Object deletion
- [ ] Signed URL generation

### Phase 8: Analytics
- [ ] Traffic analytics
- [ ] Bandwidth statistics
- [ ] Cache hit ratio
- [ ] Security events analytics
- [ ] Analytics caching layer

### Phase 9: Page Rules
- [ ] List page rules
- [ ] Create page rule
- [ ] Update page rule
- [ ] Delete page rule
- [ ] Priority reordering

### Phase 10: Advanced Features
- [ ] Cache warming worker
- [ ] Webhook notifications (Slack)
- [ ] Email alerts
- [ ] Performance optimization
- [ ] Rate limiting handling

---

## 11. Testing Strategy

### Unit Tests
- Cloudflare client methods
- Service layer business logic
- Request/response serialization

### Integration Tests
- API endpoint testing
- Database operations
- Background workers

### Mock Server
- Create mock Cloudflare API for development
- Simulate various error conditions
- Test rate limiting handling

---

## 12. Security Considerations

1. **API Token Storage**: Encrypt API tokens at rest using AES-256
2. **Token Validation**: Validate tokens before storing, test permissions
3. **Rate Limiting**: Implement client-side rate limiting to avoid Cloudflare limits
4. **Audit Logging**: Log all purge operations and security changes
5. **RBAC**: Integrate with RustPress role-based access control
6. **Input Validation**: Validate all user inputs before sending to Cloudflare

---

## Notes

- The admin UI is already complete with React/TypeScript/Vite
- Mock data is available for development testing
- Dark theme consistent with RustPress has been applied
- Connection overlay shows when Cloudflare is not configured
