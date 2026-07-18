# Task 00003 — Policy, Credentials, Action Gateway, And Execution Environments

Status: open

Coverage: UX-010, UX-012, UX-015, CHAT-009, CHAT-010, SET-011; security support for all native/runtime/extension facilities.

## Summary

Implement C4OS-owned policy, secure credentials, canonical action authorization, redacted audit, approval concurrency, Git/project boundary behavior, and Local/Docker/SSH executor contracts so every effect is denied or authorized before side effects.

## Implementation Steps

1. Implement four presets, seven category groups, concrete exceptions, managed/maximum ceilings, trusted-root/sandbox/repository classification, and fail-closed unknowns.
2. Implement OS-keychain-backed installation key, encrypted vault, opaque references, password/session fallbacks, reauthentication, immediate credential invalidation, short-lived operation delivery, and redaction.
3. Implement canonical actions, persisted single-use TTL authorization, exact arguments/target/workspace/session/runtime/environment/process-generation binding, mutation/replay/cancel/revocation expiry, and serialized within-run approvals.
4. Persist pre-effect intent/decision and post-effect normalized result; visibly queue independent-run approvals.
5. Implement Local/Docker/SSH exact-effect adapters and prove workers cannot reinterpret denial or substitute environment/targets.
6. Implement repository-sensitive Chat writes and brokered Git-safe branch operations with no automatic stash/commit/reset/revert/discard.

## Verification Process

- Scenario corpus and property tests for policy resolution, ceilings, exceptions, approval lifecycle, replay, concurrency, and stale versions.
- Real-boundary denial-before-effect tests including symlink escape, canonical-path TOCTOU, argument mutation, cancellation/revocation races, environment substitution, and crash/restart.
- Credential/log/diagnostic/export/argv/environment scans for secret leakage.
- Git/non-Git, in/out-of-Project, explicit Ask, safe dirty switch, conflict, and unchanged-worktree integration matrix.

Acceptance criteria: none — implementation acceptance is delegated to the coordinator’s Agent Acceptance process for Spec 00003.

## Agent Acceptance

Result: failed — production security evidence is absent.

Required evidence: automated matrices; source/diff inspection; real denial-before-side-effect traces; audit persistence; approval UI lifecycle screenshots; secret scans; Git fixture results; executor isolation; residual risks and exact commands.

## Implementation Notes

Not started. Proof tokens lack multiple production bindings and cannot be promoted as implementation.

## Verification Notes

Not run.

## Agent Acceptance Notes

Side quest 00015A independently repeats the security review after integration.
