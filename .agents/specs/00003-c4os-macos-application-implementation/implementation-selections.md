# Implementation Selections

State: Frozen with Spec 00003 on 2026-07-18; accepted reusable baselines promoted to Context

These selections close the delegated technical Gaps without defining implementation order. Exact dependency versions are the researched baselines and must be locked in the first post-Freeze dependency task; compatible patch updates are allowed when they preserve these contracts.

## IS-001 — Renderer

- Use React 19.2 with TypeScript and Vite 8.1 as a client-only Tauri SPA.
- Use React Router 8 with a hash-based data router so development and packaged direct routes share one deterministic route model without server fallback behavior.
- Use Redux Toolkit stores divided by product domain. Rust snapshots and events remain authoritative; renderer stores are projections and drafts, and are not independently persisted to browser storage.
- Use React Aria Components for accessible unstyled interaction primitives, with C4OS semantic tokens and component-scoped CSS implementing the r013 contract. Do not adopt a visual component kit that becomes a second design system.
- Generate Rust-owned data types with `ts-rs` and keep a small handwritten, versioned Tauri command/event adapter around them. Unknown versions and stale generations fail closed.
- Use Vitest and Testing Library for component/domain behavior and Playwright for renderer routes, focus, keyboard, responsive, accessibility, and deterministic QA flows. Native macOS behavior remains a separate target test surface.
- Do not use `tauri-specta` while its selected production API remains prerelease. Do not store policy, credentials, durable records, or privileged operations in React/Redux.

## IS-002 — Rust-Owned Persistence

- Use `rusqlite` 0.40.1 with `bundled` and `backup`, plus `rusqlite_migration` 2.6.0. The bundled baseline is SQLite 3.53.2.
- Maintain one app database at `state/app.sqlite3` and one Workspace database inside the active working copy at `state/workspace.sqlite3`. Content-addressed large blobs remain files referenced by digest rather than SQLite blobs.
- Route writes through one Rust-owned database actor. Permit bounded read connections for snapshots and queries; the renderer and workers never receive a database handle or SQL primitive.
- Enable foreign keys, WAL, a bounded busy timeout, and `synchronous=FULL` for durable product state. Every multi-record state transition uses an explicit transaction.
- Compile ordered forward migrations into the application, validate their sequence in tests, and use SQLite `user_version` through the migration library. Do not discover or execute migrations from extensions or mutable workspace content.
- Before migration, create and validate an online backup. On migration or validation failure, keep the prior database and version active and surface recovery diagnostics.
- Reject an ORM or async database framework for this milestone: C4OS needs explicit transactional ownership and backup behavior, not a second persistence abstraction.

## IS-003 — macOS Browser Boundary

- Keep the main application on Tauri 2. Use a Rust-owned native `WKWebView` child/controller for arbitrary websites rather than a Tauri `WebviewWindow` or the current raw-Wry builder.
- Create the surface through public WebKit bindings, with `objc2-web-kit` 0.3.2 as the researched Rust binding baseline. Never attach a page-accessible Tauri, Wry, C4OS, custom-scheme, or script-message IPC handler.
- Send only sanitized navigation, title, loading, permission, download, crash, and generation events from the native controller to the C4OS core. Renderer commands express browser intent; website JavaScript is never a command source.
- Handle navigation, popups, downloads, and media/geolocation requests through public WebKit navigation/UI delegates. Explicit C4OS policy may deny; otherwise the delegate preserves WebKit/macOS default permission behavior rather than blanket-granting or blanket-denying.
- Map persistent Browser Environment scopes to stable `WKWebsiteDataStore` identifiers. Use `nonPersistentDataStore` for None. Use WebKit data-store APIs for inspection and scoped clearing. After clearing all data types for one persistent store, release its current handle before reopening the same stable identifier.
- `~/.c4os/browser/profiles.toml` owns the scope-to-data-store identifier registry and lifecycle. Raw cookies and website-storage bytes remain in WebKit's platform-managed application container because public WebKit APIs do not accept a custom profile path. They remain local and excluded from Workspace archives, exports, diagnostics, and model context.
- The required Proof in `proofs.md` passed twice consecutively on the locked macOS/Tauri/WebKit versions. This settles pre-Freeze feasibility only; production integration, recovery, target, release, and human-acceptance verification remain mandatory.

## IS-004 — Extension Delivery

