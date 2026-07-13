# project-and-chat-removal-semantics

Proof for `.agents/specs/00007-file-system-plugin/`.

## Question

Can C4OS keep Remove Project and Remove Chat behavior separate so workspace
membership removal preserves history while chat removal deletes C4OS-owned
history?

## Verification

```sh
node --test proofs/project-and-chat-removal-semantics/proof.test.mjs
```

## Result

Passed on 2026-07-02 as part of the third POC batch.

The harness proves:

- Remove Chat deletes the selected chat from C4OS-owned project history.
- Remove Project removes only current workspace membership.
- Preserved project chats remain available through user-level project state.

## Decision

Promote separate project membership and chat-history deletion semantics as
feasible.
