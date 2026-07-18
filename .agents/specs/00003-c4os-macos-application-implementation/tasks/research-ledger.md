# Bounded Online Research And Blocker Ledger

Scope: implementation dependency/version decisions and concrete blockers only. Access date for current entries: 2026-07-18.

## RBL-001 — Initial renderer and Tauri lock

- **Decision investigated:** Current compatible patch versions before the greenfield production scaffold is locked.
- **Queries and primary sources:** Tauri releases; npm registry pages for React, Vite, React Router, Redux Toolkit, and React Aria Components.
- **Exact versions observed:** Tauri core `2.11.5`; Tauri CLI `2.11.4`; React `19.2.7`; Vite `8.1.5`; React Router `8.2.0`; Redux Toolkit `2.12.0`; React Aria Components `1.19.0`.
- **Selected conclusion:** Preserve the Frozen major/minor architecture and begin Task 00001 with these current stable patch baselines. Lock the resolved compatible Tauri package set and auxiliary test/tool versions in committed lockfiles after build/test verification.
- **Rejected alternatives:** React canary/experimental builds, Vite 9 scaffolding, server-rendered frameworks, visual component kits, and unverified Tauri version mixing.
- **Implementation impact:** Task 00001 owns the lockfiles, engine constraints, typed boundary, and smoke matrix.
- **Residual risk:** Tauri crates and CLI publish independently; the resolved set must be verified by `cargo check`, native launch, and lockfile inspection before the task passes.
- **Sources:** https://github.com/tauri-apps/tauri/releases ; https://www.npmjs.com/package/react ; https://www.npmjs.com/package/vite ; https://www.npmjs.com/package/react-router ; https://www.npmjs.com/package/%40reduxjs/toolkit ; https://www.npmjs.com/package/react-aria-components

### Locked auxiliary row

- **Runtime dependencies:** `@tauri-apps/api 2.11.1`, React DOM `19.2.7`, React Redux `9.3.0`.
- **Build and QA dependencies:** TypeScript `6.0.2`, `@vitejs/plugin-react 6.0.3`, Vitest `4.1.10`, Testing Library React `16.3.2`, Playwright Test `1.61.1`, ESLint `10.7.0`, `typescript-eslint 8.64.0`, Prettier `3.9.5`, jsdom `29.1.1`.
- **Rust protocol dependencies:** `ts-rs 12.0.1`, Serde `1.0.228`, serde_json `1.0.150`, thiserror `2.0.18`, uuid `1.24.0`, tracing `0.1.44`, tauri-build `2.6.3`.
- **Compatibility decision:** The registry's TypeScript `7.0.2` was rejected for this lock because `typescript-eslint 8.64.0` declares TypeScript `<6.1.0`; `6.0.2` is the newest compatible stable line. The selected package graph requires Node `>=22.19.0` because that is the higher renderer/Pi floor.
- **Verification:** Exact direct trees, clean installs, compilation, native bundling/launch, type generation, npm audit, and RustSec audit all completed before Task 00001 passed.

## RBL-002 — Rust persistence, archive, and WebKit baselines

- **Decision investigated:** Whether Frozen Rust baselines remain current and mutually compatible.
- **Exact versions observed:** `rusqlite 0.40.1` with bundled SQLite `3.53.2`; `rusqlite_migration 2.6.0`; `zip 8.6.0`; `objc2-web-kit 0.3.2`.
- **Selected conclusion:** Keep the exact Frozen selections. Enable only required `zip` features, including Deflate, rather than its broad default compression/crypto set.
- **Rejected alternatives:** `zip 9.0.0-pre*`, system SQLite, an ORM/async ownership layer, Wry for arbitrary websites, and private WebKit APIs.
- **Implementation impact:** Tasks 00001, 00002, and 00010 lock and verify these versions.
- **Residual risk:** Target API availability and macOS binding interactions remain production integration work despite the passing Proof.
- **Sources:** https://docs.rs/crate/rusqlite/latest ; https://docs.rs/crate/rusqlite_migration/latest ; https://docs.rs/crate/zip/latest ; https://docs.rs/objc2-web-kit/latest/objc2_web_kit/

