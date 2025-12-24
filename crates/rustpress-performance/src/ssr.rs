//! Server-Side Rendering Optimization
//!
//! Optimizes SSR performance with streaming, caching, and hydration strategies.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// SSR errors
#[derive(Debug, Error)]
pub enum SsrError {
    #[error("Render error: {0}")]
    RenderError(String),

    #[error("Timeout error")]
    Timeout,

    #[error("Hydration mismatch: {0}")]
    HydrationMismatch(String),
}

/// SSR configuration
#[derive(Debug, Clone)]
pub struct SsrConfig {
    /// Enable streaming SSR
    pub streaming: bool,
    /// Render timeout in milliseconds
    pub timeout_ms: u64,
    /// Enable component-level caching
    pub component_cache: bool,
    /// Maximum cached components
    pub max_cached_components: usize,
    /// Component cache TTL in seconds
    pub component_cache_ttl: u64,
    /// Enable progressive hydration
    pub progressive_hydration: bool,
    /// Hydration priority levels
    pub hydration_priorities: Vec<HydrationPriority>,
    /// Enable selective hydration
    pub selective_hydration: bool,
    /// Critical path components (always render sync)
    pub critical_components: Vec<String>,
}

impl Default for SsrConfig {
    fn default() -> Self {
        Self {
            streaming: true,
            timeout_ms: 10000,
            component_cache: true,
            max_cached_components: 1000,
            component_cache_ttl: 300,
            progressive_hydration: true,
            hydration_priorities: vec![
                HydrationPriority::Critical,
                HydrationPriority::High,
                HydrationPriority::Normal,
                HydrationPriority::Low,
                HydrationPriority::Idle,
            ],
            selective_hydration: true,
            critical_components: vec![
                "header".to_string(),
                "navigation".to_string(),
                "hero".to_string(),
            ],
        }
    }
}

/// Hydration priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HydrationPriority {
    /// Hydrate immediately (above the fold)
    Critical,
    /// Hydrate after critical content
    High,
    /// Standard hydration
    Normal,
    /// Hydrate on interaction
    Low,
    /// Hydrate during idle time
    Idle,
}

impl HydrationPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::High => "high",
            Self::Normal => "normal",
            Self::Low => "low",
            Self::Idle => "idle",
        }
    }
}

/// Component render result
#[derive(Debug, Clone)]
pub struct RenderResult {
    /// Rendered HTML
    pub html: String,
    /// Serialized component state
    pub state: Option<String>,
    /// Component dependencies
    pub dependencies: Vec<String>,
    /// Render time in microseconds
    pub render_time_us: u64,
    /// Cache hit
    pub cache_hit: bool,
}

/// Cached component entry
struct CachedComponent {
    html: String,
    state: Option<String>,
    created: Instant,
    hits: u64,
}

/// SSR component cache
pub struct ComponentCache {
    cache: Arc<RwLock<HashMap<String, CachedComponent>>>,
    config: SsrConfig,
}

impl ComponentCache {
    pub fn new(config: SsrConfig) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Get cached component
    pub fn get(&self, key: &str) -> Option<RenderResult> {
        let mut cache = self.cache.write();

        if let Some(entry) = cache.get_mut(key) {
            // Check TTL
            if entry.created.elapsed().as_secs() > self.config.component_cache_ttl {
                cache.remove(key);
                return None;
            }

            entry.hits += 1;

            return Some(RenderResult {
                html: entry.html.clone(),
                state: entry.state.clone(),
                dependencies: Vec::new(),
                render_time_us: 0,
                cache_hit: true,
            });
        }

        None
    }

    /// Store rendered component
    pub fn store(&self, key: String, result: &RenderResult) {
        let mut cache = self.cache.write();

        // Evict if over capacity
        if cache.len() >= self.config.max_cached_components {
            self.evict_lru(&mut cache);
        }

        cache.insert(key, CachedComponent {
            html: result.html.clone(),
            state: result.state.clone(),
            created: Instant::now(),
            hits: 0,
        });
    }

    fn evict_lru(&self, cache: &mut HashMap<String, CachedComponent>) {
        // Find entry with lowest hits
        if let Some(key) = cache
            .iter()
            .min_by_key(|(_, v)| v.hits)
            .map(|(k, _)| k.clone())
        {
            cache.remove(&key);
        }
    }

