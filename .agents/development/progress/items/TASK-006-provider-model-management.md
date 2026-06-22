# TASK-006 Provider And Model Management Slice

Status: verified
Updated: 2026-06-22

## Source

- Task: `.agents/specs/mvp/tasks.md` `TASK-006`
- Accepted predecessor:
  `.agents/development/progress/items/TASK-005A-scoped-frontend-state.md`
- Progress manifest: `.agents/development/progress/manifest.md`

## Goal

Replace provider/model mock records behind the accepted Settings and composer
surfaces without changing the r04 shell or TASK-005 chat UX.

## Required Scope

- Support OpenAI-compatible provider profiles.
- Support non-secret provider/model records.
- Show secure API key status without exposing raw key values in frontend
  fixtures, workspace descriptors, logs, source tests, or normal app state.
- Support discovered model records where available and manual model records
  where discovery is not available.
- Support model enablement.
- Support per-session model selection through the accepted composer/settings
  surfaces.
- Preserve TASK-005A scoped DOM update behavior.

## Outputs

- Added `backend/src/provider_models.rs` as the backend-owned provider/model
  record boundary.
- Added OpenAI-compatible provider profile records with `ApiKeyStatus`
  metadata instead of raw credentials.
- Added discovered and manual model records with enablement and active-state
  fields.
- Added backend commands:
  - `list_provider_profiles`
  - `list_provider_models`
  - `save_provider_profile`
  - `set_model_enabled`
  - `set_provider_enabled`
  - `delete_provider_profile`
  - `select_session_model`
- Updated the frontend connector inventory and Tauri connector to call those
  commands.
- Updated the existing Providers settings route to render key status labels
  from backend-owned records.
- Updated the existing Models settings route to show discovered/manual source
  and enablement state through the existing row/switch shape.
- Updated existing composer provider/model pickers to use provider/model
  records instead of hard-coded provider/model rows.
- Updated chat/session send and session creation paths to pass the selected
  model through the existing connector path.
- Updated OpenRouter request construction so the selected model can override
  the previous fixed review default while retaining the same default when no
  model is selected.
- Repaired the accepted Add Provider form so `Save Provider` calls a backend
  provider save command instead of navigating without saving.
- Repaired the Providers settings row controls so `Edit` opens the accepted
  provider form with the selected profile and the row switch calls backend
  provider enablement.
- Removed the placeholder OpenRouter provider row once a real provider profile
  is saved for the same endpoint.
- Preserved provider identity during edit saves so editing a provider updates
  the existing profile instead of creating a second profile.
- Added provider removal from the accepted edit-provider form.
- Refreshed provider models after saving a provider profile so OpenRouter
  discovery results can replace the static manual review records.
- Added OpenAI-compatible model discovery support for provider profiles through
  the provider `/models` endpoint.
- Enabled all discovered provider models by default; users can disable models
  from the existing Models settings route.
- Updated the Models settings route to use only `Enabled` and `Disabled`
  statuses; the previous `Current` label is no longer shown in the model list.
- Added model search and provider filtering to the existing Models settings
  route without adding new routes or panels.
- Added a results-level model enablement control to the existing Models
  settings toolbar; it applies only to the current search/provider-filter
  result set and preserves those model records when filters are cleared.
- Reworked provider/model switches so the visual state and `aria-pressed`
  state update together.
- Restored the composer model-picker header treatment after the provider/model
  popover state changes.
- Bounded the composer model picker with internal scrolling for large enabled
  model sets.
- Removed default provider/model seed records from initial frontend and backend
  state; provider and model settings start empty until connector/backend records
  exist.
- Preserved the composer picker rule that only enabled models are selectable
  for per-session model selection.
- Added a backend process-memory credential slot for the current review
  session so entered API keys can back runtime calls without being serialized
  into frontend state, provider records, workspace descriptors, logs, or tests.
- Updated chat runtime gating so a saved backend provider key can authorize
  OpenRouter runs even when `OPENROUTER_API_KEY` is not set in the environment.
- Added a Settings entry on the start screen and routed start-screen Settings
  back navigation through a provider setup gate when projects exist but no
  provider is configured.
- Kept the provider setup gate menu-free and routed a successful provider save
  from that gate to the new-session/project selector surface.
- Fixed Settings entry origin capture so the provider setup gate sees the route
  that opened Settings rather than the Settings route after hash routing.
- Corrected Settings back routing so Settings opened from the start screen
  returns to the start screen even when no provider exists, while the provider
  setup gate remains scoped to the new-session surface.
- Kept each new chat's draft model selection independent, so a second new chat
  can choose a different model while each created chat remains locked to one
  model.
- Added active chat-session snapshots so each sidebar session owns its own
  thread, Browser record, Files record, Terminal record, and selected model,
  while Settings/provider/model configuration remains global.
- Updated sidebar session switching to swap session-owned content in place
  without replacing the accepted shell.
- Kept new-session draft model choices out of existing chat sessions until the
  new chat is created.
- Fixed provider and model composer popover headers so they stay outside the
  scroll region, align with row text, remove the provider "Select source" label,
  and keep the model provider label beside the back chevron.
- Removed the model/provider composer popover border after final visual review.

## Verification Run

