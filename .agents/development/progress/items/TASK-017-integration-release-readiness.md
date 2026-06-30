# TASK-017 Integration And Release Readiness

Status: accepted
Updated: 2026-06-30

## Source

- Frozen task basis: `.agents/specs/mvp/tasks.md` `TASK-017`
- Technical context: `.agents/context/technical-specs.md`
- Runtime gateway reference:
  `.agents/references/context/technical-specs/runtime-adapter.md`
- Work order: `.agents/context/work-orders.md` `WO-008`
- Accepted predecessors:
  - `.agents/development/progress/items/WO-006-runtime-tool-gateway-refactor.md`
  - `.agents/development/progress/items/TASK-016-security-approval-policies.md`

## Goal

Resolve the deferred `WO-008` structured runtime tool-output reflection gap
without changing the accepted Browser, Files, Terminal, provider/model,
session, or approval surfaces.

Runtime/provider command execution must arrive as structured C4OS tool
lifecycle events. `terminal.run` must execute through the C4OS tool gateway,
and Agent terminal output must reflect structured terminal tool events only.
Assistant prose or markdown remains invalid as an Agent terminal source of
truth.

## Outputs

- Extended `RuntimeEvent` with optional structured `tool` and `args` metadata.
- Added OpenRouter request tool schema for the stable C4OS `terminal.run`
  identity so provider runs have a structured tool-call target.
- Normalized provider streamed `tool_calls` into C4OS `tool_call_requested`
  runtime events, including chunked tool-call argument assembly across SSE
  deltas.
- Added session-layer handling for structured `terminal.run` events:
  - dispatches through `dispatch_tool_call`
  - passes the current C4OS session and run IDs
  - records lifecycle results back onto the run
  - appends `tool_output_delta` only from gateway terminal output
  - records `tool_call_rejected` when TASK-016 approval enforcement rejects
    an unapproved runtime-originated terminal request
- Preserved the TASK-011/TASK-011B boundary:
  - no frontend prompt-text command parser was restored
  - assistant prose/markdown is still not mirrored into the Agent terminal
  - prompt text such as `run ls` continues through runtime prompt handling
    unless a structured tool event is returned
- Preserved TASK-016 enforcement by keeping runtime-originated terminal
  requests behind the existing gateway/security policy. Unapproved
  `terminal.run` tool events are rejected and do not update Agent terminal
  output.
- Updated frontend prompt-session reconciliation so backend Agent terminal
  state is accepted only when the returned session contains structured Agent
  terminal action output. The existing `No structured agent terminal calls for
  this run.` behavior is preserved for prompt runs without terminal tool
  events.
- Added tool-only chat reflection for provider runs that return structured
  `terminal.run` output but no assistant prose. The chat transcript now reports
  the structured `tool_output_delta` result instead of preserving the provider
  fallback `OpenRouter returned no assistant content.`
- Fixed repeated-prompt sidebar reconciliation so multiple sessions with the
  same prompt title remain distinct backend session rows. Optimistic `local-*`
  placeholder rows are replaced by backend session IDs, and the selected
  Terminal tab is preserved when the optimistic session surface reconciles to
  the backend session surface.
- Added a browser-local terminal-state refresh event so direct
  `runConnectorTerminalCommand(..., "agent")` data-layer calls repaint the
  visible Agent terminal pane without using assistant prose or prompt text as
  terminal source of truth.
- Tightened the TASK-013A frontend test assertion to ignore unrelated native
  menu-state sync calls while preserving the recent-workspace open-file
  contract.
- Added focused TASK-017 tests for:
  - provider `tool_calls` normalization
  - chunked provider tool-call argument assembly
  - OpenRouter `terminal.run` tool advertising
  - structured runtime `terminal.run` execution through the gateway
  - unapproved runtime terminal tool rejection
  - tool-only structured terminal output reflected into chat
  - frontend/source boundary for structured Agent terminal reflection
  - repeated prompt titles staying distinct in the project sidebar

## Verification

- Red test confirmed `RuntimeEvent` lacked structured `tool` and `args`
  metadata before implementation:
  `cargo test --manifest-path backend/Cargo.toml
  task_017_runtime_terminal_tool_event_updates_agent_terminal_through_gateway
  -- --test-threads=1` failed to compile on missing fields.
- Red test confirmed provider `tool_calls` were ignored before normalization:
  `cargo test --manifest-path backend/Cargo.toml
  task_017_normalizes_provider_tool_calls_to_c4os_lifecycle_events --
  --test-threads=1` failed with zero events.
- Red test confirmed OpenRouter requests did not advertise C4OS tools before
  the request schema change:
  `cargo test --manifest-path backend/Cargo.toml
  task_017_openrouter_request_advertises_c4os_terminal_tool --
  --test-threads=1` failed on missing `tools`.
- Red test confirmed the frontend merge path did not accept structured Agent
  terminal output from returned session records:
  `node --test --test-concurrency=1
  tests/server/task-017-runtime-tool-events.test.js` failed on missing
  `sessionHasRuntimeAgentTerminal`.
- `cargo test --manifest-path backend/Cargo.toml task_017 --
  --test-threads=1` passed: 6 tests.
- `node --test --test-concurrency=1
  tests/server/task-017-runtime-tool-events.test.js` passed: 1 test.
- Follow-up red test confirmed tool-only structured terminal output was not
  reflected into chat before the fix:
  `cargo test --manifest-path backend/Cargo.toml
  task_017_tool_only_terminal_run_reports_structured_output_in_chat --
  --test-threads=1` failed because the chat still contained the provider
  fallback instead of `tool-chat-output`.
