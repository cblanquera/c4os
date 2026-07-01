# Plugin Shell Research Notes - Pass 2

Status: research-notes
Created: 2026-07-01
Scope: Follow-up research for the proposed numbered post-MVP C4OS specs under `.agents/specs/`.

## Purpose

This pass expands the first plugin-shell research pass with topics that should
shape proof planning before implementation:

- Tauri process lifecycle, sidecars, permissions, windows, packaging, and paths.
- Modular application-shell patterns from IDE/workbench and plugin UI systems.
- Codex/OpenAI plugin, app, skill, and configuration surfaces.
- Additional AI harness implementations beyond the first pass.
- Open questions that were answered, partially answered, or still need user
  decisions.

This document is research input only. It does not activate implementation work.

## Sources Reviewed

- Tauri v2 sidecar guide:
  `https://v2.tauri.app/develop/sidecar/`
- Tauri v2 plugin guide:
  `https://v2.tauri.app/develop/plugins/`
- Tauri v2 capability and permission model:
  `https://v2.tauri.app/security/capabilities/`
- Tauri v2 path API:
  `https://v2.tauri.app/reference/javascript/api/namespacepath/`
- Tauri v2 menu API:
  `https://v2.tauri.app/learn/window-menu/`
- Tauri updater and signing documentation:
  `https://v2.tauri.app/plugin/updater/`
- VS Code Webview API:
  `https://code.visualstudio.com/api/extension-guides/webview`
- OpenAI Apps SDK:
  `https://developers.openai.com/apps-sdk/`
- Goose:
  `https://github.com/aaif-goose/goose`
- OpenHands / Agent Canvas:
  `https://github.com/OpenHands/OpenHands`
- Cline:
  `https://github.com/cline/cline`
- Continue:
  `https://github.com/continuedev/continue`
- Aider:
  `https://github.com/Aider-AI/aider`
- Open Interpreter:
  `https://github.com/OpenInterpreter/open-interpreter`

## Technical Findings

### Tauri Plugin And Permission Boundary

Tauri plugins are a credible backend activation unit for C4OS, but the proof
needs to distinguish three layers:

- A Rust/Tauri command layer that exposes backend behavior.
- A permission/capability layer that controls which commands are available to
  which webview or window.
- A C4OS plugin registry layer that decides which plugin UI and shell panel is
  mounted.

The first implementation risk is conflating Tauri plugin installation with
C4OS plugin activation. Tauri can define backend capabilities, but C4OS still
needs its own runtime registry, settings model, panel placement, and icon order.

Proof implication:

- `proofs/tauri-plugin-activation/` should prove backend command registration,
  frontend discovery, enable/disable behavior, and capability enforcement.

### Sidecars And Runtime Process Lifecycle

Tauri sidecars are relevant for Pi runtime, terminal-owned processes, and any
future packaged agent or tool runtime. They should not be assumed as the only
process-management model, because some app tools may remain native Rust
commands or frontend-mediated calls.

Proof implication:

- Add or expand a proof around `proofs/tauri-sidecar-runtime-lifecycle/` to
  test spawning, cancellation, stdout/stderr streaming, app shutdown cleanup,
  and per-session ownership.
- Keep this proof separate from the plugin registry proof so process lifecycle
  does not block shell layout decisions.

### Paths And Cross-Platform Config

Tauri path APIs and native Rust path helpers can support the desired
`~/.c4os`-style application config location, but the product decision remains
open: whether C4OS should use a literal home-level `.c4os` directory on every
platform, platform-native app config directories, or a hybrid strategy.

Proof implication:

- `proofs/c4os-user-config-paths/` should compare macOS, Windows, and Linux
  behavior for app config, project registry, workspace registry, and plugin
  configuration.
- The proof should explicitly answer how portable backups and manual user
  inspection work.

### Menus And OS Integration

Tauri can support OS menus, which is relevant to File System plugin actions
such as `Create New Workspace`. This should remain a plugin contribution to the
app menu, not a hard-coded shell menu, if the plugin model is the long-term
direction.

Proof implication:

- `proofs/plugin-menu-contribution/` should test plugin-owned menu items,
  command dispatch, disabled states, and plugin disable/uninstall behavior.

### Updater, Signing, And Plugin Distribution

Updater and signing research matters, but it is not a blocker for the first
plugin-shell refactor unless C4OS will distribute third-party plugin bundles
outside the core app. For first-party bundled plugins, this can remain a later
packaging/security topic.

Proof implication:

- Treat `proofs/plugin-updater-signing-threat-model/` as a later research proof
  unless the user wants third-party plugin installation in the first wave.

### Modular Shell And Workbench UI

VS Code webviews reinforce a useful boundary: extension/plugin UI should be
isolated enough to prevent one plugin panel from owning the whole shell, while
still allowing structured communication with the host. C4OS does not need to
copy VS Code's UI, but the workbench model is a strong reference for:

