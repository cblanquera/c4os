# Browser Annotation Attachment Model Proof

Status: proof artifact
Updated: 2026-07-02

## Run

```sh
node --test proofs/browser-annotation-attachment-model/proof.test.mjs
```

## Result

The harness proves multiple Codex-style Browser annotations can attach to one
prompt as a Browser evidence bundle. Each annotation carries screenshot scope,
marker, comment, URL, frame, selector/path, viewport, provider compatibility,
redaction status, and prompt events; active annotations clear after send while
sent prompt attachments persist.
