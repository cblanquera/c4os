# C4OS r007 Single Page App Rebuild Specification

## Revision Summary

- Revision folder: `wireframes/r007-single-page-app-rebuild/`
- Revision: `r007-single-page-app-rebuild`
- Revision type: new major revision
- Baseline source: `wireframes/r04-single-page-app/`
- Product area: complete C4OS desktop application shell
- Current approved scope: r05 Batch 1 shell plus the accepted r007 Batch 2 Configuration and Plugins work over the restored r04 bodies
- Included from r05 Batch 1: one global header, left/right plugin icon groups, plugin-panel toggling and same-side replacement, per-chat panel restoration, Settings closure/restoration, resize collision handling, hidden activity, and invalid-layout repair
- Batch 2 boundary: complete and closed; Configuration and Plugins are accepted, while the proposed remaining Batch 2 additions were rejected as mistaken scope rather than deferred work
- Explicitly deferred: Batch 3 prompt/workspace behavior, Batch 4 file/editor changes, and Batch 5 Browser/Terminal/Chat Debug detail
- Body-source boundary: later r05 additions remain deferred, but the complete r04 body for each corresponding surface is required now and is not a later-batch addition
- Explicitly excluded: `wireframes/r06-final-implementation/`
- Trigger: user requested a truth-oriented SPA rebuilt through `chrisai-designing`, using the bundled wireframe library as a guide
- Open blocking questions: none for r05 Batch 1

## Source Of Truth

- `wireframes/r04-single-page-app/`
  - Supplies the starting route inventory, three-panel shell, visual density, rendered content, and interactive behavior.
  - It is the direct baseline for this round, not a source to redesign.
- `wireframes/r05-final-implementation/README.md`
  - Supplies the accepted additions and removals applied batch-by-batch after r04 parity approval.
  - Batch 1 and the accepted r007 Batch 2 boundary govern the current artifact; Batch 3 and later batches remain deferred until started.
- `wireframes/r05-final-implementation/notes.md` and `review-round-*.md`
  - Supply the accepted correction history needed to avoid restoring rejected intermediate r05 states.
- `wireframes/screens.md`
  - Confirms r04 as the accepted original SPA baseline and identifies where later r05 batches supersede it.
- `plans/product-interface.md`
  - Supplies the inherited three-panel interaction and content contract represented by r04.
- `.agents/context/creative-specs.md`
  - Supplies current accepted interface truth and the caution that later r05 plugin-first behavior supersedes parts of r04.
  - It governs the Batch 1 shell now applied over the approved r04 foundation.
- `chrisai-designing/assets/wireframes/lib/`
  - Supplies the grayscale token, reset, base, component, layout, icon, interaction, and state patterns used as a guide.
  - The library is a starting point rather than a complete C4OS design system.

## Application And Screen Inventory

The wireframe is an explicit single page app. `index.html` is the only product application page. Each screen below is a hash-addressable application state rendered into the same document.

### Plugin-First Shell Foundation

- Routes:
  - `./index.html#shell-foundation`
  - `./index.html#same-side-replacement`
  - `./index.html#per-chat-restore`
  - `./index.html#resize-collision`
  - `./index.html#hidden-activity`
  - `./index.html#repair-state`
  - `./index.html#settings`
- Goal: review the accepted r05 Batch 1 shell architecture as working SPA state rather than isolated static screens.
- Layout: one full-width global header, optional left and right plugin panels, and one persistent center chat/workspace region.
- States: no panel open, both sides open, same-side replacement, chat-specific restoration, collision resolution, hidden-plugin activity, Settings center route, and invalid-layout repair.
- Navigation: global header plugin icons toggle panels; Chats changes active chat; Settings temporarily replaces the center route and closes panels; Back to app restores the prior chat and panel state.
- Visible body ownership:
  - Center workspace uses the r04 chat thread, activity, approval, and composer body.
  - Chats remains the r05 main-shell chat plugin.
    - A `+ New Chat` action sits directly below chat search and opens the r04 empty-chat prompt in the center without leaving the shell.
  - File System uses the r04 project search, local project folders, and chats nested per project.
    - The Add Project control is functional and opens the same r04 empty-chat prompt state in the center workspace.
  - File Editor opens with the r04 folder/file tree and replaces that body with the r04 editor after a file is selected.
  - Browser uses the r04 Browser body.
  - Terminal uses the top region of the r04 Terminal body.
  - Chat Debug uses the bottom command-results region of the r04 Terminal body.
  - Settings uses the complete r04 Settings navigation and content body inside the r05 center-route behavior.

