# TASK-008 Files Slice

Status: accepted
Updated: 2026-06-23

## Source

- Task: `.agents/specs/mvp/tasks.md` `TASK-008`
- Accepted predecessor:
  `.agents/development/progress/items/TASK-007-runtime-adapter-persistent-sessions.md`
- Requirements: `.agents/specs/mvp/requirements.md` `REQ-008`, `REQ-013`
- Acceptance: `.agents/specs/mvp/acceptance.md` `AC-008`, `AC-017`
- Progress manifest: `.agents/development/progress/manifest.md`

## Goal

Replace Files mock behavior behind the accepted right-panel Files surfaces with
trusted-root browsing, open-file/code view, editing, saving, reverting,
guarded path rejection, and session-owned Files/File Editor restore.

## Required Scope

- Preserve the accepted r04 shell and Browser / Files / Terminal right-panel
  tab contract.
- Preserve TASK-005 chat behavior, TASK-005A scoped DOM updates, TASK-006
  global provider/model boundaries, and TASK-007 per-session ownership of
  thread, Browser, Terminal, Files/File Editor, and selected model state.
- Reject traversal outside trusted roots before read or write.
- Reject casual `.git` access before read or write.
- Keep Browser behavior, Terminal execution behavior, artifacts, extensions,
  local memory, advanced approvals, and audit-log hardening mocked.

## Outputs

- Added backend trusted file authority for active workspace roots.
- Replaced mock `read_file` and `save_file` command behavior with
  trusted-root read/list/save behavior.
- Added `File > Revert File` to the native OS menu contract and kept Save/
  Revert out of the right-panel Files UI.
- Added file state fields for current path, content, saved content, dirty
  state, and status while preserving existing roots, breadcrumbs, and lines.
- Updated session persistence so opening or saving files updates the active
  session-owned Files/File Editor record.
- Updated new session creation so chat-session Files state starts from the real
  active workspace file roots instead of the placeholder mock explorer.
- Updated frontend connector, state, and right-panel bindings so the existing
  Files explorer/editor surface supports open, dirty, save, and revert.
- Added in-place folder expand/collapse behavior behind the existing Files
  explorer rows.
- Restored visible editor line numbers without adding a new editor route or
  alternate layout.
- Kept Save/Revert in the native OS menu contract only, not in the right-panel
  Files surface.
- Kept the Files explorer independently scrollable so expanded folders do not
  move the chat-session prompt/composer.
- Removed folder active styling; the active explorer row is only the file that
  is currently open in the editor, when that row is visible.
- Removed visible `Opened`/`Ready` status rows from the Files explorer and file
  editor.
- Removed visible dirty/saved status rows entirely and moved dirty state to a
  breadcrumb asterisk, for example `mcp > src > index.ts *`.
- Updated file editor breadcrumbs so clicking a folder returns to the explorer
  with only that folder and its parent path expanded; clicking the project root
  returns to a root-only explorer.
- Updated native menu focus reporting so OS-menu Save File and Revert File are
  enabled whenever a trusted file editor is open.
- Updated file editor focus and keyboard bindings so OS-menu Save/Revert state
  is re-published when the editor is focused, and `Cmd/Ctrl+S` saves the active
  trusted file.
- Corrected native Save/Revert enablement to update the File submenu items
  directly instead of looking for nested items on the root Tauri menu.
- Kept keyboard save from replacing the file editor DOM node so saving preserves
  cursor/selection position.
- Scoped native Save/Revert enablement to actual file-editor focus, so focusing
  the chat prompt disables those OS-menu items.
- Added a direct native-menu-to-webview command hook for Save/Revert so selecting
  the OS menu items runs the same active-file save/revert handlers as the
  keyboard path.
- Added lightweight TextMate-inspired syntax token rendering under the existing
  textarea editor for common source/config files, without adding a bundler,
  route, or alternate editor surface.
- Extended trusted file-list rows with relative path/depth/git/ignored metadata
  while preserving the existing first-three-field row contract.
- Added colored common file-type row styling, git state colors, and ignored-path
  muted styling behind the existing explorer rows.
