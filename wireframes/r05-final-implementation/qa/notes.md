# QA Notes

Date: 2026-07-02

## Batch 1 Scope Check

- Artifact is limited to frontend wireframe files under `wireframes/r05-final-implementation/`.
- No `.agents` spec files, `wireframes/screens.md`, or `wireframes/ui-handoff-spec.md` were updated before approval.
- Links in `index.html` and generated route links are document-relative or hash-only.
- The artifact uses grayscale HTML/CSS/JS only.
- The local `.agents/workflows/feedback-loop.md` file requested in the goal does not exist in this checkout; the `chrisai-designing` feedback-loop workflow was used for review-round structure.

## Verification

- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static scan found no root-relative `href`, `src`, or CSS `url()` paths.
- Static server returned `200 OK` for `/`, `/script.js`, and `/styles.css` at `http://127.0.0.1:4185/`.
- Browser plugin loaded `http://127.0.0.1:4185/index.html#per-chat-restore`.
- Browser DOM check confirmed active header icon buttons for Chats and Browser, visible left `Chats panel`, visible right `Browser panel`, and no spec coverage text inside the shell route.
- Browser viewport screenshot confirmed the revised r05 uses r04-style grayscale shell, inline SVG icons, message cards, composer, and plugin panels instead of the rejected annotated shell surface.
- Browser comparison pass against current frontend screenshots confirmed side-panel headers/close buttons were removed; `panelTopbars: 0`, `closeButtons: 0`.
- Browser comparison pass confirmed the current `#per-chat-restore` route uses the app-like composer context strip, chat transcript shape, and minimal Browser panel chrome from the attached frontend screenshots.
- Browser comparison pass confirmed the current `#settings` route includes the Settings rail, search field, `Built by C4OS` filter, active Plugins nav state, and GitHub plugin row from the attached frontend screenshot.
- Static scan confirmed Search is no longer a standalone header plugin icon.
- Static scan confirmed side-panel headers and close buttons remain removed.
- Browser plugin verified `#debug` with header buttons `Chats`, `Files`,
  `Browser`, `Terminal`, `Debug`, and `Settings`; no standalone Search header
  plugin appears.
- Browser plugin verified Chats panel search is present as `Search chats`.
- Browser plugin verified Debug panel shows command terminal output plus
  structured events: `terminal.run`, `tool_call_requested`,
  `tool_output_delta`, and `approval_policy`.
- Browser plugin verified Browser, Terminal, and Debug panel body padding is
  `0px`.
- Browser plugin verified Files panel body padding is `0px` while preserving
  file-tree row spacing.
- The Batch 1 temporary static server was left running for user review at
  `http://127.0.0.1:4185/` during that review pass.

## Batch 2 Verification

- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static route inventory confirmed the 11 new Batch 2 Settings routes have
  document-relative hash links in `script.js`.
- Static scan found no root-relative `href`, `src`, or `url(...)` references in
  `wireframes/r05-final-implementation/`.
- Temporary server on `http://127.0.0.1:4186/` was started for browser
  verification.
- Browser plugin verified all Batch 2 routes at 1280px viewport:
  `#settings-plugins`, `#settings-plugin-detail`,
  `#settings-plugin-marketplace`, `#settings-plugin-states`,
  `#settings-plugin-uninstall`, `#settings-configuration`,
  `#settings-config-error`, `#settings-skills`, `#settings-skill-detail`,
  `#settings-skill-customize`, `#settings-skill-invalid`, and `#coverage`.
- Browser plugin confirmed every Batch 2 route rendered `#main`, the expected
  page heading, no `undefined`, no `[object Object]`, and no horizontal
  overflow at 1280px viewport.
- Browser plugin captured non-empty representative screenshots for
  `#settings-plugin-detail`, `#settings-configuration`, `#settings-skills`, and
  `#coverage`.

## Round 4 Feedback Verification

- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Browser plugin verified revised routes through
  `http://127.0.0.1:4186/index.html`: `#settings-plugin-detail`,
  `#settings-plugin-marketplace`, `#settings-configuration`, and
  `#settings-skills`.
- Browser plugin confirmed `#settings-plugin-detail` now has `Rendered form
  from plugin config`, `Repair states`, and `Tool policy` sections, includes an
  Enabled control, and no longer contains `Shell-reserved config`.
- Browser plugin confirmed `#settings-plugin-marketplace` includes the source
  menu pattern plus `Add plugin marketplace` modal content with `Sparse paths`.
- Browser plugin confirmed `#settings-configuration` exposes icon-only edit and
  revoke controls for remembered rules.
- Browser plugin confirmed `#settings-skills` includes the r04-style Skills
  search field and list panel.
- Browser plugin captured non-empty representative screenshots for all four
  revised routes.
- Temporary server remains running for review at `http://127.0.0.1:4186/`.

## Round 5 Feedback Verification

- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Browser plugin verified the corrected routes through
  `http://127.0.0.1:4187/index.html`: `#settings-configuration`,
  `#settings-plugin-marketplace`, `#settings-skills`, and
  `#settings-skill-detail`.
- Browser plugin confirmed Settings > Configuration no longer contains
  `Session rule` and now uses only user-global remembered-rule framing.
- Browser plugin confirmed Settings > Plugin Marketplace includes the r04-like
  plugin store, toolbar, marketplace filter menu, add-marketplace form panel,
  and 9 plugin cards.
- Browser plugin confirmed Settings > Skills includes the r04-like Skills
  search, 7 skill rows, scope column, enabled switches, and skill detail panel.
- Browser plugin confirmed no `undefined` or `[object Object]` text appears on
  the verified routes and no horizontal overflow appears at the checked
  viewport.
- Temporary server remains running for review at `http://127.0.0.1:4187/`.

## Round 6 Feedback Verification

- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static scan confirmed the active r05 JS/CSS no longer contains
  `Session rule`, `User-global remembered`, `No remembered rules`, or
  `User-global rule`.
