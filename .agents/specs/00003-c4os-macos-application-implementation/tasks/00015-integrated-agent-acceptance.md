# Task 00015 — Deterministic QA And Integrated Agent Acceptance

Status: verified

Coverage: UX-002, UX-009, QA-001, QA-002; integrated closure support for all 50 Feature Coverage IDs.

## Summary

Build the deterministic production QA adapter and perform the complete Agent Acceptance matrix on a launched local macOS development build, including every r013 destination and cross-cutting native, security, persistence, accessibility, failure, recovery, console, and overflow gate.

## Implementation Steps

1. Implement a typed, build-gated fixture schema/adapter with deterministic IDs, clock, data, reset, replay, isolation, and direct-route launcher that cannot masquerade as production state.
2. Implement tiered Rust, renderer, adapter, native, Playwright, accessibility, security, persistence, failure, recovery, and fixture test commands.
3. Build and launch the real application; traverse all 16 r013 destinations plus complete coverage-specific normal and negative paths.
4. Exercise Light/Dark, reduced motion, wide/minimum/overlay widths, keyboard-only operation, focus order/restoration, accessibility tree, native labels, menus, pickers, and failure/recovery states.
5. Assert no uncaught console errors, prohibited overflow, unauthorized side effects, fabricated runtime events, secret exposure, source-only claims, or deferred-feature leakage.
6. Record reproducible evidence, close coverage only when all primary and supporting tasks pass, and rerun the full regression set after fixes.

## Verification Process

- Run every documented deterministic and environment-specific suite with results separated by tier.
- Use Computer Use or an applicable visual browser/native tool for production-rendered walkthroughs and capture evidence artifacts.
- Inspect the production bundle, logs, authoritative databases, audits, diagnostics, and evidence index for truthful boundaries.

Acceptance criteria: none — implementation acceptance is delegated to the coordinator’s Agent Acceptance process for Spec 00003.

## Agent Acceptance

Result: passed — deterministic QA, complete renderer/browser regression, serialized Rust/security/protocol verification, rebuilt bundle/native tiers, production restart/degraded-state acceptance, all 50 coverage reconciliations, and three independent reviews passed with P0 = 0 and P1 = 0.

Required evidence: complete command/results index; launched-build identity; 16-route and 50-ID traceability; screenshots and accessibility records; native/keyboard/theme/responsive/failure/recovery matrix; console/overflow/security assertions; evidence paths and limitations.

## Implementation Notes

Completed 2026-07-24 after the verified Task 00014 checkpoint `56bd8d2`. The build-gated QA contract exposes a versioned scenario, fixed IDs and clock, independent adapter state, exact reset/replay, all 16 accepted direct routes, and a persistent fixture-only identity around production-composed QA destinations. Production composes only the product router, store, and native transport; QA selection requires the build gate and explicit in-page harness marker. Production bundles fail if they contain fixture markers or frontend source maps. Static wireframe QA and proof suites informed scenarios but never substituted for the production-rendered run.

Final native acceptance discovered a real restart defect after the diagnostic journal reached its 250-record cap. The next transition needed the union of the prior and replacement diagnostic IDs, but the database API incorrectly limited that replacement scope to 250. The repaired boundary permits at most 500 replacement IDs while inserted and retained rows remain capped at 250. Compare-and-swap, delete, insert, retention, pruning, and generation publication remain one immediate transaction. Focused maximal-replacement and saturation/restart regressions pass.

## Verification Notes

Complete renderer quality passed: formatting, lint, typecheck, 79 files and 454 unit tests, production and QA builds, both bundle-boundary checks, and 45/45 Playwright scenarios. The serialized full Rust workspace run passed every product library, integration, and doc-test target; its two outer-sandbox MCP STDIO failures passed 2/2 in the exact host rerun. Security passed 104 runnable tests with one explicit live-Keychain test ignored. Exact protocol, native build, all three packaged-tree checks, MCP HTTP 2/2, OpenCode native 7/7, OpenCode streaming 3/3, packaged runtime 4/4, and private-TLS golden 1/1 passed.

Computer Use traversed all 16 QA routes, native Settings/Back, keyboard focus, wide/narrow geometry, Light/Dark composition, and degraded state. The final production binary is SHA-256 `0fabf1fdfd4a9a63f2952a1aec7ab6dd89f935386ac32a3e37f7d41b5ed47a53`, contains zero frontend maps and zero QA markers, launched a saturated 250-diagnostic home healthy three times, recorded all nine startup boundaries, and kept a preserved Database failure blocked after Retry with no Continue path. Secret, log, database, bundle, and process cleanup evidence is indexed in `output/native/task-00015-acceptance.md`.

## Agent Acceptance Notes

Side quests 00015A, 00015B, and 00015C passed. Final independent review:

- Rust/native/security: P0 = 0, P1 = 0, P2 = 7, P3 = 0.
- Renderer/integration: P0 = 0, P1 = 0, P2 = 2, P3 = 0.
- Policy/configuration: P0 = 0, P1 = 0, P2 = 2, P3 = 0.

Retained P2s are recorded in the acceptance evidence and side-quest files. None grants renderer, peer, fixture, or stale native authority. Agent Acceptance passed only after every owner reported P0 = 0 and P1 = 0.
