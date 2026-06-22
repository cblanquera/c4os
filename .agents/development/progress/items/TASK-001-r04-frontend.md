# TASK-001 r04 Frontend

Status: verified
Updated: 2026-06-22

## Source

- Task: `.agents/specs/mvp/tasks.md` `TASK-001`
- Frozen spec: `.agents/specs/mvp/status.md`
- Traceability: `.agents/specs/mvp/traceability.md`
- Wireframes: `wireframes/r04-single-page-app/` and
  `wireframes/ui-handoff-spec.md`
- Context: `.agents/context/product-specs.md`,
  `.agents/context/technical-specs.md`, `.agents/context/creative-specs.md`,
  `.agents/context/work-orders.md`

## Goal

Build the full r04 frontend in `frontend/` as the actual product shell
frontend, ready to connect to mocked server behavior in `TASK-002`.

TASK-001 is not a copy/paste exercise. Its purpose is to turn the accepted r04
product shape into a production-quality frontend foundation: preserve r04
behavior, routes, settings structure, and information architecture, then improve
the visual presentation enough that the result feels like C4OS product software
rather than a grayscale planning artifact.

## Linked Requirements

REQ-001, REQ-003, REQ-008, REQ-010, REQ-011, REQ-013, REQ-014, REQ-015,
REQ-016

## Linked Acceptance

AC-001, AC-013, AC-014, AC-015, AC-016, AC-017, AC-018, AC-019, AC-020, AC-021

Additional frontend foundation gates: AC-022, AC-023, AC-024, AC-025

## Expected Files And Folders

- `frontend/`
- Frontend behavior tests using filenames such as `tests/frontend-*.test.*`
  when needed for rendered browser verification.
- Frontend-only support harness files under `tests/support/` when needed to run
  TASK-001 verification locally.
- Minimal `package.json` script updates when needed to expose TASK-001 frontend
  dev, test, or acceptance commands.
- `.agents/development/progress/items/TASK-001-r04-frontend.md`

No backend activation is expected in this item. Do not edit `backend/` or
`tests/server/`. If implementation uncovers a backend or mock-server concern,
record it as a blocker or next-task note in this progress item instead of
adding code outside the TASK-001 frontend scope.

## Work Instructions

- Build from r04 wireframes and `wireframes/ui-handoff-spec.md`.
- Treat `wireframes/r04-single-page-app/` as the functional baseline. Port r04
  screen structure and working interactions first; do not redesign from
  scratch.
- After functional parity is working, apply a production visual treatment while
  preserving the same routes, controls, settings structures, and interactions.
  Improve hierarchy, typography, spacing, color, panel surfaces,
  Browser/Files/Terminal polish, and button/control treatment without inventing
  product UI.
- Keep the frontend aligned with the accepted creative and product context.
- Use visible states for trust, provider/model setup, files, structured thread,
  Browser, Terminal, settings IA, extensions, concurrency, resume, local memory,
  action records, audit records, and artifact preview where those surfaces are
  part of r04 or required by acceptance.
- Preserve every r04 route and screen-specific settings layout. Providers, Add
  Provider, Models, Runtimes, Configuration, Plugins, Skills, and MCP Servers
  must not be replaced by generic cards, generic lists, or invented extension
  metadata surfaces.
- Preserve working r04 interactions: left/right panel collapse, left/right
  panel resize, Terminal bottom-panel resize, attachment preview, approval and
  branch popovers, provider/model popovers, message show/hide, plugin
  marketplace and connection dialogs, skill detail dialog, MCP add-server
  dialog, and MCP transport switching.
- Do not add new visible controls or extra surfaces that were not in r04 or the
  frozen MVP contract. Record proposed UI changes for review instead of
  silently including them in TASK-001.
- Do not leave the final TASK-001 output looking like the r04 grayscale
  wireframe. r04 visual styling is reference material for layout and hierarchy,
  not the finished product treatment.
- Keep data mocked or local to the frontend only as needed for visual and state
  completeness; do not claim real backend, runtime, filesystem, Browser,
  Terminal, extension, security, approval, or persistence behavior in this item.
- Keep any new test or harness files focused on frontend rendering,
  interaction, screenshot, and acceptance verification for TASK-001. Do not
  create a mock server harness, backend integration layer, or `tests/server/`
  content.
- Do not pause for user review during this item. Continue until the frontend is
  coherent, maintainable, and ready for `TASK-002`.

## Non-Goals

- Real backend behavior.
- Real provider or runtime integration.
- Real filesystem, Browser, Terminal, extension, security, approval, or
  persistence behavior.
