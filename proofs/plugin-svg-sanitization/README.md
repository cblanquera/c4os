# Plugin SVG Sanitization Proof

Status: proof artifact
Updated: 2026-07-02

## Run

```sh
node --test proofs/plugin-svg-sanitization/proof.test.mjs
```

## Result

The harness proves static bundled SVGs can pass while script elements, event
handler attributes, and external hrefs are rejected with a fallback icon.
