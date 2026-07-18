# Implementation Architecture

State: Accepted reusable truth
Accepted: 2026-07-18

This file defines the Frozen macOS implementation baseline that implementation tasks and future specs must inherit. Exact dependency versions are researched baselines to lock during the first dependency task; compatible patch updates are allowed only when they preserve these contracts.

## Delivery boundary

The first implementation and review milestone is a local macOS development build. Engineering tasks may be sequenced, but the completion target is the complete accepted application contract rather than partial product review slices. Signing, notarization, distributable packaging, signed updating, and Windows/Linux claims remain later target or release gates.

## Renderer

- Use a client-only Tauri SPA based on React 19.2, TypeScript, and Vite 8.1, with a React Router 8 hash-based data router.
- Divide Redux Toolkit state by product domain. Rust snapshots and events are authoritative; renderer stores are projections and drafts, not durable product state.
- Use React Aria Components as accessible unstyled primitives. C4OS semantic tokens and component-scoped CSS implement the accepted product contract; do not introduce a second visual design system.
- Generate Rust-owned data types with `ts-rs` and keep a small handwritten, versioned Tauri command/event adapter. Unknown versions and stale generations fail closed.
- Use Vitest and Testing Library for component/domain behavior, Playwright for renderer interaction and deterministic QA, and separate target tests for native macOS behavior.
- The renderer never owns policy, credentials, durable records, privileged operations, filesystem access, or arbitrary native IPC.

## Rust-owned persistence

- Use `rusqlite` 0.40.1 with `bundled` and `backup`, plus `rusqlite_migration` 2.6.0; the researched bundled SQLite baseline is 3.53.2.
- Keep one app database at `state/app.sqlite3` and one Workspace database at `state/workspace.sqlite3` inside the active working copy. Large content-addressed blobs remain digest-referenced files.
- Route writes through one Rust-owned database actor and allow only bounded read connections. Renderers and workers receive no database handle or SQL primitive.
- Enable foreign keys, WAL, a bounded busy timeout, `synchronous=FULL`, and explicit transactions for multi-record transitions.
- Compile ordered forward migrations into the application. Before migration, create and validate an online backup; failure keeps the previous database/version active and surfaces recovery diagnostics.
- Do not discover migrations from mutable Workspace or extension content, and do not add an ORM or async database layer without a demonstrated concurrency need.

## macOS Browser boundary

- Keep the main application on Tauri 2 and embed arbitrary websites in a Rust-owned native `WKWebView` child/controller created through public WebKit bindings. `objc2-web-kit` 0.3.2 is the researched binding baseline.
- Never attach page-accessible Tauri, Wry, C4OS, custom-scheme, or script-message IPC. Website JavaScript is not a command source.
- Only sanitized navigation, title, loading, permission, download, crash, and generation events cross from the native controller into the core.
- Public WebKit navigation/UI delegates mediate navigation, popups, downloads, and media/geolocation. Explicit policy may deny; otherwise preserve WebKit/macOS default permission behavior, including `Prompt` for media.
- Persistent Browser Environment scopes map to stable `WKWebsiteDataStore` identifiers; None uses `nonPersistentDataStore`. Scoped clearing removes all public data types, releases the cleared store handle, then reopens the same stable identifier.
- C4OS Home owns `browser/profiles.toml`; raw cookies and website-storage bytes remain in WebKit's platform-managed application container and never enter Workspace archives, normal exports, diagnostics, or model context.
- The exact-version macOS Proof passed twice consecutively. Production integration, recovery, target, release, and human-acceptance verification remain mandatory.

## Extension delivery