### App Start

- Route: `./index.html#app-start`
- Goal: establish a trusted local project before prompting.
- Layout: centered start content with recent workspaces.
- States: no trusted project, recent workspaces.
- Navigation: recent workspace opens New Session.

### New Session

- Route: `./index.html#new-session`
- Goal: begin work in the trusted project shell.
- Layout: resizable left navigation, center workbench, resizable right Browser panel.
- States: empty thread, composer controls, Browser selected.

### Chat Session

- Route: `./index.html#chat-session`
- Goal: review message ownership, agent activity, tool events, and approvals.
- Layout: shared three-panel shell.
- States: user message, agent message, expanded agent content, tool completion, approval wait.

### Provider And Model Selection

- Routes: `./index.html#providers-popover` and `./index.html#models-popover`
- Goal: navigate providers and select the active model.
- Layout: New Session shell with a composer popover.
- States: provider list, active provider models, selected model.

### File Explorer And Editor

- Routes: `./index.html#file-explorer` and `./index.html#file-editor`
- Goal: move between project files and a code view.
- Layout: shared shell with Files active in the right tool panel.
- States: dense file tree, selected file, breadcrumbs, code view.

### Terminal

- Route: `./index.html#terminal`
- Goal: review user terminal output and the r04 agent command-results region.
- Layout: shared shell with vertically resizable Terminal regions.
- States: terminal output and command-results preview.

### Settings

- Routes:
  - `./index.html#settings-providers`
  - `./index.html#settings-add-provider`
  - `./index.html#settings-models`
  - `./index.html#settings-runtimes`
  - `./index.html#settings-configuration`
  - `./index.html#settings-plugins`
  - `./index.html#settings-skills`
  - `./index.html#settings-mcp`
- Goal: review the complete r04 settings inventory and its form, list, toggle, modal, and detail states.
- Layout: settings navigation with one center content region.
- Navigation: Back to app returns to New Session; settings links replace the center route without loading another document.
- Accepted Batch 2 additions:
  - Canonical review route: `./index.html#settings-configuration`, rendered inside the r05 shell rather than the historical standalone r04 shell.
  - Four registered server-tool policy rows: `terminal.run`, `files.read`, `git.worktree`, and `credentials.use`.
  - Each row exposes its default and maximum policy plus edit and revoke actions.
  - Edit opens an inline policy editor and Save updates the visible row state.
  - Revoke leaves the registered tool visible while marking its remembered rule revoked.
  - Parse-error state explains that `config.toml` is authoritative, blocks saving, and keeps the last valid configuration active.
  - Plugins retain the r04 two-column list and overview dialog while Advanced settings owns dependency, repair, redaction, and uninstall states.
  - Skills retains its restored r04 body with no additional Batch 2 work; proposed later Skills additions are rejected and Batch 2 is closed.

## Workflow Starting Points

`workflows.html` is the reviewer-facing starting-points page. It is not part of the product shell and contains no implementation annotations inside `index.html`.

- Review the plugin-first shell
  - Start: `./index.html#shell-foundation`
  - Happy path: open one panel on each side, close an active panel, and preserve the center workspace.
- Replace a same-side panel
  - Start: `./index.html#same-side-replacement`
  - Happy path: select another icon on a populated side and replace that side without disturbing the opposite panel.
- Restore panels per chat
  - Start: `./index.html#per-chat-restore`
  - Happy path: change chat, change its panels, return, and recover each chat's last panel state.
- Exercise collision and activity states
  - Starts: `./index.html#resize-collision` and `./index.html#hidden-activity`
  - Happy path: resize toward the 640px center minimum and review activity on a closed plugin.
- Review repair and Settings restoration
  - Starts: `./index.html#repair-state` and `./index.html#settings`
  - Happy path: review an invalid layout declaration, enter Settings, and return to the prior chat shell.

- Trust a workspace
  - Start: `./index.html#app-start`
  - Happy path: choose a recent workspace, arrive at New Session.
- Start and inspect a chat
  - Start: `./index.html#new-session`
  - Happy path: choose the active project or chat, inspect the composer and Chat Session.
