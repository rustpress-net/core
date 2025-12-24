//! Middleware traits and common middleware implementations.

use crate::context::RequestContext;
use crate::error::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// Request/Response type for middleware chain
#[derive(Debug, Clone)]
pub struct Request {
    pub context: RequestContext,
    pub body: Option<bytes::Bytes>,
    pub headers: std::collections::HashMap<String, String>,
}

impl Request {
    pub fn new(context: RequestContext) -> Self {
        Self {
            context,
            body: None,
            headers: std::collections::HashMap::new(),
        }
    }

    pub fn with_body(mut self, body: bytes::Bytes) -> Self {
        self.body = Some(body);
        self
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }
}

/// Response from middleware chain
#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub body: Option<bytes::Bytes>,
    pub headers: std::collections::HashMap<String, String>,
}

impl Response {
    pub fn new(status: u16) -> Self {
        Self {
            status,
            body: None,
            headers: std::collections::HashMap::new(),
        }
    }

    pub fn ok() -> Self {
        Self::new(200)
    }

    pub fn with_body(mut self, body: impl Into<bytes::Bytes>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn json<T: serde::Serialize>(data: &T) -> Result<Self> {
        let body = serde_json::to_vec(data)
            .map_err(|e| crate::error::Error::Serialization {
                message: e.to_string(),
            })?;

        Ok(Self::ok()
            .with_body(body)
            .with_header("content-type", "application/json"))
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::ok()
    }
}

/// Next function in the middleware chain
pub type Next = Arc<dyn Fn(Request) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response>> + Send>> + Send + Sync>;

/// Middleware trait for request/response processing
#[async_trait]
pub trait Middleware: Send + Sync {
    /// Process the request
    async fn handle(&self, request: Request, next: Next) -> Result<Response>;

    /// Middleware name for debugging
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

/// Middleware chain that executes middlewares in order
pub struct MiddlewareChain {
    middlewares: Vec<Arc<dyn Middleware>>,
}

impl MiddlewareChain {
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }

    pub fn add<M: Middleware + 'static>(&mut self, middleware: M) -> &mut Self {
        self.middlewares.push(Arc::new(middleware));
        self
    }

    pub fn add_arc(&mut self, middleware: Arc<dyn Middleware>) -> &mut Self {
        self.middlewares.push(middleware);
        self
    }

    pub async fn execute<F, Fut>(&self, request: Request, handler: F) -> Result<Response>
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Response>> + Send + 'static,
    {
        let handler = Arc::new(handler);
        self.execute_recursive(request, 0, handler).await
    }

    fn execute_recursive<F, Fut>(
        &self,
        request: Request,
        index: usize,
        handler: Arc<F>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response>> + Send>>
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Response>> + Send + 'static,
    {
        if index >= self.middlewares.len() {
            // End of chain, call the handler
            let handler = handler.clone();
            return Box::pin(async move { handler(request).await });
        }

        let middleware = self.middlewares[index].clone();
        let middlewares = self.middlewares.clone();
        let next_index = index + 1;

        Box::pin(async move {
            let next: Next = Arc::new(move |req| {
                let handler = handler.clone();
                let middlewares = middlewares.clone();
                Box::pin(async move {
                    let chain = MiddlewareChain { middlewares };
                    chain.execute_recursive(req, next_index, handler).await
                })
            });

            middleware.handle(request, next).await
        })
    }
}

impl Default for MiddlewareChain {
    fn default() -> Self {
        Self::new()
    }
}

/// Logging middleware
pub struct LoggingMiddleware {
    log_body: bool,
}

impl LoggingMiddleware {
    pub fn new() -> Self {
        Self { log_body: false }
    }

    pub fn with_body_logging(mut self) -> Self {
        self.log_body = true;
        self
    }
}

impl Default for LoggingMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn handle(&self, request: Request, next: Next) -> Result<Response> {
        let start = std::time::Instant::now();
        let method = request.context.method.clone();
        let path = request.context.path.clone();
        let request_id = request.context.request_id;

        tracing::info!(
            request_id = %request_id,
            method = %method,
            path = %path,
            "Request started"
        );

        let response = next(request).await;
        let duration = start.elapsed();

        match &response {
            Ok(resp) => {
                tracing::info!(
                    request_id = %request_id,
                    method = %method,
                    path = %path,
                    status = resp.status,
                    duration_ms = duration.as_millis() as u64,
                    "Request completed"
                );
            }
            Err(e) => {
                tracing::error!(
                    request_id = %request_id,
                    method = %method,
                    path = %path,
                    error = %e,
                    duration_ms = duration.as_millis() as u64,
                    "Request failed"
                );
            }
        }

