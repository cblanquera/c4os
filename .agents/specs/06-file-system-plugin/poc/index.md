# File System Plugin Proof Planning

Status: proposed

Proof implementation artifacts, if approved during proof execution, must live under the listed `proofs/<proof-name>/` paths. Do not create proof code during planning replay.

| Proof Path | Proof Question |
| --- | --- |
| proofs/fs-workspace-file-and-relink/ | Prove workspace file load/save, user-level registry, missing muted state, read-only chats, and relocate migration. |
| proofs/project-chat-sharing-across-workspaces/ | Prove same canonical project folder shares chats across workspace files. |
| proofs/project-and-chat-removal-semantics/ | Prove Remove Chat deletes history and Remove Project preserves project chat history. |

## Execution Results

Verification command:

```sh
node --test proofs/fs-workspace-file-and-relink/proof.test.mjs proofs/project-chat-sharing-across-workspaces/proof.test.mjs proofs/project-and-chat-removal-semantics/proof.test.mjs
```

Reconciliation before execution: prior POC and policy decisions require
user-level app state to own project chats. Workspace files therefore prove only
portable workspace membership and project folder references, not chat
ownership.

| Proof Path | Result | Decision |
| --- | --- | --- |
| proofs/fs-workspace-file-and-relink/ | Passed. Workspace files load/save name plus project folder references, chats remain in user-level registry state, missing projects become muted/read-only with last-known path, and explicit relocation migrates chat ownership to the new canonical path. | Promote the user-level registry plus explicit workspace descriptor boundary as feasible. |
| proofs/project-chat-sharing-across-workspaces/ | Passed. Two workspace files containing the same canonical project folder hydrate the same latest chats from user-level project state. | Promote canonical-project chat sharing across workspace files as feasible. |
| proofs/project-and-chat-removal-semantics/ | Passed. Remove Chat deletes selected C4OS-owned history while Remove Project removes only current workspace membership and preserves project chats. | Promote separate project membership and chat-history deletion semantics as feasible. |
