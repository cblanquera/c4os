# Research

## State

Broad architecture and platform research is complete in Frozen Specs 00001 and 00002. The user delegated seven bounded implementation selections to research on 2026-07-18. Do not expand back into general competitor or standards research without user approval.

## Selection Topics

| ID | Topic | Gap | Required output | State |
| --- | --- | --- | --- | --- |
| R-001 | Renderer candidates for a Tauri 2 desktop application | GAP-002 | Select the framework, state/router structure, typed Rust boundary, accessibility/testing baseline, and r013 component fit. | Complete; IS-001 |
| R-002 | Rust-owned embedded persistence and migrations | GAP-003 | Select the store and migration mechanism against transactions, backup/rollback, concurrency, isolation, testing, and packaging. | Complete; IS-002 |
| R-003 | Production OpenCode and Pi version baseline | GAP-004 | Confirm versions and adapter deltas when post-Freeze adapter task planning begins. | Deferred to task planning |
| R-004 | Released Tauri/Wry browser isolation and permission boundary | GAP-005 | Source-audit exact candidates and identify the target Proof or explicit Browser feature gate. | Complete; IS-003 and Proof passed |
| R-005 | macOS distribution and updater path | GAP-006 | Revisit signing, notarization, distributable packaging, and signed-updater evidence when a distribution milestone is opened. | Deferred beyond local-development milestone |
| R-006 | Complete extension delivery boundary | GAP-007 | Select concrete package, discovery, install, activation, execution, supervision, sandbox, secret, timeout, revocation, update, rollback, and failure contracts for Plugins, Skills, and MCP Servers. | Complete; IS-004 |
| R-007 | Workspace archive lifecycle | GAP-009 | Select manifest/versioning, atomic save, locking, autosave, recovery, migration, extraction limits, cache policy, and Start-action mechanics. | Complete; IS-005 |
| R-008 | Scoped C4OS configuration | GAP-014 | Define strict secret-free schemas, precedence, validation, reload, diagnostics, activation, and UI/Rust-record reconciliation. | Complete; IS-006 |
| R-009 | C4OS Home and Workspace physical layout | GAP-015 | Define exact subpaths/formats, editable configuration, immutable packages, disposable caches, backup/reset, portability, and exclusion rules. | Complete; IS-007 |

## Research Rules

- Use primary documentation and current upstream source for exact-version claims.
- Record access date, selected version, rejected candidates, affected Gap, and remaining risk.
- Prefer the smallest choice that satisfies the accepted Context contract.
- Do not let framework or library conventions move policy, persistence, credential, or audit authority out of Rust.
- Report any proposed expansion before adding a new topic.

## Selections And Evidence

Research ran on 2026-07-18. [Implementation selections](implementation-selections.md) is the normative output; this section records source provenance, rejected candidates, and residual risk.

### R-001 — Renderer

