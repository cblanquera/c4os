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
- `./index.html#coverage`