- A persistent center editor/chat surface.
- Plugin-contributed side panels.
- Plugin-contributed commands and menus.
- Host-owned layout persistence.
- A narrow message boundary between host shell and plugin UI.

Proof implication:

- `proofs/plugin-shell-layout/` should include isolated plugin panel mounting,
  left/right placement, icon order, resize constraints, and active-panel toggle
  behavior.
- `proofs/plugin-ui-isolation-webview/` should test whether C4OS wants iframe,
  webview, component, or same-bundle React mounting for plugin panels.

### OpenAI Apps SDK And MCP UI

The OpenAI Apps SDK is useful as a reference for tool-owned UI surfaces,
structured tool metadata, and component/resource descriptors. It should not be
treated as a direct replacement for the native C4OS plugin model. The key C4OS
lesson is that plugin UI, tool descriptors, and invocation permissions should
be explicit and inspectable.

Proof implication:

- `proofs/plugin-manifest-contract/` should compare Codex plugin metadata,
  C4OS `agents/c4os.yaml`, MCP-style tool metadata, and runtime settings.
- The proof should decide what C4OS reads from plugin roots and what it ignores.

### Document Preview Safety

`.docx` and `.xlsx` preview support in the Browser plugin is likely a document
conversion and sandboxing problem, not only a browser navigation problem. The
preview should avoid granting arbitrary filesystem access to rendered document
content.

Proof implication:

- `proofs/document-preview-sandbox/` should test supported file types,
  conversion library choice, temp-file lifecycle, trusted-root checks, and
  rendering isolation.

## Additional Harness Findings

### Goose

Goose is relevant because it combines a desktop app, CLI, API, provider
abstraction, MCP extension model, and custom distributions. Its strongest
lesson for C4OS is the separation between host app, extension/tool ecosystem,
and execution providers.

Useful research angles:

- Extension discovery and configuration.
- Desktop/CLI/API surfaces sharing one agent core.
- Provider abstraction without leaking provider details into the shell.
- Custom distributions as a possible analogy for bundled C4OS plugin sets.

### OpenHands / Agent Canvas

OpenHands has shifted toward an agent control center model that can run
multiple coding agents across local, remote, cloud, and sandboxed backends.
This is relevant to C4OS runtime planning, especially if C4OS may eventually
host more than one agent harness.

Useful research angles:

- Backend switching model.
- Agent session ownership.
- Sandbox warnings and local direct mode.
- Automation/server split.
- How much runtime complexity belongs in C4OS versus external harnesses.

### Cline

Cline is highly relevant to C4OS prompt and task flow. It has visible patterns
for plan/act mode, checkpoints, file diffs, bash approval, MCP/plugin
extension, slash-command-like workflows, skills/rules, and multi-agent
interfaces.

Useful research angles:

- Human approval before file edits or shell commands.
- Prompt context assembly from files, rules, skills, and tools.
- Checkpoint and rollback UX.
- IDE-integrated file diffs and terminal workflows.
- Whether C4OS prompt tagging should mirror, adapt, or avoid these patterns.

### Continue

Continue is less useful as a forward-looking harness because its repository
currently presents itself as no longer actively maintained after its final
release. It is still useful as a historical reference for configuration,
provider abstraction, IDE integration, and context providers.

Useful research angles:

- Provider and model configuration shape.
- Context provider taxonomy.
- IDE extension architecture.

Research caution:

- Do not anchor new C4OS architecture decisions on Continue without validating
  whether a maintained successor or fork is the better reference.

### Aider

Aider is useful as a focused terminal-first editing harness. Its strongest
patterns are repository mapping, git integration, test/lint loops, web/image
context, and auto-commit workflows.

Useful research angles:

- Repo map strategy for file context.
- Git branch and commit workflow.
- Test/lint feedback loops.
- Terminal-first UX that remains understandable without a full IDE.
- Handling images and web pages as prompt context.

### Open Interpreter

Open Interpreter is especially relevant because its newer Rust direction
focuses on harness emulation, permissions, skills, MCP, AGENTS.md support,
provider switching, and local state. It is a strong reference for separating
agent harness mode from app shell UI.

Useful research angles:

- Harness emulation modes.
- Permission and approval prompts.
- Skill and hook loading.
- Local state location.
- ACP/MCP interop.
- Computer-use/browser QA integration.

### OpenClaw

OpenClaw was suggested as a candidate harness, but a direct raw README lookup
did not validate a stable source during this pass. Treat it as deferred until
the canonical repository and current project status are confirmed.

## Open Question Updates

### Answered Or Mostly Answered

- Should Tauri sidecars be researched?
  Yes. They are directly relevant to runtime, terminal, and packaged agent
  process ownership, but should remain a proof separate from shell layout.

- Should plugin UI isolation be researched?
  Yes. Workbench/webview models show that plugin UI boundaries are an
  architectural decision, not only a styling decision.

