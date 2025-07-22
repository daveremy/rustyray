# Issue Creation Guide

## Converting Roadmap Items to Issues

This guide helps convert roadmap items into well-structured GitHub issues.

## Issue Title Format

```
[Phase X.Y] Component: Brief description
```

Examples:
- `[Phase 6] GC: Implement reference counting for ObjectRef`
- `[Phase 5.5] CI: Set up branch protection rules`
- `[Phase 7] Perf: Replace std::sync with parking_lot`

## Issue Template

```markdown
## Overview
Brief description of what needs to be done and why.

## Context
- Which phase this belongs to
- Dependencies on other work
- Related issues: #xxx, #yyy

## Acceptance Criteria
- [ ] Specific measurable outcome 1
- [ ] Specific measurable outcome 2
- [ ] Tests added/updated
- [ ] Documentation updated

## Technical Details
Any implementation notes, design decisions, or technical context.

## Effort Estimate
- [ ] Small (< 1 day)
- [ ] Medium (1-3 days)
- [ ] Large (3+ days)
```

## Examples from Current Roadmap

### Example 1: Reference Counting

**Title:** `[Phase 6] GC: Add reference counting to ObjectRef lifecycle`

**Labels:** `task`, `priority:high`, `phase:6`, `area:object-store`, `medium`

**Body:**
```markdown
## Overview
Implement reference counting for ObjectRef to enable automatic garbage collection and prevent premature object eviction.

## Context
- Part of Phase 6: Garbage Collection & Production Features
- Critical for memory management
- Blocks automatic eviction features

## Acceptance Criteria
- [ ] Add reference counter to ObjectRef::new()
- [ ] Increment counter on ObjectRef::clone()
- [ ] Decrement counter on ObjectRef::drop()
- [ ] Update object store to track reference counts
- [ ] Add tests for reference counting
- [ ] Document reference counting behavior

## Technical Details
- Use Arc<AtomicUsize> for reference counting
- Consider weak references for certain use cases
- Ensure thread safety
- Handle counter overflow gracefully

## Effort Estimate
- [x] Medium (1-3 days)
```

### Example 2: CI/CD Setup

**Title:** `[Phase 5.5] CI: Enable branch protection rules`

**Labels:** `task`, `priority:high`, `phase:5.5`, `ci`, `easy`

**Body:**
```markdown
## Overview
Configure GitHub branch protection rules to enforce code quality standards.

## Context
- Part of Phase 5.5: Development Infrastructure
- Required before enabling PR-based workflow
- Depends on CI pipeline being functional

## Acceptance Criteria
- [ ] Require PR reviews before merge
- [ ] Require status checks to pass
- [ ] Require branches to be up to date
- [ ] Include administrators in restrictions
- [ ] Document settings in CONTRIBUTING.md

## Technical Details
Settings to enable:
- Require pull request reviews (1 approval)
- Dismiss stale reviews
- Require status checks: CI / Test Suite
- Require branches to be up to date
- Include administrators

## Effort Estimate
- [x] Small (< 1 day)
```

### Example 3: Good First Issue

**Title:** `[Phase 5.5] Docs: Add benchmark examples to README`

**Labels:** `docs`, `priority:low`, `good-first-issue`, `easy`

**Body:**
```markdown
## Overview
Add examples showing how to run benchmarks to the README.

## Context
- Part of improving developer documentation
- Helps new contributors understand performance testing

## Acceptance Criteria
- [ ] Add benchmark section to README
- [ ] Include example commands
- [ ] Explain how to interpret results
- [ ] Link to any benchmark docs

## Technical Details
- Show `cargo bench` usage
- Explain benchmark organization
- Mention criterion if used

## Effort Estimate
- [x] Small (< 1 day)
```

## Issue Creation Process

1. **Review roadmap section** for the current phase
2. **Break down** large items into specific, actionable tasks
3. **Create issue** with appropriate template
4. **Add labels** following our label strategy
5. **Link related issues** in the description
6. **Add to project board** if applicable

## Bulk Issue Creation

For phase kickoff, create issues in batches:

```bash
# Example using GitHub CLI
gh issue create \
  --title "[Phase 6] GC: Implement reference counting" \
  --body-file issue-body.md \
  --label "task,priority:high,phase:6,area:object-store"
```

## Tips

1. **Keep issues focused** - One clear outcome per issue
2. **Be specific** in acceptance criteria
3. **Include context** for future contributors
4. **Estimate honestly** - it's okay to be wrong
5. **Cross-reference** related issues and PRs

## Issue Lifecycle

1. Created with `status:needs-triage`
2. Triaged and assigned → `status:ready`
3. Work begins → `status:in-progress`
4. PR opened → Link issue in PR
5. PR merged → Issue auto-closes

## Milestones

Create milestones for each phase:
- Phase 5.5 - Development Infrastructure
- Phase 6 - GC & Production
- Phase 7 - Performance
- Phase 8 - Distributed
- Phase 9 - Production Ready