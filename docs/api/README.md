# RustyRay API Reference

## Overview

RustyRay provides a Ray-compatible API for distributed computing in Rust. The API is designed to be simple and ergonomic while maintaining type safety.

## Core APIs

### ray Module

The `ray` module provides the main API for interacting with the object store.

```rust
use rustyray::ray;

// Store data in the object store
let data_ref = ray::put(vec![1, 2, 3]).await?;

// Retrieve data from the object store
let data = ray::get(&data_ref).await?;

// Store multiple objects at once
let refs = ray::put_batch(vec![1, 2, 3]).await?;

// Retrieve multiple objects at once
let values = ray::get_batch(&refs).await?;

// Wait for objects to be ready
let (ready, pending) = ray::wait(&refs, Some(2), None).await?;
```

### Runtime Management

```rust
use rustyray::runtime;

// Initialize the runtime (required before using ray APIs)
runtime::init()?;

// Check if runtime is initialized
if runtime::is_initialized() {
    // Use ray APIs
}

// Get the global runtime (for advanced usage)
let rt = runtime::global()?;

// Shutdown the runtime
runtime::shutdown()?;

// Async shutdown with explicit subsystem cleanup
runtime::shutdown_async().await?;
```

## Macros

### #[rustyray::remote]

Marks an async function as remotely executable.

```rust
#[rustyray::remote]
async fn process_data(items: Vec<i32>) -> i32 {
    items.iter().sum()
}

// Usage
let result = process_data_remote::remote(vec![1, 2, 3]).await?;
let sum = result.get().await?;
```

**Features:**
- Automatic function registration
- Type-safe invocation
- Support for ObjectRef parameters
- Result<T> return types supported

### #[rustyray::actor]

Defines an actor struct.

```rust
#[rustyray::actor]
struct Counter {
    value: i32,
}

#[rustyray::actor_methods]
impl Counter {
    pub fn new(initial: i32) -> Self {
        Counter { value: initial }
    }
    
    pub async fn increment(&mut self) -> i32 {
        self.value += 1;
        self.value
    }
    
    pub fn get_value(&self) -> i32 {
        self.value
    }
}

// Usage
let counter = Counter::remote(10).await?;
let value = counter.increment().await?.get().await?;
```

**Features:**
- Generated message enum
- Typed actor handle
- Support for multiple constructors
- Async and sync methods

### #[rustyray::main]

Automatically initializes the runtime.

```rust
#[rustyray::main]
async fn main() -> Result<()> {
    // Runtime is already initialized
    let data_ref = ray::put(42).await?;
    Ok(())
}
```

## Types

### ObjectRef<T>

A reference to an object stored in the object store.

```rust
pub struct ObjectRef<T> {
    id: ObjectId,
    _phantom: PhantomData<T>,
}

impl<T> ObjectRef<T> {
    // Get the value from the object store
    pub async fn get(&self) -> Result<T>;
    
    // Get the underlying object ID
    pub fn id(&self) -> &ObjectId;
}
```

**Properties:**
- Clone-able and Send + Sync
- Type-safe with phantom data
- Implements Future<Output = Result<T>>
- Automatic error propagation

### Result<T>

All RustyRay APIs return `Result<T, RustyRayError>`.

```rust
pub enum RustyRayError {
    TaskNotFound(String),
    ActorNotFound(ActorId),
    ObjectNotFound(ObjectId),
    SerializationError(String),
    Timeout(Duration),
    Cancelled,
    Internal(String),
}
```

## Advanced APIs

### TaskBuilder

For manual task submission (low-level API).

```rust
use rustyray::TaskBuilder;

let result = TaskBuilder::new("function_name")
    .arg(42)
    .arg("hello")
    .timeout(Duration::from_secs(30))
    .submit::<String>(&task_system)
    .await?;
```

### ActorSystem

For direct actor system interaction (low-level API).

```rust
use rustyray::ActorSystem;

let actor_system = ActorSystem::new();
let actor_ref = actor_system.create_actor(
    Box::new(MyActor::new()),
    Some("my_actor".into())
).await?;
```

## Testing

### with_test_runtime

Test fixture for automatic runtime management.

```rust
use rustyray_core::test_utils::with_test_runtime;

#[tokio::test]
async fn test_something() {
    with_test_runtime(|| async {
        let obj_ref = ray::put(42).await.unwrap();
        let value = ray::get(&obj_ref).await.unwrap();
        assert_eq!(value, 42);
    }).await;
}
```

## Best Practices

### 1. Always Initialize Runtime
```rust
// In main
#[rustyray::main]
async fn main() -> Result<()> {
    // Runtime automatically initialized
}

// Or manually
runtime::init()?;
```

### 2. Use ObjectRef for Large Data
```rust
// Good: Pass by reference
let data_ref = ray::put(large_data).await?;
let result = process_remote::remote(data_ref).await?;

// Bad: Pass by value (causes serialization)
let result = process_remote::remote(large_data).await?;
```

### 3. Handle Errors Gracefully
```rust
match ray::get(&obj_ref).await {
    Ok(value) => println!("Got: {}", value),
    Err(RustyRayError::ObjectNotFound(_)) => {
        println!("Object was evicted");
    }
    Err(e) => return Err(e),
}
```

### 4. Clean Shutdown
```rust
// Ensure proper cleanup
runtime::shutdown()?;

// Or use async shutdown for graceful cleanup
runtime::shutdown_async().await?;
```

## Performance Tips

1. **Batch Operations**: Use `put_batch`/`get_batch` for multiple objects
2. **Zero-Copy**: Object store uses `bytes::Bytes` internally
3. **Reuse ObjectRefs**: They're cheap to clone
4. **Avoid Repeated Serialization**: Store once, reference many times

## Limitations

1. **Local Only**: No distributed execution yet (Phase 8)
2. **No GPU Support**: CPU only for now
3. **Single Runtime**: One runtime per process
4. **Test Isolation**: Tests must run single-threaded