# Phase 4.5: Fixing Task Result Type Storage

## Problem
Our current implementation loses type information when storing task results because we only store the serialized data bytes, not the type metadata.

## Ray's Solution
Ray uses a `RayObject` structure with:
- `data`: Serialized object bytes
- `metadata`: Type information

## Proposed Fix for RustyRay

### Option 1: Quick Fix (Minimal Changes)
Instead of using `put_with_id(bytes)`, deserialize and use the typed `put()` method:

```rust
// In TaskManager, when storing result:
match result {
    Ok(bytes) => {
        // Deserialize to preserve type
        let value: Box<dyn Any> = deserialize_to_any(&bytes)?;
        let _ = object_store.put_any(object_id, value).await;
    }
}
```

### Option 2: Proper Fix (Ray-like Architecture)
Create a `RayObject` equivalent:

```rust
pub struct StoredResult {
    data: Bytes,
    type_id: TypeId,
    type_name: String,
}

impl InMemoryStore {
    pub async fn put_result(&self, id: ObjectId, data: Vec<u8>, type_info: TypeInfo) -> Result<()> {
        // Store with proper type metadata
    }
}
```

### Option 3: Type-Erased Storage
Store everything as serialized bytes but add a deserialization wrapper:

```rust
impl ObjectRef<T> {
    pub async fn get(&self) -> Result<T> {
        // Get raw bytes
        let bytes = self.store.get_raw(self.id).await?;
        // Deserialize with type T
        deserialize::<T>(&bytes)
    }
}
```

## Recommendation

For now, let's do **Option 3** (Type-Erased Storage) because:
1. Minimal changes to existing code
2. Works with our current architecture
3. Can upgrade to full metadata later

This means:
- Store task results as raw bytes
- Let ObjectRef handle deserialization
- Type checking happens at retrieval time (like Python duck typing)