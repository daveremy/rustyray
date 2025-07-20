# RustyRay 🦀

A Rust implementation of [Ray Core](https://github.com/ray-project/ray)'s distributed actor system. RustyRay brings Ray's powerful distributed computing primitives to the Rust ecosystem with native performance and memory safety.

## What is Ray?

[Ray](https://github.com/ray-project/ray) is an open-source distributed computing framework that makes it easy to scale Python and AI applications. At its core, Ray provides:

- **Tasks**: Stateless functions that can be executed remotely
- **Actors**: Stateful worker processes that can be called remotely
- **Objects**: Immutable values that can be stored and shared across the cluster

## What is Ray Core?

Ray Core is the foundation of Ray, providing:

1. **Distributed Runtime**: Manages cluster resources and schedules work
2. **Actor System**: Creates and manages stateful services
3. **Task Execution**: Runs functions across the cluster
4. **Object Store**: Distributed memory for sharing data
5. **Global Control Store (GCS)**: Metadata management for the cluster

Ray Core is currently implemented in C++ with client libraries in Python, Java, and C++.

## What is RustyRay?

RustyRay is an implementation of Ray Core's concepts in Rust. This is NOT an official Ray project, but a serious effort to bring Ray's distributed computing model to the Rust ecosystem. Our goals:

1. **Performance**: Leverage Rust's zero-cost abstractions
2. **Safety**: Use Rust's type system to prevent distributed systems errors
3. **Compatibility**: Follow Ray's architecture while being idiomatic Rust
4. **Simplicity**: Provide both low-level control and high-level ergonomics

## 🌟 Key Features: Easy-to-Use Actor Model & Tasks

RustyRay makes distributed computing in Rust as simple as Python's Ray:

### Remote Functions (Tasks)
```rust
#[rustyray::remote]
async fn process_data(items: Vec<i32>) -> i32 {
    items.iter().sum()
}

// Call it anywhere - no registration needed!
let result = process_data_remote::remote(vec![1, 2, 3]).await?;
```

### Stateful Actors
```rust
#[rustyray::actor]
struct DataProcessor {
    processed_count: usize,
}

#[rustyray::actor_methods]
impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { processed_count: 0 }
    }
    
    pub async fn process(&mut self, data: String) -> usize {
        self.processed_count += 1;
        self.processed_count
    }
}

// Create and use actors with zero boilerplate
let processor = DataProcessor::remote().await?;
let count = processor.process("data".into()).await?.get().await?;
```

### Automatic Runtime Management
```rust
#[rustyray::main]
async fn main() -> Result<()> {
    // Runtime is automatically initialized!
    // Just write your distributed logic
}
```

**Result**: 70% less boilerplate compared to manual API, with full type safety!

## Project Status

### ✅ Completed Features

**Phase 1: Actor System**
- Actor creation, registration, and lifecycle management
- Async message passing with request-response patterns
- Graceful shutdown with proper cleanup
- Multiple actor instances with concurrent execution

**Phase 2: Task System**
- Stateless function execution with `TaskSystem`
- `ObjectRef<T>` as typed futures for task results
- Dependency resolution and task chaining
- Function registry with `task_function!` macro
- Zero-copy object store using `Bytes`
- Comprehensive error propagation
- Task cancellation and timeouts

**Phase 3: Macro System (Week 2 Complete!)**
- `#[rustyray::remote]` for automatic function registration
- `#[rustyray::actor]` for typed actor handles with generated message enums
- `#[rustyray::actor_methods]` for zero-boilerplate method dispatch
- `#[rustyray::main]` for automatic runtime initialization
- **70% reduction in boilerplate code!**

### 🚀 Currently Working On

**Phase 3: Polish & Release (Weeks 5-6)**
- Migration guide from manual API
- Performance benchmarks
- Beta release preparation

### 📅 Future Plans

- **Phase 4**: Local shared memory object store
- **Phase 5**: Distributed runtime with gRPC
- **Phase 6**: Production features (fault tolerance, monitoring)

## Getting Started

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rustyray = { git = "https://github.com/daveremy/rustyray" }
```

### Quick Example

```rust
use rustyray::prelude::*;

// Define a remote function - automatically registered!
#[rustyray::remote]
async fn compute(x: i32, y: i32) -> i32 {
    println!("Computing {} + {} on a worker", x, y);
    x + y
}

// Define an actor with methods
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
    
    pub async fn add(&mut self, x: i32) -> i32 {
        self.value += x;
        self.value
    }
}

// Main function with automatic runtime setup
#[rustyray::main]
async fn main() -> Result<()> {
    // Call remote function - that's it!
    let result = compute_remote::remote(5, 3).await?;
    println!("Result: {}", result.get().await?);  // Output: 8
    
    // Create and use an actor
    let counter = Counter::remote(10).await?;
    let value = counter.increment().await?.get().await?;
    println!("Counter: {}", value);  // Output: 11
    
    Ok(())
}
```

### Before vs After

<details>
<summary>Click to see the manual API version (what you'd write without macros)</summary>

```rust
// Manual API - lots of boilerplate!
use rustyray::prelude::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> rustyray::error::Result<()> {
    // Manual setup
    let actor_system = Arc::new(ActorSystem::new());
    let task_system = Arc::new(TaskSystem::new(actor_system.clone()));
    
    // Manual function registration
    task_system.register_function("compute", task_function!(
        |x: i32, y: i32| async move {
            Ok::<i32, RustyRayError>(x + y)
        }
    ))?;
    
    // Manual task submission
    let result_ref = TaskBuilder::new("compute")
        .arg(5)
        .arg(3)
        .submit::<i32>(&task_system)
        .await?;
    
    // Manual actor definition with message enums
    enum CounterMessage {
        Increment,
        Add(i32),
    }
    
    // ... 50+ more lines of boilerplate ...
    
    // Manual shutdown
    task_system.shutdown().await?;
    actor_system.shutdown().await?;
    
    Ok(())
}
```
</details>

### Development

```bash
# Clone the repository
git clone https://github.com/daveremy/rustyray
cd rustyray

# Run tests
cargo test

# Run examples (new macro API)
cargo run --example macro_demo
cargo run --example tasks_macro
cargo run --example counter_macro
cargo run --example comprehensive_macro_demo

# Run examples (manual API)
cargo run --example tasks
cargo run --example actors

# Check code
cargo clippy
cargo fmt
```

## Learning Resources

- [Ray Architecture Whitepaper](https://docs.ray.io/en/latest/ray-contribute/whitepaper.html)
- [Ray Core Documentation](https://docs.ray.io/en/latest/ray-core/walkthrough.html)
- Use Gemini to analyze Ray's source: see [CLAUDE.md](CLAUDE.md) for instructions

## Architecture

RustyRay implements Ray's core abstractions with Rust-native patterns:

### Actor System
- **Stateful Workers**: Each actor maintains its own state
- **Message Passing**: Async message handling with `tokio::mpsc`
- **Type Safety**: Actors use Rust traits for type-safe interfaces
- **Lifecycle**: Automatic cleanup with RAII and async drop

### Task System
- **Function Registry**: Dynamic registration with type erasure
- **ObjectRef<T>**: Typed futures that can be awaited multiple times
- **Dependency Resolution**: Automatic handling of task dependencies
- **Cancellation**: Tasks can be cancelled with proper cleanup

### Object Store
- **Zero-Copy**: Uses `bytes::Bytes` for efficient data sharing
- **Reference Counting**: Automatic garbage collection
- **Type Safety**: Compile-time type checking for stored objects

## Examples

Check out the [examples](crates/rustyray/examples/) directory:

**New Macro-Based API** (Recommended):
- [`macro_demo.rs`](crates/rustyray/examples/macro_demo.rs) - Introduction to #[remote] functions
- [`tasks_macro.rs`](crates/rustyray/examples/tasks_macro.rs) - Remote functions and actor coordination
- [`counter_macro.rs`](crates/rustyray/examples/counter_macro.rs) - Stateful actor with methods
- [`comprehensive_macro_demo.rs`](crates/rustyray/examples/comprehensive_macro_demo.rs) - All macro features

**Manual API** (For low-level control):
- [`tasks.rs`](crates/rustyray/examples/tasks.rs) - Task execution patterns
- [`actors.rs`](crates/rustyray/examples/actors.rs) - Actor communication
- [`cancellation.rs`](crates/rustyray/examples/cancellation.rs) - Timeouts and cancellation
- [`error_handling.rs`](crates/rustyray/examples/error_handling.rs) - Error propagation
- [`memory_optimization.rs`](crates/rustyray/examples/memory_optimization.rs) - Zero-copy data

## Contributing

This is a learning project. Feel free to explore and experiment!

## Roadmap

See [ROADMAP.md](ROADMAP.md) for detailed development phases.

## License

MIT License - see [LICENSE](LICENSE) file for details.

**Note**: This is an independent project, not affiliated with the official Ray project.

## Acknowledgments

- The [Ray Project](https://github.com/ray-project/ray) team for the excellent distributed computing framework
- The Rust async ecosystem, especially [Tokio](https://tokio.rs/)