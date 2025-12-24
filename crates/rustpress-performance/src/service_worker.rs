//! Service Worker for Offline Support
//!
//! Generates service worker scripts and manages offline caching strategies.

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

/// Service worker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceWorkerConfig {
    /// Service worker version
    pub version: String,
    /// Cache name prefix
    pub cache_name_prefix: String,
    /// URLs to precache
    pub precache_urls: Vec<String>,
    /// URL patterns for runtime caching
    pub runtime_caching: Vec<RuntimeCachingRule>,
    /// Offline fallback page
    pub offline_page: Option<String>,
    /// Enable navigation preload
    pub navigation_preload: bool,
    /// Skip waiting on update
    pub skip_waiting: bool,
    /// Clients claim immediately
    pub clients_claim: bool,
}

impl Default for ServiceWorkerConfig {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            cache_name_prefix: "rustpress".to_string(),
            precache_urls: vec![
                "/".to_string(),
                "/offline".to_string(),
            ],
            runtime_caching: vec![
                RuntimeCachingRule {
                    url_pattern: "^/api/".to_string(),
                    strategy: CachingStrategy::NetworkFirst,
                    options: CacheOptions {
                        cache_name: Some("api-cache".to_string()),
                        max_entries: Some(50),
                        max_age_seconds: Some(300),
                        ..Default::default()
                    },
                },
                RuntimeCachingRule {
                    url_pattern: "\\.(js|css)$".to_string(),
                    strategy: CachingStrategy::StaleWhileRevalidate,
                    options: CacheOptions {
                        cache_name: Some("static-cache".to_string()),
                        max_entries: Some(100),
                        max_age_seconds: Some(86400),
                        ..Default::default()
                    },
                },
                RuntimeCachingRule {
                    url_pattern: "\\.(png|jpg|jpeg|gif|webp|svg)$".to_string(),
                    strategy: CachingStrategy::CacheFirst,
                    options: CacheOptions {
                        cache_name: Some("image-cache".to_string()),
                        max_entries: Some(200),
                        max_age_seconds: Some(2592000), // 30 days
                        ..Default::default()
                    },
                },
            ],
            offline_page: Some("/offline".to_string()),
            navigation_preload: true,
            skip_waiting: true,
            clients_claim: true,
        }
    }
}

/// Runtime caching rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeCachingRule {
    /// URL pattern (regex)
    pub url_pattern: String,
    /// Caching strategy
    pub strategy: CachingStrategy,
    /// Cache options
    pub options: CacheOptions,
}

/// Caching strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CachingStrategy {
    /// Try network first, fall back to cache
    NetworkFirst,
    /// Try cache first, fall back to network
    CacheFirst,
    /// Return cache immediately, update cache in background
    StaleWhileRevalidate,
    /// Only use network
    NetworkOnly,
    /// Only use cache
    CacheOnly,
}

impl CachingStrategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NetworkFirst => "NetworkFirst",
            Self::CacheFirst => "CacheFirst",
            Self::StaleWhileRevalidate => "StaleWhileRevalidate",
            Self::NetworkOnly => "NetworkOnly",
            Self::CacheOnly => "CacheOnly",
        }
    }
}

/// Cache options
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheOptions {
    /// Custom cache name
    pub cache_name: Option<String>,
    /// Maximum number of entries
    pub max_entries: Option<usize>,
    /// Maximum age in seconds
    pub max_age_seconds: Option<u64>,
    /// Network timeout in seconds (for NetworkFirst)
    pub network_timeout_seconds: Option<u64>,
    /// Enable cache busting
    pub cache_busting: bool,
}

/// Service worker generator
pub struct ServiceWorkerGenerator {
    config: ServiceWorkerConfig,
}

impl ServiceWorkerGenerator {
    pub fn new(config: ServiceWorkerConfig) -> Self {
        Self { config }
    }

