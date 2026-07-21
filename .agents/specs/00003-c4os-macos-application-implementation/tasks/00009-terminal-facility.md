# Task 00009 — Terminal Facility

Status: verified

Coverage: ART-005; support for UX-008, UX-010, UX-014, CHAT-007, ART-001, ART-006, ART-007.

## Summary

Implement one persistent, authorized portable PTY session per Chat with bounded streaming, immutable command artifacts, stdin, resize, Stop, recovery, and expanded terminal interaction.

## Implementation Steps

1. Lock the maintained portable-pty and xterm stack and define the typed terminal command/event protocol.
2. Authorize launch and every authority-bearing operation before PTY bytes or side effects, recording immutable process and environment provenance.
3. Supervise one PTY per Chat across artifact focus and Chat inactivation, with bounded output, backpressure, sequencing, resize, reconnect, and restart recovery.
4. Implement stdin, approved command execution, immutable command cards, snapshots, exit state, and Stop as process-group cancellation with exit 130 semantics.
5. Implement compact and expanded renderer states, keyboard/focus behavior, reduced motion, accessibility, failure recovery, and the accepted Terminal Reply behavior.
6. Add real PTY tests plus policy-denial, race, recovery, output-bound, and production-rendered walkthroughs.

## Verification Process

- Rust PTY/process-group integration tests with temporary roots and deterministic fixtures.
- Renderer component and production Playwright tests for streaming, stdin, resize, Stop, focus, reconnect, snapshots, and failure states.
- Manual native launch inspection proving authorization precedes bytes and Chat removal does not terminate the scoped process.

Acceptance criteria: none — implementation acceptance is delegated to the coordinator’s Agent Acceptance process for Spec 00003.

## Agent Acceptance

Result: passed — P0=0, P1=0, P2=1 after independent inspection, scoped repairs, and exact-source re-audit. The residual P2 is assigned to the cross-cutting Task 00015A security audit and does not fail this task's acceptance threshold.

Required evidence: exact dependency lock; automated results; authorization/audit sequence; real PTY transcript including Stop/130 and recovery; production screenshots; keyboard/accessibility/overflow/console checks; evidence paths, commands, and limitations.

## Implementation Notes

Started 2026-07-21 from verified checkpoint `1a51ac5`. The smallest production-composed golden path is one active Chat owning one Rust-supervised PTY rooted in its active Project: a direct Terminal `Run` must pass the Action Gateway before any command bytes reach the shell, create one durable immutable command artifact, stream bounded sequenced output, record terminal/environment/process provenance, complete with an exit result, and render through the existing shared artifact shell inline and focused. Persistent shell reuse, stdin, resize, Stop/130, Chat-inactivation continuity, restart recovery, and immutable Reply context must extend this path. The stale proof behavior that terminates a PTY when a Chat is removed remains explicitly excluded.

## Verification Notes

Production verification is complete. Focused Terminal projection tests passed 16/16, real portable-PTY/process-group tests passed 11/11, renderer Terminal tests passed 13/13, and all three production Playwright Terminal paths passed. The complete frontend matrix passed with 233/233 unit tests and 41/41 Playwright tests; the final Rust-only repairs did not change that frontend graph. Formatting, Clippy with warnings denied, the serialized full Rust workspace matrix, protocol generation, the exact non-QA `C4OS.app` rebuild, and all three packaged-sidecar verifiers passed on the final source.

Native acceptance used production Workspace, SQLite, Action Gateway, terminal supervisor, portable PTY, artifact, conversation, and xterm composition in disposable mode-`0700` homes. It covered approval-before-bytes, persistent cwd, partial streaming, stdin, resize, Stop/130, same-shell reuse, immutable Reply, restart Recovery, replacement generation, and active background reconciliation after Chat inactivation. In the final exact-bundle run, safe partial output was durable 1 ms after Running; the Chat became inactive 8.253 s later; final output arrived 51.762 s after inactivation; completion with exit 0 was durable 31 ms later; and the command-created Project marker existed. Detailed commands, identities, timestamps, limitations, and screenshots are in `output/native/task-00009-acceptance.md`.

## Agent Acceptance Notes

The coordinator observed real PTY effects, streaming, process interruption, restart recovery, and Chat-inactivation continuity. Initial independent review identified inactive-Chat reconciliation, partial streaming, invalid-UTF-8 bounding, transactional redaction, cleanup acknowledgement/retry, startup recovery, and bounded trigger-lookahead defects. Each received a scoped production repair and focused/full/native evidence. The final exact-source re-audit passed at P0=0, P1=0, P2=1. The remaining P2 records that current Terminal policy/audit metadata overstates Project-root containment for an unsandboxed retained shell; Task 00015A owns the cross-cutting classification and audit correction.
