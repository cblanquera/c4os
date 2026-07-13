# project-chat-sharing-across-workspaces

Proof for `.agents/specs/00007-file-system-plugin/`.

## Question

Can two workspace files containing the same canonical project folder hydrate
the same project chat history from user-level state?

## Verification

```sh
node --test proofs/project-chat-sharing-across-workspaces/proof.test.mjs
```

## Result

Passed on 2026-07-02 as part of the third POC batch.

The harness proves:

- Workspace membership is independent from chat ownership.
- Canonical project folder identity is the lookup key for project chats.
- A shared project appears with the same latest chats across different
  workspace files.

## Decision

Promote canonical-project chat sharing across workspace files as feasible.
