# RustyRay Examples (Macro API) 🚀

These examples demonstrate the **recommended macro-based API** for RustyRay.

## Why Macro API?

The macro API provides a Python Ray-like experience with:
- ✨ **70% less boilerplate** compared to manual API
- 🔧 **Automatic registration** of functions and actors
- 🛡️ **Type safety** with compile-time checks
- ⚡ **Zero runtime overhead**

## Examples

### Basic Examples

- `macro_demo.rs` - Introduction to `#[remote]` functions
- `counter_macro.rs` - Stateful actor with `#[actor]` and `#[actor_methods]`
- `tasks_macro.rs` - Remote functions and actor coordination

### Advanced Examples

- `comprehensive_macro_demo.rs` - All macro features including:
  - Multiple constructors
  - Resource requirements (num_cpus, num_gpus)
  - Error handling patterns
  - Parallel execution
  
- `error_context_demo.rs` - Enhanced error propagation with context

### Other Examples

- `actor_demo.rs` - Actor patterns
- `objectref_demo.rs` - ObjectRef usage (Phase 4 preview)

## Running Examples

```bash
# Run a specific example
cargo run --example macro_demo

# Run the comprehensive demo
cargo run --example comprehensive_macro_demo
```

## Quick Start

```rust
use rustyray::prelude::*;

// Define a remote function - automatically registered!
#[rustyray::remote]
async fn compute(x: i32, y: i32) -> i32 {
    x + y
}

// Define an actor
#[rustyray::actor]
struct Counter {
    value: i32,
}

#[rustyray::actor_methods]
impl Counter {
    pub fn new() -> Self {
        Counter { value: 0 }
    }
    
    pub async fn increment(&mut self) -> i32 {
        self.value += 1;
        self.value
    }
}

// Main with automatic runtime setup
#[rustyray::main]
async fn main() -> Result<()> {
    // Call remote function
    let result = compute_remote::remote(5, 3).await?;
    println!("Result: {}", result.get().await?);
    
    // Use actor
    let counter = Counter::remote().await?;
    let value = counter.increment().await?.get().await?;
    println!("Counter: {}", value);
    
    Ok(())
}
```

## Manual API

For low-level control, see the manual API examples at `/crates/rustyray-core/examples/`.