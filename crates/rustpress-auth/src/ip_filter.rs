//! IP Allowlist/Blocklist (Point 72)
//!
//! Provides IP address filtering with support for CIDR notation,
//! country-based blocking, and dynamic rules.

use chrono::{DateTime, Utc};
use rustpress_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::RwLock;
use uuid::Uuid;

/// IP filter rule type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IpRuleType {
    Allow,
    Block,
}

/// IP filter rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpRule {
    pub id: Uuid,
    pub rule_type: IpRuleType,
    pub pattern: IpPattern,
    pub reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub hit_count: u64,
    pub last_hit_at: Option<DateTime<Utc>>,
}

impl IpRule {
    pub fn is_active(&self) -> bool {
        match self.expires_at {
            Some(expires) => Utc::now() < expires,
            None => true,
        }
    }

    pub fn matches(&self, ip: &IpAddr) -> bool {
        self.is_active() && self.pattern.matches(ip)
    }
}

/// IP pattern for matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpPattern {
    /// Single IP address
    Single(IpAddr),
    /// CIDR range (e.g., 192.168.1.0/24)
    Cidr { network: IpAddr, prefix_len: u8 },
    /// IP range (start to end)
    Range { start: IpAddr, end: IpAddr },
    /// Wildcard pattern (e.g., 192.168.*.*)
    Wildcard(String),
    /// Country code (requires GeoIP lookup)
    Country(String),
    /// ASN (Autonomous System Number)
    Asn(u32),
}

impl IpPattern {
    pub fn single(ip: IpAddr) -> Self {
        Self::Single(ip)
    }

    pub fn cidr(network: IpAddr, prefix_len: u8) -> Self {
        Self::Cidr { network, prefix_len }
    }

    pub fn range(start: IpAddr, end: IpAddr) -> Self {
        Self::Range { start, end }
    }

    pub fn from_cidr_string(cidr: &str) -> Option<Self> {
        let parts: Vec<&str> = cidr.split('/').collect();
        if parts.len() != 2 {
            return None;
        }

        let network: IpAddr = parts[0].parse().ok()?;
        let prefix_len: u8 = parts[1].parse().ok()?;

        // Validate prefix length
        let max_prefix = match network {
            IpAddr::V4(_) => 32,
            IpAddr::V6(_) => 128,
        };
        if prefix_len > max_prefix {
            return None;
        }

        Some(Self::Cidr { network, prefix_len })
    }

    pub fn matches(&self, ip: &IpAddr) -> bool {
        match self {
            Self::Single(pattern_ip) => ip == pattern_ip,
            Self::Cidr { network, prefix_len } => self.matches_cidr(ip, network, *prefix_len),
            Self::Range { start, end } => self.matches_range(ip, start, end),
            Self::Wildcard(pattern) => self.matches_wildcard(ip, pattern),
            Self::Country(_) | Self::Asn(_) => false, // Requires external lookup
        }
    }

    fn matches_cidr(&self, ip: &IpAddr, network: &IpAddr, prefix_len: u8) -> bool {
        match (ip, network) {
            (IpAddr::V4(ip), IpAddr::V4(net)) => {
                let ip_bits = u32::from(*ip);
                let net_bits = u32::from(*net);
                let mask = if prefix_len == 0 { 0 } else { !0u32 << (32 - prefix_len) };
                (ip_bits & mask) == (net_bits & mask)
            }
            (IpAddr::V6(ip), IpAddr::V6(net)) => {
                let ip_bits = u128::from(*ip);
                let net_bits = u128::from(*net);
                let mask = if prefix_len == 0 { 0 } else { !0u128 << (128 - prefix_len) };
                (ip_bits & mask) == (net_bits & mask)
            }
            _ => false,
        }
    }

    fn matches_range(&self, ip: &IpAddr, start: &IpAddr, end: &IpAddr) -> bool {
        match (ip, start, end) {
            (IpAddr::V4(ip), IpAddr::V4(s), IpAddr::V4(e)) => {
                let ip_bits = u32::from(*ip);
                let start_bits = u32::from(*s);
                let end_bits = u32::from(*e);
                ip_bits >= start_bits && ip_bits <= end_bits
            }
            (IpAddr::V6(ip), IpAddr::V6(s), IpAddr::V6(e)) => {
                let ip_bits = u128::from(*ip);
                let start_bits = u128::from(*s);
                let end_bits = u128::from(*e);
                ip_bits >= start_bits && ip_bits <= end_bits
            }
            _ => false,
        }
    }

