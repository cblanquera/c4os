# Constraints

## CON-001: Frozen MVP Scope

Status: accepted
Confidence: evidence-backed
MVP: yes
Source: `.agents/archive/planning-corpus/mvp/mvp-freeze.md`
Related:
- REQ-001

### Statement

Anything not listed in frozen MVP features is out of scope for the first
implementation pass.

## CON-002: No Product Telemetry

Status: accepted
Confidence: evidence-backed
MVP: yes
Source: `.agents/archive/planning-corpus/mvp/mvp-freeze.md`, `CONTEXT.md`
Related:
- REQ-001

### Statement

MVP sends no product telemetry. Required OpenRouter model traffic is disclosed
separately.

## CON-003: No Strong Shell Sandbox Claim

Status: accepted
Confidence: evidence-backed
MVP: yes
Source: `CONTEXT.md`
Related:
- REQ-002

### Statement

Approved shell commands run as the current OS user with normal network access
and a backend-filtered environment. The app must not claim stronger sandboxing.

## CON-004: Legacy Bank Deprecated

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.task-bank/DEPRECATED.md`, user request
Related:
- DEC-004

### Statement

`.task-bank/` must not receive new progress updates after migration to
`.agents/`. `.agent/` must not receive new C4OS planning or progress updates;
it is a deprecated compatibility redirect for reusable skills that still
mention `.agent/`.
