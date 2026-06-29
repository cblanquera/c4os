# Progress Manifest

Status: ready-for-wo-006-runtime-tool-gateway-refactor
Updated: 2026-06-30

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
- `.agents/development/progress/items/TASK-010C-artifact-preview-type-rendering.md`
- `.agents/development/progress/items/TASK-011-terminal-slice.md`
- `.agents/development/progress/items/TASK-011A-agent-command-terminal-bridge.md`
- `.agents/development/progress/items/TASK-011B-chat-session-transition-polish.md`
- `.agents/development/progress/items/TASK-012-settings-ia-extension-records.md`
- `.agents/development/progress/items/TASK-013-concurrency-restart-resume.md`
- `.agents/development/progress/items/TASK-013A-desktop-qa-bootstrap-workspace-provider-persistence.md`
- `.agents/development/progress/items/TASK-014-local-memory-action-audit-records.md`
- `.agents/development/progress/items/TASK-015-pause-at-feature-complete.md`

## Active Item

- None. Next item is
  `.agents/development/progress/items/WO-006-runtime-tool-gateway-refactor.md`.

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

TASK-015 is accepted. The feature-complete package summarizes the completed MVP
feature list, acceptance evidence by task and surface, remaining mocks or
shortcuts, and the explicit pre-`TASK-016` boundary.

Before `TASK-016`, run
`.agents/development/progress/items/WO-006-runtime-tool-gateway-refactor.md`.
This refactor should formalize runtime-driven tool execution through the
C4OS-owned tool gateway so security and approval hardening targets the correct
internal execution boundary instead of the transitional explicit command
parser.