- Browser plugin verified `#settings-configuration` through
  `http://127.0.0.1:4187/index.html`: policy rows have zero remembered-rule
  text paragraphs and retain icon-only edit/revoke actions.
- Browser plugin verified `#settings-plugin-marketplace`: 5 plugin add buttons
  are present, the Built by C4OS filter starts hidden, the filter menu opens,
  `+ Add Marketplace` opens the marketplace dialog, and a plugin add button
  opens the connect dialog.
- Follow-up Browser plugin verification confirmed the marketplace route renders
  as `Plugins` with `Manage installed plugins and extension surfaces.`, has no
  duplicate generic plugin control bar, keeps one marketplace search field, and
  keeps one marketplace filter trigger.
- Browser plugin verified `#settings-skills`: 7 skill row buttons are present,
  the skill detail modal starts hidden, and clicking `ChrisAI Agents` opens the
  detail modal with matching title content.
- Round 7 Browser plugin verification confirmed `#settings-plugin-marketplace`
  renders `Plugins`, shows 5 Built by C4OS plugin cards (`File system`,
  `File editor`, `Terminal`, `Chat Debug`, `Browser`), and the marketplace menu
  contains `Built by C4OS`, a separator, and `+ Add Marketplace`.
- Round 7 Browser plugin verification confirmed the plugin connect modal DOM
  exposes `Advanced settings` with `href="#settings-plugin-detail"`.
- Round 7 Browser plugin verification confirmed `#settings-plugin-detail`
  renders `File editor plugin`, the `Rendered form from plugin config`, the
  File editor field examples, `Repair states`, and `Tool policy`.
- Temporary server remains running for review at `http://127.0.0.1:4187/`.

## Batch 3 Verification

- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static scan found no root-relative `href`, `src`, or `url(...)` references in
  `wireframes/r05-final-implementation/`.
- Static route inventory confirmed the 7 new Batch 3 prompt routes are present:
  `#prompt-suggestions`, `#approval-dialog`, `#remembered-rule-summary`,
  `#blocked-suggestion-repair`, `#branch-popover`, `#attachment-states`, and
  `#safe-fallback`.
- Static scan found no `undefined`, `[object Object]`, legacy session-rule
  labels, or stale global remembered-rule placeholder copy in the active JS/CSS.
- Local static server returned `200 OK` for `/`, `/script.js`, and
  `/styles.css` at `http://127.0.0.1:4188/`.
- Browser rendering was attempted with Playwright. The bundled Playwright
  browser is not installed, and system Chrome aborted under the current runtime,
  so no browser screenshot pass was recorded for Batch 3 in this run.
- Temporary server remains running for review at `http://127.0.0.1:4188/`.

## Round 9 Feedback Verification

- Reworked Batch 3 routes after user feedback that Round 08 felt instructional
  and annotation-like rather than functional.
- Static scan confirms the active prompt routes now use `thread-view`,
  `thread-list`, `work-log`, `composer-dock`, `permission-prompt`, and
  composer attachment/popover elements patterned after the current frontend and
  r04 chat-session mockups.
- `node --check wireframes/r05-final-implementation/script.js` passed after
  the functional chat-session rework.
- `git diff --check -- wireframes/r05-final-implementation` passed after the
  rework.
- Static scan found no root-relative `href`, `src`, or `url(...)` references in
  `wireframes/r05-final-implementation/`.
- No `.agents` specs, `wireframes/ui-handoff-spec.md`, or durable handoff docs
  were updated in this feedback round.

## Round 10 Prompt Reference Verification

- Added prompt-reference vocabulary to `CONTEXT.md`: Prompt Trigger, Active
  Query, Typeahead Menu, Resolved Inline Reference, Unresolved Token, and
  Serialized Prompt.
- Updated `#prompt-suggestions` to show trigger-scoped typeahead behavior:
  `$` Skills, `@` plugin resources before files/folders, and `/` C4OS built-in
  commands before accepted file/resource matches.
- Updated resolved inline reference styling so exact-match or selected tokens
  render blue in the composer.
- Added serialized prompt text to show the runtime-facing canonical reference
  expansion for visible prompt tokens.
- `node --check wireframes/r05-final-implementation/script.js` passed after
  the prompt-reference terminology update.
- `git diff --check -- wireframes/r05-final-implementation CONTEXT.md` passed
  after the update.

## Round 11 Prompt Typeahead Correction

- Removed chip/bubble styling from typed prompt triggers. Resolved references
  now render as inline blue text only.
- Updated `#prompt-suggestions` so the composer shows a pending `$grill` skill
  query and the typeahead menu contains only matching skills.
- Removed the visible `Typeahead menu` title, explanatory boundary text,
  cross-trigger examples, and serialized prompt annotation from the popup.
- Repositioned the typeahead popup above the fixed composer.
- Kept serialized prompt behavior as a coverage/spec boundary rather than
  visible instructional copy in the prompt wireframe.
- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation CONTEXT.md`
  passed.
- Static scan confirmed the active JS/CSS no longer contains `Typeahead menu`,
  trigger-boundary helper text, `typeahead-head`, `typeahead-examples`, or
  `serialized-prompt`.
- Browser verification on
  `http://127.0.0.1:4188/index.html#prompt-suggestions` confirmed the
  composer text is `use $grill to ask me questions`, the popup contains only
  `$ Skills`, the popup is above the composer, the pending token background is
  transparent with `0px` radius, and the resolved sent-message token is blue.
- Follow-up browser verification confirmed `#prompt-suggestions` no longer
  renders a work-log annotation for the live draft state.

## Round 12 Prompt Interaction Repair

- Restored the r04-style `Worked for 5sec >` activity row in the prompt
  session route.
- Added r04-style Show More / Show Less disclosure behavior for the agent

## Batch 4 Verification

- Added focused workspace-and-files routes for specs 06 and 07 only; no
  `.agents` specs, `wireframes/ui-handoff-spec.md`, or product code were
  updated before approval.
- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static scan found no root-relative `href`, `src`, or `url(...)` references in
  `wireframes/r05-final-implementation/`.
- Static route scan confirmed the new Batch 4 routes are present, including
  `#workspace-start` and `#file-external-conflict`.
- Temporary server on `http://127.0.0.1:4189/` returned `200 OK` for `/`,
  `/script.js`, and `/styles.css`.
- The in-app Browser tool was not active in this session, so no browser
  screenshot pass was recorded for Batch 4.

## Round 21 Functional Wireframe Correction

- Reworked Batch 4 routes after user feedback that the original round felt too
  instructional/annotated and not functional enough.
- Workspace routes now render an app-like workspace manager table, toolbar,
  missing-project action menu, and non-Git composer state
  instead of explanatory cards.
- File/editor routes now keep the editor as the primary surface and show the
  file context menu, create row, trash confirmation, save/revert controls,
  external-change conflict, and non-code empty picks in-place.
- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static scan found no root-relative `href`, `src`, or `url(...)` references in
  `wireframes/r05-final-implementation/`.
- No `.agents` specs, `wireframes/ui-handoff-spec.md`, or product code were
  updated in this correction round.

## Round 22 Workspace Start Ownership Correction

- Reworked `#workspace-start` after user feedback that Workspace belongs under
  the FS plugin and should not take over the center workbench.
- The FS left panel now contains the r04-style start actions and recent
  folder-backed workspace rows.
- The center pane now stays on the original app-shell new-chat prompt state.
- Related missing-project controls remain inside the FS panel pattern.
- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static scan found no root-relative `href`, `src`, or `url(...)` references in
  `wireframes/r05-final-implementation/`.
- No `.agents` specs, `wireframes/ui-handoff-spec.md`, or product code were
  updated in this correction round.

## Round 23 Workspace Add Project Removal

- Removed `#workspace-add-project` after user feedback that it was no longer
  relevant.
- Removed the FS panel title row that showed `File system` and the plus icon.
- Removed active review links and coverage references for the deleted route.
- Static scan confirmed no active `workspace-add-project` route, navigation
  link, or coverage row remains in the r05 HTML/CSS/JS/README surfaces.
- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static scan found no root-relative `href`, `src`, or `url(...)` references in
  `wireframes/r05-final-implementation/`.
- No `.agents` specs, `wireframes/ui-handoff-spec.md`, or product code were
  updated in this correction round.

## Round 25 Loaded Workspace Transition

- Added `#workspace-loaded` after user feedback that the transition from
  `#workspace-start` to `#workspace-missing-project` needed a normal loaded
  workspace state.
- Updated the `c4os` recent workspace row to open `#workspace-loaded`.
- Kept the loaded workspace state inside the FS left panel, with the center
  pane still showing the app-shell new-chat prompt.
- Updated the coverage matrix and README route list with `#workspace-loaded`.
- No `.agents` specs, `wireframes/ui-handoff-spec.md`, or product code were
  updated in this correction round.

## Round 26 r04 Loaded Workspace Panel Match

- Updated `#workspace-loaded` after user feedback and screenshot reference
  showed the loaded state did not match r04.
- Replaced workspace-management controls with r04-style project navigation:
  large search field, Projects heading with plus action, folder rows with
  edit/trash icons, active `c4os2` row, and indented chat sessions.
- Updated the center prompt copy to `What should we build in c4os2?`.
- No `.agents` specs, `wireframes/ui-handoff-spec.md`, or product code were
  updated in this correction round.

## Round 27 r04 Density Correction

- Corrected `#workspace-loaded` after user feedback that the r04 screenshot was
  a density reference, not a target to scale up.
- Removed oversized custom route typography and spacing.
- Restored r04-scale values for the loaded FS panel: 44px search field, 38px
  project rows, 36px session rows, 14px text, normal icon size, and the r04
  left-panel width clamp.
- Updated the search placeholder to `Search projects`.
- No `.agents` specs, `wireframes/ui-handoff-spec.md`, or product code were
  updated in this correction round.
  message.
- Added wireframe-local composer behavior so typing `$`, `@`, or `/` into the
  prompt activates the matching typeahead menu.
- `$` filters skills, `@` shows resource matches, and `/` shows runtime
  command matches. Trailing space or plain text hides the menu.
- Moved the typeahead popup to a fixed top-layer position above the bottom
  composer so rows receive pointer events instead of the thread grid.
- Browser verification on
  `http://127.0.0.1:4188/index.html#prompt-suggestions` confirmed `$`, `@`,
  and `/` all open their matching menus; plain trailing text hides the menu;
  selecting `grill-me-with-docs` resolves to inline blue text with transparent
  background and `0px` radius; `Worked for 5sec >` expands; and Show More /
  Show Less toggles correctly.

## Round 13 Caret-Scoped Typeahead Repair

- Replaced full-prompt typeahead activation with caret-scoped activation.
  Clicking in the prompt now checks the current selection offset and only opens
  typeahead when the caret is inside a pending `$`, `@`, or `/` token.
- Replaced text-arrow `Worked for 5sec >` / `Worked for 5sec v` labels with a
  real chevron icon that rotates when expanded.
- Kept typeahead hidden when the caret is outside a trigger token or after a
  whitespace boundary.
- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation CONTEXT.md`
  passed.
- Browser verification could not be completed for this round because Browser
  use rejected reloading the localhost review URL under the current URL
  policy. No alternate browser-control workaround was used.

## Round 14 Typeahead Placement Correction

- Moved the fixed typeahead popover higher above the bottom composer so it no
  longer covers the editable prompt text while typing.

## Round 15 Multi-Reference And Keyboard Typeahead Repair

- Changed suggestion resolution so the prompt is re-rendered from recognized
  references after each selection. Previously, selecting a new typeahead row
  flattened earlier blue references back to black text.
- Added ArrowDown, ArrowUp, and Enter handling while the typeahead menu is open.
- Added active-row state updates through `is-selected` and `aria-selected`.
- Prevented already-resolved blue references from reopening typeahead when the
  caret is inside them.
- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation CONTEXT.md`
  passed.
