# TASK-002 Mock Server Harness

Status: review
Updated: 2026-06-22

`tests/server/` contains the mock backend harness used by TASK-002 rendered
integration tests. It is not production backend code and does not prove real
runtime, filesystem, security, extension, Browser, Terminal, provider, or
persistence behavior.

TASK-004 adds backend/integration coverage in this folder for the first real
workspace activation path. That coverage verifies the Rust/Tauri backend now
creates or loads a `.c4os/workspace.json` descriptor and returns real workspace
identity through the existing connector boundary. The mock server endpoints
below remain mock-only.

Run `node tests/server/dev-server.js` to serve the mock API on
`http://127.0.0.1:4310` for manual review, then open the frontend with
`?connector=server&api=http%3A%2F%2F127.0.0.1%3A4310`.

## Endpoints

- `GET /api/workspace`: returns fake workspace state for the existing r04
  frontend surfaces.
- `POST /api/runs`: returns fake structured run state for the existing chat
  transcript and Agent Run row.

Both endpoints support:

- `delay=<ms>` for loading and waiting-state coverage.
- `scenario=boot-failure` for workspace loading failure.
- `scenario=run-failure` for fake agent-run failure.

## Connector Contract

TASK-002 defines the frontend connector method inventory for future server and
Tauri transports. Only the first two methods are implemented in the HTTP mock
harness. TASK-004 implements `openWorkspace()` for the Tauri backend path:

- `loadWorkspace()`
- `sendPrompt(prompt)`
- `openWorkspace()`
- `createSession()`
- `readFile()`
- `saveFile()`
- `runTerminalCommand()`
- `openBrowserPreview()`
- `listExtensions()`

`loadWorkspace()` returns these required top-level fields:

- `workspace`
- `projects`
- `providers`
- `models`
- `pluginCatalog`
- `pluginMarketplaces`
- `skillCatalog`
- `mcpServers`
- `browser`
- `files`
- `terminal`
- `thread`

## Mocked Behavior

### loadWorkspace()

- Fake workspace trust and trusted-root state.
- Fake provider, model, runtime, settings, plugin, skill, and MCP records.
- Fake Files records, editable file content, artifacts/previews, Browser state,
  and Terminal output.
- Fake approvals, local memory, action records, workspace descriptors, and
  persistence.

### openWorkspace()

- Real only on the Tauri backend path in TASK-004.
- Creates or loads `.c4os/workspace.json` for a selected local folder.
- Returns real workspace identity and descriptor data.
- Still leaves provider/model records, agent processing, Files content, Browser,
  Terminal, extensions, approvals, local memory, artifacts, actions, audit
  records, and deeper persistence mocked.

### sendPrompt(prompt)

- Fake session creation and fake structured thread/run events.
- Fake agent processing, success, failure, and waiting transitions.
- Fake provider, runtime, approval, memory, action record, and persistence
  effects.

## Behavior Summary

- Fake workspace trust and trusted-root state.
- Fake provider, model, runtime, settings, plugin, skill, and MCP records.
- Fake session creation and fake structured thread/run events.
- Fake agent processing, success, failure, and waiting transitions.
- Fake Files records, editable file content, artifacts/previews, Browser state,
  and Terminal output.
- Fake approvals, local memory, action records, workspace descriptors, and
  persistence.

The harness exists only to exercise the frontend connector interface with
dynamic mock-backed state behind the accepted UI structure. Frontend code should
depend on normalized connector methods, not directly on this mock harness.
