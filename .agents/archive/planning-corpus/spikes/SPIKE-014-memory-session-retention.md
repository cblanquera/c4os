# Objective

Document MVP retention behavior and define post-MVP memory/cleanup questions.

# Context

MVP retains app-owned sessions, messages, tool calls, approvals, local diagnostics, logs, and MVP artifacts locally and indefinitely by default. MVP has no session delete, long-term memory, cross-session memory, automatic summaries, compaction, handoff, quotas, export/import, or automatic cleanup. Reviews warn that retained history and future memory can accumulate sensitive data and stale assumptions.

# Questions To Answer

 - What exact MVP records are retained indefinitely by default?
 - What inspect, edit, and delete controls are required before post-MVP memory?
 - How are sessions migrated across runtime or model changes?
 - What quotas, cleanup, export/import, and retention controls are required after MVP?

# Hypothesis

MVP should persist sessions, messages, tool records, approvals, local diagnostics, logs, and MVP artifacts indefinitely by default, with no session delete, no cross-session memory, no automatic summaries, and no automatic durable memory writes.

# Investigation Plan

 - Inventory session data from specs and data model.
 - Classify data by sensitivity and retention need.
 - Document MVP retained record classes.
 - Document session delete, cleanup, quota, export/import, summary, and memory features to defer.

# Success Criteria

 - MVP persistence scope is defined.
 - Memory is classified as MVP, V1, or later.
 - Session delete, cleanup, quota, and export/import controls are classified as V1 or later.
 - Retention risks are documented.

# Decisions Unlocked

 - ADR-020: Session And Memory Management.
 - ADR-007: Local-First Storage Model.
 - ADR-023: Telemetry And Privacy.

# Estimated Effort

1 to 3 architecture/product days.