- The local server remained active on port `4188`. The already-open browser tab
  was reachable, but it must be refreshed to load the updated script.

## Round 16 Prompt Markup Normalization Repair

- Added prompt markup normalization on every input event so typed text cannot
  remain trapped inside the most recent blue resolved-reference span.
- Backspace/input edits now re-run active-trigger detection immediately after
  normalizing known references.
- Known `$`, `@`, and `/` references stay blue only when the visible token
  exactly matches a recognized reference.
- Attempted to use Browser for live verification as requested, but Browser use
  rejected access to the current localhost review URL under the URL policy. No
  alternate browser-control workaround was used.

## Round 17 Approval Dialog Readability Repair

- Restyled the approval dialog as a light card with explicit dark text instead
  of inheriting the dark `permission-prompt` foreground color.
- Kept the command preview as a high-contrast dark code block.
- Changed approval impact details to an even two-column grid with a fixed label
  column and flexible value column.
- Restyled remembered-choice rows so radio labels are readable on the light
  approval surface.
- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation CONTEXT.md`
  passed.

## Round 18 Approval Advanced Accordion And Responsive Repair

- Moved approval impact rows and remembered-choice controls into a collapsed
  `Advanced` accordion.
- Added the `Advanced` accordion toggle with chevron rotation.
- Changed the approval card to stretch to the available composer dock width
  instead of leaving a blank right-side column at wider responsive sizes.
- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation CONTEXT.md`
  passed.

## Round 19 Approval Action Wording

- Removed the `Ask each time` approval action.
- Added `Deny and wait` to represent denying the requested action while keeping
  the session paused for further prompt instructions.

## Round 28 Batch 4 Self-QA Correction

- Treated the workspace/files feedback as a batch-level self-QA failure rather
  than another acceptance handoff.
- Kept `#workspace-start` in the already-reviewed FS-left-panel shape.
- Tightened `#workspace-loaded`, `#workspace-missing-project`, and
  `#workspace-non-git` so the sidebar state changes match a r04-style project
  list instead of an invented center workspace manager.
- Removed rendered file-explorer explanation labels and the icon-theme note card
  from the active UI. File states now show product rows only.
- Hid `.git` from the active explorer state while leaving `.env.example` visible.
- Changed Batch 4 file route titles away from the invented `Draft handoff`
  wording.
- Removed dead workspace-manager/table/savebar functions from the script.
- Removed dead workspace-manager/table/savebar/card CSS from the stylesheet.
- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Targeted source scan found no active `Markdown icon`, `hidden file visible`,
  `hidden by rule`, `Icon theme`, `Draft handoff`,
  `workspaceManagerToolbar`, `workspaceProjectItem`, `workspaceSaveBar`, or
  rendered `.git` file row in `script.js` or `styles.css`.
- Browser verification was attempted but blocked by the in-app Browser URL
  policy for this `file://` review page, so no browser screenshot claim is
  attached to this round.

## Round 29 File System And File Editor Split

- Left-aligned `.prompt-text` so the chat prompt no longer inherits centered
  empty-workspace alignment.
- Changed the loaded workspace search control into a clickable route target for
  `#workspace-search`.
- Added a separate File Editor plugin icon and moved file explorer/editor states
  out of the File System plugin panel.
- Added `#file-editor` as the normal code-view route reached by clicking file
  rows in the explorer.
- Reworked File Editor explorer rows to match r04 structure and density more
  closely: folder rows, indented file rows, 30px row height, and active outline.
- Kept `.git` hidden while preserving `.env.example` as the hidden-file visible
  row.
- Kept the center new-chat prompt visible while File Editor is open, matching
  the r04 panel behavior.
- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Targeted source assertions passed for separate File Editor plugin ownership,
  `#file-editor`, clickable file links, `#workspace-search`, FS-only workspace
  panel ownership, left-aligned prompt text, and r04-style explorer row names.

## Round 30 r04 File Editor Match

- Compared r05 `#file-editor` against r04 source.
- r04's normal file editor is breadcrumbs plus code pane only; r05 had added
  save/revert toolbar chrome to the base editor state.
- Removed the toolbar from base `#file-editor`.
- Kept save/revert toolbar behavior only for `#file-editor-dirty` and
  `#file-external-conflict`.
- Aligned File Editor panel layout and code-pane styling with r04: 36px
  breadcrumbs, 13px monospace code, sticky line numbers, `max-content` code
  rows, and `white-space: pre`.

## Round 31 File Editor Padding

- Reduced visual inset in `#file-editor` after feedback that the editor had too
  much left and top padding.
- Changed File Editor breadcrumb rows from 36px to 32px.
- Reduced code pane top padding from 14px to 8px.
- Reduced the line-number gutter from 48px to 34px.
- Reduced line-number and code-left padding so code starts closer to the panel
  edge while preserving the r04 breadcrumb/code-pane structure.

## Round 32 Project Row Actions Menu

- Changed project row actions from pencil/trash to `...` plus pencil.
- Preserved pencil as the new-chat action.
- Added per-project `...` menu toggles.
- Kept the missing-project menu open by default in
  `#workspace-missing-project`.
- Simplified menu order by state:
  - Missing project: Relocate, Copy path, Rename, Remove.
  - Found project: Reveal, Copy path, Rename, Remove.
- Kept the current light r05 theme and current type scale.

## Round 33 Project Menu Rendering Repair

- Fixed project action menus rendering as full nested boxes under every project.
- Moved menu markup inside the project row so session rows remain in normal
  sidebar flow.
