# Prompt Tag Resolution Proof

Status: proof artifact
Updated: 2026-07-02

## Run

```sh
node --test proofs/prompt-tag-resolution/proof.test.mjs
```

## Result

The harness proves prompt display tokenization is separate from backend
resolution events: `$` routes to Skills, `@` routes to enabled files/resources,
and `/` routes to the runtime/tool gateway. Disabled or dependency-blocked
resources are not executable by frontend state and surface only with a repair
route.
