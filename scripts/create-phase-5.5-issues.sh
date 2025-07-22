#!/bin/bash
# Create initial issues for Phase 5.5

echo "Creating Phase 5.5 issues..."

# Create milestone first
echo "Creating Phase 5.5 milestone..."
gh api repos/:owner/:repo/milestones \
  --method POST \
  --field title="Phase 5.5 - Development Infrastructure" \
  --field description="Establish CI/CD pipeline and development workflows" \
  --field due_on="2025-02-08T00:00:00Z" \
  2>/dev/null || echo "Milestone might already exist"

# Issue 1: Label setup (already done)
gh issue create \
  --title "[Phase 5.5] Setup: Create and configure GitHub labels" \
  --body "## Overview
Establish consistent labeling strategy for issue tracking.

## Context
- Part of Phase 5.5: Development Infrastructure
- Required before creating other issues
- Foundation for project management

## Acceptance Criteria
- [x] Create label strategy document
- [x] Create label creation script
- [x] Apply labels to repository
- [ ] Remove unused default labels

## Status
Mostly complete - just need to run the script on the repository." \
  --label "task,priority:high,phase:5.5,ci,easy" \
  --milestone "Phase 5.5 - Development Infrastructure"

# Issue 2: Branch protection
gh issue create \
  --title "[Phase 5.5] CI: Enable branch protection rules" \
  --body "## Overview
Configure GitHub branch protection to enforce code quality.

## Context
- Part of Phase 5.5: Development Infrastructure
- Required for PR-based workflow
- Depends on CI pipeline working

## Acceptance Criteria
- [ ] Require PR reviews before merge
- [ ] Require CI status checks to pass
- [ ] Require branches to be up to date
- [ ] Include administrators
- [ ] Update CONTRIBUTING.md with process

## Technical Details
- Wait until CI is proven stable
- Start with basic rules, add more later
- Document any exceptions" \
  --label "task,priority:high,phase:5.5,ci,easy" \
  --milestone "Phase 5.5 - Development Infrastructure"

# Issue 3: Pre-commit hooks
gh issue create \
  --title "[Phase 5.5] DX: Set up pre-commit hooks" \
  --body "## Overview
Configure pre-commit hooks for local development.

## Context
- Part of Phase 5.5: Development Infrastructure
- Improves developer experience
- Catches issues before CI

## Acceptance Criteria
- [ ] Create .pre-commit-config.yaml
- [ ] Add rustfmt check
- [ ] Add clippy check
- [ ] Add test subset runner
- [ ] Create installation script
- [ ] Document in CONTRIBUTING.md

## Technical Details
- Use pre-commit framework
- Make it easy to bypass when needed
- Keep checks fast" \
  --label "task,priority:medium,phase:5.5,ci,medium" \
  --milestone "Phase 5.5 - Development Infrastructure"

# Issue 4: Development scripts
gh issue create \
  --title "[Phase 5.5] DX: Create developer automation scripts" \
  --body "## Overview
Create scripts to automate common development tasks.

## Context
- Part of Phase 5.5: Development Infrastructure
- Reduces friction for contributors
- Standardizes common operations

## Acceptance Criteria
- [ ] dev-setup.sh - Set up development environment
- [ ] run-checks.sh - Run all pre-commit checks
- [ ] create-issue.sh - Create issue from template
- [ ] benchmark.sh - Run benchmarks locally
- [ ] Document all scripts

## Technical Details
- Make scripts cross-platform where possible
- Add error handling
- Keep them simple and maintainable" \
  --label "task,priority:low,phase:5.5,ci,medium,help-wanted" \
  --milestone "Phase 5.5 - Development Infrastructure"

# Issue 5: Update CONTRIBUTING.md
gh issue create \
  --title "[Phase 5.5] Docs: Update CONTRIBUTING.md with new workflows" \
  --body "## Overview
Update contribution guidelines with new development workflows.

## Context
- Part of Phase 5.5: Development Infrastructure
- Critical for new contributors
- Should reflect all new processes

## Acceptance Criteria
- [ ] Document PR-based workflow
- [ ] Explain branch naming conventions
- [ ] Add label usage guide
- [ ] Include CI/CD information
- [ ] Add troubleshooting section

## Technical Details
- Keep it concise but complete
- Add examples
- Link to other docs as needed" \
  --label "docs,priority:medium,phase:5.5,easy,good-first-issue" \
  --milestone "Phase 5.5 - Development Infrastructure"

# Issue 6: CI improvements
gh issue create \
  --title "[Phase 5.5] CI: Add caching to speed up builds" \
  --body "## Overview
Optimize CI pipeline performance with better caching.

## Context
- CI is currently functional but could be faster
- Better caching reduces wait times
- Improves developer experience

## Acceptance Criteria
- [ ] Cache cargo registry
- [ ] Cache build artifacts appropriately
- [ ] Cache between jobs where possible
- [ ] Measure improvement
- [ ] Document cache strategy

## Technical Details
- Already using Swatinem/rust-cache
- Could optimize further
- Consider sccache for distributed caching" \
  --label "task,priority:low,phase:5.5,ci,performance,medium" \
  --milestone "Phase 5.5 - Development Infrastructure"

echo ""
echo "✅ Phase 5.5 issues created!"
echo ""
echo "View issues with:"
echo "  gh issue list --label 'phase:5.5'"
echo ""
echo "View in browser:"
echo "  gh issue list --label 'phase:5.5' --web"