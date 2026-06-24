# TASK-010B Native Browser Webview Or External-Open Fallback

Status: verified
Updated: 2026-06-24

## Source

- Task: `.agents/specs/mvp/tasks.md` `TASK-010B`
- Accepted predecessor:
  `.agents/development/progress/items/TASK-010A-browser-address-bar-local-target-ui.md`
- Requirements: `.agents/specs/mvp/requirements.md` `REQ-010`, `REQ-013`,
  `REQ-016`
- Acceptance: `.agents/specs/mvp/acceptance.md` `AC-010`
- Progress manifest: `.agents/development/progress/manifest.md`

## Goal

Handle public websites that cannot be displayed in the current Browser iframe
because of `X-Frame-Options`, CSP `frame-ancestors`, or equivalent embedding
restrictions.

## Required Scope

- Preserve the accepted r04 shell and Browser / Files / Terminal right-panel
  tab contract.
- Preserve TASK-005 chat UX, TASK-005A scoped DOM updates, TASK-006
  provider/model boundaries, TASK-007 per-session ownership, TASK-008
  trusted-root Files/File Editor authority, TASK-009 artifact preview records,
  TASK-010 project-scoped Browser records, and TASK-010A editable address bar
  behavior.
- Implement the accepted raw Wry native webview-backed Browser surface for
  iframe-blocked public pages.
- Keep user browsing authority separate from agent browsing authority.
- Keep artifact previews distinct from general Browser browsing.
- Do not weaken TASK-009 generated/untrusted HTML sandboxing.
- Do not add downloads, unrelated settings abstractions, alternate app layouts,
  React/Preact/JSX, or a bundler.

## Required Verification

- Iframe-blocked public pages surface an intentional fallback instead of a
  blank or misleading Browser frame.
- Public pages that can render in the Browser still render through the
  accepted Browser tab surface.
- User-triggered fallback behavior does not grant agent browsing authority.
- Agent-triggered public browsing still requires clear request authority and is
  recorded as a Browser action.
- TASK-009 artifact preview remains sandboxed and distinct from general
  browsing.
- Files and Terminal tab behavior remains unchanged.
- No new Browser route, panel, tab, alternate layout, download flow, or
  settings abstraction is introduced.

## Outputs

- Implemented a raw Wry native Browser surface for public Browser pages inside
  the existing right-panel Browser tab boundary.
- Replaced the public Browser iframe with a full-width, full-height native
  host placeholder under the address bar. The backend positions the raw Wry
  child webview over that placeholder.
- Added a distinct `syncNativeBrowser` connector method backed by the Tauri
  `sync_native_browser` command.
- Corrected Browser geometry after the sticky-resize and preview-padding
  fixes: the frontend sends viewport bounds with a measured address-input
  native y inset and reduces native height by the same inset, while the native
  container remains parented through `NSWindow.contentView()`.
- Added resize observation for the native Browser host so the raw Wry child
  webview follows right-panel resize and window resize changes.
- Added a retained macOS `NSView` container under `NSWindow.contentView()` and
  aligned that native container to the accepted Browser placeholder. The raw
  Wry child webview fills the container at local `0,0`, avoiding direct
  Browser-page positioning against the full app window.
- Used raw `wry::WebViewBuilder::build_as_child(&host.parent)` rather than
  Tauri `WebviewWindow`, preserving the POC finding that direct Tauri
  WebviewWindow exposed `window.__TAURI_INTERNALS__`.
- Did not register a Wry IPC handler for Browser page content.
- Restricted native Browser navigation to public `http://` and `https://`
  URLs; local files remain outside the native public Browser host path.
- Hid the native Wry Browser surface when Files, Terminal, artifact previews,
  or local-file Browser previews own the right-panel body.
- Preserved TASK-009 artifact previews as sandboxed, product-owned preview
  records rendered through the DOM artifact frame.
- Preserved local-file Browser previews as local Browser iframe state using
  the TASK-008/TASK-010A file authority path.
- Preserved the accepted Browser / Files / Terminal right-panel tab contract
  and did not add Browser routes, panels, tabs, alternate layouts, downloads,
  React/Preact/JSX, a bundler, or settings abstractions.

## Deferred

- Final Browser and approval policy hardening remains TASK-016.
- Project-scoped persistent browser profile hardening remains outside this UI
  boundary and should be finalized with the Browser policy hardening work.
- Artifact preview type rendering follows in TASK-010C.
- Terminal execution behavior remains TASK-011.

## Verification Run

- `node --test tests/frontend-task-010B.test.js
  tests/server/task-010b-native-browser.test.js` passed after the approved
  measured-inset geometry fix: 5 tests, 5 pass.
- `cargo test --manifest-path backend/Cargo.toml task_010b --
  --test-threads=1` passed after the approved measured-inset geometry fix: 1
  test, 1 pass.
- `node --test tests/frontend-task-009.test.js tests/frontend-task-010.test.js
  tests/frontend-task-010A.test.js tests/frontend-task-010B.test.js
  tests/frontend-connectors.test.js tests/server/task-010b-native-browser.test.js`
  passed: 25 tests, 25 pass.
- `node --test tests/frontend-connectors.test.js` passed: 7 tests, 7 pass.
- User visual review approved the measured-inset geometry fix after external
  public page loading no longer blocked the Browser address bar.
- `npm test` passed: 107 tests, 107 pass.
- `cargo test --manifest-path backend/Cargo.toml -- --test-threads=1`
  passed: 35 tests, 35 pass.
- `cargo fmt --manifest-path backend/Cargo.toml -- --check` passed.
- `npm run backend:build` passed.
- `git diff --check` passed.
