# Task 00008 — Artifact Framework And File/Folder Facilities

Status: open

Coverage: UX-008; ART-001, ART-003, ART-004, ART-007; support for CHAT-009.

## Summary

Implement the typed artifact-provider framework, shared shells and lifecycle, immutable Reply context snapshots, and brokered File and Folder facilities with capability-bounded native operations.

## Implementation Steps

1. Define versioned artifact state, provider registration, shared compact/expanded shells, history, loading/error/degraded/unknown-version states, focus transitions, and restart continuity.
2. Build immutable Artifact Context Snapshots with stable references, per-type capture, budgets, truncation disclosure, redaction, unsaved-state markers, brokered expansion, and stale/live conflict checks.
3. Implement File read, edit, draft, proposal, atomic write, approval, conflict, version history, breadcrumb, and recovery flows through WorkspaceService and ActionGateway.
4. Implement Folder bounded listing, traversal, refresh, synchronization, selection, conversion to File artifacts, and capability-safe path handling.
5. Integrate Git-aware diffs and completed artifacts without repository mutation outside explicitly authorized broker operations.
6. Add provider contract, persistence, policy, hostile-path, symlink/race, accessibility, responsive, and rendered-state tests.

## Verification Process

- Rust tests for path capabilities, symlink and time-of-check races, atomic writes, conflicts, snapshots, redaction, recovery, and audits.
- Renderer tests for provider contracts, shells, File/Folder workflows, Reply context, focus, unknown versions, and accessibility.
- Production Playwright and native walkthroughs using temporary trusted roots, with console and overflow assertions.

Acceptance criteria: none — implementation acceptance is delegated to the coordinator’s Agent Acceptance process for Spec 00003.

## Agent Acceptance

Result: failed — production artifact and facility evidence is absent.

Required evidence: automated results; schema/provider inspection; temporary-root operation logs and persisted audits; File/Folder/context screenshots across normal and failure states; keyboard/accessibility/overflow/console checks; evidence paths, commands, and limitations.

## Implementation Notes

Not started. Historical proofs that place Chat state outside a Workspace or delete removed Chats are explicitly excluded.

## Verification Notes

Not run.

## Agent Acceptance Notes

The coordinator must observe real brokered operations and restart continuity before passing this task.
