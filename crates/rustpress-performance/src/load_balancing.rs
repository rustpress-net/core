//! Load Balancing Session Handling
//!
//! Provides session management and sticky session support for load-balanced environments.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Session cookie name
    pub cookie_name: String,
    /// Session timeout in seconds
    pub timeout_seconds: u64,
    /// Enable sticky sessions
    pub sticky_sessions: bool,
    /// Session affinity cookie name
    pub affinity_cookie: String,
    /// Maximum sessions per user
    pub max_sessions_per_user: usize,
    /// Enable session replication
    pub replicate_sessions: bool,
    /// Session store type
    pub store_type: SessionStoreType,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            cookie_name: "RSESSID".to_string(),
            timeout_seconds: 1800, // 30 minutes
            sticky_sessions: true,
            affinity_cookie: "SERVERID".to_string(),
            max_sessions_per_user: 5,
            replicate_sessions: true,
            store_type: SessionStoreType::Memory,
        }
    }
}

/// Session store type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStoreType {
    Memory,
    Redis,
    Database,
    Memcached,
}

/// Session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session ID
    pub id: String,
    /// User ID (if authenticated)
    pub user_id: Option<i64>,
    /// Session data
    pub data: HashMap<String, serde_json::Value>,
    /// Creation time
    pub created_at: i64,
    /// Last access time
    pub last_accessed: i64,
    /// Server ID that owns this session
    pub server_id: Option<String>,
    /// IP address
    pub ip_address: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
}

impl Session {
    pub fn new(id: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id,
            user_id: None,
            data: HashMap::new(),
            created_at: now,
            last_accessed: now,
            server_id: None,
            ip_address: None,
            user_agent: None,
        }
    }

    /// Check if session is expired
    pub fn is_expired(&self, timeout_seconds: u64) -> bool {
        let now = chrono::Utc::now().timestamp();
        (now - self.last_accessed) as u64 > timeout_seconds
    }

    /// Touch session (update last accessed)
    pub fn touch(&mut self) {
        self.last_accessed = chrono::Utc::now().timestamp();
    }

    /// Get a value from session
    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.data.get(key).and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Set a value in session
    pub fn set<T: Serialize>(&mut self, key: &str, value: T) {
        if let Ok(v) = serde_json::to_value(value) {
            self.data.insert(key.to_string(), v);
        }
    }

    /// Remove a value from session
    pub fn remove(&mut self, key: &str) -> Option<serde_json::Value> {
        self.data.remove(key)
    }
}

/// In-memory session store
pub struct MemorySessionStore {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    config: SessionConfig,
}

impl MemorySessionStore {
    pub fn new(config: SessionConfig) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Get a session
    pub fn get(&self, id: &str) -> Option<Session> {
        let sessions = self.sessions.read();
        sessions.get(id).cloned()
    }

    /// Save a session
    pub fn save(&self, session: Session) {
        self.sessions.write().insert(session.id.clone(), session);
    }

    /// Delete a session
    pub fn delete(&self, id: &str) {
        self.sessions.write().remove(id);
    }

    /// Clean expired sessions
    pub fn cleanup(&self) -> usize {
        let mut sessions = self.sessions.write();
        let before = sessions.len();

        sessions.retain(|_, s| !s.is_expired(self.config.timeout_seconds));

        before - sessions.len()
    }

    /// Get session count
    pub fn count(&self) -> usize {
        self.sessions.read().len()
    }

    /// Get sessions for a user
    pub fn get_user_sessions(&self, user_id: i64) -> Vec<Session> {
        self.sessions.read()
            .values()
            .filter(|s| s.user_id == Some(user_id))
            .cloned()
            .collect()
    }

    /// Delete all sessions for a user
    pub fn delete_user_sessions(&self, user_id: i64) -> usize {
        let mut sessions = self.sessions.write();
        let before = sessions.len();

        sessions.retain(|_, s| s.user_id != Some(user_id));

        before - sessions.len()
    }
}

/// Session ID generator
pub fn generate_session_id() -> String {
    use rand::Rng;

    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    hex::encode(bytes)
}

/// Server node in load balancer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerNode {
    /// Server identifier
    pub id: String,
    /// Server address
    pub address: String,
    /// Server weight (for weighted round-robin)
    pub weight: u32,
    /// Server is healthy
    pub healthy: bool,
    /// Current active connections
    pub active_connections: u32,
    /// Total requests handled
    pub total_requests: u64,
    /// Last health check time
    pub last_health_check: i64,
}

impl ServerNode {
    pub fn new(id: &str, address: &str) -> Self {
        Self {
            id: id.to_string(),
            address: address.to_string(),
            weight: 1,
            healthy: true,
            active_connections: 0,
            total_requests: 0,
            last_health_check: chrono::Utc::now().timestamp(),
        }
    }
}

