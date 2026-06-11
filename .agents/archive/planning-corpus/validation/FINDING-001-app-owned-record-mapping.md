# FINDING-001 App-Owned Record Mapping Evidence

Date: 2026-06-10

## Question

Can OpenCode session IDs, tool IDs, permission IDs, logs, and persistence map
to app-owned canonical records without becoming the source of truth?

## Evidence Sources

- `.agents/archive/planning-corpus/specs/data-model-specification.md`
- `.agents/archive/planning-corpus/adr/008-unified-tool-invocation-and-ledger.md`
- `.agents/archive/planning-corpus/validation/FINDING-001-structured-events.md`
- `.agents/archive/planning-corpus/validation/FINDING-001-pre-execution-interception.md`
- `.agents/archive/planning-corpus/validation/FINDING-001-stop-behavior.md`

The data model already requires that the app owns canonical MVP records in
SQLite. Runtime-native OpenCode session IDs, logs, and persistence paths are
adapter references only.

ADR-008 requires executable capabilities to pass through a unified tool-call
model and tool ledger. OpenCode, MCP, plugin, GUI, shell, Git, and artifact
actions must therefore normalize into app-owned records instead of remaining
runtime-native records.

The live OpenCode probes observed stable structured references for:

- `sessionID`
- `messageID`
- `partID`
- `callID`
- permission request `id`
- permission `metadata`
- tool `state.input`
- tool `state.output`
- tool `state.error`
- session status and error events

## Canonical Ownership Rule

The app generates and owns primary keys for all MVP records:

- `projects.id`
- `sessions.id`
- `messages.id`
- `tool_calls.id`
- `approvals.id`
- `artifacts.id`

OpenCode identifiers are stored only as adapter references in explicit fields
or JSON metadata. They must never be used as primary keys, user-facing record
identity, export identity, or replay authority.

OpenCode logs and persistence directories may be retained as troubleshooting
references only. SQLite remains the source of truth for session history, tool
ledger status, approval decisions, artifacts, and user-facing transcript state.

## Mapping

### Sessions

| App record | OpenCode source | Mapping |
| --- | --- | --- |
| `sessions.id` | app-generated | Canonical session identifier. |
| `sessions.runtime` | adapter config | `opencode`. |
| `sessions.runtime_session_ref` | `session.created.properties.sessionID` or `session.created.properties.info.id` | Runtime reference only. |
| `sessions.project_id` | active app project | App-owned project link. |
| `sessions.model_id` | app-selected model at session start | Canonical selected model, not inferred from ambient OpenCode config. |
| `sessions.agent_ref` | app-selected agent at session start | Canonical selected agent or mode reference. |
| `sessions.status` | `session.status`, `session.idle`, `session.error`, `session.updated` | Normalized app status. |

Recommended session status mapping:

| OpenCode event/state | App status |
| --- | --- |
| `session.status.type = busy` | `running` |
| `session.status.type = idle` or `session.idle` after normal completion | `complete` or `idle`, depending on whether the assistant turn is complete |
| `session.error.name = MessageAbortedError` | `stopped` |
| Other `session.error` | `failed` |

`session.status: idle` is a runtime-settled signal only. It is not sufficient
by itself to prove successful completion after abort or error.

### Messages

| App record | OpenCode source | Mapping |
| --- | --- | --- |
| `messages.id` | app-generated | Canonical message identifier. |
| `messages.session_id` | app session | Canonical session link. |
| `messages.role` | `message.updated.properties.info.role` | Normalized role. |
| `messages.content` | `message.part.delta`, final message parts | User or assistant text accumulated by the adapter. |
| `messages.status` | message/session/tool terminal events | `complete`, `stopped`, or `failed`. |
| `messages.metadata_json` | OpenCode `messageID`, `partID`, model/provider refs | Adapter references and non-authoritative runtime metadata. |

OpenCode `messageID` and `partID` are idempotency inputs and trace references.
They do not replace `messages.id`.

Assistant partial output is retained when stopped. It must be persisted as
`status = stopped`, not deleted and not treated as complete.

### Tool Calls

| App record | OpenCode source | Mapping |
| --- | --- | --- |
| `tool_calls.id` | app-generated | Canonical tool-call identifier. |
| `tool_calls.session_id` | app session | Canonical session link. |
| `tool_calls.message_id` | app assistant message | Canonical assistant-message link. |
| `tool_calls.tool_source` | adapter | `opencode`. |
| `tool_calls.tool_name` | `message.part.updated.properties.part.tool` | Normalized tool name such as `bash`, `write`, or `read`. |
| `tool_calls.arguments_json` | `part.state.input` and permission metadata | Structured tool input. |
| `tool_calls.status` | `part.state.status`, permission replies, session errors, abort metadata | Normalized canonical status. |
| `tool_calls.started_at` | adapter receive time for first pending/running event | Runtime event time if available; otherwise adapter receive time. |
| `tool_calls.completed_at` | adapter receive time for terminal event | Runtime event time if available; otherwise adapter receive time. |
| `tool_calls.working_directory` | session directory, bash metadata, or adapter launch cwd | App-normalized path. |
| `tool_calls.output_summary` | bounded summarized `state.output` | Redacted/truncated summary only. |
| `tool_calls.result_json` | terminal tool state and safe result metadata | Non-secret structured result. |
| `tool_calls.denial_category` | permission rejection/tool error | Normalized denial category. |
| `tool_calls.denial_message` | `state.error` | Safe denial message. |
| `tool_calls.runtime_call_ref` | `part.state.callID` plus `part.id` when needed | Runtime reference only. |