- Mock server harness.
- Backend integration tests or server behavior under `tests/server/`.
- `src-tauri/`.
- New controls, filters, metadata cards, settings abstractions, or alternate
  route structures not present in r04 or the frozen MVP contract.
- Structural redesign. Production visual treatment is required; changing the
  product flow, controls, or information architecture is not.

## Verification

Run these before marking this item `review`, `done`, or `verified`:

- `git diff --check`
- `npm run dev` for local browser review when a frontend dev surface exists.
- Browser automation that renders every r04 route and verifies structure,
  control placement, and settings screen parity against
  `wireframes/r04-single-page-app/`.
- Browser automation that clicks every collapse control, opens every popover
  and dialog listed in Work Instructions, toggles message/detail states, and
  drags every resize handle far enough to prove dimensions change.
- Screenshot review across all routes that confirms the frontend preserves r04
  structure but no longer reads as a grayscale wireframe.
- `npm test` with rendered behavior coverage. Source-string checks alone are
  not sufficient.
- `npm run acceptance:mvp` once the acceptance script can evaluate the frontend
  surface and includes behavior-based checks for AC-022, AC-023, AC-024, and
  AC-025.

Record skipped commands with the reason in this file before marking the item
`review`, `done`, or `verified`.

## Next Step

Proceed to `TASK-002` mock server connection using the TASK-001 frontend routes,
screen structure, and interaction tests as the baseline.

## Implementation Notes

- Created the production frontend shell in `frontend/` as a no-build vanilla
  SPA with the full r04 route inventory.
- Preserved App Start, New Session, Chat Session, provider/model popovers,
  Files explorer/editor, Terminal, Providers, Add Provider, Models, Runtimes,
  Configuration, Plugins, Skills, and MCP Servers.
- Preserved r04 interactions for side-panel collapse, side-panel resize,
  Terminal bottom-panel resize, attachment preview, approval/branch popovers,
  provider/model popovers, message disclosure, plugin connection and
  marketplace dialogs, skill detail dialog, MCP add-server dialog, and MCP
  transport switching.
- Applied a restrained production desktop-app treatment with off-white canvas,
  graphite/green rails, indigo/teal accents, denser tool surfaces, and code/
  terminal styling while preserving r04 information architecture.
- Added frontend-only browser harness and rendered behavior coverage under
  `tests/`.
- Screenshot evidence for all r04 routes is generated under
  `tests/support/task-001-screenshots/`.

## Verification Notes

- `node --test tests/frontend-r04.test.js`: pass, 9 rendered behavior tests.
  First sandboxed run was blocked by localhost `EPERM`; reran with the correct
  local-browser execution boundary.
- `npm test`: pass, 9 rendered behavior tests. npm emitted an existing
  user-config warning for `python`.
- `npm run dev`: pass, served frontend at `http://127.0.0.1:3000`; HTTP smoke
  returned `200 OK`; server was stopped after verification.
- `git diff --check`: pass.
- Browser automation rendered every r04 route, exercised required popovers,
  dialogs, collapse controls, message state, and resize handles, and captured
  screenshot evidence.
- `npm run acceptance:mvp`: skipped because `package.json` points to
  `scripts/mvp-acceptance.js`, but that script is not present and cannot
  evaluate TASK-001 yet.
- Repair pass after visual review: normalized settings form typography and
  spacing, removed the MCP dialog `Docs` header action, made MCP modal headers
  sticky, wired MCP add/remove row controls, hid non-required provider fields
  for non-OpenAI-compatible provider types, tightened the file editor line
  number gutter, made editor code cells contenteditable, and added rendered
  regression coverage for these behaviors.
- Repair verification: `node --test tests/frontend-r04.test.js` pass, 14
  rendered behavior tests; live Playwright check against
  `http://127.0.0.1:3000/#settings-mcp` confirmed no `Docs` text in the MCP
  dialog, sticky header, normal input font weight, hidden STDIO fields after
  Streamable HTTP selection, and one initial HTTP header row.

## Strict QA Pass - 2026-06-22

QA status: pass after scoped r04 parity repairs.

Fixes made:

- Restored r04 plugin connection dialog surfaces for source/target connection,
  control, risk, data-shared blocks, and Advanced settings action.
- Restored r04 skill detail dialog surfaces for skill icon, type label,
  enabled switch, More actions control, summary, split detail body, uninstall,
  and Try in chat actions.
- Restored the MCP Servers route-level Learn more action alongside Add server.
- Expanded rendered Playwright coverage so the plugin dialog, skill detail
  dialog, and MCP header action fail if these r04 surfaces disappear.