- Are there more relevant harnesses than the first-pass list?
  Yes. Goose, OpenHands, Cline, Aider, and Open Interpreter should be added to
  the harness research set. Continue is useful only with maintenance caveats.

- Is `config.toml` worth treating as a first-class design surface?
  Yes. Codex, Goose, Continue, Cline, Aider, and Open Interpreter all reinforce
  that provider/tool/runtime configuration becomes a durable user contract.

### Partially Answered

- Where should C4OS user config live?
  Research confirms available path primitives, but the product choice remains:
  literal `~/.c4os`, platform-native app config, or a hybrid.

- What should `agents/c4os.yaml` contain?
  Research supports a plugin-owned application descriptor, but exact fields
  still need design: panel contribution, commands, menu items, settings,
  permissions, required plugins, and C4OS-specific assets.

- Should C4OS support third-party plugin installation in the first refactor?
  Research shows this would require packaging, signing, trust, and update
  policy decisions. It should be deferred unless the user explicitly wants it
  in the first plugin wave.

- Should C4OS integrate with MCP Apps/UI?
  Research suggests using MCP/App patterns as references for descriptors and
  tool UI, not as a replacement for the native plugin shell.

### Still Open

- Should plugin panels be mounted as React components, iframe/webview surfaces,
  or separate windows/webviews?
- What is the fixed minimum center pane width across desktop breakpoints?
- Does each chat session own exactly one browser, one terminal, and one debug
  stream, or can plugins expose multiple instances per session?
- Should Chat Debug be visible to normal users or hidden behind developer mode?
- Which app tools require default approval policies, and which can be silent by
  default?
- Should the File System plugin be required for all project-aware behavior, or
  can some project metadata remain in the core shell?
- What is the exact branch UX when a project has multiple worktrees or nested
  git repositories?
- Which document-preview formats are required for the first Browser plugin
  pass beyond `.docx` and `.xlsx`?

## Updated Proof Recommendations

The existing proof list should be expanded or refined with these proof records:

- `proofs/tauri-sidecar-runtime-lifecycle/`
  Validate spawning, streaming, cancellation, shutdown cleanup, and per-session
  ownership for sidecar-managed processes.

- `proofs/plugin-menu-contribution/`
  Validate plugin-owned OS menu items such as `Create New Workspace`.

- `proofs/plugin-ui-isolation-webview/`
  Compare same-bundle React mounting, iframe isolation, Tauri webviews, and
  separate windows for plugin panels.

- `proofs/plugin-manifest-contract/`
  Define and validate the relationship between Codex plugin metadata,
  `agents/c4os.yaml`, `c4os/` plugin code, settings UI, and runtime loading.

- `proofs/document-preview-sandbox/`
  Validate `.docx` and `.xlsx` preview conversion, containment, temp-file
  cleanup, and trusted-root enforcement.

- `proofs/agent-harness-command-routing-comparison/`
  Compare slash commands, prompt tags, file context, approvals, and tool routing
  across Cline, Aider, Open Interpreter, Goose, and OpenHands.

- `proofs/acp-mcp-agent-interop/`
  Determine whether ACP and MCP should influence C4OS runtime/plugin
  interfaces now, later, or not at all.

- `proofs/plugin-updater-signing-threat-model/`
  Research-only proof for third-party plugin distribution, signing, and update
  trust. This can be deferred for first-party bundled plugins.

## Spec Impact Notes

- `01-shell-plugin-architecture-refactor`
  Add explicit separation between Tauri plugin activation, C4OS plugin
  registration, UI panel mounting, and app tool permissions.

- `02-core-app-shell-ux`
  Keep the persistent chat shell host-owned. Treat plugin UI as contributed
  side panels with host-owned layout constraints.

- `03-plugin-system-settings-management`
  Promote `agents/c4os.yaml`, user config paths, plugin settings, menu
  contribution, and permissions into first-class design topics.

- `04-runtime-tool-policy`
  Add sidecar lifecycle, harness-mode comparison, ACP/MCP interop, and app-tool
  approval policy proofs.

- `05-chat-prompt-interactions`
  Use Cline, Aider, and Open Interpreter as strong references for slash
  commands, file tagging, approval prompts, and context assembly.

- `06-file-system-plugin`
  Keep workspace/project registry ownership with the FS plugin, but resolve
  whether storage is literal `~/.c4os` or platform-native config.

- `07-file-editor-plugin`
  Use Aider/Cline research for file context, diffs, right-click actions, and
  add-to-chat behavior.

- `08-terminal-plugin`
  Tie terminal session ownership to the sidecar/runtime lifecycle proof before
  task implementation.

- `09-chat-debug-plugin`
  Decide whether debug visibility and tool-call parameter rendering are normal
  user features or developer-mode features.

- `10-browser-plugin`
  Add document-preview sandboxing and screenshot/annotation attachment proofs.

- `11-skills-settings`
  Use Codex skills plus Cline/Open Interpreter rules/skills as references, but
  keep C4OS skill creator behavior scoped to Settings.
