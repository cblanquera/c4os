# Shell Plugin Architecture Refactor Decisions

Status: proposed

### DEC-001: 001: C4OS Grill Question 001 - Plugin Packaging Boundary

Source: `.agents/references/research/final-implementation-import/grill-session/001-c4os-grill-question-001-plugin-packaging-boundary.json`

  - For C4OS final implementation, how should built-in plugins be packaged?: Bundled plugin roots inside the main repo
  - Notes: Builtin plugins should be part of the default app install, but not enabled by default. App plugins are a different concept from Tauri plugins.

### DEC-002: 002: C4OS Grill Question 002 - Plugin Backend Authority

Source: `.agents/references/research/final-implementation-import/grill-session/002-c4os-grill-question-002-plugin-backend-authority.json`

  - How should C4OS app plugins access backend authority?: See Notes.
  - Notes: Tauri backend is formed similar to how MCP tools work. The available tools on tauri should be based on tauri and tauri native plugins only. At the same time tauri tools should be robust enough to process almost any practical use case. By default, unless part of the app shell, most tools should remain dormant, yet available for an app plugin to utilize.  If this is unclear, ask more questions about this.

### DEC-003: 002A: C4OS Grill Question 002A - Tauri Tool Authority Boundary

Source: `.agents/references/research/final-implementation-import/grill-session/002a-c4os-grill-question-002a-tauri-tool-authority-boundary.json`

  - What does it mean for a Tauri tool to be dormant but available?: Registered backend tool, callable by any enabled app plugin that declares the tool
  - Notes: Correct me if Im wrong, but the way i'm thinking about this is `input message -> runtime -> tool call`. Without any plugins, it's possible for the runtime to call tools if it can figure out how to call and use it (like MCP servers usually have a list_tools call). With app plugins, it applies frontend app views to the tool calls that plugin supports (and given a plugin config).

### DEC-004: 002B: C4OS Grill Question 002B - Runtime Tool Discovery And Invocation

Source: `.agents/references/research/final-implementation-import/grill-session/002b-c4os-grill-question-002b-runtime-tool-discovery-and-invocation.json`

  - Without app plugins enabled, what can the runtime do with Tauri tools?: Discover and invoke any registered backend tool if tool-level approval policy allows it
  - Notes: Dont take the term "dormant" literally. What I really mean is it's generally available for runtime to discover and try to use if it wants. For example opening a website or local file without a browser/preview plugin to open the view is kind of pointless at first thought, nor do I want to ultimately assume is pointless.  At the same time, if a user doesnt like the default browser plugin, they are free to create another one themselves.
  - View-oriented tool behavior without plugin views: If approval policy allows
    it, the runtime may execute view-oriented tools fully even when no
    compatible app-plugin view is enabled or visible. C4OS must store the
    resulting inspectable state so a compatible view can display it later.
  - View-oriented tool state ownership: Inspectable state from a view-oriented
    tool run before a compatible plugin view exists lives as app-owned
    per-chat tool result state keyed by tool identity plus resource/session
    target. Compatible plugin views hydrate from that state when opened or
    enabled. The state is deleted with the chat unless it is attached to a sent
    prompt or saved by a plugin-specific action.
  - Shared hydration rule: Multiple compatible enabled plugin instances in the
    chat may hydrate the same app-owned source state. Each plugin may keep its
    own view-local UI state, but cannot claim, mutate, or delete the shared
    source state unless it calls an explicit C4OS action governed by policy.
  - Refinement source: 2026-07-02 grill intake, "Tool Invocation Without
    Plugin Views"; 2026-07-02 grill intake, "View-Oriented Tool State
    Ownership"; 2026-07-02 grill intake, "Shared Hydration Of View Tool
    State".

### DEC-005: 003: C4OS Grill Question 003 - Tool View Selection

Source: `.agents/references/research/final-implementation-import/grill-session/003-c4os-grill-question-003-tool-view-selection.json`

  - When multiple app plugins support the same Tauri tool, how should C4OS choose the view?: Other
  - Notes: From my original task list:  ``` - Frontend - No right panel by default. One singular header (with plugin icons to the far left or right based on each config. Ability to reorder icons.). Clicking plugin icons will toggle show the plugin panel view. Clicking an active plugin icon will close the relative panel view. Both active left and right panel should be able resize up to the center pane fixed minimum width (set a fix minimum width for center pane) ```  What that means is it's possible to have more than one plugin to use the same Tauri tool, depends on what is the active plugin in the panel view (left or right). If there is a browser plugin (browser-1) on the left and active and another different browser plugin (browser-2) on the right panel, then saying "load x.com in browser" should load x.com on both browser plugins.  Let me know if there are discrepencies with this because im just answering hypotheticals on the fly.

### DEC-006: 003A: C4OS Grill Question 003A - Tool Event Fanout

Source: `.agents/references/research/final-implementation-import/grill-session/003a-c4os-grill-question-003a-tool-event-fanout.json`

  - When multiple active plugin panels support a tool call, what should happen?: One tool call event fans out to all active compatible plugin views
  - What about enabled but hidden compatible plugins?: Enabled hidden plugins receive events too
  - Hidden plugin event semantics: The backend tool runs once and C4OS records
    one event. Visible compatible plugin views may render or act immediately.
    Hidden enabled compatible plugin instances may update per-chat state and
    unread/activity indicators, but must not open panels, steal focus, prompt
    the user, or trigger another backend call.
  - Refinement source: 2026-07-02 grill intake, "Hidden Plugin Tool Event
    Semantics".

### DEC-007: 004: C4OS Grill Question 004 - Plugin Instance Scope

