#!/bin/bash
# Gemini final review for Phase 4.5 completion

# Load API key from .env
export $(cat .env | grep GEMINI_API_KEY)

# Run comprehensive review
gemini -p "@docs/design/phase4.5/gemini-final-review-prompt.md \
@crates/rustyray-core/src/object_ref.rs \
@crates/rustyray-core/src/ray.rs \
@crates/rustyray-core/src/task/system.rs \
@crates/rustyray-core/src/runtime.rs \
@crates/rustyray-core/examples/actor_object_sharing.rs \
@crates/rustyray-core/examples/object_store_demo.rs \
@crates/rustyray-core/tests/integration_tests.rs \
@ROADMAP.md \
@DECISIONS.md \
@docs/ADR-001-phase4.5-decisions.md \
Review the Phase 4.5 implementation and provide comprehensive feedback."