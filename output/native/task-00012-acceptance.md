# Task 00012 MCP Lifecycle Acceptance Evidence

Date: 2026-07-23

Target: macOS 26.5.2 arm64, production-composed debug `C4OS.app`

Verified Task 00011 base: `e912e22`

Disposable acceptance home: `/private/tmp/c4os-task12-acceptance.XICVo2` (owner-only mode `0700`)

## Golden path

1. Rust owns one durable disabled MCP server definition. Saving never grants trust or starts a process.
2. The user reviews the exact executable digest or authenticated endpoint and explicitly allows one definition-bound trust prompt.
3. C4OS tests the exact MCP `2025-11-25` initialize/capability exchange, bounds and validates untrusted peer fields, then publishes one immutable tools/resources catalog.
4. Explicit enablement starts one supervised lifecycle generation. Tool/resource work and model sampling remain capability-, policy-, approval-, credential-, cancellation-, and audit-bound; peer instructions grant no authority.
5. Disable, failure recovery, restart, credential invalidation, and revocation cancel exact work, stop matching workers, retain durable redacted evidence, and reject stale generations.
6. The production Settings route projects empty, pending-trust, failure/recovery, ready, disabled, and revoked states, while Chat exposes only safe run-bound MCP provenance.

## Dependency and protocol lock

The exact resolved graph is `rmcp 2.2.0`, `tokio 1.53.0`, `reqwest 0.13.4`, and `process-wrap 9.1.0`. `rmcp` has default features disabled and enables only `client`, `transport-child-process`, `transport-streamable-http-client-reqwest`, and `reqwest`. C4OS independently enforces protocol `2025-11-25`, lifecycle generations, absolute pinned STDIO executables, sanitized environments, bounded framing/pagination/output, authenticated HTTPS destinations, no transparent session replacement, and post-validation of every peer field.

The SDK owns typed protocol and transport mechanics only. C4OS retains process supervision, credential delivery, destination policy, durable state, capability truth, Action Gateway mediation, redaction, audit, cancellation, and revocation.

## Automated verification

All implementation writers were frozen for the final matrix. Cargo-producing commands ran one at a time.

| Gate | Final result | Measured time |
| --- | --- | ---: |
| `npm test --prefix sidecars/pi` | 50/50 passed, including cancellation after credential claim, sampling settlement, operation-local credential delivery, replay denial, and zeroization | 1.49 s test time |
| `npm test` | 56 files and 297/297 tests passed | 7.21 s |
| `npm run test:e2e -- --workers=1` | 42/42 passed, including the production MCP route at 1100 and 622 px, exact secret-reference rendering, modal focus restoration, console/page-error absence, and document containment | 21.6 s |
| `npm run lint` | passed with zero errors | 4.62 s |
| `npm run typecheck` | passed | 3.26 s |
| `npm run format:check` | passed | 1.77 s |
| `cargo fmt --all -- --check` | passed | 1.10 s |
| `npm run protocol:generate` | passed; Rust protocol export 1/1 and generated MCP/Chat bindings refreshed | passed |
| `cargo test --workspace --no-fail-fast` | passed with no failures; library 221 passed and 4 expected ignored tiers, followed by every runnable integration and doc-test target | approximately 9 min 5 s wall time; library target 44.84 s |
| `cargo test -p c4os --test mcp_transport_fixtures -- --ignored` | authenticated Streamable HTTP 2/2 passed for session state, auth, bad credentials, protocol substitution, cancellation, restart, and bounds | 0.57 s test time |
| `cargo clippy --workspace --all-targets -- -D warnings` | supplementary non-baseline-clean gate: every Task 00012-introduced warning was repaired; the command still exits 101 on exactly 17 inherited warnings in previously accepted Task 00011-or-earlier code | not a required-matrix pass |
| `npm run tauri:build` | passed; rebuilt debug `target/debug/bundle/macos/C4OS.app` with the pinned sidecars and Node runtime | passed |
| `tools/run-task-00004-native-golden.zsh target/debug/bundle/macos/C4OS.app/Contents/Resources` | existing production OpenCode/Pi native provider golden path 1/1 passed after making its Python process-group wrapper safe when already invoked as a process-group leader | 55.07 s |
| Task 00012 native seed | passed with the exact validated MCP turn snapshot and production Workspace/session serialization | passed |

The first full Playwright matrix after adding MCP provenance correctly failed 13 legacy Chat/artifact/Terminal cases from one shared fixture-contract omission. The rendered failure said `MCP provenance is invalid`; adding explicit `mcpProvenance: null` to those historical turns fixed all 15 affected cases, and the complete 42-case matrix then passed. Unchanged failures were not blindly rerun.

