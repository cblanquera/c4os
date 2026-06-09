# Data Model Specification

## Storage Recommendation

Use SQLite for structured metadata and a local artifact directory for files and previews.

Alternatives considered:
 - JSON files: portable, but weak for concurrent sessions.
 - Postgres: powerful, unnecessary for local desktop MVP.

Why selected: SQLite is durable, queryable, transactional, and local-first.

## Core Tables

### projects

 - `id`
 - `name`
 - `root_path`
 - `trust_level`
 - `default_model`
 - `default_agent`
 - `created_at`
 - `updated_at`

### sessions

 - `id`
 - `project_id`
 - `parent_session_id`
 - `title`
 - `status`
 - `mode`
 - `agent_id`
 - `model_id`
 - `worktree_id`
 - `created_at`
 - `updated_at`
 - `archived_at`

### messages

 - `id`
 - `session_id`
 - `role`
 - `content`
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
 - `stdout_path`
 - `stderr_path`
 - `result_json`
 - `risk_level`

### approvals

 - `id`
 - `tool_call_id`
 - `policy_source`
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

### worktrees

 - `id`
 - `project_id`
 - `session_id`
 - `path`
 - `branch`
 - `base_ref`
 - `status`
 - `cleanup_policy`
 - `created_at`
 - `removed_at`

### skills

 - `id`
 - `scope`
 - `source_path`
 - `name`
 - `description`
 - `plugin_id`
 - `metadata_json`
 - `indexed_at`

### plugins

 - `id`
 - `scope`
 - `name`
 - `version`
 - `source_type`
 - `source_uri`
 - `manifest_path`
 - `status`
 - `permissions_json`
 - `installed_at`
 - `enabled_at`

### mcp_servers

 - `id`
 - `project_id`
 - `plugin_id`
 - `name`
 - `transport`
 - `config_json`
 - `status`
 - `last_seen_at`

### models

 - `id`
 - `provider`
 - `model_slug`
 - `display_name`
 - `capabilities_json`
 - `pricing_json`
 - `context_window`
 - `enabled`

### policies

 - `id`
 - `scope`
 - `scope_id`
 - `policy_json`
 - `created_at`
 - `updated_at`

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

## Export Format

Export should include:
 - Project metadata.
 - Sessions and messages.
 - Tool ledger.
 - Artifacts.
 - Skills/plugins references.
 - Redacted secret references.

Do not export raw secrets.
