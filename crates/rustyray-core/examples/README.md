# RustyRay Core Examples (Manual API)

These examples demonstrate the **low-level manual API** for RustyRay Core. 

## ⚠️ Important Note

**For most users, we recommend using the macro-based API instead!** 

The macro API provides:
- 70% less boilerplate code
- Automatic function registration
- Type-safe actor handles
- Zero-cost abstractions

See the main examples at `/crates/rustyray/examples/` for the recommended macro-based approach.

## When to Use Manual API

The manual API is useful when you need:
- Fine-grained control over the runtime
- Custom actor implementations
- Integration with existing systems
- Understanding how RustyRay works internally

## Examples

- `counter.rs` - Basic actor with manual message handling
- `tasks.rs` - Manual task registration and execution
- `cancellation.rs` - Task cancellation with manual setup
- `error_handling.rs` - Error propagation details
- `memory_optimization.rs` - Zero-copy optimizations

## Running Examples

```bash
cd crates/rustyray-core
cargo run --example counter
```

## Comparison

### Manual API (what you see here):
```rust
use rustyray_core::actor::{Actor, ActorSystem};
use rustyray_core::task_function;

// ... 50+ lines of boilerplate ...
```

### Macro API (recommended):
```rust
use rustyray::prelude::*;

#[rustyray::actor]
struct Counter { value: i32 }

#[rustyray::actor_methods]
impl Counter {
    pub fn new() -> Self { Counter { value: 0 } }
    pub async fn increment(&mut self) -> i32 { 
        self.value += 1; 
        self.value 
    }
}
```

For the best experience, check out the macro examples!