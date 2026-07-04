# Review Round 34

Date: 2026-07-03

## User Feedback

- `#workspace-missing-project` project actions menu had too much padding.
- Menu text needed to align left.
- `Copy path` was missing an icon.
- Missing-project menu behavior must be: `Relocate`, `Copy path`, `Rename`, `Remove`.
- Found-project menu behavior must be: `Reveal`, `Copy path`, `Rename`, `Remove`.

## Changes

- Added a `copy` icon and applied it to `Copy path`.
- Removed `Reveal` from missing-project menus.
- Kept `Reveal` only for found-project menus.
- Tightened project action popover padding and row height.
- Forced project action rows to left-align text and icon/text columns.

## Verification

- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Rendered `#workspace-missing-project` with Chromium and confirmed:
  - Missing project menu: `Relocate`, `Copy path`, `Rename`, `Remove`.
  - Found project menu: `Reveal`, `Copy path`, `Rename`, `Remove`.
  - Every menu row has one icon.
  - Menu row text aligns left.
  - Menu padding is `4px`.
  - `Homepage refresh` and `Launch copy` chat rows remain present.
