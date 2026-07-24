# Task 00012 — MCP Lifecycle

Status: verified

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

Result: passed — independent frozen-tree review reported P0=0, P1=0, P2=1, and P3=0.

Required evidence: complete in `.agents/resources/native/task-00012-acceptance.md` and the ten Task 00012 native captures.

## Implementation Notes

Started 2026-07-22 from verified Task 00011 checkpoint `e912e22`. The disposable STDIO proof is useful scenario input but is not 2025-11-25 production conformance: it speaks `2025-03-26`, uses an ad hoc Node client, and supplies neither authenticated Streamable HTTP nor durable C4OS authority, policy, audit, recovery, or Settings evidence.

The smallest production-composed golden path is one disabled, trusted STDIO server definition owned by the Rust core. C4OS tests an exact `2025-11-25` initialize/capability handshake, explicitly enables the server for the next turn, snapshots one bounded listed tool and resource, invokes one tool only after capability preflight and Action Gateway authorization, persists a redacted untrusted result, then disables and terminates the server. Authenticated Streamable HTTP, opaque secret delivery, sampling, notifications, cancellation, crash/restart recovery, revocation, and truthful Settings states extend this same service and authority model.

Primary-source research selected and locked official Rust SDK `rmcp 2.2.0`: its 2026-07-08 release reports client conformance and fixes from a `2025-11-25` conformance audit. The final graph pins `tokio 1.53.0`, `reqwest 0.13.4`, and `process-wrap 9.1.0`; disables `rmcp` defaults; and enables only the client, child-process, reqwest Streamable HTTP client, and reqwest features.

Completed 2026-07-23. The Rust core now owns schema-1 definitions, lifecycle generations, a durable append-only journal, exact definition trust, scoped launch authority, strict STDIO and authenticated Streamable HTTP transports, capability discovery, immutable per-turn catalogs, JSON Schema input validation, bounded tool/resource results, notifications, cancellation, timeout, restart/recovery, revocation, and secret-reference-only persistence. MCP tool and sampling effects cross the existing production runtime broker and Action Gateway; sampling uses the configured model route only after informed approval and retains atomic approval/authorization/origin state when persistence fails.

The production renderer now has real MCP Settings list/empty/add/edit/test/trust/enable/disable/details/recovery/revoke states and a shared live runtime approval center. Chat exposes one safe immutable MCP provenance projection per captured turn. The Pi sidecar now settles cancellation through credential claim/provider execution without a late prompt, returns the requested route identity, releases operation credentials, and launches from a retained private double-verified source snapshot.

## Verification Notes

- `npm test --prefix sidecars/pi`: 50/50 passed.
- `npm test`: 56 files and 297/297 passed.
- `npm run test:e2e -- --workers=1`: 42/42 passed, including the production MCP Settings console/overflow/focus/reference check and updated legacy turn fixtures.
- `npm run lint`, `npm run typecheck`, `npm run format:check`, and `cargo fmt --all -- --check`: passed.
- `npm run protocol:generate`: passed; Rust export 1/1 and generated MCP/Chat bindings match the source contracts.
- `cargo test --workspace --no-fail-fast`: passed with no failures; library 221 passed and 4 expected ignored tiers, followed by every runnable integration and doc-test target.
- Authenticated Streamable HTTP ignored tier: 2/2 passed; STDIO, lifecycle, persistence, production broker, production sampling, approval rollback, credential, hostile-input, cancellation, restart, and revocation coverage also passed in the focused/full matrix.
- Supplementary `cargo clippy --workspace --all-targets -- -D warnings`: all Task 00012-introduced warnings were repaired; the command still exits 101 on exactly 17 inherited warnings in previously accepted Task 00011-or-earlier code and is not represented as a required-matrix pass.
- `npm run tauri:build`: passed. Existing packaged OpenCode/Pi native golden path 1/1 passed in 55.07 seconds.
- Native macOS acceptance against `/private/tmp/c4os-task12-acceptance.XICVo2`: real Settings create/trust/test/enable/disable/reconfigure/revoke/restart passed; one supervised worker was observed only while enabled; exact durable generation 22 and 22 journal events were inspected; secret/prompt persistence scans and final process cleanup passed.
- Production evidence: `.agents/resources/native/task-00012-acceptance.md` plus ten Task 00012 PNG captures covering Chat provenance, empty, trust, failure/recovery, ready/details, narrow, HTTP reference, revoked, and restart states.

## Agent Acceptance Notes

PASS 2026-07-23. The independent frozen-tree review traced production composition, exact trust and credential authority, both supervised transports, Pi sampling settlement, cancellation/revocation/restart, persistence, Settings/Chat projection, native evidence, and scope isolation. Final counts: P0=0, P1=0, P2=1, P3=0.

The non-blocking P2 is retained for Task 00014 recovery hardening and Task 00015A classification: if the terminal invocation CAS/database commit fails after the peer effect and authority settlement, execution cannot replay because preflight no longer sees `Ready`, but the in-memory server can remain `Executing` with one active request and a parked connection until disable or restart. This is honest fail-safe effect behavior without an authority bypass; later integrated recovery must close the worker and project an explicit failed/recoverable state on that persistence fault.