- Captured extra modal screenshots under
  `tests/support/task-001-screenshots/` for the repaired plugin and skill
  dialogs.

Verification results:

- `npm test`: pass, 14 rendered behavior tests. npm emitted the existing
  user-config warning for `python`.
- `node --test tests/frontend-r04.test.js`: pass, 14 rendered behavior tests.
- `node --check frontend/app.js`: pass.
- `git diff --check`: pass.
- Live Playwright route check: pass for every r04 route at 1440, 1100, and
  980 pixel viewport widths; route main surfaces remained visible.
- Live modal screenshot check: pass for repaired plugin connection and skill
  detail dialogs.
- `npm run dev`: default port 3000 was unavailable with `EADDRINUSE`; reran
  with `PORT=3001`, served `http://127.0.0.1:3001/`, HTTP smoke returned
  `200 OK`, and the server was stopped after verification.
- `npm run acceptance:mvp`: unavailable. `package.json` points to
  `scripts/mvp-acceptance.js`, but that script is not present, so it cannot
  evaluate TASK-001 yet.

Integration readiness notes:

- The frontend route inventory, shell state, composer state, settings route
  shapes, dialogs, popovers, resize handlers, and fixture arrays are structured
  enough for TASK-002 to replace local fixtures with mock-server state without
  changing the accepted r04 route structure.
- Fixture-only areas still include workspace/project/session records,
  provider/model rows, file rows, editable code content, terminal output,
  Browser preview state, plugin records, skill records, MCP rows, approval
  state, branch/model context, attachment state, switches, and dialog actions.
- These cannot be prepared further in TASK-001 without inventing backend or
  mock-server contracts for persistence, provider discovery, filesystem
  authority, Browser state, terminal execution, extension installation,
  approvals, secure credential handling, or runtime effects.

Review feedback repair:

- Fixed the expanded chat transcript visual stack so the agent message pushes
  Tool Call and Agent Run cards down cleanly, with more reliable vertical
  separation and less oversized message minimum height.
- Rebuilt the plugin connection dialog to match the r04 wireframe structure:
  centered `AI ... GH` connection graphic, centered title/status, one stacked
  bordered review panel with dividers, full-width `Continue to GitHub` action,
  text-style Advanced settings action, and a simpler top-right close control.
- Updated rendered regression coverage for expanded chat card spacing and the
  r04 plugin modal structure.
- Refreshed review screenshots:
  `tests/support/task-001-screenshots/chat-session-expanded.png` and
  `tests/support/task-001-screenshots/settings-plugin-connect-modal.png`.

Review feedback verification:

- `npm test`: pass, 14 rendered behavior tests.
- `node --test tests/frontend-r04.test.js`: pass, 14 rendered behavior tests.
- `node --check frontend/app.js`: pass.
- `git diff --check`: pass.

Second review feedback repair:

- Fixed the constrained-width chat bubble overlap by changing the transcript
  stack from CSS grid to flex column and preventing message/activity flex items
  from shrinking below their content height.
- Preserved user/agent alignment after the transcript stack change by moving
  bubble alignment to flex-compatible `align-self` rules.
- Added rendered regression coverage for constrained-width expanded chat
  bubbles so activity cards cannot cover the Show More/Show Less control or
  overlap the expanded message body.
- Fixed right-panel resize clipping by capping side-panel resize width against
  the live viewport, other side panel width, resize handles, and required
  center workbench width.
- Reduced the plugin connection modal width, padding, node sizes, title/status
  font sizes, review body text, primary action size, and secondary action
  scale so it follows the app's modal density while preserving the r04 modal
  structure.
- Added rendered regression coverage for right-panel resize visibility and
  plugin modal scale.
- Refreshed review screenshots:
  `tests/support/task-001-screenshots/chat-session-expanded-constrained.png`,
  `tests/support/task-001-screenshots/right-panel-resize-capped.png`, and
  `tests/support/task-001-screenshots/settings-plugin-connect-modal.png`.

Second review feedback verification:

- `npm test`: pass, 16 rendered behavior tests.
- `node --test tests/frontend-r04.test.js`: pass, 16 rendered behavior tests.
- `node --check frontend/app.js`: pass.
- `git diff --check`: pass.

Provider type reference repair:

- Rechecked `.agents/context/` and routed references for provider types.
- Confirmed the documented built-in provider profile types in
  `.agents/references/context/product-specs/feature-surfaces.md`,
  `.agents/references/context/product-specs/mvp-feature-list.md`, and
  `.agents/references/context/work-orders/decisions.md`: OpenAI, OpenRouter,
  Hugging Face router, LiteLLM proxy, and custom OpenAI-compatible endpoint.
