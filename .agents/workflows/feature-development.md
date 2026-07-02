# Feature Development Workflow

Use this after MVP scope is accepted for bounded feature streams, hardening, polish, expansion, release readiness, or maintenance.

## Read First

- `.agents/context/product-brief.md`
- `.agents/context/product-specs.md`
- `.agents/context/work-orders.md`
- The relevant spec folder or MVP records
- `.agents/development/mvp/manifest.md`, if MVP progress is involved
- `.agents/development/<spec-id>/manifest.md`, if active progress exists for the target spec

## Process

1. Reconcile the requested feature goal into a major context document or a bounded spec.
2. Create a sibling spec only when the work has its own acceptance criteria, risks, decisions, POC, or multiple implementation items.
3. Build each spec from `.agents/context/`, `.agents/references/`, and the
   feature's own files. Do not rely on another feature spec for project-wide
   truth.
4. Put decisions in the affected spec's `decisions.md`. If a decision affects
   multiple feature specs, copy the relevant decision into each affected spec
   and cite the same source.
5. Promote shared reusable product, technical, creative, or sequencing truth
   into the relevant `.agents/context/` file.
6. Keep long research, provenance, and rationale in `.agents/references/`.
7. Convert accepted tasks into progress items only when implementation starts.
   Use `.agents/development/<spec-id>/` for future frozen specs, not the MVP
   execution folder.
8. Verify changed surfaces before marking progress done or verified.

## Reference Routing

Feature specs must not contain bare source lists. Every linked context, reference, evidence file, grill answer, proof result, or imported source must include:

```md
- `.agents/references/<area>/<file>.md`
  Purpose: What this file contributes to the feature decision or evidence chain.
  Load when: The task needs that source, rationale, decision, proof, or accepted-answer detail.
  Skip when: The current spec record and `.agents/context/` already answer the task.
```

`Purpose:` and `Load when:` are required. `Skip when:` is optional but preferred.

## Stop

Stop when the feature is routed to a spec, progress item, review, validation target, or explicit non-goal.
