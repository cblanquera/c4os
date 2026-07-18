# Decisions and Gaps

This ledger is Frozen as of 2026-07-18. P-001 through P-018 are accepted; GAP-008 and GAP-013 are deferred release decisions with explicit triggers.

## Accepted decisions

### P-001 — Treat r012 as product intent

State: Accepted 2026-07-18

The wireframe specifies navigation, states, and interaction contracts. Its deferred native, network, filesystem, credential, persistence, PTY, and runtime behavior must not be cited as implemented capability.

### P-002 — Keep C4OS policy above runtime adapters

State: Accepted 2026-07-18

C4OS should own approval policy, trusted roots, artifact identity, persistence, credentials, and normalized events. OpenCode and Pi are peer user-selectable runtimes implemented through separate `OCAdapter` and `PIAdapter` contracts; neither is architecturally primary. A shared C4OS baseline is derived from their declared capabilities rather than imposed before those capabilities are mapped.

### P-003 — Implement OpenCode as a peer runtime through a managed server

State: Accepted 2026-07-18; tested adapter boundary passed

OpenCode exposes an HTTP/SSE SDK surface, and OpenWork demonstrates a Tauri host managing that topology. This is a stronger reference for C4OS than Atomic Chat's external terminal launch. The process must bind to loopback, use authentication, isolate state, and have explicit start, health, restart, and shutdown semantics. Its adapter must satisfy the same C4OS contract as Pi without making OpenCode's native concepts product-wide requirements.

### P-004 — Implement Pi as a peer runtime through a C4OS Node SDK sidecar

State: Accepted 2026-07-18; tested transport selected

Pi offers an embeddable TypeScript SDK and a line-delimited JSON RPC mode. PIAdapter should use a C4OS-owned Node SDK sidecar because the current SDK provides direct pre-tool interception, explicit tool/resource control, and stronger adapter-owned correlation. RPC remains a documented alternative. This selects Pi's integration transport; it does not rank Pi against OpenCode or make either runtime primary.

### P-005 — Model plugins as declarative bundles

State: Accepted 2026-07-18; local lifecycle and macOS hook boundary passed

Start with metadata, skills, MCP/app declarations, settings schemas, and explicitly reviewed hooks. Do not permit plugin-provided native Rust or arbitrary shell execution by implication. Every executable surface must remain subject to C4OS trust and approval policy.

### P-006 — Treat arbitrary website content as unprivileged while allowing open web navigation

State: Accepted 2026-07-18; permission UX is a feature acceptance gate

Users may navigate to any ordinary `http` or `https` website they choose. “Untrusted” describes the page's technical authority, not a C4OS allowlist or a judgment about the website: all website JavaScript is unprivileged relative to the desktop application. The page must not receive Tauri internals, C4OS commands, credentials, filesystem authority, or a page-accessible native IPC handler.

Web capabilities such as downloads, camera/microphone, popups, external protocols, and user-selected files should remain generally available. When the browser engine exposes a permission request, C4OS may add a browser-like, per-origin security prompt and remembered choice only if it preserves expected browser behavior. Choosing `Default` must fall through to the platform/browser permission experience. C4OS must not blanket-deny these capabilities merely because a C4OS-specific permission UI is unavailable.

The current pinned Wry `0.55.1` exposes download and new-window handlers but predates its unified permission callback. Wry merged a cross-backend `with_permission_handler` for its planned `0.56.0`, including `Allow`, `Deny`, and `Default`. The Browser implementation must use a released/audited permission-capable Wry boundary and prove real prompt flows on each supported target. The normal Tauri proof exposed Tauri internals to page content; the revalidated raw-Wry/no-page-IPC architecture did not. Retain a separate-process option if another target cannot meet the same isolation boundary.

### P-007 — Simplify Advanced Policies while retaining the scenario corpus

State: Accepted

The 71 r012 authority identities remain an internal policy-scenario corpus rather than the permanent settings schema. The user-facing model uses four presets (`Ask for approval`, `Approve safe actions`, `Approve for me`, and `Custom`), seven broad policy groups, and concrete remembered exceptions. `Approve for me` remains bounded by sandbox, trusted-root, maximum-authority, and managed-policy constraints. Update the UI only in a future wireframe revision; preserve `r012-cleanup` unchanged.

### P-008 — Do not imply Codex compatibility unless C4OS intentionally adds an importer

State: Accepted 2026-07-18; Codex import excluded from current product scope

