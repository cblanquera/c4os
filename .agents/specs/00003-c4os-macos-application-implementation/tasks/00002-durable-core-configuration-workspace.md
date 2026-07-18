# Task 00002 — Durable Core, Configuration, And Workspace Lifecycle

Status: open

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

Result: failed — production persistence and Workspace evidence is absent.

Required evidence: migrations and backup results; hostile archive matrix; restart/recovery paths; same-Project isolation; configuration precedence and stale-write conflicts; inactivation/no-delete proof; rendered Start/recovery states; evidence paths/commands; residual limits.

## Implementation Notes

Not started. Historical proofs with shared/deleted Chats are explicitly non-authoritative.

## Verification Notes

Not run.

## Agent Acceptance Notes

Coordinator must inspect the schema, migrations, diff, archive code, filesystem effects, and recovery evidence directly.