/// Load balancing strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    /// Round-robin
    RoundRobin,
    /// Weighted round-robin
    WeightedRoundRobin,
    /// Least connections
    LeastConnections,
    /// IP hash (sticky)
    IpHash,
    /// Random
    Random,
    /// Least response time
    LeastResponseTime,
}

/// Load balancer
pub struct LoadBalancer {
    nodes: Arc<RwLock<Vec<ServerNode>>>,
    strategy: LoadBalancingStrategy,
    current_index: Arc<RwLock<usize>>,
    session_affinity: Arc<RwLock<HashMap<String, String>>>,
}

impl LoadBalancer {
    pub fn new(strategy: LoadBalancingStrategy) -> Self {
        Self {
            nodes: Arc::new(RwLock::new(Vec::new())),
            strategy,
            current_index: Arc::new(RwLock::new(0)),
            session_affinity: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a server node
    pub fn add_node(&self, node: ServerNode) {
        self.nodes.write().push(node);
    }

    /// Remove a server node
    pub fn remove_node(&self, id: &str) {
        self.nodes.write().retain(|n| n.id != id);
    }

    /// Get next server based on strategy
    pub fn next(&self, client_ip: Option<&str>, session_id: Option<&str>) -> Option<ServerNode> {
        // Check session affinity first
        if let Some(sid) = session_id {
            let affinity = self.session_affinity.read();
            if let Some(node_id) = affinity.get(sid) {
                let nodes = self.nodes.read();
                if let Some(node) = nodes.iter().find(|n| &n.id == node_id && n.healthy) {
                    return Some(node.clone());
                }
            }
        }

        let nodes = self.nodes.read();
        let healthy_nodes: Vec<&ServerNode> = nodes.iter().filter(|n| n.healthy).collect();

        if healthy_nodes.is_empty() {
            return None;
        }

        let selected = match self.strategy {
            LoadBalancingStrategy::RoundRobin => {
                let mut idx = self.current_index.write();
                let node = &healthy_nodes[*idx % healthy_nodes.len()];
                *idx = (*idx + 1) % healthy_nodes.len();
                node
            }
            LoadBalancingStrategy::WeightedRoundRobin => {
                self.weighted_round_robin(&healthy_nodes)
            }
            LoadBalancingStrategy::LeastConnections => {
                healthy_nodes.iter()
                    .min_by_key(|n| n.active_connections)
                    .unwrap()
            }
            LoadBalancingStrategy::IpHash => {
                if let Some(ip) = client_ip {
                    let hash = self.hash_string(ip);
                    &healthy_nodes[hash % healthy_nodes.len()]
                } else {
                    &healthy_nodes[0]
                }
            }
            LoadBalancingStrategy::Random => {
                use rand::Rng;
                let idx = rand::thread_rng().gen_range(0..healthy_nodes.len());
                &healthy_nodes[idx]
            }
            LoadBalancingStrategy::LeastResponseTime => {
                // In a real implementation, this would track response times
                healthy_nodes.iter()
                    .min_by_key(|n| n.active_connections)
                    .unwrap()
            }
        };

        // Set session affinity
        if let Some(sid) = session_id {
            self.session_affinity.write().insert(sid.to_string(), selected.id.clone());
        }

        Some((*selected).clone())
    }

    fn weighted_round_robin<'a>(&self, nodes: &[&'a ServerNode]) -> &'a ServerNode {
        let total_weight: u32 = nodes.iter().map(|n| n.weight).sum();

        use rand::Rng;
        let mut point = rand::thread_rng().gen_range(0..total_weight);

        for node in nodes {
            if point < node.weight {
                return node;
            }
            point -= node.weight;
        }

        nodes[0]
    }

    fn hash_string(&self, s: &str) -> usize {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish() as usize
    }

    /// Mark a node as healthy/unhealthy
    pub fn set_node_health(&self, id: &str, healthy: bool) {
        let mut nodes = self.nodes.write();
        if let Some(node) = nodes.iter_mut().find(|n| n.id == id) {
            node.healthy = healthy;
            node.last_health_check = chrono::Utc::now().timestamp();
        }
    }

    /// Record request to a node
    pub fn record_request(&self, node_id: &str) {
        let mut nodes = self.nodes.write();
        if let Some(node) = nodes.iter_mut().find(|n| n.id == node_id) {
            node.total_requests += 1;
            node.active_connections += 1;
        }
    }

    /// Record request completion
    pub fn record_completion(&self, node_id: &str) {
        let mut nodes = self.nodes.write();
        if let Some(node) = nodes.iter_mut().find(|n| n.id == node_id) {
            if node.active_connections > 0 {
                node.active_connections -= 1;
            }
        }
    }

    /// Get all nodes
    pub fn nodes(&self) -> Vec<ServerNode> {
        self.nodes.read().clone()
    }

    /// Get healthy nodes count
    pub fn healthy_count(&self) -> usize {
        self.nodes.read().iter().filter(|n| n.healthy).count()
    }
}

