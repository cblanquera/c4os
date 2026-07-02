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
