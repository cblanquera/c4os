# Data Model Specification

## Storage Recommendation

Use SQLite for structured metadata and a local artifact directory for files and previews.

The app owns canonical MVP records. Runtime-native OpenCode session IDs, logs, or persistence paths are adapter references only and must not become the source of truth for user-facing history.

MVP settings may store app-owned OpenCode launch settings and adapter references required to invoke the runtime. They do not represent OpenCode config import, mirror, edit, export, or round-trip compatibility.

MVP retains app-owned sessions, messages, tool calls, approvals, local diagnostics, logs, and MVP artifacts locally and indefinitely by default. Session delete, automatic cleanup, quota management, export/import, and long-term memory are post-MVP.

Session transcripts are append-only in MVP. User and assistant messages are not edited or deleted after creation. Runtime-generated records may receive status updates such as `complete`, `stopped`, or `failed`.

Alternatives considered:
 - JSON files: portable, but weak for concurrent sessions.
 - Postgres: powerful, unnecessary for local desktop MVP.

Why selected: SQLite is durable, queryable, transactional, and local-first.

## MVP Core Tables

### projects

 - `id`
 - `name`
 - `root_path`
 - `default_model`
 - `default_agent_ref`
 - `created_at`
 - `updated_at`

### sessions

 - `id`
 - `project_id`
 - `title`
 - `status`
 - `mode`
 - `agent_ref`
 - `model_id`
 - `runtime`
 - `runtime_session_ref`
 - `created_at`
 - `updated_at`

### messages

 - `id`
 - `session_id`
 - `role`
 - `content`
 - `status`
 - `metadata_json`
 - `created_at`

### tool_calls

 - `id`
 - `session_id`
 - `message_id`
 - `tool_source`
 - `tool_name`
 - `arguments_json`
 - `status`
 - `started_at`
 - `completed_at`
 - `exit_code`
 - `working_directory`
 - `output_summary`
 - `output_truncated`
 - `redaction_applied`
 - `output_summary_reason_labels`
 - `context_source_summary`
 - `result_json`
 - `denial_category`
 - `denial_message`
 - `risk_level`
 - `runtime_call_ref`

### approvals

 - `id`
 - `tool_call_id`
 - `approval_source`
 - `decision`
 - `scope`
 - `decided_by`
 - `decided_at`
 - `expires_at`

### artifacts

 - `id`
 - `project_id`
 - `session_id`
 - `tool_call_id`
 - `type`
 - `title`
 - `mime_type`
 - `file_path`
 - `preview_path`
 - `metadata_json`
 - `created_at`

### models

 - `id`
 - `provider`
 - `model_slug`
 - `display_name`
 - `capabilities_json`
 - `pricing_json`
 - `context_window`
 - `enabled`

### settings

 - `id`
 - `key`
 - `value_json`
 - `created_at`
 - `updated_at`

MVP settings are app-owned. OpenCode-related settings are adapter inputs and references, not an OpenCode config compatibility layer.

MVP stores the OpenRouter API key only in the OS keychain or platform credential store. SQLite stores only non-secret readiness state or keychain references for provider configuration. Raw provider keys, env-file contents, and exported secrets are excluded from MVP metadata.

Running sessions keep the provider credential reference captured at session start. Provider credential update or revoke is allowed only when no session is running, so MVP does not need versioned credential history, hot key rotation state, or mid-call credential retry records.

Model-call records store bounded context-source summaries, not raw prompt exports. The summary identifies source categories such as active transcript, selected model/routing metadata, explicit file reads, and tool summaries. Shell tool context uses only the persisted bounded redacted/truncated summary, output-omitted marker, and safe output summary reason labels; live raw terminal buffers, omitted raw shell output, redacted substrings, sensitive raw byte counts, offsets, hashes, and reconstruction metadata are excluded. Token-by-token context inspection and editable context payload storage are post-MVP.

Assistant messages can be marked `complete`, `stopped`, or `failed` in MVP. When the user stops generation, any partial assistant content is retained with `status = stopped` rather than deleted, regenerated, or treated as complete.

Message edit/delete fields and transcript rewrite metadata are intentionally excluded from MVP. Corrections are represented as follow-up messages, not mutations of prior messages.

## Post-MVP Tables

These tables are intentionally excluded from the MVP schema:

 - `worktrees`
 - `skills`
 - `plugins`
 - `mcp_servers`
 - `policies`
 - child-session relationship fields
 - worktree relationship fields

## Artifact Directory Layout

```text
app-data/
  artifacts/
    <project-id>/
      <artifact-id>/
        original
        preview
        metadata.json
  logs/
    tool-calls/
```

MVP tool-call logs store redacted/truncated shell output summaries by default. Live terminal output may be visible while a command is running or while the live terminal buffer remains open, and an open drawer may remain temporarily visible after completion in the same app session when labeled live/ephemeral. Live terminal buffers are not retained after navigation away, reload, app close, or session restore. App-owned persisted tool-call records remain bounded summaries. If a safe summary cannot be produced, the persisted record stores command metadata and an explicit output-omitted marker instead of raw stdout/stderr. Tool-call records may store safe output summary reason labels such as `truncated_by_size`, `redacted_secret_pattern`, and `output_omitted_safe_summary_failed`, but must not store redacted substrings, sensitive raw byte counts, offsets, hashes, or other reconstruction paths to raw output. Normal OS text selection/copy of visible summaries is acceptable, but unlimited raw stdout/stderr capture, retained live terminal buffers, raw-output fallback after summary failure, dedicated raw-output copy controls, and raw log export are post-MVP.

## Export Format

Export should include:
 - Project metadata.
 - Sessions and messages.
 - Tool ledger.
 - Artifacts.
 - Redacted secret references.

Do not export raw secrets.
Do not export raw shell stdout/stderr in MVP.
