# Skills Settings Invalid States Proof

Status: proof artifact
Updated: 2026-07-02

## Run

```sh
node --test proofs/skills-settings-invalid-states/proof.test.mjs
```

## Result

The harness proves Settings can list bundled, user-global, plugin-provided, and
project-local skill records from metadata only; customization creates an editable
user-global copy; invalid states remain visible with repair reasons; and only
enabled valid eligible skills enter `$` suggestions or runtime context.
