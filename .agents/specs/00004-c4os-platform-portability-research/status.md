# Status

## Lifecycle

- Phase: Architecture comparison ready for user review
- Freeze state: Not Frozen
- Opened: 2026-07-27
- Last updated: 2026-07-28
- Classification: Research-only
- Audited implementation: commit `336dd5e8f53fed85f02adf692afb0e9b53cc59b0` plus read-only working-tree observation
- Online research: Architecture-critical primary sources refreshed 2026-07-28
- Context promotion: Not started; current Tauri/Rust authority remains accepted

## Work Items

| Work item | State | Next action |
| --- | --- | --- |
| Preserve the revised user goal and scope | Complete | Keep implementation edits outside this package. |
| Inventory current platform coupling | Complete | Re-audit after any accepted implementation baseline changes. |
| Review existing repository research | Complete | Preserve provenance; do not reinterpret old Tauri conclusions as current acceptance. |
| Refresh architecture-critical upstream research | Complete | Revalidate exact versions when a Proof spec begins. |
| Compare current and proposed architectures | Complete | User accepts, revises, or rejects the controlled-restart recommendation. |
| Define restart preservation boundary | Proposed | Confirm which compatibility and migration guarantees matter. |
| Resolve critical architecture Gaps | Open | Resolve GAP-001 through GAP-012 before Freeze. |
| Execute architecture and target Proofs | Not started | Create a bounded Proof spec after direction acceptance. |
| Promote accepted architecture truth | Not started | Update Context only after accepted evidence. |
| Draft implementation contract | Blocked | Requires successful Proof disposition and Context alignment. |

## Current Recommendation

Restart the technical foundation, not the product contract. Preserve the renderer product intent, schemas, identifiers, state machines, Action Gateway/policy semantics, workspace/session/turn behavior, test fixtures, and acceptance/adversarial cases. Replace the Tauri/Rust shell-and-core composition only after a three-target Electron/Node/SDK Proof demonstrates that the new boundary is safer and materially simpler.

## Material Findings

- The current implementation is not a portable Tauri application awaiting a few adapters; macOS/Unix choices cross shell, Browser, credentials, runtime launch, process control, Terminal, filesystem safety, persistence, resources, packaging, tests, and evidence.
- Electron can unify the shipped Chromium/browser host, window/session/theme/menu APIs, JavaScript toolchain, and SDK integration across targets.
- Electron does not unify credential strength, filesystem safety, sandboxing, process-tree cleanup, PTY behavior, installers, signing, updates, or native acceptance. These still require explicit platform adapters and exact-target Proofs.
- OpenCode's current Electron desktop and Node SDK/server are strong architecture evidence, but its internal authentication and authority model must not be copied as a substitute for C4OS's stricter policy, secret, and audit boundaries.
- Pi and OpenCode should be peer runtime workers behind C4OS authority. Neither SDK belongs in the renderer or receives ambient product authority.

## Freeze Blockers

- The user has not yet accepted replacing the accepted Tauri/Rust authority boundary.
- No Electron/Node/SDK architecture Proof has run on macOS, Windows, or Linux.
- Control transport, credential fallback, SQLite driver, filesystem hardening, extension containment, migration compatibility, and first target matrix remain unresolved.
- Context and Frozen Spec 00003 still govern the current implementation.

## Recommended Next Action

Review `architecture-comparison.md` and the proposed decisions. If the direction is accepted, create one bounded architecture-Proof spec before deleting or rewriting production code.
