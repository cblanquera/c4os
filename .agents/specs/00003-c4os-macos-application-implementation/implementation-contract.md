# Implementation Contract

State: Frozen and accepted 2026-07-18

This file defines production requirements and component ownership. It does not define task order; sequencing belongs under `tasks/` after Freeze.

[Implementation selections](implementation-selections.md) is normative for the concrete renderer, persistence, Browser, extension, Workspace archive, configuration, and physical-storage boundaries.

## Application Topology

The production application has four authority tiers:

1. **Tauri/Rust core:** owns product records, policy, audit, native integration, secure-storage access, process supervision, updates, and the privileged action gateway.
2. **Renderer:** renders the accepted product state, submits typed intent, and receives normalized events. It has no direct filesystem, credential, shell, runtime, or arbitrary native IPC access.
3. **Runtime and extension workers:** OCAdapter, PIAdapter sidecar, runtime services, MCP servers, and reviewed plugin hooks operate as supervised, versioned, least-authority workers.
4. **Execution environments:** Local, Docker, and Remote SSH perform exact authorized effects while C4OS retains policy and audit authority.

Every cross-tier message uses a versioned schema, stable correlation identifiers, bounded payloads, explicit error types, and secret-redaction rules. Unknown or stale message versions fail closed.

## Core Service Boundaries

The Rust core must expose cohesive services rather than a flat command collection:

- `WorkspaceService`: zip archive manifests, unpacked working copies, trusted roots, Project ordering, scoped configuration/state, missing/relocated paths, Workspace open/save/clone, and recents.
- `SessionService`: provisional chats, first-submit binding, titles, immutable User Turns, Run Attempts, cancellation, Retry, and recovery.
- `ArtifactService`: stable artifact identity, source-run provenance, state snapshots, focus state, and brokered File/Browser/Terminal operations.
- `CapabilityService`: provider/model route discovery, declared/normalized/observed/effective capability resolution, preflight, and run snapshots.
- `PolicyService`: presets, category rules, remembered exceptions, managed ceilings, trusted-root checks, and canonical decisions.
- `ActionGateway`: single-use authorization issuance, exact effect execution, denial-before-side-effect, result normalization, and audit append.
- `CredentialService`: opaque references, OS credential service master key, encrypted vault, short-lived worker delivery, reauthentication, and redaction.
- `RuntimeSupervisor`: pinned worker versions, lifecycle, health, restart, shutdown, descendant cleanup, state namespaces, and compatibility checks.
- `ExtensionService`: marketplace catalogs, plugin staging, skill resolution, MCP definitions, enablement, revocation, transactional updates, and rollback.
- `ConfigurationService`: app/Workspace/Project/Chat configuration loading, validation, precedence, activation, diagnostics, and secret-reference enforcement.
- `UpdateService`: separate application/runtime/plugin channels, staged verification, compatibility, health, last-known-good retention, and migration snapshots.
- `PlatformService`: menus, Settings action, native pickers, window state, platform/theme snapshot, and target-qualified capabilities.

Service names are conceptual. Final code names may differ, but ownership may not collapse into renderer state or runtime-native storage.

## Workspace Archive And C4OS Home

A Workspace is a portable zip archive that owns an ordered Project list, Workspace-level configuration, per-Project configuration, and per-Chat configuration/cache/archive records. Project paths point to external trusted folders; Project and Chat overlays are keyed within the Workspace so loading a different Workspace may give the same folder different configuration and Chat history.

`~/.c4os` is C4OS Home. It contains the unpacked last-loaded Workspace plus app-level MCP, skill, plugin, and marketplace configuration, C4OS `config.toml`, and the protected local Browser profile registry. The Rust core owns archive extraction, validation, migration, save/repack, recovery, and scope resolution. C4OS-managed raw credentials remain in the credential vault and never enter a Workspace archive or ordinary C4OS Home file; website-controlled Browser state follows the local-only public-WebKit boundary below.

IS-005 through IS-007 define the archive lifecycle, scoped precedence/schema, and exact physical layout.

## Durable Record Model

The Rust-owned store must represent at least:

- installation and schema version;
- Workspace archive identity/version, unpacked-working-copy state, Projects, trusted roots, ordering, recents, and path status;
- app/Workspace/Project/Chat configuration scopes and their validated effective snapshot;
- Chat Sessions, runtime/environment binding, titles, cache/archive state, and lifecycle state;
- immutable User Turns and attachment snapshots;
- Run Attempts, correlation IDs, status, interruption/retry ancestry, and exact route/configuration/capability snapshots;
- normalized messages and streaming completion state;
- Response Artifacts, type-specific state, source run, and version history where applicable;
- provider profiles and opaque credential references;
- model routes, observations, effective descriptors, availability, and checked times;
- approval presets, category rules, remembered exceptions, maximum/managed constraints, single-use authorization state, and redacted audit events;
- runtime installations, native versions, compatibility, process generations, and health;
- execution environments and non-secret connection references;
- marketplace sources, plugin/skill/MCP identities, immutable digests, declarations, trust, activation, versions, and rollback state;
- application/runtime/plugin update state and last-known-good versions.

Use transactional writes for state transitions that cross records. Persist an event/result before reporting a consequential operation as completed. Migration failures must preserve a recoverable pre-migration snapshot.

## Typed Command And Event Boundary

Renderer commands express intent such as open workspace, create pending chat, submit turn, cancel run, answer approval, focus artifact, save file, or update configuration. They must not expose arbitrary path, process, SQL, secret, or shell primitives.

Core events include authoritative state snapshots and normalized deltas for session, run, message, work activity, approval, artifact, runtime health, configuration activation, and errors. Every run-scoped event carries session, turn, attempt, runtime, environment, and correlation identity. The core rejects stale generation events and renderer requests against obsolete state versions.

## Runtime Adapter Contract

`OCAdapter` and `PIAdapter` implement the same C4OS-owned contract:

- versioned manifest and native runtime identity;
- start, ready, health, cancel, restart, and shutdown lifecycle;
- session creation/resume capability without owning the C4OS session;
- normalized provider/model route discovery;
- model and runtime-session capability evidence;
- turn dispatch and streaming normalized events;
- tool/action requests routed only through the Action Gateway;
- bounded attachment and structured-output handling;
- explicit unsupported/degraded/unknown states;
- secret delivery through a short-lived operation channel;
- stale-correlation rejection and redacted diagnostics.

OpenCode runs as an authenticated loopback service with isolated C4OS-managed state. Pi runs through a C4OS-owned Node SDK sidecar. Per-request runtime tool settings may narrow behavior but may never exceed C4OS policy.

## Turn And Capability Lifecycle

The first valid text or attachment submission promotes a provisional chat and atomically captures its runtime, adapter, execution environment, Workspace/Project mapping, and capability baseline. Onboarding cannot reach this state until the provider test has succeeded and at least one usable model exists. The successful-test state adds no model picker: C4OS automatically chooses the production-ready model with the most normalized `supported` features, using discovery rank and stable identity as tie-breakers, and Continue persists it with OpenCode and Local. Ordinary model controls may revise the route until first-submit binding. Before every dispatch, C4OS:

1. creates stable turn, attempt, and correlation identities;
2. resolves the full provider/endpoint/model-revision/adapter/runtime route;
3. computes the effective capability intersection;
4. preflights attachments, reasoning controls, output mode, context limits, installed resources, and policy;
5. leaves incompatible input visible and requires an explicit change, conversion, removal, or cancellation;
6. snapshots prompt, attachments, route, configuration, resources, and effective capabilities;
7. dispatches through the selected adapter and normalizes deltas, activity, optional reasoning summary, tool requests, media, errors, and completion;
8. persists the final attempt state without replacing earlier attempts.

Cancel aborts only the active attempt. Retry creates a new attempt with fresh authorizations. Unknown prior side effects require user review.

## Approval And Action Execution

The UI exposes `Ask for approval`, `Approve safe actions`, `Approve for me`, and `Custom`, plus seven understandable policy groups and concrete exceptions. Internal classification may remain more granular.

Every proposed effect becomes a canonical action containing identity, normalized arguments, target, workspace, environment, requesting runtime, risk, and requested authority. Policy resolution applies managed constraints, maximum authority, sandbox, trusted roots, preset/category/exception rules, and current configuration. An approval token is single-use, short-lived, bound to exact canonical arguments and process generation, and invalid after denial, mutation, replay, cancellation, revocation, or environment substitution.