Codex appears in this research package because the requested research scope explicitly included Codex plugin, marketplace, and `config.toml` specifications. That request established a comparison topic, not a required C4OS feature. Unless C4OS intentionally adds “Import from Codex,” Codex remains research evidence only and no compatibility claim belongs in the product. If such an importer is later selected, it must be a named, versioned, one-way subset with per-field diagnostics; it must not imply exact Codex emulation or silently ingest raw secrets, managed policy, or executable hooks.

### P-009 — Treat marketplaces as plugin catalogs and keep individual installation C4OS-owned

State: Accepted 2026-07-18

Adding a marketplace registers a catalog source—typically a Git repository or local directory containing plugin listings. C4OS fetches and validates enough catalog metadata to list and search the plugins; adding the marketplace does not install or execute every plugin it lists. When the user chooses an individual plugin, C4OS owns that plugin's installation lifecycle: resolve its exact source/version, verify available integrity metadata, show capabilities before enablement, install disabled into a C4OS-owned cache, update transactionally, preserve the prior version on failure, uninstall, and honor revocation. A catalog may point to plugin packages stored elsewhere; it is not necessarily the package store itself.

### P-010 — Use a Rust-authoritative split plane

State: Accepted 2026-07-18; resolves GAP-003

The Tauri/Rust core should own authoritative workspace, session, turn, run, artifact, approval, credential-reference, and audit records. It should also own native windows, dialogs, secure-storage access, process supervision, updates, and the privileged tool gateway. The renderer submits user intent but receives no ambient authority.

`OCAdapter`, `PIAdapter`, managed runtime processes, MCP servers, and Local/Docker/SSH executors remain supervised workers. They may retain runtime-native state and request actions, but they may not decide C4OS policy, mint authorizations, read raw credentials, or mutate through an unbrokered privileged path. A remote executor may perform the exact authorized remote effect, while the local C4OS core remains the decision and audit authority.

This split does not require all computation to run inside Rust. It requires durable product state and security authority to remain in the Rust-owned C4OS core while sidecars stay replaceable and recoverable.

### P-011 — Bind runtime and environment on the first valid submission

State: Accepted 2026-07-18; resolves GAP-015

Runtime and execution-environment settings should be defaults for new chats. A provisional blank chat captures its runtime kind, adapter binding, environment identity, workspace mapping, and capability baseline when its first valid prompt or attachment is submitted. Later default changes affect only new chats.

An existing chat does not migrate implicitly. Runtime and adapter/native versions remain recorded per run so a compatible supervised update can be observed honestly. Moving a conversation to another runtime or environment should initially mean duplicating its user-visible context into a new chat; true native-session migration remains deferred until state compatibility is proven.

### P-012 — Activate configuration at explicit safety boundaries

State: Accepted 2026-07-18; resolves GAP-016

Configuration should use these activation boundaries:

- Display-only metadata updates immediately.
- Policy tightening, disablement, revocation, and credential removal take effect immediately, invalidate outstanding authorizations, and stop affected executable services when required.
- Skills, prompt resources, ordinary plugin enablement, MCP tool availability, and provider configuration are snapshotted at turn start and affect the next turn, never the middle of an active run.
- Executable plugin or MCP changes become available only after validation and successful process readiness; failure preserves the prior working state.
- Runtime/environment defaults affect only new chats under P-011.
- Adapter or native-runtime updates wait for active runs to finish or be cancelled, pass compatibility checks, and record the new version on the next run.

Every run retains the effective configuration and resource provenance needed to explain what was available when it executed.

### P-013 — Make Retry a new run attempt, never an implicit continuation

State: Accepted 2026-07-18; resolves GAP-017

Retry should create a new run attempt under the same immutable user turn. The failed or interrupted attempt, its output, completed tool effects, and audit records remain intact. The new attempt reuses the original user prompt and attachment snapshot, captures current approved configuration, and receives fresh single-use authorizations for every proposed side effect.

C4OS should not claim cross-runtime checkpoint continuation. If the prior attempt may have produced an effect whose completion is unknown, the UI must require review of that action before retrying. Automatic retry is permitted only before any side effect or when C4OS can prove the attempted operation was idempotent and did not complete.

### P-014 — Show compact provenance at the point of consequence

State: Accepted 2026-07-18; resolves GAP-018

The assistant identity remains C4OS, while provenance appears where it changes user understanding or authority:

