# Progress Manifest

Status: ready-for-task-010C
Updated: 2026-06-24

## Active Stream

MVP implementation queue from frozen spec `.agents/specs/mvp/status.md`.

## Last Verified Item

- `.agents/development/progress/items/TASK-001-r04-frontend.md`
- `.agents/development/progress/items/TASK-002-mock-server-connection.md`
- `.agents/development/progress/items/TASK-003-backend-mock-parity.md`
- `.agents/development/progress/items/TASK-004-first-user-flow.md`
- `.agents/development/progress/items/TASK-005-openrouter-chat-session.md`
- `.agents/development/progress/items/TASK-005A-scoped-frontend-state.md`
- `.agents/development/progress/items/TASK-006-provider-model-management.md`
- `.agents/development/progress/items/TASK-007-runtime-adapter-persistent-sessions.md`
- `.agents/development/progress/items/TASK-008-files-slice.md`
- `.agents/development/progress/items/TASK-009-artifact-safe-html-preview.md`
- `.agents/development/progress/items/TASK-010-browser-slice.md`
- `.agents/development/progress/items/TASK-010A-browser-address-bar-local-target-ui.md`
- `.agents/development/progress/items/TASK-010B-native-browser-webview-or-external-open-fallback.md`

## Scope Rules

- Progress items execute the frozen MVP contract; they do not redefine product
  architecture or MVP scope.
- Production implementation paths are `backend/`, `frontend/`, and
  `tests/server/`.
- TASK-001 is the frontend-foundation exception: use `frontend/`,
  `tests/frontend-*.test.*`, `tests/support/`, minimal package scripts, and the
  active progress item only. Do not create `tests/server/` content until
  TASK-002 or later.
- Do not create or use `src-tauri/`.
- Proposed task records remain in `.agents/specs/mvp/tasks.md`; active work
  lives in `.agents/development/progress/items/`.
- Mock-backed phases must state exactly what is mocked before acceptance.
- TASK-001 is a parity-preserving production frontend task. Preserve r04 route
  structures, working interactions, and settings screen shapes, then apply a
  production desktop-app visual treatment. Do not add invented UI, stop at a
  grayscale wireframe copy, or accept source-string-only verification.
- After TASK-001, implementation may switch static fixtures and static
  component behavior to dynamic, backend-backed, or working behavior behind the
  accepted UI. Do not add new visible UI components, controls, panels, menus,
  cards, settings abstractions, or route structures unless explicitly accepted
  and documented before implementation.

## Next Step

Start `TASK-010C` Artifact Preview Type Rendering from the verified TASK-010B
raw Wry native Browser boundary. Preserve the accepted r04 shell, Browser /
Files / Terminal right-panel tab contract, TASK-005 chat UX, TASK-005A scoped
update patterns, TASK-006 provider/model boundaries, TASK-007 session
ownership, TASK-008 Files/File Editor authority, TASK-009 product-owned
artifact preview records, TASK-010 project-scoped Browser records, TASK-010A
editable Browser address bar behavior, and TASK-010B public-page native Wry
Browser host behavior. TASK-010C extends artifact previews by MIME type or
file extension while keeping artifacts distinct from general Browser
navigation and without adding a new Artifacts tab, route, panel, alternate
layout, downloads, React/Preact/JSX, a bundler, or settings abstraction.
TASK-011 resumes Terminal work after TASK-010C.
