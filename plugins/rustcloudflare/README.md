# RustCloudflare

<div align="center">

![RustCloudflare Banner](https://raw.githubusercontent.com/rustpress-net/rustcloudflare/master/assets/banner.png)

**Seamless Cloudflare integration for RustPress CMS**

[![Version](https://img.shields.io/badge/version-1.0.1-blue.svg)](https://github.com/rustpress-net/rustcloudflare/releases)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![RustPress](https://img.shields.io/badge/RustPress-%3E%3D1.0.0-orange.svg)](https://rustpress.io)

[Features](#features) | [Installation](#installation) | [Configuration](#configuration) | [Screenshots](#screenshots) | [API](#api-reference) | [CLI](#cli-commands)

</div>

---

## Overview

RustCloudflare is a comprehensive Cloudflare integration plugin for RustPress CMS. It provides a beautiful admin interface to manage all aspects of your Cloudflare configuration directly from your RustPress dashboard - CDN, DNS, Security, SSL, Workers, R2 Storage, D1 Database, Stream Video, and more.

## Features

### CDN & Performance
- **Cache Management** - Purge cache by URL, tags, prefixes, or purge everything
- **Auto Minification** - Automatically minify JavaScript, CSS, and HTML
- **Brotli Compression** - Enable Brotli for faster content delivery
- **Early Hints (103)** - Preload resources for faster page loads
- **Rocket Loader** - Async loading of JavaScript
- **HTTP/2 & HTTP/3** - Modern protocol support with QUIC
- **0-RTT Connection Resumption** - Faster repeat connections

### DNS Management
- **Full DNS Control** - Create, update, and delete DNS records
- **Record Types** - A, AAAA, CNAME, MX, TXT, SRV, CAA, NS
- **Proxy Toggle** - Enable/disable Cloudflare proxy per record
- **Zone Import/Export** - Import and export zone files
- **Search & Filter** - Find records quickly

### Security & WAF
- **Security Levels** - Off, Low, Medium, High, Under Attack
- **Web Application Firewall** - Custom WAF rules
- **Firewall Rules** - IP blocking, country blocking, custom expressions
- **Bot Management** - Detect and manage bot traffic
- **DDoS Protection** - Advanced DDoS mitigation
- **Under Attack Mode** - One-click emergency protection
- **Browser Integrity Check** - Block malicious browsers
- **IP Access Rules** - Whitelist/blacklist IP addresses

### SSL/TLS
- **SSL Modes** - Off, Flexible, Full, Full (Strict)
- **Certificate Management** - View and manage SSL certificates
- **Always HTTPS** - Force HTTPS redirects
- **Minimum TLS Version** - Set minimum TLS 1.0, 1.1, 1.2, or 1.3
- **Automatic HTTPS Rewrites** - Fix mixed content
- **Opportunistic Encryption** - HTTP/2 encryption

### Workers & Edge Computing
- **Workers Management** - Deploy and manage Workers scripts
- **Worker Routes** - Configure URL patterns for Workers
- **Workers KV** - Key-value storage at the edge
- **KV Browser** - Browse and edit KV namespaces
- **Code Editor** - Edit Worker scripts with syntax highlighting

### R2 Object Storage
- **Bucket Management** - Create and manage R2 buckets
- **Object Browser** - Browse, upload, and delete objects
- **Media Sync** - Sync RustPress media to R2
- **Public URLs** - Configure custom domains for R2
- **Multipart Uploads** - Handle large file uploads

### D1 Database
- **Database Management** - Create and manage D1 databases
- **Table Browser** - View database tables and schema
- **Query Editor** - Execute SQL queries
- **Query History** - Track recent queries
- **Schema Viewer** - Inspect table structures

### Stream Video
- **Video Hosting** - Upload and manage videos
- **Live Streaming** - Create and manage live inputs
- **RTMP/SRT/WebRTC** - Multiple streaming protocols
- **Embed Codes** - Generate embed codes with options
- **Video Analytics** - View video statistics

### Analytics & Monitoring
- **Traffic Analytics** - Requests, bandwidth, visitors
- **Geographic Data** - Traffic by country
- **Threat Analytics** - Security events and blocked threats
- **Cache Analytics** - Hit ratio, bandwidth saved
- **Performance Metrics** - Response times, status codes
- **Time Range Selection** - 1 hour to 30 days

### Page Rules
- **Rule Management** - Create and manage page rules
- **Cache Settings** - Custom cache rules per URL
- **Forwarding Rules** - URL redirects
- **Security Rules** - Per-URL security settings
- **Performance Rules** - Custom optimization per URL

---

## Screenshots

### Dashboard
![Dashboard](https://raw.githubusercontent.com/rustpress-net/rustcloudflare/master/assets/screenshots/dashboard.png)
*Overview of your Cloudflare zone with key metrics and quick actions*

### DNS Management
![DNS Management](https://raw.githubusercontent.com/rustpress-net/rustcloudflare/master/assets/screenshots/dns.png)
*Full DNS record management with search, filtering, and bulk operations*

### Security & WAF
![Security](https://raw.githubusercontent.com/rustpress-net/rustcloudflare/master/assets/screenshots/security.png)
*Security settings, WAF rules, and IP access management*

### Cache Management
![Cache](https://raw.githubusercontent.com/rustpress-net/rustcloudflare/master/assets/screenshots/cache.png)
*Purge cache by URL, tags, or purge everything with one click*

### Workers & KV
![Workers](https://raw.githubusercontent.com/rustpress-net/rustcloudflare/master/assets/screenshots/workers.png)
*Deploy and manage Cloudflare Workers with KV storage*

### R2 Storage
![R2 Storage](https://raw.githubusercontent.com/rustpress-net/rustcloudflare/master/assets/screenshots/r2.png)
*Object storage management with bucket and file browser*

### D1 Database
![D1 Database](https://raw.githubusercontent.com/rustpress-net/rustcloudflare/master/assets/screenshots/d1.png)
*Edge database management with query editor and table browser*

### Analytics
![Analytics](https://raw.githubusercontent.com/rustpress-net/rustcloudflare/master/assets/screenshots/analytics.png)
*Comprehensive traffic and performance analytics*

---

## Installation

### Via RustPress Admin
1. Go to **Plugins > Add New** in your RustPress admin
2. Search for "RustCloudflare"
3. Click **Install** and then **Activate**

### Via CLI
```bash
rustpress plugin install rustcloudflare
rustpress plugin activate rustcloudflare
```

### Manual Installation
1. Download the latest release from [GitHub Releases](https://github.com/rustpress-net/rustcloudflare/releases)
2. Extract to `plugins/rustcloudflare/` in your RustPress installation
3. Activate via admin or CLI

---

## Configuration

### Getting Your Cloudflare Credentials

#### API Token (Recommended)
1. Log in to [Cloudflare Dashboard](https://dash.cloudflare.com)
2. Go to **My Profile > API Tokens**
3. Click **Create Token**
4. Use the **Edit zone DNS** template or create a custom token with:
   - **Zone:Zone:Read**
   - **Zone:Zone Settings:Edit**
   - **Zone:DNS:Edit**
   - **Zone:Cache Purge:Purge**
   - **Account:Workers Scripts:Edit** (for Workers)
   - **Account:Workers KV Storage:Edit** (for KV)
   - **Account:Cloudflare D1:Edit** (for D1)
   - **Account:Stream:Edit** (for Stream)
   - **Account:R2 Storage:Edit** (for R2)

#### Account ID & Zone ID
1. Log in to Cloudflare Dashboard
2. Select your domain
3. On the overview page, scroll down to find **Zone ID** and **Account ID** in the right sidebar

### Settings Reference

#### Credentials Group
| Setting | Type | Description | Required |
|---------|------|-------------|----------|
| `api_token` | Password | Cloudflare API token | Yes |
| `account_id` | String | Cloudflare Account ID | Yes |
| `zone_id` | String | Zone ID for your domain | Yes |
| `email` | Email | Account email (legacy API) | No |

#### CDN Group
| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `cdn_enabled` | Boolean | `true` | Enable Cloudflare CDN |
| `auto_minify` | Multi-select | `["javascript", "css", "html"]` | Assets to minify |
| `brotli_compression` | Boolean | `true` | Enable Brotli compression |
| `early_hints` | Boolean | `true` | Enable Early Hints (103) |
| `rocket_loader` | Boolean | `false` | Async script loading |

#### Cache Group
| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `cache_level` | Select | `aggressive` | Cache level (bypass/basic/simplified/aggressive) |
| `browser_cache_ttl` | Integer | `14400` | Browser cache TTL in seconds |
| `edge_cache_ttl` | Integer | `86400` | Edge cache TTL in seconds |
| `cache_by_device_type` | Boolean | `false` | Separate cache for mobile/desktop |
| `cache_deception_armor` | Boolean | `true` | Protect against cache deception |

#### Security Group
| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `security_level` | Select | `medium` | Security level |
| `waf_enabled` | Boolean | `true` | Enable WAF |
| `bot_management` | Boolean | `true` | Enable bot detection |
| `ddos_protection` | Boolean | `true` | Enable DDoS protection |
| `challenge_passage` | Integer | `1800` | Challenge validity (seconds) |
| `browser_integrity_check` | Boolean | `true` | Check browser headers |

#### SSL Group
| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `ssl_mode` | Select | `strict` | SSL mode (off/flexible/full/strict) |
| `always_use_https` | Boolean | `true` | Force HTTPS |
| `min_tls_version` | Select | `1.2` | Minimum TLS version |
| `automatic_https_rewrites` | Boolean | `true` | Rewrite HTTP to HTTPS |
| `opportunistic_encryption` | Boolean | `true` | Enable opportunistic encryption |

#### Performance Group
| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `argo_smart_routing` | Boolean | `false` | Smart routing (paid) |
| `argo_tiered_caching` | Boolean | `false` | Tiered caching (paid) |
| `http2` | Boolean | `true` | Enable HTTP/2 |
| `http3` | Boolean | `true` | Enable HTTP/3 (QUIC) |
| `zero_rtt` | Boolean | `true` | 0-RTT connection resumption |
| `websockets` | Boolean | `true` | Allow WebSockets |

#### Images Group
| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `polish` | Select | `lossy` | Image optimization (off/lossless/lossy) |
| `webp` | Boolean | `true` | Convert to WebP |
| `mirage` | Boolean | `true` | Mobile image optimization |
| `image_resizing` | Boolean | `true` | On-the-fly resizing |

#### Workers Group
| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `workers_enabled` | Boolean | `true` | Enable Workers |
| `workers_kv_enabled` | Boolean | `true` | Enable KV storage |
| `workers_kv_namespace` | String | - | KV namespace ID |

#### R2 Group
| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `r2_enabled` | Boolean | `false` | Enable R2 storage |
| `r2_bucket` | String | - | R2 bucket name |
| `r2_access_key_id` | String | - | R2 access key ID |
| `r2_secret_access_key` | Password | - | R2 secret key |
| `r2_public_url` | URL | - | R2 public URL |

#### D1 Group
| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `d1_enabled` | Boolean | `false` | Enable D1 database |
| `d1_database_id` | String | - | D1 database ID |

#### Stream Group
| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `stream_enabled` | Boolean | `false` | Enable Stream video |

#### Analytics Group
| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `analytics_enabled` | Boolean | `true` | Enable analytics |
| `analytics_token` | String | - | Web Analytics token |

#### DNS Group
| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `dns_management` | Boolean | `true` | Allow DNS management |
| `auto_dns_sync` | Boolean | `false` | Auto-sync DNS records |

---

## API Reference

### Cache Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/cache/purge` | Purge specific URLs |
| `POST` | `/cache/purge/all` | Purge entire cache |
| `POST` | `/cache/purge/tags` | Purge by cache tags |
| `POST` | `/cache/purge/prefix` | Purge by URL prefix |
| `GET` | `/cache/status` | Get cache statistics |

### DNS Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/dns/records` | List DNS records |
| `POST` | `/dns/records` | Create DNS record |
| `PUT` | `/dns/records/:id` | Update DNS record |
| `DELETE` | `/dns/records/:id` | Delete DNS record |
| `POST` | `/dns/import` | Import zone file |
| `GET` | `/dns/export` | Export zone file |

### Security Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/security/waf/rules` | List WAF rules |
| `POST` | `/security/waf/rules` | Create WAF rule |
| `PUT` | `/security/waf/rules/:id` | Update WAF rule |
| `DELETE` | `/security/waf/rules/:id` | Delete WAF rule |
| `GET` | `/security/firewall/rules` | List firewall rules |
| `POST` | `/security/firewall/rules` | Create firewall rule |
| `GET` | `/security/events` | Get security events |
| `POST` | `/security/under-attack` | Toggle Under Attack mode |
| `GET` | `/security/ip-access/rules` | List IP access rules |
| `POST` | `/security/ip-access/rules` | Create IP access rule |

### SSL Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/ssl/status` | Get SSL status |
| `GET` | `/ssl/certificates` | List certificates |
| `POST` | `/ssl/certificates/order` | Order certificate |
| `POST` | `/ssl/verify` | Verify SSL config |

### Workers Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/workers` | List Workers |
| `POST` | `/workers` | Deploy Worker |
| `GET` | `/workers/:name` | Get Worker details |
| `PUT` | `/workers/:name` | Update Worker |
| `DELETE` | `/workers/:name` | Delete Worker |
| `GET` | `/workers/:name/routes` | List routes |
| `POST` | `/workers/:name/routes` | Create route |
| `GET` | `/workers/kv/namespaces` | List KV namespaces |
| `POST` | `/workers/kv/namespaces` | Create namespace |
| `GET` | `/workers/kv/:namespace/keys` | List KV keys |
| `GET` | `/workers/kv/:namespace/values/:key` | Get KV value |
| `PUT` | `/workers/kv/:namespace/values/:key` | Set KV value |
| `DELETE` | `/workers/kv/:namespace/values/:key` | Delete KV value |

### R2 Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/r2/buckets` | List buckets |
| `POST` | `/r2/buckets` | Create bucket |
| `GET` | `/r2/buckets/:bucket/objects` | List objects |
| `PUT` | `/r2/buckets/:bucket/objects/:key` | Upload object |
| `DELETE` | `/r2/buckets/:bucket/objects/:key` | Delete object |
| `POST` | `/r2/sync` | Sync media to R2 |

### D1 Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/d1/databases` | List databases |
| `POST` | `/d1/databases` | Create database |
| `POST` | `/d1/databases/:id/query` | Execute query |

### Stream Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/stream/videos` | List videos |
| `POST` | `/stream/videos` | Upload video |
| `GET` | `/stream/videos/:id` | Get video details |
| `DELETE` | `/stream/videos/:id` | Delete video |
| `GET` | `/stream/live-inputs` | List live inputs |
| `POST` | `/stream/live-inputs` | Create live input |

### Analytics Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/analytics` | Get analytics |
| `GET` | `/analytics/realtime` | Get real-time analytics |

### Zone Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/zone/settings` | Get zone settings |
| `PATCH` | `/zone/settings` | Update zone settings |
| `POST` | `/zone/development-mode` | Toggle dev mode |

---

## CLI Commands

```bash
# Show Cloudflare connection status
rustpress cloudflare status

# Purge cache
rustpress cloudflare purge --all
rustpress cloudflare purge --urls https://example.com/page1,https://example.com/page2
rustpress cloudflare purge --tags tag1,tag2

# DNS management
rustpress cloudflare dns list
rustpress cloudflare dns add A www 192.168.1.1
rustpress cloudflare dns delete <record-id>

# Workers management
rustpress cloudflare workers list
rustpress cloudflare workers deploy <script-name> <script-file>
rustpress cloudflare workers delete <script-name>

# Toggle Under Attack mode
rustpress cloudflare under-attack on
rustpress cloudflare under-attack off
```

---

## Permissions

| Permission | Description |
|------------|-------------|
| `manage_cloudflare` | Full Cloudflare management access |
| `view_cloudflare_analytics` | View analytics data |
| `manage_cloudflare_cache` | Manage cache settings and purge |
| `view_cloudflare_cache` | View cache statistics |
| `manage_cloudflare_dns` | Manage DNS records |
| `manage_cloudflare_security` | Manage security and WAF |
| `view_cloudflare_security` | View security events |
| `manage_cloudflare_ssl` | Manage SSL/TLS settings |
| `view_cloudflare_ssl` | View SSL status |
| `manage_cloudflare_workers` | Manage Workers and KV |
| `manage_cloudflare_r2` | Manage R2 storage |
| `manage_cloudflare_d1` | Manage D1 database |
| `manage_cloudflare_stream` | Manage Stream video |
| `manage_cloudflare_rules` | Manage page rules |

---

## Hooks & Auto-Purge

RustCloudflare automatically purges relevant cache when content changes in RustPress:

| Event | Cache Action |
|-------|--------------|
| Post published | Purge post URL, homepage, feeds |
| Post updated | Purge post URL |
| Post deleted | Purge post URL, homepage |
| Page updated | Purge page URL |
| Theme changed | Purge all cache |
| Menu updated | Purge all cache |

---

## Scheduled Tasks

| Task | Schedule | Description |
|------|----------|-------------|
| `cloudflare-analytics-sync` | Hourly | Sync analytics data |
| `cloudflare-cache-warmup` | Daily | Warm cache for popular pages |
| `cloudflare-ssl-check` | Daily | Check SSL certificate expiry |
| `cloudflare-security-report` | Weekly | Generate security report |

---

## Requirements

- RustPress >= 1.0.0
- Cloudflare account (free tier works for basic features)
- API Token with appropriate permissions

### Feature Requirements

| Feature | Cloudflare Plan |
|---------|-----------------|
| CDN, DNS, SSL | Free |
| Cache Purge | Free |
| Page Rules | Free (3 rules), Pro (20+) |
| WAF | Pro |
| Bot Management | Enterprise |
| Argo Smart Routing | Paid add-on |
| Workers | Free (100k req/day) |
| Workers KV | Paid |
| R2 Storage | Paid |
| D1 Database | Beta/Paid |
| Stream | Paid |
| Cache Tags | Enterprise |

---

## Development

### Building the Admin UI

```bash
cd admin-ui
npm install
npm run dev     # Development server
npm run build   # Production build
```

### Running Tests

```bash
cargo test
```

### Project Structure

```
rustcloudflare/
├── src/
│   ├── api/           # API endpoint handlers
│   ├── services/      # Cloudflare API services
│   ├── models/        # Data models
│   ├── hooks/         # RustPress hooks
│   ├── client.rs      # Cloudflare API client
│   ├── config.rs      # Plugin configuration
│   ├── error.rs       # Error types
│   └── lib.rs         # Plugin entry point
├── admin-ui/
│   └── src/
│       ├── pages/     # Admin page components
│       ├── components/# Reusable components
│       ├── stores/    # State management
│       └── lib/       # API client & utilities
├── migrations/        # Database migrations
├── assets/           # CSS & JS assets
├── templates/        # Server-side templates
└── plugin.toml       # Plugin manifest
```

---

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## Support

- [Documentation](https://rustpress.io/plugins/rustcloudflare)
- [GitHub Issues](https://github.com/rustpress-net/rustcloudflare/issues)
- [Discord Community](https://discord.gg/rustpress)

---

<div align="center">
Made with :heart: by the <a href="https://rustpress.io">RustPress Team</a>
</div>
