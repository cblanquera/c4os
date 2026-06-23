# TASK-010C Artifact Preview Type Rendering

Status: queued
Updated: 2026-06-23

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
