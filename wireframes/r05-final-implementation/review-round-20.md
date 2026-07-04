# r05 Final Implementation Wireframes - Review Round 20

## Status

Ready for Batch 4 workspace-and-files review.

## What Changed

- Added focused workspace/project states for specs 06 and 07 without recreating
  all r04 workspace/file routes.
- Added create/load/save workspace, folder/repository opening controls,
  missing project, center search, and non-Git workspace states.
- Added configurable left/right File Explorer and File Editor plugin panel
  states.
- Added file context menu, Add to chat, create/rename/delete-to-trash
  confirmation, save/revert dirty state, external-change conflict, icon theme,
  hidden-file, and non-code empty states.
- Updated `#coverage` with Batch 4 spec 06 and 07 route coverage.

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

- Workspace safety: do create/load/save workspace controls make it clear
  that workspace files contain folder references while chats stay in user-level
  C4OS state?
- Missing/relink clarity: does the muted strike-through project row, last-known
  path, relocate action, and read-only chat state communicate the right recovery
  behavior?
- File operation affordances: are Copy Path, Add to chat, reveal, rename,
  create, guarded delete-to-trash, save, revert, and external-change conflict
  visible at the right moments?
- Non-code usability: does the Files/Editor surface work for notes, docs,
  config, and operations folders without implying every workspace is a code
  repository?

## Approval Boundary

No `.agents` specs, `wireframes/ui-handoff-spec.md`, or durable handoff docs
were updated in this round. If Batch 4 is approved, the next step is to update
specs 06 and 07 with accepted wireframe evidence and promote approved behavior
into handoff docs where it should guide implementation.