No adapter, plugin, MCP server, renderer component, or execution environment may bypass this gateway or reinterpret denial as permission.

For natural-language Chat file changes, `WorkspaceService` resolves every target against the active Project and detects whether that Project is version-controlled. In-root version-controlled writes add no separate change-set approval prompt when the effective policy allows them; an explicit `Ask` rule still applies. A non-version-controlled Project or any out-of-Project target requires pre-write approval showing the exact paths and proposed changes, and stricter sandbox or managed policy may deny it. The UI shows concise work activity, validation, and final changed-file/diff artifacts. C4OS must not automatically create a Git branch, commit, reset, or revert; Git disposition belongs to the user.

Effectful approvals serialize within each Run Attempt. Independent runs may queue approvals visibly. Every prompt binds to one canonical action, exposes pending/expired/denied/completed state, and expires on target or relevant-version change.

The composer Branch control targets only the active Project folder's Git repository and is absent for a non-Git Project. It never forks C4OS conversation or runtime state. Explicit branch operations cross the Action Gateway. C4OS never auto-stashes, commits, resets, or discards. It allows Git-safe switches that preserve dirty changes and otherwise blocks with the conflicting paths.

Remove transitions a Chat, Project, or Workspace record to inactive and excludes it from active surfaces. It performs no deletion, purge/export, or scoped-process termination.

## Credentials And Sensitive State

The OS credential service holds an installation master key for a C4OS-owned encrypted vault. Product records store opaque credential references only. A worker receives only the secret required for the current operation through a short-lived channel; secrets never appear in command-line arguments, broad inherited environment, logs, renderer state, diagnostics, workspace files, or normal exports.

When the OS credential service is unavailable, the only accepted fallbacks are an explicit password-protected vault or session-only credentials. Imports require reauthentication. Credential removal takes effect immediately and invalidates affected operations.

## Renderer And Product Surfaces

The renderer implements the complete accepted r013 hierarchy through reusable domain components and state stores, not one page-sized mutable fixture. Required domains are:

- onboarding and workspace start;
- workspace shell, projects, sessions, search, pending Chat, transcript, composer, attachments, Reply, and model/session controls;
- shared Response Artifact shells plus File, Folder, Browser, and Terminal providers in inline, contextual-pane, and focused states;
- Settings shell plus Providers, Models, Runtimes, Configuration, Advanced Policies, Plugins, Skills, and MCP Servers;
- dialogs, popovers, menus, notices, streaming activity, errors, empty/loading/degraded states, and accessible focus restoration.

Production routing must make every top-level surface directly addressable for development and QA while native Settings entry remains the user-facing route. State that must survive Settings navigation or artifact focus belongs in domain state synchronized with the authoritative core, not DOM placement.

## Normative Feature Coverage

[Feature coverage](feature-coverage.md) is part of this implementation contract. It maps every accepted usability area and routed Reference File to its implementation owner, required verification, and human-reviewable acceptance surface.

Future implementation planning must account for every row. A task may combine rows, but it may not silently omit, weaken, or defer a row. Any deferral must be explicit in this spec, preserve the complete product contract, and identify the release or feature gate that owns the remaining work.

## Platform Presentation

On macOS, use the native application menu, `Cmd+,` Settings route, and standard window decorations. Resolve the initial system scheme before revealing the shell. Use a native current-theme snapshot when available and independently observe live webview `prefers-color-scheme`; retain application state across changes.

All components consume the semantic token contract from the platform visual Reference File. Light and dark values, focus, contrast, reduced motion, platform labels, keyboard symbols, scroll behavior, and native pickers are implementation requirements. Custom titlebars, transparent/overlay chrome, or moved traffic lights require a new target-specific Proof.

## Response Artifacts And Native Facilities