- `cargo test --manifest-path backend/Cargo.toml task_006` passed.
- `node --test tests/server/task-006-provider-models.test.js` passed.
- `node --test tests/frontend-task-006.test.js` passed.
- `node --test tests/frontend-task-005.test.js tests/frontend-task-005A.test.js
  tests/frontend-task-006.test.js` passed.
- `node --test tests/server/task-005-openrouter-chat.test.js
  tests/server/task-006-provider-models.test.js` passed.
- `cargo test --manifest-path backend/Cargo.toml` passed.
- `npm test` passed: 57 tests, 57 pass.
- Follow-up regression after manual review:
  - `node --test tests/server/task-006-provider-models.test.js` passed.
  - `node --test tests/frontend-task-006.test.js` passed.
  - `cargo test --manifest-path backend/Cargo.toml task_006` passed.
- Follow-up regression after placeholder/provider-control/model-refresh review:
  - `node --test tests/server/task-006-provider-models.test.js` passed.
  - `node --test tests/frontend-task-006.test.js` passed.
  - `npm test` passed: 61 tests, 61 pass.
  - `cargo test --manifest-path backend/Cargo.toml` passed: 19 tests, 19 pass.
  - `rustfmt --check backend/src/commands.rs backend/src/mock_data.rs
    backend/src/openrouter.rs backend/src/provider_models.rs` passed.
- Follow-up regression after provider edit/delete and switch review:
  - `node --test tests/frontend-task-006.test.js` passed: 8 tests, 8 pass.
  - `node --test tests/server/task-006-provider-models.test.js` passed: 4
    tests, 4 pass.
  - `cargo test --manifest-path backend/Cargo.toml` passed: 20 tests, 20 pass.
  - `npm test` passed: 64 tests, 64 pass.
  - `rustfmt --check backend/src/commands.rs backend/src/mock_data.rs
    backend/src/openrouter.rs backend/src/provider_models.rs` passed.
  - `git diff --check` passed.
- Follow-up regression after empty-state, saved-key, model-popover, and
  bulk-toggle review:
  - `node --test tests/frontend-task-006.test.js` passed: 11 tests, 11 pass.
  - `node --test tests/server/task-006-provider-models.test.js` passed: 5
    tests, 5 pass.
  - `cargo test --manifest-path backend/Cargo.toml task_006 --
    --test-threads=1` passed: 7 tests, 7 pass.
  - `npm test` passed: 68 tests, 68 pass.
  - `cargo test --manifest-path backend/Cargo.toml` passed: 21 tests, 21 pass.
  - `rustfmt --check backend/src/commands.rs backend/src/mock_data.rs
    backend/src/openrouter.rs backend/src/provider_models.rs` passed.
  - `git diff --check` passed.
- Follow-up regression after start-screen settings gate, per-new-chat model
  selection, and composer popover header review:
  - `node --test tests/frontend-task-006.test.js` passed: 14 tests, 14 pass.
  - `npm test` passed: 71 tests, 71 pass.
  - `cargo test --manifest-path backend/Cargo.toml` passed: 21 tests, 21 pass.
  - `rustfmt --check backend/src/commands.rs backend/src/mock_data.rs
    backend/src/openrouter.rs backend/src/provider_models.rs` passed.
  - `git diff --check` passed.
  - Secret-material scan across `frontend`, `backend`, `tests`, and `.agents`
    passed with no raw key-pattern matches outside the explicit TASK-006 secret
    input fixtures.
- Follow-up regression after start-screen Settings back-route correction:
  - `node --test tests/frontend-task-005A.test.js
    tests/frontend-task-006.test.js` passed: 23 tests, 23 pass.
  - `npm test` passed: 71 tests, 71 pass.
  - `git diff --check` passed.
- Follow-up regression after chat-session state isolation correction:
  - `node --test tests/frontend-task-006.test.js` passed: 14 tests, 14 pass.
  - `node --test tests/frontend-task-005A.test.js
    tests/frontend-task-006.test.js` passed: 23 tests, 23 pass.
  - `npm test` passed: 71 tests, 71 pass.
  - `cargo test --manifest-path backend/Cargo.toml` passed: 21 tests, 21 pass.
  - `rustfmt --check backend/src/commands.rs backend/src/mock_data.rs
    backend/src/openrouter.rs backend/src/provider_models.rs` passed.
  - `git diff --check` passed.
  - Secret-material scan across `frontend`, `backend`, `tests`, and `.agents`
    passed with no raw key-pattern matches outside the explicit TASK-006 secret
    input fixtures.
- Final approved visual-polish pass after removing the composer popover border:
  - `node --test tests/frontend-task-005A.test.js
    tests/frontend-task-006.test.js` passed: 23 tests, 23 pass.
  - `git diff --check` passed.

## Still Out Of Scope

- Files content and mutation.
- Browser behavior.
- Terminal behavior.
- Extensions, plugins, skills, and MCP runtime behavior.
- Local memory.
- Artifacts.
- Broad restart/resume.
- Advanced approval policy.
- Audit-log hardening.
- Non-chat feature slices.
- Durable provider persistence and OS keychain storage; this slice exposes
  secure key status, keeps entered keys in backend process memory for the
  current review session, and avoids secret leakage, but deeper credential
  hardening remains a later security-policy task.

## Acceptance

TASK-006 is approved. TASK-007 may start from this verified provider/model
management boundary.
