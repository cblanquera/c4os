# Review Round 48

Date: 2026-07-06

## Scope

Remove the standalone Browser annotation route and keep annotation behavior in
Markdown instead of the rendered review artifact.

## Changes

- Removed `#browser-annotations` from live route wiring.
- Removed the Browser annotation link from the Batch 5 link hub and README
  route table.
- Removed rendered annotation marker/comment UI and related CSS.
- Removed the annotation toolbar control from `#browser-preview-host`.
- Added `browser-annotations.md` for markdown-only annotation behavior notes.
- Replaced visible annotation sample data in prompt attachments and Debug event
  detail with file/screenshot/evidence examples.
- Bumped the document-relative CSS/JS cache token to `batch5-r50`.

## Review Focus

- Confirm `#browser-annotations` is no longer a rendered review state.
- Confirm `#browser-preview-host` only shows document preview hosting and no
  annotation control.
- Confirm markdown-only annotation behavior is captured in
  `browser-annotations.md`.

## Verification

- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static scan found no live `#browser-annotations` links in rendered
  HTML/CSS/JS, README route table, or Batch 5 link hub.
- Static scan found no rendered annotation marker/comment component names in
  HTML/CSS/JS.
- Filesystem check verified `browser-annotations.md` contains the markdown-only
  annotation behavior note.
- In-app Browser QA verified `?qa=r50-final#browser-annotations` falls back to
  the default shell.
- In-app Browser QA verified `?qa=r50-final#browser-preview-host` renders the
  PDF/document preview host with no `Annotate` toolbar button and no annotation
  text/classes.
- In-app Browser QA verified `?qa=r50-final#coverage`,
  `?qa=r50-final#attachment-states`, and `?qa=r50-final#debug-event-detail`
  no longer show annotation content.
- In-app Browser console error log was empty during the affected-route checks.
