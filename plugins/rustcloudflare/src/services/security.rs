//! Security management service (WAF, Firewall, Bot Management)

use crate::client::CloudflareClient;
use crate::error::{CloudflareError, CloudflareResult};
use crate::models::*;
use sqlx::PgPool;
use std::sync::Arc;

pub struct SecurityService {
    client: Option<Arc<CloudflareClient>>,
    #[allow(dead_code)]
    db: PgPool,
}

impl SecurityService {
    pub fn new(client: Arc<CloudflareClient>, db: PgPool) -> Self {
        Self { client: Some(client), db }
    }

    /// Create without a configured client (for initial setup)
    pub fn new_unconfigured(db: PgPool) -> Self {
        Self { client: None, db }
    }

    /// Get the client or return an error if not configured
    fn get_client(&self) -> CloudflareResult<&CloudflareClient> {
        self.client.as_ref()
            .map(|c| c.as_ref())
            .ok_or_else(|| CloudflareError::ConfigError("Cloudflare not configured. Please connect your account.".to_string()))
    }

    pub async fn get_security_level(&self) -> CloudflareResult<String> {
        let client = self.get_client()?;
        let setting = client.get_security_level().await?;
        Ok(setting.value.as_str().unwrap_or("medium").to_string())
    }

    pub async fn set_security_level(&self, level: &str) -> CloudflareResult<()> {
        let client = self.get_client()?;
        client.set_security_level(level).await?;
        Ok(())
    }

    pub async fn toggle_under_attack(&self, enabled: bool) -> CloudflareResult<()> {
        let client = self.get_client()?;
        client.toggle_under_attack_mode(enabled).await?;
        Ok(())
    }

    pub async fn list_waf_rules(&self) -> CloudflareResult<Vec<WafRule>> {
        let client = self.get_client()?;
        client.list_waf_rules().await
    }

    pub async fn list_firewall_rules(&self) -> CloudflareResult<Vec<FirewallRule>> {
        let client = self.get_client()?;
        client.list_firewall_rules().await
    }

    pub async fn create_firewall_rule(&self, rule: CreateFirewallRule) -> CloudflareResult<FirewallRule> {
        let client = self.get_client()?;
        client.create_firewall_rule(rule).await
    }

    pub async fn list_ip_access_rules(&self) -> CloudflareResult<Vec<IpAccessRule>> {
        let client = self.get_client()?;
        client.list_ip_access_rules().await
    }

    pub async fn block_ip(&self, ip: &str, note: Option<&str>) -> CloudflareResult<IpAccessRule> {
        let client = self.get_client()?;
        client.create_ip_access_rule(CreateIpAccessRule {
            mode: "block".to_string(),
            configuration: IpConfiguration {
                target: "ip".to_string(),
                value: ip.to_string(),
            },
            notes: note.map(|s| s.to_string()),
        }).await
    }

    pub async fn allow_ip(&self, ip: &str, note: Option<&str>) -> CloudflareResult<IpAccessRule> {
        let client = self.get_client()?;
        client.create_ip_access_rule(CreateIpAccessRule {
            mode: "whitelist".to_string(),
            configuration: IpConfiguration {
                target: "ip".to_string(),
                value: ip.to_string(),
            },
            notes: note.map(|s| s.to_string()),
        }).await
    }
}
