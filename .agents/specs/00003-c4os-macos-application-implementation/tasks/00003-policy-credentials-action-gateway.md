# Task 00003 — Policy, Credentials, Action Gateway, And Execution Environments

Status: verified

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

Result: passed — the coordinator inspected the production source, exact effect boundaries, durable security journal, renderer/native evidence, and complete Task 00003 verification matrix on 2026-07-19. The final independent security re-audit reported P0=0 and P1=0.

Required evidence: automated matrices; source/diff inspection; real denial-before-side-effect traces; audit persistence; approval UI lifecycle screenshots; secret scans; Git fixture results; executor isolation; residual risks and exact commands.

## Implementation Notes

Started and verified 2026-07-19. Production now owns four presets, seven composable policy groups, concrete exceptions, managed and maximum ceilings, fail-closed unknown classification, exact repository-sensitive Chat write rules, durable approval queues, canonical single-use authorization, and one atomic `consumed` plus `effect-started` executor barrier. Executor authority is a gateway-owned, moved-by-value, non-cloneable capability with mutually exclusive Local/Docker/SSH or Git tool/authority/digest families; neither tests nor another core caller can self-mint or replay it.

The app database schema v4 owns bounded current security state plus immutable cursor-paged events. Persisted action data uses typed redacted bindings and hashes for arguments, requested authority, canonical targets, versions, decisions, and changed targets. State transitions reject regression/rebinding and batch writes share one transaction. Restart cancels unreconstructable prompts and prompt-derived capabilities, restores only exact still-issued non-prompt verifiers, and converts interrupted effects into explicit unknown results.

Credentials use macOS Keychain custody for a random installation key, XChaCha20-Poly1305 authenticated encryption, Argon2id password mode, opaque references, explicit session-only fallback, reauthentication-gated import, zeroized buffers, and one-use operation delivery synchronized against replacement/removal. The opt-in live login-Keychain round trip passed on the unlocked Mac under a unique disposable service, and that exact test key was deleted immediately afterward.

The renderer includes production-built Advanced Policies and approval-activity review surfaces with seven groups, search, dirty/revert/save, exceptions, guardrails, and pending/queued/expired/denied/completed lifecycle fixtures. Proof tokens remain scenario inputs only and were not promoted as production authorization.

## Verification Notes

- `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings` passed.
- `cargo test --workspace --quiet` passed 149 deterministic Rust tests with the one explicitly opt-in live Keychain test excluded from that default tier. The Task 00003 security-focused rows passed 59/59: 9 Action Gateway, 10 deterministic credential, 15 execution/Git, 17 policy/authorization, and 8 security-persistence tests; database transition and migration coverage added another 24/24.
- `C4OS_LIVE_KEYCHAIN_TEST_BUNDLE=dev.c4os.live-test.task-00003-20260719 cargo test --test credential_vault live_macos_keychain_round_trip_is_opt_in -- --ignored --exact` passed 1/1. `security delete-generic-password` then deleted only `dev.c4os.live-test.task-00003-20260719.credential-vault` / `installation-master-key-v1`.
- `npm run typecheck`, `npm run lint`, and `npm test -- --run` passed; Vitest reported 8 files and 28/28 tests.
- `npm run test:e2e -- --grep "Advanced Policies|approval activity"` passed 2/2 against the production QA build after the required loopback-sandbox rerun.
- `npm audit --audit-level=high` reported zero vulnerabilities. `cargo audit` scanned 479 locked crates with zero vulnerabilities and the same 17 accepted target/maintenance warnings already dispositioned in RBL-007.
- `VITE_C4OS_QA_FIXTURES=1 VITE_C4OS_QA_ENTRY=policy npm run tauri -- build --debug --bundles app` built `target/debug/bundle/macos/C4OS.app`. macOS Computer Use launched that exact bundle, confirmed all seven policy groups and the complete lifecycle accessibility tree, and verified Deny changed the surface to `0 pending · 1 queued` with `No side effect was released to the worker.`
- `git diff --check` passed. The Agent Workspace validator passed with only its pre-existing journey-file line-count warning.

## Agent Acceptance Notes

The independent Task 00003 security re-audit passed with P0=0 and P1=0 after four adversarial repair passes. Evidence includes `output/playwright/task-00003-advanced-policies.png`, `output/playwright/task-00003-approval-lifecycle.png`, `output/playwright/task-00003-native-advanced-policies.png`, and `output/playwright/task-00003-native-approval-denied.png`.

Two accepted P2 hardening items remain visible for Task 00015A and degraded-state integration: terminal security rows are retained in the bounded current-state table, so reaching 4,096 rows fails closed instead of compacting terminal rows; and the generic database journal boundary validates envelope/index identity, digests, size, and transitions but does not independently deserialize every record kind into its typed payload schema. Neither permits an unauthorized effect, secret release, state regression, or journal rebinding. Side quest 00015A independently repeats the security review after full integration.
