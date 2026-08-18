# Future Spec Projections

These are proposed planning boundaries, not accepted implementation contracts. Each future spec starts from accepted Context. This research package does not authorize the restart.

## Proposed Sequence

1. **Electron/Node architecture Proof** — prove the shell/service/worker boundary and the critical cross-platform seams on macOS arm64, Windows 11 x64, and Ubuntu 24.04 x64 GNOME/Wayland. It creates no production support claim.
2. **C4OS Electron/Node application implementation** — after successful Proofs and Context acceptance, port product contracts in bounded vertical slices while retaining the current application as a regression reference.
3. **Cross-platform native acceptance** — complete exact-target product, security, accessibility, recovery, and process evidence before declaring any OS supported.
4. **Target distribution specs** — separately own installers/packages, signing/notarization, signed update feeds, rollback, minimum-version policy, and release evidence.

## Architecture Proof Spec

| Workstream | Bounded output |
| --- | --- |
| Shell and control plane | Sandboxed renderer, narrow preload API, thin Electron main, utility-process Node service, typed/versioned protocol, crash/restart behavior |
| Runtime workers | One Pi SDK session and one OpenCode SDK/server session with isolated state, credentials, events, cancellation, and restart |
| Browser | `WebContentsView`, persistent/ephemeral partitions, permission handlers, clearing, crash recovery, no page bridge |
| Credentials | `safeStorage` capability probe, vault-wrapping-key flow, locked/unavailable/fallback behavior on each target |
| Terminal/process | PTY packaging, resize/interrupt, process-tree ownership/cleanup, bounded readiness and secret channels |
| Persistence/filesystem | SQLite candidate comparison, one-writer/WAL/recovery, hostile path/link/reparse and atomicity tests, native-helper decision |
| Containment | Hook/plugin/MCP threat model and at least one denial/cleanup Proof per target; fail-closed disposition where unavailable |
| Packaging | Development artifact with exact per-target native dependencies, SDK/server assets, locks, digests, and verification |

## Application Implementation Spec

The implementation should proceed by product behavior rather than by translating Rust modules one-for-one:

1. platform shell, renderer transport, diagnostics, and recovery;
2. workspace/session/thread/turn persistence and lifecycle;
3. providers, vault, Pi and OpenCode runtime parity;
4. Action Gateway, approvals, audit, execution, Terminal, and Git;
5. Browser, Reply context, profiles, permissions, and clearing;
6. MCP, plugins, skills, hooks, updates, archives, and remaining accepted coverage;
7. full cross-platform regression and native acceptance.

Each slice must state which existing contracts/tests are preserved, intentionally changed, or deferred. Compatibility changes require user acceptance; uncommitted work is not silently discarded.

## Scope Rules

- Keep Electron main thin and never run runtime SDKs, plugins, or arbitrary work in the renderer.
- C4OS application-service authority remains distinct from Pi/OpenCode server authority.
- Use narrow target adapters for credentials, filesystem enforcement, process trees, PTYs, containment, and packaging.
- A compiling/packageable artifact is not a native behavior claim; CI is not interactive visual or accessibility evidence.
- Keep exact-target development acceptance separate from public distribution unless the user explicitly combines them.
