# Creatives Workflow

Use this for visual direction, brand exploration, creative briefs, copy
explorations, asset notes, and creative review rounds. Creative direction is
optional for C4OS MVP work unless a spec or user decision requires it, but it
must be accepted or explicitly deferred before production frontend
implementation depends on visual styling rules.

## Read First

- `.agents/AGENTS.md`
- `.agents/context/index.md`
- relevant spec brief, audience, non-goals, constraints, and acceptance
- existing root `creatives/` artifacts, if any

## May Update

- root `creatives/<direction-or-screen-set>/`
- root `creatives/<direction-or-screen-set>/index.md`
- root `creatives/<direction-or-screen-set>/guidelines.md` after approval
- root `creatives/<direction-or-screen-set>/reviews.md`
- `.agents/context/` after accepted guideline creation
- `.agents/references/` for provenance, rationale, or large examples
- spec records when creative direction changes scope, acceptance, or product truth

## Process

1. Identify audience, mood, brand constraints, output surfaces, and review target.
2. Keep creative notes and assets separate from implementation source.
3. Run review rounds explicitly; approval applies only to the named round unless full creative direction is approved.
4. Record accepted direction, rejected alternatives, unresolved feedback, and open questions.
5. Link large binaries, generated images, Figma files, or production assets through provenance notes instead of storing them in `.agents/`.
6. When direction is approved, write `creatives/<direction-or-screen-set>/guidelines.md` with color, typography, spacing, components, pages, imagery, motion, copy, accessibility, implementation notes, QA checks, deferrals, and context promotion.
7. Promote accepted creative rules into `.agents/context/` and relevant spec records before implementation depends on them.

## Stop

Stop when review state, approval scope, accepted direction, unresolved
feedback, durable records updated, and the recommended next step are clear.
