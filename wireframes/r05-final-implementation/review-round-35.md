# Review Round 35

Date: 2026-07-04

## User Feedback

- Remove visible annotation copy from `#workspace-missing-project`.
- Keep the wireframe surface product-like instead of explaining behavior inside
  the shell.

## Changes

- Removed the center-panel `read-only-banner` from
  `#workspace-missing-project`.
- Removed the unused `read-only-banner` styling.
- Left the missing-project sidebar state and project action menu unchanged.

## Verification

- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Source scan found no remaining `read-only-banner` class or
  `moved-marketing-site chats are visible` annotation string in active
  JS/CSS.
- Rendered `#workspace-missing-project` with Chromium and confirmed the center
  prompt has only the heading and composer, with no read-only banner.
- Rendered check also confirmed the missing-project menu still shows
  `Relocate`, `Copy path`, `Rename`, `Remove`.
