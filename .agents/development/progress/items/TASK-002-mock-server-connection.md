# TASK-002 Mock Server Connection

Status: accepted
Updated: 2026-06-22

## Source

- Task: `.agents/specs/mvp/tasks.md` `TASK-002`
- Frozen spec: `.agents/specs/mvp/status.md`
- Progress manifest: `.agents/development/progress/manifest.md`
- Baseline: `.agents/development/progress/items/TASK-001-r04-frontend.md`

## Goal

Connect the verified TASK-001 frontend to a mock server harness under
`tests/server/` without changing the accepted r04 screen structure, route
inventory, or visible control set.

## Linked Requirements

REQ-001, REQ-003, REQ-008, REQ-010, REQ-011, REQ-013, REQ-014, REQ-015,
REQ-016

## Linked Acceptance

AC-001, AC-013, AC-014, AC-015, AC-016, AC-017, AC-018, AC-019, AC-020, AC-021,
AC-022, AC-023, AC-024, AC-025

## Implementation Notes

- Added `tests/server/mock-server.js` as a deterministic local HTTP mock
  server with workspace and run endpoints.
- Added `tests/server/mock-data.js` as the mock workspace payload source.
- Added `tests/server/dev-server.js` for manual acceptance review against a
  local mock API.
- Added `tests/server/README.md` to state the mocked behavior and endpoint
  scenarios.
- Added rendered integration coverage in `tests/frontend-task-002.test.js`.
- Added `frontend/connectors.js` as the normalized frontend connector boundary.
  The current server transport uses `?connector=server&api=<origin>`; a disabled
  Tauri transport placeholder preserves the same method shape for TASK-003.
- Added `frontend/connector-contract.js` to define the normalized method
  inventory and required workspace payload fields for server and Tauri
  transports.
- Updated `frontend/data.js` to hydrate the existing TASK-001 fallback state
  through connector methods instead of mock-server-specific frontend functions.
- Updated `frontend/app.js` so existing App Start, Chat, Browser, Files,
  Terminal, Providers, Models, Plugins, Skills, and MCP surfaces read from the
  hydrated state.
- Wired the existing Send Prompt button to the connector `sendPrompt()` method.
  The existing transcript and Agent Run row show waiting, success, and failure
  states; no route, panel, menu, card, settings abstraction, or visible control
  was added.

## Mocked Behavior

- Fake workspace trust and trusted-root state.
- Fake provider/model records and settings IA records.
- Fake session creation and fake structured thread/run events.
- Fake agent processing, including loading, waiting, success, and failure.
- Fake files, file editor content, artifacts/previews, Browser state, and
  Terminal output.
- Fake plugin, skill, MCP, extension, approval, local memory, action record,
  workspace descriptor, and persistence state.

None of the above is production backend behavior. TASK-003 must keep returning
the same mock data from the real backend surface before later tasks replace
mock behavior one slice at a time.

## Verification Notes

- `node --test tests/frontend-task-002.test.js`: pass, 4 rendered integration
  tests for loading, boot success, boot failure, waiting, run success, run
  failure, and state transition behavior. Sandboxed localhost binding failed
  with `EPERM`; reran with the local-server execution boundary.
- `node --test tests/frontend-r04.test.js`: pass, 17 rendered TASK-001 parity
  tests after mock wiring.
- `npm test`: pass, 24 tests across TASK-001 parity, TASK-002 mock-backed
  integration, and connector boundary coverage. npm emitted the existing
  user-config warning for `python`.
- `node --check frontend/app.js`, `node --check frontend/data.js`,
  `node --check frontend/connectors.js`,
  `node --check tests/server/mock-server.js`,
  `node --check tests/server/dev-server.js`, and
  `node --check tests/frontend-task-002.test.js`: pass.
- `git diff --check`: pass.

## Review Feedback Repair

- Replaced mock-server-specific frontend functions with a connector interface:
  `createConnector()`, `loadWorkspace()`, and `sendPrompt()`.
- Kept mock data and mock endpoint behavior under `tests/server/`.
- Removed the frontend `mockApi` query contract in favor of
  `connector=server&api=<origin>`.
- Added tests that fail if `frontend/app.js` reintroduces
  `beginMockStateLoad`, `sendMockPrompt`, or `mockRuntime`.

## Connector Hardening Pass

- Defined the future connector method inventory:
  `loadWorkspace`, `sendPrompt`, `openWorkspace`, `createSession`, `readFile`,
  `saveFile`, `runTerminalCommand`, `openBrowserPreview`, and
  `listExtensions`.
- Added required top-level workspace payload validation for `workspace`,
  `projects`, `providers`, `models`, `pluginCatalog`, `pluginMarketplaces`,
  `skillCatalog`, `mcpServers`, `browser`, `files`, `terminal`, and `thread`.
- Updated connector tests so the mock server payload is the contract parity
  target for TASK-003.
- Documented mocked behavior by connector method in `tests/server/README.md`.
- `npm test`: pass, 27 tests across connector contract, TASK-001 parity, and
  TASK-002 rendered integration. npm emitted the existing user-config warning
  for `python`.

## Acceptance

- User accepted TASK-002 on 2026-06-22.
- TASK-002 is cleared for TASK-003 backend mock parity.
- Final verification after acceptance: `npm test` pass, 27 tests across
  connector contract, TASK-001 parity, and TASK-002 rendered integration. npm
  emitted the existing user-config warning for `python`.
