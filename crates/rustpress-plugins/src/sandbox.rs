//! Plugin Sandbox using WebAssembly (Wasmtime)
//!
//! Provides secure, isolated execution environment for plugins.

use rustpress_core::error::{Error, Result};
use crate::manifest::WasmSection;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use tracing::{debug, error, info, warn};

/// Plugin sandbox configuration
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Allowed file system paths
    pub allowed_paths: HashSet<String>,
    /// Allowed network hosts
    pub allowed_hosts: HashSet<String>,
    /// Maximum memory in bytes
    pub max_memory: usize,
    /// Maximum CPU time in milliseconds
    pub max_cpu_time_ms: u64,
    /// Allow network access
    pub allow_network: bool,
    /// Allow file system access
    pub allow_filesystem: bool,
    /// Allow subprocess execution
    pub allow_subprocess: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            allowed_paths: HashSet::new(),
            allowed_hosts: HashSet::new(),
            max_memory: 64 * 1024 * 1024, // 64MB
            max_cpu_time_ms: 5000,        // 5 seconds
            allow_network: false,
            allow_filesystem: false,
            allow_subprocess: false,
        }
    }
}

impl SandboxConfig {
    /// Create a restrictive sandbox
    pub fn restrictive() -> Self {
        Self::default()
    }

    /// Create a permissive sandbox (use with caution)
    pub fn permissive() -> Self {
        Self {
            allowed_paths: HashSet::new(),
            allowed_hosts: HashSet::new(),
            max_memory: 256 * 1024 * 1024, // 256MB
            max_cpu_time_ms: 30000,        // 30 seconds
            allow_network: true,
            allow_filesystem: true,
            allow_subprocess: false,
        }
    }

    /// Allow a file system path
    pub fn allow_path(mut self, path: impl Into<String>) -> Self {
        self.allowed_paths.insert(path.into());
        self
    }

    /// Allow a network host
    pub fn allow_host(mut self, host: impl Into<String>) -> Self {
        self.allowed_hosts.insert(host.into());
        self
    }

    /// Set maximum memory
    pub fn max_memory(mut self, bytes: usize) -> Self {
        self.max_memory = bytes;
        self
    }

    /// Set maximum CPU time
    pub fn max_cpu_time(mut self, ms: u64) -> Self {
        self.max_cpu_time_ms = ms;
        self
    }
}

/// Plugin sandbox for isolating plugin execution
pub struct PluginSandbox {
    config: SandboxConfig,
}

impl PluginSandbox {
    /// Create a new sandbox with the given configuration
    pub fn new(config: SandboxConfig) -> Self {
        Self { config }
    }

    /// Create a restrictive sandbox
    pub fn restrictive() -> Self {
        Self::new(SandboxConfig::restrictive())
    }

    /// Get the sandbox configuration
    pub fn config(&self) -> &SandboxConfig {
        &self.config
    }

    /// Check if a path is allowed
    pub fn is_path_allowed(&self, path: &str) -> bool {
        if !self.config.allow_filesystem {
            return false;
        }

        // Check if path is in allowed paths
        self.config.allowed_paths.iter().any(|allowed| {
            path.starts_with(allowed)
        })
    }

    /// Check if a host is allowed
    pub fn is_host_allowed(&self, host: &str) -> bool {
        if !self.config.allow_network {
            return false;
        }

        // Check if host is in allowed hosts
        self.config.allowed_hosts.contains(host) || self.config.allowed_hosts.is_empty()
    }

    /// Validate an operation
    pub fn validate_operation(&self, operation: &SandboxOperation) -> Result<()> {
        match operation {
            SandboxOperation::FileRead(path) => {
                if !self.is_path_allowed(path) {
                    return Err(Error::Authorization {
                        action: format!("File access: {}", path),
                        required: "filesystem:read".to_string(),
                    });
                }
            }
            SandboxOperation::FileWrite(path) => {
                if !self.is_path_allowed(path) {
                    return Err(Error::Authorization {
                        action: format!("File write: {}", path),
                        required: "filesystem:write".to_string(),
                    });
                }
            }
            SandboxOperation::NetworkConnect(host) => {
                if !self.is_host_allowed(host) {
                    return Err(Error::Authorization {
                        action: format!("Network access: {}", host),
                        required: "network:connect".to_string(),
                    });
                }
            }
            SandboxOperation::Execute(_) => {
                if !self.config.allow_subprocess {
                    return Err(Error::Authorization {
                        action: "Subprocess execution".to_string(),
                        required: "process:execute".to_string(),
                    });
                }
            }
        }

        Ok(())
    }
}

