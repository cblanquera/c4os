# User Directed File Access Policy Proof

Status: proof artifact
Updated: 2026-07-02

## Run

```sh
node --test proofs/user-directed-file-access-policy/proof.test.mjs
```

## Result

The harness proves the policy distinction between user-directed outside-project
reads, agent-initiated outside-project reads, trusted project writes,
destructive trusted-project deletes, and outside-project writes.
