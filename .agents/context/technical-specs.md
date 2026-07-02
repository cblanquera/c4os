# Technical Specs

Status: active
Created: 2026-06-21
Updated: 2026-07-02
Source Note: Normalized from product-model, runtime-adapter, and constraints context. Detailed technical records are preserved under `.agents/references/context/technical-specs/`.

## Purpose

Use this as the technical gate. It summarizes product-owned system concepts, runtime boundaries, persistence, security, Browser constraints, Terminal constraints, and implementation locations before deeper technical references or specs are loaded.

## Load When

- You need architecture, product model, runtime adapter, persistence, secret handling, or implementation-location rules.
- You are working on Browser isolation, Terminal ownership, approval policy, provider credentials, extensions, or trusted-root behavior.
- You need to decide whether to open detailed technical references or POC evidence.

## Skip When

- You only need product thesis, users, goals, or vocabulary; load `product-brief.md`.
- You only need user-facing feature behavior or MVP feature inventory; load `product-specs.md`.
- You only need interface layout, interaction, visual, or accessibility guidance; load `creative-specs.md`.
- You only need sequencing, validation work, accepted decisions, or deferred work; load `work-orders.md`.

## Owns

- Product model, runtime adapter boundary, persistence, credential handling, approval/security constraints, Browser technical boundary, Terminal technical boundary, and implementation-location guardrails.

## Does Not Own

- Customer-facing feature prose, visual layout contract, accepted work-order status, active progress, or detailed spec acceptance.

## Reference Routing

- `.agents/references/context/technical-specs/product-model.md`
  Purpose: Detailed entity model, workspace descriptors, sessions, runs, and secrets ownership.
  Load when: checking data model and ownership questions.
  Skip when: the compact technical context already answers the boundary.

- `.agents/references/context/technical-specs/runtime-adapter.md`
  Purpose: OpenCode/Pi proof findings, adapter boundary, and runtime implications.
  Load when: checking runtime selection or adapter work.
  Skip when: the task does not affect runtime adapter behavior.

- `.agents/references/context/technical-specs/constraints.md`
  Purpose: Trust, credential, Browser, Terminal, extension, and validation constraints.
  Load when: reviewing safety-sensitive implementation, proof, or approval behavior.
  Skip when: the task is unrelated to trust or execution boundaries.

- `.agents/references/context/technical-specs/extension-loading.md`
  Purpose: Skill, plugin, and MCP discovery/loading contract.
  Load when: implementing or reviewing extension install, discovery, loading, enablement, or invocation.
  Skip when: the task is unrelated to extension behavior.

- `.agents/references/context/source-provenance.md`
  Purpose: Source and artifact provenance for technical facts.
  Load when: tracing where technical context came from.
  Skip when: current context already supplies the accepted fact.

- `.agents/references/research/final-implementation-source-inventory.md`
  Purpose: Final-implementation replay source inventory and research evidence routing.
  Load when: auditing imported goals, grill answers, and research evidence.
  Skip when: working from accepted context/spec records only.

## Summary

C4OS owns the product model and safety boundary even when it delegates execution to OpenCode, Pi, MCP servers, shells, Browser engines, or other runtime components. Runtime concepts should stay behind app-owned adapters unless exposed for advanced compatibility inspection.

## Backend App Architect Profile

Final implementation specs should resolve uncertain backend/product questions
using this profile:

- Prefer popular AI ecosystem conventions before inventing local shapes. Check
  OpenAI/Codex conventions first, then Claude/Anthropic conventions, then
  broader open standards such as MCP when the local record is uncertain.
- Model Tauri tools like MCP tools structurally: stable IDs, human-readable
  titles/descriptions, JSON-schema-like input and output contracts, capability
  metadata, approval policy, structured results, and lifecycle events.
- Keep C4OS as the host/gateway. Runtime providers and plugins may request or
  contribute tools, but C4OS owns execution, approval, persistence, trusted-root
  enforcement, audit records, and app-owned per-chat result state.
- Use event-driven communication between backend tools, the shell, and app
  plugins. UI surfaces hydrate from typed C4OS state and events; they must not
  become the authority for security, persistence, tool execution, or prompt
  parsing.
- Keep the shell thin: chat/session identity, settings, navigation, approval,
  attachment, and plugin mounting are shell primitives. Rich work surfaces
  belong to plugins.
