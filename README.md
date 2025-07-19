# RustyRay

A Rust implementation of Ray Core - exploring how to build a distributed actor system in Rust.

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

RustyRay is an exploration of implementing Ray Core's concepts in Rust. This is NOT an official Ray project, but rather a learning exercise to:

1. Understand Ray's architecture deeply
2. Explore how Rust's ownership model maps to distributed systems
3. Learn about building actor systems in Rust
4. See how far we can get with a pure Rust implementation

## Project Status

🚧 **Early Development** - We're just getting started!

### Current Focus

Starting with the absolute basics:
- [ ] Basic actor creation and registration
- [ ] Local message passing (no network yet)
- [ ] Simple actor lifecycle (create, call, destroy)

### Future Goals

- Task execution system
- Distributed object store
- Network communication (gRPC)
- Cluster management
- Fault tolerance

## Getting Started

```bash
# Clone the repository
git clone <your-repo-url>
cd rustyray

# Build the project
cargo build

# Run tests
cargo test

# Run the example
cargo run
```

## Learning Resources

- [Ray Architecture Whitepaper](https://docs.ray.io/en/latest/ray-contribute/whitepaper.html)
- [Ray Core Documentation](https://docs.ray.io/en/latest/ray-core/walkthrough.html)
- Use Gemini to analyze Ray's source: see [CLAUDE.md](CLAUDE.md) for instructions

## Architecture Notes

*As we build RustyRay, we'll document our understanding of Ray's architecture here.*

### Core Concepts We're Implementing

#### Actors (Starting Here!)
- Stateful workers that process messages
- Each actor has a unique ID
- Actors process messages sequentially (actor model)
- In Ray: Implemented in C++ with Python/Java bindings
- In RustyRay: We'll use Rust's type system and async runtime

#### Tasks (Future)
- Stateless function execution
- Can run in parallel
- Return futures/promises

#### Objects (Future)
- Immutable data in distributed memory
- Reference counted
- Automatic garbage collection

## Contributing

This is a learning project. Feel free to explore and experiment!

## License

MIT (This is an educational project, not affiliated with the official Ray project)