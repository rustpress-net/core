//! Event bus for publishing and subscribing to events.

use crate::event::{DomainEvent, EventType};
use crate::subscriber::Subscriber;
use dashmap::DashMap;
use parking_lot::RwLock;
use rustpress_core::error::{Error, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Event bus for decoupled component communication
pub struct EventBus {
    /// Subscribers grouped by event type
    subscribers: DashMap<EventType, Vec<Arc<Subscriber>>>,
    /// Broadcast channel for async subscribers
    broadcast_tx: broadcast::Sender<Arc<DomainEvent>>,
    /// Event history for replay (optional)
    history: Option<RwLock<Vec<Arc<DomainEvent>>>>,
    /// Configuration
    config: EventBusConfig,
}

/// Event bus configuration
#[derive(Debug, Clone)]
pub struct EventBusConfig {
    /// Maximum events in history
    pub max_history: usize,
    /// Enable event history
    pub enable_history: bool,
    /// Broadcast channel capacity
    pub broadcast_capacity: usize,
    /// Continue on handler error
    pub continue_on_error: bool,
}

impl Default for EventBusConfig {
    fn default() -> Self {
        Self {
            max_history: 1000,
            enable_history: false,
            broadcast_capacity: 1024,
            continue_on_error: true,
        }
    }
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        Self::with_config(EventBusConfig::default())
    }

    /// Create an event bus with custom configuration
    pub fn with_config(config: EventBusConfig) -> Self {
        let (broadcast_tx, _) = broadcast::channel(config.broadcast_capacity);
        let history = if config.enable_history {
            Some(RwLock::new(Vec::new()))
        } else {
            None
        };

        Self {
            subscribers: DashMap::new(),
            broadcast_tx,
            history,
            config,
        }
    }

    /// Subscribe to events
    pub fn subscribe(&self, subscriber: Subscriber) -> &Self {
        let subscriber = Arc::new(subscriber);

        for event_type in &subscriber.config.event_types {
            self.subscribers
                .entry(event_type.clone())
                .or_insert_with(Vec::new)
                .push(subscriber.clone());
        }

        // Sort by priority (higher priority first)
        for mut entry in self.subscribers.iter_mut() {
            entry.value_mut().sort_by(|a, b| {
                b.config.priority.cmp(&a.config.priority)
            });
        }

        self
    }

    /// Unsubscribe by subscriber name
    pub fn unsubscribe(&self, name: &str) {
        for mut entry in self.subscribers.iter_mut() {
            entry.value_mut().retain(|s| s.name != name);
        }
    }

    /// Publish an event
    pub async fn publish(&self, event: DomainEvent) -> Result<()> {
        let event = Arc::new(event);

        tracing::debug!(
            event_type = %event.event_type,
            event_id = %event.id,
            "Publishing event"
        );

        // Add to history if enabled
        if let Some(history) = &self.history {
            let mut h = history.write();
            h.push(event.clone());
            if h.len() > self.config.max_history {
                h.remove(0);
            }
        }

        // Get subscribers for this event type
        let event_type = EventType::new(&event.event_type);
        let subscribers = self
            .subscribers
            .get(&event_type)
            .map(|s| s.clone())
            .unwrap_or_default();

        // Sync subscribers
        let mut errors = Vec::new();
        for subscriber in subscribers.iter().filter(|s| !s.config.async_handler) {
            if let Err(e) = subscriber.handle(event.clone()).await {
                tracing::error!(
                    subscriber = %subscriber.name,
                    event_type = %event.event_type,
                    error = %e,
                    "Sync event handler failed"
                );
                if !self.config.continue_on_error {
                    return Err(e);
                }
                errors.push(e);
            }
        }

        // Async subscribers via broadcast
        let async_subscribers: Vec<_> = subscribers
            .iter()
            .filter(|s| s.config.async_handler)
            .cloned()
            .collect();

        if !async_subscribers.is_empty() {
            let event_clone = event.clone();
            tokio::spawn(async move {
                for subscriber in async_subscribers {
                    if let Err(e) = subscriber.handle(event_clone.clone()).await {
                        tracing::error!(
                            subscriber = %subscriber.name,
                            error = %e,
                            "Async event handler failed"
                        );
                    }
                }
            });
        }

        // Broadcast for external listeners
        let _ = self.broadcast_tx.send(event);

        if errors.is_empty() {
            Ok(())
        } else {
            tracing::warn!(
                error_count = errors.len(),
                "Some event handlers failed"
            );
            Ok(()) // We continue on error by default
        }
    }

    /// Publish multiple events
    pub async fn publish_all(&self, events: Vec<DomainEvent>) -> Result<()> {
        for event in events {
            self.publish(event).await?;
        }
        Ok(())
    }

    /// Get a broadcast receiver for external listeners
    pub fn subscribe_broadcast(&self) -> broadcast::Receiver<Arc<DomainEvent>> {
        self.broadcast_tx.subscribe()
    }

    /// Get event history (if enabled)
    pub fn history(&self) -> Vec<Arc<DomainEvent>> {
        self.history
            .as_ref()
            .map(|h| h.read().clone())
            .unwrap_or_default()
    }

    /// Get history for a specific event type
    pub fn history_for(&self, event_type: &str) -> Vec<Arc<DomainEvent>> {
        self.history()
            .into_iter()
            .filter(|e| e.event_type == event_type)
            .collect()
    }

    /// Clear event history
    pub fn clear_history(&self) {
        if let Some(history) = &self.history {
            history.write().clear();
        }
    }

    /// Get subscriber count for an event type
    pub fn subscriber_count(&self, event_type: &EventType) -> usize {
        self.subscribers
            .get(event_type)
            .map(|s| s.len())
            .unwrap_or(0)
    }

    /// Get all registered event types
    pub fn event_types(&self) -> Vec<EventType> {
        self.subscribers.iter().map(|e| e.key().clone()).collect()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for EventBus
pub struct EventBusBuilder {
    config: EventBusConfig,
    subscribers: Vec<Subscriber>,
}

impl EventBusBuilder {
    pub fn new() -> Self {
        Self {
            config: EventBusConfig::default(),
            subscribers: Vec::new(),
        }
    }

    pub fn with_history(mut self, max_size: usize) -> Self {
        self.config.enable_history = true;
        self.config.max_history = max_size;
        self
    }

    pub fn broadcast_capacity(mut self, capacity: usize) -> Self {
        self.config.broadcast_capacity = capacity;
        self
    }

    pub fn continue_on_error(mut self, continue_on_error: bool) -> Self {
        self.config.continue_on_error = continue_on_error;
        self
    }

    pub fn subscriber(mut self, subscriber: Subscriber) -> Self {
        self.subscribers.push(subscriber);
        self
    }

    pub fn build(self) -> EventBus {
        let bus = EventBus::with_config(self.config);
        for subscriber in self.subscribers {
            bus.subscribe(subscriber);
        }
        bus
    }
}

impl Default for EventBusBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_publish_subscribe() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        bus.subscribe(Subscriber::for_event("test.event", move |_| {
            let c = counter_clone.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }));

        let event = DomainEvent::new("test.event", serde_json::json!({}));
        bus.publish(event).await.unwrap();

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicU32::new(0));

        for _ in 0..3 {
            let c = counter.clone();
            bus.subscribe(Subscriber::for_event("test.event", move |_| {
                let c = c.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            }));
        }

        let event = DomainEvent::new("test.event", serde_json::json!({}));
        bus.publish(event).await.unwrap();

        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_event_history() {
        let bus = EventBusBuilder::new()
            .with_history(10)
            .build();

        for i in 0..5 {
            let event = DomainEvent::new("test.event", serde_json::json!({"i": i}));
            bus.publish(event).await.unwrap();
        }

        let history = bus.history();
        assert_eq!(history.len(), 5);
    }

    #[tokio::test]
    async fn test_unsubscribe() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let subscriber = Subscriber::new(
            "test_sub",
            crate::subscriber::SubscriberConfig::new(vec![EventType::new("test.event")]),
            move |_| {
                let c = counter_clone.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            },
        );

        bus.subscribe(subscriber);

        // First event should be handled
        bus.publish(DomainEvent::new("test.event", serde_json::json!({}))).await.unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        // Unsubscribe
        bus.unsubscribe("test_sub");

        // Second event should not be handled
        bus.publish(DomainEvent::new("test.event", serde_json::json!({}))).await.unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_broadcast_receiver() {
        let bus = EventBus::new();
        let mut receiver = bus.subscribe_broadcast();

        let event = DomainEvent::new("test.event", serde_json::json!({"key": "value"}));
        bus.publish(event).await.unwrap();

        let received = receiver.recv().await.unwrap();
        assert_eq!(received.event_type, "test.event");
    }
}