- Design for workers as well as coders. Product language, empty states, repair
  flows, settings, and plugin surfaces should support operations, support,
  research, admin, and normal knowledge work, not only Git/code workflows.
- Treat C4OS as a reusable generic AI app harness. Chat sessions, provider
  adapters, tools, attachments, approvals, plugin views, debug/audit, and
  runtime adapters should be reusable for future domain-specific AI apps.
- Keep memory and RAM cost explicit. Heavy services start lazily, are shared at
  the narrowest safe app/user/workspace/project scope, expose health/activity
  state, and shut down on idle, disable/uninstall, close, or app exit.
- Preserve Windows compatibility even before Windows QA exists. Filesystem
  paths, trash/recycle, config directories, PTY/ConPTY, file watchers, shell
  environment, browser/webview behavior, and secret storage must sit behind
  platform adapters with documented fallback behavior.
- Generate maintainable code: small modules, explicit state machines for
  lifecycles, narrow service responsibilities, typed contracts, clear naming,
  JSDoc or Rust doc comments for non-obvious behavior, and readable error
  paths instead of clever ad hoc logic.

## Product Model

Minimum product-owned entities include:

- Workspace
- Workspace file
- Project folder
- Trust record
- Session
- Agent run
- Transcript message
- Runtime event
- Approval request and decision
- Saved approval rule
- Artifact
- Provider profile and model profile
- Credential reference
- Local memory record
- Extension inventory item
- Skill, MCP server, and plugin references
- Browser state record
- Terminal session record
- Audit log entry

## Workspace And Persistence

C4OS stores project/workspace registry state and chats in user-level app state,
not in hidden project-root `.c4os` folders. User-global config uses the
platform-standard per-user app config directory. Project identity is the
canonical project folder path; when a project path is missing, the UI keeps the
project visible as missing, makes its chats read-only, and allows relocation.
Relocation migrates chats to the new canonical path.

Workspace files remain explicit load/save artifacts such as
`workspace.c4os.json`. They contain the workspace name and project folder
references only. Workspace files are groupings of project folders, not chat
owners. The same project folder appearing in multiple workspace files shares
the same user-level project chats.

Workspace files, config files, and user-level registries must not contain raw
secrets. Full export/import remains outside current accepted scope unless a
separate spec explicitly promotes it.

## Runtime Boundary

OpenCode is the accepted implementation target behind a thin C4OS-owned runtime adapter. Pi requires a proof before runtime adapter scope is accepted. C4OS owns user-facing session identity, workspace persistence, artifact identity, approval policy, provider settings, local memory, runtime lifecycle supervision, runtime recovery, and error reporting.

OpenCode proof findings are preserved in `.agents/references/context/technical-specs/runtime-adapter.md`. Credentialed model-backed prompt execution and live permission-request capture still need validation because the proof avoided provider credentials and token spend.

Runtime-driven tool execution should use a formal C4OS tool gateway instead of
frontend natural-language command parsing. The target flow is: send the prompt
to the runtime, stream runtime events in real time, let the runtime request
tools through a generic tool-call contract, execute approved tools through
C4OS-owned authority boundaries, stream tool output back to the runtime and UI,
then persist the final response and run records. This is MCP-shaped, but C4OS
must remain the host/gateway that owns execution, approval, persistence, and
trusted-root enforcement.

Per-session tool configuration should map tool identities such as
`terminal.run`, `files.read`, `files.write`, `browser.open`, and
`artifact.preview` to enabled state, access level, and approval policy. Runtime
events should use lifecycle names such as `tool_call_requested`,
`tool_call_started`, `tool_output_delta`, `tool_call_completed`, and
`final_response`; event names are not tool identities. Tool implementations may
define default and maximum approval levels, and session config may narrow but
must not silently widen tool authority.

## Final Implementation Plugin And Tool Model

C4OS app plugins are distinct from Tauri plugins. Built-in C4OS app plugins are
installed with the default app, disabled by default, and modeled through
Codex-compatible marketplace semantics. Installed plugin bundles live in a
C4OS-owned cache; reinstall uses the marketplace source.

C4OS exposes an app-owned tool gateway. Runtime discovery and tool invocation
can occur without an enabled plugin view, subject to per-tool approval policy.
App plugins declare tool contributions, consumers, settings, dependencies, and
tool views in `agents/c4os.yaml`; plugin `c4os/` code implements bindings to
preinstalled C4OS/Tauri native modules. C4OS owns default and maximum approval
policy; plugins may request narrower defaults.

