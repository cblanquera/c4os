# Task 00014 — Updates, Recovery, Diagnostics, And Degraded-State Integration

Status: open

Coverage: cross-cutting normative requirements; supporting work for UX-010, UX-012, UX-013, UX-015, SET-002, SET-007 through SET-011, QA-002.

## Summary

Integrate independent application, runtime, and Plugin update channels; last-known-good and rollback behavior; startup/restart recovery; redacted diagnostics; and honest degraded/error presentation for the complete local development build.

## Implementation Steps

1. Implement separate version discovery and local-development update state for the app, OpenCode/Pi runtimes, and Plugins without signing, notarization, publishing, or distribution claims.
2. Implement transactional activation, last-known-good selection, rollback, revocation, interrupted-operation journals, startup recovery, and bounded cleanup.
3. Implement structured redacted diagnostics, crash/failure correlation, user-visible recovery actions, export-safe records, and secret/provenance exclusions.
4. Integrate authoritative degraded, unavailable, stale, conflicted, revoked, restarting, and recovered states across the renderer.
5. Exercise database, archive, runtime, extension, MCP, Browser, Terminal, credential, and renderer failure/recovery matrices.

## Verification Process

- Fault-injection tests for interrupted activation, corrupt state, crash loops, stale generations, revocation, recovery, cleanup, and last-known-good behavior.
- Security tests for redaction and absence of raw secrets/environment values in logs, UI, diagnostics, and exported evidence.
- Production walkthroughs of visible failure/degraded/recovery states with restart, console, accessibility, and overflow assertions.

Acceptance criteria: none — implementation acceptance is delegated to the coordinator’s Agent Acceptance process for Spec 00003.

## Agent Acceptance

Result: failed — integrated update, recovery, and diagnostics evidence is absent.

Required evidence: fault-injection and redaction results; transactional/journal inspection; restart/recovery logs; production failure-state screenshots; console/accessibility/overflow checks; evidence paths, commands, and limitations.

## Implementation Notes

Not started. Distribution signing, notarization, signed updater evidence, publishing, and deployment remain explicit external gates.

## Verification Notes

Not run.

## Agent Acceptance Notes

Happy-path-only behavior cannot pass this task.
