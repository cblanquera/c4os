# Bounded Online Research And Blocker Ledger

Scope: implementation dependency/version decisions and concrete blockers only. Access dates for current entries run through 2026-07-20.

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
- **Exact versions observed:** Deprecated `@mariozechner/pi-coding-agent 0.73.1`; maintained `@earendil-works/pi-coding-agent 0.80.10`, `@earendil-works/pi-agent-core 0.80.10`, and `@earendil-works/pi-ai 0.80.10` in the implemented lock graph.
- **Selected conclusion:** Use the official maintained successor `@earendil-works/pi-coding-agent 0.80.10` through the documented programmatic SDK inside the C4OS-owned Node sidecar. This is a bounded dependency substitution, not a product or authority change.
- **Rejected alternatives:** Pinning the deprecated Mario-scope package, granting Pi's own extensions ambient authority, using Pi persistence as C4OS authority, or replacing the selected SDK sidecar with its CLI/RPC without need.
- **Implementation impact:** Task 00004 must define a narrow wrapper, disable/replace native tools with C4OS action requests, and conformance-test event barriers, cancellation, capabilities, and redaction.
- **Residual risk:** The successor publishes frequently; lock the full dependency graph and re-run the exact coding-agent/core/AI compatibility suite before changing any package version.
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

## RBL-008 — Credential encryption and macOS installation-key custody

- **Access date:** 2026-07-19.
- **Decision investigated:** The smallest maintained Rust dependency set that implements the Frozen credential contract: macOS Keychain custody for the installation key, authenticated encryption for durable credential values, memory-hard password verification, random nonces/salts/tokens, and zeroization of owned secret buffers.
- **Exact versions observed:** `security-framework 3.7.0`, `chacha20poly1305 0.11.0`, `argon2 0.5.3`, `getrandom 0.3.4`, and `zeroize 1.9.0`.
- **Selected conclusion:** Use the platform Security framework only for the random installation key; encrypt vault records with XChaCha20-Poly1305 and record-bound associated data; use Argon2id PHC strings for password credentials; use the operating-system RNG for keys, nonces, salts, and opaque delivery tokens; zeroize owned plaintext and key buffers. Persist only opaque credential references and authenticated ciphertext metadata outside Keychain. Keep session-only credentials exclusively in memory and deliver decrypted values only through short-lived, single-consumer leases.
- **Rejected alternatives:** Persisting plaintext, using a fast digest as a password hash, placing the entire credential corpus in Keychain, browser storage, unauthenticated encryption, a new immature cross-platform security wrapper, and stale keyring abstractions that would dilute the exact macOS custody contract.
- **Implementation impact:** Task 00003 owns the credential vault, Keychain adapter, reauthentication/import state, lease invalidation, redaction/scanning, hostile corruption tests, and restart evidence. Task 00013 may expose only opaque references and safe metadata through Settings.
- **Residual risk:** CI and non-macOS unit tests cannot prove a live login Keychain interaction. The macOS acceptance tier must exercise the platform adapter in the unlocked user session without printing or persisting secret material; any Keychain access-control prompt or OS behavior change remains visible native evidence rather than being substituted with the deterministic test backend.
- **Sources:** https://github.com/kornelski/rust-security-framework ; https://github.com/RustCrypto/AEADs/tree/master/chacha20poly1305 ; https://github.com/RustCrypto/password-hashes/tree/master/argon2 ; https://docs.rs/getrandom/0.3.4/getrandom/ ; https://docs.rs/zeroize/1.9.0/zeroize/

## RBL-009 — OpenCode native authority boundary

