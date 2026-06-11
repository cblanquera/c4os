# Evidence

## EVD-001: Frozen MVP Contract

Status: accepted
Confidence: evidence-backed
MVP: yes
Source: `.agents/archive/planning-corpus/mvp/mvp-freeze.md`
Related:
- REQ-001
- CON-001

### Statement

`.agents/archive/planning-corpus/mvp/mvp-freeze.md` dated 2026-06-10 freezes MVP scope for
implementation planning.

## EVD-002: Legacy Progress Bank

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.task-bank/`
Related:
- AC-002

### Statement

`.task-bank/` contained progress bank files for items `item-001` through
`item-034`, logs, batch metadata, outputs, and a progress CSV.

## EVD-003: Migrated Progress Bank

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/progress/`
Related:
- AC-002

### Statement

`.agents/progress/` now contains the active progress dashboard, item packets,
logs, batches, outputs, decisions, conventions, and CSV progress sheet.

## EVD-004: Deprecated Legacy Marker

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.task-bank/DEPRECATED.md`
Related:
- DEC-004

### Statement

`.task-bank/DEPRECATED.md` points future workers to `.agents/progress/` and
`.agents/specs/c4os-mvp/`.

## EVD-005: Archived Human Planning Corpus

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/archive/planning-corpus/MIGRATION.md`
Related:
- AC-004

### Statement

The detailed human planning corpus is preserved under
`.agents/archive/planning-corpus/` with 105 Markdown files and internal links
rewritten to the archived location.
