# Decisions And Gaps

## Inherited Accepted Constraints

These constraints come from Context and are not reopened by this spec:

- The Rust/Tauri C4OS core retains durable product state and security authority; the renderer and supervised workers receive no ambient authority.
- OpenCode and Pi are peer runtimes behind separate adapters.
- Plugins are declarative bundles whose executable surfaces remain C4OS-governed.
- Websites are unprivileged and receive no page-accessible C4OS or Tauri IPC.
- Only the Rust core reads stored secrets; other records carry opaque credential references.
- Runtime/environment defaults bind on a chat's first valid submission and never migrate an existing chat implicitly.
- Retry creates a new Run Attempt under an immutable User Turn and never silently duplicates a side effect.
- The forward product experience is r013, including four approval presets, seven policy groups, capability-aware chat, compact provenance, and system-following platform presentation.
- macOS uses standard decorations, a native application menu, `Cmd+,` for Settings, a native initial-theme snapshot when available, and an independent live webview theme listener.

## Implementation Boundaries From Accepted Context

### D-001 — Use a split-plane application

State: Inherited from accepted Context; no user decision required

Use one Tauri application whose Rust core owns commands, records, policy, secure-storage access, supervision, and native integration. The renderer communicates through a small typed command/event boundary. Runtime adapters, MCP servers, and execution environments are workers behind the core, never alternate authorities.

### D-002 — Keep product records runtime-neutral

State: Inherited from accepted Context; no user decision required

Persist C4OS identifiers and normalized records independently of OpenCode or Pi native session formats. Store native runtime identifiers and versions as provenance so adapters can be replaced, recovered, or upgraded without transferring ownership of the C4OS transcript or audit history.

### D-003 — Implement one privileged action gateway

State: Inherited from accepted Context; no user decision required

All filesystem writes, process execution, network/share operations, browser/desktop facilities, credential use, and extension execution cross one canonical action boundary. The gateway classifies the action, resolves policy, validates trusted scope, mints a single-use authorization, executes the exact approved effect, and appends a redacted audit result.

### D-004 — Use semantic UI contracts rather than wireframe internals

State: Inherited from accepted Context; no user decision required

Implement production components, routes, and state models from the usability and Reference contracts. Reuse r013 only as a behavioral and visual acceptance oracle. Do not copy its fixture IDs, simulated persistence, review-only routes, or static SPA state as production architecture.

## Evidence-Resolved Implementation Clarifications

### D-005 — Gate first launch on a configured provider

State: Accepted by the user 2026-07-18. When no provider exists, launch into the standalone provider form without Settings navigation. Continue requires the latest connection test to succeed and at least one usable model to be available. C4OS stores secret material outside renderer/product records, retains redacted provider metadata, and proceeds to Workspace Start. D-020 defines the initial model/runtime/environment defaults.

### D-006 — Treat non-Chat composer modes as direct operation lanes

State: Accepted by the user 2026-07-18. Chat and Reply submit natural-language AI turns. Files `Open`, Browser `Open` and navigation, Terminal `Run`, and expanded Terminal input are direct C4OS-brokered operations and do not invoke the model merely because their results render as C4OS Response Artifacts. Approval and security policy still apply even when model and approval selectors are hidden. Replying to an artifact returns to Chat/AI semantics.

### D-007 — Keep chat search results inside the project/session navigator

State: Accepted by the user 2026-07-18 and reconciled to the complete accepted r013 revision. A non-empty chat-session search replaces the Projects heading and hierarchy in the left navigator with flat session-title results. Every result identifies its owning Project. Opening a result preserves the search query and keeps the selected Chat in the center workspace rather than opening a search-results page. Explicit clear or Escape restores Projects with its prior ordering and expansion state.

### D-008 — Scope terminal shells to one chat

State: Accepted by the user 2026-07-18. Each Chat owns one environment-qualified persistent shell identity. Its Terminal artifacts are immutable command snapshots of that shell. Other Chats do not share the shell, even when they belong to the same Project or Workspace. Browser artifacts/history are Chat-owned; D-014 defines profile/environment sharing.

### D-009 — Separate direct File editing from agent-directed Chat changes