- The chat header shows runtime, execution environment/host alias, workspace, and health; the existing model control shows the current model.
- Each turn retains provider/model, adapter/native version, environment, and effective capability/configuration provenance in expandable details.
- Approval surfaces always show the concrete action, target, workspace, environment/host, requesting runtime, and effective policy result.
- File, Terminal, and Browser artifacts show their source run and relevant path, shell, environment, or URL context in compact details.
- Errors identify the failing runtime/environment/version boundary and required recovery action.
- Audit and diagnostics retain the full redacted provenance and correlation chain.

This information should use compact chips or details rather than changing the response author from C4OS or crowding every message.

### P-015 — Resolve skills by trust, scope, and explicit source identity

State: Accepted 2026-07-18; resolves GAP-007

Every skill keeps a stable source-qualified identity rather than being overwritten by another package with the same `name`. Only valid, enabled, eligible skills enter suggestions or runtime context. Project/workspace skills are suppressed until that root is trusted; plugin skills require the providing plugin to be trusted and enabled.

When the same skill name exists at multiple eligible scopes, the default precedence is `project-local > workspace-local > user-global > plugin-provided > bundled`. An explicit user selection for the workspace overrides that default. C4OS must show the active source and collision state; fully qualified identities remain addressable, and a collision never deletes or mutates another source. Customizing an immutable bundled/plugin skill creates a user-owned copy and an explicit override rather than editing the source package.

Skill metadata is discovered eagerly, but full `SKILL.md` instructions and referenced resources load progressively only when the resolved skill activates. Invalid or shadowed records stay visible in Settings with repair or precedence diagnostics.

### P-016 — Store secrets behind opaque references and isolate runtime state by binding

State: Accepted 2026-07-18; resolves GAP-010

Only the Rust-authoritative core accesses secrets. C4OS stores an installation master key in the operating system credential service and stores encrypted secret material in a C4OS-owned vault; application databases, renderer state, logs, diagnostics, workspace files, runtime configuration, and exports contain opaque credential references only. If the OS credential service is unavailable, C4OS must not fall back to plaintext: it may offer an explicit password-protected vault or session-only credentials.

Credential records are scoped by installation/profile, provider or service, and optional workspace/plugin binding. Workers receive only the secret needed for the current operation through a short-lived channel, never command-line arguments or broad inherited environment. Remote execution uses local credential references and established SSH/OS agents where possible; copying a reusable raw secret to a remote host requires a separate explicit feature and approval boundary.

Runtime state is namespaced by runtime kind, native version, workspace binding, and process generation under C4OS application data—not inside the user's project unless the user exports an artifact there. Reset is available per credential, provider, runtime binding, workspace, or installation. Normal export excludes secrets; import requires reauthentication. Logs and crash reports redact both known secret values and credential-bearing fields.

### P-017 — Update the app, runtimes, and plugins through independent last-known-good channels

State: Accepted 2026-07-18; resolves GAP-012

C4OS application updates, OpenCode/Pi runtime updates, and plugin updates are separate versioned channels. No update may replace an active component during a run. Each channel downloads to staging, verifies its signature or pinned digest and compatibility metadata, waits for an activation boundary, performs health/compatibility checks, and retains a last-known-good version for rollback.

Application updates require the platform's distributable signature and the Tauri update signature. Before a data migration, C4OS creates a restorable database snapshot and does not accept new work until startup health succeeds. Runtime updates remain pinned to the C4OS adapter compatibility matrix and activate only after active runs finish or are cancelled. Executable plugin/capability changes stage disabled and require review; an unchanged trusted capability set may preserve enablement only after validation.

Offline operation continues on installed last-known-good versions without requiring an update check. Update failure preserves the current working version and data, reports the exact failed boundary, and never silently widens permissions. Air-gapped installation may use locally supplied, equivalently signed or digest-pinned packages.

### P-018 — Normalize model capabilities and snapshot the effective set per run

State: Accepted 2026-07-18; resolves GAP-019

C4OS should own a versioned model-capability descriptor rather than adopting OpenCode, Pi, or any provider schema as the product contract. `OCAdapter` and `PIAdapter` retain raw capability evidence and normalize the fields that change chat behavior: identity/revision, lifecycle, input/output modalities, context/output limits, reasoning modes and controls, tool calling, structured output, streaming, sampling controls, and caching/session requirements.

