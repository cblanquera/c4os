# Objective

Validate the local storage model for session persistence, artifact storage, audit ledger needs, and future portability.

# Context

Specs recommend SQLite plus local artifact files and OS keychain secrets. Reviews warn about audit integrity, path portability, artifact growth, and future sync.

# Questions To Answer

 - Is SQLite sufficient for active session, transcript, tool ledger, and artifact metadata writes?
 - What data is portable versus machine-local?
 - Is the tool ledger for convenience, debugging, compliance, or security?
 - Is tamper-evidence required?
 - How should absolute paths be represented in exports?
 - What retention, quota, and garbage-collection policy is needed?
 - How are secrets excluded from exports and logs?

# Hypothesis

SQLite is sufficient for MVP if audit is explicitly non-compliance-grade and artifact retention is simple, but future sync requires portable IDs and path abstraction now.

# Investigation Plan

 - Model expected MVP write patterns and storage growth.
 - Evaluate SQLite concurrency assumptions for one active session.
 - Define portable versus local fields.
 - Review audit integrity options.
 - Draft storage lifecycle constraints.
 - Identify export/import blockers.

# Success Criteria

 - Storage model risk is classified for MVP and V1.
 - Audit assurance level is recommended.
 - Portability boundaries are documented.
 - Artifact retention assumptions are defined.

# Decisions Unlocked

 - ADR-007: Local-First Storage Model.
 - ADR-008: Unified Tool Invocation And Ledger.
 - ADR-020: Session And Memory Management.

# Estimated Effort

2 to 4 engineering days.

