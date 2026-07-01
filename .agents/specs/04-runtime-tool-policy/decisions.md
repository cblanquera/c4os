# Runtime And Tool Policy Decisions

Status: proposed

### DEC-001: 002: C4OS Grill Question 002 - Plugin Backend Authority

Source: `.agents/references/research/final-implementation-import/grill-session/002-c4os-grill-question-002-plugin-backend-authority.json`

  - How should C4OS app plugins access backend authority in v1?: See Notes.
  - Notes: Tauri backend is formed similar to how MCP tools work. The available tools on tauri should be based on tauri and tauri native plugins only. At the same time tauri tools should be robust enough to process almost any practical use case. By default, unless part of the app shell, most tools should remain dormant, yet available for an app plugin to utilize.  If this is unclear, ask more questions about this.

### DEC-002: 002A: C4OS Grill Question 002A - Tauri Tool Authority Boundary

Source: `.agents/references/research/final-implementation-import/grill-session/002a-c4os-grill-question-002a-tauri-tool-authority-boundary.json`

  - What does it mean for a Tauri tool to be dormant but available?: Registered backend tool, callable by any enabled app plugin that declares the tool
  - Notes: Correct me if Im wrong, but the way i'm thinking about this is `input message -> runtime -> tool call`. Without any plugins, it's possible for the runtime to call tools if it can figure out how to call and use it (like MCP servers usually have a list_tools call). With app plugins, it applies frontend app views to the tool calls that plugin supports (and given a plugin config).

### DEC-003: 002B: C4OS Grill Question 002B - Runtime Tool Discovery And Invocation

Source: `.agents/references/research/final-implementation-import/grill-session/002b-c4os-grill-question-002b-runtime-tool-discovery-and-invocation.json`

  - Without app plugins enabled, what can the runtime do with Tauri tools?: Discover and invoke any registered backend tool if tool-level approval policy allows it
  - Notes: Dont take the term "dormant" literally. What I really mean is it's generally available for runtime to discover and try to use if it wants. For example opening a website or local file without a browser/preview plugin to open the view is kind of pointless at first thought, nor do I want to ultimately assume is pointless.  At the same time, if a user doesnt like the default browser plugin, they are free to create another one themselves.

### DEC-004: 003A: C4OS Grill Question 003A - Tool Event Fanout

Source: `.agents/references/research/final-implementation-import/grill-session/003a-c4os-grill-question-003a-tool-event-fanout.json`

  - When multiple active plugin panels support a tool call, what should happen?: One tool call event fans out to all active compatible plugin views
  - What about enabled but hidden compatible plugins?: Enabled hidden plugins receive events too

### DEC-005: 020: C4OS Grill Question 020 - App Tool Approval Defaults

Source: `.agents/references/research/final-implementation-import/grill-session/020-c4os-grill-question-020-app-tool-approval-defaults.json`

  - How should C4OS define default approval policy?: Per Tauri tool/app tool with default policy and max authority
  - Which first-pass approval categories should exist?: allow, ask, deny, remember

### DEC-006: 021: C4OS Grill Question 021 - Core Tool Approval Defaults

Source: `.agents/references/research/final-implementation-import/grill-session/021-c4os-grill-question-021-core-tool-approval-defaults.json`

  - What should low-risk read/preview tools default to?: Allow all reads and previews
  - What should high-risk mutating tools default to?: Allow trusted-project file writes but ask terminal/git/network/credentials

### DEC-007: 021A: C4OS Grill Question 021A - Read Approval Boundary

Source: `.agents/references/research/final-implementation-import/grill-session/021a-c4os-grill-question-021a-read-approval-boundary.json`

  - When you say allow all reads and previews, what boundary applies?: Allow all filesystem reads without asking
  - Should previews follow the same boundary as reads?: Yes, previews follow the same boundary as reads
  - Notes: This will be the 3rd time I mentioned this, so i hope it's recorded this time. This app will be installed on a personal computer for personal consumption. It should be assumed that if a user wants to read any file on their computer, they should be allowed to. same goes if a user asks an agent to read a file. If an agent tries to read a file outside of the project folder and the user did not explicitly ask for it to be read, then the app should ask for permissions.

### DEC-008: 022: C4OS Grill Question 022 - Mutating Tool Defaults

Source: `.agents/references/research/final-implementation-import/grill-session/022-c4os-grill-question-022-mutating-tool-defaults.json`

  - What should trusted-project file writes default to?: Allow implied trusted-project writes; ask destructive deletes or broad rewrites
  - What should outside-project file writes default to?: Ask by default unless explicitly requested in the current task

