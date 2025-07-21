# Documentation Organization

## Structure

```
docs/
├── README.md                    # Documentation overview
├── ORGANIZATION.md             # This file
├── ADR-001-phase4.5-decisions.md  # Architecture Decision Records
├── archive/                    # Historical documents
│   ├── bootstrap-prompt.md
│   └── phase2-task-design-prompt.md
├── design/                     # Phase-specific design documents
│   ├── phase1/
│   │   └── phase1-design.md
│   ├── phase2/
│   │   ├── phase2-design.md
│   │   ├── phase2-implementation-plan.md
│   │   └── ... (task serialization, API improvements)
│   ├── phase3/
│   │   ├── phase3-implementation-plan.md
│   │   ├── phase3-gemini-review.md
│   │   └── ... (macro system documentation)
│   ├── phase4/
│   │   ├── phase4-object-store-design.md
│   │   └── phase4-object-store-design-v2.md
│   ├── phase4.5/
│   │   ├── phase4.5-design.md
│   │   ├── phase4.5-design-v2.md
│   │   ├── phase4.5-completed.md
│   │   ├── phase4.5-final-recommendations.md
│   │   ├── ray-vs-rustyray-comparison.md
│   │   └── ... (implementation and analysis docs)
│   └── phase5/
│       └── phase5-reference-counting-design.md
└── reviews/                    # Code review results
    ├── comprehensive-review-results.md
    ├── gemini-comprehensive-review.md
    └── ... (review prompts and results)
```

## Key Documents

### Project Level
- `/README.md` - Project overview and quick start
- `/ROADMAP.md` - Development roadmap and progress tracking
- `/DECISIONS.md` - Architectural decisions tracking
- `/CLAUDE.md` - AI assistant instructions

### Architecture
- `/docs/ADR-001-phase4.5-decisions.md` - First Architecture Decision Record
- `/docs/design/phase*/` - Phase-specific design documents

### Reviews
- `/docs/reviews/` - Code review results from Gemini and other sources

## Document Types

1. **Design Documents** - Planning and architecture for each phase
2. **Implementation Plans** - Detailed steps for building features
3. **Review Results** - External code review feedback
4. **Architecture Decision Records (ADRs)** - Key decisions with rationale
5. **Prompts** - AI prompts used for analysis (in archive or with results)

## Naming Conventions

- `phase{N}-{topic}.md` - Phase-specific documents
- `ADR-{NNN}-{topic}.md` - Architecture Decision Records
- `gemini-{topic}.md` - Gemini AI analysis results
- `{topic}-prompt.md` - Prompts used for AI analysis