    /// Clear the cache
    pub fn clear(&self) {
        self.cache.write().clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let cache = self.cache.read();
        CacheStats {
            entries: cache.len(),
            total_hits: cache.values().map(|c| c.hits).sum(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub total_hits: u64,
}

/// SSR renderer
pub struct SsrRenderer {
    config: SsrConfig,
    component_cache: ComponentCache,
}

impl SsrRenderer {
    pub fn new(config: SsrConfig) -> Self {
        let component_cache = ComponentCache::new(config.clone());
        Self {
            config,
            component_cache,
        }
    }

    /// Generate cache key for a component
    pub fn cache_key(&self, component: &str, props: &HashMap<String, String>) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(component.as_bytes());

        let mut keys: Vec<_> = props.keys().collect();
        keys.sort();

        for key in keys {
            hasher.update(key.as_bytes());
            if let Some(value) = props.get(key) {
                hasher.update(value.as_bytes());
            }
        }

        hasher.finalize().to_hex()[..16].to_string()
    }

    /// Render a component
    pub fn render_component(
        &self,
        component: &str,
        props: &HashMap<String, String>,
        render_fn: impl FnOnce() -> Result<String, SsrError>,
    ) -> Result<RenderResult, SsrError> {
        let cache_key = self.cache_key(component, props);

        // Check cache
        if self.config.component_cache {
            if let Some(cached) = self.component_cache.get(&cache_key) {
                return Ok(cached);
            }
        }

        // Render component
        let start = Instant::now();
        let html = render_fn()?;
        let render_time_us = start.elapsed().as_micros() as u64;

        let result = RenderResult {
            html,
            state: None,
            dependencies: Vec::new(),
            render_time_us,
            cache_hit: false,
        };

        // Cache result
        if self.config.component_cache {
            self.component_cache.store(cache_key, &result);
        }

        Ok(result)
    }

    /// Check if component is critical
    pub fn is_critical(&self, component: &str) -> bool {
        self.config.critical_components.contains(&component.to_string())
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        self.component_cache.stats()
    }
}

/// Streaming SSR writer
pub struct StreamingRenderer {
    config: SsrConfig,
    /// Chunks ready to send
    chunks: Arc<RwLock<Vec<String>>>,
    /// Whether streaming has started
    started: Arc<RwLock<bool>>,
    /// Whether streaming has completed
    completed: Arc<RwLock<bool>>,
}

impl StreamingRenderer {
    pub fn new(config: SsrConfig) -> Self {
        Self {
            config,
            chunks: Arc::new(RwLock::new(Vec::new())),
            started: Arc::new(RwLock::new(false)),
            completed: Arc::new(RwLock::new(false)),
        }
    }

    /// Start streaming with initial HTML shell
    pub fn start(&self, shell: &str) {
        *self.started.write() = true;
        self.chunks.write().push(shell.to_string());
    }

    /// Push a chunk to the stream
    pub fn push_chunk(&self, chunk: &str) {
        if *self.started.read() && !*self.completed.read() {
            self.chunks.write().push(chunk.to_string());
        }
    }

    /// Push a hydration boundary
    pub fn push_suspense_boundary(&self, id: &str, fallback: &str) {
        let chunk = format!(
            r#"<div id="{}" data-suspense="pending">{}</div>"#,
            id, fallback
        );
        self.push_chunk(&chunk);
    }

    /// Resolve a suspense boundary
    pub fn resolve_suspense(&self, id: &str, content: &str, state: Option<&str>) {
        let state_script = state
            .map(|s| format!(
                r#"<script>window.__COMPONENT_STATE__["{}"] = {};</script>"#,
                id, s
            ))
            .unwrap_or_default();

        let chunk = format!(
            r#"<template id="{}$">{}</template>{}
<script>
(function() {{
  var t = document.getElementById("{}$");
  var b = document.getElementById("{}");
  b.innerHTML = t.content.textContent;
  b.dataset.suspense = "resolved";
  t.remove();
}})();
</script>"#,
            id, content, state_script, id, id
        );

        self.push_chunk(&chunk);
    }

    /// Complete the stream
    pub fn complete(&self, closing_html: &str) {
        self.push_chunk(closing_html);
        *self.completed.write() = true;
    }

    /// Get all accumulated chunks
    pub fn get_chunks(&self) -> Vec<String> {
        self.chunks.read().clone()
    }

    /// Take chunks (drains the queue)
    pub fn take_chunks(&self) -> Vec<String> {
        std::mem::take(&mut *self.chunks.write())
    }

    /// Check if streaming is complete
    pub fn is_complete(&self) -> bool {
        *self.completed.read()
    }
}

/// Hydration script generator
pub struct HydrationScriptGenerator {
    config: SsrConfig,
}

impl HydrationScriptGenerator {
    pub fn new(config: SsrConfig) -> Self {
        Self { config }
    }

    /// Generate the hydration bootstrap script
    pub fn generate_bootstrap(&self) -> String {
        let progressive = if self.config.progressive_hydration {
            r#"
  // Progressive hydration support
  const priorities = ['critical', 'high', 'normal', 'low', 'idle'];
  let currentPriority = 0;

  function hydrateByPriority(priority) {
    const components = document.querySelectorAll(`[data-hydrate="${priority}"]`);
    components.forEach(el => {
      const componentName = el.dataset.component;
      const state = window.__COMPONENT_STATE__[el.id];
      if (window.__HYDRATE_COMPONENT__) {
        window.__HYDRATE_COMPONENT__(el, componentName, state);
        el.dataset.hydrated = 'true';
      }
    });
  }

  function hydrateNext() {
    if (currentPriority < priorities.length) {
      hydrateByPriority(priorities[currentPriority]);
      currentPriority++;

      if (currentPriority < 3) {
        // Critical and high priority hydrate immediately
        requestAnimationFrame(hydrateNext);
      } else {
        // Lower priority hydrate during idle
        requestIdleCallback(hydrateNext, { timeout: 1000 });
      }
    }
  }
"#
        } else {
            ""
        };

        let selective = if self.config.selective_hydration {
            r#"
  // Selective hydration on interaction
  document.addEventListener('click', (e) => {
    let target = e.target;
    while (target && target !== document) {
      if (target.dataset.hydrate === 'low' && target.dataset.hydrated !== 'true') {
        const componentName = target.dataset.component;
        const state = window.__COMPONENT_STATE__[target.id];
        if (window.__HYDRATE_COMPONENT__) {
          window.__HYDRATE_COMPONENT__(target, componentName, state);
          target.dataset.hydrated = 'true';
        }
        return;
      }
      target = target.parentElement;
    }
  }, true);

  // Hydrate on visibility
  const observer = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
      if (entry.isIntersecting) {
        const el = entry.target;
        if (el.dataset.hydrate && el.dataset.hydrated !== 'true') {
          const componentName = el.dataset.component;
          const state = window.__COMPONENT_STATE__[el.id];
          if (window.__HYDRATE_COMPONENT__) {
            window.__HYDRATE_COMPONENT__(el, componentName, state);
            el.dataset.hydrated = 'true';
          }
          observer.unobserve(el);
        }
      }
    });
  }, { rootMargin: '100px' });