- **Selected:** React 19.2, TypeScript, Vite 8.1, React Router 8, Redux Toolkit, React Aria Components, `ts-rs`, Vitest/Testing Library, and Playwright.
- **Evidence:** Tauri recommends Vite for SPA frameworks and acts as a static host; React's current stable documentation is 19.2; Redux Toolkit is the recommended Redux approach; React Aria supplies accessible unstyled primitives; `ts-rs` generates TypeScript declarations from Rust types.
- **Rejected:** server-rendered/meta-framework routing; renderer-local durable persistence; a visual component kit; `tauri-specta` while its selected APIs remain prerelease.
- **Risk:** exact compatible patch versions must be locked together and exercised in packaged-route and native-focus tests.
- **Sources:** [Tauri frontend configuration](https://v2.tauri.app/start/frontend/), [React versions](https://react.dev/versions), [Redux Toolkit](https://redux-toolkit.js.org/introduction/why-rtk-is-redux-today), [React Aria](https://react-spectrum.adobe.com/react-aria/getting-started.html), [`ts-rs`](https://docs.rs/ts-rs/latest/ts_rs/), [`tauri-specta` releases](https://github.com/specta-rs/tauri-specta/releases).

### R-002 — Persistence

- **Selected:** `rusqlite` 0.40.1 with bundled SQLite 3.53.2 and online backup, plus `rusqlite_migration` 2.6.0.
- **Evidence:** the crate exposes the SQLite online backup API, fixes the SQLite engine version through `bundled`, and the migration library validates and atomically applies ordered migrations.
- **Rejected:** a system SQLite dependency, ORM ownership, renderer storage, extension-provided migrations, and an async database layer without a C4OS concurrency need.
- **Risk:** crash, disk-full, interrupted-backup, WAL, and restore paths still require production tests.
- **Sources:** [`rusqlite` 0.40.1](https://docs.rs/crate/rusqlite/latest), [`rusqlite` backup](https://docs.rs/rusqlite/latest/rusqlite/backup/struct.Backup.html), [`rusqlite_migration` 2.6.0](https://docs.rs/rusqlite_migration/latest/rusqlite_migration/struct.Migrations.html), [SQLite WAL](https://www.sqlite.org/wal.html), [SQLite backup API](https://www.sqlite.org/backup.html).

### R-004 — Browser

- **Selected:** a Rust-owned native `WKWebView` controller using public WebKit bindings; do not ship arbitrary websites in Tauri `WebviewWindow` or Wry 0.55.1.
- **Evidence:** the current Wry 0.55.1 builder exposes navigation, popup, and download handlers but no permission handler. Apple exposes permission delegates plus identifier-based persistent and nonpersistent `WKWebsiteDataStore` profiles.
- **Platform correction:** public WebKit accepts a profile identifier, not a C4OS-selected filesystem path. C4OS Home therefore owns the profile registry and lifecycle while raw website data remains in WebKit's managed app container.
- **Rejected:** Tauri `WebviewWindow`, Wry 0.55.1 as the production surface, blanket permission denial, a page IPC bridge, and private API/profile-path manipulation.
- **Proof result:** the exact-version macOS Proof passed twice consecutively. It verified native attachment/resize/focus, no page IPC, sanitized controller events, real `Prompt` media delegation, persistent isolation, ephemeral destruction, and scoped clearing. Reliable clearing releases the cleared store handle before reopening the same stable identifier.
- **Residual risk:** production integration, forced WebContent-process recovery, broad website compatibility, target/release verification, and human acceptance remain open. The Proof registers the real process-termination callback but exercises its C4OS normalization without private crash APIs.
- **Sources:** [Wry 0.55.1 builder](https://docs.rs/wry/latest/wry/struct.WebViewBuilder.html), [Apple `WKWebsiteDataStore`](https://developer.apple.com/documentation/webkit/wkwebsitedatastore), [Apple `websiteDataStore`](https://developer.apple.com/documentation/webkit/wkwebviewconfiguration/websitedatastore), [Apple media permission delegate](https://developer.apple.com/documentation/webkit/wkuidelegate/webview(_:requestmediacapturepermissionfor:initiatedbyframe:type:decisionhandler:)), [`objc2-web-kit`](https://docs.rs/objc2-web-kit/latest/objc2_web_kit/), [macOS native-WebKit Proof](../../../proofs/macos-wkwebview-production-boundary/macos-wkwebview-production-boundary-evidence-2026-07-18.md), [inherited raw-Wry Proof](../../../proofs/native-browser-wry/README.md), [inherited browser-boundary Proof](../../../proofs/untrusted-browser-boundary/README.md).

### R-006 — Extension Delivery

- **Selected:** immutable signed packages, disabled staging, host-rendered declarative Plugins, progressive instruction/resource Skills, supervised STDIO and Streamable HTTP MCP, sandboxed reviewed hooks, brokered effects, scoped secrets, immediate revocation, and transactional update/rollback.
- **Evidence:** current MCP transport, tools, and client-security guidance plus inherited macOS Proofs cover manifest-only discovery, MCP calls, signature/digest verification, disabled install, sandboxed hooks, process cleanup, revocation, and failed-update rollback.
- **Rejected:** extension renderer bundles, native libraries, ambient credentials/authority, package-code execution during install, unsupervised MCP children, and metadata-only completion.
- **Risk:** public marketplace governance remains separately gated; production packages and servers still require conformance and hostile-fixture tests.
- **Sources:** [MCP transports](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports), [MCP tools](https://modelcontextprotocol.io/specification/2025-11-25/server/tools), [MCP security practices](https://modelcontextprotocol.io/docs/tutorials/security/security_best_practices), [MCP client practices](https://modelcontextprotocol.io/docs/develop/clients/client-best-practices), [extension trust Proof](../../../proofs/extension-trust-and-hooks/README.md), [marketplace rollback Proof](../../../proofs/marketplace-trust-and-rollback/README.md), [extension-system Proof](../../../proofs/extension-system/README.md).

### R-007 — Workspace Archive

- **Selected:** Deflate zip, a live unpacked working copy, a validated atomic archive snapshot, advisory writer lock, generation-based autosave/recovery, online database backup, and bounded safe extraction.
- **Evidence:** `zip` 8.6.0 exposes `enclosed_name`, entry type, encryption, and size metadata needed for defensive validation; SQLite provides consistent online backup; same-parent rename provides the final replacement primitive.
- **Rejected:** opening SQLite inside the zip, repacking every transaction, extracting with raw entry names, following symlinks, and writing C4OS metadata into Projects.
- **Risk:** limits require documented conservative defaults and hostile/corrupt archive fixtures.
- **Sources:** [`zip` 8.6.0](https://docs.rs/crate/zip/latest), [`ZipFile::enclosed_name`](https://docs.rs/zip/latest/zip/read/struct.ZipFile.html), [SQLite backup API](https://www.sqlite.org/backup.html), [`std::fs::rename`](https://doc.rust-lang.org/std/fs/fn.rename.html).

### R-008 — Scoped Configuration

- **Selected:** strict versioned TOML, app < Workspace < Project < Chat precedence, whole-scope validation, immutable effective snapshots, last-known-good recovery, parent-directory watching, and generation-checked UI writes.
- **Evidence:** TOML/Serde support strict typed parsing and unknown-field rejection; `notify` supports macOS filesystem events. Rust remains the only activation authority.
- **Rejected:** partial application of invalid files, list concatenation, renderer-owned merge rules, secrets in TOML, database-as-a-second-editable-source, and watcher events that mutate state without full validation.
- **Risk:** editor replace/write patterns, rapid event coalescing, external/UI conflicts, and network-mounted Project paths need production fixtures.
- **Sources:** [`toml`](https://docs.rs/toml/latest/toml/), [Serde container attributes](https://serde.rs/container-attrs.html), [`notify`](https://docs.rs/notify/latest/notify/), [`arc-swap`](https://docs.rs/arc-swap/latest/arc_swap/).

### R-009 — Physical Layout

- **Selected:** separate app and Workspace databases; editable scoped TOML; a vault; immutable content-addressed packages/blobs; distinct staging, runtime, cache, log, recovery, and temporary roots; a C4OS-owned Browser profile registry.
- **Evidence:** accepted C4OS ownership boundaries plus the R-002, R-004, R-006, R-007, and R-008 selections determine the minimal non-overlapping layout.
- **Rejected:** per-Project `.c4os` writes, raw Browser data or credentials in archives, mutable package directories as installation authority, archive backups inside archives, and duplicate writable configuration records.
- **Risk:** permissions, backup/reset UX, disk pressure, orphan cleanup, and path migration need production tests.

## Result

R-001, R-002, R-004, R-006, R-007, R-008, and R-009 are complete. R-004's one required pre-Freeze macOS Browser Proof passed twice consecutively. Existing evidence is sufficient for the selections at the planning boundary; none substitutes for production verification.
