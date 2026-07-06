# Review Round 47

Date: 2026-07-06

## Scope

Remove the Chat Debug off panel route because disabled plugin state is not a
wireframeable runtime panel surface.

## Changes

- Removed `#debug-disabled-entry` from route wiring.
- Removed the disabled Chat Debug panel component and unused disabled notice.
- Removed `#debug-disabled-entry` from the Batch 5 link hub and README route
  table.
- Moved disabled-by-default Chat Debug coverage into spec/settings behavior.
- Bumped the document-relative CSS/JS cache token to `batch5-r49`.

## Review Focus

- Confirm the remaining Chat Debug routes are `#debug`, `#debug-timeline`, and
  `#debug-event-detail`.
- Confirm disabled-by-default Chat Debug is no longer represented as a right
  plugin-panel wireframe.

## Verification

- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static scan found no live `#debug-disabled-entry` references in rendered
  HTML/CSS/JS, README route table, or Batch 5 link hub.
- In-app Browser QA verified `?qa=r49-final#debug-disabled-entry` falls back to
  the default shell and does not render a Chat Debug off panel.
- In-app Browser QA verified `#debug`, `#debug-timeline`, and `#coverage`
  still render without console errors.