State: Accepted by the user 2026-07-18. Direct user editing inside a File artifact is distinct from a natural-language Chat request such as creating `AGENTS.md` and updating `CONTEXT.md`. The Chat agent reads and changes files only through C4OS-brokered tools and policy. For version-controlled targets inside the active Project, C4OS adds no separate change-set approval prompt when the effective policy allows the writes; an explicit `Ask` rule can still prompt. A non-version-controlled Project or any target outside the active Project requires path-and-change-specific approval before writing, subject to stricter sandbox or managed denial. The transcript shows concise activity, validation, and completed changed-file/diff artifacts. C4OS never creates a Git branch, commits, resets, or reverts automatically; the user owns Git disposition. GAP-011 is resolved; D-013 separately defines artifact context.

### D-010 — Keep C4OS configuration distinct from Codex configuration

State: Accepted product boundary 2026-07-18. C4OS `config.toml` lives under `~/.c4os`, cannot imply Codex compatibility, and excludes raw secrets and non-bypassable managed policy. Workspace-, Project-, and Chat-scoped overlays stay in the Workspace archive. D-026 and IS-006 fix the evidence-backed schema, precedence, reload, diagnostics, and Rust-record reconciliation.

### D-011 — Use `~/.c4os` as C4OS Home

State: Accepted by the user 2026-07-18. `~/.c4os` contains the unpacked last-loaded Workspace, app-level MCP/skill/plugin configuration, marketplace configuration, C4OS `config.toml`, and the local Browser profile registry. Workspace-, Project-, and Chat-scoped overlays stay in the Workspace archive so the same Project folder can behave differently in different Workspaces. C4OS-managed credentials remain vault-only. D-027 fixes the selected physical layout; D-023 records the public-WebKit storage boundary.

### D-012 — Model a Workspace as a portable archive

State: Accepted by the user 2026-07-18. A Workspace is a zip archive analogous to a VS Code workspace. It owns its ordered Project references, Workspace configuration, per-Project configuration, and per-Chat configuration/cache/archive records used to reconstruct the main screen. A Project is an external folder/trusted root rather than the global owner of those overlays.

### D-013 — Use bounded Artifact Context Snapshots with brokered expansion

State: Accepted by the user 2026-07-18. Reply captures an immutable, type-specific snapshot and stable artifact reference in the User Turn. File, Folder, Browser, and Terminal each contribute only their relevant selected/current state within a deterministic effective-context budget; truncation and unsaved File state are explicit, sensitive Browser/Terminal state is excluded, and normalized capabilities contain no secrets or authorization tokens. Additional reads remain brokered. C4OS revalidates live versions before effects and surfaces stale-state conflicts. The Reply strip identifies the target, while expandable details expose the supplied/truncated context.

### D-014 — Partition every applicable browser storage category
State: Accepted by the user 2026-07-18. Browser Environment sharing covers cookies, `sessionStorage`, `localStorage`, and IndexedDB where applicable while preserving origin semantics. All browsers uses one persistent app-wide profile; Per project and Per chat session use persistent Workspace+Project- and Chat-keyed profiles; None is per-artifact ephemeral and removed on close. Inactivation retains profiles; explicit clearing is scoped. C4OS Home owns the protected profile registry; public WebKit owns the raw local website-data container. Neither enters Workspace archives, normal exports, diagnostics, or model context.
### D-015 — Make Branch a Git-only Project control
State: Accepted by the user 2026-07-18. Branch represents the active Project folder's Git repository and applies only to files/folders inside it; it never forks C4OS conversation or runtime state and is hidden for non-Git Projects. Explicit operations remain brokered. C4OS never auto-stashes, commits, resets, or discards; Git-safe switches may preserve dirty changes, while conflicting switches are blocked with affected paths.
### D-016 — Treat Remove as inactivation only
State: Accepted by the user 2026-07-18. Removing a Chat, Project, or Workspace only marks its product record inactive and excludes it from active surfaces. It does not delete user files or C4OS records, purge/export state, or terminate scoped processes.
### D-017 — Serialize and state-bind approvals
State: Accepted by the user 2026-07-18. Effectful approvals serialize within one Run Attempt; independent runs queue visibly. Each prompt binds to one canonical action, distinguishes pending, expired, denied, and completed state, and expires when its target or relevant version changes.