Source: `.agents/references/research/final-implementation-import/grill-session/004-c4os-grill-question-004-plugin-instance-scope.json`

  - What is the default state scope for tool-view plugins?: Per chat session
  - Can the same plugin have multiple instances in one chat session?: One instance per plugin per chat session
  - Instance clarification: Per-chat plugin instances are lightweight
    view/state instances, not backend service/process instances. Heavy plugin
    services must not be spawned per chat by default; they are shared at the
    narrowest safe scope and reused across chats where safe.
  - Refinement source: 2026-07-02 grill intake, "Plugin Lifecycle
    Restart-Gated Backend Registration".

### DEC-008: 017: C4OS Grill Question 017 - C4OS Plugin Manifest

Source: `.agents/references/research/final-implementation-import/grill-session/017-c4os-grill-question-017-c4os-plugin-manifest.json`

  - What should `agents/c4os.yaml` be responsible for?: App-shell contribution metadata only
  - Which fields should `agents/c4os.yaml` include?: Other
  - Metadata boundary: Preserve Codex `plugin.json` as the compatibility
    surface for Codex-defined plugin identity/capabilities such as skills,
    apps/MCP, version, and marketplace metadata. Put only C4OS app-shell
    contributions in `agents/c4os.yaml`: `schemaVersion`, C4OS plugin id/name,
    icon asset path, panel/view contributions, settings schema including
    reserved shell keys, tool contributions/consumers, dependencies, service
    lifecycle declarations, and compatibility constraints.
  - Dependency types: `agents/c4os.yaml` supports typed dependencies for app
    plugins, C4OS/preinstalled native modules, heavy services, and required
    C4OS capabilities/tool identities.
  - Manifest section boundary: Dependencies are a separate top-level
    `agents/c4os.yaml` manifest section, not `settings` fields. The `settings`
    section remains a user-facing form schema from Q015A/Q016.
  - Refinement source: 2026-07-02 grill intake, "Codex Plugin Metadata
    Boundary"; 2026-07-02 grill intake, "Plugin Dependency Types";
    2026-07-02 grill intake, "Dependency Schema Separate From Settings".
  - Notes: Follow standards on Codex's plugin.json spec. For `agents/c4os.yaml` convert my last json example that describes the form builder config for plugin settings to yaml format as well as id, name, icon. ("panel" should be included in form builder config if it applies)

### DEC-009: 038: C4OS Grill Question 038 - Codex Plugin Compatibility Boundary

Source: `.agents/references/research/final-implementation-import/grill-session/038-c4os-grill-question-038-codex-plugin-compatibility-boundary.json`

  - What minimum Codex plugin subset should C4OS support?: Full Codex plugin compatibility
  - Should bundled built-in plugins be uninstallable?: Uninstallable like user-installed plugins
  - What icon format should plugin metadata support?: Bundled SVG asset path
  - How should C4OS handle agents/c4os.yaml version incompatibility?: Require schemaVersion; disable incompatible plugins with visible reason
  - Notes: User correction requires final-implementation framing and rejects version-phase language.

### DEC-010: 039: C4OS Grill Question 039 - Plugin Marketplace And Lifecycle

Source: `.agents/references/research/final-implementation-import/grill-session/039-c4os-grill-question-039-plugin-marketplace-and-lifecycle.json`

  - Which Codex marketplace sources should C4OS support?: Bundled/default, repo, personal, local root, GitHub, Git URL, ref, and sparse paths
  - Should built-in C4OS plugins be modeled as a bundled marketplace source?: Yes; built-ins are a bundled/default marketplace source
  - Where should installed plugin bundles live?: C4OS-owned cache path mirroring Codex cache semantics
  - After uninstall, how should reinstall work?: Remove installed cache copy; reinstall from marketplace source
  - When should plugin lifecycle changes require restart?: Live for UI/settings; restart required for native backend registration changes
  - What happens to plugin-owned user data when a plugin is uninstalled?: Prompt during uninstall whether to delete data

### DEC-011: 040: C4OS Grill Question 040 - Plugin Backend Registration Boundary

Source: `.agents/references/research/final-implementation-import/grill-session/040-c4os-grill-question-040-plugin-backend-registration-boundary.json`

  - How should C4OS expose plugin backend capabilities?: App-owned tool gateway with plugin-declared tool contributions and consumers
  - Where should plugin tool contributions be declared?: agents/c4os.yaml declares tools; plugin c4os/ code implements bindings
  - How should native backend registration work for marketplace plugins?: Marketplace plugins bind to preinstalled C4OS/Tauri native modules only
  - Who owns approval policy for plugin-contributed tools?: C4OS owns default and maximum approval policy; plugins can request narrower defaults

### DEC-012: 050: C4OS Grill Question 050 - Plugin Migration Failure Handling

Source: `.agents/references/research/final-implementation-import/grill-session/050-c4os-grill-question-050-plugin-migration-failure-handling.json`

  - Which plugin migration failures should C4OS automatically recover from?: Auto-recover cache, reinstallable bundle, and stale-index failures only
  - Which plugin migration failures should cause visible disablement instead of recovery?: Unsupported agents/c4os.yaml schemaVersion, Missing or disabled dependency, Security policy violation, Unavailable required preinstalled native module, Invalid or unreadable plugin manifest, Corrupt plugin-owned user config, Missing marketplace source or non-reinstallable plugin bundle
  - How should C4OS handle corrupt plugin-owned user config during migration?: Reset corrupt plugin config automatically and keep plugin enabled
  - Where should migration failures be surfaced to the user?: Settings > Plugins with disabled reason and repair actions