- Change the model
  - Start: `./index.html#models-popover`
  - Happy path: return to providers, choose a provider, select a model.
- Browse and edit files
  - Start: `./index.html#file-explorer`
  - Happy path: select a file, inspect breadcrumbs and code view, return through a folder breadcrumb.
- Inspect terminal surfaces
  - Start: `./index.html#terminal`
  - Happy path: review output and resize the bottom results region.
- Configure C4OS
  - Start: `./index.html#settings-providers`
  - Happy path: navigate the complete settings route set and open representative forms or details.

## Layout System

### Start Layout

- Used by App Start.
- Regions: lead content and recent workspaces.
- Behavior: adapts from split content to stacked content at constrained widths.

### Three-Panel Application Shell

- Used by New Session, Chat Session, popovers, File Explorer, File Editor, and Terminal.
- Regions: resizable left project navigation, center workbench with top bar and composer/thread, resizable right workspace tool panel.
- Behavior: left and right panels can collapse; both can resize while preserving a usable center width.
- Source decision: the bundled panel layout was inspected as a guide, but r04's custom two-sided resizable shell remains the faithful layout contract.

### Plugin-First Application Shell

- Used by every accepted r05 Batch 1 route.
- Regions: one global header spanning the window, zero or one left plugin panel, a center chat/workspace region with a 640px minimum, and zero or one right plugin panel.
- Header ownership: configured plugin icons live in fixed left and right groups; the active chat title remains centered.
- Panel behavior: selecting an inactive icon opens its panel; selecting its active icon closes it; selecting another icon on the same side replaces that panel; left and right panels can coexist.
- Settings behavior: Settings is a center route, not a plugin panel. Entering it closes both panels and leaving restores the prior chat's panel state.
- Responsive behavior: resize collision closes the opposite-side panel before clamping the dragged panel so the center never falls below 640px.
- Composition boundary: r05 supplies the global header, plugin placement, panel state, restoration, and collision rules; it does not replace r04 bodies with simplified substitutes.

### Settings Layout

- Used by every settings route.
- Regions: persistent settings navigation and scrollable settings content.
- Behavior: route changes replace settings content in the SPA.

## Component Inventory

- Buttons, icon buttons, chips, status pills, cards, settings rows, toggles, form fields, popovers, dialogs, tabs, file-tree rows, breadcrumbs, message bubbles, activity blocks, composer, and resize handles.
- Icons use local inline Lucide-guided SVG paths with accessible names on icon-only controls.
- Focus-visible styling is required for links, buttons, inputs, selects, textareas, and editable elements.
- Dialog and popover controls must be keyboard-reachable and close through their visible controls; Escape behavior is required where the r04 artifact already models it.
- The Send control keeps one stable emphasized treatment and does not introduce a separate hover fill.
- Composer action controls have no left container padding and retain `10px` right padding around the Send control.
- The provider-list header contains only a left-aligned, vertically centered `Providers` title; both the model-list header container and its back row have no horizontal padding.
- The plugin source menu repeats `Built by C4OS`, then uses a divider before `+ Add Marketplace`.
- Unselected runtime rows rely on their radio control without a redundant `Choose runtime` status label.
- The MCP connection dialog omits the `Docs` action.
- The Settings label has `12px` top padding below Back to app.

## Interaction And State Contract

- Plugin panel toggle
  - Trigger: select a global-header plugin icon.
  - Result: toggle that plugin on its configured side without changing the active chat.
- Same-side replacement
  - Trigger: select a different plugin icon on a side that already has a visible panel.
  - Result: replace only that side's panel; preserve the opposite side and center route.
- Per-chat panel persistence
  - Trigger: change between chat sessions in the Chats panel.
  - Result: save the outgoing chat's left/right panel selection and restore the incoming chat's last selection.
- Empty chat prompt
  - Trigger: select `+ New Chat` in Chats or Add Project in File System.
  - Result: retain the current plugin shell and replace the center thread with r04's `What should we build in c4os2?` prompt state.
- New-chat model selection
  - Trigger: select the model chip in the r04 empty-chat composer.
  - Result: open the provider/model picker locally above the composer, preserve the r05 global header and open plugin panels, and update the selected model without navigating to an r04 shell route.
- Settings closure and restoration
  - Trigger: enter Settings, then use Back to app.
  - Result: close both plugin panels while Settings is visible, then restore the prior active chat and its panels.
