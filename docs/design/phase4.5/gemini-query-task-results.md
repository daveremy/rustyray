# Gemini Query: How Ray Handles Task Result Storage

We're implementing RustyRay and hit a critical issue with task result storage. Please analyze how Ray handles this.

## The Problem

When a task completes:
1. It produces a typed result (e.g., `int`, `string`, custom object)
2. This gets serialized to bytes for storage
3. It needs to be stored in the object store
4. Later, when retrieved via ObjectRef, it needs to deserialize to the correct type

Our current approach fails because:
- We serialize task results to `Vec<u8>`
- We store them with type = `Vec<u8>` 
- When retrieved as the original type (e.g., `int`), type check fails

## Questions

1. **Type Preservation**: How does Ray preserve type information when storing task results in the plasma store?
   - Does it store type metadata alongside the data?
   - Is type checking done at retrieval time or storage time?

2. **Task Result Flow**: Trace the exact flow when a task completes:
   - Where does the task result get serialized?
   - How is it put into the object store?
   - What metadata is stored with it?

3. **ObjectRef Creation**: When a task is submitted:
   - Is the ObjectRef created immediately with a placeholder?
   - How does the ObjectRef know the expected type?
   - What happens if the actual result type doesn't match?

4. **CoreWorker Integration**: 
   - How does CoreWorker::Put handle task results differently from ray.put()?
   - Is there special handling for task results vs user puts?

5. **Code Examples**: Please show:
   - The C++ code where task results are stored
   - How type information is preserved
   - The retrieval path that validates types

Focus on:
- `src/ray/core_worker/core_worker.cc` - especially task execution and result handling
- `src/ray/core_worker/store_provider/` - object store integration
- `src/ray/common/task/task_spec.h` - task result type information
- Any serialization code that preserves type metadata