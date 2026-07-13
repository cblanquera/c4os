# C4OS Architecture And Interface Contracts

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
