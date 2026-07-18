# Task 00004 — Runtime Adapters, Providers, And Capability Lifecycle

Status: open

Coverage: supports UX-007, UX-010, UX-012, CHAT-003 through CHAT-007, SET-001, SET-004 through SET-006.

## Summary

Implement provider/model routes, effective capabilities, first-submit binding, immutable turns/attempts, the Rust supervisor, and peer OpenCode/Pi adapters against current pinned upstream versions.

## Implementation Steps

1. Implement provider profiles, opaque credentials, test/discovery, model routes, declared/normalized/observed/effective capability descriptors, checked time, availability, and preflight.
2. Implement provisional Chat promotion, immutable User Turns, Run Attempts, route/config/resource/capability snapshots, streaming, cancellation, Retry ancestry, stale correlation rejection, and recovery.
3. Implement RuntimeSupervisor version/install/state/process-generation/health/restart/shutdown/descendant cleanup and compatibility records.
4. Implement authenticated isolated OpenCode `1.18.3` loopback adapter with typed SDK/OpenAPI, tool requests routed only through Action Gateway, and per-request authority narrowing.
5. Implement the maintained Pi `0.80.10` SDK sidecar wrapper with C4OS tools, barrier-safe normalized events, no Pi authority/persistence ownership, cancellation, and health.
6. Share one adapter conformance suite; record exact native versions and supported/degraded/unknown capability behavior.

## Verification Process

- Provider field/test/discovery and zero/one/many model matrices.
- Shared adapter contract tests plus exact-version native OpenCode and Pi integration tests.
- Process supervision, restart, descendant cleanup, stale events, incompatible versions, secret delivery, redaction, cancellation, retry, unknown side effects, and recovery tests.
- Capability preflight, model-switch conflict, attachment conversion/removal/cancel, reasoning controls, context limits, and first-submit binding tests.

Acceptance criteria: none — implementation acceptance is delegated to the coordinator’s Agent Acceptance process for Spec 00003.

## Agent Acceptance

Result: failed — production adapter and capability evidence is absent.

Required evidence: exact lock versions; upstream primary-source decisions; conformance/native test output; process traces; Action Gateway integration; no direct worker effects; rendered provider/model/runtime/conflict/retry/degraded states; evidence paths/commands; residual version risk.

## Implementation Notes

Not started. RBL-003 and RBL-004 govern initial upstream pins.

## Verification Notes

Not run.

## Agent Acceptance Notes

OpenCode and Pi remain architectural peers even if implemented sequentially.