- Added explicit `[hidden]` display suppression for project menus.
- Positioned the open menu as a compact absolute popover under the row actions.
- Removed outlined button treatment from project menu items.
- Preserved chat/session rows under active/missing projects.
- Corrected menu order:
  - Missing project: Relocate, Copy path, Rename, Remove.
  - Found project: Reveal, Copy path, Rename, Remove.
- Rendered `#workspace-missing-project` with Playwright and confirmed exactly
  one visible project menu, five hidden menus, two visible session rows, and
  menu items in the expected order.

## Round 34 Project Menu Acceptance Fix

- Added the missing `Copy path` icon.
- Removed `Reveal` from the missing-project menu.
- Kept `Reveal` only for found-project menus.
- Tightened project menu popover padding from the inherited `12px` panel
  padding to `4px`.
- Left-aligned menu button text and verified icon/text grid alignment.
- Rendered `#workspace-missing-project` with Chromium and confirmed:
  - Missing project menu: Relocate, Copy path, Rename, Remove.
  - Found project menu: Reveal, Copy path, Rename, Remove.
  - Every menu row has one icon.
  - Menu row text aligns left.
  - Menu padding is `4px`.
  - `Homepage refresh` and `Launch copy` remain present.

## Round 35 Annotation Removal

- Removed the visible center-panel annotation from
  `#workspace-missing-project`.
- Removed the unused `read-only-banner` styling.
- Kept the missing-project sidebar row, open project actions menu, and read-only
  chat rows intact.
- Source scan found no remaining `read-only-banner` class or
  `moved-marketing-site chats are visible` string in active JS/CSS.
- Rendered `#workspace-missing-project` with Chromium and confirmed the center
  prompt has only the heading and composer, with no read-only banner.
- Rendered check confirmed the missing-project menu still shows Relocate, Copy
  path, Rename, Remove.

## Batch 4 Approval Sync

- User approved Batch 4 on 2026-07-04.
- Synced approved workspace/files evidence into specs 06 and 07.
- Promoted durable workspace/files UI behavior into the compact creative
  context and `wireframes/ui-handoff-spec.md`.
- Updated `wireframes/screens.md` and this artifact README to mark Batch 4 as
  approved.
- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- .agents wireframes` passed.
- Static link scan found no root-relative links in the synced surfaces.
- Rendered Chromium checks confirmed:
  - `#workspace-missing-project` has no visible annotation banner.
  - Missing-project menu items are Relocate, Copy path, Rename, Remove.
  - `#file-editor` renders breadcrumbs and code view without the dirty toolbar.
  - `#coverage` contains the Batch 4 matrix with no stale
    `reveal/copy/remove actions` missing-project wording.

## Manual Review Targets

- `./index.html#shell-foundation`
- `./index.html#same-side-replacement`
- `./index.html#per-chat-restore`
- `./index.html#resize-collision`
- `./index.html#hidden-activity`
- `./index.html#debug`
- `./index.html#repair-state`
- `./index.html#settings`
- `./index.html#settings-plugins`
- `./index.html#settings-plugin-detail`
- `./index.html#settings-plugin-marketplace`
- `./index.html#settings-plugin-states`
- `./index.html#settings-plugin-uninstall`
- `./index.html#settings-configuration`
- `./index.html#settings-config-error`
- `./index.html#settings-skills`
- `./index.html#settings-skill-detail`
- `./index.html#settings-skill-customize`
- `./index.html#settings-skill-invalid`
- `./index.html#prompt-suggestions`
- `./index.html#approval-dialog`
- `./index.html#remembered-rule-summary`
- `./index.html#blocked-suggestion-repair`
- `./index.html#branch-popover`
- `./index.html#attachment-states`
- `./index.html#safe-fallback`
- `./index.html#workspace-start`
- `./index.html#workspace-loaded`
- `./index.html#workspace-missing-project`
- `./index.html#workspace-search`
- `./index.html#workspace-non-git`
- `./index.html#files-left-panel`
- `./index.html#files-right-panel`
- `./index.html#file-editor`
- `./index.html#file-context-menu`
- `./index.html#file-operations`
- `./index.html#file-editor-dirty`
- `./index.html#file-external-conflict`
- `./index.html#file-empty-states`
- `./index.html#coverage`

## Review Round 36 Batch 5 Runtime Plugin Panels

- Added Batch 5 runtime plugin panel routes for Terminal, Browser, Chat Debug,
  and related runtime state:
  - `./index.html#terminal-user-pty`
  - `./index.html#terminal-lifecycle`
  - `./index.html#terminal-cleanup`
  - `./index.html#terminal-settings`
  - `./index.html#terminal-scrollback`
  - `./index.html#browser-navigation`
  - `./index.html#browser-annotations`
  - `./index.html#browser-clear-after-send`
  - `./index.html#browser-preview-host`
  - `./index.html#browser-state-hydration`
  - `./index.html#browser-security-boundary`
  - `./index.html#debug-disabled-entry`
  - `./index.html#debug-timeline`
  - `./index.html#debug-event-detail`
  - `./index.html#debug-retention`
  - `./index.html#coverage`
- Added `./batch-5-links.html` as the Batch 5 acceptance review hub with
  document-relative route links.
- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static link scan found no root-relative links in Batch 5 review files.
- In-app Browser blocked direct `file://` review; QA continued through the
  supported local URL `http://127.0.0.1:4191/`.
- In-app Browser verified all 16 Batch 5 routes rendered expected text,
  expected route state, workbench content, and panel count.
- In-app Browser representative screenshot byte checks were nonblank for
  `#terminal-user-pty`, `#browser-annotations`, `#debug-event-detail`, and
  `#coverage`.
- In-app Browser click QA passed for Terminal, Browser, Debug, Terminal close,
  timeline-to-detail navigation, and the Batch 5 link hub route.
- In-app Browser console error log was empty during click QA.
- In-app Browser layout pass across all 16 Batch 5 routes found no overflowing
  button, link, chip, status pill, or Debug tab text.
