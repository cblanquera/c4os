# Research Findings And Sources

## Research Boundary

This pass reviewed the repository's accepted Context, all three earlier spec research packages, current source/configuration/tests, and architecture-critical primary upstream documentation/source. Access date for online sources: 2026-07-28. Documentation and source establish API shape and upstream practice; they do not substitute for native C4OS Proofs.

## Repository Research Synthesis

### R-001 — The Electron/SDK direction was already visible in the original research

State: Complete

- Spec 00001 recorded that OpenCode Desktop had moved to Electron with a managed server utility process, while other harnesses used Tauri.
- The same research identified Pi as an embeddable TypeScript SDK and OpenCode as a managed server/SDK integration.
- The earlier accepted Tauri/Rust direction was a deliberate choice, not the only feasible direction. The present request therefore reopens an architectural decision rather than discovering a missing adapter.

### R-002 — Current implementation coupling is architectural

State: Complete at audited commit plus read-only working-tree observation

- Platform qualification and production composition are exact macOS/arm64 contracts.
- The only embedded Browser host is AppKit/WKWebView.
- Credentials use macOS Keychain; hooks and STDIO MCP use macOS `sandbox-exec`.
- Runtime launch and readiness use Unix sockets, inherited file descriptors, POSIX groups/signals, Unix permissions, and Darwin-arm64 asset trees.
- Filesystem and recovery guarantees rely on POSIX descriptor-relative operations, no-follow flags, device/inode identity, modes, directory sync, and `/dev/fd`.
- Packaging scripts assume `.app/Contents/Resources` and `ditto`; fixtures and native evidence are macOS-shaped.

Affected: GAP-001, GAP-006, GAP-007, GAP-008, GAP-010.

## Primary-Source Findings

### R-003 — Electron provides a genuinely shared shell and process model

State: Complete for API research; native Proof not run

- Electron embeds Chromium and Node and targets macOS, Windows, and Linux from one JavaScript codebase.
- Electron documents `utilityProcess` as a Node child-process boundary with MessagePort support and recommends it over `child_process.fork` for Electron child processes.
- Electron security guidance requires context isolation, renderer sandboxing, no Node integration for remote content, restrictive navigation/window handling, CSP, narrow IPC exposure, and message-sender/input validation.

Sources:

