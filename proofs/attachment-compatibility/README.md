# Attachment Compatibility Proof

Status: proof artifact
Updated: 2026-07-02

## Run

```sh
node --test proofs/attachment-compatibility/proof.test.mjs
```

## Result

The harness proves prompt-level files, Browser screenshots, and multiple Browser
annotations become C4OS attachment records with source, target metadata,
limits, compatibility, fallback, and redaction fields. It adapts supported
attachments to an OpenAI-compatible provider shape, warns on unsupported
attachments, records degradation, redacts logs, and clears active Browser
annotations after send.
