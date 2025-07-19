//! RustyRay - A Rust implementation of Ray Core
//! 
//! This is the main entry point that re-exports everything from the core
//! and macro crates, providing a unified API surface.

// Re-export core functionality
pub use rustyray_core::*;

// Re-export macros
pub use rustyray_macros::*;

// Prelude for common imports
pub mod prelude {
    pub use crate::{
        // Core types
        actor::{Actor, ActorRef, ActorSystem},
        task::{ObjectRef, TaskBuilder, TaskSystem},
        error::{RustyRayError, Result},
        
        // Macros
        remote, actor, actor_methods, main,
    };
    
    pub use async_trait::async_trait;
}