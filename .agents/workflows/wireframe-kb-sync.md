# Wireframe-To-KB Sync Workflow

Use this workflow after every new wireframe revision and after any material update to an existing wireframe revision.

## Sync Trigger

Run a sync when any of these changes:

- screen inventory, route, layout, navigation, component, interaction, state, validation, accessibility, responsive behavior, platform behavior, visual direction, copy that carries product meaning, simulated/deferred boundary, or approval status
- revision-root `specs.md` or `notes.md`
- nested `qa/notes.md` in a way that changes confidence, exposes a defect, or verifies a previously unverified requirement

Pure formatting, comments, class renames, generated bundle refreshes, screenshots with no new finding, or cache-busting changes may close as `no-kb-impact`, but record that conclusion in the revision notes.

## Authority Rules

1. `.agents/context/` remains the accepted source of truth.
2. Wireframes are review artifacts and evidence.
3. A pending or unapproved wireframe must not silently replace accepted KB behavior. Record it in the relevant Spec File until accepted.
4. Explicit accepted user feedback can update the KB immediately when its scope is clear.
5. Later accepted decisions supersede conflicting earlier decisions; preserve the reason in the decision ledger.

## Sync Pass

1. Identify the latest revision and whether the change is a review round or a new revision.
2. Read the latest revision's complete `specs.md`, root `notes.md`, and nested `qa/notes.md`.
3. When a revision copied prior work forward, read its declared source revision and any notes not already represented in the decision ledger.
4. Compare the artifact against:
   - `.agents/context/usability-and-interface.md`
   - `.agents/context/runtime-session-architecture.md`
   - references 00004 through 00009
   - any intersecting runtime/security Context Files
5. Classify each change as `accepted-new`, `accepted-superseding`, `pending`, `simulated-only`, `deferred`, `verification-only`, or `no-kb-impact`.
6. Update the narrowest owning Context/Reference File. Do not paste the whole revision spec into the KB.
7. Update `00009-wireframe-decision-ledger.md` with the round's durable effect and any superseded rule.
8. Update `00008-reconstruction-fixtures-and-acceptance.md` when the deterministic fixture or review matrix changes.
9. If a wireframe exposes a runtime/security conflict, stop before changing accepted context and ask the user to resolve it.
10. Update `.agents/context/index.md` only when routing or file ownership changes.

## Reconstruction Parity Check

After material sync, ask: “Could an agent with only `.agents/context/` and its linked Reference Files recreate an equivalent latest accepted artifact?” Verify that the KB contains:

- every screen and starting route
- layout regions, dimensions, breakpoints, and platform adaptation
- components and meaningful variants
- triggers, before/after state, visible result, and persistence rules
- forms, fields, options, validation, and submit results
- responsive, keyboard, screen-reader, reduced-motion, light/dark, and cross-platform behavior
- fixture data needed to demonstrate each state
- simulated/deferred boundaries
- full acceptance checks

If the answer is no, add the missing durable detail before closing the sync.

## Validation

Run:

```bash
python3 .agents/scripts/validate-agent-workspace.py
git diff --check
```

For a material visual or interaction update, also run the revision's browser QA at wide and narrow sizes, in light and dark mode, and on every platform whose presentation changed. Record console state and any unverified platform explicitly.

## Closeout

Report:

- revision and review round processed
- KB files changed
- accepted, superseded, deferred, and no-impact findings
- validation and browser/platform QA results
- remaining blocker or user decision

Do not claim sync completion from source inspection alone when the change was browser-visible and rendered review was possible.