The continuously measured final required-matrix commands above consumed about 10 minutes 42 seconds, excluding the separately timed native review capture interval, final app build, protocol-export traversal, setup, iterative focused tests, the supplementary Clippy run, and independent review. Native evidence captures span 17 minutes 17 seconds from the first Chat provenance image to the controlled restart image. Total Task 00012 wall time was not continuously machine-timed.

## Native macOS matrix

The final debug bundle launched through macOS LaunchServices with the debug-only validated acceptance-home switch. The home was created and seeded through production Workspace, database, session, and MCP snapshot validation paths.

| Check | Result |
| --- | --- |
| Production Chat provenance | passed; Run details displayed snapshot `42c95c98bb95…`, one captured tool from one user-configured server, and `task-00012-stdio/echo` without raw arguments, prompts, or credentials |
| Native Settings entry | passed; `Cmd+,` used the application menu and entered the single production Settings router |
| Empty state and create | passed; no-server state was distinct, and the real add dialog saved an exact disabled/untrusted STDIO definition with absolute executable, arguments, digest, scope, timeout, and output bound |
| Trust | passed; Review trust created one exact pending approval, Deny/Allow controls were visible, and Allow bound authority only to the reviewed definition digest |
| Containment failure | passed as a fail-closed negative case; testing a fixture stored under C4OS Home failed because an application-scoped MCP worker has no ambient read authority. The durable journal recorded `lifecycle.testing` then `lifecycle.failed`, and Settings exposed bounded failure/recovery state |
| STDIO test and discovery | passed after moving the fixture beside the pinned gitignored Node asset; editing invalidated trust, re-review was required, then Test negotiated `2025-11-25`, peer `c4os-stdio-fixture 1.0.0`, one tool, one resource, and Tools/Resources capabilities |
| Enable and process supervision | passed; enable projected Ready and exactly one active server process. Settings-to-Chat navigation preserved the worker until explicit disable; disable terminated it |
| Streamable HTTP schema | passed; the real dialog saved `https://mcp.example.test/rpc` with only environment reference `TASK12_MCP_TOKEN`. No bearer value was collected or rendered |
| Sampling | automated production-composed allow, deny-before-model, stale CAS, provider drift, pending cancel, in-flight cancel, timeout cleanup, persistence retry, and cleanup tests passed. The native acceptance home intentionally had no configured live model route, so no native sampling prompt or model call is claimed |
| Revoke | passed; revocation reason was recorded, the server became Revoked/Authority revoked, active work was cancelled, and no matching worker remained |
| Restart | passed after complete native teardown; the controlled relaunch restored Chat and then the exact HTTP pending/disabled and STDIO revoked states from SQLite generation 22 |
| Responsive and keyboard | passed at 1100×761 and 623×761; Settings remained contained, navigation compressed, MCP details stayed internally scrollable, Return opened Details, Escape closed it, and focus returned exactly to the Details button |
| Browser console and overflow | passed through the production-composed Playwright adapter: zero console warnings/errors, zero page errors, and no document-level horizontal overflow at 1100 or 622 px |
| Final cleanup | passed; the app, STDIO fixture, Pi sidecar, and matching MCP worker processes were absent after native Quit |

## Persisted audit

The app database retained MCP schema 1 at generation 22 with a 2,866-byte canonical document and SHA-256 `78b6f472ecaf9b2b360e12d0c01735734e0eb7f62eda92e7ef68781d10174933`.

It retained two user definitions:

- `task-00012-http`: application-scoped, Streamable HTTP, disabled, trust pending, lifecycle generation 1, zero active requests, and only environment reference `TASK12_MCP_TOKEN`.
- `task-00012-stdio`: application-scoped, STDIO, revoked, lifecycle generation 5, protocol `2025-11-25`, peer `c4os-stdio-fixture 1.0.0`, one tool, one resource, zero active requests, and final failure code `revoked`.

The immutable MCP journal contains 22 ordered events across service initialization, save, trust request/answer, the contained negative test, successful test, connect/enable, disable, edited-definition re-review, and revoke. The final events are `lifecycle.disabled` at generation 21 and `lifecycle.revoked` at generation 22.

The Workspace session record is schema 3/revision 7. Turn `turn:acceptance` retains one validated immutable MCP snapshot with service generation 7, exact Workspace/Project/session bindings, one safe tool descriptor, zero omitted tools, and digest `sha256:42c95c98bb95f50bd7d100d590af88f444f6bf0a07e609fea9c5baae3c4bd2e9`.

The final persistence scan covered app/workspace SQLite databases and WALs, configuration, browser state, MCP scratch, logs, and vault surfaces. Neither the fixture's known peer-error secret sentinel nor the native sampling prompt text was present. The fixture source itself was excluded because it intentionally contains the hostile test values it emits. No active C4OS or MCP worker remained.

