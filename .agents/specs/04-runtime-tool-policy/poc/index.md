# Runtime And Tool Policy Proof Planning

Status: proposed

Proof implementation artifacts, if approved later, must live under the listed `proofs/<proof-name>/` paths. Do not create proof code during planning replay.

| Proof Path | Proof Question |
| --- | --- |
| proofs/runtime-tool-discovery-without-plugin-view/ | Prove runtime can discover/invoke registered tools without plugin views subject to policy. |
| proofs/user-directed-file-access-policy/ | Prove user-directed reads, agent outside-project read prompts, trusted writes, and destructive/outside write prompts. |
| proofs/pi-runtime-app-layer-proof/ | Prove Pi prompt execution, streaming, tool-call interception, approval denial, and resume. |
| proofs/model-attachment-adapter/ | Prove attachment adapter translation/degradation for OpenAI-compatible provider path. |
