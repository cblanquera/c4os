# Plugin Shell Research Notes

Status: research-notes
Created: 2026-06-30
Scope: numbered plugin-shell specs under `.agents/specs/01-*` through
`.agents/specs/11-*`.

## Purpose

Capture online research topics, general findings, and proof candidates for the
plugin-first C4OS shell refactor. These notes are research input only. They do
not freeze architecture, create implementation tasks, or override the accepted
MVP baseline.

## Sources Reviewed

- Tauri v2 plugin development: `https://v2.tauri.app/develop/plugins/`
- Tauri v2 permissions and capabilities: `https://v2.tauri.app/security/capabilities/`
- Tauri v2 JavaScript path API, including app config/data directory helpers:
  `https://v2.tauri.app/reference/javascript/api/namespacepath/`
- Tauri v2 menu API: `https://v2.tauri.app/reference/javascript/api/namespacemenu/`
- Codex plugin build guidance: `https://developers.openai.com/codex/plugins/build.md`
- Codex plugins overview: `https://developers.openai.com/codex/plugins.md`
- Codex skills guidance: `https://developers.openai.com/codex/skills.md`
- Codex config guidance: `https://developers.openai.com/codex/config-advanced.md`
- AI harness repositories:
  - `https://github.com/Onelevenvy/flock`
  - `https://github.com/openchamber/openchamber`
  - `https://github.com/thClaws/thClaws`
  - `https://github.com/nomifun/nomifun-tauri`

## Research Topics

- Tauri plugin shape: Rust crate/plugin builder, JavaScript guest API,
  command registration, generated permissions, and desktop/mobile setup split.
- Tauri permission model: capabilities as explicit windows/webviews plus
  permission sets, not a loose global backend surface.
- Native shell integration: menu/menu-item creation, plugin-owned menu entries,
  and OS-specific file reveal behavior.
- Cross-platform config paths: app config/data directory helpers and platform
  path differences for `~/.c4os`-style state.
- Modular app shell patterns: persistent host shell plus dynamically mounted
  panels, plugin metadata, dependency checks, and panel placement policies.
- Codex plugin structure: `.codex-plugin/plugin.json`, skills, MCP, apps,
  marketplaces, install/enable lifecycle, and restart expectations.
- Codex skills: `SKILL.md` progressive disclosure, `$` invocation, skill
  metadata, and distinction between reusable workflow and installable plugin.
- Codex config: user config layering, project trust boundaries, profile-style
  overrides, provider config, approval/sandbox settings, and structured tool
  telemetry.
- AI harness examples: how existing projects split chat, workspace context,
  tool/debug surfaces, browser/preview, config, and runtime execution.

## General Findings

- Tauri plugins are a useful implementation boundary only if C4OS also defines
  app-level plugin lifecycle, dependencies, settings, permissions, and panel
  contracts. Tauri alone does not define C4OS feature activation semantics.
- Tauri capabilities/permissions are a strong fit for plugin-owned backend
  command exposure. Each C4OS plugin should map its commands and webview access
  to explicit capabilities rather than exposing a broad app backend.
- C4OS should treat Codex plugin compatibility as a packaging and discovery
  model, not as a direct app-shell model. Codex plugins package skills, apps,
  MCP servers, and metadata; C4OS still needs its own `agents/c4os.yaml` to
  describe panels, app tools, native menus, and backend activation.
- Codex skills reinforce the need for progressive disclosure: C4OS should
  discover plugin and skill metadata before loading full instructions or
  activating runtime impact.
- Codex config guidance supports a useful `config.toml` direction: user-level
  defaults, trusted project boundaries, provider/tool policy, and environment
  controls should be explicit rather than hidden in UI-only state.
- The AI harness repositories should be used as implementation-pattern
  references, not copied wholesale. The useful comparison points are shell
  decomposition, chat/runtime boundary, tool invocation, panel state, config
  persistence, and debug observability.

## AI Harness Findings

### Flock

Repository: `https://github.com/Onelevenvy/flock`

Relevant patterns:

- Desktop harness built with Rust, Tauri, and React.
- Uses a modular crate split: core config/DB/encryption, agent executor,
  workflow compiler, tools registry, skills loader, and UI.
- Treats tools as a registry that can route local host tools, MCP servers, and
  sandboxed tools.
- Makes human approval and sandbox/VNC visibility explicit in the product
  story.
- Supports visual workflow composition and execution history.

C4OS implications:

- Useful reference for modular Rust crate boundaries and tool registry shape.
- Useful comparison for Chat Debug because it emphasizes visible tool execution
  and human-in-the-loop approvals.
- Avoid copying the visual workflow editor unless C4OS explicitly promotes
  workflow-building as a product surface.

### OpenChamber

Repository: `https://github.com/openchamber/openchamber`

Relevant patterns:

- Provides a rich GUI for OpenCode across desktop, browser/PWA, and VS Code.
- Emphasizes branchable chat timelines, tool UIs, permissions, task progress,
  plan/build mode, and context visibility.
- Desktop features include native windows, notifications, open-in-editor/file
  manager actions, remote instance switching, deep links, and SSH remote
  access.
