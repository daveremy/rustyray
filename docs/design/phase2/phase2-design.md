# Phase 2: Task Execution System Design

Based on analysis of Ray's implementation and Gemini's insights, this document outlines the design for RustyRay's task execution system.

## Executive Summary

Ray tasks are stateless function executions that form a dynamic dataflow graph. Unlike actor method calls (which are stateful and ordered), tasks can execute anywhere and in parallel. The key innovation is using `ObjectRef`s as futures that enable dependency tracking and asynchronous execution.

## Core Concepts

### Task vs Actor Method
- **Task**: Stateless computation that can run anywhere (`NORMAL_TASK`)
- **Actor Method**: Stateful computation bound to a specific actor (`ACTOR_TASK`)
- **Actor Creation**: Special task that spawns an actor (`ACTOR_CREATION_TASK`)

All use the same underlying `TaskSpec` structure but with different execution semantics.

### Key Components

1. **TaskSpec**: The universal task representation containing:
   - Task ID and type
   - Function identifier (not the function itself)
   - Serialized arguments
   - Dependency ObjectRefs
   - Resource requirements

2. **ObjectRef<T>**: A typed future/handle to a value that may not exist yet
   - Contains unique ObjectID
   - Enables async dataflow
   - Type parameter is compile-time only

3. **TaskManager**: Coordinates task execution
   - Dependency resolution
   - Function dispatch
   - Result storage

## Implementation Strategy

### Phase 2A: Core Infrastructure (Week 1)

#### 1. Task Types
```rust
// Core task representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    pub task_id: TaskId,
    pub task_type: TaskType,
    pub function_id: FunctionId,
    pub args: Vec<TaskArg>,
    pub num_returns: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    NormalTask,
    ActorTask(ActorId),
    ActorCreationTask,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskArg {
    Value(Vec<u8>), // Serialized value
    ObjectRef(ObjectId), // Reference to another task's result
}
```

#### 2. ObjectRef Implementation
```rust
pub struct ObjectRef<T> {
    id: ObjectId,
    receiver: Option<oneshot::Receiver<Vec<u8>>>, // For local execution
    _phantom: PhantomData<T>,
}

impl<T: DeserializeOwned> ObjectRef<T> {
    pub async fn get(self) -> Result<T> {
        let bytes = self.receiver.unwrap().await?;
        Ok(bincode::deserialize(&bytes)?)
    }
}
```

#### 3. Function Registry
```rust
// Global registry for remote functions
type TaskFunction = Box<dyn Fn(Vec<u8>) -> BoxFuture<'static, Result<Vec<u8>>> + Send + Sync>;

lazy_static! {
    static ref FUNCTION_REGISTRY: DashMap<FunctionId, TaskFunction> = DashMap::new();
}

// Registration macro
#[macro_export]
macro_rules! register_task {
    ($id:expr, $func:expr) => {
        FUNCTION_REGISTRY.insert($id, Box::new(move |args| {
            Box::pin(async move {
                let result = $func(args).await?;
                Ok(result)
            })
        }));
    };
}
```

### Phase 2B: Local Execution (Week 2)

#### 1. Task Submission
```rust
pub struct TaskSystem {
    task_manager: Arc<TaskManager>,
    actor_system: Arc<ActorSystem>, // Integration with Phase 1
}

impl TaskSystem {
    pub async fn submit_task<T>(&self, spec: TaskSpec) -> Result<ObjectRef<T>> {
        // Create ObjectRef with oneshot channel
        let (tx, rx) = oneshot::channel();
        let object_ref = ObjectRef {
            id: ObjectId::new(),
            receiver: Some(rx),
            _phantom: PhantomData,
        };
        
        // Queue task for execution
        self.task_manager.queue_task(spec, tx).await?;
        
        Ok(object_ref)
    }
}
```

#### 2. Task Execution
```rust
impl TaskManager {
    async fn execute_task(&self, spec: TaskSpec, result_tx: oneshot::Sender<Vec<u8>>) {
        // Resolve dependencies
        let resolved_args = self.resolve_dependencies(&spec.args).await?;
        
        // Look up function
        let function = FUNCTION_REGISTRY.get(&spec.function_id)
            .ok_or(Error::FunctionNotFound)?;
        
        // Execute (use spawn_blocking for CPU-bound tasks)
        let result = tokio::task::spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(function(resolved_args))
        }).await??;
        
        // Send result
        let _ = result_tx.send(result);
    }
    
    async fn resolve_dependencies(&self, args: &[TaskArg]) -> Result<Vec<u8>> {
        // Wait for all ObjectRef dependencies
        // This is simplified - real implementation needs proper serialization
    }
}
```

### Phase 2C: Integration & Examples (Week 3)

#### 1. Task-Actor Integration
- Tasks can create actors by submitting `ActorCreationTask`
- Actors can submit tasks using the same `TaskSystem`
- Tasks can call actor methods via `ActorTask` type

#### 2. Example: Parallel Computation
```rust
#[ray::remote]
async fn compute_square(x: i32) -> i32 {
    x * x
}

#[ray::remote]
async fn sum_values(values: Vec<i32>) -> i32 {
    values.iter().sum()
}

async fn example() {
    let system = TaskSystem::new();
    
    // Submit parallel tasks
    let futures: Vec<_> = (0..10)
        .map(|i| system.submit_task(compute_square.task_spec(i)))
        .collect();
    
    // Get results
    let squares: Vec<i32> = futures::future::join_all(futures)
        .await
        .into_iter()
        .map(|r| r.get())
        .collect();
    
    // Submit dependent task
    let total = system.submit_task(sum_values.task_spec(squares))
        .await?
        .get()
        .await?;
}
```

## Key Design Decisions

### 1. Use Tokio Throughout
- Leverage `tokio::spawn` for async tasks
- Use `spawn_blocking` for CPU-bound work
- Consistent with our actor system

### 2. Function Registration
- Functions must be pre-registered (like Ray)
- Use a macro to simplify registration
- Defer dynamic code loading to later phases

### 3. Local-First Design
- Start with in-process execution
- Use channels for ObjectRef implementation
- Prepare abstractions for distributed execution

### 4. Type Erasure Strategy
- Serialize arguments with bincode
- Type parameter on ObjectRef for ergonomics
- Runtime type checking where needed

## Deferred Features

1. **Distributed Execution**: Focus on local execution first
2. **Object Store**: Use simple in-memory storage initially
3. **Resource Management**: Basic implementation only
4. **Fault Tolerance**: No retry logic in Phase 2
5. **Scheduling**: Simple FIFO queue, no optimization

## Success Criteria

Phase 2 is complete when:
- [ ] Can define and register remote functions
- [ ] Can submit tasks and get ObjectRefs
- [ ] Dependencies are properly resolved
- [ ] Tasks can create and call actors
- [ ] Actors can submit tasks
- [ ] Basic examples work (map-reduce, pipeline)
- [ ] Performance is reasonable for local execution

## Migration Path

The design prepares for Phase 3 (Object Store) and Phase 4 (Distributed):
- ObjectRef abstraction can be backed by object store
- TaskManager can be split into local and distributed components
- Function registry can be synchronized across nodes
- TaskSpec is already serializable for network transfer