    /// Generate the service worker JavaScript code
    pub fn generate(&self) -> String {
        let mut sw = String::new();

        // Header comment
        sw.push_str(&format!(
            "// RustPress Service Worker v{}\n",
            self.config.version
        ));
        sw.push_str("// Auto-generated - do not edit manually\n\n");

        // Cache names
        sw.push_str(&self.generate_cache_names());

        // Precache manifest
        sw.push_str(&self.generate_precache_manifest());

        // Install event
        sw.push_str(&self.generate_install_handler());

        // Activate event
        sw.push_str(&self.generate_activate_handler());

        // Fetch event
        sw.push_str(&self.generate_fetch_handler());

        // Strategy implementations
        sw.push_str(&self.generate_strategies());

        // Utility functions
        sw.push_str(&self.generate_utilities());

        sw
    }

    fn generate_cache_names(&self) -> String {
        format!(
            r#"
const CACHE_VERSION = '{}';
const CACHE_PREFIX = '{}';
const PRECACHE_NAME = `${{CACHE_PREFIX}}-precache-${{CACHE_VERSION}}`;
const RUNTIME_CACHE_NAME = `${{CACHE_PREFIX}}-runtime-${{CACHE_VERSION}}`;

"#,
            self.config.version, self.config.cache_name_prefix
        )
    }

    fn generate_precache_manifest(&self) -> String {
        let urls_json: Vec<String> = self.config.precache_urls
            .iter()
            .map(|u| format!("  '{}'", u))
            .collect();

        format!(
            r#"
const PRECACHE_URLS = [
{}
];

"#,
            urls_json.join(",\n")
        )
    }

    fn generate_install_handler(&self) -> String {
        let skip_waiting = if self.config.skip_waiting {
            "self.skipWaiting();"
        } else {
            ""
        };

        format!(
            r#"
self.addEventListener('install', (event) => {{
  event.waitUntil(
    caches.open(PRECACHE_NAME)
      .then((cache) => {{
        console.log('[SW] Precaching app shell');
        return cache.addAll(PRECACHE_URLS);
      }})
      .then(() => {{
        {}
      }})
  );
}});

"#,
            skip_waiting
        )
    }

    fn generate_activate_handler(&self) -> String {
        let clients_claim = if self.config.clients_claim {
            "return self.clients.claim();"
        } else {
            ""
        };

        let navigation_preload = if self.config.navigation_preload {
            r#"
    if (self.registration.navigationPreload) {
      await self.registration.navigationPreload.enable();
    }
"#
        } else {
            ""
        };

        format!(
            r#"
self.addEventListener('activate', (event) => {{
  event.waitUntil(
    (async () => {{
      // Clean up old caches
      const cacheNames = await caches.keys();
      await Promise.all(
        cacheNames
          .filter((name) => name.startsWith(CACHE_PREFIX) && name !== PRECACHE_NAME && name !== RUNTIME_CACHE_NAME)
          .map((name) => caches.delete(name))
      );
      {}
      {}
    }})()
  );
}});

"#,
            navigation_preload, clients_claim
        )
    }

    fn generate_fetch_handler(&self) -> String {
        let mut rules_code = String::new();

        for rule in &self.config.runtime_caching {
            let strategy = rule.strategy.as_str();
            let cache_name = rule.options.cache_name
                .as_ref()
                .map(|n| format!("'{}'", n))
                .unwrap_or_else(|| "RUNTIME_CACHE_NAME".to_string());

            let options = self.generate_cache_options(&rule.options);

            rules_code.push_str(&format!(
                r#"
  if (/{pattern}/.test(url.pathname) || /{pattern}/.test(url.href)) {{
    event.respondWith({strategy}(event.request, {cache}, {options}));
    return;
  }}
"#,
                pattern = rule.url_pattern,
                strategy = strategy.to_lowercase(),
                cache = cache_name,
                options = options
            ));
        }

        let offline_fallback = self.config.offline_page
            .as_ref()
            .map(|page| format!(
                r#"
  // Handle navigation requests with offline fallback
  if (event.request.mode === 'navigate') {{
    event.respondWith(
      (async () => {{
        try {{
          const preloadResponse = await event.preloadResponse;
          if (preloadResponse) {{
            return preloadResponse;
          }}
          return await fetch(event.request);
        }} catch (error) {{
          const cache = await caches.open(PRECACHE_NAME);
          return cache.match('{}');
        }}
      }})()
    );
    return;
  }}
"#,
                page
            ))
            .unwrap_or_default();

        format!(
            r#"
self.addEventListener('fetch', (event) => {{
  const url = new URL(event.request.url);

  // Skip cross-origin requests
  if (url.origin !== location.origin) {{
    return;
  }}

  // Skip non-GET requests
  if (event.request.method !== 'GET') {{
    return;
  }}
{rules}
{offline}
  // Default: Network first for navigation, cache first for others
  if (event.request.mode === 'navigate') {{
    event.respondWith(networkFirst(event.request, RUNTIME_CACHE_NAME, {{}}));
  }} else {{
    event.respondWith(staleWhileRevalidate(event.request, RUNTIME_CACHE_NAME, {{}}));
  }}
}});

"#,
            rules = rules_code,
            offline = offline_fallback
        )
    }

