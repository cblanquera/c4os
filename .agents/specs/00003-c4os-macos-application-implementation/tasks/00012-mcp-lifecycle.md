# Task 00012 — MCP Lifecycle

Status: open

Coverage: SET-009; support for UX-010, UX-012, UX-015 and runtime capability discovery.

## Summary

Implement the MCP 2025-11-25 lifecycle for supervised STDIO and authenticated Streamable HTTP servers, including definitions, secrets, trust, capabilities, policy mediation, bounded execution, restart, revocation, and failure UX.

## Implementation Steps

1. Complete bounded primary-source research and lock a compatible maintained MCP implementation for the frozen 2025-11-25 protocol.
2. Define durable server records with transport-specific fields, opaque credential references, trust state, scope, timeout/output bounds, and lifecycle generations.
3. Implement supervised STDIO and authenticated Streamable HTTP initialize, sessions, tools, resources, sampling, notifications, cancellation, restart, and recovery.
4. Route every authority-bearing MCP operation through capability preflight, PolicyService, ActionGateway, audit, redaction, revocation, and stale-generation rejection.
5. Implement Settings add/edit/test/enable/disable/details/failure/recovery states and hostile deterministic fixtures.

## Verification Process

- Protocol conformance and transport tests for initialize, sessions, capabilities, authentication, cancellation, restart, notifications, hostile content, timeouts, and output limits.
- Denial-before-effect, secret isolation, audit, revocation, crash, and recovery tests.
- Production Settings and Chat walkthroughs across lifecycle, responsive, accessible, error, console, and overflow states.

Acceptance criteria: none — implementation acceptance is delegated to the coordinator’s Agent Acceptance process for Spec 00003.

## Agent Acceptance

Result: failed — production MCP evidence is absent.

Required evidence: primary-source decision and exact lock; protocol/transport/security results; supervised process/session logs; persisted redacted audit; production screenshots; accessibility/overflow/console checks; evidence paths, commands, and limitations.

## Implementation Notes

Not started. The disposable STDIO proof is useful scenario input but is not 2025-11-25 production conformance.

## Verification Notes

Not run.

## Agent Acceptance Notes

Both transports and their failure/recovery paths must be exercised.
