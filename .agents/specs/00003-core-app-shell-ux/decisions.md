# Core App Shell UX Decisions

Status: proposed

### DEC-001: 003: C4OS Grill Question 003 - Tool View Selection

Source: `.agents/resources/grill/final-implementation/answers/003-c4os-grill-question-003-tool-view-selection.json`

  - When multiple app plugins support the same Tauri tool, how should C4OS choose the view?: Other
  - Notes: From my original task list:  ``` - Frontend - No right panel by default. One singular header (with plugin icons to the far left or right based on each config. Ability to reorder icons.). Clicking plugin icons will toggle show the plugin panel view. Clicking an active plugin icon will close the relative panel view. Both active left and right panel should be able resize up to the center pane fixed minimum width (set a fix minimum width for center pane) ```  What that means is it's possible to have more than one plugin to use the same Tauri tool, depends on what is the active plugin in the panel view (left or right). If there is a browser plugin (browser-1) on the left and active and another different browser plugin (browser-2) on the right panel, then saying "load x.com in browser" should load x.com on both browser plugins.  Let me know if there are discrepencies with this because im just answering hypotheticals on the fly.

### DEC-002: 003A: C4OS Grill Question 003A - Tool Event Fanout

Source: `.agents/resources/grill/final-implementation/answers/003a-c4os-grill-question-003a-tool-event-fanout.json`

  - When multiple active plugin panels support a tool call, what should happen?: One tool call event fans out to all active compatible plugin views
  - What about enabled but hidden compatible plugins?: Enabled hidden plugins receive events too
  - Hidden plugin event semantics: The backend tool runs once and C4OS records
    one event. Visible compatible plugin views may render or act immediately.
    Hidden enabled compatible plugin instances may update per-chat state and
    unread/activity indicators, but must not open panels, steal focus, prompt
    the user, or trigger another backend call.
  - Refinement source: 2026-07-02 grill intake, "Hidden Plugin Tool Event
    Semantics".

### DEC-003: 004: C4OS Grill Question 004 - Plugin Instance Scope

Source: `.agents/resources/grill/final-implementation/answers/004-c4os-grill-question-004-plugin-instance-scope.json`

  - What is the default state scope for tool-view plugins?: Per chat session
  - Can the same plugin have multiple instances in one chat session?: One instance per plugin per chat session

### DEC-004: 010: C4OS Grill Question 010 - Unassigned Chat Scope

Source: `.agents/resources/grill/final-implementation/answers/010-c4os-grill-question-010-unassigned-chat-scope.json`

  - Where should unassigned chats live?: User-global shell chat history
  - What should the unassigned chat area be called in the UI?: Chats

### DEC-005: 011: C4OS Grill Question 011 - Panel Coexistence

Source: `.agents/resources/grill/final-implementation/answers/011-c4os-grill-question-011-panel-coexistence.json`

  - Can multiple plugin panels on the same side be visible at once?: One visible plugin panel per side; same-side icons replace the visible panel
  - Can left and right plugin panels be visible at the same time?: Yes, one left and one right panel can be visible together

### DEC-006: 012: C4OS Grill Question 012 - Panel Visibility Persistence

Source: `.agents/resources/grill/final-implementation/answers/012-c4os-grill-question-012-panel-visibility-persistence.json`

  - Should currently open plugin panels persist after app restart?: Yes, persist per chat session
  - When switching chat sessions, should visible panels change?: Restore visible panels from each chat session

### DEC-007: 013: C4OS Grill Question 013 - Center Pane And Panel Resize

Source: `.agents/resources/grill/final-implementation/answers/013-c4os-grill-question-013-center-pane-and-panel-resize.json`

  - What should the center chat pane minimum width be?: 640px
  - What happens when side panels would shrink the center below its minimum?: Automatically close the opposite-side panel then, stop panel expansion at the center minimum

### DEC-008: 014: C4OS Grill Question 014 - Settings Placement