    fn matches_wildcard(&self, ip: &IpAddr, pattern: &str) -> bool {
        let ip_str = ip.to_string();
        let ip_parts: Vec<&str> = ip_str.split('.').collect();
        let pattern_parts: Vec<&str> = pattern.split('.').collect();

        if ip_parts.len() != pattern_parts.len() {
            return false;
        }

        for (ip_part, pattern_part) in ip_parts.iter().zip(pattern_parts.iter()) {
            if *pattern_part != "*" && *pattern_part != *ip_part {
                return false;
            }
        }

        true
    }
}

/// IP filter check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpCheckResult {
    pub allowed: bool,
    pub matched_rule: Option<IpRule>,
    pub reason: Option<String>,
}

impl IpCheckResult {
    pub fn allowed() -> Self {
        Self {
            allowed: true,
            matched_rule: None,
            reason: None,
        }
    }

    pub fn blocked(rule: IpRule) -> Self {
        Self {
            allowed: false,
            reason: rule.reason.clone(),
            matched_rule: Some(rule),
        }
    }
}

/// IP filter configuration
#[derive(Debug, Clone)]
pub struct IpFilterConfig {
    /// Default action when no rules match
    pub default_action: IpRuleType,
    /// Enable automatic rate-based blocking
    pub auto_block_enabled: bool,
    /// Threshold for automatic blocking
    pub auto_block_threshold: u32,
    /// Window for counting requests (seconds)
    pub auto_block_window_secs: u64,
    /// Duration for automatic blocks (seconds)
    pub auto_block_duration_secs: u64,
    /// Trusted proxy headers
    pub trusted_proxy_headers: Vec<String>,
    /// Maximum number of proxies to trust
    pub max_proxy_depth: usize,
}

impl Default for IpFilterConfig {
    fn default() -> Self {
        Self {
            default_action: IpRuleType::Allow,
            auto_block_enabled: false,
            auto_block_threshold: 100,
            auto_block_window_secs: 60,
            auto_block_duration_secs: 3600,
            trusted_proxy_headers: vec![
                "X-Forwarded-For".to_string(),
                "X-Real-IP".to_string(),
            ],
            max_proxy_depth: 1,
        }
    }
}

/// IP filter storage trait
#[async_trait::async_trait]
pub trait IpFilterStore: Send + Sync {
    async fn add_rule(&self, rule: &IpRule) -> Result<()>;
    async fn get_rule(&self, id: Uuid) -> Result<Option<IpRule>>;
    async fn get_all_rules(&self) -> Result<Vec<IpRule>>;
    async fn get_rules_by_type(&self, rule_type: IpRuleType) -> Result<Vec<IpRule>>;
    async fn update_rule(&self, rule: &IpRule) -> Result<()>;
    async fn delete_rule(&self, id: Uuid) -> Result<()>;
    async fn increment_hit_count(&self, id: Uuid) -> Result<()>;
    async fn cleanup_expired(&self) -> Result<u64>;
}

/// IP filter manager
pub struct IpFilter<S: IpFilterStore> {
    store: S,
    config: IpFilterConfig,
    /// Cached rules for performance
    rules_cache: RwLock<Option<(Vec<IpRule>, DateTime<Utc>)>>,
    cache_ttl_secs: i64,
}

impl<S: IpFilterStore> IpFilter<S> {
    pub fn new(store: S, config: IpFilterConfig) -> Self {
        Self {
            store,
            config,
            rules_cache: RwLock::new(None),
            cache_ttl_secs: 60,
        }
    }

    /// Check if an IP is allowed
    pub async fn check(&self, ip: &str) -> Result<IpCheckResult> {
        let parsed_ip: IpAddr = ip.parse().map_err(|_| Error::InvalidInput {
            field: "ip".to_string(),
            message: "Invalid IP address format".to_string(),
        })?;

        self.check_ip(&parsed_ip).await
    }

    /// Check if an IP address is allowed
    pub async fn check_ip(&self, ip: &IpAddr) -> Result<IpCheckResult> {
        let rules = self.get_rules_cached().await?;

        // Check block rules first
        for rule in rules.iter().filter(|r| r.rule_type == IpRuleType::Block) {
            if rule.matches(ip) {
                // Update hit count asynchronously
                let _ = self.store.increment_hit_count(rule.id).await;
                return Ok(IpCheckResult::blocked(rule.clone()));
            }
        }

        // Check allow rules
        let has_allow_rules = rules.iter().any(|r| r.rule_type == IpRuleType::Allow);
        if has_allow_rules {
            for rule in rules.iter().filter(|r| r.rule_type == IpRuleType::Allow) {
                if rule.matches(ip) {
                    let _ = self.store.increment_hit_count(rule.id).await;
                    return Ok(IpCheckResult::allowed());
                }
            }
            // If there are allow rules but none matched, block by default
            return Ok(IpCheckResult {
                allowed: false,
                matched_rule: None,
                reason: Some("IP not in allowlist".to_string()),
            });
        }

        // Apply default action
        Ok(IpCheckResult::allowed())
    }

