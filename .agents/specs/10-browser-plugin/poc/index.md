# Browser Plugin Proof Planning

Status: proposed

Proof implementation artifacts, if approved during proof execution, must live under the listed `proofs/<proof-name>/` paths. Do not create proof code during planning replay.

| Proof Path | Proof Question |
| --- | --- |
| proofs/browser-annotation-attachment-model/ | Prove many annotations attach with screenshot, metadata, marker, comment, URL, frame, selector/path, viewport and clear after send. |
| proofs/browser-document-preview-boundary/ | Prove Browser hosts rendered document output without owning document-family parsing. |
| proofs/browser-state-hydration-without-visible-view/ | Prove Browser navigation/action results created without a visible Browser plugin view are stored as app-owned per-chat tool result state and hydrate when compatible Browser views are later opened, without one view claiming or forking the shared source state. |
