# Shell Panel Resize And Restore Proof

Status: proof artifact
Updated: 2026-07-02

## Run

```sh
node --test proofs/shell-panel-resize-and-restore/proof.test.mjs
```

## Result

The harness proves plugin icon panel toggles, same-side replacement,
per-session visibility restore, Settings route close/restore behavior, and
resize collision handling that preserves a 640px center pane minimum.