- **Access date:** 2026-07-19.
- **Decision investigated:** Whether OpenCode's native permission replies can be treated as C4OS authorization and whether a C4OS approval may be returned as native `once`.
- **Observed upstream contract:** OpenCode `1.18.3` exposes an OpenAPI 3.1 server and generated SDK; its server uses `OPENCODE_SERVER_PASSWORD` for HTTP Basic authentication. Its permission configuration defaults most tools to `allow`, accepts global and tool-specific `allow`/`ask`/`deny`, and its native approval outcomes include `once`, session-scoped `always`, and `reject`.
- **Selected conclusion:** OpenCode permission state is not C4OS policy authority. Launch the exact native process on authenticated loopback with default `deny`, allowing only the exact C4OS-owned `c4os_propose_action` and `c4os_read_resource` broker tools. Per request, expose only those broker IDs. Route proposal identity through the sealed C4OS Action Gateway, execute effects only in a C4OS worker, and return native `reject` even after a C4OS effect completes so OpenCode itself never receives effect authority. This is a C4OS defense-in-depth inference from the upstream permission and server contracts, not an upstream claim that OpenCode supplies an OS sandbox.
- **Rejected alternatives:** Wildcard `ask`, native `once` after C4OS approval, native `always`, built-in write/bash execution, renderer-originated permission decisions, relying on permissive defaults, and describing OpenCode permissions as the C4OS security boundary.
- **Implementation impact:** Task 00004 owns digest-pinned authority configuration, isolated state, authenticated transport, broker-only per-request tools, exact intent binding, fail-closed native replies, and negative tests proving direct native-effect paths are unavailable.
- **Residual risk:** The two broker tool definitions must remain C4OS-owned and digest verified when materialized for the native runtime. Any upstream permission, custom-tool, or server API change must create a new compatibility row rather than silently widening authority.
- **Sources:** https://opencode.ai/docs/permissions/ ; https://opencode.ai/docs/server/ ; https://opencode.ai/docs/sdk/ ; local exact `@opencode-ai/sdk 1.18.3` generated OpenAPI types.

## RBL-010 — OpenCode provider credential delivery

- **Access date:** 2026-07-19.
- **Decision investigated:** How an exact OpenCode `1.18.3` worker can authenticate a selected C4OS provider without moving the raw provider credential into ordinary runtime state, command arguments, inherited environment, logs, renderer data, or native-owned authority.
- **Queries and primary sources:** OpenCode provider/auth documentation and source; exact local `@opencode-ai/sdk 1.18.3` generated `AuthSetData` types; exact local `@opencode-ai/plugin 1.18.3` `chat.headers` hook types; upstream credential-storage issue and current plugin examples.
- **Observed upstream contract:** The generated SDK's standard `PUT /auth/{id}` path accepts the raw provider auth value. OpenCode documents that provider credentials are stored in `auth.json`, and its current CLI/source paths also recognize environment/config credentials. The exact plugin contract can inject request headers, but it supplies no C4OS vault or operation-lease boundary by itself.
- **Selected conclusion:** Do not call the native auth persistence API and do not place provider keys in OpenCode configuration or environment. Extend the already authenticated, descriptor-bound C4OS plugin bridge with a separate bounded credential descriptor. Rust leases the exact provider credential for the active session operation, delivers one correlation/provider/session-bound frame, and the C4OS plugin consumes it once while constructing that request's provider header. The secret channel is distinct from the Action Gateway broker so ordinary broker frames remain secret-rejecting.
- **Rejected alternatives:** Native `auth.json`, `PUT /auth/{id}`, provider keys in `opencode.json`, `.env`, or inherited environment, renderer delivery, command-line delivery, a long-lived plaintext sidecar cache, weakening the broker's secret scanner, and declaring OpenCode provider authentication unsupported while presenting the runtime as ready.
- **Implementation impact:** Task 00004 must wire the private descriptor through the exact OpenCode launcher/plugin graph, derive the provider credential reference inside Rust-owned route state, deliver it before the exact dispatch, consume it once in `chat.headers`, and prove with live native tests and state/log/argv/environment scans that the raw value is absent outside the channel and transient worker memory.
- **Residual risk:** OpenCode's plugin/auth internals can change rapidly. Any upgrade must re-run the exact live credentialed dispatch and secret-absence matrix; the standard upstream auth API remains intentionally unused.
- **Sources:** https://opencode.ai/docs/providers/ ; https://opencode.ai/docs/cli/ ; https://github.com/anomalyco/opencode/issues/5423 ; https://github.com/anomalyco/opencode/blob/dev/packages/opencode/src/plugin/codex.ts ; local exact `@opencode-ai/sdk 1.18.3` and `@opencode-ai/plugin 1.18.3` type surfaces.

## RBL-011 — OpenCode server-secret descriptor build

