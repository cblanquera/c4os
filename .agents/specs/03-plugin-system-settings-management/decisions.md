# Plugin System And Settings Management Decisions

Status: proposed

### DEC-001: 001: C4OS Grill Question 001 - Plugin Packaging Boundary

Source: `.agents/references/research/final-implementation-import/grill-session/001-c4os-grill-question-001-plugin-packaging-boundary.json`

  - For C4OS v1, how should built-in plugins be packaged?: Bundled plugin roots inside the main repo
  - Notes: Builtin plugins should be part of the default app install, but not enabled by default. App plugins are a different concept from Tauri plugins.

### DEC-002: 005: C4OS Grill Question 005 - Plugin Configuration Scope

Source: `.agents/references/research/final-implementation-import/grill-session/005-c4os-grill-question-005-plugin-configuration-scope.json`

  - Where should C4OS app plugin enablement live by default?: User/global only
  - Where should panel side, icon order, and hidden/visible view config live?: User/global only

### DEC-003: 006: C4OS Grill Question 006 - User Config Location

Source: `.agents/references/research/final-implementation-import/grill-session/006-c4os-grill-question-006-user-config-location.json`

  - Where should C4OS user-global config live?: Other
  - For Windows, what should the equivalent be?: Other
  - Notes: You can decide this. Follow industry standard for app config per user location depending on OS. Somewhat related: In the current MVP it puts project level config in `.c4os` folder in the project root, I didnt want to pollute project folders with an app hidden folder. I wanted it all in a config per user folder instead.

### DEC-004: 015A: C4OS Grill Question 015A - Plugin Configuration Entry

Source: `.agents/references/research/final-implementation-import/grill-session/015a-c4os-grill-question-015a-plugin-configuration-entry.json`

  - How should users open a plugin's configuration form?: Primary click toggles panel; configuration is only in Settings > Plugins
  - Where should users change a plugin's left/right placement?: Other
  - Notes: For example a plugin config could look like:  ``` {   settings: {     username: {        field: 'string'     },     prompt: {        default: false     },     panel: {        field: ['left', 'right'],        default: 'left'     }   } } ```  Where as:  ``` {   field?: 'string' | 'text' | 'boolean' | string[];   default?: string|boolean|number; } ```  Then the app shell picks up certain keys like "panel" to determine location on app view.

### DEC-005: 016: C4OS Grill Question 016 - Plugin Settings Schema

Source: `.agents/references/research/final-implementation-import/grill-session/016-c4os-grill-question-016-plugin-settings-schema.json`

  - What field types should plugin settings support in v1?: string, text, boolean, number, and enum arrays
  - Which plugin settings keys should the app shell reserve?: Reserve `panel`, `enabled`, and `iconOrder`
  - Notes: What field types should plugin settings support in v1? - string = input type text - text = textarea  - boolean = switch  - number = input type number - enum arrays = select  Which plugin settings keys should the app shell reserve? You can make it up as we go along.

### DEC-006: 017: C4OS Grill Question 017 - C4OS Plugin Manifest

Source: `.agents/references/research/final-implementation-import/grill-session/017-c4os-grill-question-017-c4os-plugin-manifest.json`

  - What should `agents/c4os.yaml` be responsible for in v1?: App-shell contribution metadata only
  - Which fields should `agents/c4os.yaml` include first?: Other
  - Notes: Follow standards on Codex's plugin.json spec. For `agents/c4os.yaml` convert my last json example that describes the form builder config for plugin settings to yaml format as well as id, name, icon. ("panel" should be included in form builder config if it applies)

### DEC-007: 018: C4OS Grill Question 018 - Plugin Dependencies

Source: `.agents/references/research/final-implementation-import/grill-session/018-c4os-grill-question-018-plugin-dependencies.json`

  - Where should C4OS app plugin dependencies be declared?: `agents/c4os.yaml`
  - What should happen when enabling a plugin with a disabled dependency?: Block enablement until the user manually enables dependencies

### DEC-008: 019: C4OS Grill Question 019 - Disabling Plugin Dependencies

Source: `.agents/references/research/final-implementation-import/grill-session/019-c4os-grill-question-019-disabling-plugin-dependencies.json`

  - What happens when a user disables a plugin that other enabled plugins depend on?: Automatically disable dependent plugins
  - How should dependency-blocked plugins appear in Settings?: Visible with disabled controls and dependency-blocked reason

### DEC-009: 038: C4OS Grill Question 038 - Codex Plugin Compatibility Boundary

Source: `.agents/references/research/final-implementation-import/grill-session/038-c4os-grill-question-038-codex-plugin-compatibility-boundary.json`

  - What minimum Codex plugin subset should C4OS support in v1?: Full Codex plugin compatibility
  - Should bundled built-in plugins be uninstallable?: Uninstallable like user-installed plugins
  - What icon format should plugin metadata support first?: Bundled SVG asset path first
  - How should C4OS handle agents/c4os.yaml version incompatibility?: Require schemaVersion; disable incompatible plugins with visible reason
  - Notes: There is no more soft MVP, MVP, Post MVP, v1. There is only the final implementation. Please stop referencing v1 as if there will be a v2...

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

### DEC-012: 044: C4OS Grill Question 044 - Plugin SVG Icon Constraints

Source: `.agents/references/research/final-implementation-import/grill-session/044-c4os-grill-question-044-plugin-svg-icon-constraints.json`

  - Where can plugin SVG icons come from?: Installed plugin bundle assets only, referenced by relative path
  - What SVG content should C4OS allow for plugin icons?: Static sanitized SVG only; no scripts, external refs, event handlers, foreignObject, animation, or embedded data
  - How should plugin icons render in the shell?: Fixed-size icon slots with fallback icon on failure
  - Should plugin SVG icons be themeable by C4OS?: Preserve author colors; allow declared monochrome mask mode

### DEC-013: 050: C4OS Grill Question 050 - Plugin Migration Failure Handling

Source: `.agents/references/research/final-implementation-import/grill-session/050-c4os-grill-question-050-plugin-migration-failure-handling.json`

  - Which plugin migration failures should C4OS automatically recover from?: Auto-recover cache, reinstallable bundle, and stale-index failures only
  - Which plugin migration failures should cause visible disablement instead of recovery?: Unsupported agents/c4os.yaml schemaVersion, Missing or disabled dependency, Security policy violation, Unavailable required preinstalled native module, Invalid or unreadable plugin manifest, Corrupt plugin-owned user config, Missing marketplace source or non-reinstallable plugin bundle
  - How should C4OS handle corrupt plugin-owned user config during migration?: Reset corrupt plugin config automatically and keep plugin enabled
  - Where should migration failures be surfaced to the user?: Settings > Plugins with disabled reason and repair actions
