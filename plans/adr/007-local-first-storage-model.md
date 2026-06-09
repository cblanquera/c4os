# ADR-007: Local-First Storage Model

Status: Provisional.

## Context

The data model recommends SQLite for structured metadata, a local artifact directory for originals/previews, and OS keychain or encrypted vault storage for secrets. Requirements say project files, session metadata, artifacts, and logs should live locally by default.

The review flags migration risks: local absolute paths do not port across machines, SQLite may not support future team sync without careful design, and audit logs may not be tamper-evident.

## Decision

Use local-first storage with SQLite metadata, local artifact files, and OS keychain/encrypted vault secrets for MVP.

This decision is provisional because retention, portability, sync, and audit-integrity semantics are unresolved.

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
 - Artifact retention and quotas must be defined.
 - Export must redact secrets.
 - Future team sync may require schema and identity changes.

## Follow-Up Questions

 - Is the audit ledger for convenience, debugging, compliance, or security?
 - What data is portable by design?
 - How are artifact quotas, retention, backup, and deletion handled?
 - How are absolute paths represented in exports?

## ADR Recommendation

Keep this ADR provisional and resolve storage lifecycle requirements before implementation planning.

