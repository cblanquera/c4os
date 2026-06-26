# TASK-012 Settings IA And Extension Records Slice

Status: approved
Updated: 2026-06-26

## Source

- Task: `.agents/specs/mvp/tasks.md` `TASK-012`
- Predecessor:
  `.agents/development/progress/items/TASK-011B-chat-session-transition-polish.md`
- MVP requirements: `REQ-013`, `REQ-015`, `REQ-016`
- MVP acceptance: `AC-018`, `AC-019`

## Goal

Replace settings, plugin, skill, and MCP mock records with app-owned records
behind the accepted Settings IA while keeping runtime impact disabled until
explicit enablement.

## Required Scope

- Preserve Settings navigation order: Providers, Models, Runtimes,
  Configuration, Plugins, Skills, MCP Servers.
- Preserve verified TASK-011, TASK-011A, and TASK-011B Terminal and
  new-session transition behavior.
- Keep plugin, skill, and MCP records app-owned with provenance, scopes,
  workspace/project scope, shared data, runtime/tool access, enabled state, and
  audit records.
- Do not implement extension invocation, runtime tool gateway, broad
  approval-policy hardening, concurrency, restart/resume, memory, audit
  hardening, or TASK-016 security policy.
- Do not expand the transitional explicit prompt command parser.
- Do not renumber frozen MVP tasks.

## Outputs

- Added `backend/src/extensions.rs` with structured C4OS-owned plugin, skill,
  and MCP records and inert runtime access defaults.
- Updated backend workspace payloads and `list_extensions` to use structured
  extension records instead of mock string lists.
- Updated frontend fallback and connector-hydrated extension state to normalize
  structured extension records while tolerating older empty/string test
  payloads.
- Updated Plugins, Skills, and MCP Settings routes to render provenance, scope,
  shared data, runtime access, tool access, enabled state, and audit details
  without changing the accepted Settings navigation order or r04 dialog labels.
- Updated the shared mock server payload to use the TASK-012 extension record
  shape.
- Added focused TASK-012 frontend and server tests.
- Added empty states for Plugins, Skills, and MCP Servers when no extension
  records are installed or connected.
- Added a frontend guard for minimal duplicate `create_session` responses so
  local new-session state remains stable in connector harnesses while real
  backend session records still use backend session ids.
- Added `.agents/references/context/technical-specs/extension-loading.md` to
  preserve research on future skill, plugin, and MCP discovery/loading without
  expanding TASK-012 or TASK-013 scope.

## Verification

- `node --test --test-concurrency=1 tests/frontend-task-012.test.js` passed:
  3 tests for Settings IA order, app-owned extension record rendering, empty
  extension states, and inert prompt tags before runtime enablement.
- `node --test --test-concurrency=1 tests/server/task-012-extension-records.test.js`
  passed: 2 tests for backend extension record fields and inert Rust records.
- `cargo test --manifest-path backend/Cargo.toml -- --test-threads=1` passed:
  41 backend tests.
- `node --test --test-concurrency=1 tests/server/task-006-provider-models.test.js
  tests/server/task-011-terminal.test.js tests/server/task-012-extension-records.test.js`
  passed: 11 tests.
- `node --test --test-concurrency=1 tests/frontend-r04.test.js
  tests/frontend-task-012.test.js` passed: 21 tests covering Settings routes
  and r04 plugin, skill, and MCP dialog preservation.
- `node --test --test-concurrency=1 tests/frontend-task-006.test.js` passed:
  14 tests covering provider/model management and per-session model selection.
- `node --test --test-concurrency=1 tests/frontend-task-011.test.js
  tests/frontend-task-011B.test.js` passed: 21 tests covering Terminal and
  chat-session transition behavior.
- `node --test --test-concurrency=1 tests/frontend-connectors.test.js` passed:
  8 tests covering connector contract and payload validation.

## Deferred

- Runtime extension invocation, runtime tool gateway, prompt tag execution, MCP
  invocation, concurrency, restart/resume, memory, audit hardening, and broad
  security/approval policy hardening remain deferred to later frozen tasks.
- Runtime impact is record-visible but disabled until explicit enablement.
- Extension discovery/loading research is documented as a proposed future
  technical reference and work-order, not as active runtime behavior.

## Next Step

Continue with `.agents/specs/mvp/tasks.md` `TASK-013` for concurrency and
restart/resume. Preserve TASK-012 extension records as inert app-owned records
until a later explicit enablement/invocation task wires runtime impact.
