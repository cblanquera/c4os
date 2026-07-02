# Chat Prompt Interactions Proof Planning

Status: proposed

Proof implementation artifacts, if approved during proof execution, must live under the listed `proofs/<proof-name>/` paths. Do not create proof code during planning replay.

| Proof Path | Proof Question |
| --- | --- |
| proofs/approval-ui-flow/ | Prove approval popover/dialog decisions end to end, including remember duration choices, applied remembered-rule summaries in thread/Chat Debug, and review/edit/revoke routing to Settings > Configuration per registered server tool. |
| proofs/prompt-tag-resolution/ | Prove $, @, / routing and disabled resource behavior. |
| proofs/attachment-compatibility/ | Prove files, screenshots, and annotations through OpenAI-compatible adapter fallback. |