- **Reply context:** `ArtifactService` captures an immutable Artifact Context Snapshot and stable reference in the User Turn. The deterministic effective-context budget prioritizes selection, visible/current state, recent state, then metadata and marks truncation. The Reply strip identifies the target; expandable details show supplied and truncated context. Normalized capabilities may be included, but secrets and authorization tokens may not.
- **File/Folder:** native pickers yield capability-scoped paths. Reads and listings are brokered. Writes use atomic replacement/conflict handling and the Action Gateway. Renderer drafts have no independent write authority.
- **Terminal:** one environment-qualified persistent shell identity per chat. Each command requires authorization before bytes reach the shell. Output is normalized and snapshotted; Stop targets only the active foreground process and preserves or honestly replaces the shell.
- **Browser:** arbitrary ordinary websites run in a Rust-owned native `WKWebView` child/controller with no page-accessible Tauri, Wry, or C4OS bridge. Public WebKit navigation/UI delegates mediate navigation, popups, downloads, and platform-default permission behavior. Browser Environment sharing partitions applicable cookies, `sessionStorage`, `localStorage`, and IndexedDB without weakening origin semantics. App-wide, Workspace+Project, and Chat profiles use persistent `WKWebsiteDataStore` identifiers; None uses a per-artifact nonpersistent data store destroyed on close. Inactivation retains profiles. Scoped clearing removes all public WebKit data types, releases the cleared store handle, and reopens the same stable identifier. C4OS Home owns the identifier registry; raw website data stays in WebKit's managed app container. Neither enters portable Workspace archives, normal exports, diagnostics, or model context. The exact-version pre-Freeze Proof passed; production availability remains gated on implementation and production verification.

File context uses selection or the current document, explicitly including and marking unsaved drafts. Folder context is bounded and non-recursive. Browser context excludes cookies, credentials, storage, and unrelated history; screenshots require an explicit supported need. Terminal context excludes raw environment values, passwords, and unrelated shell history. Additional inspection uses brokered tools, and every resulting effect revalidates the live version or returns a visible stale-state conflict.

## Extensions And Updates

Marketplace addition registers a catalog; it never installs every listing. Selected plugins stage immutable verified content disabled. Skill metadata may load for discovery, while full instructions/resources load only after eligibility and activation. MCP and executable hooks require explicit trust, sandbox, secret, timeout, output, revocation, and process-supervision rules.

This implementation must deliver the complete supported Plugin, Skill, and MCP lifecycle, including installation, activation, execution, supervision, revocation, update, rollback, and visible failure behavior. Metadata-only Settings facades or intermediate product-review slices do not satisfy the contract. Post-Freeze engineering may be sequenced, and the implementation agent may resolve bounded technical blockers from current research with a recorded rationale, but it may not silently defer functionality or weaken product and security boundaries.

Application, runtime, and plugin updates remain independent. Each update stages and verifies before activation, waits for affected active work to finish or cancel, checks compatibility/health, preserves the current working version on failure, and retains auditable provenance.

## Verification And Acceptance Gates

The first implementation and review milestone is a local macOS development build. Signing, notarization, distributable packaging, and signed-updater evidence remain later release gates; final verification and human acceptance still apply to the complete local-build contract. Production implementation must include:

- Rust unit tests for state transitions, policy, capability resolution, authorization, redaction, migrations, and recovery;
- adapter contract tests shared by OpenCode and Pi plus native integration tests for each adapter;
- denial-before-side-effect tests through real worker and execution boundaries;
- renderer component and state tests for deterministic domain behavior;
- browser-level tests for all accepted r013 routes, interactions, responsive states, accessibility, focus, overflow, theme changes, and artifact transitions;
- persistence/restart tests for sessions, attempts, artifacts, configuration activation, interrupted work, and migration rollback;
- process-supervision tests for health, cancellation, restart, descendant cleanup, stale events, and incompatible versions;
- target-specific File, Browser, Terminal, credential, menu/window, and permission checks, plus signing/updater checks when the later distribution milestone opens;
- human review of the production-rendered r013 experience in macOS Light, Dark, minimum window size, dialogs/popovers, focused artifacts, and representative failure states.

Proofs from Specs 00001 and 00002 inform test design but cannot substitute for verification against production code.

## Deferred Or Separately Gated

- Windows and Linux implementation and release evidence.
- Public marketplace authority, moderation, and compromise response.
- Codex import or compatibility claims.
- Native migration of active chats across runtimes or environments.
- Full-screen terminal applications and password-entry flows.
- Browser sub-tabs, multiple simultaneous Reply targets, and detached Chat windows.
- Named Remote SSH production profiles until each host passes its own validation.