- Follow-up red test confirmed repeated prompt titles still left an optimistic
  local placeholder in the sidebar before reconciliation/render fixes:
  `node --test --test-concurrency=1 --test-name-pattern
  "keeps repeated prompt sessions" tests/frontend-task-011B.test.js` failed
  with `local-repeat-this-prompt` present instead of two backend session IDs.
- Backend regression shards passed:
  - `cargo test --manifest-path backend/Cargo.toml task_006 --
    --test-threads=1`: 7 tests.
  - `cargo test --manifest-path backend/Cargo.toml task_008 --
    --test-threads=1`: 4 tests.
  - `cargo test --manifest-path backend/Cargo.toml task_010 --
    --test-threads=1`: 7 tests.
  - `cargo test --manifest-path backend/Cargo.toml task_011 --
    --test-threads=1`: 4 tests.
  - `cargo test --manifest-path backend/Cargo.toml task_013 --
    --test-threads=1`: 12 tests.
  - `cargo test --manifest-path backend/Cargo.toml task_014 --
    --test-threads=1`: 2 tests.
  - `cargo test --manifest-path backend/Cargo.toml wo_006 --
    --test-threads=1`: 3 tests.
  - `cargo test --manifest-path backend/Cargo.toml task_016 --
    --test-threads=1`: 8 tests.
- Server regression bundle passed: 23 tests across TASK-006, TASK-008,
  TASK-010B, TASK-011, TASK-013, TASK-013A, TASK-014, TASK-016, TASK-017, and
  WO-006.
- Relevant frontend browser regression bundle passed with loopback permission:
  64 tests across TASK-006, TASK-008, TASK-010A, TASK-010B, TASK-010C,
  TASK-011, TASK-011B, and TASK-013A.
- Follow-up after live provider QA chunked-stream fix:
  `cargo test --manifest-path backend/Cargo.toml -- --test-threads=1`
  passed: 73 tests.
- Follow-up focused frontend regression bundle passed with loopback
  permission: 26 tests across TASK-011, TASK-011B, and TASK-013A.
- Follow-up duplicate-title focused browser regression passed:
  `node --test --test-concurrency=1 --test-name-pattern
  "keeps repeated prompt sessions" tests/frontend-task-011B.test.js`.
- Follow-up full TASK-011B browser regression passed:
  `node --test --test-concurrency=1 tests/frontend-task-011B.test.js`: 13
  tests.
- Requested release-readiness regression subset passed with loopback
  permission:
  `node --test --test-concurrency=1 tests/frontend-task-006.test.js
  tests/frontend-task-008.test.js tests/frontend-task-010A.test.js
  tests/frontend-task-010B.test.js tests/frontend-task-010C.test.js
  tests/frontend-task-011.test.js tests/frontend-task-011B.test.js
  tests/frontend-task-013A.test.js tests/server/task-006-provider-models.test.js
  tests/server/task-008-files.test.js tests/server/task-010b-native-browser.test.js
  tests/server/task-011-terminal.test.js
  tests/server/task-013-concurrency-resume.test.js
  tests/server/task-013a-desktop-qa-bootstrap.test.js
  tests/server/task-014-records.test.js
  tests/server/wo-006-tool-gateway.test.js
  tests/server/task-016-security-approval.test.js
  tests/server/task-017-runtime-tool-events.test.js`: 88 tests.
- `cargo fmt --manifest-path backend/Cargo.toml -- --check` passed.
- `git diff --check` passed.

## Manual QA

Credentialed built-app QA passed on 2026-06-30 with OpenRouter credentials
supplied by environment and no raw key recorded in progress notes.

- Launch command shape:
  `npm run backend:run:qa -- --workspace-file
  tests/projects/workspace.c4os.json`
- Isolated stores:
  - `C4OS_SESSION_STORE=/tmp/c4os-task-017-qa-sessions.json`
  - `C4OS_PROVIDER_STORE=/tmp/c4os-task-017-provider-settings.json`
  - `C4OS_PROVIDER_KEY_STORE=/tmp/c4os-task-017-provider-keys.json`
- Model: `google/gemini-2.5-flash-lite`.
- Route sequence: built app opened `#new-session` for `project-a`, prompt
  submission created a chat session at `#chat-session`, and the Terminal tab
  remained in the accepted right-panel surface.
- Prompt asked the provider to request `terminal.run` for `ls` with
  `terminalKind: "agent"` and approval evidence.
- First live pass showed the provider streamed a `terminal.run` request with
  incomplete arguments; TASK-016 correctly rejected it and the Agent terminal
  stayed at `No structured agent terminal calls for this run.`
- After adding chunked tool-call argument assembly, the second live pass
  persisted:
  - `tool_call_requested` for `terminal.run` with structured args
  - `tool_call_started`
  - `tool_output_delta`
  - `tool_call_completed`
- The visible Agent terminal showed:
  `$ ls`
  `completed with no output`
- The trusted cwd was
  `tests/projects/project-a`. That fixture directory is empty, so the no-output
  command result is expected.
- Backend logs showed normal build/run output and no panic.

## Deferred Gaps

- Runtime-originated terminal execution still requires an approval marker to
  satisfy TASK-016 policy. No new approval UI was added in TASK-017.
- Browser downloads, file create/delete/rename, extension invocation, MCP
  execution, packaging/distribution signing, and full external release
  packaging remain outside this item.

## Next Step

Prepare the MVP release-readiness closeout or the next packaging/signing work
item if distribution packaging is required. Keep any future provider/tool
follow-up behind structured C4OS lifecycle events and the TASK-016 gateway
approval boundary.