- Follow-up in-app Browser requirement audit found `#browser-navigation`
  needed visible Back / Forward / Refresh action text. Added that state row and
  reran the audit with a fresh-load URL.
- Fresh-load in-app Browser requirement audit passed all 15 focused Batch 5
  routes, with no rendered review-note, TODO, or implementation note terms.
- Continuation audit confirmed the current worktree remains limited to
  `wireframes/r05-final-implementation/`, `node --check` still passes,
  `git diff --check -- wireframes/r05-final-implementation` still passes, and
  the root-relative link scan remains clean.
- Continuation in-app Browser live audit reloaded the Batch 5 review hub and
  sampled `#terminal-user-pty`, `#browser-navigation`,
  `#browser-annotations`, `#browser-clear-after-send`,
  `#debug-event-detail`, `#debug-retention`, and `#coverage`; all sampled
  routes showed the expected state evidence, no export button, and no rendered
  review/TODO/implementation-note terms.

## Review Round 37 Terminal Scope Correction

- Applied feedback that the Batch 5 Terminal wireframe should be the r04 user
  terminal output pane with the agent debug/results pane removed, not a set of
  standalone backend-state screens.
- Removed the Terminal lifecycle/cleanup/settings/scrollback, Browser
  clear-after-send/hydration/security-boundary, and Chat Debug retention routes
  from the current route map and link hub.
- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static scan found no stale removed Batch 5 route names in current review
  files: `script.js`, `batch-5-links.html`, `README.md`, `notes.md`, and
  `review-round-37.md`.
- Static scan found no root-relative links in the revised r05 review files.
- Local static server ran at `http://127.0.0.1:4191/`.
- In-app Browser verified the revised link hub has 8 links only:
  `#terminal-user-pty`, `#browser-navigation`, `#browser-annotations`,
  `#browser-preview-host`, `#debug-disabled-entry`, `#debug-timeline`,
  `#debug-event-detail`, and `#coverage`.
- In-app Browser route sweep passed all 8 current routes with no stale removed
  route links, no rendered runtime-state cards, no horizontal overflow, and no
  old r04 agent-results pane copy on `#terminal-user-pty`.
- In-app Browser click QA passed for the link hub Terminal route, Terminal
  header close, Browser header open, Debug header open, and Debug inspect link.
- In-app Browser console error log was empty during click QA.
- In-app Browser mobile viewport sweep at 390x844 passed all 8 current routes
  with no document overflow or visible control text overflow after excluding the
  hidden skip link.

## Review Round 38 User Terminal Annotation Removal

- Applied feedback that `#terminal-user-pty` still annotated the user terminal
  with explanatory chrome.
- Removed the `Interactive terminal` status strip from the Terminal panel.
- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Existing `file://` tab reload was blocked by Browser policy, so rendered QA
  used a local static server at `http://127.0.0.1:4192/`.
- In-app Browser verified `#terminal-user-pty` has the terminal panel and output
  block, no `.terminal-status-strip`, no `.runtime-state-card`, no
  `Interactive terminal`, no `User PTY`, no `zsh -` chrome, and no console
  errors.

## Review Round 39 Terminal Spacing and Browser Chrome

- Applied feedback to reduce top padding and add left padding in the user
  Terminal panel.
- Updated Browser toolbar controls to show Back, Forward, Refresh, centered
  `iamawesome.com`, Screenshot, Annotate, and Browser menu.
- Added Browser toolbar menu and page right-click context menu states matching
  the provided reference contents.
- Repaired a missing CSS brace that prevented later Browser/Terminal rules from
  applying in the in-app Browser.
- Added document-relative asset cache tokens in `index.html` for reliable
  Browser QA of current CSS/JS.
- Renamed Browser frame state classes to `browser-mode-*` so
  `.browser-page-context-menu` applies only to the actual page menu.
- Tightened Browser toolbar spacing and constrained the Browser toolbar menu to
  the approved right plugin panel width.
- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static scan found no root-relative links in the edited review files.
- In-app Browser QA used local server `http://127.0.0.1:4194/`.
- In-app Browser verified `#terminal-user-pty` has `4px` top padding, `20px`
  left padding, no `.terminal-status-strip`, and no console errors.
- In-app Browser verified `#browser-navigation`, `#browser-menu`, and
  `#browser-page-context-menu` with current asset token `batch5-r40`, expected
  toolbar controls, expected menu text, one scoped page context menu, and no
  console errors.

## Review Round 40 Browser Wireframe Chrome

- Applied feedback that the Browser navigation chrome was too black for the
  gray/white wireframe artifact.
- Changed Browser toolbar, toolbar menu, and page context menu surfaces to
  light gray/white styling.
- Added a dedicated `commentPlus` icon and used it for the Annotate toolbar
  control.
- Bumped document-relative CSS/JS asset references to `batch5-r41`.
- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static scan found no root-relative links in the edited review files.
- In-app Browser QA used local server `http://127.0.0.1:4195/`.
- In-app Browser verified `#browser-navigation` has toolbar background
  `rgb(247, 247, 247)`, `blackChrome: false`, `iamawesome.com`, all six
  toolbar controls, and an Annotate icon using the comment-plus path.
- In-app Browser verified `#browser-menu` and
  `#browser-page-context-menu` use light gray/white menu panels, retain the
  expected menu text, and show no console errors.

## Review Round 41 Browser Menu Access

- Applied feedback that Browser navigation was missing visible access to the
  Browser context menu and page right-click context menu.
- Wired the Browser toolbar menu button in `#browser-navigation` to navigate to
  `#browser-menu`.
- Wired right-click on the Browser document page in `#browser-navigation` to
  navigate to `#browser-page-context-menu`.
- Added `data-browser-document` to scope the right-click handler to the Browser
  preview only.
