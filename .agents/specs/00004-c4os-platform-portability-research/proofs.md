# Proof Plan

State: Proposed; no Electron/Node restart Proof has run

Every result is scoped to the exact OS release/build, architecture, Electron/Chromium/Node versions, package locks, native dependency builds, toolchain, and artifact. CI, Docker, simulation, VM, emulation, and interactive hardware labels remain distinct.

| Proof | macOS arm64 | Windows 11 x64 | Ubuntu 24.04 x64 GNOME/Wayland | Required signal |
| --- | --- | --- | --- | --- |
| PF-001 Shell/process isolation | `not run` | `not run` | `not run` | Sandboxed renderer exposes only narrow typed preload methods; service runs in a utility process; malformed/spoofed IPC fails closed; service crash/restart cannot corrupt product state. |
| PF-002 Pi SDK worker | `not run` | `not run` | `not run` | Exact Pi package launches in an isolated worker; model/auth/tool/session/events/cancel/restart normalize without leaking credentials or bypassing approvals. |
| PF-003 OpenCode SDK/server worker | `not run` | `not run` | `not run` | Exact OpenCode SDK/server launches with isolated state and bounded authentication; sessions/events/cancel/restart normalize under C4OS authority. |
| PF-004 Browser/profile isolation | `not run` | `not run` | `not run` | `WebContentsView` has no Node/preload/product bridge; persistent and ephemeral partitions, permissions, navigation, clearing, crash recovery, focus, geometry, and Reply capture satisfy the contract. |
| PF-005 Theme/shell/accessibility | `not run` | `not run` | `not run` | System/light/dark changes, contrast/forced-colors where available, motion, text scaling, shortcuts, menus, dialogs, focus, resize, and native vocabulary work interactively. |
| PF-006 Credential protection | `not run` | `not run` | `not run` | Vault wrapping key uses the intended OS protection; locked/unavailable/fallback/restart/reauth paths fail closed; Linux `basic_text` is rejected; logs and IPC contain no secret. |
| PF-007 Terminal/process tree | `not run` | `not run` | `not run` | Exact PTY dependency packages; resize/interrupt/foreground behavior works; cancel, revoke, crash, and quit terminate every descendant; bounded secret/readiness channels close correctly. |
| PF-008 Filesystem/SQLite/recovery | `not run` | `not run` | `not run` | Trusted-root, link/reparse defense, identity freshness, atomic create/replace, private files, locks/watchers, WAL, backup/restore, corrupted DB, and hostile archives meet the accepted guarantees. |
| PF-009 Hook/plugin/MCP containment | `not run` | `not run` | `not run` | Filesystem/network/process/environment denial, exact grants, timeout, descendant cleanup, revocation, and bounded I/O pass adversarial tests; unsupported targets fail closed. |
| PF-010 Development packaging | `not run` | `not run` | `not run` | Packaged artifact contains exact Electron, app, Pi, OpenCode, PTY/watcher/SQLite assets and digests; missing/mismatched resources fail before launch. |
| PF-011 Integrated native acceptance | `not run` | `not run` | `not run` | Preserved product, security, accessibility, responsive, restart, recovery, degraded-state, secret/log, process, and P0/P1 matrices pass on the named interactive target. |

## Proof Requirements

- Define hypothesis, expected signal, failure signal, scope, non-goals, and target record before execution.
- Preserve exact commit, OS/build, architecture, Electron/Chromium/Node/package-manager/package versions, lockfile, commands, logs, artifact digests, and native captures.
- Distinguish public Pi/OpenCode SDK use from imports of upstream internal modules.
- Exercise main, renderer, preload, service, runtime-worker, Browser, and native dependency compromise/failure boundaries.
- Preserve the current macOS implementation as a comparison baseline; a new proof is not permission to delete it.
- A failed Proof yields a fallback only through the decision ledger and user acceptance.
