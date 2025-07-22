# Phase 4.5 - ObjectRef Automatic Resolution Limitation

## Current Limitation

In the current implementation, when passing ObjectRef as an argument to a remote function, the ObjectRef is NOT automatically resolved to its value as in Ray. This is a known limitation that requires manual resolution.

### Ray's Behavior

In Ray, when you pass an ObjectRef as an argument:
```python
# Ray Python
data_ref = create_data.remote()
result_ref = process_data.remote(data_ref)  # Ray resolves data_ref automatically
```

The remote function receives the resolved value:
```python
@ray.remote
def process_data(data):  # Receives actual data, not ObjectRef
    return sum(data)
```

### Current RustyRay Behavior

In RustyRay, you must manually resolve ObjectRefs before passing them:
```rust
// Current limitation - must resolve manually
let data_ref = create_data_remote::remote().await?;
let data = data_ref.get().await?;  // Manual resolution
let result_ref = process_data_remote::remote(data).await?;
```

The remote function must accept the resolved type:
```rust
#[rustyray::remote]
async fn process_data(data: Vec<i32>) -> i32 {  // Takes Vec<i32>, not ObjectRef<Vec<i32>>
    data.iter().sum()
}
```

## Why This Limitation Exists

1. **Macro Complexity**: The #[remote] macro would need to generate different signatures for the public API vs the registered function
2. **Type System**: Rust's type system makes it challenging to automatically transform function signatures
3. **Serialization Context**: ObjectRef serialization requires runtime context that's not available during macro expansion

## Workaround Pattern

For now, use this pattern:
```rust
// Define remote functions to accept resolved values
#[rustyray::remote]
async fn process_data(data: Vec<i32>) -> Result<i32> {
    Ok(data.iter().sum())
}

// In your code, manually resolve ObjectRefs
let data_ref = create_data_remote::remote().await?;
let data = ray::get(&data_ref).await?;  // or data_ref.get().await?
let result_ref = process_data_remote::remote(data).await?;
```

## Future Solution

To properly implement Ray's behavior, we would need to:
1. Modify the #[remote] macro to detect ObjectRef parameters
2. Generate a wrapper function that accepts ObjectRef
3. Have the wrapper resolve ObjectRefs before calling the actual function
4. Register the actual function with resolved parameter types

This is planned for a future phase of development.