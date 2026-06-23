# TASK-010B Native Browser Webview Or External-Open Fallback

Status: ready
Updated: 2026-06-23

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
- Choose and implement an accepted fallback for iframe-blocked public pages:
  native webview-backed Browser surface, system-browser handoff, or explicit
  `Open externally` fallback.
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
