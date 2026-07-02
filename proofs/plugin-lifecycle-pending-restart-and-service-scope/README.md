# Plugin Lifecycle Pending Restart And Service Scope Proof

Status: proof artifact
Updated: 2026-07-02

## Run

```sh
node --test proofs/plugin-lifecycle-pending-restart-and-service-scope/proof.test.mjs
```

## Result

The harness proves live UI/settings changes remain separate from
restart-gated backend tool registration, unavailable tools are hidden,
required dependencies block enablement, optional dependencies degrade only the
dependent contribution, and heavy services are lazy/shared with defined
shutdown reasons.
