# Plugin Settings Renderer Proof

Status: proof artifact
Updated: 2026-07-02

## Run

```sh
node --test proofs/plugin-settings-renderer/proof.test.mjs
```

## Result

The harness proves schema-driven settings rendering and persistence, simple
`visibleWhen`, ignored unknown keys with warnings, shell-reserved key
validation, and sensitive value storage through a secret reference plus
redacted display value.