### D-018 — Use a local development build as the first milestone
State: Accepted by the user 2026-07-18. The first implementation and review milestone is a local macOS development build. Signing, notarization, distributable packaging, and signed-updater evidence remain later release gates; this delivery posture does not remove production behavior from the implementation contract.

### D-019 — Implement the complete contract without review-slice gates
State: Accepted by the user 2026-07-18. The implementation target is the complete accepted application contract, including real supported Plugin, Skill, and MCP installation, activation, execution, supervision, revocation, and failure behavior. Metadata-only facades or a partial product slice do not count as completion. Task sequencing may divide engineering work, but it must not introduce intermediate product-review gates or silently defer accepted functionality. During post-Freeze implementation, the agent is delegated authority to resolve bounded technical blockers from current research and record the decision and rationale; it may not weaken accepted product behavior, security boundaries, final verification, or human acceptance.

### D-020 — Select onboarding defaults without another step
State: Corrected by the user 2026-07-27 after production review. After the required successful provider test yields at least one usable model, onboarding does not show a model picker or default-confirmation panel. C4OS automatically selects the production-ready model with the most normalized `supported` features, breaking ties by discovery rank and stable model identity, then persists it with OpenCode and Local when Continue enters Workspace Start. The ordinary post-onboarding model controls own later changes before first-Chat binding.

