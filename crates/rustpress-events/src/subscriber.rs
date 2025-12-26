//! Event subscribers and handlers.

use crate::event::{DomainEvent, EventType};
use async_trait::async_trait;
use rustpress_core::error::Result;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Event handler function type
pub type HandlerFn = Arc<
    dyn Fn(Arc<DomainEvent>) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>
        + Send
        + Sync,
>;

/// Event handler trait
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle an event
    async fn handle(&self, event: Arc<DomainEvent>) -> Result<()>;

    /// Get the event types this handler is interested in
    fn event_types(&self) -> Vec<EventType>;

    /// Handler name for debugging
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// Whether this handler should receive events asynchronously
    fn is_async(&self) -> bool {
        false
    }
}

/// Subscriber configuration
#[derive(Debug, Clone)]
pub struct SubscriberConfig {
    /// Event types to subscribe to
    pub event_types: Vec<EventType>,
    /// Whether to run asynchronously
    pub async_handler: bool,
    /// Maximum retries on failure
    pub max_retries: u32,
    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
    /// Priority (higher = earlier execution)
    pub priority: i32,
}

impl Default for SubscriberConfig {
    fn default() -> Self {
        Self {
            event_types: Vec::new(),
            async_handler: false,
            max_retries: 3,
            retry_delay_ms: 1000,
            priority: 0,
        }
    }
}

impl SubscriberConfig {
    pub fn new(event_types: Vec<EventType>) -> Self {
        Self {
            event_types,
            ..Default::default()
        }
    }

    pub fn async_handler(mut self) -> Self {
        self.async_handler = true;
        self
    }

    pub fn with_retries(mut self, max_retries: u32, delay_ms: u64) -> Self {
        self.max_retries = max_retries;
        self.retry_delay_ms = delay_ms;
        self
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

/// Event subscriber
pub struct Subscriber {
    pub config: SubscriberConfig,
    pub handler: HandlerFn,
    pub name: String,
}

impl Subscriber {
    pub fn new<F, Fut>(name: impl Into<String>, config: SubscriberConfig, handler: F) -> Self
    where
        F: Fn(Arc<DomainEvent>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        Self {
            name: name.into(),
            config,
            handler: Arc::new(move |event| Box::pin(handler(event))),
        }
    }

    pub fn for_event<F, Fut>(event_type: impl Into<EventType>, handler: F) -> Self
    where
        F: Fn(Arc<DomainEvent>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let event_type = event_type.into();
        let name = format!("subscriber_{}", event_type);
        let config = SubscriberConfig::new(vec![event_type]);
        Self::new(name, config, handler)
    }

    pub fn for_events<F, Fut>(event_types: Vec<EventType>, handler: F) -> Self
    where
        F: Fn(Arc<DomainEvent>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let name = format!("subscriber_multi_{}", event_types.len());
        let config = SubscriberConfig::new(event_types);
        Self::new(name, config, handler)
    }

    /// Handle the event with retry logic
    pub async fn handle(&self, event: Arc<DomainEvent>) -> Result<()> {
        let mut attempts = 0;

        loop {
            match (self.handler)(event.clone()).await {
                Ok(()) => return Ok(()),
                Err(e) if attempts < self.config.max_retries => {
                    attempts += 1;
                    tracing::warn!(
                        subscriber = %self.name,
                        event_type = %event.event_type,
                        attempt = attempts,
                        error = %e,
                        "Event handler failed, retrying"
                    );
                    tokio::time::sleep(tokio::time::Duration::from_millis(
                        self.config.retry_delay_ms * attempts as u64,
                    ))
                    .await;
                }
                Err(e) => {
                    tracing::error!(
                        subscriber = %self.name,
                        event_type = %event.event_type,
                        attempts = attempts,
                        error = %e,
                        "Event handler failed after max retries"
                    );
                    return Err(e);
                }
            }
        }
    }

    /// Check if this subscriber handles the given event type
    pub fn handles(&self, event_type: &EventType) -> bool {
        self.config.event_types.iter().any(|et| et == event_type)
    }
}

/// Builder for creating subscribers
pub struct SubscriberBuilder {
    name: Option<String>,
    event_types: Vec<EventType>,
    async_handler: bool,
    max_retries: u32,
    retry_delay_ms: u64,
    priority: i32,
}

impl SubscriberBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            event_types: Vec::new(),
            async_handler: false,
            max_retries: 3,
            retry_delay_ms: 1000,
            priority: 0,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn event_type(mut self, event_type: impl Into<EventType>) -> Self {
        self.event_types.push(event_type.into());
        self
    }

    pub fn event_types(mut self, types: impl IntoIterator<Item = impl Into<EventType>>) -> Self {
        self.event_types.extend(types.into_iter().map(|t| t.into()));
        self
    }

    pub fn async_handler(mut self) -> Self {
        self.async_handler = true;
        self
    }

    pub fn retries(mut self, max: u32, delay_ms: u64) -> Self {
        self.max_retries = max;
        self.retry_delay_ms = delay_ms;
        self
    }

    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn build<F, Fut>(self, handler: F) -> Subscriber
    where
        F: Fn(Arc<DomainEvent>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let name = self.name.unwrap_or_else(|| {
            format!("subscriber_{}", self.event_types.len())
        });

        let config = SubscriberConfig {
            event_types: self.event_types,
            async_handler: self.async_handler,
            max_retries: self.max_retries,
            retry_delay_ms: self.retry_delay_ms,
            priority: self.priority,
        };

        Subscriber::new(name, config, handler)
    }
}

impl Default for SubscriberBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_subscriber_handles() {
        let sub = Subscriber::for_event("test.event", |_| async { Ok(()) });
        assert!(sub.handles(&EventType::new("test.event")));
        assert!(!sub.handles(&EventType::new("other.event")));
    }

    #[tokio::test]
    async fn test_subscriber_handle() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let sub = Subscriber::for_event("test.event", move |_| {
            let c = counter_clone.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        });

        let event = Arc::new(DomainEvent::new("test.event", serde_json::json!({})));
        sub.handle(event).await.unwrap();

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_subscriber_builder() {
        let sub = SubscriberBuilder::new()
            .name("test_subscriber")
            .event_type("event.one")
            .event_type("event.two")
            .priority(10)
            .build(|_| async { Ok(()) });

        assert_eq!(sub.name, "test_subscriber");
        assert_eq!(sub.config.event_types.len(), 2);
        assert_eq!(sub.config.priority, 10);
    }
}