- **Access date:** 2026-07-19.
- **Decision investigated:** Whether the stock OpenCode `1.18.3` executable can satisfy the Frozen requirement that the authenticated loopback server secret never enter arguments, the inherited environment, logs, configuration, or persistent runtime state.
- **Observed upstream contract:** Exact tag `v1.18.3` at commit `127bdb30784d508cc556c71a0f32b508a3061517` reads the server password from `OPENCODE_SERVER_PASSWORD`. Stock bytes therefore cannot meet the stricter C4OS secret-absence contract even though the generated SDK/OpenAPI and native compatibility version remain usable.
- **Selected conclusion:** Build a distinct `1.18.3+c4os-auth-fd.1` artifact from that exact tag. The bounded patch reads the server secret once from the inherited `C4OS_OPENCODE_SERVER_SECRET_FD`, rejects simultaneous environment password delivery, clears descriptor metadata, closes the descriptor, and preserves stock behavior only outside the C4OS launch. Pin Node `26.3.0` (`sha256:cdf556966c52b321abb07cd565edb5fb2ca61c467f84a434bb28e0d33a3c9580`), Bun `1.3.14` (`sha256:e0c90ec15d33363e6b70713d56bc3b2c7585c17f40a0fe0f8fd9305901d4e233`), patch `sha256:82af7d9b917e6fe31fa524b37840cdd95f86944e9ea0c566eb0e061020bc99c0`, the compressed models.dev snapshot `sha256:bc9565e9e805f3a5496e674dccbce5ed0338dabf0e2d93c6ddbf41d00533216d`, and the resulting arm64 binary `sha256:e117af53becb62e91a8b8ba4e4965aeab3ec77082e31c118420f2b779753bc77` (137,518,946 bytes). Two independent clean dependency reconstructions produced those exact bytes. Runtime provenance records the C4OS flavor separately and never calls it the official npm binary.
- **Rejected alternatives:** Shipping the raw password in `OPENCODE_SERVER_PASSWORD`, placing it in argv/configuration, weakening the Frozen secret contract, silently substituting unrecorded bytes, or claiming the downstream artifact is the official npm release.
- **Implementation impact:** `sidecars/opencode-native/c4os-build.json` is the build-input ledger; `build-c4os.mjs --force` is restricted to the build-owned checkout, rejects unexpected untracked source, clears ignored dependency/build state, restores the frozen lockfile under an allowlisted environment with no inherited Sentry/Vite credentials, and compares the rebuilt digest; the production asset factory re-verifies the manifest, patch, model snapshot, complete native tree, executable digest, version, and build flavor before spawn. Exact native tests must scan both server and provider secrets across process and state surfaces.
- **Residual risk:** OpenCode may change its auth initialization or build inputs in a future release. Every upgrade requires a new compatibility row, patch review, forced reproducibility build, exact-native golden path, and renewed secret scan. The server secret necessarily exists in transient OpenCode worker memory while Basic authentication is active; no other surface is permitted.
- **Sources:** https://github.com/anomalyco/opencode/blob/v1.18.3/packages/opencode/src/server/auth.ts ; https://github.com/anomalyco/opencode/blob/v1.18.3/packages/opencode/script/build.ts ; https://opencode.ai/docs/server/ ; pinned local downstream patch and build manifest.

## RBL-012 — Published C4OS OpenCode proof reuse boundary

- **Access date:** 2026-07-20.
- **Decision investigated:** Whether the OpenCode source already published in `cblanquera/c4os` materially helps Task 00004 and can replace any production evidence.
- **Observed repository state:** Published `main` contains `proofs/opencode-runtime/opencode_runtime_poc.mjs` and its evidence note. The proof used OpenCode `1.17.8` and demonstrated isolated SDK start, session create/resume, typed event observation, abort, and shutdown. Its evidence explicitly did not prove a credentialed model prompt, live approval path, or application-owned persistent state. The local `build/mvp-3` Task 00004 source is not published on the repository's current branch list.
- **Selected conclusion:** Reuse the published proof only as architectural orientation for the SDK/session/event/abort sequence. The production `1.18.3` composition, private credential paths, C4OS Action Gateway, exact attachment delivery, persistence, capability atomicity, and cleanup must remain locally verified Task 00004 evidence.
- **Rejected alternatives:** Treating the older proof as production acceptance, downgrading to its `1.17.8` pin, or publishing the local implementation without an explicit user request.
- **Implementation impact:** No source was copied and no push occurred. The live repository check confirmed that Task 00004's local production path materially supersedes the published proof.
- **Residual risk:** Published proof wording can become misleading if read without its evidence boundary; future documentation should link the production compatibility row when the implementation is intentionally published.
- **Sources:** https://github.com/cblanquera/c4os ; https://github.com/cblanquera/c4os/blob/main/proofs/opencode-runtime/opencode_runtime_poc.mjs ; https://github.com/cblanquera/c4os/blob/main/proofs/opencode-runtime/opencode-runtime-evidence-2026-06-20.md