Recommended tool status mapping:

| OpenCode state | App status |
| --- | --- |
| `pending` | `pending_approval` or `pending`, depending on whether a permission request exists |
| `running` before permission answer | `pending_approval` when paired with `permission.asked`; do not treat as execution proof |
| `running` after approval | `running` |
| `completed` after normal output | `complete` |
| `error` after `permission.replied.reply = reject` | `denied` |
| `error` for non-denial runtime/tool failure | `failed` |
| `completed` with `<shell_metadata>User aborted the command</shell_metadata>` | `stopped` |
| session `MessageAbortedError` correlated to active tool | `stopped` |

The pre-execution probe proved that protected side effects did not happen
before `permission.asked` was answered, even though OpenCode emitted a tool
state of `running`. The app adapter must therefore use permission events and
side-effect terminal states as the enforcement boundary, not the word
`running`.

### Approvals

| App record | OpenCode source | Mapping |
| --- | --- | --- |
| `approvals.id` | app-generated | Canonical approval identifier. |
| `approvals.tool_call_id` | app tool call | Link through `permission.tool.callID`, `part.state.callID`, or correlated `part.id`. |
| `approvals.approval_source` | adapter | `opencode-permission`. |
| `approvals.decision` | `permission.replied.properties.reply` | `once`, `reject`, or broader scopes if later enabled. |
| `approvals.scope` | permission reply and app policy | One-time by default for Phase 0 evidence. |
| `approvals.decided_by` | app actor | Current user or policy actor. |
| `approvals.decided_at` | adapter decision time | Runtime event time if available; otherwise adapter receive time. |
| `approvals.expires_at` | app policy | Null for one-time decisions. |

OpenCode permission request IDs are stored in metadata for traceability and
idempotency. The app-owned approval row is the durable decision record.

### Artifacts

| App record | OpenCode source | Mapping |
| --- | --- | --- |
| `artifacts.id` | app-generated | Canonical artifact identifier. |
| `artifacts.project_id` | app project | Canonical project link. |
| `artifacts.session_id` | app session | Canonical session link. |
| `artifacts.tool_call_id` | app tool call | Link to the creating tool call. |
| `artifacts.type` | tool name and output metadata | File, patch, preview, log summary, or other app type. |
| `artifacts.file_path` | app artifact directory | App-owned artifact path, not OpenCode persistence path. |
| `artifacts.metadata_json` | permission metadata, diff, safe output summary, runtime refs | Non-secret runtime metadata. |

Runtime-created files may be represented as artifacts only after the app
accepts and records the creating tool call. OpenCode worktree files and logs
remain external runtime state until normalized into the app artifact directory
or recorded as references.

## Idempotency And Ordering

Event ingestion should be transactional per event or per correlated event
batch. Recommended idempotency keys:

- Session: `runtime = opencode` + `runtime_session_ref`
- Message: `runtime_session_ref` + OpenCode `messageID`
- Message part: `runtime_session_ref` + OpenCode `messageID` + `partID`
- Tool call: `runtime_session_ref` + `callID`, with `partID` as a fallback
- Permission: `runtime_session_ref` + permission request `id`
- Approval reply: permission request `id` + reply value + adapter decision time

Status updates are append-observed but record-mutating in MVP: the canonical
record is created once, then status, timestamps, output summary, and result
fields are updated as terminal evidence arrives.

The adapter must tolerate out-of-order or duplicate SSE events. Duplicate
runtime events must update the same app record, not create duplicate
transcript, tool-call, approval, or artifact rows.

## Non-Goals

This mapping does not prove runtime config isolation, provider/model isolation,
instruction-source control, data-flow policy enforcement, tamper-evident
ledger integrity, or OpenCode config round-trip compatibility.

Those remain separate validation gates or post-MVP decisions.

## Decision

Mark the app-owned record mapping gate as passed for Phase 0.

The implementation contract is clear enough to start adapter design once the
remaining FINDING-001 gates pass:

- Runtime IDs are adapter references only.
- SQLite records are canonical.
- Tool calls and approvals normalize into the app ledger.
- Stopped and denied states have explicit canonical mappings.
- OpenCode logs and persistence are not user-facing source of truth.

FINDING-001 remains blocked until runtime config isolation and
instruction-loading observability are resolved.
