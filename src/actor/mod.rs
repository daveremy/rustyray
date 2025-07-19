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

use crate::error::{Result, RustyRayError};
use crate::types::ActorId;
use std::any::Any;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::{mpsc, oneshot, Mutex};
use std::collections::HashMap;

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

/// Message sent to an actor with optional response channel.
enum ActorMessage {
    /// Regular message with optional response channel
    Message {
        payload: Box<dyn Any + Send>,
        response_tx: Option<oneshot::Sender<Result<Box<dyn Any + Send>>>>,
    },
    /// Shutdown signal
    Shutdown,
}

/// A handle to an actor that can be used to send messages.
/// 
/// This is similar to Ray's ActorHandle. It provides location transparency -
/// the actor could be local or remote (in future versions).
#[derive(Clone)]
pub struct ActorRef {
    /// The unique ID of this actor
    id: ActorId,
    /// Channel for sending messages to the actor
    sender: mpsc::Sender<ActorMessage>,
}

impl ActorRef {
    /// Get the actor's ID.
    pub fn id(&self) -> ActorId {
        self.id
    }
    
    /// Send a one-way message to the actor.
    /// 
    /// This is "fire and forget" - we don't wait for a response.
    pub async fn send(&self, msg: Box<dyn Any + Send>) -> Result<()> {
        let message = ActorMessage::Message {
            payload: msg,
            response_tx: None,
        };
        
        self.sender
            .send(message)
            .await
            .map_err(|_| RustyRayError::ActorNotFound(self.id))
    }
    
    /// Call the actor and wait for a response.
    /// 
    /// This is a request-response pattern where we wait for the actor
    /// to process the message and return a result.
    pub async fn call(&self, msg: Box<dyn Any + Send>) -> Result<Box<dyn Any + Send>> {
        let (tx, rx) = oneshot::channel();
        
        let message = ActorMessage::Message {
            payload: msg,
            response_tx: Some(tx),
        };
        
        self.sender
            .send(message)
            .await
            .map_err(|_| RustyRayError::ActorNotFound(self.id))?;
        
        rx.await
            .map_err(|_| RustyRayError::ActorNotFound(self.id))?
    }
}

/// Tracks actor lifecycle for graceful shutdown
struct ActorHandle {
    sender: mpsc::Sender<ActorMessage>,
    shutdown_complete: oneshot::Receiver<()>,
}

/// The actor system manages the lifecycle of all actors.
/// 
/// This is the main entry point for creating and managing actors. It handles:
/// - Actor creation and registration
/// - Message routing
/// - Lifecycle management
/// - Graceful shutdown
pub struct ActorSystem {
    /// Registry of all active actors with shutdown tracking
    actors: Arc<Mutex<HashMap<ActorId, ActorHandle>>>,
}

