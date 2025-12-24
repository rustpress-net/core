//! # RustPress Events
//!
//! Event bus and messaging system for decoupled component communication.

pub mod bus;
pub mod event;
pub mod subscriber;

pub use bus::EventBus;
pub use event::{Event, EventType, DomainEvent};
pub use subscriber::{EventHandler, Subscriber};
