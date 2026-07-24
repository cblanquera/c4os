# Task 00002 — Durable Core, Configuration, And Workspace Lifecycle

Status: verified

Coverage: UX-013, UX-014, SET-002; persistence support for session, artifact, provider, policy, extension, and update tasks.

## Summary

Implement the Rust-owned database actor, durable records, migrations, physical layout, scoped configuration, portable Workspace archive, locking, save, recovery, recents, project paths, and inactivation semantics.

## Implementation Steps

1. Implement app and Workspace SQLite ownership, schemas, transactions, bounded reads, WAL/full-sync settings, compiled migrations, online backup, and rollback diagnostics.
2. Implement C4OS Home and Workspace layouts from IS-007 with no duplicate writable owner.
3. Implement strict TOML schemas, scope subsets, precedence, managed ceilings, watching, last-known-good activation, generation-checked atomic saves, and diagnostics.
4. Implement defensive zip manifest/extraction limits, advisory writer locking, authoritative working copy, generation recovery, online-backup repack, fsync, validation, and atomic rename.
5. Implement Open Folder, Open Workspace, Clone, Save Workspace, recents, add/relocate/reorder Projects, trusted roots, same-Project/different-Workspace isolation, and missing paths.
6. Implement Chat/Project/Workspace inactivation without deletion, purge/export, or process termination.

## Verification Process

- Rust unit/property/hostile-fixture tests for schema, migrations, archive traversal/duplicates/types/sizes/ratios/digests, disk/interruption failures, locks, recovery, and configuration conflicts.
- Restart and round-trip integration tests with two Workspaces referencing the same Project.
- Inspect files to prove secret, raw Browser, Project-folder metadata, and duplicate authority exclusion.
- Launch Start flows with deterministic native picker/archive fixtures and recovery notices.

Acceptance criteria: none — implementation acceptance is delegated to the coordinator’s Agent Acceptance process for Spec 00003.

## Agent Acceptance

Result: passed — the coordinator inspected the production source, diffs, filesystem behavior, rendered evidence, and complete verification matrix on 2026-07-19.

Required evidence: migrations and backup results; hostile archive matrix; restart/recovery paths; same-Project isolation; configuration precedence and stale-write conflicts; inactivation/no-delete proof; rendered Start/recovery states; evidence paths/commands; residual limits.

## Implementation Notes

Started 2026-07-18 and verified 2026-07-19. Production now owns app and Workspace database actors, exact compiled schemas and migrations, bounded complete snapshots, strict scoped TOML with one serialized generation authority, long-lived native-plus-content-poll watchers, portable archive preflight/staging/recovery, transactional create/open/save bookkeeping, exact inactivation compensation, recents, Project lifecycle, and the Workspace Start transport/renderer surface. Historical proofs with user-level shared Chats or destructive removal remain superseded feasibility artifacts; the Frozen Workspace-owned and inactivation-only contract governs production.

## Verification Notes

- `cargo fmt --all -- --check` passed.
- `cargo clippy --workspace --all-targets -- -D warnings` passed.
- `cargo test --workspace --all-targets` passed 90/90: 7 library, 16 configuration, 24 database, 13 protocol, 1 integration, 19 archive, and 10 Workspace service tests.
- The strengthened real Workspace/Project/Chat watcher test passed 11/11 consecutive runs, including atomic replacements with preserved mtimes.
- `npm run check` passed Prettier, ESLint, TypeScript, 25/25 Vitest tests, the Rust suite, and generated-protocol drift checking.
- `npm run test:e2e` passed 4/4 production/QA Workspace Start scenarios after rerunning outside the restricted loopback sandbox.
- `npm audit --audit-level=high` reported zero vulnerabilities. `cargo audit` scanned 460 locked dependencies with zero vulnerabilities and the 17 already-disposed target/maintenance warnings in RBL-007.
- `git diff --check` and the Agent Workspace validator passed; the validator retained only the unrelated pre-existing journey-file line-count warning.
- The full debug app bundle was built at `target/debug/bundle/macos/C4OS.app` and inspected through macOS Computer Use. Its Start route, accessibility tree, fail-closed Open Folder action, and narrow resize were correct.

## Agent Acceptance Notes

Coordinator inspection found no P0/P1 defects after three repair passes. Evidence includes `tests/results/playwright/task-00002-final-rebuilt-native-start.jpeg`, `tests/results/playwright/task-00002-native-workspace-start.jpeg`, `tests/results/playwright/task-00002-native-workspace-start-narrow.jpeg`, `tests/results/playwright/task-00002-native-action-fail-closed.jpeg`, `tests/results/playwright/task-00002-qa-workspace-start.png`, and `tests/results/playwright/task-00002-qa-workspace-recovery.png`.

The accepted residual limits are P2 and remain visible for later degraded-state integration: a failed post-commit watcher-target refresh retains the existing watcher and records degradation, but retries only on a later refresh-triggering operation and does not yet clear the stored error after repair; public creation rollback has production guarding plus direct active/recovery-root regression coverage, but coordinator-start and cleanup failures are not separately injected through the public creation API; Task 00002 Start actions intentionally fail closed until PlatformService and Action Gateway integration in Tasks 00003 and 00005.