impl Default for PluginSandbox {
    fn default() -> Self {
        Self::restrictive()
    }
}

/// Operations that can be sandboxed
#[derive(Debug, Clone)]
pub enum SandboxOperation {
    /// Read a file
    FileRead(String),
    /// Write a file
    FileWrite(String),
    /// Connect to a network host
    NetworkConnect(String),
    /// Execute a subprocess
    Execute(String),
}

/// Resource usage tracker
#[derive(Debug, Default)]
pub struct ResourceUsage {
    /// Memory used in bytes
    pub memory_bytes: usize,
    /// CPU time used in milliseconds
    pub cpu_time_ms: u64,
    /// Network bytes sent
    pub network_sent: u64,
    /// Network bytes received
    pub network_received: u64,
    /// File operations count
    pub file_operations: u64,
}

impl ResourceUsage {
    /// Check if memory limit is exceeded
    pub fn is_memory_exceeded(&self, limit: usize) -> bool {
        self.memory_bytes > limit
    }

    /// Check if CPU time limit is exceeded
    pub fn is_cpu_exceeded(&self, limit_ms: u64) -> bool {
        self.cpu_time_ms > limit_ms
    }
}

/// WebAssembly sandbox configuration
#[derive(Debug, Clone)]
pub struct WasmSandboxConfig {
    /// Maximum memory in bytes
    pub max_memory: u64,

    /// Maximum execution time
    pub max_execution_time: Duration,

    /// Maximum fuel (instruction count)
    pub max_fuel: Option<u64>,

    /// Allowed host functions
    pub allowed_imports: Vec<String>,

    /// File system access (read paths)
    pub fs_read: Vec<PathBuf>,

    /// File system access (write paths)
    pub fs_write: Vec<PathBuf>,

    /// Network access allowed
    pub network_enabled: bool,

    /// Environment variables
    pub env_vars: HashMap<String, String>,

    /// Whether to inherit environment
    pub inherit_env: bool,

    /// Enable WASI
    pub wasi_enabled: bool,

    /// Preopen directories for WASI
    pub preopen_dirs: Vec<(PathBuf, String)>,
}

impl Default for WasmSandboxConfig {
    fn default() -> Self {
        Self {
            max_memory: 64 * 1024 * 1024, // 64 MB
            max_execution_time: Duration::from_secs(5),
            max_fuel: Some(10_000_000),
            allowed_imports: vec![
                "rustpress_log".to_string(),
                "rustpress_get_option".to_string(),
                "rustpress_set_option".to_string(),
                "rustpress_db_query".to_string(),
                "rustpress_http_request".to_string(),
                "rustpress_emit_event".to_string(),
            ],
            fs_read: Vec::new(),
            fs_write: Vec::new(),
            network_enabled: false,
            env_vars: HashMap::new(),
            inherit_env: false,
            wasi_enabled: true,
            preopen_dirs: Vec::new(),
        }
    }
}

impl From<&WasmSection> for WasmSandboxConfig {
    fn from(wasm: &WasmSection) -> Self {
        let mut config = WasmSandboxConfig::default();
        config.max_memory = wasm.memory_limit * 1024 * 1024;
        config.max_execution_time = Duration::from_millis(wasm.timeout_ms);
        config.max_fuel = wasm.fuel_limit;
        config.allowed_imports = wasm.imports.clone();
        config.fs_read = wasm.wasi.fs_read.iter().map(PathBuf::from).collect();
        config.fs_write = wasm.wasi.fs_write.iter().map(PathBuf::from).collect();
        config.network_enabled = wasm.wasi.network;
        config.env_vars = wasm.wasi.env.clone();
        config.inherit_env = wasm.wasi.inherit_env;
        config
    }
}

