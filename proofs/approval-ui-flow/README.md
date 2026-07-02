# Approval UI Flow Proof

Status: proof artifact
Updated: 2026-07-02

## Run

```sh
node --test proofs/approval-ui-flow/proof.test.mjs
```

## Result

The harness proves approval decisions produce typed events for allow, deny, and
remembered allow decisions; remembered-rule summaries include tool, risk/action,
target scope, plugin id, and duration; thread context and Chat Debug receive
the summary; Settings > Configuration owns review, edit, and revoke controls
per registered server tool.
