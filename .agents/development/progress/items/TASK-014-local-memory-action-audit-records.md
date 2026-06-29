# TASK-014 Local Memory, Action Records, And Audit Records

Status: accepted
Updated: 2026-06-30

## Source

- Frozen task basis: `.agents/specs/mvp/tasks.md` `TASK-014`
- MVP requirements: `REQ-005`, `REQ-007`, `REQ-015`, `REQ-016`
- MVP acceptance: `AC-005`, `AC-007`, `AC-011`, `AC-012`, `AC-019`
- Context owners: `.agents/context/product-specs.md`,
  `.agents/context/technical-specs.md`

## Goal

Implement the smallest app-owned local memory, action record, and audit record
model needed by current MVP surfaces while preserving the accepted Browser,
Files, Terminal, provider/model, workspace-file, trusted-folder isolation, and
restart/resume behavior.

## Required Scope

- Add backend-owned records for scoped local memory, broad action records, and
  audit records.
- Persist these records separately from raw provider state, workspace
  descriptors, and runtime-local storage.
- Mirror existing Browser and Terminal action events into the app-owned action
  and audit record store without removing accepted session-owned state.
- Add commands for listing and writing MVP record data where current surfaces
  can consume it later.
- Keep records secret-safe: no raw provider keys, no full transcripts, no
  artifact archives, and no private runtime-local state.

## Out Of Scope

- TASK-016 security/approval hardening.
- Extension invocation or runtime tool gateway work.
- Broad approval-policy changes.
- Transitional prompt parser expansion.
- New routes, panels, settings categories, or visible UI surfaces.
- Storing local memory/action/audit data in workspace descriptors.

## Expected Implementation Paths

- `backend/`
- `frontend/` only if an accepted state path already exists
- `tests/server/`
- `.agents/development/progress/`

## Verification Plan

- Add failing tests first for app-owned record persistence and separation from
  workspace/provider/runtime stores.
- Run focused Rust tests for `task_014`.
- Run focused Node server coverage for `task-014`.
- Run related regression checks for TASK-013/TASK-013A behavior.
- Run built-app manual QA with
  `npm run backend:run:qa -- --workspace-file tests/projects/workspace.c4os.json`
  before marking ready for stakeholder acceptance.

## Outputs

- Added `backend/src/records.rs` as the app-owned record store for local
  memory records, action records, and audit records.
- Added `C4OS_RECORD_STORE` persistence separate from `C4OS_SESSION_STORE`,
  `C4OS_PROVIDER_STORE`, `C4OS_PROVIDER_KEY_STORE`, workspace descriptors, and
  runtime-local state.
- Added backend commands for listing local memory records, saving a local memory
  record, listing action records, and listing audit records.
- Registered the new commands through the Tauri invoke handler.
- Mirrored Browser actions and Terminal command actions into the app-owned
  action/audit record store while preserving existing session-owned Browser and
  Terminal state used by accepted UI surfaces.
- Mirrored file saves into app-owned action/audit records when a session-backed
  trusted workspace is active.
- Added focused server coverage for the record ownership boundary and Rust
  persistence/separation tests.

## Verification

- Red test confirmed TASK-014 record ownership was missing before
  implementation:
  `node --test --test-concurrency=1 tests/server/task-014-records.test.js`
  failed on missing `backend/src/records.rs`.
- `node --test --test-concurrency=1 tests/server/task-014-records.test.js`
  passed: 2 tests.
- `cargo test --manifest-path backend/Cargo.toml task_014 --
  --test-threads=1` passed: 2 tests.
- `cargo test --manifest-path backend/Cargo.toml task_013 --
  --test-threads=1` passed: 12 tests.
- `node --test --test-concurrency=1
  tests/server/task-013-concurrency-resume.test.js
  tests/server/task-013a-desktop-qa-bootstrap.test.js` passed: 4 tests.
- `node --test --test-concurrency=1 tests/server/task-011-terminal.test.js
  tests/server/task-010b-native-browser.test.js` passed: 5 tests.
- `cargo test --manifest-path backend/Cargo.toml task_011 --
  --test-threads=1` passed: 4 tests.
- `cargo fmt --manifest-path backend/Cargo.toml -- --check` passed.
- `git diff --check` passed.
- Built-app manual QA launched
  `npm run backend:run:qa -- --workspace-file
  tests/projects/workspace.c4os.json`; confirmed the app opened
  `#new-session` with `project-a` active, `project-a` and `project-b` visible
  in the project sidebar, Browser active with the address bar visible, Files
  showing the `project-a` file tree, and Terminal showing the preserved
  user/agent split. The QA app process was stopped after inspection.
- Follow-up Browser regression check reproduced that the deterministic
  `#new-session` workspace fixture showed a Browser address bar before any chat
  session existed, but backend `open_browser` still required a session. Added
  coverage for public URL and trusted local file opening from an active
  workspace before the first chat session, then fixed Browser session resolution
  to create/reuse a backend C4OS session for that workspace path while keeping
  public URL handling independent from local-file trusted-root resolution.
- `cargo test --manifest-path backend/Cargo.toml task_010 --
  --test-threads=1` passed: 7 tests.
- `node --test --test-concurrency=1 tests/frontend-task-010A.test.js
  tests/frontend-task-010B.test.js` passed with loopback permission: 10 tests.
- Stakeholder acceptance recorded on 2026-06-30 after the Browser regression
  follow-up passed review.

## Remaining Mocks And Out Of Scope

- No new visible UI surfaces were added. The new commands are backend-owned
  record access points for existing/future accepted state paths.
- Extension invocation, runtime tool gateway work, approval-policy hardening,
  security-policy enforcement, and broader TASK-016 audit semantics remain out
  of scope.
- Local memory save is a backend command only in this slice; no new Memory
  route, panel, or settings category was introduced.
