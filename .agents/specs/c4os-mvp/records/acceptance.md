# Acceptance Criteria

## AC-001: MVP Acceptance Set

Status: accepted
Confidence: evidence-backed
MVP: yes
Source: `.agents/archive/planning-corpus/mvp/mvp-freeze.md`
Related:
- REQ-001

### Statement

MVP completion depends on the required acceptance documents listed in
`.agents/archive/planning-corpus/mvp/mvp-freeze.md`.

### Acceptance Links

- `.agents/archive/planning-corpus/acceptance/openrouter-integration.md`
- `.agents/archive/planning-corpus/acceptance/project-management.md`
- `.agents/archive/planning-corpus/acceptance/file-access.md`
- `.agents/archive/planning-corpus/acceptance/security-and-permissions.md`
- `.agents/archive/planning-corpus/acceptance/shell-execution.md`
- `.agents/archive/planning-corpus/acceptance/agent-execution.md`
- `.agents/archive/planning-corpus/acceptance/sessions.md`
- `.agents/archive/planning-corpus/acceptance/git-integration.md`
- `.agents/archive/planning-corpus/acceptance/artifacts.md`
- `.agents/archive/planning-corpus/acceptance/skills.md`
- `.agents/archive/planning-corpus/acceptance/settings-and-configuration.md`
- `.agents/archive/planning-corpus/acceptance/telemetry-and-diagnostics.md`

## AC-002: Progress Migration Complete

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.task-bank/`, `.agents/progress/`
Related:
- DEC-004

### Statement

The legacy progress bank is ported to `.agents/progress/`, including manifest,
brief, decisions, conventions, outputs, CSV progress sheet, batches, item
packets, and logs.

## AC-003: Source Retirement Decision Exists

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/specs/c4os-mvp/generated/source-retirement-review.md`
Related:
- DEC-004

### Statement

The source-retirement review explicitly classifies `.task-bank/` and other
planning sources before any removal.

## AC-004: Human Planning Corpus Archived

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/archive/planning-corpus/MIGRATION.md`
Related:
- EVD-005

### Statement

The detailed human planning corpus is archived under
`.agents/archive/planning-corpus/`, compact routing facts are available under
`.agents/specs/c4os-mvp/`, and future workers can use `.agents` without reading
the repository-root human planning directory.
