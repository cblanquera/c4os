# Task 00004 — Runtime Adapters, Providers, And Capability Lifecycle

Status: verified

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

Result: passed — the production Rust composition, exact OpenCode and Pi peers, provider/capability/session lifecycle, private credential and attachment delivery, Action Gateway continuation, cleanup, rendered QA, and final rebuilt-bundle evidence all passed. Independent review reported P0 = 0 and P1 = 0.

Required evidence: exact lock versions; upstream primary-source decisions; conformance/native test output; process traces; Action Gateway integration; no direct worker effects; rendered provider/model/runtime/conflict/retry/degraded states; evidence paths/commands; residual version risk.

## Implementation Notes

Implemented 2026-07-19 through 2026-07-20. RBL-003, RBL-004, and RBL-009 through RBL-013 govern the exact peer-runtime and native-evidence decisions.

The task-owned production surface now includes:

- Rust-owned provider profiles, route identity, availability/connectivity, zero/one/many model discovery, effective capability evidence, atomic coordinator/capability generations, and fail-closed preflight.
- Atomic Workspace/root/runtime-installation binding; provisional Chat promotion; immutable User Turns and Run Attempts; streaming, cancellation, retry ancestry, durable recovery, stale-correlation rejection, and unknown-effect review.
- Descriptor-rooted attachment materialization with symlink substitution resistance, exact byte/digest/version checks, per-file/count/aggregate limits, and exact native image/PDF parts where supported.
- A production `RuntimeProductionApplication<RuntimeProductionBootstrap>` reachable from Tauri activation, shutdown, pump, snapshot, and approval commands, including zero-provider activation and fail-closed dispatch.
- Exact OpenCode `1.18.3` and SDK `1.18.3` assets, authenticated fixed loopback, isolated state, one-use descriptor-bound server/provider credentials, exact C4OS broker tools, Action Gateway continuations, and real process-group/descendant cleanup.
- Exact Pi `0.80.10` coding-agent/core/AI graph, C4OS-owned Node sidecar, internal provider mapping, one-use credential delivery, supported image input, normalized streaming/tool events, allow/deny receipt delivery, and sidecar cleanup.
- Strict renderer/Tauri runtime envelopes plus QA-gated provider, model-preflight, runtime-selection, approval, recovery, and responsive review surfaces.

## Verification Notes

Passed on the frozen merged tree.

Static and deterministic gates:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- Focused Rust targets for adapter conformance/behavior, providers, capabilities/evidence, attachments, session/attempt lifecycle, supervision/coordinator/dispatch/persistence, Action Gateway bridging, OpenCode adapter/assets/broker/credential/native/SDK, Pi adapter/process, and production composition.
- Focused counts included Rust library `46 passed, 3 ignored`, `pi_adapter` `23 passed`, `pi_process` `5 passed, 1 ignored`, `runtime_dispatch` `19 passed`, and deterministic `runtime_production` `3 passed, 5 ignored`.
- OpenCode SDK sidecar `27/27`, OpenCode native-source/build sidecar `10/10`, OpenCode launcher `2/2`, and Pi sidecar `44/44`.
- `cargo test --workspace --all-targets -- --test-threads=1`
- `npm run format:check`, `npm run lint`, `npm run typecheck`, `npm run test:unit` (`10` files, `41` tests), `npm run protocol:check`, and `npm run test:e2e` (`8/8`).
- Root, OpenCode SDK, OpenCode native, and Pi `npm audit --audit-level=high`: zero vulnerabilities in all four graphs.
- `cargo audit`: zero vulnerabilities; 17 previously accepted transitive warnings remain recorded in RBL-007.

Final bundle and native gates:

- `VITE_C4OS_QA_FIXTURES=1 VITE_C4OS_QA_ENTRY=runtime npm run tauri:build` rebuilt `target/debug/bundle/macos/C4OS.app`; source pins and post-copy completed.
- `npm run bundle:opencode-sdk:verify`, `npm run bundle:opencode-assets:verify`, and `npm run bundle:pi:verify` passed the exact bundled-tree/version checks.
- Ignored OpenCode native suite: `7/7`; authenticated OpenCode stream suite: `3/3`.
- Packaged production bootstrap, packaged Pi ready/pump/shutdown, exact OpenCode attach-CAS descendant cleanup, and exact Pi attach-CAS cleanup each passed against the rebuilt bundle.
- Opt-in macOS login-Keychain round trip passed; the exact disposable `dev.c4os.live-test.task00004-final-20260720-01.credential-vault` item was deleted and absence was verified immediately afterward.
- Final `tools/run-task-00004-native-golden.zsh` rerun passed in `88.17s` after the last app rebuild. It recorded seven TLS 1.3 requests across the OpenCode allow/complete/deny/complete and Pi allow/deny/complete stages, exact image digest `sha256:431ced6916a2a21a156e38701afe55bbd7f88969fbbfc56d7fe099d47f265460`, the exact `c4os_propose_action` and `c4os_read_resource` tools, one provider-credential digest with no raw secret, allow/deny receipts, durable completion, and process cleanup.

