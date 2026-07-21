# Task 00008 — Artifact Framework And File/Folder Facilities

Status: verified

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

Result: passed — P0=0, P1=0, P2=0 after independent inspection, scoped repairs, and final re-audit.

Required evidence: automated results; schema/provider inspection; temporary-root operation logs and persisted audits; File/Folder/context screenshots across normal and failure states; keyboard/accessibility/overflow/console checks; evidence paths, commands, and limitations.

## Implementation Notes

Started 2026-07-21 from verified checkpoint `e638380`. Historical proofs that place Chat state outside a Workspace or delete removed Chats are explicitly excluded. The smallest production-composed golden path is one trusted-root text file selected through a Rust-owned opaque grant, read into one durable versioned File artifact, projected through a shared typed provider/shell, explicitly focused without cloning the transcript, edited as a retained draft, and saved by one generation/hash-checked atomic write through the existing Action Gateway. Folder listing, immutable Reply context, history, degraded/unknown states, and broader File/Folder behavior must extend this path without giving the renderer filesystem or persistence authority.

## Verification Notes

Production verification is complete. Focused artifact and renderer tests passed, including the final 12/12 File/Folder selection-lifecycle slice; the complete frontend matrix passed with 216/216 unit tests and 38/38 Playwright tests; `cargo test --workspace --all-targets -- --test-threads=1`, Clippy with warnings denied, and formatting passed; and the exact non-QA `C4OS.app` rebuilt successfully after the final source repair.

Native acceptance used real macOS File/Folder panels and disposable trusted roots. It covered File read/edit/approval/atomic save, Folder traversal and conversion, immutable Reply context, focus composition, optimistic conflict and retained draft, restart continuity, and POSIX mode preservation. A dedicated process-restart run proved that the security gateway cancels its non-reconstructable prompt while the durable artifact marker safely revalidates and issues a fresh Ask; Allow then completed the exact write, cleared the marker, advanced the File version, and retained mode `0751`. Detailed commands, identities, persisted audit data, limitations, and screenshots are in `output/native/task-00008-acceptance.md`.

## Agent Acceptance Notes

The coordinator observed real brokered operations and restart continuity. Initial independent review reported P0=0, P1=4, and P2=3. All seven findings received scoped production repairs and regression/native evidence. A later re-audit found two renderer selection-lifecycle edges; those also received scoped repairs and positive/negative regressions. The same independent reviewer then passed the task at P0=0, P1=0, P2=0.
