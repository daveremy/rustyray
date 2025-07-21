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
        actor,
        // Core types
        actor::{Actor, ActorRef, ActorSystem},
        actor_methods,
        error::{Result, RustyRayError},

        main,
        object_ref::ObjectRef,
        // Macros
        remote,
        task::{TaskBuilder, TaskSystem},
    };

    pub use async_trait::async_trait;
}