The descriptor distinguishes declared, adapter-normalized, observed, and effective capability layers. Material fields use `supported`, `unsupported`, `unknown`, or `degraded`, with source, time, constraints, and reasons. Evidence is scoped to the complete provider/endpoint/adapter/runtime route. The effective set is the narrow intersection of the selected model/provider endpoint, adapter, runtime, execution environment, installed resources, C4OS configuration, and policy; numeric limits use the narrowest confirmed value.

Adapter-declared dynamic session options remain separate from normalized model capabilities. A model or dependent-option change atomically replaces the complete recomputed session-control snapshot, then C4OS intersects that state with the effective descriptor and policy. Every run snapshots the exact provider/model/endpoint/revision and effective descriptor. C4OS preflights model switches and sends against the current draft, attachments, reasoning/output mode, context budget, and tool needs. It never silently drops an input, fabricates reasoning, treats prompted JSON as schema-guaranteed, exposes an unsupported control, or equates model tool calling with tool installation or authorization. Unknown capability disables only the dependent experience with an explanation, not ordinary compatible chat.

ACP may be used as an optional transport or future compatibility surface, but it does not replace the separate OCAdapter and PIAdapter contracts or C4OS authority. The detailed contract, open-source cross-check, and wireframe implications are in `model-capabilities.md` and `open-source-runtime-capability-patterns.md`.

## Open gaps

None. GAP-008 and GAP-013 remain explicitly deferred release decisions below; every other identified material research Gap is resolved by an accepted decision.

## Deferred release gaps

| ID | Deferred question | Trigger for resolution |
| --- | --- | --- |
| GAP-008 | Which public marketplace signing/origin authorities, moderation process, and remote compromise response does C4OS operate? | Before launching a public C4OS marketplace. User-added catalogs and local verified installation do not depend on this service decision. |
| GAP-013 | Which compatibility names, licenses, and trademarks may C4OS use publicly? | Before publishing compatibility marketing, a public plugin directory, or distributable bundles containing third-party components. |

## Resolved gaps

| ID | Resolution | Accepted decision |
| --- | --- | --- |
| GAP-003 | Orchestration uses a Rust-authoritative split plane; sidecars and remote executors remain supervised workers. | P-010 |
| GAP-001 | The pinned OCAdapter/PIAdapter mappings and live denial paths passed; their conformance suite remains a version-update gate. | P-002, P-003, P-004 |
| GAP-002 | A shared required baseline plus explicit per-runtime optional capability manifests defines parity without ranking either runtime. | P-002, P-003, P-004 |
| GAP-004 | Codex remains standards research; C4OS makes no Codex compatibility claim unless a future importer is intentionally scoped. | P-008 |
| GAP-005 | Marketplaces are generic plugin catalogs; adding one does not imply direct Codex marketplace ingestion or plugin installation. | P-008, P-009 |
| GAP-009 | Arbitrary websites remain unprivileged but broadly usable; permission mediation must preserve browser defaults and real prompt UX. | P-006 |
| GAP-006 | Declarative hooks are supported only after explicit trust; the macOS sandbox/revocation boundary passed and other targets remain feature-gated. | P-005 |
| GAP-007 | Skills retain source-qualified identities and resolve by trust, explicit selection, and deterministic scope precedence with visible collisions. | P-015 |
| GAP-011 | Local, Docker, and real OpenSSH transport semantics passed; each named external host is validated as a deployment feature gate. | P-010, P-011 |
| GAP-010 | Secrets use an OS-protected master key, encrypted C4OS vault, opaque references, scoped delivery, redacted output, and no plaintext fallback. Runtime state is namespaced by binding and generation. | P-016 |
| GAP-012 | App, runtime, and plugin updates use independent verified staging, activation boundaries, compatibility checks, last-known-good rollback, and offline continuity. | P-017 |
| GAP-014 | C4OS does not import Codex `config.toml` in the current product scope. | P-008 |
| GAP-015 | The first valid submission binds runtime/environment; later default changes affect new chats only. | P-011 |
| GAP-016 | Configuration uses immediate security boundaries, per-turn resource snapshots, readiness gates, and new-chat defaults. | P-012 |
| GAP-017 | Retry creates a new run under the same immutable turn and never silently continues or replays effects. | P-013 |
| GAP-018 | Compact provenance appears at the point of consequence while the assistant remains C4OS. | P-014 |
| GAP-019 | Model-dependent chat behavior uses a C4OS-owned, route-scoped, evidence-bearing descriptor; runtime session options remain separate and are atomically recomputed. | P-018 |
