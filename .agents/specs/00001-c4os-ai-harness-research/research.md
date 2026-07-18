# Initial Research

Initial research date: 2026-07-17
Codex `config.toml` addendum: 2026-07-18

## Model-capability addendum

Current provider and runtime specifications confirm that model-dependent input/output modalities, reasoning controls, tool calling, structured output, context limits, streaming, sampling controls, and caching can materially change a chat session. These fields do not share one universal schema, and they must not be conflated with runtime capabilities, installed tools, or C4OS authorization. The detailed comparison, proposed normalized descriptor, effective-capability rules, wireframe implications, and proof mapping are in [Model capabilities and chat experience](model-capabilities.md).

A second open-source comparison separates runtime-host patterns from model-registry patterns. ACP, Goose, and OpenHands inform agent negotiation, dynamic session controls, and the runtime-versus-execution boundary; Cline, Cherry Studio, and Continue inform path-specific capability metadata and overrides. The findings and P-018 refinements are in [Open-source runtime and model-capability patterns](open-source-runtime-capability-patterns.md).

## Method

The review used the r012 specification, source structure, QA notes, and representative QA images. External findings prefer official documentation and current repository source over search summaries or stale prose. GitHub relationships and absent integrations are described narrowly.

## Product functionality observed in r012

The product is a desktop AI workspace rather than only a chat client.

### Workspace and chat

- First-run provider onboarding, followed by opening a folder, workspace, clone target, or recent workspace.
- Projects with nested, searchable chats; project add, rename, reveal, copy, remove, relink, and reorder behaviors.
- A new chat remains unsaved until the first valid prompt or attachment, then derives its title from that input.
- Chat, Files, Browser, and Terminal composer modes, with mode-specific controls for model, approval, branch, and execution behavior.
- Response artifacts for files, folders, browser state, and terminal commands that can expand into the main surface while retaining chat context.
- Attachments, drag-and-drop, copying, reply actions, streaming responses, and prompt tags.

### Native work surfaces

- A per-chat terminal concept with inline command snapshots, expanded PTY interaction, input, resize, cancellation, and cleanup.
- File/editor and browser surfaces that can be focused from model-produced artifacts.
- Local Desktop, Docker, and Remote SSH execution-environment concepts.
- A separate desktop Settings window concept.

### Settings and extension surfaces

- Providers and model inventory, including OpenAI-compatible endpoints.
- Runtime selection and fallback configuration.
- Installed/catalog plugin views, marketplace sources, GitHub/ref installation, plugin details, legal metadata, enablement, and uninstall.
- Skills discovery, metadata, source-file display, scope, validity, customization, and enablement.
- MCP server configuration and lower-level configuration editing.

These are interaction requirements. The wireframe explicitly does not prove real provider calls, runtime processes, network access, filesystem mutation, credential storage, PTY behavior, persistence, or native multi-window behavior.

## Tauri

### Current specification findings

- A Tauri application combines a Rust core process with HTML rendered in the operating system's webview. The frontend and Rust core communicate through message passing.
- The Rust core is inside the privileged trust boundary. Frontend webviews should receive only narrowly granted commands and resources.
- Capabilities assign permissions to named windows and webviews. Multiple matching capabilities merge, so capability composition must be reviewed as a union.
- Tauri can bundle and launch sidecar executables through `externalBin`. Platform-specific binaries use target-triple filenames.
- Shell execution is blocked by default and must be enabled with scoped program and argument permissions. Spawn, execute, kill, open, and stdin have distinct permission surfaces.
- Production application updates are signed. The updater uses a public key in the application and requires the private key to remain protected.

### C4OS implications

- Tauri is a suitable native host for windows, menus, dialogs, keychain integration, filesystem mediation, updater behavior, and managed runtime processes.
- Tauri should not itself become the semantic runtime API. A runtime adapter must normalize OpenCode, Pi, and future runtimes.
- Sidecars and shell commands need separate least-privilege scopes. Installing a runtime and running a runtime are different permissions.
- A webview that displays arbitrary remote content must not share the privileged application bridge.
- Each window should have a small, explicit capability file. Settings, main workspace, terminal, and any browser window should not inherit the same authority by convenience.

