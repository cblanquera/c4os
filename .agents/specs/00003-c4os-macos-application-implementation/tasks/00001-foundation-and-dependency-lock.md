# Task 00001 — Foundation And Dependency Lock

Status: verified

Coverage: foundation for all IDs; direct support for UX-001, UX-004, UX-010, QA-001.

## Summary

Create the greenfield production Cargo/Tauri/React workspace and lock the researched toolchain, dependency, typed-boundary, test-runner, and deterministic-QA foundations without importing wireframe architecture.

## Implementation Steps

1. Create root npm, Rust workspace, Tauri, renderer, sidecar, test, fixture, and evidence directories with single-purpose ownership.
2. Lock the RBL-001 through RBL-005 versions, engine/toolchain constraints, Tauri permissions/capabilities, minimal `zip` features, and committed lockfiles.
3. Define versioned Rust-owned IDs, errors, envelopes, snapshots, events, redaction markers, correlation/generation rules, and `ts-rs` generation.
4. Add a small handwritten renderer adapter; reject unknown protocol versions and stale generations.
5. Configure format, lint, unit, component, Playwright, Rust, native, and deterministic-QA commands plus a production build/launch command.
6. Prove the shell builds, launches, and directly resolves a minimal QA route without granting renderer authority.

## Verification Process

- Install from lockfiles; inspect dependency trees and advisories.
- Run TypeScript, Rust, lint, unit, component, build, and protocol-generation checks.
- Launch the macOS development build and inspect the minimal route, menu shell, console, and generated types.
- Verify no root proof or wireframe source is bundled as production code.

Acceptance criteria: none — implementation acceptance is delegated to the coordinator’s Agent Acceptance process for Spec 00003.

## Agent Acceptance

Result: passed — the locked foundation, fail-closed protocol, deterministic QA boundary, production build, and launched native shell passed the coordinator's 2026-07-18 review.

Required evidence: lockfiles and exact versions; successful clean install/build/test; source/diff inspection; generated-type determinism; stale/unknown boundary denial; launched macOS screenshot and accessibility snapshot; no console errors; evidence commands and paths; unresolved limitations.

## Implementation Notes

Completed 2026-07-18. Added the root npm/Rust workspace, exact npm and Cargo locks, Rust 1.96 and Node 22.19 floors, Tauri 2 application/capability/CSP configuration, React/Vite/Router/Redux/React Aria shell, generated `ts-rs` bindings, one-command renderer adapter, deterministic build-gated QA route, and tiered test/build commands. The renderer now reports a connected Rust authority only after the allowlisted `foundation_snapshot` command returns a correlated, current protocol envelope. No Proof or r013 source was imported into production.

## Verification Notes

- `npm run check`: passed formatting, ESLint, TypeScript, 18 Vitest tests, 12 Rust tests, and deterministic binding generation.
- `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all -- --check`: passed.
- `npm run test:e2e`: Chromium QA-route/reset and production-renderer fail-closed tests passed with explicit console and horizontal-overflow checks after a discovered favicon 404 was fixed and the gate was strengthened through `networkidle`.
- `npm audit`: zero vulnerabilities across production and development dependencies.
- `cargo audit`: zero vulnerabilities; 17 advisory warnings remain. The GTK/glib warnings are not in the arm64 macOS target tree; current-target `unic-*` warnings are unmaintained transitive Tauri/urlpattern crates, not reported vulnerabilities.
- `npx tauri build --debug --bundles app`: passed; local binary SHA-256 `c1c551276b2fab463183b067909980fe2dfef44d71bbccf8758271f54d598962`.
- Computer Use inspected the rebuilt `.app`: native 1100×761 screenshot, coherent accessibility tree, standard window controls/menu bar, disabled unfinished action, no visible overflow, and `Connected · Rust core` returned by the live command boundary. Evidence SHA-256 `faab40d445ee7af1aa0d68fae759e5676607f0bf723d12dc95b793131fdd543c`: `tests/results/playwright/task-00001-native.png`.
- Playwright CLI inspected the gated QA route, reset behavior, accessibility snapshot, corrected console state, and screenshot. Evidence: `tests/results/playwright/task-00001-qa-route.png`.
- Bundle/source scan found no Proof or wireframe imports. Renderer persistence remains absent; the only `localStorage` occurrence is an explanatory QA source comment.

## Agent Acceptance Notes

Passed after coordinator diff inspection and reruns. Task 00005 retains the initial native pre-reveal theme/visibility gate: Computer Use can observe one blank launch frame before WebView content attaches. Task 00006 retains the complete production route/component shell. No Feature Coverage ID closes from this foundation alone.
