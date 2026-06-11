# Requirements

## REQ-001: MVP Coding Workspace

Status: accepted
Confidence: evidence-backed
MVP: yes
Source: `.agents/archive/planning-corpus/mvp/mvp-freeze.md`, `CONTEXT.md`
Related:
- CAP-001
- AC-001

### Statement

C4OS MVP is a single-user local desktop app for one selected local Git project
and one active agent session at a time.

### Rationale

This is the narrow validation loop frozen for implementation.

### Acceptance Links

- `.agents/archive/planning-corpus/acceptance/project-management.md`
- `.agents/archive/planning-corpus/acceptance/sessions.md`

## REQ-002: Backend-Owned Safety Boundary

Status: accepted
Confidence: evidence-backed
MVP: yes
Source: `.agents/archive/planning-corpus/mvp/mvp-freeze.md`
Related:
- CAP-002
- AC-002

### Statement

The backend owns filesystem access, shell execution, Git state-changing policy,
process supervision, secrets, approvals, persistence, and runtime adapter
control.

### Rationale

The UI must display enforced state, not become the policy authority.

### Acceptance Links

- `.agents/archive/planning-corpus/acceptance/security-and-permissions.md`
- `.agents/archive/planning-corpus/acceptance/file-access.md`
- `.agents/archive/planning-corpus/acceptance/shell-execution.md`

## REQ-003: OpenRouter-Only Model Path

Status: accepted
Confidence: evidence-backed
MVP: yes
Source: `.agents/archive/planning-corpus/mvp/mvp-freeze.md`, `CONTEXT.md`
Related:
- CAP-003
- AC-003

### Statement

MVP model-backed sessions route through OpenRouter with credentials stored only
in OS keychain or platform credential storage.

### Rationale

Direct provider integrations and local models are post-MVP.

### Acceptance Links

- `.agents/archive/planning-corpus/acceptance/openrouter-integration.md`

## REQ-004: Hardened OpenCode Adapter

Status: accepted
Confidence: evidence-backed
MVP: yes
Source: `.agents/archive/planning-corpus/mvp/mvp-freeze.md`
Related:
- CAP-004
- AC-004

### Statement

MVP uses a hardened OpenCode adapter path for launch, event ingestion,
permission routing, stop mapping, config redaction, and adapter references.

### Rationale

Unconstrained direct OpenCode use cannot satisfy the MVP safety and persistence
model.

### Acceptance Links

- `.agents/archive/planning-corpus/acceptance/agent-execution.md`
- `.agents/archive/planning-corpus/validation/`

## REQ-005: Reviewable Local Work

Status: accepted
Confidence: evidence-backed
MVP: yes
Source: `.agents/archive/planning-corpus/mvp/mvp-freeze.md`
Related:
- CAP-005
- AC-005

### Statement

Users can inspect tool activity, approval results, changed files, file diffs,
and text-like artifacts after agent work.

### Rationale

Trust and control are core MVP validation goals.

### Acceptance Links

- `.agents/archive/planning-corpus/acceptance/git-integration.md`
- `.agents/archive/planning-corpus/acceptance/artifacts.md`

