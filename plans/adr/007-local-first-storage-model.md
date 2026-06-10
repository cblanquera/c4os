# ADR-007: Local-First Storage Model

Status: Finalized for MVP. Post-MVP portability, sync, quotas, and audit integrity remain deferred.

## Context

The data model recommends SQLite for structured metadata, a local artifact directory for originals/previews, and OS keychain or platform credential storage for the MVP OpenRouter API key. Requirements say project files, session metadata, artifacts, and logs should live locally by default.

The review flags migration risks: local absolute paths do not port across machines, SQLite may not support future team sync without careful design, and audit logs may not be tamper-evident.

## Decision

Use local-first storage with SQLite metadata, local artifact files, and OS keychain/platform credential storage for the MVP OpenRouter API key.

If keychain or platform credential storage is unavailable or fails, OpenRouter provider setup is blocked with an actionable error. MVP does not provide plaintext secret storage, project-file secret storage, env-file writes, custom encrypted vaults, or secret export/import.

MVP retains app-owned sessions, messages, tool calls, approvals, local diagnostics, logs, and MVP artifacts locally and indefinitely by default.

MVP transcripts are append-only. User and assistant messages cannot be edited or deleted after creation, though runtime-generated records may receive status updates such as `complete`, `stopped`, or `failed`. Users correct prior input through follow-up messages or a new session.

Session delete, message edit/delete, transcript pruning, message redaction UI, branch-from-message, conversation rewrite, automatic cleanup, quota management, export/import, sync, and audit-integrity semantics are post-MVP.

## Alternatives Considered

 - SQLite plus artifact directory.
 - JSON files only.
 - Postgres.

## Alternatives That Should Be Considered

 - Event log plus SQLite projections.
 - Content-addressable artifact store with deduplication.
 - Sync-ready local model with stable machine-local versus portable boundaries.
 - Tamper-evident audit log if compliance/security is a goal.

## Tradeoffs

SQLite is reliable, transactional, inspectable, and appropriate for desktop MVP.

JSON files are easier to inspect and move, but weaker for concurrent sessions and relational queries.

Postgres is more powerful, but too heavy for local desktop MVP.

## Consequences

 - The app must distinguish portable records from machine-local records.
 - Artifact retention is indefinite by default for MVP.
 - Transcript correction happens through follow-up messages, not mutation of prior messages.
 - Quotas and cleanup controls are deferred.
 - Future export must redact secrets and cannot include raw OpenRouter keys unless a later ADR explicitly adds secret export.
 - Provider setup depends on OS keychain/platform credential availability.
 - Future team sync may require schema and identity changes.

## Follow-Up Questions

 - Is the audit ledger for convenience, debugging, compliance, or security?
 - What data is portable by design?
 - What post-MVP artifact quotas, backup, and deletion controls are needed?
 - How are absolute paths represented in future exports?

## ADR Recommendation

Validate SQLite persistence and crash recovery before implementation planning. Revisit portability, sync, quota, export/import, and audit-integrity requirements after MVP.
