# r05 Final Implementation Wireframes - Review Round 21

## Status

Ready for corrected Batch 4 functional review.

## What Changed

- Reworked the Batch 4 workspace routes from explanatory cards into functional
  app-like states.
- `#workspace-start` now shows a workspace manager table with real project
  rows, reorder handles, rename/collapse/more controls, and save/load/add
  project actions.
- `#workspace-missing-project` now shows the missing project as an active
  muted strike-through row with a concrete action menu for relocate, reveal
  last known path, copy path, and remove from this workspace.
- `#workspace-non-git` now shows a normal non-Git workspace with the composer
  omitting branch controls.
- File/editor routes now keep the editor as the main surface and show context
  menus, create row, delete confirmation, dirty save/revert state,
  external-change conflict, and non-code empty picks in-place.

## Review Links

- `./index.html#workspace-start`
- `./index.html#workspace-missing-project`
- `./index.html#workspace-search`
- `./index.html#workspace-non-git`
- `./index.html#files-left-panel`
- `./index.html#files-right-panel`
- `./index.html#file-context-menu`
- `./index.html#file-operations`
- `./index.html#file-editor-dirty`
- `./index.html#file-external-conflict`
- `./index.html#file-empty-states`
- `./index.html#coverage`

## Review Questions

- Does the workspace manager now feel like the real surface you would operate,
  rather than a description of the behavior?
- Is missing-project recovery clear from the sidebar row plus action menu?
- Are the file context menu, create row, trash confirmation, save/revert, and
  conflict states visible at the moment they would happen?
- Does the non-Git route feel like a first-class workspace rather than a
  disabled code workspace?

## Approval Boundary

No `.agents` specs, `wireframes/ui-handoff-spec.md`, or durable handoff docs
were updated in this round. If this corrected Batch 4 round is approved, the
next step is to update specs 06 and 07 with accepted wireframe evidence and
promote approved behavior into handoff docs where it should guide
implementation.
