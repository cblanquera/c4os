# TASK-003 Backend Mock Parity

Status: accepted
Updated: 2026-06-22

## Source

- Task: `.agents/specs/mvp/tasks.md` `TASK-003`
- Frozen spec: `.agents/specs/mvp/status.md`
- Progress manifest: `.agents/development/progress/manifest.md`
- Accepted baseline: `.agents/development/progress/items/TASK-002-mock-server-connection.md`

## Goal

Build the backend mock-parity surface in `backend/` using the TASK-002
connector contract, mock data shape, and fake processing behavior proven
through `tests/server/`, while preserving the accepted TASK-001/TASK-002
frontend without adding visible UI.

## Linked Requirements

REQ-001, REQ-003, REQ-008, REQ-010, REQ-011, REQ-013, REQ-014, REQ-015,
REQ-016

## Linked Acceptance

AC-001, AC-013, AC-014, AC-015, AC-016, AC-017, AC-018, AC-019, AC-020, AC-021,
AC-022, AC-023, AC-024, AC-025

## Expected Changed Paths

- `backend/`
- `tests/server/`
- `.agents/development/progress/`

## Verification

- `node --test tests/server/backend-tauri-scaffold.test.js`
- `cargo test --manifest-path backend/Cargo.toml`
- `npm test`
- `git diff --check`

## Implementation Notes

- Added a root `backend/` Rust/Tauri scaffold: `Cargo.toml`,
  `tauri.conf.json`, `build.rs`, `src/main.rs`, and `src/lib.rs`.
- Added Rust command modules for the TASK-002 connector method inventory:
  `load_workspace`, `send_prompt`, `open_workspace`, `create_session`,
  `read_file`, `save_file`, `run_terminal_command`, `open_browser_preview`,
  and `list_extensions`.
- Added `backend/src/mock_data.rs` as the backend-owned Rust copy of the
  TASK-002 mock workspace payload shape.
- Added `backend/src/menu.rs` to install the OS-level native desktop menu with
  Tauri `MenuBuilder`/`SubmenuBuilder`, custom File commands, native Edit
  commands, focus-based enabled state, and backend menu-event emission.
- Added `backend/README.md` to document Rust/Tauri backend mock parity, native
  menu behavior, and mocked behavior.
- Added `tests/server/backend-tauri-scaffold.test.js` to verify the backend is
  Rust/Tauri-only, installs the native menu, documents the menu behavior, and
  passes Rust tests.
- Added package scripts: `backend:build`, `backend:run`, and `backend:test`.
- Updated the frontend connector so `connector=tauri` invokes Rust/Tauri native
  commands and the Tauri connector is auto-selected when the native invoke
  bridge is available.
- Updated `npm test` to include `tests/server/*.test.js`.
- No visible UI components, controls, panels, menus, cards, settings
  abstractions, or route structures were added.

## Still Mocked

- Workspace trust, trusted-root state, and workspace descriptors.
- Provider/model records, settings IA records, and key storage.
- Session creation, structured thread/run events, and agent processing.
- File tree, file editor content, save behavior, artifacts/previews, Browser
  state, and Terminal output.
- Plugin, skill, MCP, extension, approval, local memory, action record, audit
  record, and persistence state.
- Native desktop menu File actions emit shell events and use mock handler
  semantics only; no production workspace/file persistence is claimed yet.

## Verification Notes

- `node --test tests/server/backend-tauri-scaffold.test.js`: pass, 4
  TASK-003 scaffold tests. The first run failed until the Rust/Tauri scaffold
  replaced the incorrect JavaScript backend. Later review found the OS menu was
  only documented; tests were expanded and the backend now installs a real
  Tauri app-wide menu with `menu::build_app_menu`.
- `cargo test --manifest-path backend/Cargo.toml`: pass, Rust tests for
  TASK-002 mock payload shape, fake run success/failure strings, mock file and
  terminal commands, and native menu enabled-state rules.
- `npm run backend:build`: pass, Rust/Tauri backend builds through the package
  script.
- `npm test`: pass, 32 tests across TASK-001 frontend parity, TASK-002
  mock-backed frontend integration, frontend connector coverage, and TASK-003
  Rust/Tauri backend scaffold coverage. npm emitted the existing user-config
  warning for `python`.

## Acceptance

User accepted TASK-003 on 2026-06-22 after the Rust/Tauri backend scaffold,
frontend Tauri connector, package scripts, and actual OS menu implementation
were corrected and verified.
