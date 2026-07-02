# Model Attachment Adapter Proof

Status: proof artifact
Updated: 2026-07-02

## Run

```sh
node --test proofs/model-attachment-adapter/proof.test.mjs
```

## Result

The harness proves supported text and image attachments can be translated into
an OpenAI-compatible provider request shape, unsupported attachments produce an
explicit degradation record, and logs avoid raw encoded attachment data.