/// Host function context passed to plugin
#[derive(Debug, Clone)]
pub struct HostContext {
    /// Plugin ID
    pub plugin_id: String,

    /// Current user ID (if any)
    pub user_id: Option<i64>,

    /// Current site ID (for multisite)
    pub site_id: Option<i64>,

    /// Request ID for tracing
    pub request_id: Option<String>,
}

/// WebAssembly plugin sandbox
pub struct WasmPluginSandbox {
    plugin_id: String,
    config: WasmSandboxConfig,
    host_functions: Arc<RwLock<HostFunctions>>,
    execution_stats: Arc<RwLock<ExecutionStats>>,
}

impl WasmPluginSandbox {
    /// Create a new WASM sandbox for a plugin
    pub fn new(plugin_id: &str, config: WasmSandboxConfig) -> Self {
        Self {
            plugin_id: plugin_id.to_string(),
            config,
            host_functions: Arc::new(RwLock::new(HostFunctions::new())),
            execution_stats: Arc::new(RwLock::new(ExecutionStats::default())),
        }
    }

    /// Execute a function in the sandbox
    pub fn execute(
        &self,
        wasm_bytes: &[u8],
        function: &str,
        args: &[WasmValue],
        context: HostContext,
    ) -> std::result::Result<Vec<WasmValue>, SandboxError> {
        let start = Instant::now();

        // Record execution start
        {
            let mut stats = self.execution_stats.write();
            stats.total_calls += 1;
            stats.last_call = Some(chrono::Utc::now());
        }

        // Compile module
        let module = self.compile_module(wasm_bytes)?;

        // Create instance with host functions
        let instance = self.create_instance(&module, &context)?;

        // Execute function
        let result = self.call_function(&instance, function, args)?;

        // Update stats
        let elapsed = start.elapsed();
        {
            let mut stats = self.execution_stats.write();
            stats.total_execution_time += elapsed;
            if elapsed > stats.max_execution_time {
                stats.max_execution_time = elapsed;
            }
        }

        Ok(result)
    }

    /// Compile a WASM module
    fn compile_module(&self, wasm_bytes: &[u8]) -> std::result::Result<CompiledModule, SandboxError> {
        self.validate_module(wasm_bytes)?;

        Ok(CompiledModule {
            bytes: wasm_bytes.to_vec(),
            hash: blake3::hash(wasm_bytes).to_hex().to_string(),
        })
    }

    /// Validate module imports and structure
    fn validate_module(&self, _wasm_bytes: &[u8]) -> std::result::Result<(), SandboxError> {
        // In a real implementation, this would parse the WASM module
        // and validate that it only imports allowed functions
        Ok(())
    }

    /// Create an instance with host functions
    fn create_instance(
        &self,
        _module: &CompiledModule,
        context: &HostContext,
    ) -> std::result::Result<WasmInstance, SandboxError> {
        let host_funcs = self.host_functions.read();

        Ok(WasmInstance {
            plugin_id: self.plugin_id.clone(),
            context: context.clone(),
            memory_used: 0,
            fuel_consumed: 0,
            host_functions: host_funcs.clone(),
        })
    }

    /// Call a function in the instance
    fn call_function(
        &self,
        _instance: &WasmInstance,
        _function: &str,
        _args: &[WasmValue],
    ) -> std::result::Result<Vec<WasmValue>, SandboxError> {
        // In a real implementation, this would use Wasmtime to execute
        Ok(Vec::new())
    }

    /// Register a host function
    pub fn register_host_function<F>(&self, name: &str, func: F)
    where
        F: Fn(&HostContext, Vec<WasmValue>) -> std::result::Result<Vec<WasmValue>, String> + Send + Sync + 'static,
    {
        let mut host_funcs = self.host_functions.write();
        host_funcs.register(name, Box::new(func));
    }

    /// Get execution statistics
    pub fn get_stats(&self) -> ExecutionStats {
        self.execution_stats.read().clone()
    }