Sources: [Architecture](https://v2.tauri.app/concept/architecture/), [Security](https://v2.tauri.app/security/), [Capabilities](https://v2.tauri.app/security/capabilities/), [Sidecars](https://v2.tauri.app/develop/sidecar/), [Shell plugin](https://v2.tauri.app/plugin/shell/), [Updater](https://v2.tauri.app/plugin/updater/).

## Agent Skills and `SKILL.md`

### Package contract

- A skill is a directory containing at least `SKILL.md`.
- `SKILL.md` starts with YAML frontmatter, then Markdown instructions.
- Required fields are `name` and `description`. Names are lowercase alphanumeric plus hyphens, with length and boundary restrictions. Description states both what the skill does and when to use it.
- Optional fields include `license`, `compatibility`, `metadata`, and experimental `allowed-tools`.
- Optional `scripts/`, `references/`, and `assets/` directories support executable helpers, on-demand documentation, and output resources.

### Loading model

The public specification recommends progressive disclosure: load name and description for discovery, load the full `SKILL.md` on activation, and load referenced resources only when needed. It recommends keeping the entrypoint concise, using relative references, and avoiding deep reference chains.

### C4OS implications

- Store parsed metadata separately from the complete instruction body so settings and prompt suggestions do not require loading every skill.
- Validate the public format without inventing incompatible required fields. C4OS-specific state such as enabled, source, scope, trust, and last error belongs in C4OS metadata, not in the portable skill file.
- Scope precedence, name collisions, customization, and invalid-state repair are C4OS product contracts and remain open.

Sources: [Agent Skills specification](https://agentskills.io/specification), [Agent Skills overview](https://agentskills.io/home).

Codex `config.toml` is researched separately in [Codex config](codex-config.md) because its layered host configuration, managed-policy boundary, and import implications are distinct from plugin packaging.

## OpenAI Codex plugins

### Bundle structure

The current public Codex plugin documentation describes a directory with required `.codex-plugin/plugin.json` and optional `skills/`, hooks, app descriptors, MCP descriptors, and assets. The manifest contains package identity, component paths, installation metadata, interface copy, capabilities, legal links, prompts, branding, and screenshots. Component paths are relative to the plugin root.

Plugins may combine:

- skills;
- MCP servers;
- apps/connectors; and
- hooks reviewed under the host's trust policy.

Installing a plugin does not override the host sandbox or approval model. External connectors and MCP servers can have their own authentication flows. Removing a plugin does not necessarily disconnect an already authorized external connector.

### Compatibility warning

The current public build documentation includes hooks, while a current sample plugin-creator validation reference in the Codex repository says hooks are rejected. This is specification drift that must be tested against the exact Codex version C4OS intends to support. C4OS must not advertise hook compatibility from documentation alone.

Sources: [Plugins overview](https://learn.chatgpt.com/docs/plugins), [Build plugins](https://learn.chatgpt.com/docs/build-plugins), [Codex plugin manifest reference in source](https://github.com/openai/codex/blob/main/codex-rs/skills/src/assets/samples/plugin-creator/references/plugin-json-spec.md), [Codex app-server plugin API](https://github.com/openai/codex/blob/main/codex-rs/app-server/README.md).

## Codex plugin marketplaces and directory

Codex currently distinguishes package catalogs from installed plugins.

- Repository marketplaces can be declared at `.agents/plugins/marketplace.json`.
- Personal marketplaces can be declared at `~/.agents/plugins/marketplace.json`.
- Sources may be local paths or remote Git repositories, with marketplace policy controlling visibility, installation availability/defaults, authentication timing, ordering, and category.
- The desktop app also reads curated sources and legacy marketplace locations.
- CLI operations support adding, listing, upgrading, and removing marketplace sources, and browsing/installing plugins.
- Install and uninstall are not enough for C4OS compatibility: version resolution, source integrity, caching, admin policy, authentication, update failure, and rollback all need explicit behavior.

For the public directory, OpenAI documents skills-only, app-only, and app-plus-skills submissions. Submission requires publisher and listing information, legal and policy fields, test prompts, availability settings, and review. MCP tools need accurate read-only, open-world, and destructive annotations. Publishing follows review rather than occurring automatically at submission.

C4OS can initially import a documented subset without claiming it is a Codex marketplace. Exact compatibility should be a versioned adapter backed by fixtures and live validator tests.

Sources: [Build plugins](https://learn.chatgpt.com/docs/build-plugins), [Submit plugins](https://learn.chatgpt.com/docs/submit-plugins), [Codex app-server plugin API](https://github.com/openai/codex/blob/main/codex-rs/app-server/README.md).

## Reference implementations

| Project | Current desktop shell | Runtime relationship | Transport and ownership | C4OS relevance |
| --- | --- | --- | --- | --- |
| [Jan](https://github.com/janhq/jan) | Tauri 2 | Jan runs local models and provider/MCP features. No embedded OpenCode integration was found in the current public main tree. The Jan organization separately owns a fork of OpenCode. | Native Tauri plugins, local inference and OpenAI-compatible API. | Strong Tauri desktop, local model, updater, keychain, MCP, and provider reference; not evidence of OpenCode-as-runtime. |
| [Atomic Chat](https://github.com/AtomicBot-ai/Atomic-Chat) | Tauri, derived from Jan's architecture | Detects, installs, configures, and launches external OpenCode, Pi, Codex, and other CLIs. | Writes an `atomic` OpenAI-compatible provider into OpenCode configuration, then opens the CLI in an OS terminal. Pi follows the same launcher pattern. | Useful installer/configuration/provider-bridge reference; not an internal chat-runtime implementation. |
| [OpenWork](https://github.com/different-ai/openwork) | Tauri | Manages OpenCode as the agent backend. | Host mode orchestrates a server; direct fallback spawns `opencode serve` on loopback. UI uses the OpenCode SDK for sessions, prompts, SSE, todos, and permission requests. | Closest public reference for a Tauri shell plus managed OpenCode server and normalized application policy. |
| [OpenChamber](https://github.com/openchamber/openchamber) | Current desktop code is Electron; legacy path used Tauri | OpenCode server is the runtime. | HTTP/SSE SDK; legacy Tauri bundled a compiled server sidecar, while current Electron can host the server in a Node process. | Useful comparison of sidecar packaging versus embedded host process. Current code must take priority over older Tauri descriptions. |
| [OpenCode](https://github.com/anomalyco/opencode) | Current desktop is Electron | Native OpenCode runtime | Electron starts a utility-process sidecar server, health-checks it, uses loopback authentication, and isolates state through environment variables. | Useful lifecycle, authentication, health, and state-isolation reference even though it is no longer a current Tauri example. |
| [Pi coding agent](https://github.com/badlogic/pi-mono) | No required desktop shell | Embeddable agent runtime | TypeScript SDK creates sessions directly; RPC mode exposes newline-delimited JSON for non-Node hosts. | Gives C4OS an embedded SDK option and a process-isolated RPC option. Existing local proofs cover core adapter behavior. |

## Key distinctions

### Jan is not evidence of an OpenCode runtime integration

Jan's current main repository is a mature Tauri application, but no OpenCode integration was found in that application tree. The separate `janhq/opencode` fork establishes organizational interest, not an application integration contract.

### Atomic Chat is a launcher and provider bridge

Atomic Chat installs OpenCode with npm, preserves/upserts its configuration, points it at Atomic's OpenAI-compatible endpoint, and opens it in a terminal. C4OS needs tighter session ownership, event streaming, tool approvals, artifact identity, and lifecycle management than this pattern provides.

### OpenWork is the closest topology match

OpenWork demonstrates the most relevant public combination: Tauri for the desktop shell, OpenCode as a managed local server, SDK/SSE for sessions and events, and an application-owned layer for approvals and filesystem semantics.

### Current OpenCode is a lifecycle reference, not a Tauri reference

Older text and search results may still describe a Tauri desktop. Current source identifies Electron and shows a managed utility-process server. The useful lesson is the authenticated loopback sidecar lifecycle, not its UI shell technology.

## Accepted architecture direction

```text
Tauri desktop host
  windows, OS integration, updater, keychain, process lifecycle
        |
C4OS application service and policy gateway
  workspaces, sessions, approvals, trusted roots, artifacts,
  persistence, plugin trust, normalized events, audit history
        |
        +--> OCAdapter --> managed OpenCode server
        |
        +--> PIAdapter --> Pi SDK sidecar or RPC process
        |
        +--> future adapter
```

OpenCode and Pi are peer user-selectable runtimes. There is no primary/default hierarchy at the architecture level. Their native capabilities are specified independently in `OCAdapter` and `PIAdapter`; the C4OS-required baseline is derived afterward. Capability reporting preserves runtime-specific strengths and honest unsupported states. OpenCode server/SSE and Pi SDK-sidecar/RPC remain transport choices inside their respective adapters rather than product hierarchy decisions.

Plugins should contribute declarative skills, apps, MCP declarations, settings, and explicitly trusted hooks through the policy gateway. They should not gain Rust-core authority or unrestricted shell access merely by being installed.

## Misleading leads to avoid

- Do not infer that Jan embeds OpenCode because its organization owns an OpenCode fork.
- Do not describe Atomic Chat's terminal launch as an embedded runtime.
- Do not repeat stale claims that the current OpenCode desktop uses Tauri without checking current source.
- Do not claim complete Codex plugin compatibility while hooks, marketplace versions, and validation behavior are unresolved.
- Do not treat a Tauri capability file as protection from malicious Rust code, overly broad command scopes, or a privileged bridge exposed to untrusted web content.
