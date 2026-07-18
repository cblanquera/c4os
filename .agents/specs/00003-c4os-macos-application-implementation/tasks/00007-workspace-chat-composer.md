# Task 00007 — Workspace, Chat, Composer, And Conversation

Status: open

Coverage: UX-005, UX-006, UX-007; CHAT-001 through CHAT-008; ART-006; support for CHAT-010.

## Summary

Implement the accepted r013 Workspace and Chat behavior against authoritative services: project/session navigation, Round 7 search, pending Chat promotion, transcript and Markdown composer, attachments, model controls, modes, Reply, branch presentation, focused artifacts, and streaming work history.

## Implementation Steps

1. Build Project and Chat navigation from WorkspaceService snapshots, including Add/Relocate, ordering, inactivation-only removal, expansion, selection, and the accepted flat non-empty search result replacement.
2. Implement pending Chat creation, first-turn promotion, title lifecycle, retry/cancellation, durable drafts, and per-Chat runtime/model binding.
3. Implement transcript rendering, a source-preserving Markdown composer, shortcuts, sanitization, undo, streaming, and fixed composer geometry.
4. Implement native-picker/drop attachments, immutable attachment records, compatibility filtering, preview/removal, restart continuity, and capability preflight.
5. Implement model/profile navigation, reasoning controls, effective-capability conflicts, Chat information, modes, direct operations, Activity, and provenance.
6. Implement Reply orchestration and focused-artifact continuity; Terminal Reply creates a new Terminal artifact in the same session.
7. Verify reduced-motion parity, failure states, stale-generation rejection, focus restoration, accessibility, responsiveness, and overflow.

## Verification Process

- Rust and renderer integration tests for Workspace/session persistence, search, pending promotion, runtime binding, attachments, preflight, cancellation, and stale generations.
- Vitest/Testing Library tests for composer, transcript, shortcuts, modes, Reply, focus, activity, and capability conflicts.
- Playwright production-build walkthroughs for all mapped r013 routes, widths, themes, keyboard paths, streaming states, errors, console, and overflow.

Acceptance criteria: none — implementation acceptance is delegated to the coordinator’s Agent Acceptance process for Spec 00003.

## Agent Acceptance

Result: failed — production Workspace and Chat evidence is absent.

Required evidence: focused automated results; authoritative snapshot and persistence inspection; production screenshots for normal, search, pending, streaming, failure, focus, and responsive states; keyboard/accessibility results; console/overflow checks; evidence paths, commands, and limitations.

## Implementation Notes

Not started. r013 is behavioral intent only; its DOM snapshots, timers, localStorage authority, and simulated operations must not enter production.

## Verification Notes

Not run.

## Agent Acceptance Notes

No mapped ID closes until service-backed behavior and production rendering both pass.
