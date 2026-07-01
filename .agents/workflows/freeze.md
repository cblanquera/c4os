# Freeze Workflow

Use this when accepted MVP or bounded feature records should become an implementation contract.

## Entry Gate

- Target scope is clear.
- The freeze target is `.agents/specs/mvp/` or a bounded feature spec.
- Blockers are resolved or explicitly accepted.
- Requirements link to acceptance criteria.
- Important decisions and risks are recorded.
- Decisions are recorded in the affected spec's `decisions.md`; multi-spec
  decisions are copied into each affected spec and cite the same source.
- POC, wireframe, creative, review, QA, feedback, and validation outcomes are reconciled or explicitly deferred.
- Reusable product understanding has been promoted or reconciled into the relevant major `.agents/context/` document.
- Specs are understandable from `.agents/context/`, `.agents/references/`, and
  their own files, without sibling specs as the source of project-wide truth.
- Imported archive material is provenance only, not live project truth.
- The target spec states implementation paths and verification expectations.

For the distributable desktop MVP, implementation paths must be `backend/`,
`frontend/`, and `tests/server/`. Do not create or use `src-tauri/`.

## Process

1. Confirm the freeze target and source records.
2. Reject hidden scope and stale assumptions.
3. Confirm research findings and POC results have promotion, replacement, discard, or follow-up decisions.
4. Confirm reusable final findings have been promoted or reconciled into the relevant major `.agents/context/` document.
5. Confirm accepted grill or decision-question IDs are reconciled into affected
   spec decisions or marked superseded, rejected, or deferred with reasons.
6. Confirm `.agents/specs/mvp/status.md` or the bounded feature spec can be
   marked `frozen-for-implementation`.
7. Convert accepted tasks into implementation-ready proposed work.
8. Create progress items only if active execution is requested.

## Stop

Stop when the frozen scope can be implemented without rereading raw planning docs for baseline requirements.