## Review artifacts

| Artifact | Review purpose | Dimensions | SHA-256 |
| --- | --- | ---: | --- |
| `task-00012-chat-provenance.png` | safe run-bound Chat MCP provenance | 1100×761 | `e5ce9d6c256ecbc469c69cbd074739d8109d48b4f833f78d2c6ef9575ba67119` |
| `task-00012-mcp-empty.png` | empty MCP Settings state | 1100×761 | `23145bf52f073900501bf0cff7f6c8df36703d4190017c294f5152871ac54ac8` |
| `task-00012-mcp-trust-pending.png` | exact pending trust review | 1100×761 | `a0adb086f626a797ae332c68b20fe8143eabb79b6d9bab3ab7bdb23699c2c8f4` |
| `task-00012-mcp-failure-recovery.png` | fail-closed containment and recovery state | 1100×761 | `1597903e5285ccac7b0ddc867e77687d50a61e07390849b765a440a4dbe3c2ec` |
| `task-00012-mcp-ready-details.png` | negotiated identity, capabilities, tools, resources, and connection authority | 1100×761 | `1a3d2cdbdedaaa487faf4068980ef8f2794dbea7c968de28851e542a9b66e630` |
| `task-00012-mcp-ready.png` | enabled Ready server and one active process | 1100×761 | `5604ae430fafe26f8cb4d6bfbc6c7e1b2b9037d761e353762c3e6180ac6b8eeb` |
| `task-00012-mcp-narrow.png` | narrow contained Settings layout | 623×761 | `5b9e0ae3899dbff672de15f688cce74850e877d4e0635d82d8615d7153652a00` |
| `task-00012-mcp-http-reference.png` | authenticated HTTP reference without a bearer value | 623×761 | `82d47056d7ca2f320f26ee7fa7bccdb6cc4d5fc0ec70ba1d8cf6ff8a2ca06bf4` |
| `task-00012-mcp-revoked.png` | visible revoked authority | 623×761 | `3d5ebab6c9ec2d1ad2fed5d3ffc022856b8a81da63f26b42eada8246b151d981` |
| `task-00012-mcp-restart.png` | restored durable state after controlled relaunch | 1100×761 | `7a3a2d2a5b86c000338e808a1d02d3ce6176cb8bc389746867dbef183b0feb38` |

## Scope and limitations

- The native UI matrix uses a deterministic application-scoped STDIO fixture and a schema-valid HTTPS definition. The authenticated Streamable HTTP connection itself is proven by the isolated Rust loopback tests rather than an external service.
- The Chat provenance image uses a production-validated seeded historical turn. It proves safe durable projection and native rendering, not a live model-generated tool call.
- The acceptance Workspace intentionally has no configured model route. Native sampling is therefore correctly unavailable; production-composed tests, not a native screenshot, prove sampling approval and cancellation semantics.
- One immediate relaunch after Quit produced a blank window while old WebKit XPC helpers were still tearing down. A process sample showed the Rust/database/runtime threads parked and valid SQLite generation 22. That attempt is diagnostic only. After native Quit completed and the exact app/WebKit helper processes exited, the controlled relaunch passed and is the recorded restart evidence.
- The test fixture first stored beneath C4OS Home was deliberately inaccessible to an application-scoped MCP worker. That failure is retained as containment evidence; the passing fixture lives beside the pinned Node asset in gitignored `target/c4os-runtime-assets`.
- No signing, notarization, distribution, push, pull request, or deferred-scope expansion occurred.
- Task 00013 still owns complete onboarding and Settings integration. Task 00014 owns integrated updates, diagnostics, and degraded-state composition; Task 00015A owns the final security classification.

## Context promotion review

No Context File change is needed. Task 00012 implements the already accepted runtime/session, security, credential, persistence, provenance, and Settings authority boundaries. The exact SDK/version decision and residual compatibility risk remain implementation-specific in research ledger RBL-018.

## Agent Acceptance

PASS 2026-07-23. Independent frozen-tree review reported P0=0, P1=0, P2=1, and P3=0 after tracing production composition, protocol/transport lifecycle, exact trust and credential authority, cancellation/revocation/restart, persistence, Pi sampling settlement, Settings/Chat UI, native evidence, and scope isolation.

The retained P2 is assigned to Task 00014 recovery hardening and Task 00015A classification. If the terminal invocation CAS/database write fails after the peer effect and authority settlement, replay remains denied because execution preflight requires `Ready`, but the in-memory server can remain `Executing` with one active request and a parked connection until disable or restart. The later integrated recovery epoch must close that worker and publish an explicit failed/recoverable projection on this persistence fault.
