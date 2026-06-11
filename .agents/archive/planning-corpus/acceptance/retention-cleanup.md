# V1 Retention And Cleanup Acceptance

## Proposed Support Tier

`archived_session_delete_only`

Accepted on 2026-06-12.

## Scope

V1 may add an explicit, user-initiated delete flow for archived sessions only.
The flow may remove the selected archived session's app-owned session record,
messages, tool calls, approval ledger entries, diagnostics linked only to that
session, and app-managed artifact metadata/files linked only to that session.

## Required Behavior

Given a session is archived and not pinned
When the user explicitly selects delete and confirms the destructive action
Then the app may delete that archived session and session-owned records.

Given a session is active, latest, running, pending approval, or pinned
When delete is requested
Then deletion is blocked with a structured reason and no records are removed.

Given a delete operation succeeds
When the project session catalog is refreshed
Then the deleted session is absent and the selected session falls back to a
remaining valid project session.

Given artifact files are shared by another session or are outside app-managed
artifact storage
When an archived session is deleted
Then those files are not removed.

## Out Of Scope

- Automatic retention cleanup.
- Storage quotas.
- Message-level edit, delete, prune, or redaction UI.
- Transcript rewrite, branch, regenerate, or compaction controls.
- Delete for active, latest, running, pending-approval, or pinned sessions.
- Cross-project cleanup.
- Provider credential deletion.
- Project deletion.
- Raw shell output cleanup beyond existing retained summaries.
- Import or round-trip compatibility.
- Long-term memory or cross-session memory.

## Failure Conditions

- A non-archived session can be deleted.
- A pinned session can be deleted without first unpinning.
- A running session or session waiting for approval can be deleted.
- Delete removes another session's records.
- Delete removes project files, external local paths, provider credentials, or
  artifact files not owned exclusively by the deleted session.
- Delete edits, rewrites, or partially mutates remaining transcript records.
- The UI implies automatic cleanup, quotas, memory controls, import, or
  round-trip portability exists.
