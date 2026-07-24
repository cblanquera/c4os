# Task 00014 — Updates, Recovery, Diagnostics, And Degraded-State Integration

Status: verified

Coverage: UX-010 and UX-013; supporting evidence for UX-015, SET-007 through SET-011, and QA-002.

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

Result: passed — the final local-development update, startup recovery, configuration recovery, diagnostics, degraded-state, rebuilt-native, restart, redaction, and cleanup matrices passed. Independent Rust/security, renderer/integration, and policy/configuration reviews each reported P0 = 0 and P1 = 0.

Required evidence is recorded in `.agents/resources/native/task-00014-acceptance.md`.

## Implementation Notes

Completed 2026-07-24 after the verified Task 00013 checkpoint `6f6672d`. Rust now owns separate application, runtime, and Plugin update channels; bounded candidate verification and content-addressed staging; transactional activation, revocation, rollback, cleanup, and recovery journals; startup boundary recovery; and structured redacted diagnostics. The renderer projects those authorities without receiving paths, secrets, raw provenance, or an activation action when native recovery requires an application or runtime rebuild.

Configuration coordination now keeps failed policy recovery sticky at both app and Workspace scope. Ordinary watcher errors, content rejection, unchanged files, and self-write deduplication cannot clear unresolved recovery; only proven compensation or successful observer reconciliation of the current durable last-known-good document may do so. Repeated stable content rejection settles instead of retrying forever, while exact diagnostics are deduplicated and bounded to the newest 128 records.

The macOS SQLite boundary canonicalizes an existing parent before opening the unchanged final filename with `NOFOLLOW`, so the lexical `/var` alias remains compatible while final-component symlinks stay denied. Backup inspection uses an already-open immutable descriptor source rather than reopening an attacker-substitutable path.

Distribution signing, notarization, signed updater evidence, publishing, deployment, and automatic public update discovery remain explicit external gates.

## Verification Notes

Complete renderer quality passed: formatting, lint, typecheck, 77 files and 441 unit tests, production and QA builds, and 43/43 Playwright scenarios. The serialized Rust workspace regression passed every runnable library, integration, and doc-test target; the two sandbox-only MCP STDIO fixture failures passed 2/2 in the exact host rerun. Final focused configuration regressions passed 10/10, the Workspace sticky-recovery regression passed 1/1, and the app sticky-recovery slice passed 3/3. Workspace check, exact protocol export, final debug app build, all three packaged-tree verifiers, ignored MCP/OpenCode/stream/packaged-runtime tiers, and the private-TLS native golden passed.

Final rebuilt-native acceptance passed healthy staged recovery, action suppression for `rebuild_application`, redacted export, 624 px responsive containment, controlled restart persistence, blocked startup recovery with no Continue path, retry/relaunch, persistence/log/diagnostic scans, original-home restoration, and final process cleanup. Exact commands, timings, durable state, hashes, and limitations are in `.agents/resources/native/task-00014-acceptance.md`.

## Agent Acceptance Notes

Final independent review results:

- Rust/security: P0 = 0, P1 = 0, P2 = 1, P3 = 0. The retained P2 is that `RestoreValidatedBackup` is not normally reachable because backup authority is captured only after successful Database startup; any future repair must preserve exact-Database scope and independently persisted, revalidated authority.
- Renderer/integration: P0 = 0, P1 = 0, P2 = 1, P3 = 0. The retained P2 is that pending native update operations require manual Refresh because the controller has no bounded polling or subscription.
- Policy/configuration: P0 = 0, P1 = 0, P2 = 1 after ledger alignment, P3 = 0. The retained P2 is that a successful Settings-based policy save can leave an older sticky app recovery notice visible until observer reconciliation, restart, or another externally activated change.

These P2s remain explicit inputs to Tasks 00015 through 00015C and do not weaken the fail-closed authority boundary. Agent Acceptance passed only after every independent reviewer reported P0 = 0 and P1 = 0.
