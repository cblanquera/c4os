# Task 00007 — Workspace, Chat, Composer, And Conversation

Status: verified

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

Result: passed — P0=0, P1=0, P2=1. The non-blocking P2 records that pending, streaming, failure, and responsive states are covered by the production browser matrix rather than separate exact-native screenshots.

Required evidence: focused automated results; authoritative snapshot and persistence inspection; production screenshots for normal, search, pending, streaming, failure, focus, and responsive states; keyboard/accessibility results; console/overflow checks; evidence paths, commands, and limitations.

## Implementation Notes

Started 2026-07-21 from verified checkpoint `ff85865`. r013 is behavioral intent only; its DOM snapshots, timers, localStorage authority, and simulated operations did not enter production. The smallest production-composed golden path is one Rust-authoritative active Workspace/Project projection creating one memory-only pending Chat, promoting it atomically on the first valid text or attachment submission into the existing Workspace database/session repository, dispatching through the already verified runtime coordinator, and rendering the resulting immutable user turn plus normalized work/final events in the Task 00006 shell. Search, attachments, model controls, modes, Reply, artifact focus, and failure states extend that same path without introducing renderer authority or a second session store.

The stabilized implementation retains every retry attempt in authoritative order, persists immutable attachment reference numbers and their monotonic next counter across restart, and restores durable Reply state without permitting delayed bootstrap, attachment, submit-success, or submit-failure responses to overwrite newer renderer-local Reply intent. Session- and Project-changing commands may replace the target because the previous Chat target is no longer valid.

## Verification Notes

Focused renderer verification passed after the final Reply race repair: 5 files and 31 tests. TypeScript type checking, ESLint, Prettier, and `git diff --check` passed. The complete frontend regression passed with 39 files and 184 tests; the production Playwright matrix passed 32/32 after its preview server was allowed to bind locally. The complete Rust workspace, clippy-with-warnings-denied, Rust format, and generated-protocol checks had already passed after the Rust contracts stabilized; no unchanged Rust gate was rerun after the renderer-only race repair.

The exact non-QA debug application rebuilt successfully after the final renderer repair. Native acceptance used an isolated production-service home and verified Workspace/Project/Chat restore, transcript and provenance, safe work activity, focused contextual Chat, focus restoration, search, Settings round trip, rename-dialog semantics, durable Reply across two process launches, and native menus. The durable database record retained `replyTargetId: turn:acceptance`. See `.agents/resources/native/task-00007-acceptance.md` and its seven review images.

## Agent Acceptance Notes

Independent read-only Agent Acceptance passed with P0=0 and P1=0. P2=1 is limited to the absence of separate exact-native screenshots for pending, streaming, failure, and responsive states; the production renderer and 32/32 browser matrix cover those states. File/Folder artifacts, Terminal, native Browser, Plugin/Skill, MCP, complete Settings, updates/recovery/diagnostics, and final QA/security/accessibility/coverage audits remain explicitly deferred to Tasks 00008 through 00015C.
