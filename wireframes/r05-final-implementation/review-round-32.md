# Review Round 32 - Project Row Actions Menu

Date: 2026-07-03

## Feedback Applied

Project row actions needed to match the screenshot interaction pattern without
adopting the screenshot theme or type scale.

## Correction Pass

- Changed project row actions from pencil/trash to `...` plus pencil.
- Preserved pencil as the new-chat action, not rename.
- Added per-project `...` menu toggles.
- Kept the missing-project menu open by default on
  `#workspace-missing-project` for review.
- Simplified menu labels and order:
  - Missing project: Relocate, Copy path, Rename, Remove.
  - Found project: Reveal, Copy path, Rename, Remove.
- Kept existing r05 light grayscale styling and current font sizing.

## Scope Boundary

No specs, durable handoff docs, or product implementation files were updated.