- Define one versioned C4OS package manifest for Plugins and Skills. Resolve marketplace selectors to immutable content, verify digest and Ed25519 origin/content signatures, stage in quarantine, and install disabled.
- Plugins are declarative packages rendered through host-owned schemas and components. They receive no renderer bundle, native library, ambient filesystem/process/network access, or direct secret access.
- Skills contain instructions and resources. Load metadata for discovery, then load complete instructions/resources only after source-qualified resolution, eligibility, and activation. Collisions require an explicit source identity.
- Run reviewed executable hooks out of process with a sanitized environment, exact grants, bounded input/output, timeout, process-group cleanup, and a macOS sandbox profile. Every proposed effect still crosses the Action Gateway.
- Implement MCP against the 2025-11-25 protocol baseline: supervised STDIO subprocesses and authenticated Streamable HTTP. Isolate server sessions, treat tool annotations and server content as untrusted, validate HTTP origins/authentication, and route tools, resources, sampling, and effects through C4OS policy.
- Store only opaque credential references in definitions. Deliver the minimum secret through a short-lived core-owned channel for the current operation.
- Activation is transactional. Updates stage and verify before switching; failed activation/migration keeps the prior version. Revocation immediately disables new use and terminates affected active workers. Uninstall removes active records and disposable cache without executing package code.
- Existing macOS trust, sandbox, supervision, MCP, revocation, and rollback Proofs establish this selection's pre-Freeze feasibility. Production conformance and failure tests remain mandatory.

## IS-005 — Workspace Archive Lifecycle

- Use `zip` 8.6.0 with Deflate as the portable archive format. The archive root contains `manifest.toml`, the Workspace database, scoped configuration, Chat cache/archive records, and content-addressed blobs; it never contains Project folders, credentials, or raw Browser data.
- Treat the unpacked working copy as the authoritative autosave state while a Workspace is open. The `.zip` is an atomic portable snapshot, not the live database.
- On open, validate the manifest, schema/minimum application version, entry names, duplicates, entry types, declared digests, entry count, per-entry size, aggregate expanded size, and compression ratio before promoting a temporary extraction directory. Reject encrypted entries, symlinks, unsupported types, absolute/traversal paths, and undeclared content.
- Acquire an advisory Workspace lock before mutation. If another writer owns it, open read-only or report the owner; never create two writable working copies silently.
- Save by creating a consistent SQLite online backup, staging configuration/blobs, writing a same-parent temporary archive, validating it, syncing the file and parent directory, and atomically renaming it. Retain the last validated archive until the replacement succeeds.
- Advance a durable working-copy generation after each committed state change. Coalesce archive repacks after idle periods and on close/switch. If restart finds a newer working-copy generation, recover it automatically and show a recovery notice before the next archive save.
- Open Folder and Clone Repository create an untitled working copy containing one Project reference. Save Workspace chooses the archive destination. Open Workspace extracts a saved archive. None of these actions writes C4OS metadata into the external Project folder.

## IS-006 — Scoped Configuration

- Use strict TOML 1.1 documents parsed by Rust with Serde `deny_unknown_fields`. Every document starts with `schema_version = 1`; unknown keys, invalid types, secrets, and unsupported enum values reject the whole changed scope.
- App `config.toml` owns application defaults only: restore-last-workspace; optional model-route reference; default runtime, environment, and approval preset; shell-environment inheritance; Browser Environment default; independent update channels; and diagnostics retention/redaction preferences.
- Workspace, Project, and Chat `config.toml` files use the declared subset valid for that scope. Raw credentials, provider secrets, Chat history, package manifests, and managed-policy ceilings are not configuration values.
- Resolve app < Workspace < Project < Chat. Missing values inherit; scalar values replace; declared tables merge by key; lists replace as a whole. Apply managed ceilings and non-bypassable security constraints after all user scopes.
- Watch each configuration file's parent directory because editors may replace the file. Debounce events, require a stable read, parse and validate the complete scope, then atomically publish an immutable effective snapshot. Invalid edits keep the last-known-good snapshot active and emit path/key diagnostics.
- UI saves go through `ConfigurationService` with a base generation. Rust rejects stale saves with changed-key details, writes a same-directory temporary file, syncs and atomically replaces it, and deduplicates the resulting watcher event.
- SQLite stores the last-known-good canonical document, effective generation, and audit/diagnostic record for recovery; it is not a second independently editable configuration source.

## IS-007 — Physical Storage Layout

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
  cache/
  logs/
  tmp/
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

- `manifest.toml` carries the schema version, Workspace UUID, generation, minimum application version, ordered Project references and last-known paths, and file digests.
- Editable configuration, immutable packages, mutable databases, disposable caches, diagnostics, recovery state, and credentials occupy distinct roots. Caches may be rebuilt and are never archive authority.
- App state owns recents and provider/extension/runtime/audit records. Workspace state owns Projects, Chats, turns, runs, artifacts, and archive metadata. Do not maintain two writable owners for the same record.
- Backup and reset operations work by declared root. Never put archive backups inside the archive, package caches inside editable package sources, credentials in configuration, or website data in C4OS-managed portable records.
