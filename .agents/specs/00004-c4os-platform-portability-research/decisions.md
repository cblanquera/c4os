# Decisions And Gaps

## Constraints To Preserve Across Architectures

- C4OS, not a renderer, website, runtime, plugin, hook, or MCP server, owns durable product state, approvals, policy, audit, credentials, and execution authority.
- Renderers and embedded websites receive no ambient Node, Electron, C4OS, filesystem, shell, or broad IPC authority.
- Credentials remain vault-owned; product records contain only opaque references and bounded leases.
- Runtime adapters preserve shared lifecycle, capability, cancellation, event, and error contracts.
- Target evidence is not interchangeable across OS, architecture, Linux desktop/display protocol, CI, VM, or emulation.
- Existing accepted product behavior and coverage remain a regression floor unless a new spec explicitly changes them.
- Context and Frozen Specs are not changed by this Proposed package.

## Proposed Architecture Decisions

### D-001 — Restart the technical foundation, not product behavior

State: Proposed; user acceptance required

Preserve product contracts, schemas, durable identifiers, state machines, policy/action semantics, wireframes, fixtures, and acceptance/adversarial cases. Replace shell/core infrastructure only after the new architecture proves parity or an accepted change.

### D-002 — Use Electron as a thin native shell

State: Proposed; user acceptance and Proof required

Electron main owns windows, dialogs, menus, `nativeTheme`, `WebContentsView`, `session` partitions, app lifecycle, and secure-storage integration. It must not become the product domain monolith or run untrusted/runtime code in-process.

### D-003 — Put product authority in a Node application-service utility process

State: Proposed; user acceptance and Proof required

Run durable product services, policy, audit, workspace/session state, persistence, runtime registry, and MCP/plugin orchestration in an isolated Node utility process. Expose a versioned, validated, narrow control protocol through preload wrappers. Use loopback HTTP only where a real network client requires it; do not expose a broad localhost API merely because the layer is called a server.

### D-004 — Integrate Pi and OpenCode as peer runtime workers

State: Proposed; user acceptance and Proof required

Use the maintained Pi SDK in one isolated worker and the OpenCode SDK/server in another. C4OS remains the authority for runtime selection, credentials, approvals, policy, persistence, audit, cancellation, and normalized events.

### D-005 — Centralize portable behavior and isolate unavoidable native differences

State: Proposed; user acceptance and Proof required

Use shared Electron/Node paths for renderer, browser host, theme vocabulary, menu roles, runtime SDKs, application services, and most tests. Keep credentials, filesystem enforcement, process trees, PTYs, containment, signing, packaging, and native QA behind narrow target adapters.

### D-006 — Prove three targets before authorizing the rewrite

State: Proposed; user acceptance required

Require architecture Proofs on macOS arm64, Windows 11 x64, and Ubuntu 24.04 x64 GNOME/Wayland before a production implementation spec Freezes. Exact versions and hardware/VM labels remain part of the proof record.

## Gaps

### GAP-001 — Does the user accept reopening the architecture decision?

- State: Unresolved; user-owned.
- Proposed answer: Yes, conditionally, through a controlled restart after Proofs rather than an immediate destructive rewrite.

### GAP-002 — What is the renderer-to-service control transport?

- State: Unresolved; architecture Proof required.
- Proposed answer: narrow preload methods over Electron IPC/MessagePorts; use authenticated loopback HTTP only for consumers that require network transport.

### GAP-003 — Which Node and Electron release lines are support baselines?

- State: Unresolved; research refresh and Proof required.
- Rule: pin exact versions and digests in the Proof; do not inherit upstream latest labels.

### GAP-004 — Which SQLite implementation replaces Rust/rusqlite?

- State: Unresolved; benchmark, recovery, packaging, and maintenance Proof required.
- Risk: current Node 24 `node:sqlite` documentation still labels the API release-candidate stability; a mature native driver adds platform prebuild obligations.

### GAP-005 — What credential guarantee is required on each platform?

- State: Unresolved; security decision and native Proof required.
- Proposed boundary: use the asynchronous `safeStorage` API only for a vault-wrapping key, fail closed on Linux `basic_text`, and retain explicit password/session-only modes. Document Windows DPAPI's same-user limitation.

### GAP-006 — Does high-assurance filesystem authority remain possible in Node alone?

- State: Unresolved; hostile-filesystem Proof required.
- Candidate outcomes: audited Node-native implementation, a small platform-native helper, or an explicit reduction in scope. Do not silently weaken trusted-root, link/reparse, freshness, atomicity, privacy, or recovery guarantees.

### GAP-007 — How are process descendants, PTYs, and bounded secret channels controlled?

- State: Unresolved; native Proof required.
- Candidates: Electron utility processes and MessagePorts, platform process groups or Job Objects, `node-pty`/prebuilt PTY variants, and explicit process-tree termination.

### GAP-008 — What containment boundary is required for hooks, plugins, and STDIO MCP?

- State: Unresolved; threat model and adversarial Proof required.
- Rule: Node child processes and PTYs are not sandboxes. Unsupported containment must fail closed.

### GAP-009 — How are Browser sessions mapped to Electron partitions?

- State: Unresolved; functional and adversarial Proof required.
- Proposed answer: `WebContentsView` with dedicated persistent or in-memory `session` partitions, permission handlers, scoped clearing, no preload, no Node integration, and no page-accessible product bridge.

### GAP-010 — What current data/workspace compatibility is required?

- State: Unresolved; user-owned after migration inventory.
- Candidates: preserve SQLite schema and archive format, provide one-way import, or declare a development-only reset. Never infer permission to discard current data.

### GAP-011 — What is the first supported target matrix?

- State: Unresolved; user-owned after Proof results.
- Proposed Proof matrix: macOS arm64, Windows 11 x64, and Ubuntu 24.04 x64 GNOME/Wayland. Other architectures/desktops remain separate evidence rows.

### GAP-012 — Where does the old implementation live during the restart?

- State: Unresolved; workflow decision before implementation.
- Proposed answer: retain the current branch/tree as the regression reference and build the new foundation in an explicitly accepted branch or worktree; do not delete or rewrite the reference in place.

## Freeze Rule

Every Gap must be accepted, evidence-resolved, explicitly deferred, or accepted as unresolved. Critical security and architecture Gaps cannot be deferred into production implementation. Accepted architecture truth must be promoted to Context before a future sibling implementation spec depends on it.
