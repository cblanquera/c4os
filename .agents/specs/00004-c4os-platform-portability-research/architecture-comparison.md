# Architecture Comparison And Recommendation

## Recommendation

A controlled restart is justified. The present cross-platform cost is structural: macOS/Unix assumptions sit inside the production composition, Browser, credential store, runtime launch, process supervision, Terminal, filesystem authority, persistence, bundle scripts, fixtures, and native evidence. Continuing with target-by-target Tauri/Rust adapters is possible, but it preserves three native browser stacks and keeps Pi/OpenCode JavaScript SDK integration behind a Rust-owned process boundary.

The restart should replace the technical foundation only after a bounded Proof. The product model and verified behavior are valuable assets and should remain the regression contract.

## Proposed Shape

```text
sandboxed React renderer
  -> narrow preload/contextBridge API
  -> thin Electron main
       windows, menus, dialogs, nativeTheme, WebContentsView/session, app lifecycle
  -> C4OS Node application-service utility process
       durable product state, SQLite, workspace/session, Action Gateway,
       policy, approvals, audit, runtime registry, MCP/plugin coordination
       -> Pi SDK utility worker
       -> OpenCode SDK/server utility worker
       -> Terminal, hook, MCP, and plugin workers
```

The service may expose an authenticated loopback interface to a genuine web/network client. The desktop control plane should prefer a smaller IPC/MessagePort contract so a compromised renderer cannot discover or call an unnecessarily broad local API.

## Comparison

| Concern | Current Tauri/Rust build | Proposed Electron/Node foundation | Judgment |
| --- | --- | --- | --- |
| Desktop shell | Tauri with macOS-gated composition and macOS menu/bundle assumptions | One Electron API surface across macOS, Windows, and Linux | Strong portability improvement |
| Renderer | React over Tauri invoke/events | Reuse React behind sandboxed preload wrappers | Mostly reusable; replace transport |
| Embedded Browser | AppKit/WKWebView now; separate WebView2 and WebKitGTK hosts later | `WebContentsView` and Electron `session` partitions use shipped Chromium across targets | Major simplification and behavior convergence |
| Theme/accessibility inputs | macOS snapshot with future target adapters | `nativeTheme`, CSS media queries, semantic tokens; some OS-specific signals remain | Meaningful simplification, not full equivalence |
| Domain/application layer | Large Rust core and command composition | TypeScript Node service/utility process | Better SDK/toolchain fit; rewrite and security-review cost |
| Pi integration | Node SDK sidecar behind Rust process/protocol layer | Pi SDK worker behind Node service | Removes a language/protocol layer while preserving isolation |
| OpenCode integration | Managed native/server sidecars and Rust adapter | OpenCode SDK/server worker behind Node service | Closer to upstream architecture; keep C4OS authority separate |
| Credentials | macOS Keychain plus explicit fallbacks | Electron `safeStorage` adapter over Keychain, DPAPI, or Linux secret store | Less adapter code, but guarantees differ by OS |
| Terminal/process | Rust PTY plus POSIX groups/signals/fds | Node PTY plus utility/child workers and target tree-control adapter | Cross-platform library support improves; cleanup still native-sensitive |
| Filesystem/persistence | Strong POSIX descriptor/inode/no-follow model | Node service plus selected SQLite/filesystem implementation and possibly a small native helper | Highest parity risk; must be proved, not assumed |
| Hooks/plugins/MCP | macOS `sandbox-exec` and fail-closed non-macOS paths | Isolated workers plus separately selected target containment | Electron does not solve sandboxing |
| Packaging/runtime assets | macOS `.app`, `ditto`, Darwin arm64 assets | Electron target packaging and per-target native dependencies/assets | Shared pipeline shape; signing/installers remain target-specific |
| Security surface | Rust core plus WebKit/Tauri bridge and sidecars | Chromium + Node + npm/native-module supply chain, strict IPC boundary required | More uniform but larger privileged/supply-chain surface |

## What To Preserve

- React product flows, semantic design tokens, wireframe intent, accessibility and responsive acceptance criteria.
- Stable IDs, schemas, migration intent, immutable records, generations, leases, recovery states, and archive/workspace contracts.
- Workspace, session, thread, turn, runtime, provider, capability, approval, Action Gateway, audit, and redaction semantics.
- Runtime conformance fixtures and normalized lifecycle/event/error contracts.
- Security/adversarial test cases and exact-target evidence rules.

Preservation means porting contracts and tests, not mechanically translating every Rust type or command.

## What To Replace Or Re-Prove

- Tauri command/event transport and the Rust production-composition surface.
- Native AppKit/WKWebView host and macOS-only Browser geometry/profile plumbing.
- `security-framework` Keychain integration, `sandbox-exec`, Unix socket/fd secret channels, POSIX process groups/signals, and macOS bundle/resource scripts.
- POSIX filesystem and SQLite enforcement where Node cannot demonstrate equivalent guarantees.
- Platform snapshots, menu accelerators, shell labels, packaging, runtime assets, and native acceptance on every named target.

## Benefits

- One JavaScript/TypeScript application stack around the two JavaScript SDKs.
- A shipped Chromium version and one embedded-browser API across target OSes.
- Less shell/theme/menu/profile branching and fewer Rust-to-Node protocol seams.
- Easier upstream alignment with OpenCode's current Electron desktop architecture.
- More shared unit/integration coverage before exact-target native acceptance.

## Costs And Risks

- The domain/application layer is a substantial rewrite; local counts on 2026-07-28 were about 135,786 Rust lines and 53,479 frontend TS/TSX/CSS lines.
- Electron adds Chromium/Node footprint and a larger privileged/supply-chain surface.
- A compromised Electron main or broad preload API has high authority; context isolation, sandboxing, input validation, CSP, navigation restrictions, and fuses are mandatory.
- Pi/OpenCode packages evolve quickly; exact versions, public API boundaries, digests, and upgrade policy must be pinned.
- Native Node dependencies require a per-OS/per-architecture prebuild, signing, and verification matrix.
- Credential, filesystem, PTY, containment, process-tree, and packaging behavior remains target-specific.

## Restart Gate

Do not delete or mutate the current implementation as the first step. Build a small architecture proof that demonstrates renderer isolation, typed control transport, service crash/restart, Pi and OpenCode sessions, Browser partition clearing, credential locked/unavailable behavior, PTY interruption/descendant cleanup, filesystem/SQLite recovery, and development packaging on all three targets. Authorize production migration only if the evidence supports the simplification claim.