### D-021 — Select the renderer stack
State: Evidence-backed selection 2026-07-18. Use the React/TypeScript/Vite SPA, domain Redux Toolkit stores, React Aria primitives, typed Rust boundary, and test layers defined by [IS-001](implementation-selections.md#is-001--renderer).

### D-022 — Select Rust-owned SQLite persistence
State: Evidence-backed selection 2026-07-18. Use bundled `rusqlite`, compiled atomic migrations, a single writer, WAL, online pre-migration backup, and separate app/Workspace databases as defined by [IS-002](implementation-selections.md#is-002--rust-owned-persistence).

### D-023 — Select a public-WebKit macOS Browser controller
State: Evidence-backed selection 2026-07-18; required Proof passed 2026-07-18. Use the no-page-IPC native `WKWebView` boundary, stable/ephemeral `WKWebsiteDataStore` profiles, and C4OS-owned registry defined by [IS-003](implementation-selections.md#is-003--macos-browser-boundary). Scoped clearing releases the cleared store handle before reopening the same stable identifier. Production and release verification remain required.

### D-024 — Select the complete extension boundary
State: Evidence-backed selection 2026-07-18. Use signed immutable declarative packages, progressive Skills, supervised MCP, sandboxed reviewed hooks, scoped secrets, brokered effects, revocation, and transactional rollback as defined by [IS-004](implementation-selections.md#is-004--extension-delivery).

### D-025 — Select the Workspace archive lifecycle
State: Evidence-backed selection 2026-07-18. Use a live locked working copy plus defensively validated, generation-tracked, atomically replaced zip snapshots and crash recovery as defined by [IS-005](implementation-selections.md#is-005--workspace-archive-lifecycle).

### D-026 — Select strict scoped configuration
State: Evidence-backed selection 2026-07-18. Use versioned secret-free TOML, app-to-Chat precedence, whole-scope validation, last-known-good snapshots, watched external edits, and generation-checked UI writes as defined by [IS-006](implementation-selections.md#is-006--scoped-configuration).

### D-027 — Select the physical state layout
State: Evidence-backed selection 2026-07-18. Use the separated configuration, databases, vault, immutable packages/blobs, working copy, recovery, runtime, cache, diagnostics, and Browser registry roots defined by [IS-007](implementation-selections.md#is-007--physical-storage-layout).

## Gap Disposition

### GAP-001 — What is the first shippable implementation slice?

- **State:** Resolved by accepted D-019. Engineering tasks may be sequenced after Freeze, but only the complete contract is the implementation-completion target; there are no partial product review slices.

### GAP-002 — Which renderer framework and front-end structure should production use?

- **State:** Resolved by R-001, D-021, and IS-001. Dependency versions are locked during the first post-Freeze dependency task without reopening the selected architecture.

### GAP-003 — Which durable store and migration mechanism should the Rust core own?

- **State:** Resolved by R-002, D-022, and IS-002. Production recovery and failure-path tests remain implementation verification.

### GAP-004 — In what order should OCAdapter and PIAdapter reach production readiness?

- **State:** Deferred by user direction to post-Freeze task planning. Both adapters retain peer architectural status; delivery order does not block Freeze.

### GAP-005 — Which exact released Wry/Tauri browser boundary will production use?

- **State:** Implementation selection resolved by R-004, D-023, and IS-003. Current Wry lacks the required public permission boundary, so the selected native `WKWebView` controller must pass the exact-version pre-Freeze Proof; Browser functionality is not deferred.

### GAP-006 — What is the first release posture?

- **State:** Resolved by accepted D-018. The first milestone is a local macOS development build; distribution signing, notarization, packaging, and signed-updater evidence remain later release gates.

### GAP-007 — How much executable extension behavior belongs in the first slice?

- **State:** Resolved by accepted D-019 plus R-006, D-024, and IS-004. Plugins, Skills, and MCP Servers require their complete supported installation and execution behavior; metadata-only UI does not satisfy the contract.

### GAP-008 — Workspace, Project, and Chat relationship

- **State:** Resolved by accepted D-012. Each Chat and Project overlay belongs to its Workspace; each Project references an external trusted root.

### GAP-009 — How is a Workspace archive loaded, saved, and recovered safely?

- **State:** Resolved by R-007, D-025, and IS-005. Concrete extraction limits are conservative implementation constants covered by hostile/corrupt archive tests.

### GAP-010 — How are initial operating defaults selected after onboarding?

- **State:** Resolved by D-020 as corrected during production review. The successful-test state keeps the compact provider form, automatically chooses the strongest supported usable model, and leaves later changes to ordinary model controls.

### GAP-011 — What does the user see during agent-directed file changes from Chat?

- **State:** Resolved by accepted D-009. Repository and Project boundaries determine whether an extra pre-write review is required; exact batching may vary as long as guarded writes are approved before effect and all changes remain brokered, visible, and attributable.

### GAP-012 — What artifact state is supplied to AI and how may AI act on it?

- **State:** Resolved by accepted D-013. The type-specific snapshot, context-budget, sensitive-state exclusion, transparency, brokered expansion, and freshness contract are fixed; exact numeric budget tuning remains implementation configuration rather than a product-boundary Gap.

### GAP-013 — What does each Browser Environment sharing option mean?

- **State:** Resolved by accepted D-014. Applicable storage categories, scope identities, restart persistence, ephemeral destruction, inactivation retention, explicit clearing, and portable-archive exclusion are fixed.

### GAP-014 — What is the scoped C4OS configuration contract?

- **State:** Resolved by D-010 plus R-008, D-026, and IS-006. The accepted scope ownership is unchanged; strict schema and reconciliation behavior are now selected.

### GAP-015 — What is the physical layout beneath C4OS Home and Workspace archives?

- **State:** Resolved by D-011 plus R-009, D-027, and IS-007. Public WebKit owns raw website-data bytes in its managed app container; C4OS Home owns the profile registry and lifecycle.

### GAP-016 — What does the Branch composer control do?

- **State:** Resolved by accepted D-015. Branch is Git-only and Project-scoped; dirty changes are preserved only when Git permits, and conflicting switches block without automatic worktree mutation.

### GAP-017 — What are deletion, retention, recovery, and export rules?

- **State:** Resolved by accepted D-016. Remove only marks the selected product record inactive and has no deletion, purge/export, or process-termination side effect.

### GAP-018 — How are concurrent approvals and agent actions presented?

- **State:** Resolved by accepted D-017. Approvals serialize within a run, queue visibly across runs, bind to canonical actions, expose lifecycle state, and expire on target/version change.

## Freeze Rule

Convert each assumption into an accepted decision, an evidence-backed answer, or an explicit deferral before Freeze. If a selected implementation conflicts with Context, record the conflict and resolve it through Context review rather than silently weakening the inherited contract.
