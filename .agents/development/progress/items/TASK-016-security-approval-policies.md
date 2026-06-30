# TASK-016 Security And Approval Policies

Status: approved
Updated: 2026-06-30

## Source

- Frozen task basis: `.agents/specs/mvp/tasks.md` `TASK-016`
- Accepted predecessor:
  `.agents/development/progress/items/TASK-015-pause-at-feature-complete.md`
- Gateway predecessor:
  `.agents/development/progress/items/WO-006-runtime-tool-gateway-refactor.md`
- Technical context: `.agents/context/technical-specs.md`
- Runtime gateway reference:
  `.agents/references/context/technical-specs/runtime-adapter.md`
- Work orders: `.agents/context/work-orders.md` `WO-006`, `WO-008`

## Goal

Harden the feature-complete MVP product surface against the C4OS-owned runtime
tool gateway by adding formal tool approval policy storage, effective policy
snapshots, trusted-root and authority enforcement, approval gates, extension
runtime gates, key-storage policy, and security audit records without changing
the accepted UI surface.

## Required Scope

- Preserve accepted TASK-011/TASK-011B behavior: no frontend prompt-text
  command parser, and structured command execution belongs behind the runtime
  tool gateway.
- Harden real backend behavior for Files, Browser, Terminal, artifact preview,
  provider key storage, extension records, action records, and audit records.
- Provision workspace/project default tool policy, optional session override
  handling, and run-scoped effective-policy snapshots.
- Map stable tool identities to enabled state, access, and approval policy.
- Keep session overrides unable to silently widen beyond workspace policy or
  tool maximum authority.

## Outputs

- Added `backend/src/security.rs` as the backend-owned policy layer.
- Added `ToolApprovalPolicyStore`, `WorkspaceToolPolicy`,
  `SessionToolPolicyOverride`, `EffectiveToolPolicySnapshot`, and
  `SecureKeyStoragePolicy`.
- Added default tool policy for stable identities:
  `terminal.run`, `files.read`, `files.write`, `browser.open`, and
  `artifact.preview`.
- Added persisted policy storage through `C4OS_TOOL_POLICY_STORE`, with
  workspace defaults scoped to the active trusted workspace when available.
- Added effective-policy snapshot creation and persistence for gateway tool
  requests.
- Routed gateway dispatch through `security::enforce_tool_request`, recording
  allowed/rejected tool audit events.
- Added explicit approval gates for `ask_every_time` tools and terminal command
  policy; direct accepted UI actions pass explicit approval markers, while
  runtime-originated structured calls without approval are rejected.
- Preserved Browser local-file authority for user-entered absolute local paths:
  users may open any existing local file outside `.git`, while agent-entered
  local Browser paths remain trusted-root scoped.
- Preserved the separate provider-key store fallback so existing provider
  access survives restart when `C4OS_PROVIDER_KEY_STORE` is not explicitly set.
  Raw keys remain outside workspace descriptors, provider/model records, and
  normal app records.
- Reconciled provider key status against the secure key store and
  `OPENROUTER_API_KEY` on provider-profile reads so stale non-secret settings
  cannot claim `Key configured` after the key-store file is missing.
- Added security audit recording through the app-owned audit store.
- Added extension runtime enablement gate coverage while preserving existing
  inert TASK-012 extension records.
- Exposed `load_tool_approval_policy` and `save_tool_approval_policy` backend
  commands and registered them in the Tauri command handler.
- Added follow-up native Edit menu focus handling so search fields, prompt,
  terminal input, and Browser address enable OS Edit commands without enabling
  File Save/Revert.
- Updated Browser test expectations to preserve user local-file browsing while
  keeping agent local Browser paths scoped.
- Kept WO-008 out of scope: assistant prose or markdown output is still not
  treated as structured `terminal.run` output.

## Verification

- Red test confirmed missing TASK-016 policy surface before implementation:
  `node --test --test-concurrency=1
  tests/server/task-016-security-approval.test.js` failed on missing
  `backend/src/security.rs`.
- `node --test --test-concurrency=1
  tests/server/task-016-security-approval.test.js` passed: 2 tests.
- `cargo test --manifest-path backend/Cargo.toml task_016 --
  --test-threads=1` passed: 5 tests.
- `cargo test --manifest-path backend/Cargo.toml wo_006 --
  --test-threads=1` passed: 3 tests.
- `cargo test --manifest-path backend/Cargo.toml task_006 --
  --test-threads=1` passed: 7 tests.
- `cargo test --manifest-path backend/Cargo.toml task_014 --
  --test-threads=1` passed: 2 tests.
- Follow-up regression tests reproduced and fixed provider/model access loss
  after restart, user-local Browser path rejection, and OS Edit menu disabled
  state on non-file-editor editable controls.
- Follow-up `cargo test --manifest-path backend/Cargo.toml task_006 --
  --test-threads=1` passed: 7 tests.
- Follow-up `cargo test --manifest-path backend/Cargo.toml task_010 --
  --test-threads=1` passed: 7 tests.
- Follow-up `cargo test --manifest-path backend/Cargo.toml task_016 --
  --test-threads=1` passed: 8 tests.
- Follow-up `node --test --test-concurrency=1 tests/frontend-task-008.test.js
  tests/frontend-task-011.test.js` passed with loopback permission: 20 tests.
- Follow-up built-app manual QA launched
  `npm run backend:run:qa -- --workspace-file
  tests/projects/workspace.c4os.json` on 2026-06-30. The app opened
  `#new-session`, the composer still showed the saved model
  `sakana/fugu-ultra`, Settings still showed `OpenRouter - Personal`, and the
  provider row correctly changed to `Key not configured` because
  `$TMPDIR/c4os-provider-keys.json` was absent and `OPENROUTER_API_KEY` was
  unset. No raw key was entered or displayed during QA.
- Follow-up `cargo test --manifest-path backend/Cargo.toml --
  --test-threads=1` passed: 66 tests.
- Relevant server regression bundle passed: 24 tests across TASK-006,
  TASK-008, TASK-010B, TASK-011, TASK-012, TASK-013, TASK-013A, TASK-014,
  WO-006, and TASK-016.
- Relevant frontend browser regression bundle required loopback permission after
  sandbox `listen EPERM`, then passed: 67 tests across TASK-006, TASK-008,
  TASK-010A, TASK-010B, TASK-010C, TASK-011, TASK-011B, TASK-012, and
  TASK-013A.
- `cargo test --manifest-path backend/Cargo.toml -- --test-threads=1`
  passed: 67 tests.
- `cargo fmt --manifest-path backend/Cargo.toml -- --check` passed.
- `git diff --check` passed.

## Deferred Gaps

- WO-008 remains deferred to TASK-017: runtime/provider output must still emit
  structured C4OS tool lifecycle events before Agent terminal output can
  reflect real `terminal.run` calls from the runtime path.
- No new approval UI was added. TASK-016 hardens backend policy, persistence,
  gateway enforcement, and records against the accepted feature-complete
  surface.
- Browser downloads, file create/delete/rename, extension invocation, MCP
  execution, and release packaging remain out of scope for this item.

## Next Step

Proceed to `.agents/specs/mvp/tasks.md` `TASK-017` for remaining integration
and release readiness, including the deferred WO-008 structured runtime
tool-output reflection gap.