    fn generate_cache_options(&self, options: &CacheOptions) -> String {
        let mut opts = vec![];

        if let Some(max_entries) = options.max_entries {
            opts.push(format!("maxEntries: {}", max_entries));
        }

        if let Some(max_age) = options.max_age_seconds {
            opts.push(format!("maxAgeSeconds: {}", max_age));
        }

        if let Some(timeout) = options.network_timeout_seconds {
            opts.push(format!("networkTimeoutSeconds: {}", timeout));
        }

        format!("{{ {} }}", opts.join(", "))
    }

    fn generate_strategies(&self) -> String {
        r#"
// Network First Strategy
async function networkFirst(request, cacheName, options) {
  const cache = await caches.open(cacheName);

  try {
    const networkResponse = await fetchWithTimeout(request, options.networkTimeoutSeconds || 3);
    if (networkResponse.ok) {
      cache.put(request, networkResponse.clone());
    }
    return networkResponse;
  } catch (error) {
    const cachedResponse = await cache.match(request);
    if (cachedResponse) {
      return cachedResponse;
    }
    throw error;
  }
}

// Cache First Strategy
async function cacheFirst(request, cacheName, options) {
  const cache = await caches.open(cacheName);
  const cachedResponse = await cache.match(request);

  if (cachedResponse) {
    // Check if cached response is still valid
    if (!isExpired(cachedResponse, options.maxAgeSeconds)) {
      return cachedResponse;
    }
  }

  try {
    const networkResponse = await fetch(request);
    if (networkResponse.ok) {
      cache.put(request, networkResponse.clone());
    }
    return networkResponse;
  } catch (error) {
    if (cachedResponse) {
      return cachedResponse;
    }
    throw error;
  }
}

// Stale While Revalidate Strategy
async function staleWhileRevalidate(request, cacheName, options) {
  const cache = await caches.open(cacheName);
  const cachedResponse = await cache.match(request);

  const fetchPromise = fetch(request)
    .then((networkResponse) => {
      if (networkResponse.ok) {
        cache.put(request, networkResponse.clone());
      }
      return networkResponse;
    })
    .catch(() => null);

  return cachedResponse || await fetchPromise;
}

// Network Only Strategy
async function networkOnly(request, cacheName, options) {
  return fetch(request);
}

// Cache Only Strategy
async function cacheOnly(request, cacheName, options) {
  const cache = await caches.open(cacheName);
  const cachedResponse = await cache.match(request);

  if (cachedResponse) {
    return cachedResponse;
  }

  throw new Error('No cached response available');
}

"#
        .to_string()
    }

