# Policy Classification and Resolution Evidence — 2026-07-18

## Command

```sh
node --test proofs/policy-classification-and-resolution/proof.test.mjs
```

## Observed signal

- 7 tests passed; 0 failed, skipped, cancelled, or todo.
- The proof read `wireframes/r012-cleanup/script.js` and matched all 71 identities in source order.
- Every legacy unknown fixture converted to ambiguous confidence and an `unknown` effect.
- Presets, multi-category precedence, ceilings, granted external scope, authenticated publishing, exception narrowing/expiry, and exact single-use authorization all passed.

## Result

**Passed** for the pure C4OS policy decision boundary.

## Remaining unknowns

- OS sandbox and managed-policy distribution.
- Cryptographic authorization representation.
- Adapter enforcement before runtime tool side effects.
- Cross-process audit persistence and revocation races.