Source: `.agents/resources/grill/final-implementation/answers/014-c4os-grill-question-014-settings-placement.json`

  - Where should Settings open in the plugin-first shell?: Center pane shell route
  - What happens to visible plugin panels when Settings opens?: Close all plugin panels, then restore per-chat panels when leaving Settings

### DEC-009: 015: C4OS Grill Question 015 - Plugin Icon Reordering

Source: `.agents/resources/grill/final-implementation/answers/015-c4os-grill-question-015-plugin-icon-reordering.json`

  - Where should users reorder plugin icons?: Directly in the header only
  - Where should users change a plugin icon's left/right side?: Other
  - Notes: A plugin should be able to configure its own settings form for the user to fill out. The user needs to click the plugin to configure it. Actual user configs for all plugins should be stored in the user level.

### DEC-010: 015A: C4OS Grill Question 015A - Plugin Configuration Entry

Source: `.agents/resources/grill/final-implementation/answers/015a-c4os-grill-question-015a-plugin-configuration-entry.json`

  - How should users open a plugin's configuration form?: Primary click toggles panel; configuration is only in Settings > Plugins
  - Where should users change a plugin's left/right placement?: Other
  - Notes: For example a plugin config could look like:  ``` {   settings: {     username: {        field: 'string'     },     prompt: {        default: false     },     panel: {        field: ['left', 'right'],        default: 'left'     }   } } ```  Where as:  ``` {   field?: 'string' | 'text' | 'boolean' | string[];   default?: string|boolean|number; } ```  Then the app shell picks up certain keys like "panel" to determine location on app view.
  - Reserved-key validation: `panel`, `enabled`, and `iconOrder` are
    shell-interpreted user settings. Plugins may declare them only with
    compatible field types and allowed values. Invalid reserved-key
    declarations disable or repair the affected shell contribution with a
    visible reason.
  - Refinement source: 2026-07-02 grill intake, "Plugin Settings Field
    Metadata".

### DEC-011: 044: C4OS Grill Question 044 - Plugin SVG Icon Constraints

Source: `.agents/resources/grill/final-implementation/answers/044-c4os-grill-question-044-plugin-svg-icon-constraints.json`

  - Where can plugin SVG icons come from?: Installed plugin bundle assets only, referenced by relative path
  - What SVG content should C4OS allow for plugin icons?: Static sanitized SVG only; no scripts, external refs, event handlers, foreignObject, animation, or embedded data
  - How should plugin icons render in the shell?: Fixed-size icon slots with fallback icon on failure
  - Should plugin SVG icons be themeable by C4OS?: Preserve author colors; allow declared monochrome mask mode

### DEC-012: Backend App Architect Resolution - Shell UX

Source: 2026-07-02 user architect profile in active chat.

  - Shell principle: The app shell stays intentionally thin. It owns chat/session
    identity, Settings routing, plugin mounting, approval surfaces, and shared
    layout state; plugins own enhanced work surfaces.
  - Event model: Plugin panel visibility, unread/activity indicators, tool
    result hydration, and Settings restore behavior are driven by typed C4OS
    shell/plugin events, not direct cross-plugin UI calls.
  - Worker-friendly UX: Header icons, panel toggles, empty states, repair
    states, and Settings language must be understandable to operations,
    support, admin, research, and normal knowledge workers, not only coders.
  - Maintainability: Shell layout behavior must be implemented as a small
    state machine with documented transitions for icon click, same-side
    replacement, opposite-side collision, Settings enter/leave, session switch,
    and hidden plugin event delivery.

### DEC-013: 2026-07-02 POC Batch - Shell Panel State Machine

Source: `proofs/shell-panel-resize-and-restore/`.

  - Promote a shell-owned state machine for plugin panel toggles, same-side
    replacement, per-chat visible panel restoration, Settings close/restore,
    and resize collision handling.
  - Resize behavior must preserve the 640px center-pane minimum by closing the
    opposite-side panel when needed before clamping panel expansion.
  - This is a POC decision only. It does not freeze this spec or create
    implementation progress items.
