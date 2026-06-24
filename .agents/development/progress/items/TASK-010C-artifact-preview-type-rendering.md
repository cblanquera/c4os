# TASK-010C Artifact Preview Type Rendering

Status: verified
Updated: 2026-06-24

## Source

- Task: `.agents/specs/mvp/tasks.md` `TASK-010C`
- Planned predecessor:
  `.agents/development/progress/items/TASK-010B-native-browser-webview-or-external-open-fallback.md`
- Requirements: `.agents/specs/mvp/requirements.md` `REQ-009`, `REQ-010`,
  `REQ-013`, `REQ-016`
- Acceptance: `.agents/specs/mvp/acceptance.md` `AC-009`, `AC-010`
- Progress manifest: `.agents/development/progress/manifest.md`

## Goal

Extend product-owned artifact preview rendering inside the accepted
Browser/Preview surface so artifacts render according to MIME type or file
extension.

## Required Scope

- Preserve the accepted r04 shell and Browser / Files / Terminal right-panel
  tab contract.
- Preserve TASK-009 artifact preview records, TASK-010 project-scoped Browser
  records, TASK-010A editable address bar behavior, and TASK-010B public-site
  fallback behavior.
- Support safe previews for common MVP artifact types where practical:
  generated or untrusted HTML, Markdown, images, PDF, plain text, JSON, and
  code.
- Keep generated/untrusted HTML sandboxed.
- Keep artifact previews distinct from general Browser navigation.
- Do not render artifacts in Terminal.
- Do not add a new Artifacts tab, route, panel, alternate layout, downloads,
  React/Preact/JSX, a bundler, or settings abstraction.

## Required Verification

- Artifact records with supported MIME types or file extensions render through
  the existing Browser/Preview tab surface.
- Generated/untrusted HTML artifact previews remain sandboxed and cannot access
  app APIs, credentials, Files state, or Terminal state.
- Markdown, image, PDF, plain text, JSON, and code previews render with bounded
  empty/error states when content is missing or unsupported.
- General public browsing is not treated as artifact preview rendering.
- Files and Terminal tab behavior remains unchanged.
- No new Artifacts route, panel, tab, alternate layout, download flow, or
  settings abstraction is introduced.

## Outputs

- Extended the existing Browser/Preview artifact branch to classify
  product-owned artifact records by MIME type, filename extension, or kind.
- Preserved generated or untrusted HTML on the existing sandboxed artifact
  iframe path with `sandbox="allow-scripts"`.
- Added safe Browser/Preview renderers for Markdown, images, PDF, plain text,
  JSON, and code without adding an Artifacts tab, route, panel, alternate
  layout, downloads, React/Preact/JSX, a bundler, or settings abstraction.
- Kept artifact previews distinct from public Browser navigation and the raw
  Wry native Browser host.
- Simplified trusted local-file Browser previews after user-approved scope
  widening: trusted non-Markdown project files now use the native Wry Browser
  host instead of product-owned per-file-type DOM/data-URL renderers.
- Kept trusted `.md` and `.markdown` Browser previews as the only explicit
  local-file DOM renderer because native WebKit/Wry does not format Markdown.
- Removed the intermediate local-file HTML asset inlining and binary data-URL
  preview path; CSS, JS, images, PDFs, and other trusted project-local files are
  delegated to native Wry after Browser authority accepts them.
- Corrected the native Wry local-file validator to match the accepted TASK-010A
  Browser authority: user-opened absolute local files may render through Wry,
  missing files and `.git` internals remain rejected, and agent-opened local
  files remain scoped by `open_browser` trusted-root checks.
- Preserved Files and Terminal tab behavior, including the rule that Terminal
  does not render artifacts.
- Added typed artifact preview fields to backend product-owned artifact
  records while preserving the existing generated-HTML command contract.

## Deferred

- Full Browser and approval-policy hardening remains TASK-016.
- Terminal execution behavior remains TASK-011.

## Verification Run

- `node --test tests/frontend-task-010.test.js tests/frontend-task-010A.test.js
  tests/frontend-task-010B.test.js tests/frontend-task-010C.test.js` passed:
  17 tests, 17 pass.
- `cargo test --manifest-path backend/Cargo.toml task_010 -- --test-threads=1`
  passed: 6 tests, 6 pass.
- `node --test tests/frontend-task-009.test.js tests/frontend-task-010.test.js
  tests/frontend-task-010A.test.js tests/frontend-task-010B.test.js
  tests/frontend-task-010C.test.js tests/frontend-connectors.test.js
  tests/server/task-010b-native-browser.test.js` passed: 29 tests, 29 pass.
- `cargo test --manifest-path backend/Cargo.toml -- --test-threads=1`
  passed: 36 tests, 36 pass.
- `cargo fmt --manifest-path backend/Cargo.toml -- --check` passed.
- `npm run backend:build` passed.
- `npm test` passed: 111 tests, 111 pass.
- `git diff --check` passed.
- Follow-up absolute local-file native Wry verification:
  `cargo test --manifest-path backend/Cargo.toml task_010 -- --test-threads=1`
  passed: 6 tests, 6 pass; `cargo fmt --manifest-path backend/Cargo.toml
  -- --check` passed.
