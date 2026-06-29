# TASK-013A Desktop QA Bootstrap And Workspace/Provider Persistence

Status: accepted
Updated: 2026-06-30

## Source

- Follow-up to: `.agents/development/progress/items/TASK-013-concurrency-restart-resume.md`
- Frozen task basis: `.agents/specs/mvp/tasks.md` `TASK-013`, `TASK-014`
- MVP requirements: `REQ-001`, `REQ-002`, `REQ-003`, `REQ-005`, `REQ-006`,
  `REQ-013`, `REQ-016`
- MVP acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-005`, `AC-006`, `AC-014`

## Goal

Remove routine desktop QA friction after TASK-013 by making the app able to
boot into a deterministic trusted workspace, open/save a non-secret workspace
file, and persist provider/model configuration across restart before TASK-014
continues with local memory and audit records.

## Required Scope

- Preserve TASK-013 trusted-folder/session isolation and restart/resume
  behavior.
- Add an agent-friendly boot path that can activate a trusted workspace from an
  explicit launch flag for a workspace path or workspace file.
- Implement real workspace file save/open behavior for non-secret descriptors.
- Persist provider profiles, provider enablement, model records, model
  enablement, and selected model context across app restart.
- Keep raw provider keys out of workspace descriptors, session records, normal
  provider settings, frontend fixtures, and logs.
- Update manual QA workflow so agents use deterministic bootstrapping before
  asking for stakeholder acceptance.

## Out Of Scope

- Local memory, broad action records, and audit records for TASK-014.
- Approval-policy hardening and TASK-016 security policy.
- Runtime tool gateway work or expansion of the transitional prompt-command
  parser.
- New visible panels, routes, settings categories, or alternate onboarding.

## Expected Implementation Paths

- `backend/`
- `frontend/`
- `tests/server/`
- `.agents/development/progress/`
- `.agents/workflows/manual-qa.md`

## Verification Plan

- Add failing backend tests for workspace bootstrap/open-save and provider
  persistence before implementation.
- Run focused Rust tests for `task_013a`.
- Run focused Node server coverage for `task-013a`.
- Run relevant frontend tests for app-start/new-session routing after bootstrap.
- Run built-app manual QA through the updated deterministic workflow before
  marking this verified.

## Outputs

- Added backend workspace bootstrap from explicit launch flags:
  `--workspace` and `--workspace-file`.
- Added backend workspace file save/open commands for non-secret descriptors.
- `load_workspace` now attempts only configured launch-flag bootstrap before
  falling back to mock/no-active workspace state, so normal human launches
  still start at the start screen.
- Added provider/model settings persistence through `C4OS_PROVIDER_STORE`,
  with raw keys kept out of that normal settings file and loaded through the
  separate `C4OS_PROVIDER_KEY_STORE` key-store abstraction.
- Added Tauri command registrations and connector methods for workspace file
  open/save.
- Added frontend boot routing so a connector-hydrated trusted workspace moves
  from `#app-start` to `#new-session` or `#chat-session`.
- Tightened frontend boot routing so normal human launches with only mock or
  recent-project IDs remain on `#app-start`; bypass requires a real trusted
  root path from the launch flag.
- Removed mock/recent workspace rows from normal no-trusted-workspace startup.
  The backend no-active payload now returns no project rows, and the frontend
  app-start no-connector fallback renders no fake recent workspaces.
- Routed OS File menu workspace open/save events to workspace-file load/save
  actions, and made no-path workspace-file backend commands open native macOS
  file picker/save dialogs for app menu usage.
- Repaired debug app launch so `npm run backend:run` / `backend:run:qa`
  builds and syncs the current debug binary into the macOS `.app` wrapper
  before launch; this prevents Computer Use/manual QA from attaching to stale
  wrapper code.
- Added an explicit macOS app menu ahead of File so C4OS exposes a real
  top-level File menu instead of folding workspace commands into the app menu.
- Updated manual QA workflow to prefer deterministic workspace bootstrapping
  and require recording bootstrap/provider-store evidence.