/// Health checker for server nodes
pub struct HealthChecker {
    interval: Duration,
    timeout: Duration,
    unhealthy_threshold: u32,
    healthy_threshold: u32,
    check_path: String,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            interval: Duration::from_secs(10),
            timeout: Duration::from_secs(5),
            unhealthy_threshold: 3,
            healthy_threshold: 2,
            check_path: "/health".to_string(),
        }
    }

    /// Set health check interval
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    /// Set health check timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set check path
    pub fn with_path(mut self, path: &str) -> Self {
        self.check_path = path.to_string();
        self
    }

    /// Check health of a node (mock implementation)
    pub async fn check(&self, node: &ServerNode) -> bool {
        // In a real implementation, this would make an HTTP request
        // to the node's health check endpoint
        tracing::debug!("Health check for node {}: {}{}", node.id, node.address, self.check_path);
        true
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Session cookie generator
pub struct SessionCookie {
    pub name: String,
    pub value: String,
    pub path: String,
    pub domain: Option<String>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: SameSite,
    pub max_age: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

impl SameSite {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Strict => "Strict",
            Self::Lax => "Lax",
            Self::None => "None",
        }
    }
}

impl SessionCookie {
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
            path: "/".to_string(),
            domain: None,
            secure: true,
            http_only: true,
            same_site: SameSite::Lax,
            max_age: None,
        }
    }

    /// Generate Set-Cookie header value
    pub fn to_header(&self) -> String {
        let mut parts = vec![format!("{}={}", self.name, self.value)];

        parts.push(format!("Path={}", self.path));

        if let Some(ref domain) = self.domain {
            parts.push(format!("Domain={}", domain));
        }

        if self.secure {
            parts.push("Secure".to_string());
        }

        if self.http_only {
            parts.push("HttpOnly".to_string());
        }

        parts.push(format!("SameSite={}", self.same_site.as_str()));

        if let Some(max_age) = self.max_age {
            parts.push(format!("Max-Age={}", max_age));
        }

        parts.join("; ")
    }
}

/// Server affinity cookie generator
pub fn generate_affinity_cookie(node_id: &str, cookie_name: &str) -> SessionCookie {
    SessionCookie {
        name: cookie_name.to_string(),
        value: node_id.to_string(),
        path: "/".to_string(),
        domain: None,
        secure: true,
        http_only: true,
        same_site: SameSite::Lax,
        max_age: Some(86400), // 1 day
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let mut session = Session::new(generate_session_id());
        session.set("user_name", "test_user");

        let name: Option<String> = session.get("user_name");
        assert_eq!(name, Some("test_user".to_string()));
    }

    #[test]
    fn test_session_expiry() {
        let mut session = Session::new("test".to_string());
        session.last_accessed = chrono::Utc::now().timestamp() - 3600;

        assert!(session.is_expired(1800));
        assert!(!session.is_expired(7200));
    }

    #[test]
    fn test_load_balancer_round_robin() {
        let lb = LoadBalancer::new(LoadBalancingStrategy::RoundRobin);

        lb.add_node(ServerNode::new("server1", "http://server1:8080"));
        lb.add_node(ServerNode::new("server2", "http://server2:8080"));

        let node1 = lb.next(None, None).unwrap();
        let node2 = lb.next(None, None).unwrap();

        assert_ne!(node1.id, node2.id);
    }

    #[test]
    fn test_load_balancer_ip_hash() {
        let lb = LoadBalancer::new(LoadBalancingStrategy::IpHash);

        lb.add_node(ServerNode::new("server1", "http://server1:8080"));
        lb.add_node(ServerNode::new("server2", "http://server2:8080"));

        let node1 = lb.next(Some("192.168.1.1"), None).unwrap();
        let node2 = lb.next(Some("192.168.1.1"), None).unwrap();

        // Same IP should always go to same server
        assert_eq!(node1.id, node2.id);
    }

    #[test]
    fn test_session_cookie() {
        let cookie = SessionCookie::new("RSESSID", "abc123");
        let header = cookie.to_header();

        assert!(header.contains("RSESSID=abc123"));
        assert!(header.contains("Secure"));
        assert!(header.contains("HttpOnly"));
        assert!(header.contains("SameSite=Lax"));
    }
}
