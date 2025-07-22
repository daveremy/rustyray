//! Function registry for task execution.
//!
//! Ray doesn't serialize functions - it serializes function IDs. This module
//! implements a registry where functions are registered at startup
//! and can be looked up by ID during task execution.

use crate::error::{Result, RustyRayError};
use crate::task::context::DeserializationContext;
use crate::task::BoxFuture;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;

/// Type alias for a task function.
///
/// Task functions take a vector of serialized arguments and a deserialization context,
/// returning serialized results. Each argument is serialized individually, matching Ray's model.
/// The context provides access to runtime resources needed during deserialization.
pub type TaskFunction = Arc<
    dyn Fn(Vec<Vec<u8>>, DeserializationContext) -> BoxFuture<'static, Result<Vec<u8>>>
        + Send
        + Sync,
>;

/// Unique identifier for a registered function.
///
/// In production Ray, this might include version information and
/// be content-addressed. For now, we use simple string IDs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FunctionId(String);

impl FunctionId {
    /// Create a new function ID
    pub fn new(id: impl Into<String>) -> Self {
        FunctionId(id.into())
    }
}

impl From<&str> for FunctionId {
    fn from(s: &str) -> Self {
        FunctionId::new(s)
    }
}

impl From<String> for FunctionId {
    fn from(s: String) -> Self {
        FunctionId::new(s)
    }
}

impl fmt::Display for FunctionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Global function registry.
///
/// This is where all remote functions are registered. When a task
/// is executed, we look up the function here by its ID.
pub struct FunctionRegistry {
    functions: DashMap<FunctionId, TaskFunction>,
}

impl FunctionRegistry {
    /// Create a new function registry
    pub fn new() -> Self {
        FunctionRegistry {
            functions: DashMap::new(),
        }
    }

    /// Register a function with the given ID
    pub fn register<F>(&self, id: FunctionId, function: F) -> Result<()>
    where
        F: Fn(Vec<Vec<u8>>, DeserializationContext) -> BoxFuture<'static, Result<Vec<u8>>> + Send + Sync + 'static,
    {
        if self.functions.contains_key(&id) {
            return Err(RustyRayError::Internal(format!(
                "Function {} already registered",
                id
            )));
        }

        self.functions.insert(id, Arc::new(function));
        Ok(())
    }

    /// Look up a function by ID
    pub fn get(&self, id: &FunctionId) -> Option<TaskFunction> {
        self.functions.get(id).map(|entry| entry.clone())
    }

    /// Check if a function is registered
    pub fn contains(&self, id: &FunctionId) -> bool {
        self.functions.contains_key(id)
    }

    /// Get the number of registered functions
    pub fn len(&self) -> usize {
        self.functions.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.functions.is_empty()
    }

    /// Clear all registered functions (mainly for testing)
    pub fn clear(&self) {
        self.functions.clear();
    }
}

