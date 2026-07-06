# Review Round 37

Date: 2026-07-04

## Scope

Batch 5 correction after feedback that implied/backend runtime behavior should
not be displayed as standalone wireframe screens.

## Changes

- Reduced Terminal review to `#terminal-user-pty`.
- Reworked `#terminal-user-pty` as the r04 Terminal output pane with the agent
  debug/results pane removed.
- Removed Terminal lifecycle, cleanup, settings, and scrollback routes from the
  route map and link hub.
- Removed Browser clear-after-send, Browser state hydration, Browser security
  boundary, and Chat Debug retention from the route map and link hub.
- Removed the explanatory runtime-state card from Batch 5 center chat routes.
- Updated `#coverage` so non-visual backend/runtime requirements are marked as
  spec-only behavior instead of visible routes.
- Updated `README.md` and `notes.md` to describe the corrected route set.

## Review Focus

- Does `#terminal-user-pty` now match the expected r04 terminal minus the agent
  debug pane?
- Are the remaining Browser and Chat Debug routes concrete visible UI states?
- Does `#coverage` clearly separate visible route evidence from spec-only
  backend/runtime requirements?

## Verification

- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static scan found no stale removed Batch 5 route names in current review
  files: `script.js`, `batch-5-links.html`, `README.md`, `notes.md`, and this
  round file.
- Static scan found no root-relative links in the revised r05 review files.
- Local static server ran at `http://127.0.0.1:4191/`.
- In-app Browser verified the reduced 8-link hub and all 8 current routes:
  `#terminal-user-pty`, `#browser-navigation`, `#browser-annotations`,
  `#browser-preview-host`, `#debug-disabled-entry`, `#debug-timeline`,
  `#debug-event-detail`, and `#coverage`.
- In-app Browser confirmed no stale removed routes appear in the link hub, no
  runtime-state cards render on Batch 5 routes, and `#terminal-user-pty` does
  not show the old r04 agent results pane or technical `User PTY` label.
- In-app Browser click QA passed for the link hub Terminal route, Terminal
  header close, Browser header open, Debug header open, and Debug inspect link.
- In-app Browser console error log was empty during click QA.
- Mobile viewport sweep at 390x844 passed all 8 current routes with no
  document overflow or visible control text overflow after excluding the hidden
  skip link.
