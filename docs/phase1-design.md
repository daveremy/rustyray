# Phase 1: Local Actor System Design

This document details the design for RustyRay's initial actor system implementation.

## Overview

We're building a local (single-node) actor system inspired by Ray's actor model but leveraging Rust's strengths. This phase focuses on core actor functionality without distributed features.

## Key Design Decisions

### 1. Async Runtime: Tokio

We'll use Tokio for:
- Async message processing
- Task spawning for actors
- Channel implementations for mailboxes
- Timer functionality

### 2. Message Passing: Type-Erased with Downcasting

Following Ray's model where actors can receive different message types:

```rust
// Messages are type-erased using Any
trait Message: Send + Any + 'static {
    fn as_any(&self) -> &dyn Any;
}

// Actors handle messages by downcasting
trait Actor: Send + Sync + 'static {
    async fn handle(&mut self, msg: Box<dyn Message>) -> Result<Box<dyn Any>>;
}
```

### 3. Actor References: Smart Pointers + Channels

```rust
// ActorRef is how users interact with actors
pub struct ActorRef<A: Actor> {
    id: ActorId,
    sender: mpsc::Sender<Envelope>,
    _phantom: PhantomData<A>,
}

// Envelope wraps messages with metadata
struct Envelope {
    msg: Box<dyn Message>,
    reply_to: oneshot::Sender<Result<Box<dyn Any>>>,
}
```

### 4. Actor Lifecycle

```
Created -> Running -> Stopping -> Stopped
                |
                +-> Failed -> Restarting
```

## Architecture Components

### Core Types

```rust
// Unique actor identifier
pub struct ActorId(u64);

// Handle to an actor (like Ray's ActorHandle)
pub struct ActorHandle {
    id: ActorId,
    metadata: ActorMetadata,
}

// Runtime actor state
enum ActorState {
    Running,
    Stopping,
    Stopped,
    Failed(Error),
}
```

### Actor System

```rust
pub struct ActorSystem {
    actors: Arc<RwLock<HashMap<ActorId, ActorContainer>>>,
    runtime: Handle,
}

struct ActorContainer {
    handle: ActorHandle,
    sender: mpsc::Sender<Envelope>,
    task: JoinHandle<()>,
    state: ActorState,
}
```

### Message Flow

1. User calls `actor_ref.send(msg)` or `actor_ref.call(msg)`
2. Message wrapped in Envelope with reply channel
3. Envelope sent to actor's mailbox (mpsc channel)
4. Actor processes messages sequentially
5. Results sent back via reply channel

## Example Usage

```rust
// Define an actor
struct Counter {
    count: i32,
}

#[derive(Message)]
struct Increment;

#[derive(Message)]
struct GetCount;

impl Actor for Counter {
    async fn handle(&mut self, msg: Box<dyn Message>) -> Result<Box<dyn Any>> {
        if let Some(_) = msg.downcast_ref::<Increment>() {
            self.count += 1;
            Ok(Box::new(()))
        } else if let Some(_) = msg.downcast_ref::<GetCount>() {
            Ok(Box::new(self.count))
        } else {
            Err(RustyRayError::UnknownMessage)
        }
    }
}

// Use the actor
#[tokio::main]
async fn main() -> Result<()> {
    let system = ActorSystem::new();
    
    // Create actor
    let counter = system.create_actor(Counter { count: 0 }).await?;
    
    // Send messages
    counter.send(Increment).await?;
    counter.send(Increment).await?;
    
    // Call and get result
    let count: i32 = counter.call(GetCount).await?;
    println!("Count: {}", count); // Count: 2
    
    system.shutdown().await?;
    Ok(())
}
```

## Implementation Steps

1. **Core Types** (types.rs)
   - ActorId with unique generation
   - Message trait and derive macro
   - Result types

2. **Actor Trait** (actor/mod.rs)
   - Base Actor trait
   - ActorRef implementation
   - Message envelope

3. **Actor System** (actor/system.rs)
   - System initialization
   - Actor creation and registration
   - Message routing
   - Shutdown coordination

4. **Mailbox** (actor/mailbox.rs)
   - Bounded/unbounded channel options
   - Backpressure handling
   - Priority messages (future)

5. **Examples** (examples/)
   - Counter actor
   - Echo actor
   - Ping-pong actors
   - Supervisor pattern

## Testing Strategy

1. **Unit Tests**
   - Type generation (IDs)
   - Message serialization
   - Actor lifecycle

2. **Integration Tests**
   - Multi-actor communication
   - System shutdown
   - Error propagation

3. **Benchmarks**
   - Message throughput
   - Actor creation overhead
   - Memory usage

## Migration Path to Distributed

The local actor system is designed to extend naturally:

1. **ActorRef** becomes location-transparent
2. **ActorSystem** gains networking capabilities
3. **Messages** gain serialization for network transfer
4. **GCS** added for global actor registry

## Open Questions for Phase 1

1. **Message Types**: Should we require Serialize/Deserialize now?
2. **Supervision**: Include basic supervision in Phase 1?
3. **Typed Actors**: Support typed actors (no downcasting)?
4. **Mailbox Size**: Configurable per actor or system-wide?

## Success Criteria

Phase 1 is complete when:
- [x] Can create and destroy actors
- [x] Can send one-way messages
- [x] Can make request-response calls
- [x] Graceful system shutdown
- [x] Basic error handling
- [x] Working examples
- [x] Documentation and tests