    fn generate_utilities(&self) -> String {
        r#"
// Fetch with timeout
function fetchWithTimeout(request, timeoutSeconds) {
  return new Promise((resolve, reject) => {
    const timeoutId = setTimeout(() => {
      reject(new Error('Network timeout'));
    }, timeoutSeconds * 1000);

    fetch(request)
      .then((response) => {
        clearTimeout(timeoutId);
        resolve(response);
      })
      .catch((error) => {
        clearTimeout(timeoutId);
        reject(error);
      });
  });
}

// Check if cached response is expired
function isExpired(response, maxAgeSeconds) {
  if (!maxAgeSeconds) return false;

  const dateHeader = response.headers.get('date');
  if (!dateHeader) return false;

  const date = new Date(dateHeader);
  const age = (Date.now() - date.getTime()) / 1000;

  return age > maxAgeSeconds;
}

// Trim cache to max entries
async function trimCache(cacheName, maxEntries) {
  const cache = await caches.open(cacheName);
  const keys = await cache.keys();

  if (keys.length > maxEntries) {
    for (let i = 0; i < keys.length - maxEntries; i++) {
      await cache.delete(keys[i]);
    }
  }
}

// Background sync for offline actions
self.addEventListener('sync', (event) => {
  if (event.tag === 'sync-posts') {
    event.waitUntil(syncPendingPosts());
  }
});

async function syncPendingPosts() {
  // Implement background sync logic here
  console.log('[SW] Syncing pending posts');
}

// Push notification handler
self.addEventListener('push', (event) => {
  const data = event.data ? event.data.json() : {};

  const options = {
    body: data.body || 'New notification',
    icon: data.icon || '/icon-192.png',
    badge: data.badge || '/badge.png',
    data: data.url || '/'
  };

  event.waitUntil(
    self.registration.showNotification(data.title || 'RustPress', options)
  );
});

self.addEventListener('notificationclick', (event) => {
  event.notification.close();

  event.waitUntil(
    clients.openWindow(event.notification.data)
  );
});

"#
        .to_string()
    }

    /// Generate the registration script to include in HTML
    pub fn generate_registration_script(&self) -> String {
        format!(
            r#"
if ('serviceWorker' in navigator) {{
  window.addEventListener('load', () => {{
    navigator.serviceWorker.register('/sw.js')
      .then((registration) => {{
        console.log('SW registered:', registration.scope);

        // Check for updates periodically
        setInterval(() => {{
          registration.update();
        }}, 60 * 60 * 1000); // Every hour
      }})
      .catch((error) => {{
        console.error('SW registration failed:', error);
      }});
  }});
}}
"#
        )
    }
}

/// Workbox-compatible manifest entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkboxManifestEntry {
    pub url: String,
    pub revision: Option<String>,
    pub integrity: Option<String>,
}

/// Generate a Workbox-compatible precache manifest
pub fn generate_workbox_manifest(assets: &[(String, String)]) -> Vec<WorkboxManifestEntry> {
    assets
        .iter()
        .map(|(url, hash)| WorkboxManifestEntry {
            url: url.clone(),
            revision: Some(hash.clone()),
            integrity: None,
        })
        .collect()
}

/// App shell configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppShellConfig {
    /// HTML shell file
    pub shell_html: String,
    /// Shell includes (CSS, JS)
    pub includes: Vec<String>,
    /// Routes to serve with app shell
    pub shell_routes: Vec<String>,
}

impl Default for AppShellConfig {
    fn default() -> Self {
        Self {
            shell_html: "/shell.html".to_string(),
            includes: vec![
                "/css/app.css".to_string(),
                "/js/app.js".to_string(),
            ],
            shell_routes: vec![
                "/".to_string(),
                "/posts/*".to_string(),
                "/pages/*".to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_worker_generation() {
        let config = ServiceWorkerConfig::default();
        let generator = ServiceWorkerGenerator::new(config);
        let sw = generator.generate();

        assert!(sw.contains("self.addEventListener('install'"));
        assert!(sw.contains("self.addEventListener('activate'"));
        assert!(sw.contains("self.addEventListener('fetch'"));
    }

    #[test]
    fn test_registration_script() {
        let config = ServiceWorkerConfig::default();
        let generator = ServiceWorkerGenerator::new(config);
        let script = generator.generate_registration_script();

        assert!(script.contains("serviceWorker"));
        assert!(script.contains("register"));
    }

    #[test]
    fn test_caching_strategy() {
        assert_eq!(CachingStrategy::NetworkFirst.as_str(), "NetworkFirst");
        assert_eq!(CachingStrategy::CacheFirst.as_str(), "CacheFirst");
        assert_eq!(CachingStrategy::StaleWhileRevalidate.as_str(), "StaleWhileRevalidate");
    }
}
