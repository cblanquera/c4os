# Policy Classification and Resolution Proof

Status: passed
Updated: 2026-07-18

## Question

Can C4OS replace the r012 wireframe's 71 permanent policy rows with composable action facts, four bounded presets, seven user-facing policy groups, and narrow concrete exceptions without weakening resolution?

## Run

```sh
node --test proofs/policy-classification-and-resolution/proof.test.mjs
```

## Acceptance signal

- all 71 unique legacy identities convert into facts or explicit ambiguity;
- unknown actions cannot silently allow;
- managed and safety ceilings remain effective under every preset;
- `Deny` overrides `Ask`, which overrides `Allow`, across multi-fact actions;
- explicit outside-workspace grants are distinguishable from ambient authority;
- remembered rules do not broaden operation, target, runtime/plugin, or duration; and
- an allow produces only an exact, generation-bound, single-use authorization.

## Non-goals

This proof does not enforce an OS sandbox, finalize policy UI copy, configure runtime-native permissions, distribute organizational policy, or provide cryptographically signed authorization tokens.

## Result

Passed on 2026-07-18. All seven deterministic tests passed. The test reads the unchanged r012 policy block and requires exact ordered equality with the 71-fixture corpus, so a missing, reordered, or newly added wireframe identity fails rather than silently drifting.

This establishes the application-layer resolution contract. It does not prove enforcement by an OS sandbox or either runtime; adapter conformance must prove that an allowed action receives an exact single-use authorization before side effects.

See `policy-classification-and-resolution-evidence-2026-07-18.md`.
