# Feature Development Workflow

Use this after MVP scope is accepted for bounded feature streams, hardening, polish, expansion, release readiness, or maintenance.

## Read First

- `.agents/context/product-brief.md`
- `.agents/context/product-specs.md`
- `.agents/context/work-orders.md`
- The relevant spec folder or MVP records
- `.agents/development/progress/manifest.md`, if active progress exists

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
8. Verify changed surfaces before marking progress done or verified.

## Stop

Stop when the feature is routed to a spec, progress item, review, validation target, or explicit non-goal.