impl ActorSystem {
    /// Create a new actor system.
    pub fn new() -> Self {
        ActorSystem {
            actors: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Create a new actor and return a reference to it.
    pub async fn create_actor<A: Actor>(&self, mut actor: A) -> Result<ActorRef> {
        let id = ActorId::new();
        let (tx, mut rx) = mpsc::channel::<ActorMessage>(100); // Buffer size of 100
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        
        // Create the actor reference
        let actor_ref = ActorRef {
            id,
            sender: tx.clone(),
        };
        
        // Register the actor with shutdown tracking
        {
            let mut actors = self.actors.lock().await;
            actors.insert(id, ActorHandle {
                sender: tx,
                shutdown_complete: shutdown_rx,
            });
        }
        
        // Spawn the actor's message processing loop
        tokio::spawn(async move {
            // Call on_start
            if let Err(e) = actor.on_start().await {
                eprintln!("Actor {} failed to start: {:?}", id, e);
                let _ = shutdown_tx.send(());
                return;
            }
            
            // Process messages
            while let Some(message) = rx.recv().await {
                match message {
                    ActorMessage::Message { payload, response_tx } => {
                        let result = actor.handle(payload).await;
                        
                        // If there's a response channel, send the result
                        if let Some(tx) = response_tx {
                            let _ = tx.send(result);
                        }
                    }
                    ActorMessage::Shutdown => {
                        // Exit the message loop
                        break;
                    }
                }
            }
            
            // Call on_stop when the channel is closed
            if let Err(e) = actor.on_stop().await {
                eprintln!("Actor {} failed to stop cleanly: {:?}", id, e);
            }
            
            // Signal shutdown is complete
            let _ = shutdown_tx.send(());
        });
        
        Ok(actor_ref)
    }
    
    /// Shutdown the actor system gracefully.
    /// 
    /// This will:
    /// 1. Stop accepting new messages
    /// 2. Wait for all pending messages to be processed
    /// 3. Call `on_stop` for all actors
    /// 4. Clean up resources
    pub async fn shutdown(self) -> Result<()> {
        // Get all actors and send shutdown messages
        let mut actors = self.actors.lock().await;
        let mut shutdown_receivers = Vec::new();
        
        // Send shutdown message to all actors
        for (_, handle) in actors.iter() {
            let _ = handle.sender.send(ActorMessage::Shutdown).await;
        }
        
        // Collect shutdown receivers
        for (id, handle) in actors.drain() {
            shutdown_receivers.push((id, handle.shutdown_complete));
        }
        
        // Wait for all actors to complete shutdown
        for (id, receiver) in shutdown_receivers {
            match receiver.await {
                Ok(()) => {},
                Err(_) => eprintln!("Actor {} shutdown receiver dropped unexpectedly", id),
            }
        }
        
        Ok(())
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
    
    // Simple test actor that echoes messages
    struct EchoActor;
    
    #[async_trait]
    impl Actor for EchoActor {
        async fn handle(&mut self, msg: Box<dyn Any + Send>) -> Result<Box<dyn Any + Send>> {
            Ok(msg)
        }
    }
    
    #[tokio::test]
    async fn test_actor_send_and_call() {
        let system = ActorSystem::new();
        let actor_ref = system.create_actor(EchoActor).await.unwrap();
        
        // Test send (fire-and-forget)
        actor_ref.send(Box::new("hello".to_string())).await.unwrap();
        
        // Test call (request-response)
        let response = actor_ref.call(Box::new(42i32)).await.unwrap();
        let value = response.downcast::<i32>().unwrap();
        assert_eq!(*value, 42);
        
        system.shutdown().await.unwrap();
    }
    
    // Actor that tracks lifecycle calls
    struct LifecycleActor {
        started: Arc<Mutex<bool>>,
        stopped: Arc<Mutex<bool>>,
    }
    
    #[async_trait]
    impl Actor for LifecycleActor {
        async fn handle(&mut self, _msg: Box<dyn Any + Send>) -> Result<Box<dyn Any + Send>> {
            Ok(Box::new(()))
        }
        
        async fn on_start(&mut self) -> Result<()> {
            *self.started.lock().await = true;
            Ok(())
        }
        
        async fn on_stop(&mut self) -> Result<()> {
            *self.stopped.lock().await = true;
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn test_actor_lifecycle() {
        let system = ActorSystem::new();
        
        let started = Arc::new(Mutex::new(false));
        let stopped = Arc::new(Mutex::new(false));
        
        let actor = LifecycleActor {
            started: started.clone(),
            stopped: stopped.clone(),
        };
        
        let actor_ref = system.create_actor(actor).await.unwrap();
        
        // Give actor time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        assert!(*started.lock().await);
        
        // Send a message to ensure actor is running
        actor_ref.send(Box::new(())).await.unwrap();
        
        // Shutdown should trigger on_stop
        system.shutdown().await.unwrap();
        assert!(*stopped.lock().await);
    }
    
    #[tokio::test]
    async fn test_multiple_actors() {
        let system = ActorSystem::new();
        
        // Create multiple actors
        let actor1 = system.create_actor(EchoActor).await.unwrap();
        let actor2 = system.create_actor(EchoActor).await.unwrap();
        let actor3 = system.create_actor(EchoActor).await.unwrap();
        
        // Verify they have different IDs
        assert_ne!(actor1.id(), actor2.id());
        assert_ne!(actor2.id(), actor3.id());
        assert_ne!(actor1.id(), actor3.id());
        
        // Send messages to all actors
        actor1.call(Box::new(1)).await.unwrap();
        actor2.call(Box::new(2)).await.unwrap();
        actor3.call(Box::new(3)).await.unwrap();
        
        system.shutdown().await.unwrap();
    }
}