One tool call becomes one C4OS tool event. The event fans out to enabled
compatible plugin views, including hidden enabled views, without duplicating the
backend invocation. Plugin state is per chat session with one instance per
plugin per chat session.

User-global `config.toml` is both user-editable and UI-editable. It owns
runtime, provider/model defaults, marketplaces, plugin enablement, and app-tool
policy. Settings writes to it; parse errors keep the last valid config.

User-directed reads/previews are allowed across the filesystem. Agent-initiated
outside-project reads ask unless the user explicitly requested that
file/location. Trusted-project writes are allowed when the current request
implies project file work; destructive deletes, broad rewrites, and
outside-project writes ask unless explicitly requested. Terminal commands ask by
default with remembered safe-command rules; trusted-project git/worktree actions
are allowed; network mutation and credential use ask.

## Cross-Spec Interface Contracts

These contracts are shared context, not owned by any one pending spec:

- Plugin manifests: `.codex-plugin/plugin.json` owns Codex package metadata;
  `agents/c4os.yaml` owns C4OS app-shell metadata, settings schema,
  dependencies, tool declarations, and tool-view declarations.
- Tool gateway: runtime and plugin tool calls flow through the C4OS-owned
  gateway. One backend invocation emits one tool event that fans out to enabled
  compatible plugin views, including hidden enabled views.
- Plugin settings: Settings renders plugin-declared fields and persists values
  in user-level config. Shell-reserved fields include `enabled`, `panel`, and
  `iconOrder`.
- Prompt attachments: Browser, IDE, and other plugins contribute C4OS
  attachment/reference records. The prompt/model layer owns adapter translation,
  warning, and safe degradation for provider-specific capabilities.
- FS identity: plugins consume canonical project paths, workspace files, and
  user-level registries from the FS/workspace model. No plugin should create
  hidden project-root C4OS identity folders.
- Terminal/debug split: Terminal owns user PTY panel state. Runtime terminal
  tools remain gateway-owned and surface in thread context plus Chat Debug.
- Document preview: document-family plugins own document renderers; Browser may
  host rendered output when a compatible plugin provides it.

## Security And Trust

- Project-local operations require explicit trusted-root containment.
- High-impact actions require app-owned approval policy before runtime execution.
- High-impact categories include shell commands, file writes/deletes, MCP mutation, credential use, networked tools, worktree mutation, share/export, Browser agent actions, extension runtime impact, and destructive actions.
- Raw API keys must be stored only in OS keychain or an equivalent secure storage abstraction.
- Secrets are secure references, not raw values in workspace files, general app tables, transcripts, or normal app data.
- Generated or untrusted HTML must render without provider credentials, arbitrary workspace file access, shell state, or privileged app APIs.
- Extensions are disabled by default before they affect runtime execution, model context, tools, or app-owned state.
- Skill, plugin, and MCP discovery/loading should read metadata first and defer
  full instruction loading, hook execution, MCP launch, and runtime access until
  explicit enablement through app-owned records.

## Browser Constraints

Browser is a user-owned desktop surface with project-scoped profile. Agent Browser use is request-scoped, and agent file/Browser actions must be recorded.

The implementation must not ignore the failed direct Tauri `WebviewWindow` proof: page content observed `window.__TAURI_INTERNALS__` in that POC. Local-file browsing, logged-in use, privileged bridge exposure, and agent authority require explicit handling before Browser claims are trusted.

## Terminal Constraints

Terminal sessions are backend-owned; renderer code must not spawn arbitrary shells directly. Terminal implementation requires trusted-root cwd validation, deterministic command allowlist, approval policy, sanitized environment, backend-owned lifecycle, bounded renderer event transport, backpressure handling, audit persistence, and cross-platform PTY/ConPTY confirmation.

The TASK-011A/TASK-011B explicit prompt command bridge is transitional polish.
It must not become the long-term command-planning mechanism. Runtime command
selection should come from runtime tool-call requests and flow through the
C4OS tool gateway.

## Implementation Locations

Production implementation belongs in `backend/`, `frontend/`, and `tests/server/`. Do not create or use `src-tauri/`. POC implementation artifacts belong under `proofs/<proof-name>/`.
