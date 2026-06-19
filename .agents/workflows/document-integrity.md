# Document Integrity Workflow

Use this when context, specs, wireframes, progress, releases, or derived views may have drifted.

## Check

- Links and source references are valid enough for local routing.
- Context contains only shared reusable product truth.
- Spec records contain detailed requirements, risks, evidence, tasks, and acceptance criteria.
- Progress files exist only for active execution.
- Final accepted records have been considered for context promotion.
- Verification claims name actual checks.
- Generated `.agents/**/*.md` files stay under 500 lines.

## Repair Rules

- Repair routing, indexes, statuses, and stale references compactly.
- Do not change product scope while doing integrity repair.
- Do not retire original planning sources without a source-retirement pass.

## Stop

Stop when the changed documents are coherent and any unresolved conflict is called out explicitly.
