# Chat Debug Redaction History Proof

Status: proof artifact
Updated: 2026-07-02

## Run

```sh
node --test proofs/chat-debug-redaction-history/proof.test.mjs
```

## Result

The harness proves Chat Debug can display bounded active-chat history as typed
events, surface approval decisions in both thread context and Chat Debug, redact
secrets before persistence/display, prune older runs by retention limit, and
exclude an export surface.
