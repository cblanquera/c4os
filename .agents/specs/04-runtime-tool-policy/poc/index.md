# Runtime And Tool Policy Proof Planning

Status: proposed

Proof implementation artifacts, if approved during proof execution, must live under the listed `proofs/<proof-name>/` paths. Do not create proof code during planning replay.

| Proof Path | Proof Question |
| --- | --- |
| proofs/runtime-tool-discovery-without-plugin-view/ | Prove runtime can discover/invoke registered tools without plugin views subject to policy, including a view-oriented tool that executes fully, leaves app-owned per-chat inspectable state, and lets multiple compatible views hydrate the same source state without claiming or mutating it. |
| proofs/user-directed-file-access-policy/ | Prove user-directed reads, agent outside-project read prompts, trusted writes, and destructive/outside write prompts. |
| proofs/approval-remember-policy/ | Prove remembered approval rules are keyed by tool id, action/risk category, normalized target scope, and plugin id when relevant; session-only rules expire with the active session, user-global rules persist in config policy, and Settings > Configuration shows per-server-tool policy items with explanations and revoke/edit controls. |
| proofs/pi-runtime-app-layer-proof/ | Prove Pi prompt execution, streaming, tool-call interception, approval denial, and resume. |
| proofs/model-attachment-adapter/ | Prove attachment adapter translation/degradation for OpenAI-compatible provider path. |

## Execution Results

Verification command:

```sh
node --test proofs/runtime-tool-discovery-without-plugin-view/proof.test.mjs proofs/user-directed-file-access-policy/proof.test.mjs proofs/approval-remember-policy/proof.test.mjs proofs/pi-runtime-app-layer-proof/proof.test.mjs proofs/model-attachment-adapter/proof.test.mjs
```

| Proof Path | Result | Decision |
| --- | --- | --- |
| proofs/runtime-tool-discovery-without-plugin-view/ | Passed. Runtime discovers registered tools without plugin views, executes view-oriented `browser.open`, stores C4OS-owned state, and lets multiple views hydrate without claiming or mutating source state. | Promote view-independent runtime tool discovery and C4OS-owned inspectable state as feasible. |
| proofs/user-directed-file-access-policy/ | Passed. User-directed outside reads allow, agent-initiated outside reads ask, trusted writes allow, destructive trusted deletes ask, and outside writes ask unless explicitly requested. | Promote explicit user-directed file policy boundary as feasible. |
| proofs/approval-remember-policy/ | Passed. Remembered rules use narrow keys, session rules expire, user-global rules persist, and Settings surfaces per-tool review/edit/revoke controls. | Promote remembered-rule semantics and Settings policy surface as feasible. |
| proofs/pi-runtime-app-layer-proof/ | Passed. App layer streams, intercepts tool calls, denies approval without execution, and resumes with trace continuity. | Promote Pi app-layer interception/resume contract as feasible, pending integration against the actual runtime package. |
| proofs/model-attachment-adapter/ | Passed. Text/image attachments translate to OpenAI-compatible parts, unsupported attachments degrade visibly, and logs redact raw encoded attachment data. | Promote provider attachment adapter direction as feasible. |