    /// Get rules from cache or storage
    async fn get_rules_cached(&self) -> Result<Vec<IpRule>> {
        {
            let cache = self.rules_cache.read().map_err(|_| Error::Internal {
                message: "Lock poisoned".to_string(),
                request_id: None,
            })?;

            if let Some((rules, cached_at)) = &*cache {
                if Utc::now().signed_duration_since(*cached_at).num_seconds() < self.cache_ttl_secs {
                    return Ok(rules.clone());
                }
            }
        }

        // Refresh cache
        let rules = self.store.get_all_rules().await?;
        let active_rules: Vec<IpRule> = rules.into_iter().filter(|r| r.is_active()).collect();

        {
            let mut cache = self.rules_cache.write().map_err(|_| Error::Internal {
                message: "Lock poisoned".to_string(),
                request_id: None,
            })?;
            *cache = Some((active_rules.clone(), Utc::now()));
        }

        Ok(active_rules)
    }

    /// Invalidate cache
    pub fn invalidate_cache(&self) -> Result<()> {
        let mut cache = self.rules_cache.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        *cache = None;
        Ok(())
    }

    /// Add a block rule
    pub async fn block(
        &self,
        pattern: IpPattern,
        reason: Option<String>,
        expires_at: Option<DateTime<Utc>>,
        created_by: Option<Uuid>,
    ) -> Result<IpRule> {
        let rule = IpRule {
            id: Uuid::now_v7(),
            rule_type: IpRuleType::Block,
            pattern,
            reason,
            expires_at,
            created_at: Utc::now(),
            created_by,
            hit_count: 0,
            last_hit_at: None,
        };

        self.store.add_rule(&rule).await?;
        self.invalidate_cache()?;
        Ok(rule)
    }

    /// Add an allow rule
    pub async fn allow(
        &self,
        pattern: IpPattern,
        reason: Option<String>,
        created_by: Option<Uuid>,
    ) -> Result<IpRule> {
        let rule = IpRule {
            id: Uuid::now_v7(),
            rule_type: IpRuleType::Allow,
            pattern,
            reason,
            expires_at: None,
            created_at: Utc::now(),
            created_by,
            hit_count: 0,
            last_hit_at: None,
        };

        self.store.add_rule(&rule).await?;
        self.invalidate_cache()?;
        Ok(rule)
    }

    /// Block an IP temporarily
    pub async fn block_temporarily(
        &self,
        ip: &str,
        duration_secs: u64,
        reason: Option<String>,
    ) -> Result<IpRule> {
        let parsed_ip: IpAddr = ip.parse().map_err(|_| Error::InvalidInput {
            field: "ip".to_string(),
            message: "Invalid IP address format".to_string(),
        })?;

        let expires_at = Utc::now() + chrono::Duration::seconds(duration_secs as i64);
        self.block(IpPattern::Single(parsed_ip), reason, Some(expires_at), None).await
    }

    /// Block a CIDR range
    pub async fn block_cidr(
        &self,
        cidr: &str,
        reason: Option<String>,
        created_by: Option<Uuid>,
    ) -> Result<IpRule> {
        let pattern = IpPattern::from_cidr_string(cidr).ok_or_else(|| Error::InvalidInput {
            field: "cidr".to_string(),
            message: "Invalid CIDR notation".to_string(),
        })?;

        self.block(pattern, reason, None, created_by).await
    }

    /// Remove a rule
    pub async fn remove_rule(&self, id: Uuid) -> Result<()> {
        self.store.delete_rule(id).await?;
        self.invalidate_cache()?;
        Ok(())
    }

    /// Get all rules
    pub async fn get_rules(&self) -> Result<Vec<IpRule>> {
        self.store.get_all_rules().await
    }

    /// Cleanup expired rules
    pub async fn cleanup(&self) -> Result<u64> {
        let count = self.store.cleanup_expired().await?;
        if count > 0 {
            self.invalidate_cache()?;
        }
        Ok(count)
    }

    /// Extract client IP from headers
    pub fn extract_client_ip(&self, headers: &HashMap<String, String>, remote_addr: &str) -> String {
        for header_name in &self.config.trusted_proxy_headers {
            if let Some(value) = headers.get(header_name) {
                let ips: Vec<&str> = value.split(',').map(|s| s.trim()).collect();
                if !ips.is_empty() {
                    let depth = ips.len().saturating_sub(self.config.max_proxy_depth);
                    if let Some(ip) = ips.get(depth) {
                        return ip.to_string();
                    }
                }
            }
        }
        remote_addr.to_string()
    }