Evidence:

- `output/native/task-00004-provider-evidence.json`
- `output/native/task-00004-providers.jpeg`
- `output/native/task-00004-model-preflight-blocked.jpeg`
- `output/native/task-00004-model-preflight-compatible.jpeg`
- `output/native/task-00004-runtime-selection.jpeg`
- `output/native/task-00004-recovery-blocked.jpeg`
- `output/native/task-00004-recovery-ready.jpeg`
- `output/native/task-00004-recovery-retried.jpeg`
- `output/native/task-00004-recovery-cancelled.jpeg`
- `output/native/task-00004-approval-pending.jpeg`
- `output/native/task-00004-approval-allowed.jpeg`
- `output/native/task-00004-approval-denied.jpeg`
- `output/native/task-00004-responsive-820x620.jpeg`

Computer Use on the rebuilt native app verified provider profiles and zero/one/many discovery, blocked/compatible attachment preflight, OpenCode/Pi draft-save and restart generation, first-submit Chat binding, recovery conflicts, retry/cancel history, and the exact `820x620` responsive layout. The native app exposed no fabricated approval. The QA-gated policy surface supplied pending/allow/deny visual evidence; the real provider continuation was proven separately through the production golden path. A browser-only attempt to settle a runtime approval without Tauri failed closed as designed.

The cumulative coordinator objective meter read `75,092s` (`20h 51m 32s`) when final documentation began. Earlier task epochs did not instrument category totals separately, so only the final-stage minima are auditable: two final app builds took about `181s` total; the post-build golden runs took `88.95s` and `88.17s`; the final OpenCode native suite took `57.23s`; final OpenCode/Pi attach-CAS cleanup took `54.19s` and `54.03s`; packaged Pi ready/pump/shutdown took `54.09s`; packaged bootstrap took `28.60s`; and the final login-Keychain test took `2.00s`. Task 00005 should start a per-command timing ledger before implementation.

## Agent Acceptance Notes

The independent final Task 00004 audit reported P0 = 0 and P1 = 0 across production reachability, exact attachment bytes, Rust-derived provider credentials, atomic capabilities/snapshots, approval continuation, real attach-failure cleanup, secret absence, and zero/one/many provider routing.

Residual risks accepted for this task:

- JavaScript cannot guarantee zeroization of immutable strings after transient credential parsing; secrets remain bounded to one-use private channels and are excluded from renderer, argv, inherited environment, logs, configuration, and persistent runtime state.
- Pi requires Node `--allow-net`; Node's permission model does not scope that grant to a destination. All production routes remain Rust-validated and the sidecar has no C4OS policy or persistence authority.
- Pi's current composite tool IDs use a safe `|` form and hash correctly. A future upstream ID containing `:` or `/` would fail closed at the stricter Rust Action Gateway grammar until the canonicalizer is revised.
- One stale comment in `dispatch.rs` describes a known-terminal tail as stale even though exact known inactive-session terminal frames are intentionally dropped; unknown sessions still fail closed.
- OpenCode and Pi upgrade rapidly. Any version, build-input, plugin/auth, SDK-event, permission, or tool-ID change requires a new compatibility row and the full native/secret/cleanup matrix.

Invalidated or repaired runs are retained as process evidence rather than counted as acceptance: Bun inherited-listener adoption, readiness descendant retention, pure-plugin disablement, `.mjs` glob exclusion, zero model limits, `/var` canonicalization, Bun socket adoption, macOS subsecond socket timeout, broker disposal, fixture identity gaps, Project-scope drift, lossy Keychain parsing, packaged Pi zero-provider setup, machine-trust certificate experiments, stale/duplicate OpenCode terminal frames, Pi permission/tool-ID/receipt/delta mismatches, an incorrect one-tool fixture assertion, an ESLint global-timer error, sandboxed loopback/browser attempts, and one invalid OpenCode-launcher npm script invocation were all diagnosed before the passing replacement command.