## Verification

- Red test pass confirmed missing `task_013a` workspace/provider APIs before
  implementation.
- Follow-up red test pass confirmed the previous env-based bypass violated the
  clarified human-start-screen contract before implementation was corrected.
- `cargo test --manifest-path backend/Cargo.toml task_013a --
  --test-threads=1` passed: 5 tests.
- Follow-up frontend red test confirmed normal `load_workspace` mock/recent
  project state still skipped to `#chat-session`; fixed by requiring a real
  `workspace.rootPath` for boot-route bypass.
- Follow-up red tests confirmed normal no-trusted-root startup still returned
  `Mock Workspace Alpha` and frontend no-connector app-start still rendered
  fallback fake rows; fixed by returning/rendering an empty recent-workspace
  list unless a real or explicit mock test connector supplies projects.
- `cargo test --manifest-path backend/Cargo.toml task_006 --
  --test-threads=1` passed: 7 tests.
- `node --test --test-concurrency=1
  tests/server/task-006-provider-models.test.js
  tests/server/task-013a-desktop-qa-bootstrap.test.js` passed: 7 tests.
- `node --test --test-concurrency=1 tests/frontend-task-013A.test.js` first
  hit sandbox `listen EPERM` on `127.0.0.1`; rerun with loopback permission
  passed: 1 test.
- `node --test --test-concurrency=1 tests/frontend-task-004.test.js
  tests/frontend-task-013A.test.js` passed with loopback permission: 5 tests.
- `node --test --test-concurrency=1 tests/frontend-task-002.test.js
  tests/frontend-task-004.test.js tests/frontend-task-013A.test.js` passed with
  loopback permission: 10 tests.
- `cargo test --manifest-path backend/Cargo.toml
  command_inventory_returns_task_002_mock_payloads -- --test-threads=1`
  passed: 1 test.
- `cargo fmt --manifest-path backend/Cargo.toml -- --check` passed.
- `git diff --check` passed.
- `npm test` passed 149/150 tests, then failed the TASK-003 wrapper around
  full Rust tests without surfacing the inner assertion. The exact wrapper
  test `node --test --test-concurrency=1
  tests/server/backend-tauri-scaffold.test.js` passed on rerun, and direct
  `cargo test --manifest-path backend/Cargo.toml -- --test-threads=1` passed:
  48 tests.
- Follow-up red frontend test confirmed OS menu `file.openWorkspace` /
  `file.saveWorkspace` events were not routed to workspace-file actions before
  implementation.
- `node --test --test-concurrency=1 tests/frontend-task-013A.test.js` passed
  with loopback permission: 4 tests.
- `node --test --test-concurrency=1
  tests/server/task-013a-desktop-qa-bootstrap.test.js` passed: 2 tests.
- `node --test --test-concurrency=1 tests/frontend-task-008.test.js` passed
  with loopback permission: 11 tests.
- `cargo fmt --manifest-path backend/Cargo.toml --check` passed.
- `git diff --check` passed.
- Manual desktop QA reproduced two real desktop-only failures: `cargo run`
  opened stale `backend/target/debug/C4OS.app/Contents/MacOS/C4OS` code, and
  macOS showed `C4OS` / `Edit` with no top-level `File` menu because the File
  submenu was first in the custom menu.
- Manual desktop QA after the fix used
  `npm run backend:run:qa -- --workspace /private/tmp/c4os-menu-qa-workspace`;
  confirmed the synced app launched to `#new-session`, showed a real top-level
  `File` menu, and File > Save Workspace opened the native save panel.
- Saved `/private/tmp/c4os-menu-qa-workspace/menu-workspace.c4os.json`;
  verified the file contains only the workspace descriptor with root path,
  trusted flag, and timestamps.
- Relaunched with `npm run backend:run:qa` and no workspace flag; confirmed the
  centered human start screen stayed visible with no recent-workspaces card.
- Used File > Open Workspace from the start screen, selected the saved
  descriptor in the native open panel, and confirmed the app reached
  `#new-session` with `c4os-menu-qa-workspace` active.