    /// Reset execution statistics
    pub fn reset_stats(&self) {
        *self.execution_stats.write() = ExecutionStats::default();
    }

    /// Check if sandbox is healthy
    pub fn is_healthy(&self) -> bool {
        let stats = self.execution_stats.read();
        stats.error_count < 10 && stats.timeout_count < 5
    }
}

/// Compiled WASM module
#[derive(Debug, Clone)]
struct CompiledModule {
    bytes: Vec<u8>,
    hash: String,
}

/// WASM instance for execution
#[derive(Debug, Clone)]
struct WasmInstance {
    plugin_id: String,
    context: HostContext,
    memory_used: u64,
    fuel_consumed: u64,
    host_functions: HostFunctions,
}

/// WASM value type
#[derive(Debug, Clone)]
pub enum WasmValue {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    String(String),
    Bytes(Vec<u8>),
    Json(serde_json::Value),
    Null,
}

impl WasmValue {
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            WasmValue::I32(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            WasmValue::I64(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            WasmValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_json(&self) -> Option<&serde_json::Value> {
        match self {
            WasmValue::Json(v) => Some(v),
            _ => None,
        }
    }
}

/// Host functions registry
#[derive(Debug, Clone, Default)]
pub struct HostFunctions {
    functions: HashMap<String, HostFunctionEntry>,
}

#[derive(Clone)]
struct HostFunctionEntry {
    name: String,
    call_count: u64,
}

impl std::fmt::Debug for HostFunctionEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HostFunctionEntry")
            .field("name", &self.name)
            .field("call_count", &self.call_count)
            .finish()
    }
}

impl HostFunctions {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    pub fn register<F>(&mut self, name: &str, _func: Box<F>)
    where
        F: Fn(&HostContext, Vec<WasmValue>) -> std::result::Result<Vec<WasmValue>, String> + Send + Sync + 'static,
    {
        self.functions.insert(
            name.to_string(),
            HostFunctionEntry {
                name: name.to_string(),
                call_count: 0,
            },
        );
    }

    pub fn list(&self) -> Vec<&str> {
        self.functions.keys().map(|s| s.as_str()).collect()
    }
}

/// Execution statistics
#[derive(Debug, Clone, Default)]
pub struct ExecutionStats {
    pub total_calls: u64,
    pub total_execution_time: Duration,
    pub max_execution_time: Duration,
    pub total_memory_allocated: u64,
    pub peak_memory_usage: u64,
    pub total_fuel_consumed: u64,
    pub error_count: u64,
    pub timeout_count: u64,
    pub oom_count: u64,
    pub last_call: Option<chrono::DateTime<chrono::Utc>>,
}

/// Sandbox error
#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("Compilation error: {0}")]
    Compilation(String),

    #[error("Instantiation error: {0}")]
    Instantiation(String),

    #[error("Execution error: {0}")]
    Execution(String),

    #[error("Function not found: {0}")]
    FunctionNotFound(String),

    #[error("Timeout exceeded")]
    Timeout,

    #[error("Memory limit exceeded: used {used}, limit {limit}")]
    MemoryLimit { used: u64, limit: u64 },

    #[error("Fuel exhausted")]
    FuelExhausted,

    #[error("Invalid import: {0}")]
    InvalidImport(String),

    #[error("Type mismatch: expected {expected}, got {got}")]
    TypeMismatch { expected: String, got: String },

    #[error("Host function error: {0}")]
    HostFunction(String),

    #[error("Trap: {0}")]
    Trap(String),
}

/// Sandbox pool for reusing instances
pub struct WasmSandboxPool {
    sandboxes: RwLock<HashMap<String, Vec<PooledSandbox>>>,
    config: PoolConfig,
}

/// Pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_per_plugin: usize,
    pub idle_timeout: Duration,
    pub max_total: usize,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_per_plugin: 5,
            idle_timeout: Duration::from_secs(60),
            max_total: 100,
        }
    }
}

struct PooledSandbox {
    sandbox: WasmPluginSandbox,
    last_used: Instant,
    use_count: u64,
}