- Corrected ignored-path metadata so folders are muted only when the exact path
  is ignored by Git, not merely because they contain ignored descendants such as
  `node_modules`.
- Updated the responsive shell so medium widths keep the existing grid contract
  without clipping the workbench, while very narrow widths let the side panels
  overlay the workbench and remain collapsible from the top bar.
- Added TASK-008 frontend and server tests for trusted browsing, editor open,
  dirty/save/revert, folder scroll/active-state behavior, syntax token
  rendering, file/git/ignored row styling, guard rejection, and session
  restore.

## Deferred

- Create file, create folder, rename, and delete remain deferred because the
  accepted Files surface has no existing mutation controls for them.
- Browser behavior, Terminal execution behavior, artifacts, extensions, local
  memory, advanced approvals, and audit-log hardening remain mocked.

## Verification Run

- `node --test tests/frontend-task-008.test.js` passed: 8 tests, 8 pass.
- `node --test tests/frontend-task-008.test.js` passed after breadcrumb/menu
  review fixes: 9 tests, 9 pass.
- `cargo test --manifest-path backend/Cargo.toml task_008 -- --test-threads=1`
  passed after exact ignored-path metadata fix: 4 tests, 4 pass.
- `cargo test --manifest-path backend/Cargo.toml menu -- --test-threads=1`
  passed: 3 tests, 3 pass.
- `node --test tests/frontend-task-008.test.js
  tests/server/backend-tauri-scaffold.test.js` passed after the review fixes.
- `node --test tests/server/task-008-files.test.js` passed.
- `node --test tests/frontend-task-005.test.js
  tests/frontend-task-005A.test.js tests/frontend-task-006.test.js
  tests/frontend-task-007.test.js tests/frontend-task-008.test.js` passed:
  39 tests, 39 pass.
- `node --test tests/server/task-005-openrouter-chat.test.js
  tests/server/task-006-provider-models.test.js
  tests/server/task-007-runtime-sessions.test.js` passed.
- `node --test tests/frontend-r04.test.js tests/frontend-task-008.test.js`
  passed.
- `node --test tests/frontend-r04.test.js tests/frontend-task-006.test.js
  tests/frontend-task-008.test.js` passed after responsive shell review fixes:
  41 tests, 41 pass.
- `node --test tests/frontend-task-005.test.js
  tests/frontend-task-005A.test.js tests/frontend-task-007.test.js` passed
  after responsive shell review fixes: 16 tests, 16 pass.
- `node --test tests/frontend-r04.test.js tests/frontend-task-005.test.js
  tests/frontend-task-005A.test.js tests/frontend-task-006.test.js
  tests/frontend-task-007.test.js tests/frontend-task-008.test.js` passed
  after OS-menu and keyboard-save review fixes: 58 tests, 58 pass.
- `node --test tests/frontend-task-008.test.js
  tests/server/backend-tauri-scaffold.test.js` passed after native submenu and
  cursor-preserving save fixes: 14 tests, 14 pass.
- `cargo test --manifest-path backend/Cargo.toml menu -- --test-threads=1`
  passed after native submenu fix: 3 tests, 3 pass.
- `node --test tests/server/backend-tauri-scaffold.test.js` passed after native
  submenu fix: 4 tests, 4 pass.
- `node --test tests/frontend-task-008.test.js
  tests/server/backend-tauri-scaffold.test.js` passed after focus-scoped native
  menu and direct OS-menu command dispatch fixes: 15 tests, 15 pass.
- `node --test tests/frontend-r04.test.js tests/frontend-task-005.test.js
  tests/frontend-task-005A.test.js tests/frontend-task-006.test.js
  tests/frontend-task-007.test.js tests/frontend-task-008.test.js` passed after
  focus-scoped native menu fixes: 59 tests, 59 pass.
- `cargo test --manifest-path backend/Cargo.toml -- --test-threads=1`
  passed: 29 tests, 29 pass.
- `npm test` passed: 83 tests, 83 pass.
- `rustfmt --check backend/src/commands.rs backend/src/files.rs
  backend/src/lib.rs backend/src/menu.rs backend/src/mock_data.rs
  backend/src/runtime_sessions.rs backend/src/workspace.rs` passed.
- `git diff --check` passed.