- Backend stdout during manual QA showed no command errors or panics.
- `node --test --test-concurrency=1
  tests/server/backend-tauri-scaffold.test.js` passed: 4 tests.
- `node --test --test-concurrency=1
  tests/server/task-013a-desktop-qa-bootstrap.test.js` passed: 2 tests.
- `node --test --test-concurrency=1 tests/frontend-task-013A.test.js` passed
  with loopback permission: 4 tests.
- `cargo test --manifest-path backend/Cargo.toml task_013a --
  --test-threads=1` passed: 5 tests.
- Follow-up red Rust test reproduced the two-project workspace-file regression:
  reopening a portable workspace file restored only 1 project instead of 2.
- Follow-up backend fix saves portable workspace files as an active descriptor
  plus project descriptor list, preserves backwards compatibility with old
  single-descriptor files, and persists real recent project descriptors for the
  no-active-workspace start screen.
- Follow-up frontend fix keeps recent projects separate from active workspace
  state, so no-flag human launches stay on the start screen while rendering the
  right recent-workspaces card when real recents exist.
- Recent workspace rows now call `open_workspace` with the saved root path
  instead of only navigating to `#new-session`.
- `cargo test --manifest-path backend/Cargo.toml task_013a --
  --test-threads=1` passed: 7 tests.
- `node --test --test-concurrency=1 tests/frontend-task-013A.test.js` passed
  with loopback permission: 5 tests.
- `node --test --test-concurrency=1
  tests/server/task-013a-desktop-qa-bootstrap.test.js` passed: 2 tests.
- `cargo fmt --manifest-path backend/Cargo.toml --check` passed.
- `git diff --check` passed.
- Built-app manual QA used isolated fixtures under
  `/private/tmp/c4os-two-project-manual-qa`, launched
  `npm run backend:run:qa -- --workspace-file
  /private/tmp/c4os-two-project-manual-qa/two-projects.c4os.json`, and
  confirmed `#new-session` showed `manual-project-two` active with both
  `manual-project-two` and `manual-project-one` in the sidebar.
- Built-app manual QA then launched `npm run backend:run:qa` with the same
  isolated recents store and no workspace flag; confirmed the app stayed on the
  human start screen and showed the right recent-workspaces card with both
  projects.
- Built-app manual QA clicked `manual-project-two` from the recent card and
  confirmed the app opened `#new-session` with `manual-project-two` active and
  both projects still listed.
- Follow-up fixture pass made `tests/projects/workspace.c4os.json` an
  agent-usable workspace-file boot fixture: project IDs may be omitted, relative
  `rootPath` values resolve from the workspace file folder, and the start-card
  label is `Recent Workspaces`.
- `cargo test --manifest-path backend/Cargo.toml task_013a --
  --test-threads=1` passed: 8 tests.
- `node --test --test-concurrency=1 tests/frontend-task-013A.test.js` passed:
  5 tests.
- `node --test --test-concurrency=1
  tests/server/task-013a-desktop-qa-bootstrap.test.js` passed: 2 tests.
- `cargo fmt --manifest-path backend/Cargo.toml --check` passed.
- `git diff --check` passed.
- Built-app manual QA launched `npm run backend:run:qa -- --workspace-file
  tests/projects/workspace.c4os.json` and confirmed the app opened
  `#new-session` with `project-a` active and both `project-a` and `project-b`
  in the project sidebar.
- Stakeholder acceptance recorded on 2026-06-30 after workspace open/save,
  recent-workspace display, relative workspace-file roots, and the checked-in
  QA boot fixture were confirmed working.

## Notes

- This item is an execution hardening slice inserted before TASK-014. It does
  not redefine the frozen MVP task sequence.
- Built-app manual QA for OS menu workspace save/open passed on 2026-06-29.
- Full frontend test execution regenerated TASK-001 screenshot evidence files;
  those generated PNG diffs are not part of the intended TASK-013A source
  change.
