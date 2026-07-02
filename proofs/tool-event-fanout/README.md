# Tool Event Fanout Proof

Status: proof artifact
Updated: 2026-07-02

## Run

```sh
node --test proofs/tool-event-fanout/proof.test.mjs
```

## Result

The harness proves a single `browser.open` backend execution fans out to all
compatible enabled views. Visible views render immediately, hidden compatible
views update per-chat state only, and no hidden view opens a panel, steals
focus, prompts, or triggers a duplicate backend call.