### Task 00002 auxiliary lock

- **Exact versions observed from crates.io metadata:** TOML `1.1.3+spec-1.1.0`, SHA-2 `0.11.0`, fs4 `1.1.0`, notify `8.2.0`, tempfile `3.27.0` for tests.
- **Selected conclusion:** Lock TOML's stable spec-1.1 parser, the current RustCrypto SHA-2 line, maintained pure-Rust fs4 synchronous advisory locking, stable notify 8.2 rather than the 9.0 release candidate, and test-only tempfile. Lock `zip 8.6.0` with only `deflate-flate2-zlib-rs`; do not enable its default crypto and alternate-compression graph.
- **Rejected alternatives:** Deprecated fs2 `0.4.3`, notify `9.0.0-rc.4`, SHA-2 `0.10.9` when the compatible stable `0.11.0` exists, broad zip defaults, and handwritten TOML or archive parsing.
- **Verification gate:** Cargo resolution, target tree, RustSec audit, hostile archive/configuration tests, and production integration remain required before Task 00002 passes.
- **Sources:** https://crates.io/crates/toml/1.1.3+spec-1.1.0 ; https://crates.io/crates/sha2/0.11.0 ; https://crates.io/crates/fs4/1.1.0 ; https://crates.io/crates/notify/8.2.0 ; https://crates.io/crates/tempfile/3.27.0

## RBL-003 — OpenCode adapter baseline

- **Decision investigated:** Current native runtime and SDK pair for authenticated loopback integration.
- **Exact versions observed:** OpenCode release and `@opencode-ai/sdk` both `1.18.3` on 2026-07-18.
- **Selected conclusion:** Pin both to `1.18.3` for the first adapter compatibility row; start `opencode serve` on loopback with C4OS-owned Basic authentication, isolated state, health/version checks, and the official typed SDK/OpenAPI surface.
- **Rejected alternatives:** The archived 2025 Go project with the same name, unauthenticated server use, renderer-direct access, and floating `latest` at runtime.
- **Implementation impact:** Task 00004 records the compatibility row and conformance evidence.
- **Residual risk:** OpenCode releases rapidly; C4OS must retain a compatibility matrix and last-known-good runtime rather than auto-adopting updates.
- **Sources:** https://github.com/anomalyco/opencode/releases ; https://www.npmjs.com/package/%40opencode-ai/sdk ; https://opencode.ai/docs/server/ ; https://opencode.ai/docs/sdk/

## RBL-004 — Pi SDK ownership transfer

- **Decision investigated:** The selected Pi SDK package changed maintainers/scope after Freeze.
- **Exact versions observed:** Deprecated `@mariozechner/pi-coding-agent 0.73.1`; maintained `@earendil-works/pi-coding-agent 0.80.10`; maintained `@earendil-works/pi-agent-core 0.80.7`.
- **Selected conclusion:** Use the official maintained successor `@earendil-works/pi-coding-agent 0.80.10` through the documented programmatic SDK inside the C4OS-owned Node sidecar. This is a bounded dependency substitution, not a product or authority change.
- **Rejected alternatives:** Pinning the deprecated Mario-scope package, granting Pi's own extensions ambient authority, using Pi persistence as C4OS authority, or replacing the selected SDK sidecar with its CLI/RPC without need.
- **Implementation impact:** Task 00004 must define a narrow wrapper, disable/replace native tools with C4OS action requests, and conformance-test event barriers, cancellation, capabilities, and redaction.
- **Residual risk:** The successor publishes frequently and its core/coding-agent patch numbers are not identical; lock the full dependency graph and test the exact pair.
- **Sources:** https://www.npmjs.com/package/%40mariozechner/pi-coding-agent ; https://www.npmjs.com/package/%40earendil-works/pi-coding-agent ; https://www.npmjs.com/package/%40earendil-works/pi-agent-core ; https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/sdk.md

