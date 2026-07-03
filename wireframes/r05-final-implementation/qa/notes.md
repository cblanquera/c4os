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
- `./index.html#coverage`
