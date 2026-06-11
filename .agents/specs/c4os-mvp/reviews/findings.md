# Findings

Source: `.agent/specs/c4os-mvp/reviews/findings.md`
Migrated: 2026-06-12

## FIND-001: Product Boundary Is Frozen

Severity: resolved
Source: `grill-session.md`
Related:
- DEC-001

### Finding

The earlier general-purpose workspace ambiguity has been resolved into a
coding-first MVP.

## FIND-002: Runtime Control Is A Phase 0 Gate

Severity: blocker
Source:
`grill-session.md#20-question-what-must-opencode-prove-before-direct-mvp-use`
Related:
- Q-001
- TASK-001

### Finding

Direct OpenCode use remains blocked until runtime control and observability are
proven.

## FIND-003: Shell Output Contract Is Frozen

Severity: resolved
Source:
`grill-session.md#54-question-should-exact-shell-redactiontruncation-limits-be-defined-now`
Related:
- DEC-003
- Q-002

### Finding

The shell output contract is frozen, but exact redaction and truncation limits
remain spike work.
