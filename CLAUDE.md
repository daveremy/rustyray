# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

RustyRay is an exploration of implementing Ray Core (the distributed actor system) in Rust. Ray is an open-source distributed computing framework originally written in C++, Java, and Python, used for scaling Python and AI applications.

- Original Ray project: https://github.com/ray-project/ray
- RustyRay repository: https://github.com/daveremy/rustyray

## Development Setup

### Initialize the Project
```bash
cargo init --name rustyray
```

### Common Commands

**Build & Run:**
- `cargo build` - Compile the project
- `cargo build --release` - Build optimized release version
- `cargo run` - Build and run the project
- `cargo test` - Run all tests
- `cargo test [test_name]` - Run specific test

**Code Quality:**
- `cargo fmt` - Format code according to Rust standards
- `cargo clippy` - Run linter for common mistakes and improvements

**Documentation:**
- `cargo doc --open` - Generate and view documentation

## Ray Core Architecture (from original Ray project)

Key components to implement in Rust:

1. **Actor System**
   - Actor creation and lifecycle management
   - Message passing between actors
   - Location transparency
   - Fault tolerance

2. **Task Scheduling**
   - Task submission and execution
   - Distributed scheduling
   - Resource management

3. **Object Store**
   - Distributed object storage
   - Serialization/deserialization
   - Reference counting and garbage collection

4. **Core Worker**
   - Task execution engine
   - Actor method invocation
   - Object transfer

5. **Global Control Store (GCS)**
   - Cluster metadata management
   - Actor registry
   - Node management

## Research Resources

- Use Gemini for analyzing the large Ray codebase
- Focus on ray/src/ray/core_worker and ray/src/ray/gcs for core functionality
- Study ray/src/ray/protobuf for protocol definitions

## Implementation Strategy

1. Start with basic actor creation and message passing
2. Implement local task execution before distributed
3. Use tokio for async runtime
4. Consider using tonic for gRPC communication (as Ray uses gRPC)
5. Study Ray's protocol buffers for wire format compatibility

## Key Rust Crates to Consider

- `tokio` - Async runtime
- `tonic` - gRPC implementation
- `prost` - Protocol buffers
- `dashmap` - Concurrent hashmap for actor registry
- `bytes` - Efficient byte buffer manipulation
- `serde` - Serialization framework

## Using Gemini CLI for Large Codebase Analysis

When analyzing the Ray codebase or understanding its architecture, use the Gemini CLI with its massive context window. Use `gemini -p` to leverage Google Gemini's large context capacity.

### When to Use Gemini vs Claude

**Use Gemini when:**
- Analyzing the entire Ray C++/Java/Python codebase
- Understanding Ray's complete architecture across multiple files
- Comparing Ray's implementation patterns across languages
- Verifying how specific features work in the original Ray
- Exploring large directories like ray/src/ray/core_worker
- Finding all usages of a pattern across Ray's codebase

**Use Claude when:**
- Writing new Rust code for RustyRay
- Making targeted changes to specific files
- Following the interactive development workflow
- Real-time coding assistance is needed
- Working within a single component or module

### File and Directory Inclusion Syntax

Use the `@` syntax to include files and directories in your Gemini prompts. You'll need to clone the Ray repository first:

```bash
# Clone Ray repository for analysis
git clone https://github.com/ray-project/ray.git ~/code/ray
```

**Loading Gemini API Key:**
The GEMINI_API_KEY is stored in the `.env` file in the project root. To use it:
```bash
# Load the API key from .env before calling gemini
export $(cat .env | grep GEMINI_API_KEY) && gemini -p "your prompt here"
```

**Examples:**
```bash
# Analyze Ray's actor system (with API key)
export $(cat .env | grep GEMINI_API_KEY) && gemini -p "@~/code/ray/src/ray/core_worker/ How does Ray implement the actor system?"

# Understand Ray's protobuf definitions
gemini -p "@~/code/ray/src/ray/protobuf/ What are the key protocol buffer messages?"

# Compare task scheduling across languages
gemini -p "@~/code/ray/python/ray/actor.py @~/code/ray/src/ray/core_worker/actor_manager.cc How do Python and C++ handle actors?"

# Full Ray architecture analysis
cd ~/code/ray && gemini --all_files -p "Explain Ray's distributed computing architecture"
```

### Ray-Specific Analysis Prompts

```bash
# Core Actor System
gemini -p "@~/code/ray/src/ray/core_worker/actor_manager.cc @~/code/ray/src/ray/gcs/gcs_server/gcs_actor_manager.cc Explain Ray's actor lifecycle"

# Task Scheduling
gemini -p "@~/code/ray/src/ray/core_worker/task_manager.cc @~/code/ray/src/ray/raylet/scheduling/ How does Ray schedule tasks?"

# Object Store
gemini -p "@~/code/ray/src/ray/object_manager/ @~/code/ray/src/ray/core_worker/reference_count.cc Explain Ray's distributed object store"

# gRPC Communication
gemini -p "@~/code/ray/src/ray/protobuf/*.proto @~/code/ray/src/ray/rpc/ What RPC services does Ray expose?"
```

## Development Strategy

1. **Start Small**: Begin with a minimal actor system
   - Actor creation and registration
   - Local message passing
   - Basic lifecycle management

2. **Study Ray's Patterns**: Use Gemini to understand Ray's implementation
   - How Ray handles actor references
   - Message serialization approach
   - Error handling patterns

3. **Iterate**: Build incrementally
   - Get basic actors working locally first
   - Add distributed features later
   - Focus on Rust idioms over direct translation