- Bumped document-relative CSS/JS asset references to `batch5-r42`.
- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static scan found no root-relative links in the edited review files.
- In-app Browser QA used local server `http://127.0.0.1:4195/`.
- In-app Browser verified `#browser-navigation` starts without open menus,
  clicking the three-dot Browser menu opens `#browser-menu` with expected menu
  text, right-clicking the Browser document opens
  `#browser-page-context-menu` with expected page menu text, and no console
  errors were reported.

## Review Round 42 Browser Menu Typography

- Applied feedback that Browser context menu and right-click context menu type
  was too large for the r05 wireframe typography.
- Reduced Browser menu item type to `13px`.
- Set Browser menu containers to `var(--font-ui)` so the page context menu no
  longer inherits the Browser document serif type.
- Reduced menu row height, divider spacing, and zoom-control type/height.
- Bumped document-relative CSS/JS asset references to `batch5-r43`.
- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static scan found no root-relative links in the edited review files.
- In-app Browser QA used local server `http://127.0.0.1:4195/`.
- In-app Browser verified `#browser-menu` and
  `#browser-page-context-menu` both use `13px` `var(--font-ui)` menu
  typography, retain the expected menu text, and show no console errors.

## Review Round 43 Browser Menu Weight

- Applied feedback to remove bold item text from the Browser navigation context
  menu and right-click context menu.
- Set shared Browser menu item weight to `400` while preserving `13px`
  `var(--font-ui)` typography.
- Bumped document-relative CSS/JS asset references to `batch5-r44`.
- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static scan found no root-relative links in the edited review files.
- In-app Browser QA used local server `http://127.0.0.1:4195/`.
- In-app Browser verified `#browser-menu` opens from `#browser-navigation` and
  computes Browser menu item typography as `13px` with `font-weight: 400`.
- In-app Browser verified right-clicking the Browser document opens
  `#browser-page-context-menu` and computes page menu item typography as
  `13px` with `font-weight: 400`.
- In-app Browser console error log was empty during the affected-route checks.

## Review Round 44 Chat Debug r04 Correction

- Applied feedback that Agent/Chat Debug should be based on the r04
  command/results panel rather than opening on the disabled-entry card.
- Changed Debug header icon routing to `#debug`.
- Reworked active Debug content as a command log plus visible result rows for
  CLI command, tool call, tool result, and approval result.
- Tightened `#debug-disabled-entry` so the status pill and settings button do
  not stretch into oversized shapes.
- Added `#debug` to the Batch 5 route hub.
- Bumped document-relative CSS/JS asset references to `batch5-r45`.
- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static scan found no root-relative links in the edited review files.
- In-app Browser QA used local server `http://127.0.0.1:4195/`.
- In-app Browser verified clicking the Debug header icon from
  `#shell-foundation` opens `#debug`, not `#debug-disabled-entry`.
- In-app Browser verified `#debug` renders `.agent-debug-console`, command log
  text including `npm test -- --runInBand`, `tool_call_requested`,
  `tool_output_delta`, and `approval_policy`, plus four result rows.
- In-app Browser verified the four result rows include `terminal.run`,
  `browser.screenshot`, `browser.annotation.created`, and `approval_policy`.
- In-app Browser verified `#debug-disabled-entry` remains available with a
  compact status pill (`141x27`) and settings button (`189x36`).
- In-app Browser console error log was empty during the affected-route checks.

## Review Round 45 Chat Debug Label Removal

- Applied feedback that the active Agent/Chat Debug wireframe still contained
  annotation-like record type labels.
- Removed visible `CLI command`, `Tool call`, `Tool result`, and `Approval`
  labels from `#debug`.
- Replaced the active Debug result rows with realistic `terminal.run` and
  `browser.screenshot` debug records.
- Added visible JSON-like parameter and result payloads for both records.
- Bumped document-relative CSS/JS asset references to `batch5-r46`.
- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static scan found no root-relative links in the edited review files.
- In-app Browser QA used local server `http://127.0.0.1:4195/`.
- In-app Browser verified `#debug` renders `.agent-debug-console` with two
  result rows.
- In-app Browser verified the active Debug panel contains `terminal.run`, the
  `npm test -- --runInBand` command, command parameters, `exitCode: 0`, and
  `stdout: 18 tests passed`.
- In-app Browser verified the active Debug panel contains `browser.screenshot`,
  screenshot parameters, and returned screenshot metadata for
  `pricing-page.png`.
- In-app Browser verified the scoped Debug panel no longer visibly contains
  `CLI command`, `Tool call`, `Tool result`, or `Approval`.
- In-app Browser console error log was empty during the affected-route check.

## Review Round 46 Chat Debug Secondary States

- Applied feedback to fix `#debug-disabled-entry` and `#debug-timeline`
  without adding backend/background functionality screens.
- Kept `#debug-disabled-entry` as a compact off state with a settings path.
- Removed the extra disabled notice from the center composer dock.
- Reworked `#debug-timeline` into a distinct run-history selector with current
  and historical run rows plus selected-run event rows.
- Bumped document-relative CSS/JS asset references to `batch5-r48`.
- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static scan found no root-relative links in the edited review files.
- In-app Browser QA used local server `http://127.0.0.1:4195/`.
- In-app Browser verified `#debug-disabled-entry` at
  `?qa=r48-final#debug-disabled-entry` shows `Debug off`, `Off`, and
  `Debug records are hidden for this chat.`
- In-app Browser verified `#debug-disabled-entry` no longer contains
  `Disabled by default`, the old diagnostic explanation copy, the duplicated
  disabled notice, retention wording, or backend/background wording.
- In-app Browser verified `#debug-timeline` at
  `?qa=r48-final#debug-timeline` renders three run cards and four selected-run
  event rows with `terminal.run`, `browser.screenshot`, and `approval_policy`.
- In-app Browser verified `#debug-timeline` does not render
  `.agent-debug-console`, keeping it distinct from the active `#debug` console.
- In-app Browser console error log was empty during the affected-route checks.

## Review Round 47 Remove Chat Debug Off Route

- Applied feedback that `#debug-disabled-entry` is background plugin state and
  should not be represented as a wireframe panel.
