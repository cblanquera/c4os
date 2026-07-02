# Browser Document Preview Boundary Proof

Status: proof artifact
Updated: 2026-07-02

## Run

```sh
node --test proofs/browser-document-preview-boundary/proof.test.mjs
```

## Result

The harness proves Browser can host sandboxed rendered output from Documents and
Spreadsheets plugins while document-family plugins own parsing. Browser-native
PDF preview is allowed, unsupported formats show a boundary message, and preview
handoffs emit typed Browser events.
