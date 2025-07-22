#!/bin/bash
# Create GitHub labels according to our label strategy

echo "Creating GitHub labels for RustyRay..."

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo "Error: GitHub CLI (gh) is not installed."
    echo "Please install it from: https://cli.github.com/"
    exit 1
fi

# Check if we're in a git repo
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "Error: Not in a git repository"
    exit 1
fi

# Type labels
echo "Creating type labels..."
gh label create "bug" --description "Something isn't working" --color "d73a4a" --force
gh label create "feature" --description "New feature or request" --color "0e8a16" --force
gh label create "task" --description "Development task from roadmap" --color "1d76db" --force
gh label create "docs" --description "Documentation improvements" --color "0075ca" --force
gh label create "refactor" --description "Code improvement without functional changes" --color "8b4fbc" --force
gh label create "test" --description "Test improvements or additions" --color "fbca04" --force
gh label create "ci" --description "CI/CD and infrastructure" --color "000000" --force

# Priority labels
echo "Creating priority labels..."
gh label create "priority:critical" --description "Drop everything" --color "b60205" --force
gh label create "priority:high" --description "Important, do soon" --color "ff9500" --force
gh label create "priority:medium" --description "Normal priority" --color "fbca04" --force
gh label create "priority:low" --description "Nice to have" --color "d4d4d4" --force

# Status labels
echo "Creating status labels..."
gh label create "status:needs-triage" --description "Needs evaluation" --color "fff5b1" --force
gh label create "status:ready" --description "Ready to work on" --color "fbca04" --force
gh label create "status:in-progress" --description "Being worked on" --color "ff9500" --force
gh label create "status:blocked" --description "Blocked by dependency" --color "d73a4a" --force
gh label create "status:needs-review" --description "PR needs review" --color "8b4fbc" --force

# Effort labels
echo "Creating effort labels..."
gh label create "good-first-issue" --description "Good for newcomers" --color "7bd938" --force
gh label create "easy" --description "Straightforward task" --color "0e8a16" --force
gh label create "medium" --description "Moderate complexity" --color "006b75" --force
gh label create "hard" --description "Complex, needs experience" --color "051c5b" --force

# Component labels
echo "Creating component labels..."
gh label create "area:actor-system" --description "Actor system component" --color "8b4fbc" --force
gh label create "area:task-system" --description "Task system component" --color "8b4fbc" --force
gh label create "area:object-store" --description "Object store component" --color "8b4fbc" --force
gh label create "area:runtime" --description "Runtime and initialization" --color "8b4fbc" --force
gh label create "area:macros" --description "Macro system" --color "8b4fbc" --force
gh label create "area:api" --description "Public API surface" --color "8b4fbc" --force

# Phase labels
echo "Creating phase labels..."
gh label create "phase:5.5" --description "Development Infrastructure" --color "1d76db" --force
gh label create "phase:6" --description "Garbage Collection & Production" --color "1d76db" --force
gh label create "phase:7" --description "Performance Optimizations" --color "1d76db" --force
gh label create "phase:8" --description "Distributed Foundation" --color "1d76db" --force
gh label create "phase:9" --description "Production Features" --color "1d76db" --force

# Meta labels
echo "Creating meta labels..."
gh label create "duplicate" --description "Duplicate issue" --color "cfd3d7" --force
gh label create "wontfix" --description "Won't be worked on" --color "ffffff" --force
gh label create "help-wanted" --description "Extra attention needed" --color "008672" --force
gh label create "question" --description "Further information requested" --color "d876e3" --force
gh label create "breaking-change" --description "Breaking API change" --color "d73a4a" --force
gh label create "performance" --description "Performance related" --color "f4a261" --force
gh label create "security" --description "Security implications" --color "d73a4a" --force

echo ""
echo "✅ Label creation complete!"
echo ""
echo "Optional: Delete default GitHub labels that we don't use:"
echo "  gh label delete 'enhancement' --yes"
echo "  gh label delete 'invalid' --yes"
echo "  gh label delete 'documentation' --yes"
echo ""
echo "To view all labels:"
echo "  gh label list"