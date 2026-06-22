# TASK-004 First User Flow

Status: accepted
Updated: 2026-06-22

## Source

- Task: `.agents/specs/mvp/tasks.md` `TASK-004`
- Frozen spec: `.agents/specs/mvp/status.md`
- Progress manifest: `.agents/development/progress/manifest.md`
- Accepted baseline: `.agents/development/progress/items/TASK-003-backend-mock-parity.md`

## Goal

Activate the first real user journey through the accepted r04 frontend without
redesigning the UI: App Start -> Open Folder -> descriptor-backed workspace
activation -> existing workspace/session shell with real workspace identity.

## Linked Requirements

REQ-001, REQ-002, REQ-003, REQ-004, REQ-005, REQ-008, REQ-013, REQ-014,
REQ-016

## Linked Acceptance

AC-001, AC-002, AC-003, AC-004, AC-005, AC-008, AC-013, AC-014, AC-015,
AC-016, AC-017

## Changed Paths

- `backend/src/workspace.rs`
- `backend/src/commands.rs`
- `backend/src/lib.rs`
- `backend/README.md`
- `frontend/app.js`
- `frontend/data.js`
- `tests/frontend-task-004.test.js`
- `tests/server/task-004-first-user-flow.test.js`
- `tests/server/README.md`
- `.agents/development/progress/`

## Implementation Notes

- Added a Rust backend workspace module that canonicalizes a selected local
  folder, creates or loads `.c4os/workspace.json`, and returns a non-secret
  descriptor.
- Added a macOS native folder chooser for the App Start `Open Folder` path when
  the frontend does not already provide a path.
- Updated `open_workspace` to return descriptor-backed workspace identity and a
  connector-compatible workspace payload instead of the TASK-003 `/mock`
  workspace response.
- Kept the accepted App Start UI structure and bound the existing `Open Folder`
  button to the Tauri connector; after activation it routes to the existing
  `new-session` shell.
- Updated the accepted new-session shell copy to include the active folder name
  and made first prompt submission create a visible chat session row, route to
  the existing chat shell, and show observable thinking/activity state before
  the mock response paints.
- Removed the synthetic `Workspace started` session from folder activation;
  opening a folder with no existing chat history now shows no chat session
  rows until the first prompt creates one.
- Corrected prompt focus and send-button hover styling without adding new
  visible controls.
- Preserved TASK-003 mock behavior for the unrelated product surfaces.

## Still Mocked

- Boot-time `load_workspace` state before opening a folder.
- Provider/model records, settings IA records, key storage, and model fetching.
- Session creation beyond the first shell identity, structured thread/run
  events, and agent processing.
- Browser state, Files explorer/editor content, file save behavior, Terminal
  output/execution, artifacts/previews, extensions, plugins, skills, MCP
  behavior, approvals, local memory, action records, audit records, restart
  resume, and deeper app-owned persistence.
- Security and approval hardening beyond canonicalizing the selected folder and
  writing the non-secret descriptor.

## Verification

- `node --test tests/server/task-004-first-user-flow.test.js`
- `node --test tests/frontend-task-004.test.js`
- `cargo test --manifest-path backend/Cargo.toml`
- `npm test`
- `git diff --check`

## Verification Notes

- `node --test tests/server/task-004-first-user-flow.test.js`: pass, 2
  TASK-004 backend/integration tests.
- `node --test tests/frontend-task-004.test.js`: pass, 1 browser test after
  rerunning with localhost access because the sandbox blocked `127.0.0.1`.
  Later review feedback expanded this to 2 browser tests covering the folder
  name heading, prompt focus style, send hover style, first prompt session row,
  thinking/activity indicators, and final mock response.
- `cargo test --manifest-path backend/Cargo.toml`: pass, 10 Rust tests.
- `npm test`: pass, 36 tests across frontend connector coverage, r04 parity,
  TASK-002 mock integration, TASK-003 backend scaffold, and TASK-004 first
  user flow. npm emitted the existing user-config warning for `python`.
- `git diff --check`: pass.

## Acceptance

User accepted TASK-004 on 2026-06-22 after review feedback on folder choosing,
empty session state after folder open, folder-name copy, prompt focus, send
hover, first prompt session creation, thinking/activity indicators, and mock
response visibility was addressed and verified.
