# Review Round 41

Date: 2026-07-06

## Scope

Focused correction for missing Browser menu access from the Browser navigation
review state.

## Changes

- Wired the Browser toolbar menu button in `#browser-navigation` to open the
  Browser menu state.
- Wired right-click on the Browser document page in `#browser-navigation` to
  open the page context menu state.
- Added a `data-browser-document` marker to the Browser document surface so the
  right-click interaction is scoped to the Browser preview.
- Bumped the document-relative CSS/JS cache token to `batch5-r42`.

## Review Focus

- From `#browser-navigation`, does clicking the three-dot Browser menu open the
  Browser context menu?
- From `#browser-navigation`, does right-clicking the Browser document page
  open the page context menu?
- Do both menu states remain gray/white and consistent with the r05 wireframe
  format?

## Verification

- `node --check wireframes/r05-final-implementation/script.js` passed.
- `git diff --check -- wireframes/r05-final-implementation` passed.
- Static scan found no root-relative links in the edited r05 review files.
- In-app Browser QA ran through the local review server at
  `http://127.0.0.1:4195/`.
- In-app Browser verified `#browser-navigation` loads `batch5-r42` assets and
  starts with no open Browser menu or page context menu.
- In-app Browser clicked the Browser menu button and verified it navigates to
  `#browser-menu` with the expected Browser menu text.
- In-app Browser right-clicked the Browser document page and verified it
  navigates to `#browser-page-context-menu` with the expected page context menu
  text.
- In-app Browser console error log was empty during the affected-route checks.
