# r05 Final Implementation Wireframes - Review Round 22

## Status

Ready for corrected workspace-start ownership review.

## What Changed

- Moved `#workspace-start` workspace content fully into the File System left
  panel.
- Replaced the center workspace manager page with the original app-shell
  new-chat prompt state.
- Form-fit the r04 start-screen actions into the FS panel: Open Folder, Clone
  Repository, Open Workspace File, and Recent folder-backed workspaces.
- Kept missing-project actions inside the FS panel for the related workspace
  route.
- Preserved the r05 global header and plugin-panel shell conventions.

## Review Links

- `./index.html#workspace-start`
- `./index.html#workspace-missing-project`
- `./index.html#workspace-non-git`

## Review Questions

- Does `#workspace-start` now feel correctly owned by the FS plugin left panel?
- Does the center pane look like the original new-chat app shell state?
- Are the r04 start actions still recognizable after being fit into the left
  panel?

## Approval Boundary

No `.agents` specs, `wireframes/ui-handoff-spec.md`, or durable handoff docs
were updated in this round.
