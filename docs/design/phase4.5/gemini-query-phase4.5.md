# Gemini Query: Ray's ObjectRef and Store Architecture

Please analyze Ray's codebase to understand the architectural relationship between ObjectRef, the object store, actors, and tasks. Specifically:

## 1. ObjectRef Location and Design
- Where does Ray place the ObjectRef class in its module hierarchy?
- Is ObjectRef a task-specific concept or a universal primitive?
- How does Ray handle the relationship between ObjectRef and the plasma/object store?

## 2. Actor-Store Integration
- How do actors interact with the object store in Ray?
- Do actors have direct access to put/get operations?
- Is there an actor context that provides store access?
- Code examples of actors using object store

## 3. Task-Store Integration  
- How do tasks interact with the object store?
- Are task results automatically stored in the object store?
- How does Ray handle the conversion from task results to ObjectRefs?
- Code examples of tasks using object store

## 4. Shared Store Architecture
- Is there a single shared object store instance across actors and tasks?
- How is the object store initialized and passed to different components?
- Where in the runtime/worker hierarchy does the object store live?

## 5. API Design
- What are the main APIs for putting/getting objects (ray.put, ray.get)?
- How do these APIs route to the underlying store?
- Are there different APIs for actors vs tasks?

Please provide:
- File paths for key components
- Code snippets showing the integration patterns
- The module hierarchy for ObjectRef, store, actors, and tasks
- Any important design decisions or patterns

Focus on the core Ray (not Ray Serve or other libraries) and examine:
- python/ray/core/
- src/ray/core_worker/
- src/ray/object_manager/
- Any other relevant directories