        response
    }

    fn name(&self) -> &str {
        "LoggingMiddleware"
    }
}

/// Request ID middleware
pub struct RequestIdMiddleware;

#[async_trait]
impl Middleware for RequestIdMiddleware {
    async fn handle(&self, request: Request, next: Next) -> Result<Response> {
        let request_id = request.context.request_id.to_string();

        let mut response = next(request).await?;
        response.headers.insert("x-request-id".to_string(), request_id);

        Ok(response)
    }

    fn name(&self) -> &str {
        "RequestIdMiddleware"
    }
}

/// Timeout middleware
pub struct TimeoutMiddleware {
    timeout: std::time::Duration,
}

impl TimeoutMiddleware {
    pub fn new(timeout: std::time::Duration) -> Self {
        Self { timeout }
    }
}

#[async_trait]
impl Middleware for TimeoutMiddleware {
    async fn handle(&self, request: Request, next: Next) -> Result<Response> {
        match tokio::time::timeout(self.timeout, next(request)).await {
            Ok(result) => result,
            Err(_) => Err(crate::error::Error::Network {
                message: "Request timeout".to_string(),
                source: None,
            }),
        }
    }

    fn name(&self) -> &str {
        "TimeoutMiddleware"
    }
}

/// CORS middleware
pub struct CorsMiddleware {
    allowed_origins: Vec<String>,
    allowed_methods: Vec<String>,
    allowed_headers: Vec<String>,
    max_age: u64,
}

impl CorsMiddleware {
    pub fn new() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "PATCH".to_string(),
                "OPTIONS".to_string(),
            ],
            allowed_headers: vec!["*".to_string()],
            max_age: 86400,
        }
    }

    pub fn allow_origins(mut self, origins: Vec<String>) -> Self {
        self.allowed_origins = origins;
        self
    }

    pub fn allow_methods(mut self, methods: Vec<String>) -> Self {
        self.allowed_methods = methods;
        self
    }

    pub fn allow_headers(mut self, headers: Vec<String>) -> Self {
        self.allowed_headers = headers;
        self
    }
}

impl Default for CorsMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for CorsMiddleware {
    async fn handle(&self, request: Request, next: Next) -> Result<Response> {
        let mut response = next(request).await?;

        response.headers.insert(
            "access-control-allow-origin".to_string(),
            self.allowed_origins.join(", "),
        );
        response.headers.insert(
            "access-control-allow-methods".to_string(),
            self.allowed_methods.join(", "),
        );
        response.headers.insert(
            "access-control-allow-headers".to_string(),
            self.allowed_headers.join(", "),
        );
        response.headers.insert(
            "access-control-max-age".to_string(),
            self.max_age.to_string(),
        );

        Ok(response)
    }

    fn name(&self) -> &str {
        "CorsMiddleware"
    }
}

/// Compression middleware
pub struct CompressionMiddleware {
    min_size: usize,
}

impl CompressionMiddleware {
    pub fn new() -> Self {
        Self { min_size: 1024 }
    }

    pub fn min_size(mut self, size: usize) -> Self {
        self.min_size = size;
        self
    }
}

impl Default for CompressionMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for CompressionMiddleware {
    async fn handle(&self, request: Request, next: Next) -> Result<Response> {
        // In a real implementation, this would compress the response
        // For now, just pass through
        next(request).await
    }

    fn name(&self) -> &str {
        "CompressionMiddleware"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_middleware_chain() {
        let mut chain = MiddlewareChain::new();
        chain.add(RequestIdMiddleware);

        let request = Request::new(RequestContext::test());

        let response = chain
            .execute(request, |_req| async { Ok(Response::ok()) })
            .await
            .unwrap();

        assert!(response.headers.contains_key("x-request-id"));
    }

    #[tokio::test]
    async fn test_cors_middleware() {
        let mut chain = MiddlewareChain::new();
        chain.add(CorsMiddleware::new());

        let request = Request::new(RequestContext::test());

        let response = chain
            .execute(request, |_req| async { Ok(Response::ok()) })
            .await
            .unwrap();

        assert!(response.headers.contains_key("access-control-allow-origin"));
    }

    #[tokio::test]
    async fn test_timeout_middleware() {
        let mut chain = MiddlewareChain::new();
        chain.add(TimeoutMiddleware::new(std::time::Duration::from_millis(10)));

        let request = Request::new(RequestContext::test());

        let result = chain
            .execute(request, |_req| async {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                Ok(Response::ok())
            })
            .await;

        assert!(result.is_err());
    }
}