## RBL-013 — Private native TLS verification capability

- **Access date:** 2026-07-20.
- **Decision investigated:** How to prove the exact authenticated OpenCode/Pi provider paths without persistently altering the user's login or System keychain.
- **Observed blocker:** macOS rejected the serialized System.keychain trust attempts before a usable administrator interaction completed. Each attempt was followed by exact certificate, keychain search-list, lock, and temporary-file reconciliation; no trust item remained.
- **Selected conclusion:** Generate an ephemeral private CA and leaf inside a mode-`0700` temporary root, pass the CA only to the ignored native test through a test-only capability, require machine trust to reject the leaf before and after the run, and clean the isolated Cargo/native process group plus temporary material under a wrapper trap. Production provider TLS continues to use normal platform trust.
- **Rejected alternatives:** Leaving a fingerprint-bound test root in System.keychain, weakening TLS verification, disabling certificate validation, or claiming browser/provider success from an untrusted local endpoint.
- **Implementation impact:** `tools/run-task-00004-native-golden.zsh` owns the private capability, process isolation, trust pre/postcondition, evidence path, and cleanup invariant. The final post-bundle run passed.
- **Residual risk:** This proves the app-owned TLS/authentication path under a private test capability, not enterprise proxy/root deployment behavior. Platform trust-store integration remains a deployment concern rather than a Task 00004 runtime authority.
- **Sources:** project-owned native wrapper, production integration test, and final evidence JSON.

## RBL-014 — macOS menu, theme, and native picker composition

- **Access date:** 2026-07-20.
- **Decision investigated:** How Task 00005 should compose a system-following initial theme, the native application Settings menu and `Cmd+,`, and Rust-owned file/folder/workspace pickers on the locked Tauri line.
- **Observed upstream contract:** Tauri `2.11.5` exposes the current window theme, app-wide menus and menu events, standard menu builders, and accelerator parsing where `CmdOrCtrl` maps to Command on macOS. The official `tauri-plugin-dialog 2.7.2` supports Rust-side non-blocking pickers and blocking pickers from asynchronous command contexts; its blocking API must not run on the main thread. Apple documents that a `nil` app appearance follows the current system appearance and that effective appearances and adaptive colors are the native Dark Mode model.
- **Selected conclusion:** Keep the standard decorated window and system-owned appearance, capture Tauri's current native window theme before reveal, retain an independent webview `prefers-color-scheme` listener for live changes, build an app-wide native menu with a typed Settings event and `CmdOrCtrl+,`, and pin `tauri-plugin-dialog 2.7.2` for Rust-owned opaque-grant pickers. Do not grant the renderer the dialog plugin's generic command surface or raw filesystem authority.
- **Rejected alternatives:** A custom titlebar, a persisted C4OS theme override, dependence on `WindowEvent::ThemeChanged`, renderer-owned generic dialog calls, blocking a picker on the main thread, and exposing selected paths as ambient renderer filesystem authority.
- **Implementation impact:** Task 00005 owns the PlatformService snapshot, pre-reveal/reveal handshake, typed Settings event, native menu, picker-grant lifecycle, semantic theme listener, and exact native/rendered tests. Task 00006 and Task 00013 retain the full shell and Settings content.
- **Residual risk:** The rebuilt macOS app passed the Task 00005 initial/live theme, shortcut, cancellation, grant-isolation, focus, responsive, and picker matrix; Task 00015B must repeat the integrated native/accessibility audit. Any Tauri or dialog-plugin upgrade requires repeating the matrix.
- **Sources:** https://docs.rs/tauri/2.11.5/tauri/struct.App.html ; https://docs.rs/tauri/2.11.5/tauri/menu/ ; https://docs.rs/tauri/2.11.5/tauri/window/struct.Window.html ; https://docs.rs/tauri-plugin-dialog/2.7.2/tauri_plugin_dialog/struct.FileDialogBuilder.html ; https://developer.apple.com/documentation/appkit/nsapplication/appearance ; https://developer.apple.com/documentation/appkit/nsappearance
