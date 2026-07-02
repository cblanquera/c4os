# Approval Remember Policy Proof

Status: proof artifact
Updated: 2026-07-02

## Run

```sh
node --test proofs/approval-remember-policy/proof.test.mjs
```

## Result

The harness proves remembered approval rules are keyed by tool id, risk/action
category, normalized target scope, and plugin id; session-only decisions expire
with the session; user-global decisions persist; and Settings can surface one
policy item per registered tool with review, edit, and revoke controls.
