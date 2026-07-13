# Browser Plugin Proof Planning

Status: proposed

Proof implementation artifacts, if approved during proof execution, must live under the listed `proofs/<proof-name>/` paths. Do not create proof code during planning replay.

| Proof Path | Proof Question |
| --- | --- |
| proofs/browser-annotation-attachment-model/ | Prove many annotations attach with screenshot, metadata, marker, comment, URL, frame, selector/path, viewport and clear after send. |
| proofs/browser-document-preview-boundary/ | Prove Browser hosts rendered document output without owning document-family parsing. |
| proofs/browser-state-hydration-without-visible-view/ | Prove Browser navigation/action results created without a visible Browser plugin view are stored as app-owned per-chat tool result state and hydrate when compatible Browser views are later opened, without one view claiming or forking the shared source state. |

## Execution Results

Verification command:

```sh
node --test proofs/browser-annotation-attachment-model/proof.test.mjs proofs/browser-document-preview-boundary/proof.test.mjs proofs/browser-state-hydration-without-visible-view/proof.test.mjs
```

| Proof Path | Result | Decision |
| --- | --- | --- |
| proofs/browser-annotation-attachment-model/ | Passed. Multiple Codex-style Browser annotations attach to one prompt bundle with screenshot scope, marker, comment, URL, frame, selector/path, viewport, provider compatibility, redaction status, and clear-after-send events. | Promote many-annotation prompt evidence bundles as feasible, reusing the prior attachment-adapter record model. |
| proofs/browser-document-preview-boundary/ | Passed. Documents and Spreadsheets own parsing and pass rendered output to Browser for sandboxed hosting; Browser-native PDF preview remains allowed; unsupported formats show a document-family-plugin boundary. | Promote Browser-as-host and document-family-plugin parsing ownership as feasible; Q036 remains superseded by Q036A. |
| proofs/browser-state-hydration-without-visible-view/ | Passed. A Browser-oriented runtime tool executes once without a visible Browser view, stores app-owned per-chat state, and later hydrates multiple compatible Browser views without one view claiming or forking source state. | Promote view-independent Browser tool execution and shared app-owned hydration state as feasible. |
