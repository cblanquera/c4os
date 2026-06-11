# MVP App-Build And Product QA Report

Date: 2026-06-11

## Environment

- Host: macOS arm64 (`Darwin ... RELEASE_ARM64_T6000 arm64`)
- Rust: `rustc 1.96.0 (ac68faa20 2026-05-25)`
- Cargo: `cargo 1.96.0 (30a34c682 2026-05-25)`
- Build target validated locally: macOS Apple Silicon

## Automated Gate Results

| Gate | Result | Evidence |
| --- | --- | --- |
| Rust unit/integration tests | Pass | `cargo test --manifest-path src-tauri/Cargo.toml` passed: 94 passed, 1 ignored keychain smoke. |
| JS smoke tests | Pass | `npm test` passed: 2 passed. |
| Web build | Pass | `npm run build` passed. |
| Native Tauri build | Pass | `npm run tauri -- build` passed and produced `src-tauri/target/release/c4os`. |

Additional update on 2026-06-11:

- `npm test` passed after adding first-run command-boundary coverage.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed with 117 passed and
  1 ignored keychain smoke after wiring prompt submission through a runtime
  runner boundary and adding OpenCode JSON event parsing coverage.
- `npm run build` passed after replacing the static status grid with the
  first-run workflow UI.
- `npm run tauri -- build` passed after connecting prompt submission to the
  backend OpenCode JSON runner path.

Additional freeze/hang mitigation update on 2026-06-11:

- `npm test` passed after adding the visible first-run workflow changes.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed with 119 passed and
  1 ignored keychain smoke after adding bounded OpenCode process timeout
  coverage and non-interactive permission-policy coverage.
- `npm run build` passed after adding pending-state UI for provider, project,
  and session actions.
- The OpenCode JSON runner now times out stuck prompt runs instead of leaving
  the app indefinitely waiting.
- The runtime config allows passive project inspection tools (`read`, `list`,
  `grep`, `glob`) and denies mutation/shell/network tools in this MVP path so
  OpenCode does not wait on an invisible approval prompt.

Additional error-display recovery update on 2026-06-11:

- `npm test` passed after adding UI-state coverage for Tauri string rejections,
  structured unknown failures, active-session prompt blocking, and active-run
  transcript messaging.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed with 119 passed and
  1 ignored keychain smoke after wiring startup recovery to mark persisted
  active sessions as interrupted.
- `npm run build` passed after routing command errors through normalized UI
  error text and disabling prompt submission for active backend sessions.

Additional OpenCode event parsing update on 2026-06-11:

- Runtime tests pass for the observed OpenCode `tool_use` event shape emitted
  during a real `inspect project` run.
- OpenCode nonzero-exit errors now prefer stderr or structured `session.error`
  messages. If the runtime only emits JSON progress events, the UI receives a
  concise plain-English last-event summary instead of raw JSON payloads.
- Top-level OpenCode `error` events now extract their message payload instead
  of surfacing `Last event: error`.
- Unknown OpenCode error-event shapes now search nested payloads recursively
  and otherwise show one bounded error event for diagnosis.

Additional responsiveness update on 2026-06-11:

- Prompt submission now returns after creating a running app-owned session and
  executes OpenCode on a background worker with its own SQLite connection.
- The frontend polls session status while a backend run is active, so the
  window can remain responsive during long model/runtime calls.

## MVP Journey Coverage

| Journey | Local QA Status | Notes |
| --- | --- | --- |
| Configure provider | Pass by automated coverage and UI surface | Provider readiness, invalid-key fail-closed behavior, credential references, active-session update guard, and the OpenRouter key/model form are covered. |
| Register a local Git project | Pass by automated coverage and UI surface | Temporary Git repository tests cover registration, dirty state, branch/status, root `AGENTS.md` display, and the UI now accepts a Git project path. |
| Start one agent session | Pass by automated coverage; real-provider manual validation still required | The UI accepts a prompt, shows an immediate pending state, blocks duplicate starts while a backend session is active, the backend starts an app-owned session, invokes the OpenCode JSON runner with bounded timeout, parses structured output, persists assistant output, and marks the session complete or failed. |
| Approve or deny an action | Pass by automated coverage | Approval ledger, prompt choices, destructive-command no-session-allow, denials, and policy blocks are covered. |
| Inspect agent changes | Pass by automated coverage | Modified, added, deleted, and untracked file diff states are covered with temporary Git repositories. |
| Resume context | Pass by automated coverage | SQLite persistence tests cover projects, sessions, messages, artifacts, and restart recovery behavior. |
| Recover from failure | Pass by automated coverage | Stop, interrupted restart state, retry-as-append, and failed runtime states are covered. |

## Remaining MVP Validation Gap

The app no longer stops at a static backend status surface or forced
"streaming not connected" failure. It now exposes the first-run controls needed
for OpenRouter setup, project selection, and prompt submission, and the desktop
backend routes prompt submission through the OpenCode JSON runner.

The remaining validation gap is manual end-to-end testing with a real
OpenRouter key, reachable network, and accepted model route. Provider or runtime
failure should now appear as a failed assistant/run record while preserving the
submitted user message.

Manual validation should also confirm the window remains responsive during a
real prompt run and that the pending message appears before the OpenCode call
returns.

Manual validation should confirm rejected Tauri commands render visible notice
text instead of an empty red banner.

## Text-Like Artifact Preview Report

| Artifact Type | Result | Notes |
| --- | --- | --- |
| Plain text | Pass | Stored under app-managed artifact layout and reopened after restart. |
| Markdown | Pass | Stored and listed as passive text-like artifact. |
| Logs | Pass | Stored with secret-shaped lines redacted. |
| Diffs | Pass | Stored and listed as passive text-like artifact. |
| Generated source/config | Pass | Source/config artifact types are accepted and listed. |
| Active HTML/executable content | Pass | Opened only as passive text; no active rendering or rich preview is exposed. |

## Release Platform Matrix

| Platform | Status | Evidence |
| --- | --- | --- |
| macOS Apple Silicon | Pass locally | Native Tauri build and automated QA gates pass on macOS arm64. |
| Windows 11 x64 | Re-scoped out of this MVP release | User approved public MVP release scope without Windows 11 x64 validation on 2026-06-11. |
| Linux x64 | Deferred | Deferred by MVP platform matrix. |

## Release Decision

Implementation and local macOS Apple Silicon validation passed.

Public MVP release may proceed under the approved re-scoped platform target:
macOS Apple Silicon only. Windows 11 x64 compatibility validation remains
recommended before expanding the release scope to Windows users.
