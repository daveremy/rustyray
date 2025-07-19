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

### 🚀 Currently Working On

**Phase 3: Macro System for API Ergonomics**
- `#[rustyray::remote]` for automatic function registration
- `#[rustyray::actor]` for typed actor handles
- `#[rustyray::main]` for global runtime initialization

### 📅 Future Plans

- **Phase 4**: Local shared memory object store
- **Phase 5**: Distributed runtime with gRPC
- **Phase 6**: Production features (fault tolerance, monitoring)

## Getting Started

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rustyray = { git = "https://github.com/yourusername/rustyray" }
```

### Quick Example

```rust
use rustyray::actor::ActorSystem;
use rustyray::task::{TaskSystem, TaskBuilder};
use rustyray::task_function;
use std::sync::Arc;

#[tokio::main]
async fn main() -> rustyray::error::Result<()> {
    // Initialize systems
    let actor_system = Arc::new(ActorSystem::new());
    let task_system = Arc::new(TaskSystem::new(actor_system.clone()));
    
    // Register a function
    task_system.register_function("add", task_function!(|a: i32, b: i32| async move {
        Ok::<i32, rustyray::error::RustyRayError>(a + b)
    }))?;
    
    // Submit a task
    let result_ref = TaskBuilder::new("add")
        .arg(5)
        .arg(3)
        .submit::<i32>(&task_system)
        .await?;
    
    // Get the result
    let result = result_ref.get().await?;
    println!("5 + 3 = {}", result);
    
    // Shutdown
    task_system.shutdown().await?;
    actor_system.shutdown().await?;
    
    Ok(())
}
```

### Development

```bash
# Clone the repository
git clone https://github.com/yourusername/rustyray
cd rustyray

# Run tests
cargo test

# Run examples
cargo run --example tasks
cargo run --example actors
cargo run --example cancellation

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

Check out the [examples](examples/) directory:

- [`tasks.rs`](examples/tasks.rs) - Task execution patterns
- [`actors.rs`](examples/actors.rs) - Actor communication
- [`cancellation.rs`](examples/cancellation.rs) - Timeouts and cancellation
- [`error_handling.rs`](examples/error_handling.rs) - Error propagation
- [`memory_optimization.rs`](examples/memory_optimization.rs) - Zero-copy data

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