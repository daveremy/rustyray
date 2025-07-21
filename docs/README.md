# RustyRay Documentation

This directory contains design documents, reviews, and architectural decisions for the RustyRay project.

## Directory Structure

- **`design/`** - Phase-specific design documents
  - `phase1/` - Actor system design
  - `phase2/` - Task execution system design
  - `phase3/` - Macro system design
  - `phase4/` - Object store design
  - `phase4.5/` - Actor-object store integration
  - `phase5/` - Reference counting design (current)

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
- [`design/phase5/phase5-reference-counting-design.md`](design/phase5/phase5-reference-counting-design.md) - Reference counting implementation

### Architecture
- [`ORGANIZATION.md`](ORGANIZATION.md) - Documentation structure guide
- [`ADR-001-phase4.5-decisions.md`](ADR-001-phase4.5-decisions.md) - Key architectural decisions for object store integration
- [`../DECISIONS.md`](../DECISIONS.md) - All architectural decisions tracked
- [`../ROADMAP.md`](../ROADMAP.md) - Overall project roadmap

### Completed Phases
- Phase 1: Actor System ✅
- Phase 2: Task Execution ✅
- Phase 3: Macro System ✅ (70% boilerplate reduction achieved!)
- Phase 4: Object Store ✅ (Production-ready with CLRU cache)
- Phase 4.5: Universal Object Sharing ✅ (Actors and tasks share seamlessly)

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