- [Electron documentation](https://www.electronjs.org/docs/latest/)
- [Process model](https://www.electronjs.org/docs/latest/tutorial/process-model)
- [`utilityProcess`](https://www.electronjs.org/docs/latest/api/utility-process)
- [Security recommendations](https://www.electronjs.org/docs/latest/tutorial/security)
- [IPC guidance](https://www.electronjs.org/docs/latest/tutorial/ipc)
- [Electron fuses](https://www.electronjs.org/docs/latest/tutorial/fuses)

Affected: GAP-002, GAP-003, GAP-007, GAP-008.

### R-004 — Electron simplifies Browser and appearance integration

State: Complete for API research; native Proof not run

- `WebContentsView` provides an embedded `webContents`; Electron warns that the `<webview>` tag has architectural instability and recommends alternatives.
- `session` supports persistent `persist:` partitions, in-memory partitions, permission handlers, storage clearing, and per-session policy hooks.
- `nativeTheme` supplies shared system/light/dark observation and platform appearance inputs. High contrast and forced-colors behavior still varies by OS, so product semantics must remain normalized.
- Menu roles and `CommandOrControl` accelerators reduce shortcut branching while allowing native presentation differences.

Sources:

- [`WebContentsView`](https://www.electronjs.org/docs/latest/api/web-contents-view)
- [`<webview>` warning](https://www.electronjs.org/docs/latest/api/webview-tag)
- [`session`](https://www.electronjs.org/docs/latest/api/session)
- [`nativeTheme`](https://www.electronjs.org/docs/latest/api/native-theme)
- [`MenuItem`](https://www.electronjs.org/docs/latest/api/menu-item)

Affected: GAP-009, GAP-011.

### R-005 — Electron storage is portable in API, not identical in guarantee

State: Complete for API research; native Proof not run

- `safeStorage` uses Keychain on macOS, DPAPI on Windows, and a Secret Service backend or portal on Linux.
- Electron recommends its asynchronous encryption API for non-blocking operation, key rotation, and temporary-unavailability handling.
- Electron documents that Windows protection prevents other users but not other applications running as the same user.
- Electron exposes a Linux `basic_text` backend when no secret store is available; C4OS must detect this and fail closed rather than persist a plaintext-equivalent wrapping key.
- The API is suitable for protecting a vault-wrapping key, not replacing the existing vault/lease/redaction boundary.

Source: [`safeStorage`](https://www.electronjs.org/docs/latest/api/safe-storage)

Affected: GAP-005.

### R-006 — OpenCode validates the topology, not the C4OS authority model

State: Complete for current upstream source; C4OS Proof not run

- Current OpenCode Desktop source is Electron-based. Its package declares macOS, Windows, and Linux packaging scripts and platform/architecture optional dependencies for PTY and filesystem-watcher binaries.
- Its main process starts the server in an Electron utility process, exchanges readiness/authentication through a MessagePort, binds a loopback server, and stops/kills the child on shutdown.
- OpenCode's SDK exposes typed server/client creation plus session, event, auth, and related APIs. C4OS can use these APIs inside a worker, but must retain independent policy, approval, credential, audit, and lifecycle authority.
- Upstream versions are moving: current source observed during this pass was newer than C4OS's pinned `1.18.3`. Re-pin only after the Proof selects an exact public API boundary.

Sources:

- [OpenCode SDK](https://opencode.ai/docs/sdk/)
- [OpenCode server](https://opencode.ai/docs/server/)
- [OpenCode Desktop source](https://github.com/anomalyco/opencode/tree/dev/packages/desktop)
- [Desktop server utility process](https://github.com/anomalyco/opencode/blob/dev/packages/desktop/src/main/server.ts)
- [Desktop sidecar](https://github.com/anomalyco/opencode/blob/dev/packages/desktop/src/main/sidecar.ts)
- [Desktop package and target dependencies](https://github.com/anomalyco/opencode/blob/dev/packages/desktop/package.json)

Affected: GAP-002, GAP-003, GAP-004, GAP-007.

### R-007 — Pi is a natural Node worker dependency

State: Complete for SDK research; C4OS Proof not run

- Pi documents an embeddable TypeScript SDK with programmatic agent/session construction and extension/tool integration.
- The maintained package line has moved beyond C4OS's pinned `0.80.10`; the exact version must be selected and digested in the Proof rather than inferred from a floating latest label.
- Direct Node integration removes one Rust-to-sidecar protocol layer, but Pi remains an untrusted runtime worker under C4OS policy and credential authority.

Sources:

- [Pi coding-agent SDK guide](https://github.com/badlogic/pi-mono/blob/main/packages/coding-agent/docs/sdk.md)
- [Maintained Pi coding-agent package](https://www.npmjs.com/package/@earendil-works/pi-coding-agent)

Affected: GAP-003, GAP-004, GAP-008.

### R-008 — Node improves portability but retains native seams

State: Complete for API research; C4OS Proof not run

- `node-pty` supports Linux, macOS, and Windows, using ConPTY on supported Windows releases. Its process runs with the parent's privileges and is not a sandbox.
- Native PTY/watcher/SQLite dependencies require target prebuilds or rebuilds and exact artifact verification.
- Node 24 documentation still labels `node:sqlite` stability as release candidate. C4OS should compare it with mature maintained drivers against one-writer, WAL, backup, recovery, packaging, and hostile-path requirements before selecting it.

Sources:

- [`node-pty`](https://github.com/microsoft/node-pty)
- [Node 24 `node:sqlite`](https://nodejs.org/download/release/latest-v24.x/docs/api/sqlite.html)

Affected: GAP-004, GAP-006, GAP-007, GAP-008.

## Research Conclusion

Electron/Node is a credible and likely lower-complexity cross-platform foundation for C4OS, especially for Browser convergence and Pi/OpenCode integration. The evidence supports an architecture Proof, not an immediate production rewrite. Security- and durability-sensitive native seams must remain explicit and fail closed.