- Use one versioned package manifest for Plugins and Skills. Resolve marketplace selectors to immutable content, verify digest and Ed25519 origin/content signatures, quarantine, and install disabled.
- Plugins are host-rendered declarative packages with no renderer bundle, native library, ambient filesystem/process/network access, or direct secret access.
- Load Skill metadata for discovery and full instructions/resources only after source-qualified resolution, eligibility, and activation.
- Run reviewed hooks out of process with a sanitized environment, exact grants, bounded I/O, timeout, process-group cleanup, and a macOS sandbox profile. Every proposed effect still crosses the Action Gateway.
- Implement MCP against the 2025-11-25 baseline using supervised STDIO and authenticated Streamable HTTP. Isolate sessions, distrust server content/annotations, validate HTTP origin/authentication, and route tools, resources, sampling, and effects through C4OS policy.
- Definitions store only opaque credential references. Activation and updates are transactional; failure retains the prior version. Revocation stops new use and affected workers immediately. Uninstall executes no package code.

## Workspace archive lifecycle

- Use `zip` 8.6.0 with Deflate. The portable root contains `manifest.toml`, Workspace state/configuration, Chat cache/archive records, and content-addressed blobs; it excludes Project folders, credentials, and raw Browser data.
- The locked unpacked working copy is authoritative while open; the zip is an atomic portable snapshot, not a live database.
- Before extraction, validate schema/minimum-version, paths, duplicates, types, digests, entry/expanded sizes, and compression ratios. Reject encrypted entries, links, unsupported types, traversal/absolute paths, and undeclared content.
- Permit one writable owner through an advisory lock. Save through a consistent SQLite online backup, same-parent temporary archive, validation, file/parent sync, and atomic rename while retaining the last validated archive until success.
- Advance a durable working-copy generation after committed state changes. Recover a newer working copy automatically after restart and notify the user before the next archive save.
- Open Folder and Clone Repository create an untitled one-Project working copy; Save Workspace chooses the archive destination; Open Workspace validates and extracts it. Do not write C4OS metadata into Project folders.

## Scoped configuration

- Use strict TOML 1.1 parsed in Rust with `schema_version = 1`, Serde `deny_unknown_fields`, whole-document scope validation, and no raw secrets.
- App configuration owns defaults only. Workspace, Project, and Chat files use their declared subsets. Precedence is app < Workspace < Project < Chat, followed by managed ceilings and non-bypassable security constraints.
- Missing values inherit, scalars replace, declared tables merge by key, and lists replace as a whole.
- Watch parent directories, debounce replacements, require a stable read, and atomically publish immutable effective snapshots. Invalid edits retain the last-known-good state and emit path/key diagnostics.
- UI saves use a base generation and atomic same-directory replacement. Stale generations return changed-key conflicts. SQLite stores recovery/audit copies, never a second editable configuration source.

## Physical state ownership

```text
~/.c4os/
  config.toml
  state/app.sqlite3
  vault/credentials.vault
  workspace/active/
  workspace/recovery/<workspace-id>/
  mcp/servers.toml
  skills/config.toml
  skills/user/<skill-id>/...
  plugins/config.toml
  marketplaces.toml
  extensions/packages/sha256/<digest>/
  extensions/staging/
  runtimes/<runtime>/<version>/
  browser/profiles.toml
  cache/ logs/ tmp/
```

```text
workspace.zip or workspace/active/
  manifest.toml
  state/workspace.sqlite3
  config/workspace.toml
  projects/<project-id>/config.toml
  chats/<chat-id>/config.toml
  chats/<chat-id>/cache/
  chats/<chat-id>/archive/
  blobs/sha256/<digest>/
```

Editable configuration, immutable packages, mutable databases, credentials, recovery state, diagnostics, and disposable caches have separate owners. App state owns recents and provider/extension/runtime/audit records; Workspace state owns Projects, Chats, turns, runs, artifacts, and archive metadata. No record has two writable owners.

## Provenance

- [Frozen macOS implementation spec](../specs/00003-c4os-macos-application-implementation/index.md)
- [Accepted implementation selections](../specs/00003-c4os-macos-application-implementation/implementation-selections.md)
- [macOS native-WebKit Proof](../../proofs/macos-wkwebview-production-boundary/macos-wkwebview-production-boundary-evidence-2026-07-18.md)
