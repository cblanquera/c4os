# Freeze Workflow

Use this when accepted MVP or bounded feature records should become an implementation contract.

## Entry Gate

- Target scope is clear.
- The freeze target is `.agents/specs/mvp/` or a bounded feature spec.
- Blockers are resolved or explicitly accepted.
- Requirements link to acceptance criteria.
- Important decisions and risks are recorded.
- POC, wireframe, creative, review, QA, feedback, and validation outcomes are reconciled or explicitly deferred.
- Reusable product understanding has been promoted or reconciled into `.agents/context/`.
- The target spec states implementation paths and verification expectations.

For the distributable desktop MVP, implementation paths must be `backend/`,
`frontend/`, and `tests/server/`. Do not create or use `src-tauri/`.

## Process

1. Confirm the freeze target and source records.
2. Reject hidden scope and stale assumptions.
3. Confirm research findings and POC results have promotion, replacement, discard, or follow-up decisions.
4. Confirm reusable final findings have been promoted or reconciled into `.agents/context/`.
5. Confirm `.agents/specs/mvp/status.md` can be marked `frozen-for-implementation`.
6. Convert accepted tasks into implementation-ready proposed work.
7. Create progress items only if active execution is requested.

## Stop

Stop when the frozen scope can be implemented without rereading raw planning docs for baseline requirements.