- Advanced CLI/server behavior includes remote tunnels, connect links, and
  external OpenCode server connection.
- File, diff, terminal, git, and project action surfaces are treated as rich
  UI surfaces around the coding agent.

C4OS implications:

- Useful reference for keeping OpenCode-compatible sessions visible while
  giving tools richer UI than raw transcript text.
- Useful comparison for FS, IDE, Browser, Terminal, and Chat Debug plugin
  boundaries.
- Remote access, SSH, tunnels, and multi-window behavior should remain out of
  the immediate C4OS plugin-shell refactor unless promoted later.

### thClaws

Repository: `https://github.com/thClaws/thClaws`

Relevant patterns:

- Native Rust agent workspace across desktop GUI, CLI REPL, non-interactive
  mode, and webapp, all backed by one agent loop/session/tool registry.
- Supports skills, plugins, MCP servers, hooks, subagents, side-channel agents,
  plan mode, schedule, memory, KMS, and session resume.
- Uses settings as files, with project and user config precedence.
- Distinguishes GUI Files, Terminal, and Chat tabs while sharing one engine.
- Exposes shell escape, slash commands, provider/model switching, and local
  standards such as AGENTS.md and SKILL.md.

C4OS implications:

- Strong reference for one-engine-many-surfaces architecture and config
  precedence.
- Useful comparison for prompt tagging, `/` command behavior, skill/plugin/MCP
  boundaries, and Chat Debug.
- C4OS should not absorb all thClaws surfaces at once; use it to validate
  boundaries, not to widen scope.

### NomiFun

Repository: `https://github.com/nomifun/nomifun-tauri`

Relevant patterns:

- Local-first Tauri 2 app with React frontend and Rust backend.
- Uses desktop and web host modes with a shared backend and UI.
- Emphasizes an open capability bus exposed through MCP and REST.
- Has native computer/browser use, knowledge, terminal, companion, requirements
  automation, skills, MCP, models, and settings hubs.
- Documents platform-specific packaging and signing commands.
- Uses many Rust crates for agent, backend, shared networking, redaction,
  browser/computer use, knowledge, sessions, terminal, companion, gateway, and
  more.

C4OS implications:

- Useful reference for capability-bus thinking and plugin-provided app tools.
- Useful comparison for local-first config, native Browser/Computer authority,
  MCP/REST exposure, and packaging.
- C4OS should validate whether a capability bus belongs inside the plugin
  architecture now or remains a later runtime/tool policy detail.

## Cross-Spec Proof Themes

- Plugin activation proof: can a plugin add frontend panel state, backend
  commands, settings entries, and native menu items without changing the core
  shell contract?
- Permission proof: can plugin commands be exposed through Tauri capabilities
  with least privilege per plugin/panel?
- Config proof: can C4OS resolve and persist user config consistently across
  macOS, Windows, and Linux fallback paths?
- Shell layout proof: can left and right plugin panels resize while preserving
  a fixed minimum center chat width?
- Runtime/tool proof: can app tools from plugins be registered, displayed,
  approved/denied, and called through the existing gateway without restoring
  prompt-text command parsing?
- Harness comparison proof: can the selected AI harness examples identify
  concrete patterns worth adopting or avoiding before implementation?

## Source-Specific Findings To Validate Later

- Tauri plugin permissions should be generated and inspected in a POC before
  adopting plugin-owned backend commands.
- Native menu ownership should be tested with more than one plugin so menu item
  collisions and enable/disable behavior are understood.
- App config path behavior should be tested through Tauri APIs and plain Rust
  backend code because C4OS may need config access before frontend APIs load.
- Codex plugin marketplace/install concepts are useful for package discovery,
  but C4OS needs a narrower first pass for bundled/local plugins before
  external marketplaces.
- AI harness repositories may use different stacks and assumptions; each should
  be summarized into reusable patterns only after inspecting its actual shell,
  config, tool, and runtime boundaries.

## Recommended Research Artifacts Before Implementation

- A Tauri plugin POC under `proofs/tauri-plugin-activation/`.
- A shell-panel layout POC under `proofs/plugin-shell-layout/`.
- A config path POC under `proofs/c4os-user-config-paths/`.
- A Pi runtime proof under `proofs/pi-runtime-app-layer/`.
- A prompt attachment/tagging POC under `proofs/prompt-context-routing/`.
- A Browser capture/annotation/doc-preview POC under
  `proofs/browser-plugin-capture-preview/`.
- A harness comparison note under `.agents/references/research/` summarizing
  the four referenced repositories against C4OS needs.

## Harness Comparison Questions

- Which Flock crate boundaries map cleanly to C4OS plugin/core boundaries?
- Which OpenChamber tool UI patterns should C4OS emulate for tool calls,
  permissions, diffs, files, and task progress?
- Which thClaws config and extension standards are compatible with the C4OS
  plugin-shell direction?
- Which NomiFun capability-bus ideas fit C4OS app tools without over-expanding
  the immediate refactor?
- Which referenced project demonstrates the safest model for browser/computer
  use without leaking app privileges?