  document.querySelectorAll('[data-hydrate="normal"], [data-hydrate="low"]').forEach(el => {
    observer.observe(el);
  });
"#
        } else {
            ""
        };

        format!(
            r#"
(function() {{
  'use strict';

  // Component state storage
  window.__COMPONENT_STATE__ = window.__COMPONENT_STATE__ || {{}};

  // Hydration function placeholder (to be set by framework)
  window.__HYDRATE_COMPONENT__ = window.__HYDRATE_COMPONENT__ || function(el, name, state) {{
    console.log('Hydrating:', name, el.id);
  }};
  {}
  {}
  // Start hydration after DOMContentLoaded
  if (document.readyState === 'loading') {{
    document.addEventListener('DOMContentLoaded', hydrateNext);
  }} else {{
    hydrateNext();
  }}
}})();
"#,
            progressive, selective
        )
    }

    /// Generate component wrapper HTML
    pub fn wrap_component(
        &self,
        id: &str,
        component_name: &str,
        html: &str,
        priority: HydrationPriority,
        state: Option<&str>,
    ) -> String {
        let state_attr = state
            .map(|s| format!(" data-state=\"{}\"", html_escape(s)))
            .unwrap_or_default();

        format!(
            r#"<div id="{}" data-component="{}" data-hydrate="{}"{}>{}</div>"#,
            id, component_name, priority.as_str(), state_attr, html
        )
    }
}

/// Island architecture component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Island {
    /// Island identifier
    pub id: String,
    /// Component name
    pub component: String,
    /// Props passed to component
    pub props: HashMap<String, serde_json::Value>,
    /// Hydration strategy
    pub hydrate: HydrationStrategy,
    /// Slot content (static HTML)
    pub slot: Option<String>,
}

/// Hydration strategy for islands
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HydrationStrategy {
    /// Hydrate on page load
    Load,
    /// Hydrate when visible
    Visible,
    /// Hydrate on user interaction
    Interaction,
    /// Hydrate during idle time
    Idle,
    /// Only hydrate on specific media query
    Media,
    /// Never hydrate (static only)
    None,
}