- Removed `#debug-disabled-entry` from live route wiring, Batch 5 link hub, and
  README route table.
- Removed the unused disabled panel component and disabled notice.
- Moved disabled-by-default Chat Debug into the spec/settings behavior coverage
  row.
- Bumped document-relative CSS/JS asset references to `batch5-r49`.
- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static scan found no live `#debug-disabled-entry` references in rendered
  HTML/CSS/JS, README route table, or Batch 5 link hub.
- In-app Browser QA used local server `http://127.0.0.1:4195/`.
- In-app Browser verified `?qa=r49-final#debug-disabled-entry` falls back to
  the default shell with no `.debug-disabled-panel`, no `.debug-enable-notice`,
  no Chat Debug off copy, and no link to `#debug-disabled-entry`.
- In-app Browser verified `?qa=r49-final#debug` still renders the active
  `.agent-debug-console` with command/tool result content.
- In-app Browser verified `?qa=r49-final#debug-timeline` still renders
  `.debug-run-history` with current/historical run rows and selected events.
- In-app Browser verified `?qa=r49-final#coverage` no longer lists
  `#debug-disabled-entry` and now treats disabled-by-default plugin visibility
  as spec/settings behavior.
- In-app Browser console error log was empty during the affected-route checks.

## Review Round 48 Move Browser Annotation Flow To Markdown

- Applied feedback to remove `#browser-annotations`, remove annotations from
  `#browser-preview-host`, and move annotation behavior into Markdown.
- Removed `#browser-annotations` from live route wiring, Batch 5 link hub, and
  README route table.
- Removed rendered annotation-marker/comment UI and related CSS.
- Removed the annotation control from `#browser-preview-host`.
- Added `browser-annotations.md` as the markdown-only note for annotation
  behavior.
- Replaced visible annotation sample data in prompt attachments and Debug event
  detail with file/screenshot/evidence samples.
- Bumped document-relative CSS/JS asset references to `batch5-r50`.
- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static scan found no live `#browser-annotations` links in rendered
  HTML/CSS/JS, README route table, or Batch 5 link hub.
- Static scan found no rendered annotation marker/comment component names in
  HTML/CSS/JS.
- Filesystem check verified `browser-annotations.md` contains the markdown-only
  annotation behavior note.
- In-app Browser QA used local server `http://127.0.0.1:4195/`.
- In-app Browser verified `?qa=r50-final#browser-annotations` falls back to the
  default shell with no annotation route content, no annotation classes, and no
  route link.
- In-app Browser verified `?qa=r50-final#browser-preview-host` renders the
  PDF/document preview host with one Screenshot control, no `Annotate` toolbar
  control, no annotation text, and no annotation classes.
- In-app Browser verified `?qa=r50-final#coverage` no longer lists
  `#browser-annotations` and treats Browser evidence capture as markdown/spec
  behavior.
- In-app Browser verified `?qa=r50-final#attachment-states` uses file and
  Browser screenshot attachment records without annotation content.
- In-app Browser verified `?qa=r50-final#debug-event-detail` uses
  `browser.screenshot.captured` sample data without annotation content.
- In-app Browser console error log was empty during the affected-route checks.

## Review Round 49 Remove Preview Host Helper Labels

- Applied screenshot feedback that `#browser-preview-host` still contained
  helper/callout text.
- Removed `Browser-native PDF preview` from the preview sheet.
- Removed `DOCX/XLSX rendered by document-family plugins` and
  `Browser hosts rendered output` from the rendered preview host.
- Removed unused document-boundary strip CSS.
- Added the removed helper text to `browser-annotations.md` as Markdown-only
  review context.
- Bumped document-relative CSS/JS asset references to `batch5-r51`.
- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static scan found no root-relative links in the edited review files.
- Static scan found the removed helper/callout text only in companion Markdown
  and review notes, not in rendered `index.html`, `script.js`, or `styles.css`.
- In-app Browser QA used local server `http://127.0.0.1:4195/`.
- In-app Browser verified
  `?qa=r51-final#browser-preview-host` renders with no
  `Browser-native PDF preview`, no
  `DOCX/XLSX rendered by document-family plugins`, no
  `Browser hosts rendered output`, and no `.document-boundary-strip`.
- In-app Browser verified `#browser-preview-host` still has one Screenshot
  control, no Annotate control, the `Q4 Partner Brief.pdf` preview title, and
  seven PDF line placeholders.
- In-app Browser console error log was empty during the affected-route check.

## Review Round 50 Debug Event Detail Cleanup

- Applied screenshot feedback that `#debug-event-detail` still contained
  annotation-like header and footer text.
- Removed the `Typed event` pill from `#debug-event-detail`.
- Replaced the `Back to timeline` text link with an icon-only X control that
  links to `#debug-timeline`.
- Removed the bottom no-export callout from `#debug-event-detail`.
- Updated the route subtitle and coverage row wording to avoid visible
  `Typed event` and no-export callout copy.
- Bumped document-relative CSS/JS asset references to `batch5-r52`.
- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static scan found no root-relative links in the edited review files.
- Static scan found no `Typed event`, `Back to timeline`,
  `No export action is available`, or `.debug-no-export` text/class in
  rendered `index.html`, `script.js`, or `styles.css`.
- In-app Browser QA used local server `http://127.0.0.1:4195/`.
- In-app Browser verified
  `?qa=r52-final#debug-event-detail` renders with no `Typed event`, no
  visible `Back to timeline`, no bottom no-export callout, and no status pill
  inside the detail panel.
- In-app Browser verified the event detail header has one icon-only X control
  linking to `#debug-timeline`.
- In-app Browser clicked the X control and verified it navigates to
  `?qa=r52-final#debug-timeline` with the timeline visible.
- In-app Browser returned to `?qa=r52-final#debug-event-detail` for review.
- In-app Browser console error log was empty during the affected-route checks.