- Collision handling
  - Trigger: drag a panel resize handle until the center would become narrower than 640px.
  - Result: close the opposite panel first, then clamp the dragged panel to the remaining legal width.
- Hidden activity
  - Trigger: activity arrives for a plugin whose panel is closed.
  - Result: show a restrained unread indicator on that plugin's header icon without opening or focusing its panel.
- Invalid shell repair
  - Trigger: a plugin declares reserved or invalid shell layout settings.
  - Result: preserve a usable shell and show a visible repair/disable state instead of applying the corrupt layout.

- Hash navigation
  - Trigger: internal route link or browser history change.
  - Result: replace the current rendered state inside `#app`, update the hash, and focus `#main` without a document reload.
- Panel collapse
  - Trigger: left or right top-bar control.
  - Result: toggle the corresponding panel and update `aria-pressed`.
- Panel resize
  - Trigger: pointer drag on a horizontal panel handle.
  - Result: update the panel CSS width variable within the configured minimum, maximum, and center-width constraints.
- Terminal stack resize
  - Trigger: vertical drag on the Terminal separator.
  - Result: resize the lower Terminal results region within its limits.
- Composer controls
  - Trigger: attachment, approval, branch, provider, or model controls.
  - Result: show the corresponding functional local state and update visible selections where r04 supports it.
- Message disclosure
  - Trigger: collapse/expand or Show More/Show Less.
  - Result: update visible message content and accessible expanded state.
- Settings interactions
  - Trigger: settings navigation, forms, toggles, detail controls, and modal controls.
  - Result: show the matching route or simulated local state without leaving the SPA.
- Persistence
  - Route state is linkable through the URL hash.
  - Ephemeral control states remain in memory unless cross-route persistence is necessary to understand the r04 flow.

## Library Plan

- `lib/base/tokens.css`
  - Copied and used for the canonical grayscale palette, spacing, radii, typography, dimensions, and motion tokens.
- `lib/base/reset.css`
  - Copied and used for predictable box sizing, media, control, link, and hidden-state behavior.
- `lib/base/base.css`
  - Copied and used as the shared semantic utility and accessibility foundation.
- `styles.css`
  - Adapted from r04 and remapped onto the library token system while retaining C4OS-specific shell, component, density, and responsive rules.
- `script.js`
  - Reconstructed from the r04 route/component/state model and kept as a single dependency-free SPA module.
- Bundled panel layouts and component samples
  - Inspected as implementation guides but not copied where the r04-specific shell or component treatment is more truthful.
- Remote packages and sample pages
  - Not used.

## Page Build Plan

### `index.html`

- Title: C4OS r007 Single Page App Wireframe.
- Imports: document-relative base tokens, reset, base styles, C4OS styles, and SPA script.
- Initial state: `#shell-foundation` when no valid hash route is supplied.
- Interaction hooks: hash routing, delegated SPA links, local control bindings, pointer resize bindings, and settings/modal bindings.
- Links: every product transition resolves to an `index.html#route` state in the same SPA.

### `workflows.html`

- Title: C4OS r007 Workflow Starting Points.
- Layout: simple grayscale reviewer navigation surface using the same base library.
- Links: document-relative deep links into `./index.html#route`.

## Functional Acceptance Checks

- `specs.md`, `notes.md`, `workflows.html`, and `index.html` exist.
- Every specified hash route renders inside `index.html` without a document reload.
- Shell Foundation defaults correctly and direct hash entry works.
- Global header plugin icons toggle, close, and replace panels on their configured sides.
- Left and right plugin panels can coexist while the center remains at least 640px wide.
- Chat switching restores each chat's prior panel state.
- Settings closes panels and Back to app restores the prior chat shell.
- Hidden plugin activity remains an icon indicator and does not steal focus.
- Invalid shell layout state leaves the application usable and exposes a repair path.
- Browser back and forward restore routes.
- Left and right panels collapse and resize.
- Terminal vertical resizing works.
- Model/provider, file/editor, Settings, disclosure, composer, modal, and toggle interactions required for r04 review work.
- Every workflow link resolves to a known SPA route.
- Assets use document-relative paths.
- The rendered UI is grayscale, legible, and free of annotations, TODOs, review notes, coverage matrices, and implementation commentary.
- The artifact works from static files and through a local static server.
- This round is r05 Batch 1 only and must not be treated as approval of the later r05 batches.