    /// Get config
    pub fn config(&self) -> &IpFilterConfig {
        &self.config
    }
}

/// In-memory IP filter store
pub struct InMemoryIpFilterStore {
    rules: RwLock<HashMap<Uuid, IpRule>>,
}

impl InMemoryIpFilterStore {
    pub fn new() -> Self {
        Self {
            rules: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryIpFilterStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl IpFilterStore for InMemoryIpFilterStore {
    async fn add_rule(&self, rule: &IpRule) -> Result<()> {
        let mut rules = self.rules.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        rules.insert(rule.id, rule.clone());
        Ok(())
    }

    async fn get_rule(&self, id: Uuid) -> Result<Option<IpRule>> {
        let rules = self.rules.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(rules.get(&id).cloned())
    }

    async fn get_all_rules(&self) -> Result<Vec<IpRule>> {
        let rules = self.rules.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(rules.values().cloned().collect())
    }

    async fn get_rules_by_type(&self, rule_type: IpRuleType) -> Result<Vec<IpRule>> {
        let rules = self.rules.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(rules.values().filter(|r| r.rule_type == rule_type).cloned().collect())
    }

    async fn update_rule(&self, rule: &IpRule) -> Result<()> {
        let mut rules = self.rules.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        rules.insert(rule.id, rule.clone());
        Ok(())
    }

    async fn delete_rule(&self, id: Uuid) -> Result<()> {
        let mut rules = self.rules.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        rules.remove(&id);
        Ok(())
    }

    async fn increment_hit_count(&self, id: Uuid) -> Result<()> {
        let mut rules = self.rules.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        if let Some(rule) = rules.get_mut(&id) {
            rule.hit_count += 1;
            rule.last_hit_at = Some(Utc::now());
        }
        Ok(())
    }

    async fn cleanup_expired(&self) -> Result<u64> {
        let mut rules = self.rules.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        let now = Utc::now();
        let before = rules.len();
        rules.retain(|_, r| {
            match r.expires_at {
                Some(expires) => expires > now,
                None => true,
            }
        });
        Ok((before - rules.len()) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cidr_matching() {
        let pattern = IpPattern::from_cidr_string("192.168.1.0/24").unwrap();

        assert!(pattern.matches(&"192.168.1.1".parse().unwrap()));
        assert!(pattern.matches(&"192.168.1.255".parse().unwrap()));
        assert!(!pattern.matches(&"192.168.2.1".parse().unwrap()));
    }

    #[test]
    fn test_single_ip_matching() {
        let pattern = IpPattern::Single("10.0.0.1".parse().unwrap());

        assert!(pattern.matches(&"10.0.0.1".parse().unwrap()));
        assert!(!pattern.matches(&"10.0.0.2".parse().unwrap()));
    }

    #[test]
    fn test_range_matching() {
        let pattern = IpPattern::Range {
            start: "192.168.1.10".parse().unwrap(),
            end: "192.168.1.20".parse().unwrap(),
        };

        assert!(pattern.matches(&"192.168.1.15".parse().unwrap()));
        assert!(!pattern.matches(&"192.168.1.5".parse().unwrap()));
        assert!(!pattern.matches(&"192.168.1.25".parse().unwrap()));
    }

    #[test]
    fn test_wildcard_matching() {
        let pattern = IpPattern::Wildcard("192.168.*.1".to_string());

        assert!(pattern.matches(&"192.168.1.1".parse().unwrap()));
        assert!(pattern.matches(&"192.168.2.1".parse().unwrap()));
        assert!(!pattern.matches(&"192.168.1.2".parse().unwrap()));
    }

    #[tokio::test]
    async fn test_ip_filter() {
        let store = InMemoryIpFilterStore::new();
        let filter = IpFilter::new(store, IpFilterConfig::default());

        // Block a specific IP
        filter.block(
            IpPattern::Single("1.2.3.4".parse().unwrap()),
            Some("Test block".to_string()),
            None,
            None,
        ).await.unwrap();

        // Check blocked IP
        let result = filter.check("1.2.3.4").await.unwrap();
        assert!(!result.allowed);

        // Check allowed IP
        let result = filter.check("5.6.7.8").await.unwrap();
        assert!(result.allowed);
    }

    #[tokio::test]
    async fn test_cidr_blocking() {
        let store = InMemoryIpFilterStore::new();
        let filter = IpFilter::new(store, IpFilterConfig::default());

        filter.block_cidr("10.0.0.0/8", Some("Private range".to_string()), None)
            .await
            .unwrap();

        let result = filter.check("10.1.2.3").await.unwrap();
        assert!(!result.allowed);

        let result = filter.check("11.1.2.3").await.unwrap();
        assert!(result.allowed);
    }
}
