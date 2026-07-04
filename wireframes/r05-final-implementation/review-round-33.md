# Review Round 33 - Project Menu Rendering Repair

Date: 2026-07-03

## Feedback Applied

The project action menu rendered as a full nested box under every project and
disrupted the visible chat/session rows. It did not behave like a compact
context menu.

## Correction Pass

- Moved each project menu back inside the project row so it does not create a
  separate project-list row.
- Added explicit `.project-actions-menu[hidden] { display: none; }` because
  the shared popover class sets `display: grid`.
- Positioned the visible menu absolutely under the row action area.
- Removed outlined button treatment from project menu items.
- Kept compact light r05 styling and current font sizing.
- Preserved chat/session rows under active projects.
- Preserved menu order:
  - Missing project: Relocate, Reveal, Copy path, Rename, Remove.
  - Found project: Reveal, Copy path, Rename, Remove.

## Scope Boundary

No specs, durable handoff docs, or product implementation files were updated.
