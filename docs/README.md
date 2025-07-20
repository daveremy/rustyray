# RustyRay Documentation

This directory contains design documents, reviews, and architectural decisions for the RustyRay project.

## Directory Structure

- **`design/`** - Phase-specific design documents
  - `phase1/` - Actor system design
  - `phase2/` - Task execution system design
  - `phase3/` - Macro system design
  - `phase4/` - Object store design (current)
  - `phase5/` - Distributed runtime design (future)

- **`reviews/`** - Code reviews and feedback
  - Gemini AI reviews
  - Performance analysis
  - API improvement suggestions

- **`archive/`** - Historical documents and prompts
  - Design prompts
  - Intermediate drafts
  - Superseded designs

## Key Documents

### Current Focus
- [`design/phase4/phase4-object-store-design-v2.md`](design/phase4/phase4-object-store-design-v2.md) - Active object store design

### Architecture
- [`ray-vs-rustyray-comparison.md`](ray-vs-rustyray-comparison.md) - Comparison with Ray's architecture
- [`../ROADMAP.md`](../ROADMAP.md) - Overall project roadmap

### Completed Phases
- Phase 1: Actor System ✅
- Phase 2: Task Execution ✅
- Phase 3: Macro System ✅ (70% boilerplate reduction achieved!)

## Design Philosophy

1. **Incremental Development** - Build working systems phase by phase
2. **Rust-First** - Leverage Rust's type system and ownership model
3. **Learn from Ray** - Study Ray's design but adapt for Rust idioms
4. **Developer Experience** - Prioritize ergonomic APIs (hence the macro system)

## Contributing

When adding new design documents:
1. Place in appropriate phase directory
2. Use descriptive names (e.g., `phase4-object-store-design.md`)
3. Include version numbers for major revisions
4. Move superseded designs to `archive/`