# item-044: TASK-043 Resolve V1 Long-Term Memory Scope

## Status

verified

## Objective

Choose and document whether V1 should add any durable long-term memory now that
retention/delete controls are defined.

## Decision

Accepted `no_durable_memory_v1` on 2026-06-12.

The proposed tier keeps cross-session memory, learned preferences, automatic
summaries, embeddings, memory write prompts, and memory inspect/edit/delete UI
out of V1. Existing session persistence, project JSON export, and archived
session delete remain the only durable history/portability controls.

## Dependencies

- item-043

## Inputs

- `.agents/archive/planning-corpus/acceptance/memory.md`
- `.agents/archive/planning-corpus/spikes/SPIKE-014-memory-session-retention.md`
- `.agents/archive/planning-corpus/decisions/deferred-decisions.md`
- `.agents/archive/planning-corpus/decisions/non-goals.md`
- `.agents/archive/planning-corpus/specs/data-model-specification.md`

## Deliverables

- Accepted, revised, or deferred V1 memory support tier.
- Updated acceptance criteria for the chosen tier.
- Follow-on implementation item only if a narrow tier is accepted.

## Acceptance Criteria

- User explicitly accepts the support tier.
- The tier states whether durable cross-session memory exists.
- The tier states how deleted sessions interact with memory records.
- The tier states whether automatic summaries, embeddings, learned
  preferences, and memory inspection controls are available.

## Verification

- User accepted `no_durable_memory_v1`.
- Follow-on app status and command-boundary verification is tracked in this
  item.

## Follow-On

- No implementation item is required for durable memory creation because the
  accepted tier explicitly keeps durable memory unavailable in V1.
