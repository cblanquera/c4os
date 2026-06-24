# TASK-010A Browser Address Bar And Local Target UI

Status: verified
Updated: 2026-06-23

## Source

- Task: `.agents/specs/mvp/tasks.md` `TASK-010A`
- Accepted predecessor:
  `.agents/development/progress/items/TASK-010-browser-slice.md`
- Requirements: `.agents/specs/mvp/requirements.md` `REQ-010`, `REQ-013`,
  `REQ-016`
- Acceptance: `.agents/specs/mvp/acceptance.md` `AC-010`
- Progress manifest: `.agents/development/progress/manifest.md`

## Goal

Expose the TASK-010 Browser capability through the accepted right-panel
Browser surface so users can manually enter public URLs and trusted
project-local Browser targets from the existing address bar.

## Required Scope

- Preserve the accepted r04 shell and Browser / Files / Terminal right-panel
  tab contract.
- Preserve TASK-005 chat UX, TASK-005A scoped DOM updates, TASK-006
  provider/model boundaries, TASK-007 per-session ownership, TASK-008
  trusted-root Files/File Editor authority, TASK-009 artifact preview records,
  and TASK-010 project-scoped Browser records.
- Make the existing Browser address bar user-editable.
- Submit public `http://` and `https://` targets through the TASK-010
  `open_browser` command.
- Submit project-local Browser targets through the same backend trusted-root
  authority used by TASK-008/TASK-010.
- Show bounded success and failure state inside the existing Browser surface.
- Keep artifact preview distinct from general browsing.
- Do not add Browser routes, panels, tabs, alternate layouts, downloads,
  React/Preact/JSX, a bundler, or settings abstractions.
- Do not claim Browser security hardening beyond the TASK-010A UI boundary;
  final policy hardening remains TASK-016.

## Required Verification

- Public URL entry renders the resulting Browser state in the accepted Browser
  tab surface.
- Project-local target entry resolves to absolute local file URLs for user
  navigation, while agent local Browser loading remains trusted-root scoped and
  rejects traversal or `.git` access.
- Address-bar failures render inside the Browser surface without replacing the
  chat session, Files tab, or Terminal tab.
- TASK-009 artifact preview behavior still works and is not treated as general
  browsing.
- No new Browser route, panel, tab, alternate layout, or Terminal rendering is
  introduced.
- Switching sessions restores each session's Browser address and Browser
  rendered state.
- Existing TASK-005 through TASK-010 regression tests still pass.

## Outputs

- Made the existing Browser address bar user-editable for general Browser
  state while keeping TASK-009 artifact preview as a distinct non-browsing
  preview state.
- Submitted public `http://` and `https://` targets through the existing
  `openBrowser` connector and TASK-010 `open_browser` backend command.
- Submitted project-local Browser targets through the same backend
  trusted-root authority already used by TASK-008/TASK-010.
- Rendered bounded Browser success and failure status inside the existing
  Browser surface without replacing the chat session, Files tab, or Terminal
  tab.
- Preserved the right-panel tab list as Browser, Files, and Terminal and did
  not add Browser routes, panels, tabs, alternate layouts, downloads,
  React/Preact/JSX, a bundler, or settings abstractions.
- Updated older session/browser regression tests to read the accepted editable
  Browser address field instead of the previous static address text.
- Removed the padded Browser preview card for general browsing so public and
  local Browser frames fill the Browser tab below the address bar.
- Added address resolution for bare public hostnames such as
  `iamawesome.com`, which now resolve to `https://iamawesome.com/`.
- Changed user-initiated local Browser targets to canonical absolute
  `file:///...` URLs while preserving trusted-root restrictions for agent
  Browser loads.
- Removed the iframe sandbox only for public Browser pages; TASK-009 artifact
  previews and local-file Browser frames remain distinct from general browsing.

## Deferred

- Final Browser and approval policy hardening remains TASK-016.
- Terminal execution behavior remains TASK-011.

## Verification Run

- `node --test tests/frontend-task-010A.test.js` passed: 6 tests, 6 pass.
- `node --test tests/frontend-task-009.test.js tests/frontend-task-010.test.js
  tests/frontend-task-010A.test.js` passed: 14 tests, 14 pass.
- `node --test --test-concurrency=1 tests/frontend-task-005.test.js
  tests/frontend-task-005A.test.js tests/frontend-task-006.test.js
  tests/frontend-task-007.test.js tests/frontend-task-008.test.js
  tests/frontend-task-009.test.js tests/frontend-task-010.test.js
  tests/frontend-task-010A.test.js tests/frontend-connectors.test.js` passed:
  62 tests, 62 pass.
- `cargo test --manifest-path backend/Cargo.toml task_010 --
  --test-threads=1` passed: 4 tests, 4 pass.
- `node --test tests/server/task-008-files.test.js` passed: 2 tests, 2 pass.
- `npm test` passed: 102 tests, 102 pass.
- `cargo test --manifest-path backend/Cargo.toml -- --test-threads=1`
  passed: 34 tests, 34 pass.
- `rustfmt --check backend/src/commands.rs backend/src/lib.rs
  backend/src/mock_data.rs backend/src/runtime_sessions.rs` passed.
- `git diff --check` passed.
