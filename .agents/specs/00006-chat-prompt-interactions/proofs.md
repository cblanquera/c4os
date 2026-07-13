# Chat Prompt Interactions Proof Planning

Status: proposed

Proof implementation artifacts, if approved during proof execution, must live under the listed `proofs/<proof-name>/` paths. Do not create proof code during planning replay.

| Proof Path | Proof Question |
| --- | --- |
| proofs/approval-ui-flow/ | Prove approval popover/dialog decisions end to end, including remember duration choices, applied remembered-rule summaries in thread/Chat Debug, and review/edit/revoke routing to Settings > Configuration per registered server tool. |
| proofs/prompt-tag-resolution/ | Prove $, @, / routing and disabled resource behavior. |
| proofs/attachment-compatibility/ | Prove files, screenshots, and annotations through OpenAI-compatible adapter fallback. |

## Execution Results

Verification command:

```sh
node --test proofs/approval-ui-flow/proof.test.mjs proofs/prompt-tag-resolution/proof.test.mjs proofs/attachment-compatibility/proof.test.mjs
```

| Proof Path | Result | Decision |
| --- | --- | --- |
| proofs/approval-ui-flow/ | Passed. Approval decisions emit typed allow, deny, and remembered allow events; remembered summaries include tool, action/risk, target scope, plugin id, and duration; thread context and Chat Debug receive summaries; Settings > Configuration owns review/edit/revoke controls per registered server tool. | Promote typed approval decision events and Settings-owned remembered-rule review as feasible. |
| proofs/prompt-tag-resolution/ | Passed. Prompt display tokenization is separate from backend resolution events; `$` resolves to Skills, `@` resolves enabled resources/files, `/` resolves runtime/tool-gateway commands, and disabled/dependency-blocked resources cannot execute from frontend-only state. | Promote backend-authoritative prompt resolver and disabled-resource hiding/repair routing as feasible. |
| proofs/attachment-compatibility/ | Passed. Prompt-level files, Browser screenshots, and multiple Browser annotations become C4OS attachment records with metadata, limits, compatibility, fallback, and redaction fields; supported records adapt to OpenAI-compatible parts; unsupported records warn and degrade visibly; active Browser annotations clear after send. | Promote prompt attachment record and OpenAI-compatible adapter handoff as feasible, reusing the Batch 1 provider-adapter direction instead of duplicating provider internals. |
