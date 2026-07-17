# File-System Workspace File and Relink Proof

This proof checks C4OS workspace-file and project-relink behavior. It is
self-contained and does not depend on a separate planning record.

## Question

Can C4OS load/save portable workspace files while keeping project chats in
user-level app state, marking missing projects read-only, and migrating chat
ownership only after an explicit relink?

## Verification

```sh
node --test proofs/fs-workspace-file-and-relink/proof.test.mjs
```

## Result

Passed on 2026-07-02 as part of the third POC batch.

The harness proves:

- Workspace files contain workspace name and project folder references only.
- Project records and chats live in user-level registry state.
- Missing project records retain last-known path, muted/read-only state, and
  visible chats.
- Explicit relocation replaces canonical project identity and migrates chats to
  the new canonical path.

## Decision

Promote the user-level registry plus explicit workspace descriptor boundary as
feasible. This proof follows prior POC policy that workspace files do not own
chat history.
