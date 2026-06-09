# Objective

Define safe session persistence, summary, memory, and retention behavior for MVP and later phases.

# Context

Specs include session persistence, summaries, compaction, handoff, and durable memory patterns. Reviews warn that memory can retain sensitive data and stale assumptions.

# Questions To Answer

 - What session data is durable by default?
 - Are summaries created in MVP?
 - Is any memory stored beyond session history?
 - Can users inspect, edit, and delete stored session context?
 - How are sessions migrated across runtime or model changes?
 - What retention policy applies to transcripts, logs, summaries, and artifacts?

# Hypothesis

MVP should persist transcripts and tool records only, with no cross-session memory and no automatic durable memory writes.

# Investigation Plan

 - Inventory session data from specs and data model.
 - Classify data by sensitivity and retention need.
 - Define MVP persistence minimum.
 - Identify deletion and export expectations.
 - Document summary and memory features to defer.

# Success Criteria

 - MVP persistence scope is defined.
 - Memory is classified as MVP, V1, or later.
 - Delete/inspect controls are defined at a research level.
 - Retention risks are documented.

# Decisions Unlocked

 - ADR-020: Session And Memory Management.
 - ADR-007: Local-First Storage Model.
 - ADR-023: Telemetry And Privacy.

# Estimated Effort

1 to 3 architecture/product days.

