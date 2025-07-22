# GitHub Label Strategy for RustyRay

## Overview

This document defines our GitHub label strategy to ensure consistent issue tracking and project management.

## Label Categories

### 1. Type Labels (What kind of issue?)
**Prefix:** None (primary categorization)
**Color:** Varied by type

- `bug` - Something isn't working (red: #d73a4a)
- `feature` - New feature or request (green: #0e8a16)
- `task` - Development task from roadmap (blue: #1d76db)
- `docs` - Documentation improvements (blue: #0075ca)
- `refactor` - Code improvement without functional changes (purple: #8b4fbc)
- `test` - Test improvements or additions (yellow: #fbca04)
- `ci` - CI/CD and infrastructure (black: #000000)

### 2. Priority Labels (How urgent?)
**Prefix:** `priority:`
**Color:** Shades of orange/red

- `priority:critical` - Drop everything (dark red: #b60205)
- `priority:high` - Important, do soon (orange: #ff9500)
- `priority:medium` - Normal priority (yellow: #fbca04)
- `priority:low` - Nice to have (light gray: #d4d4d4)

### 3. Status Labels (What's the state?)
**Prefix:** `status:`
**Color:** Shades of yellow

- `status:needs-triage` - Needs evaluation (light yellow: #fff5b1)
- `status:ready` - Ready to work on (yellow: #fbca04)
- `status:in-progress` - Being worked on (orange: #ff9500)
- `status:blocked` - Blocked by dependency (red: #d73a4a)
- `status:needs-review` - PR needs review (purple: #8b4fbc)

### 4. Effort Labels (How hard?)
**Prefix:** None
**Color:** Shades of green

- `good-first-issue` - Good for newcomers (bright green: #7bd938)
- `easy` - Straightforward task (green: #0e8a16)
- `medium` - Moderate complexity (teal: #006b75)
- `hard` - Complex, needs experience (dark blue: #051c5b)

### 5. Component Labels (Which part?)
**Prefix:** `area:`
**Color:** Shades of purple

- `area:actor-system` - Actor system component (purple: #8b4fbc)
- `area:task-system` - Task system component (purple: #8b4fbc)
- `area:object-store` - Object store component (purple: #8b4fbc)
- `area:runtime` - Runtime and initialization (purple: #8b4fbc)
- `area:macros` - Macro system (purple: #8b4fbc)
- `area:api` - Public API surface (purple: #8b4fbc)

### 6. Phase Labels (Which roadmap phase?)
**Prefix:** `phase:`
**Color:** Shades of blue

- `phase:5.5` - Development Infrastructure (blue: #1d76db)
- `phase:6` - Garbage Collection & Production (blue: #1d76db)
- `phase:7` - Performance Optimizations (blue: #1d76db)
- `phase:8` - Distributed Foundation (blue: #1d76db)
- `phase:9` - Production Features (blue: #1d76db)

### 7. Meta Labels (Special cases)
**Prefix:** None
**Color:** Gray shades

- `duplicate` - Duplicate issue (gray: #cfd3d7)
- `wontfix` - Won't be worked on (gray: #ffffff)
- `help-wanted` - Extra attention needed (bright green: #008672)
- `question` - Further information requested (pink: #d876e3)
- `breaking-change` - Breaking API change (red: #d73a4a)
- `performance` - Performance related (orange: #f4a261)
- `security` - Security implications (red: #d73a4a)

## Label Usage Guidelines

### When Creating Issues

1. **Always add**:
   - One type label (bug, feature, task, etc.)
   - One priority label
   - One phase label (if applicable)

2. **Add when known**:
   - Component area label
   - Effort label
   - Status label (defaults to needs-triage)

3. **Add if applicable**:
   - breaking-change
   - performance
   - security
   - help-wanted

### Label Combinations

Good examples:
- `bug` + `priority:high` + `area:runtime` + `easy`
- `feature` + `priority:medium` + `phase:6` + `help-wanted`
- `task` + `priority:low` + `area:docs` + `good-first-issue`

### Automation Rules

1. New issues automatically get `status:needs-triage`
2. Issues with no activity for 30 days get `status:stale`
3. PRs automatically get `status:needs-review`
4. Closed issues remove status labels

## Color Coding Philosophy

- **Red tones**: Problems, bugs, blockers
- **Green tones**: Features, enhancements, help
- **Blue tones**: Tasks, documentation, phases
- **Purple tones**: Components, code areas
- **Yellow/Orange**: Status, priority
- **Gray tones**: Meta, won't fix, duplicate

## Implementation

Use the provided script to create all labels:
```bash
./scripts/create-labels.sh
```

Or manually via GitHub UI:
1. Go to Settings → Labels
2. Delete default labels you don't need
3. Create new labels according to this guide

## Maintenance

- Review label usage quarterly
- Remove unused labels
- Adjust colors for better visibility
- Keep documentation updated

## Examples by Use Case

### Bug Report
- `bug`
- `priority:high`
- `area:actor-system`
- `status:ready`

### New Feature Request
- `feature`
- `priority:medium`
- `phase:7`
- `status:needs-triage`

### Good First Issue
- `task`
- `priority:low`
- `area:docs`
- `good-first-issue`
- `help-wanted`

### Roadmap Task
- `task`
- `priority:high`
- `phase:6`
- `area:object-store`
- `medium`