### DEC-009: 023: C4OS Grill Question 023 - Non-File Risk Tool Defaults

Source: `.agents/references/research/final-implementation-import/grill-session/023-c4os-grill-question-023-non-file-risk-tool-defaults.json`

  - What should terminal command execution default to?: Ask by default; remember rules can allow repeated safe commands
  - What should git/worktree, network mutation, and credential use default to?: Allow git/worktree inside trusted projects; ask network and credentials

### DEC-010: 024: C4OS Grill Question 024 - Browser Tool Defaults

Source: `.agents/references/research/final-implementation-import/grill-session/024-c4os-grill-question-024-browser-tool-defaults.json`

  - What should Browser navigation default to?: Allow all Browser navigation and actions
  - What should screenshots and annotations default to?: All screenshots/annotations attach without asking

### DEC-011: 025: C4OS Grill Question 025 - Prompt Tag Routing

Source: `.agents/references/research/final-implementation-import/grill-session/025-c4os-grill-question-025-prompt-tag-routing.json`

  - What should `@` tags target in v1?: Plugin resources and files, based on enabled plugins
  - What should `/` commands route to in v1?: Agent CLI/runtime commands through the runtime/tool gateway
  - What should `$` tags target in v1?: Skills only

### DEC-012: 040: C4OS Grill Question 040 - Plugin Backend Registration Boundary

Source: `.agents/references/research/final-implementation-import/grill-session/040-c4os-grill-question-040-plugin-backend-registration-boundary.json`

  - How should C4OS expose plugin backend capabilities?: App-owned tool gateway with plugin-declared tool contributions and consumers
  - Where should plugin tool contributions be declared?: agents/c4os.yaml declares tools; plugin c4os/ code implements bindings
  - How should native backend registration work for marketplace plugins?: Marketplace plugins bind to preinstalled C4OS/Tauri native modules only
  - Who owns approval policy for plugin-contributed tools?: C4OS owns default and maximum approval policy; plugins can request narrower defaults

### DEC-013: 041: C4OS Grill Question 041 - Config TOML And Tool Policy

Source: `.agents/references/research/final-implementation-import/grill-session/041-c4os-grill-question-041-config-toml-and-tool-policy.json`

  - How should C4OS treat config.toml?: Both user-editable and UI-editable
  - What scope should per-tool policy defaults use?: User-global defaults with session-level narrowing
  - Which settings belong in config.toml first?: Runtime, provider/model defaults, marketplaces, plugin enablement, and app-tool policy
  - When Settings UI and config.toml disagree, which wins?: config.toml is source; Settings writes to it; parse errors keep last valid config

### DEC-014: 045: C4OS Grill Question 045 - Model Attachment Compatibility

Source: `.agents/references/research/final-implementation-import/grill-session/045-c4os-grill-question-045-model-attachment-compatibility.json`

  - How should C4OS handle model/provider attachment differences?: Store C4OS attachment records; adapters translate or degrade per model
  - What should happen when a selected model cannot consume an attachment directly?: Warn visibly and use best safe adapter fallback
  - Which provider/model path should the first attachment proof target?: Current OpenAI-compatible provider path first
  - Which attachment types need first proof coverage?: Files, Browser screenshots, and Browser annotation attachments

### DEC-015: 047: C4OS Grill Question 047 - Terminal Plugin Tool Boundary

Source: `.agents/references/research/final-implementation-import/grill-session/047-c4os-grill-question-047-terminal-plugin-tool-boundary.json`

  - Should the Terminal plugin expose only the user terminal panel or also runtime terminal app tools?: User terminal panel only; runtime terminal tools stay C4OS tool-gateway owned
  - Where should runtime terminal tool activity be visible?: Thread context plus Chat Debug; Terminal panel stays user PTY
  - Which terminal settings belong in config.toml versus Terminal plugin settings?: config.toml owns shell/env defaults and tool policy; plugin settings own panel/UI preferences
  - What environment should the user terminal start with?: User login shell environment plus explicit documented C4OS variables

### DEC-016: 049: C4OS Grill Question 049 - App Tool Taxonomy And Pi Proof

Source: `.agents/references/research/final-implementation-import/grill-session/049-c4os-grill-question-049-app-tool-taxonomy-and-pi-proof.json`

  - Which app-tool taxonomy should be frozen before plugin details?: Let each plugin define its own taxonomy independently
  - Which app-tool categories must be represented first?: Only plugin-contributed tools
  - Which Pi capabilities are proof-critical?: Prompt execution, streaming, tool-call interception, approval denial, and resume
  - Where should `/` prompt commands route?: Runtime/tool gateway command handling with C4OS and plugin command definitions