## RBL-005 — Local toolchain baseline

- **Exact environment:** Node `26.2.0`, npm `11.13.0`, Rust/Cargo `1.96.0`, macOS `26.5.1 (25F80)` arm64, Xcode `26.6 (17F113)`.
- **Selected conclusion:** Record Node `>=22.19` as the supported development floor required by the selected Pi SDK and renderer tooling while verifying the current Node 26 host; use Rust 2024 edition with a pinned toolchain file after the first successful scaffold build.
- **Rejected alternatives:** Depending on the user's shell-specific version manager or treating the current host version as the only supported source contract.
- **Implementation impact:** Task 00001 adds engine/toolchain declarations and repeatable commands.
- **Residual risk:** Native dependencies may expose Node 26 or Xcode 26 regressions; research exact upstream issues before workarounds or downgrades.

## RBL-006 — Existing evidence and production baseline

- **Decision investigated:** Whether any current source or Proof can count as production implementation or production Agent Acceptance.
- **Observed baseline:** No root application manifest, production source tree, production test tree, or production lockfile exists. The accepted r013 wireframe is a static in-memory fixture; all 50 Feature Coverage IDs are open.
- **Reusable evidence boundary:** Existing Proofs may supply feasibility constraints and negative test scenarios only. Runtime adapter conformance passes `11/11`; the broad root Proof command currently reports `77/85` passing, with eight environment-dependent Docker, loopback-listener, and nested-sandbox failures.
- **Selected conclusion:** Build greenfield production modules behind shared coordinator-owned schemas. Do not incrementally convert the r013 monolith or import Proof success as task evidence. Split future tests into deterministic, native macOS, Docker/SSH, provider/network, and GUI tiers.
- **Rejected alternatives:** Claiming source-only readiness, using r013 timers/localStorage/DOM snapshots as production logic, and retaining stale Proof assertions that share Chats across Workspaces, delete removed Chats, terminate PTYs on Chat inactivation, keep a right panel, or use raw Wry for arbitrary pages.
- **Implementation impact:** Every task starts `open` with Agent Acceptance `failed`; each closes only from production evidence. Tasks 00001 and 00015 own tiered commands and a deterministic QA adapter.
- **Residual risk:** Environment-specific suites require explicit host capabilities and must remain visible rather than being silently excluded from completion evidence.

## RBL-007 — Locked-graph advisory disposition

- **Decision investigated:** Whether the first exact npm and Cargo graphs contain known vulnerabilities or unacceptable warnings.
- **Observed result:** Full `npm audit` reports zero vulnerabilities. After the Task 00002 persistence/archive dependencies were locked, `cargo audit` scans 460 locked Rust dependencies and reports zero vulnerabilities plus 17 warnings: 16 unmaintained advisories and one unsound glib advisory.
- **Target analysis:** `cargo tree --target aarch64-apple-darwin -i glib` returns no path; GTK/glib warnings are target-specific and absent from the accepted arm64 macOS build. The unmaintained `unic-*` chain is reachable through Tauri `2.11.5` to tauri-utils `2.9.3` to urlpattern `0.3.0` and has no RustSec vulnerability finding.
- **Selected conclusion:** Accept the exact local macOS foundation graph with the warnings recorded, retain current Tauri patches, and re-audit on dependency updates. Do not force unsupported transitive replacements.
- **Rejected alternatives:** Ignoring advisory output, claiming warning-free Rust dependencies, patching registry crates locally without an upstream compatibility reason, or expanding the milestone to Linux GTK support.
- **Implementation impact:** Task 00001 passes; RBL-007 remains a visible maintenance input and does not close later security acceptance.
- **Residual risk:** Upstream maintenance status can change. Any new vulnerability, reachable unsoundness on the accepted target, or Tauri patch row must reopen the owning dependency task.
