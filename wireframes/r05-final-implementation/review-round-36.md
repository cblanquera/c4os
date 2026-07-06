# Review Round 36

Date: 2026-07-04

## Scope

Batch 5 runtime plugin panel wireframes for specs 08, 09, and 10, plus directly
affected spec 04 runtime/tool policy overlap.

## Changes

- Added Terminal routes:
  - `#terminal-user-pty`
  - `#terminal-lifecycle`
  - `#terminal-cleanup`
  - `#terminal-settings`
  - `#terminal-scrollback`
- Added Browser routes:
  - `#browser-navigation`
  - `#browser-annotations`
  - `#browser-clear-after-send`
  - `#browser-preview-host`
  - `#browser-state-hydration`
  - `#browser-security-boundary`
- Added Chat Debug routes:
  - `#debug-disabled-entry`
  - `#debug-timeline`
  - `#debug-event-detail`
  - `#debug-retention`
- Updated `#coverage` with Batch 5 rows for specs 08, 09, 10, and direct 04
  overlap.
- Updated the r05 artifact README to list Batch 5 as a pending review round.
- Added a visible Browser action-state row for Back / Forward / Refresh after
  requirement-audit QA found those controls were otherwise icon-only.

## Review Focus

- Panel separation between Terminal user PTY, runtime terminal tool output, and
  Chat Debug visibility.
- Browser evidence capture and prompt attachment behavior, including
  clear-after-send.
- Chat Debug redaction clarity, retention limits, and no-export behavior.
- Runtime/tool visibility when Browser state is created by an invisible runtime
  action and later hydrated into a compatible Browser view.

## Verification

- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static link scan found no root-relative links in the Batch 5 review files.
- In-app Browser blocked direct `file://` review, so a temporary local server
  was started at `http://127.0.0.1:4191/`.
- In-app Browser route sweep passed for all 16 Batch 5 routes, including
  `#coverage`.
- Representative Browser screenshots were nonblank for:
  - `#terminal-user-pty`
  - `#browser-annotations`
  - `#debug-event-detail`
  - `#coverage`
- Browser click QA passed:
  - Terminal header icon opens `#terminal-user-pty`.
  - Active Terminal header icon closes back to `#shell-foundation`.
  - Browser header icon opens `#browser-navigation`.
  - Debug header icon opens `#debug-disabled-entry`.
  - Timeline inspect link opens `#debug-event-detail`.
- Browser console error log was empty during the click QA pass.
- Batch 5 link hub rendered with 16 document-relative links, and clicking
  `./index.html#browser-annotations` landed on the SPA route with two active
  Browser markers.
- Browser layout pass across all 16 Batch 5 routes found no overflowing
  button, link, chip, status pill, or Debug tab text.
- Follow-up requirement audit found `#browser-navigation` needed visible action
  text for Back / Forward / Refresh; the route was patched and re-queued for
  final verification.
- Fresh-load in-app Browser requirement audit passed all 15 focused Batch 5
  routes after the patch, with no rendered review-note, TODO, or implementation
  note terms.