impl HydrationStrategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Load => "load",
            Self::Visible => "visible",
            Self::Interaction => "interaction",
            Self::Idle => "idle",
            Self::Media => "media",
            Self::None => "none",
        }
    }
}

/// Island renderer
pub struct IslandRenderer {
    islands: Vec<Island>,
}

impl IslandRenderer {
    pub fn new() -> Self {
        Self { islands: Vec::new() }
    }

    /// Register an island
    pub fn register(&mut self, island: Island) {
        self.islands.push(island);
    }

    /// Render an island placeholder
    pub fn render_placeholder(&self, island: &Island, inner_html: &str) -> String {
        let props_json = serde_json::to_string(&island.props).unwrap_or_default();

        format!(
            r#"<div id="{}" data-island="{}" data-hydrate="{}" data-props="{}">
  {}
</div>"#,
            island.id,
            island.component,
            island.hydrate.as_str(),
            html_escape(&props_json),
            inner_html
        )
    }

    /// Generate island loader script
    pub fn generate_loader(&self) -> String {
        r#"
(function() {
  const islands = document.querySelectorAll('[data-island]');

  function hydrateIsland(el) {
    const component = el.dataset.island;
    const props = JSON.parse(el.dataset.props || '{}');

    // Dynamic import of island component
    import(`/islands/${component}.js`).then(module => {
      module.hydrate(el, props);
      el.dataset.hydrated = 'true';
    });
  }

  islands.forEach(el => {
    const strategy = el.dataset.hydrate;

    switch(strategy) {
      case 'load':
        hydrateIsland(el);
        break;

      case 'visible':
        const observer = new IntersectionObserver(entries => {
          entries.forEach(entry => {
            if (entry.isIntersecting) {
              hydrateIsland(el);
              observer.disconnect();
            }
          });
        });
        observer.observe(el);
        break;

      case 'interaction':
        const handler = () => {
          hydrateIsland(el);
          el.removeEventListener('click', handler);
          el.removeEventListener('focus', handler);
        };
        el.addEventListener('click', handler);
        el.addEventListener('focus', handler);
        break;

      case 'idle':
        if ('requestIdleCallback' in window) {
          requestIdleCallback(() => hydrateIsland(el));
        } else {
          setTimeout(() => hydrateIsland(el), 200);
        }
        break;

      case 'none':
        // Static island, no hydration
        break;
    }
  });
})();
"#.to_string()
    }
}

impl Default for IslandRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// HTML escape helper
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_cache() {
        let config = SsrConfig::default();
        let cache = ComponentCache::new(config);

        let result = RenderResult {
            html: "<div>Test</div>".to_string(),
            state: Some("{}".to_string()),
            dependencies: vec![],
            render_time_us: 100,
            cache_hit: false,
        };

        cache.store("test-key".to_string(), &result);

        let cached = cache.get("test-key");
        assert!(cached.is_some());
        assert!(cached.unwrap().cache_hit);
    }

    #[test]
    fn test_cache_key_generation() {
        let config = SsrConfig::default();
        let renderer = SsrRenderer::new(config);

        let mut props1 = HashMap::new();
        props1.insert("id".to_string(), "1".to_string());

        let mut props2 = HashMap::new();
        props2.insert("id".to_string(), "2".to_string());

        let key1 = renderer.cache_key("component", &props1);
        let key2 = renderer.cache_key("component", &props2);

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_streaming_renderer() {
        let config = SsrConfig::default();
        let renderer = StreamingRenderer::new(config);

        renderer.start("<html><body>");
        renderer.push_chunk("<div>Content</div>");
        renderer.complete("</body></html>");

        let chunks = renderer.get_chunks();
        assert_eq!(chunks.len(), 3);
        assert!(renderer.is_complete());
    }

    #[test]
    fn test_hydration_script() {
        let config = SsrConfig::default();
        let generator = HydrationScriptGenerator::new(config);

        let script = generator.generate_bootstrap();
        assert!(script.contains("__COMPONENT_STATE__"));
        assert!(script.contains("hydrateNext"));
    }

    #[test]
    fn test_island_placeholder() {
        let renderer = IslandRenderer::new();
        let island = Island {
            id: "counter-1".to_string(),
            component: "Counter".to_string(),
            props: HashMap::new(),
            hydrate: HydrationStrategy::Visible,
            slot: None,
        };

        let html = renderer.render_placeholder(&island, "<span>0</span>");
        assert!(html.contains("data-island=\"Counter\""));
        assert!(html.contains("data-hydrate=\"visible\""));
    }
}
