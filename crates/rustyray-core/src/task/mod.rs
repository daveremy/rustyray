//! Task execution system for RustyRay.
//!
//! This module implements Ray's task concept - stateless function execution
//! with dependencies. Tasks differ from actors in that they:
//!
//! - Are stateless (no persistent state between calls)
//! - Can execute anywhere (not bound to a specific process)
//! - Return ObjectRefs (futures) immediately
//! - Form dynamic dataflow graphs through dependencies
//!
//! # Architecture
//!
//! The task system consists of:
//! - `TaskSpec`: Universal representation of a task
//! - `ObjectRef<T>`: Type-safe future for task results
//! - `TaskManager`: Handles task execution and dependencies
//! - Function registry: Maps function IDs to implementations
//!
//! # Example
//!
//! ```ignore
//! #[ray::remote]
//! async fn add(x: i32, y: i32) -> i32 {
//!     x + y
//! }
//!
//! let system = TaskSystem::new();
//! let result_ref = system.submit(add.task_spec(1, 2)).await?;
//! let result = result_ref.get().await?; // 3
//! ```

use crate::error::Result;
use std::any::Any;
use std::future::Future;
use std::pin::Pin;

mod cancellation;
pub mod context;
mod manager;
mod registry;
pub mod serde_utils;
mod spec;
mod system;
mod test_builder_errors;
mod test_cancellation_integration;
mod test_error_propagation;
mod test_objectref_args;
mod test_objectref_passing;
mod test_objectref_sharing;

pub use manager::{TaskManager, TaskManagerConfig};
pub use registry::{FunctionId, FunctionRegistry};
pub use spec::{TaskArg, TaskSpec, TaskType};
pub use system::{TaskBuilder, TaskSystem};

/// Type alias for boxed futures used in task execution
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Trait for types that can be used as task arguments
pub trait TaskArgument: Send + 'static {
    /// Convert to serialized bytes
    fn to_bytes(&self) -> Result<Vec<u8>>;

    /// Convert from serialized bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized;
}

/// Trait for types that can be returned from tasks
pub trait TaskReturn: Send + 'static {
    /// Convert to type-erased Any
    fn into_any(self) -> Box<dyn Any + Send>;

    /// Try to convert from Any
    fn from_any(any: Box<dyn Any + Send>) -> Result<Self>
    where
        Self: Sized;
}

#[cfg(test)]
mod tests {
    use crate::types::TaskId;

    #[test]
    fn test_task_id_generation() {
        let id1 = TaskId::new();
        let id2 = TaskId::new();
        assert_ne!(id1, id2);
    }
}
