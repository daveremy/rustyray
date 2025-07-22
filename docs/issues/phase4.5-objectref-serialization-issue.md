# Phase 4.5 - ObjectRef Serialization Issue

## Problem Discovery

While implementing Phase 4.5 (integrating object store with actors and tasks), we discovered a critical issue: **ObjectRef cannot be serialized when passed as a task argument**.

### Root Cause

1. **ObjectRef contains Arc<InMemoryStore>**: Our ObjectRef struct holds a reference to the object store
2. **Task arguments are serialized**: When passing arguments to remote tasks, they're serialized to bytes
3. **Arc<InMemoryStore> cannot be serialized**: The store reference is not serializable
4. **Error message**: "ObjectRef deserialization requires runtime context - use TaskSystem methods"

### Why This Matters

This breaks a key Ray pattern - passing ObjectRefs between tasks:

```rust
// This should work but doesn't:
let data_ref = create_data_remote::remote().await?;
let result_ref = process_data_remote::remote(data_ref).await?; // FAILS!
```

## Current Architecture

```rust
pub struct ObjectRef<T> {
    id: ObjectId,
    store: Arc<InMemoryStore>,  // <-- Cannot be serialized!
    _phantom: PhantomData<T>,
}
```

## Solution Options

### Option 1: ID-Only ObjectRef (Recommended)
Make ObjectRef contain only the ID, resolve store at runtime:

```rust
pub struct ObjectRef<T> {
    id: ObjectId,
    _phantom: PhantomData<T>,
}

impl<T> ObjectRef<T> {
    pub async fn get(&self) -> Result<T> {
        let store = runtime::global()?.object_store();
        // Use store to get the value
    }
}
```

**Pros**: 
- Clean separation of concerns
- Easily serializable
- Matches Ray's design

**Cons**: 
- Requires runtime lookup for every operation
- Slightly more overhead

### Option 2: Custom Serialization
Implement custom Serialize/Deserialize that only serializes the ID:

```rust
impl<T> Serialize for ObjectRef<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.id.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for ObjectRef<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> {
        let id = ObjectId::deserialize(deserializer)?;
        // Need runtime context here to get store!
        Err(...)  // Can't work without context
    }
}
```

**Problem**: Deserialization still needs the store reference!

### Option 3: Special TaskArg Handling
Have TaskSystem handle ObjectRef arguments specially:

```rust
enum TaskArg {
    Serialized(Vec<u8>),
    ObjectRef(ObjectId),  // Special case
}
```

**Pros**: 
- No changes to ObjectRef
- Task system can inject store reference

**Cons**: 
- Complex implementation
- Breaks generic serialization

## Recommended Solution

**Option 1 (ID-Only ObjectRef)** is the cleanest solution:

1. ObjectRef becomes a simple ID wrapper
2. All operations go through runtime::global()
3. Easily serializable for task arguments
4. Matches Ray's design philosophy

## Implementation Plan

1. Refactor ObjectRef to remove store field
2. Update ObjectRef::get() to use runtime::global()
3. Update ObjectRef::new() to only take ID
4. Fix all call sites
5. Test ObjectRef serialization in tasks

## Impact

This change will:
- Enable passing ObjectRefs as task arguments (critical feature)
- Slightly increase overhead for ObjectRef operations (negligible)
- Simplify the ObjectRef implementation
- Make the system more like Ray's original design