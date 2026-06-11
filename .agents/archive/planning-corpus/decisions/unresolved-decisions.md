# Unresolved Decisions

This document summarizes what remains unresolved after the Phase 0 validation
loop on 2026-06-10.

Current readiness: READY FOR FREEZE-AND-PLAN.

Implementation planning may begin. Public release readiness is not claimed.
The remaining work is either implementation-time validation, release/platform
validation, or low-severity documentation cleanup.

`.agents/archive/planning-corpus/reviews/finding-resolution.md` was requested as review input but is not
present in this workspace.

## No Remaining Pre-Implementation Blockers

The previous BLOCKER and HIGH findings are no longer unresolved
pre-implementation gates:

- FINDING-001 is resolved by selecting the hardened OpenCode adapter with
  mandatory instruction/config disclosure. Direct, unconstrained OpenCode is
  not the MVP path.
- FINDING-002 is resolved for planning by live pre-execution interception and
  structured denial evidence for file writes, shell commands, and
  runtime-proposed Git state changes.
- FINDING-003 is resolved for planning by the concrete shell security policy.
  Implementation must still add code-level tests for the required rules.
- FINDING-004 is resolved by runtime OpenRouter routing evidence plus a scope
  change: strict hidden-context exclusion is replaced by disclosed bounded
  context-source categories.
- FINDING-005 is resolved by the instruction-loading disclosure decision.
  OpenCode-backed sessions must not claim invisible display-only instruction
  behavior.
- FINDING-014 is resolved by the accepted fallback decision tree.
- FINDING-015 is resolved by the macOS-first MVP platform matrix.
- FINDING-011 is resolved by separating Phase 0 architecture gates from product
  QA acceptance.

## Remaining Implementation-Time Validation

These items are required during implementation but do not block starting the
implementation phase:

- Adapter tests for structured events, approval routing, denial mapping, stop
  behavior, app-owned record mapping, and runtime-reference persistence.
- Instruction-source inventory and disclosure tests, including `/config`
  redaction and block-on-undisclosable-source behavior.
- Shell-policy tests for environment filtering, redaction, truncation,
  destructive classification, unclassifiable-command handling, and output
  omission on redaction failure.
- OpenRouter credential-reference tests that verify active sessions keep their
  starting keychain reference and block update/revoke while running.

## Remaining Release Or Milestone Validation

These items require a built app or prototype:

- FINDING-009 Tauri UI validation on the mandatory launch target.
- Text-like artifact preview validation in the real Tauri shell.
- End-to-end product QA for project open, session lifecycle, approvals, shell
  execution, file writes, Git diffs, credential setup, restart persistence, and
  instruction-source disclosure.
- Windows 11 x64 compatibility validation before public MVP release.

## Remaining Low-Severity Documentation Cleanup

### FINDING-012: Preplan Brief Still Says To Create Plans Even Though Corpus Exists

Status: unresolved, low severity.

Resolution path: mark `.agents/archive/planning-corpus/preplan-brief.md` historical or add a note that
later MVP documents supersede it where they conflict.

### FINDING-013: ADR Numbering And Rollup Naming Are Hard To Follow

Status: partially resolved, low severity.

Resolution path: add standalone ADR files or update ADR indexes/statuses for
rollup-only decisions if the corpus needs cleaner navigation.

## Current Readiness Summary

Remaining BLOCKER: none.

Remaining HIGH: none as pre-implementation gates.

Remaining MEDIUM before implementation: none.

Remaining implementation/release gates:

- FINDING-009 app-build UI validation.
- Code-level adapter, shell-policy, credential, and disclosure tests.
- Product QA journeys after implementation.

Remaining low/documentation cleanup:

- FINDING-012.
- FINDING-013.

## Next Step

Proceed to freeze-and-plan. The implementation backlog should explicitly carry
the deferred validation items as engineering tasks and release gates.
