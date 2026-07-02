# Runtime Tool Discovery Without Plugin View Proof

Status: proof artifact
Updated: 2026-07-02

## Run

```sh
node --test proofs/runtime-tool-discovery-without-plugin-view/proof.test.mjs
```

## Result

The harness proves runtime discovery and invocation can operate from the
registered tool catalog without any plugin view, including a view-oriented
tool that creates C4OS-owned per-chat inspectable state later hydrated by
multiple compatible views.