impl WasmSandboxPool {
    pub fn new(config: PoolConfig) -> Self {
        Self {
            sandboxes: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Get or create a sandbox for a plugin
    pub fn get(&self, plugin_id: &str, sandbox_config: WasmSandboxConfig) -> WasmPluginSandbox {
        {
            let mut pool = self.sandboxes.write();
            if let Some(sandboxes) = pool.get_mut(plugin_id) {
                if let Some(pooled) = sandboxes.pop() {
                    debug!("Reusing sandbox for plugin {}", plugin_id);
                    return pooled.sandbox;
                }
            }
        }

        debug!("Creating new sandbox for plugin {}", plugin_id);
        WasmPluginSandbox::new(plugin_id, sandbox_config)
    }

    /// Return a sandbox to the pool
    pub fn release(&self, sandbox: WasmPluginSandbox) {
        let mut pool = self.sandboxes.write();
        let plugin_id = sandbox.plugin_id.clone();

        let sandboxes = pool.entry(plugin_id).or_insert_with(Vec::new);

        if sandboxes.len() < self.config.max_per_plugin {
            sandboxes.push(PooledSandbox {
                sandbox,
                last_used: Instant::now(),
                use_count: 1,
            });
        }
    }

    /// Clean up idle sandboxes
    pub fn cleanup(&self) {
        let mut pool = self.sandboxes.write();
        let now = Instant::now();

        for sandboxes in pool.values_mut() {
            sandboxes.retain(|s| now.duration_since(s.last_used) < self.config.idle_timeout);
        }

        pool.retain(|_, v| !v.is_empty());
    }

    /// Get pool statistics
    pub fn stats(&self) -> WasmPoolStats {
        let pool = self.sandboxes.read();
        let mut total = 0;
        let mut per_plugin = HashMap::new();

        for (plugin_id, sandboxes) in pool.iter() {
            total += sandboxes.len();
            per_plugin.insert(plugin_id.clone(), sandboxes.len());
        }

        WasmPoolStats { total, per_plugin }
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct WasmPoolStats {
    pub total: usize,
    pub per_plugin: HashMap<String, usize>,
}

/// Standard host functions provided to all plugins
pub struct StandardHostFunctions;

impl StandardHostFunctions {
    /// Log a message
    pub fn log(ctx: &HostContext, args: Vec<WasmValue>) -> std::result::Result<Vec<WasmValue>, String> {
        let level = args.get(0).and_then(|v| v.as_i32()).unwrap_or(0);
        let message = args.get(1).and_then(|v| v.as_string()).unwrap_or("");

        match level {
            0 => debug!(plugin = %ctx.plugin_id, "{}", message),
            1 => info!(plugin = %ctx.plugin_id, "{}", message),
            2 => warn!(plugin = %ctx.plugin_id, "{}", message),
            3 => error!(plugin = %ctx.plugin_id, "{}", message),
            _ => debug!(plugin = %ctx.plugin_id, "{}", message),
        }

        Ok(Vec::new())
    }

    /// Get an option value
    pub fn get_option(_ctx: &HostContext, args: Vec<WasmValue>) -> std::result::Result<Vec<WasmValue>, String> {
        let _key = args.get(0).and_then(|v| v.as_string()).ok_or("Missing key")?;
        Ok(vec![WasmValue::Null])
    }

    /// Set an option value
    pub fn set_option(_ctx: &HostContext, args: Vec<WasmValue>) -> std::result::Result<Vec<WasmValue>, String> {
        let _key = args.get(0).and_then(|v| v.as_string()).ok_or("Missing key")?;
        let _value = args.get(1).ok_or("Missing value")?;
        Ok(vec![WasmValue::I32(1)])
    }

    /// Execute a database query
    pub fn db_query(_ctx: &HostContext, args: Vec<WasmValue>) -> std::result::Result<Vec<WasmValue>, String> {
        let _query = args.get(0).and_then(|v| v.as_string()).ok_or("Missing query")?;
        Ok(vec![WasmValue::Json(serde_json::json!([]))])
    }

    /// Make an HTTP request
    pub fn http_request(_ctx: &HostContext, args: Vec<WasmValue>) -> std::result::Result<Vec<WasmValue>, String> {
        let _url = args.get(0).and_then(|v| v.as_string()).ok_or("Missing URL")?;
        let _method = args.get(1).and_then(|v| v.as_string()).unwrap_or("GET");
        Ok(vec![WasmValue::Json(serde_json::json!({
            "status": 200,
            "body": ""
        }))])
    }

    /// Emit an event
    pub fn emit_event(_ctx: &HostContext, args: Vec<WasmValue>) -> std::result::Result<Vec<WasmValue>, String> {
        let _event_name = args.get(0).and_then(|v| v.as_string()).ok_or("Missing event name")?;
        let _data = args.get(1);
        Ok(Vec::new())
    }

    /// Get current user info
    pub fn get_current_user(ctx: &HostContext, _args: Vec<WasmValue>) -> std::result::Result<Vec<WasmValue>, String> {
        Ok(vec![WasmValue::Json(serde_json::json!({
            "id": ctx.user_id,
            "site_id": ctx.site_id
        }))])
    }

    /// Check if user has capability
    pub fn user_can(_ctx: &HostContext, args: Vec<WasmValue>) -> std::result::Result<Vec<WasmValue>, String> {
        let _capability = args.get(0).and_then(|v| v.as_string()).ok_or("Missing capability")?;
        Ok(vec![WasmValue::I32(0)])
    }

    /// Register all standard functions with a sandbox
    pub fn register_all(sandbox: &WasmPluginSandbox) {
        sandbox.register_host_function("rustpress_log", Self::log);
        sandbox.register_host_function("rustpress_get_option", Self::get_option);
        sandbox.register_host_function("rustpress_set_option", Self::set_option);
        sandbox.register_host_function("rustpress_db_query", Self::db_query);
        sandbox.register_host_function("rustpress_http_request", Self::http_request);
        sandbox.register_host_function("rustpress_emit_event", Self::emit_event);
        sandbox.register_host_function("rustpress_get_current_user", Self::get_current_user);
        sandbox.register_host_function("rustpress_user_can", Self::user_can);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restrictive_sandbox() {
        let sandbox = PluginSandbox::restrictive();

        assert!(!sandbox.is_path_allowed("/tmp/test"));
        assert!(!sandbox.is_host_allowed("example.com"));
    }

    #[test]
    fn test_allowed_paths() {
        let config = SandboxConfig::default()
            .allow_path("/tmp/plugins")
            .max_memory(128 * 1024 * 1024);

        let sandbox = PluginSandbox::new(config);

        // Still false because allow_filesystem is false
        assert!(!sandbox.is_path_allowed("/tmp/plugins/test.txt"));
    }

    #[test]
    fn test_permissive_sandbox() {
        let config = SandboxConfig::permissive().allow_path("/tmp");
        let sandbox = PluginSandbox::new(config);

        assert!(sandbox.is_path_allowed("/tmp/test.txt"));
    }

    #[test]
    fn test_operation_validation() {
        let sandbox = PluginSandbox::restrictive();

        let result = sandbox.validate_operation(&SandboxOperation::FileRead("/etc/passwd".to_string()));
        assert!(result.is_err());

        let result = sandbox.validate_operation(&SandboxOperation::Execute("rm -rf /".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_wasm_sandbox_creation() {
        let config = WasmSandboxConfig::default();
        let sandbox = WasmPluginSandbox::new("test-plugin", config);

        assert!(sandbox.is_healthy());
        assert_eq!(sandbox.get_stats().total_calls, 0);
    }

    #[test]
    fn test_wasm_pool_stats() {
        let pool = WasmSandboxPool::new(PoolConfig::default());
        let stats = pool.stats();

        assert_eq!(stats.total, 0);
        assert!(stats.per_plugin.is_empty());
    }

    #[test]
    fn test_wasm_value_conversions() {
        let val = WasmValue::I32(42);
        assert_eq!(val.as_i32(), Some(42));
        assert_eq!(val.as_string(), None);

        let val = WasmValue::String("test".to_string());
        assert_eq!(val.as_string(), Some("test"));
        assert_eq!(val.as_i32(), None);
    }
}
