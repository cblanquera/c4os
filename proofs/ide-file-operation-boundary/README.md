# ide-file-operation-boundary

Proof for `.agents/specs/07-file-editor-plugin/`.

## Question

Can the File Editor plugin own editor UX while routing file mutations through
C4OS backend file services and preserving conflict, trash, and prompt tag
boundaries?

## Verification

```sh
node --test proofs/ide-file-operation-boundary/proof.test.mjs
```

## Result

Passed on 2026-07-02 as part of the third POC batch.

The harness proves:

- Open/save/create/rename/delete-to-trash operations are backend service calls,
  not renderer filesystem authority.
- External changes cause conflict state and require an explicit overwrite
  choice before writing.
- Delete routes to guarded trash behavior.
- Add to chat inserts a prompt file tag reference instead of hidden attachment
  contents.
- Editor actions emit typed editor-sourced events.

## Decision

Promote the backend file-service boundary and prompt-reference insertion model
as feasible for the File Editor plugin.
