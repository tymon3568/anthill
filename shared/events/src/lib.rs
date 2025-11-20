//! Shared Events Crate
//!
//! This crate provides event definitions and NATS client wrapper for event-driven
//! communication between microservices in the Anthill platform.
//!
//! ## Features
//!
//! - Event type definitions with serialization support
//! - NATS client wrapper with connection management
//! - Async publish/subscribe methods
//! - Error handling for event operations

pub mod events;
pub mod nats;

pub use events::*;
pub use nats::*;