/// Helper macro to create a task function with automatic serialization.
///
/// This macro handles the boilerplate of deserializing arguments and
/// serializing results, letting you write normal async functions.
///
/// # Example
///
/// ```ignore
/// let add_func = task_function!(|x: i32, y: i32| async move {
///     Ok(x + y)
/// });
/// task_system.register_function("add", add_func)?;
///
/// // Also supports no arguments:
/// let no_args = task_function!(|| async move {
///     Ok(42)
/// });
/// ```
#[macro_export]
macro_rules! task_function {
    // Support || syntax for no arguments
    (|| $body:expr) => {
        $crate::task_function!(| | $body)
    };

    // Main implementation
    (|$($arg:ident : $arg_ty:ty),*| $body:expr) => {{
        use $crate::task::BoxFuture;
        use $crate::task::context::DeserializationContext;
        use $crate::error::Result;

        move |args: Vec<Vec<u8>>, _context: DeserializationContext| -> BoxFuture<'static, Result<Vec<u8>>> {
            Box::pin(async move {
                // Count the number of arguments expected
                let expected_args = 0 $(+ {let _ = stringify!($arg); 1})*;

                if args.len() != expected_args {
                    return Err($crate::error::RustyRayError::Internal(
                        format!("Expected {} arguments, got {}", expected_args, args.len())
                    ));
                }

                // Deserialize each argument individually
                #[allow(unused_mut, unused_variables)]
                let mut arg_iter = args.into_iter();

                $(
                    let $arg: $arg_ty = {
                        let arg_bytes = arg_iter.next()
                            .ok_or_else(|| $crate::error::RustyRayError::Internal(
                                "Missing argument".to_string()
                            ))?;
                        
                        // For now, just use standard deserialization
                        // ObjectRef should be passed as TaskArg::ObjectRef, not serialized directly
                        $crate::task::serde_utils::deserialize(&arg_bytes)?
                    };
                )*

                // Execute function
                let result = $body.await?;

                // Serialize result
                let config = bincode::config::standard();
                bincode::serde::encode_to_vec(&result, config)
                    .map_err(|e| $crate::error::RustyRayError::Internal(
                        format!("Failed to serialize result: {:?}", e)
                    ))
            })
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_registration() {
        let registry = FunctionRegistry::new();
        let id = FunctionId::from("test_func");

        // Register a simple function
        let func = |_args: Vec<Vec<u8>>, _context: DeserializationContext| -> BoxFuture<'static, Result<Vec<u8>>> {
            Box::pin(async move { Ok(vec![]) })
        };

        registry.register(id.clone(), func).unwrap();

        // Check it's registered
        assert!(registry.contains(&id));
        assert_eq!(registry.len(), 1);

        // Can't register twice
        let result = registry.register(id.clone(), func);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_function_execution() {
        let registry = FunctionRegistry::new();
        let id = FunctionId::from("add");

        // Register an add function
        let func = |args: Vec<Vec<u8>>, _context: DeserializationContext| -> BoxFuture<'static, Result<Vec<u8>>> {
            Box::pin(async move {
                if args.len() != 2 {
                    return Err(RustyRayError::Internal("Expected 2 arguments".to_string()));
                }
                let x: i32 = crate::task::serde_utils::deserialize(&args[0]).unwrap();
                let y: i32 = crate::task::serde_utils::deserialize(&args[1]).unwrap();
                let result = x + y;
                Ok(crate::task::serde_utils::serialize(&result).unwrap())
            })
        };

        registry.register(id.clone(), func).unwrap();

        // Execute the function
        let task_func = registry.get(&id).unwrap();
        let args = vec![
            crate::task::serde_utils::serialize(&10).unwrap(),
            crate::task::serde_utils::serialize(&20).unwrap(),
        ];
        let object_store = Arc::new(crate::object_store::InMemoryStore::new(crate::object_store::StoreConfig::default()));
        let context = DeserializationContext::new(object_store);
        let result_bytes = task_func(args, context).await.unwrap();
        let result: i32 = crate::task::serde_utils::deserialize(&result_bytes).unwrap();

        assert_eq!(result, 30);
    }

    #[tokio::test]
    async fn test_task_function_macro() {
        let registry = FunctionRegistry::new();

        // Create function using the macro
        let multiply_func = task_function!(|x: i32, y: i32| async move {
            Ok::<i32, crate::error::RustyRayError>(x * y)
        });

        // Register it
        registry
            .register(FunctionId::from("multiply"), multiply_func)
            .unwrap();

        // Execute the function
        let func = registry.get(&FunctionId::from("multiply")).unwrap();
        let args = vec![
            crate::task::serde_utils::serialize(&3).unwrap(),
            crate::task::serde_utils::serialize(&4).unwrap(),
        ];
        let object_store = Arc::new(crate::object_store::InMemoryStore::new(crate::object_store::StoreConfig::default()));
        let context = DeserializationContext::new(object_store);
        let result_bytes = func(args, context).await.unwrap();
        let result: i32 = crate::task::serde_utils::deserialize(&result_bytes).unwrap();

        assert_eq!(result, 12);
    }
}
