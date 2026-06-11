# item-042: TASK-041 Resolve V1 Retention And Session Delete Scope

## Status

verified

## Objective

Choose and document whether V1 should add narrow retention/delete controls
after session rename, pin, archive, raster artifact preview, and project JSON
export are verified.

## Decision

Accepted `archived_session_delete_only` on 2026-06-12.

The proposed tier allows explicit user-initiated deletion for archived,
unpinned sessions only. Active, latest, running, pending-approval, and pinned
sessions remain protected. Automatic cleanup, quotas, message-level deletion,
redaction, transcript rewrite, memory, import, and round-trip compatibility
remain deferred.

## Dependencies

- item-030
- item-041

## Inputs

- `.agents/progress/items/item-030.md`
- `.agents/archive/planning-corpus/acceptance/sessions.md`
- `.agents/archive/planning-corpus/acceptance/retention-cleanup.md`
- `.agents/archive/planning-corpus/spikes/SPIKE-014-memory-session-retention.md`
- `.agents/archive/planning-corpus/specs/data-model-specification.md`
- `.agents/archive/planning-corpus/reviews/final-implementation-readiness-review.md`

## Deliverables

- Accepted, revised, or deferred V1 retention/delete support tier.
- Updated acceptance criteria for the chosen tier.
- Follow-on implementation item only if a narrow tier is accepted.

## Acceptance Criteria

- User explicitly accepts the support tier.
- The tier states what records may be deleted.
- The tier states which sessions are protected from deletion.
- The tier states that automatic cleanup, quotas, memory, import, and
  round-trip compatibility remain out of scope unless explicitly accepted.

## Verification

- User accepted `archived_session_delete_only`.

## Follow-On

- item-043: implement archived-session delete behavior.
