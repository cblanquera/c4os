# Task 00009 — Terminal Facility

Status: open

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

Result: failed — production Terminal evidence is absent.

Required evidence: exact dependency lock; automated results; authorization/audit sequence; real PTY transcript including Stop/130 and recovery; production screenshots; keyboard/accessibility/overflow/console checks; evidence paths, commands, and limitations.

## Implementation Notes

Not started. The stale proof behavior that terminates a PTY when a Chat is removed must not be promoted.

## Verification Notes

Not run.

## Agent Acceptance Notes

Simulated timer output cannot satisfy this task.
