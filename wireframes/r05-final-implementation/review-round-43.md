# Review Round 43

Date: 2026-07-06

## Scope

Focused weight correction for Browser toolbar and page context menus.

## Changes

- Removed bold styling from Browser context menu item text.
- Kept the r05 compact menu typography from Review Round 42: `13px`
  `var(--font-ui)`.
- Bumped the document-relative CSS/JS cache token to `batch5-r44`.

## Review Focus

- Do the Browser toolbar context menu and page right-click context menu now
  match the regular-weight r05 wireframe typography?
- Does the menu hierarchy still read clearly without bold item labels?

## Verification

- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static scan found no root-relative links in the edited r05 review files.
- In-app Browser QA ran through the local review server at
  `http://127.0.0.1:4195/`.
- In-app Browser verified `#browser-menu` opens from `#browser-navigation` and
  uses `13px` `var(--font-ui)` menu typography with `font-weight: 400`.
- In-app Browser verified `#browser-page-context-menu` opens from right-click
  on the Browser document and uses `13px` `var(--font-ui)` menu typography with
  `font-weight: 400`.
- In-app Browser console error log was empty during the affected-route checks.
