# Risks

## RISK-001: Scope Creep Into Post-MVP Features

Status: active
Confidence: evidence-backed
MVP: yes
Source: `.agents/archive/planning-corpus/mvp/mvp-freeze.md`, `.agents/archive/planning-corpus/AGENTS.md`
Related:
- CON-001

### Statement

Future-scope features such as plugins, MCP, marketplace, worktrees, browser
panels, concurrent agents, or global search could be accidentally promoted into
MVP implementation.

### Mitigation

Check all new tasks against `.agents/archive/planning-corpus/mvp/mvp-freeze.md` and the spec records.

## RISK-002: Worktree Lifecycle Ambiguity

Status: mitigated
Confidence: evidence-backed
MVP: no
Source: `.agents/progress/items/item-032.md`
Related:
- DEC-003

### Statement

Worktree creation and cleanup lacked enough product, architecture, and
acceptance detail for safe implementation.

### Mitigation

The scope was deferred beyond current V1.

## RISK-003: Legacy Source Deletion Too Early

Status: active
Confidence: inferred
MVP: no
Source: source retirement audit
Related:
- DEC-004

### Statement

Deleting `.task-bank/` before user confirmation could remove useful historical
context or unreviewed local changes.

### Mitigation

Keep `.task-bank/` deprecated until the user confirms removal.

