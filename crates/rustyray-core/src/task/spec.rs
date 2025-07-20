//! Task specification types.
//!
//! This module defines the core data structures for representing tasks,
//! following Ray's unified task model where normal tasks, actor tasks,
//! and actor creation tasks all use the same underlying representation.

use crate::error::Result;
use crate::types::ObjectId;
use crate::types::{ActorId, TaskId};
use serde::{Deserialize, Serialize};

/// The type of task to execute.
///
/// Ray uses a unified task model where all execution types share the same
/// underlying machinery but have different execution semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    /// A normal stateless function call that can run anywhere
    NormalTask,

    /// A method call on a specific actor instance
    ActorTask {
        /// The actor to execute this task on
        actor_id: ActorId,
    },

    /// A special task that creates a new actor instance
    ActorCreationTask,
}

/// An argument to a task.
///
/// Arguments can be either concrete values (serialized) or references
/// to the outputs of other tasks (ObjectRefs).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskArg {
    /// A concrete value, serialized as bytes
    Value(Vec<u8>),

    /// A reference to another task's output
    ObjectRef(ObjectId),
}

impl TaskArg {
    /// Create a value argument from a serializable type
    pub fn from_value<T: Serialize>(value: &T) -> Result<Self> {
        let bytes = crate::task::serde_utils::serialize(value)?;
        Ok(TaskArg::Value(bytes))
    }

    /// Create an ObjectRef argument
    pub fn from_object_ref(id: ObjectId) -> Self {
        TaskArg::ObjectRef(id)
    }

    /// Check if this argument is an ObjectRef dependency
    pub fn is_dependency(&self) -> bool {
        matches!(self, TaskArg::ObjectRef(_))
    }

    /// Get the ObjectId if this is a dependency
    pub fn as_dependency(&self) -> Option<&ObjectId> {
        match self {
            TaskArg::ObjectRef(id) => Some(id),
            _ => None,
        }
    }
}

/// Universal specification for a task.
///
/// This is the core data structure that represents any kind of task
/// in the system. It contains all the information needed to execute
/// a task, including its type, function, arguments, and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    /// Unique identifier for this task
    pub task_id: TaskId,

    /// The type of task (normal, actor, or actor creation)
    pub task_type: TaskType,

    /// The function to execute (identified by registration ID)
    pub function_id: crate::task::FunctionId,

    /// Arguments to pass to the function
    pub args: Vec<TaskArg>,

    /// Number of return values (for multiple returns)
    pub num_returns: usize,

    /// Resource requirements (e.g., CPUs, GPUs) - placeholder for now
    pub resources: TaskResources,
}

/// Resource requirements for a task.
///
/// This is a placeholder for now - in the full Ray implementation,
/// this would specify CPU, GPU, memory, and custom resource requirements.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskResources {
    /// Number of CPUs required
    pub num_cpus: f64,

    /// Number of GPUs required
    pub num_gpus: f64,
}

impl TaskSpec {
    /// Create a new task specification
    pub fn new(
        task_type: TaskType,
        function_id: crate::task::FunctionId,
        args: Vec<TaskArg>,
    ) -> Self {
        TaskSpec {
            task_id: TaskId::new(),
            task_type,
            function_id,
            args,
            num_returns: 1,
            resources: TaskResources::default(),
        }
    }

    /// Create a normal task specification
    pub fn normal(function_id: crate::task::FunctionId, args: Vec<TaskArg>) -> Self {
        Self::new(TaskType::NormalTask, function_id, args)
    }

    /// Create an actor task specification
    pub fn actor_task(
        actor_id: ActorId,
        function_id: crate::task::FunctionId,
        args: Vec<TaskArg>,
    ) -> Self {
        Self::new(TaskType::ActorTask { actor_id }, function_id, args)
    }

    /// Create an actor creation task specification
    pub fn actor_creation(function_id: crate::task::FunctionId, args: Vec<TaskArg>) -> Self {
        Self::new(TaskType::ActorCreationTask, function_id, args)
    }

    /// Get all ObjectRef dependencies in this task's arguments
    pub fn dependencies(&self) -> Vec<&ObjectId> {
        self.args
            .iter()
            .filter_map(|arg| arg.as_dependency())
            .collect()
    }

    /// Check if this task has any dependencies
    pub fn has_dependencies(&self) -> bool {
        self.args.iter().any(|arg| arg.is_dependency())
    }

    /// Set resource requirements for this task
    pub fn with_resources(mut self, resources: TaskResources) -> Self {
        self.resources = resources;
        self
    }

    /// Set the number of return values
    pub fn with_num_returns(mut self, num_returns: usize) -> Self {
        self.num_returns = num_returns;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_arg_serialization() {
        // Test value argument
        let arg = TaskArg::from_value(&42i32).unwrap();
        match arg {
            TaskArg::Value(bytes) => {
                let value: i32 = crate::task::serde_utils::deserialize(&bytes).unwrap();
                assert_eq!(value, 42);
            }
            _ => panic!("Expected Value variant"),
        }

        // Test ObjectRef argument
        let obj_id = ObjectId::new();
        let arg = TaskArg::from_object_ref(obj_id);
        assert!(arg.is_dependency());
        assert_eq!(arg.as_dependency(), Some(&obj_id));
    }

    #[test]
    fn test_task_spec_creation() {
        let func_id = crate::task::FunctionId::from("test_function");
        let args = vec![
            TaskArg::from_value(&1i32).unwrap(),
            TaskArg::from_value(&2i32).unwrap(),
        ];

        let spec = TaskSpec::normal(func_id.clone(), args);
        assert_eq!(spec.task_type, TaskType::NormalTask);
        assert_eq!(spec.function_id, func_id);
        assert_eq!(spec.num_returns, 1);
        assert!(!spec.has_dependencies());
    }

    #[test]
    fn test_task_dependencies() {
        let func_id = crate::task::FunctionId::from("test_function");
        let obj_id1 = ObjectId::new();
        let obj_id2 = ObjectId::new();

        let args = vec![
            TaskArg::from_value(&1i32).unwrap(),
            TaskArg::from_object_ref(obj_id1),
            TaskArg::from_object_ref(obj_id2),
        ];

        let spec = TaskSpec::normal(func_id, args);
        assert!(spec.has_dependencies());

        let deps = spec.dependencies();
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&&obj_id1));
        assert!(deps.contains(&&obj_id2));
    }
}
