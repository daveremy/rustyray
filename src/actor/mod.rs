//! Actor system implementation for RustyRay.
//! 
//! This module implements Ray's actor model in Rust. Actors are stateful
//! workers that process messages sequentially. Each actor:
//! 
//! - Has a unique ID for addressing
//! - Maintains internal state
//! - Processes messages one at a time (by default)
//! - Can be created, called, and destroyed
//! 
//! # Example
//! 
//! ```ignore
//! use rustyray::actor::{Actor, ActorSystem};
//! use rustyray::error::Result;
//! use async_trait::async_trait;
//! use std::any::Any;
//! 
//! struct Counter {
//!     count: i32,
//! }
//! 
//! #[async_trait]
//! impl Actor for Counter {
//!     async fn handle(&mut self, msg: Box<dyn Any + Send>) -> Result<Box<dyn Any + Send>> {
//!         // Handle messages here
//!         Ok(Box::new(()))
//!     }
//! }
//! ```

use crate::error::Result;
use crate::types::ActorId;
use std::any::Any;
use async_trait::async_trait;

/// The base trait that all actors must implement.
/// 
/// This trait defines how actors handle messages. In Ray's model, actors
/// can receive different types of messages and return different types of
/// results, so we use type erasure with `Any`.
#[async_trait]
pub trait Actor: Send + Sync + 'static {
    /// Handle an incoming message.
    /// 
    /// Messages are type-erased as `Box<dyn Any>`. Actors should downcast
    /// to expected message types and return an appropriate response.
    /// 
    /// # Arguments
    /// 
    /// * `msg` - The message to handle, type-erased as `Any`
    /// 
    /// # Returns
    /// 
    /// A type-erased response or an error if the message couldn't be handled
    async fn handle(&mut self, msg: Box<dyn Any + Send>) -> Result<Box<dyn Any + Send>>;
    
    /// Called when the actor is started.
    /// 
    /// This is useful for initialization that requires async operations.
    /// Default implementation does nothing.
    async fn on_start(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Called when the actor is about to stop.
    /// 
    /// This is useful for cleanup operations.
    /// Default implementation does nothing.
    async fn on_stop(&mut self) -> Result<()> {
        Ok(())
    }
}

/// A handle to an actor that can be used to send messages.
/// 
/// This is similar to Ray's ActorHandle. It provides location transparency -
/// the actor could be local or remote (in future versions).
pub struct ActorRef {
    /// The unique ID of this actor
    id: ActorId,
    
    // TODO: Add channel for sending messages
    // TODO: Add metadata (name, owner, etc.)
}

impl ActorRef {
    /// Get the actor's ID.
    pub fn id(&self) -> ActorId {
        self.id
    }
    
    /// Send a one-way message to the actor.
    /// 
    /// This is "fire and forget" - we don't wait for a response.
    pub async fn send(&self, _msg: Box<dyn Any + Send>) -> Result<()> {
        todo!("Implement message sending")
    }
    
    /// Call the actor and wait for a response.
    /// 
    /// This is a request-response pattern where we wait for the actor
    /// to process the message and return a result.
    pub async fn call(&self, _msg: Box<dyn Any + Send>) -> Result<Box<dyn Any + Send>> {
        todo!("Implement request-response calls")
    }
}

/// The actor system manages the lifecycle of all actors.
/// 
/// This is the main entry point for creating and managing actors. It handles:
/// - Actor creation and registration
/// - Message routing
/// - Lifecycle management
/// - Graceful shutdown
pub struct ActorSystem {
    // TODO: Add actor registry
    // TODO: Add tokio runtime handle
    // TODO: Add shutdown coordination
}

impl ActorSystem {
    /// Create a new actor system.
    pub fn new() -> Self {
        ActorSystem {}
    }
    
    /// Create a new actor and return a reference to it.
    pub async fn create_actor<A: Actor>(&self, _actor: A) -> Result<ActorRef> {
        todo!("Implement actor creation")
    }
    
    /// Shutdown the actor system gracefully.
    /// 
    /// This will:
    /// 1. Stop accepting new messages
    /// 2. Wait for all pending messages to be processed
    /// 3. Call `on_stop` for all actors
    /// 4. Clean up resources
    pub async fn shutdown(self) -> Result<()> {
        todo!("Implement graceful shutdown")
    }
}

// Re-export commonly used types
pub use self::ActorRef as Handle;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_actor_id_generation() {
        let id1 = ActorId::new();
        let id2 = ActorId::new();
        assert_ne!(id1, id2);
        assert_eq!(id2.as_u64(), id1.as_u64() + 1);
    }
}