- Updated the Add Provider `Provider Type` select to use that full list.
- Kept Provider Type, Label, API Base URL, API Key, Auth, and Headers in the
  r04 field order. Hosted built-ins hide URL/Auth/Headers because references
  say hidden provider inputs can be auto-filled for built-in types on save.
- Left URL/Auth/Headers visible for LiteLLM proxy and custom
  OpenAI-compatible endpoint because the current references only say `base URL
  where applicable` and do not define a full per-provider field visibility
  matrix.
- Added rendered coverage for the full provider type list and visibility rules.

Provider type repair verification:

- `npm test`: pass, 16 rendered behavior tests.
- `node --test tests/frontend-r04.test.js`: pass, 16 rendered behavior tests.
- `node --check frontend/app.js`: pass.
- `git diff --check`: pass.

Plugin marketplace picker repair:

- Rechecked the official OpenAI Codex plugin build docs for marketplace
  behavior. The docs define a marketplace as a selectable plugin-directory
  source with its visible label coming from marketplace metadata such as
  `interface.displayName`, and a single marketplace can contain one or many
  plugin entries.
- Updated the Plugins route marketplace popover so it lists the default
  marketplace source `Built by C4OS` as the selected marketplace instead of
  showing only an add action.
- Kept `Add Marketplace` as a secondary menu action with application menu
  styling, full-width hit area, and no native browser button chrome.
- Synced the marketplace trigger `aria-expanded` state and hid the menu before
  opening the Add plugin marketplace dialog.
- Added rendered regression coverage for the marketplace source list, selected
  state, and styled add action.

Plugin marketplace picker verification:

- `npm test`: pass, 17 rendered behavior tests.
- `node --test tests/frontend-r04.test.js`: pass, 17 rendered behavior tests.
- `node --check frontend/app.js`: pass.
- `git diff --check`: pass.
- Active review server smoke at `http://127.0.0.1:3001/`: pass, HTTP `200 OK`.

Maintainability pass:

- Applied the `chrisai-coding` maintainability, JavaScript, and HTML/CSS
  guidance as a second pass after behavior was already covered by tests.
- Split frontend fixture/catalog data from the renderer into `frontend/data.js`
  so TASK-002 can replace mocked records without digging through route
  rendering functions.
- Split SVG icon path data and DOM helpers into `frontend/icons.js` and
  `frontend/dom.js` so reusable primitives have a predictable home.
- Split stateful interaction binding into `frontend/interactions.js`, leaving
  `frontend/app.js` focused on route rendering and dialog markup.
- Added section comments and JSDoc-style function summaries for the main
  render, dialog, data, DOM, and interaction ownership boundaries.
- Added stylesheet section markers for global, shell, composer, chat, tool
  panel, settings, plugin/skill directory, and modal surfaces.
- Kept the public r04 route ids, DOM data hooks, dialog controls, and rendered
  behavior unchanged.

Maintainability verification:

- `node --check frontend/app.js`: pass.
- `node --check frontend/data.js`: pass.
- `node --check frontend/dom.js`: pass.
- `node --check frontend/icons.js`: pass.
- `node --check frontend/interactions.js`: pass.
- `git diff --check`: pass.
- `node --test tests/frontend-r04.test.js`: pass, 17 rendered behavior tests.
- `npm test`: pass, 17 rendered behavior tests.
- Active review server browser smoke at `http://127.0.0.1:3001/#settings-plugins`:
  pass, no page errors and marketplace menu rendered.

User acceptance:

- 2026-06-22: Passed user review after strict visual, responsive, modal,
  marketplace, provider-field, and maintainability feedback rounds.
- TASK-001 is verified as the frontend foundation baseline for TASK-002.

Carry-forward guardrail:

- Later implementation tasks may replace frontend-local static fixtures and
  static component behavior with dynamic, backend-backed, or working
  equivalents behind the accepted TASK-001 UI.
- Later tasks must not add visible UI components, controls, panels, menus,
  cards, settings abstractions, or route structures unless the change is
  explicitly accepted and documented before implementation.
- If dynamic integration pressure reveals that a new visible UI component is
  needed, record it as a review item instead of silently adding it.

## Remaining Mocks

- All route data, provider/model rows, file rows, terminal output, browser
  preview, plugin records, skill records, MCP rows, approval state, and
  attachment state are frontend-local fixtures only.
- No real backend, runtime, filesystem, Browser, Terminal, extension, security,
  approval, provider call, persistence, or mock-server behavior is claimed in
  TASK-001.
