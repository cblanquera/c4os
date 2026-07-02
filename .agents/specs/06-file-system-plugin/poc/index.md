# File System Plugin Proof Planning

Status: proposed

Proof implementation artifacts, if approved during proof execution, must live under the listed `proofs/<proof-name>/` paths. Do not create proof code during planning replay.

| Proof Path | Proof Question |
| --- | --- |
| proofs/fs-workspace-file-and-relink/ | Prove workspace file load/save, user-level registry, missing muted state, read-only chats, and relocate migration. |
| proofs/project-chat-sharing-across-workspaces/ | Prove same canonical project folder shares chats across workspace files. |
| proofs/project-and-chat-removal-semantics/ | Prove Remove Chat deletes history and Remove Project preserves project chat history. |
