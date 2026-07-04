# Review Round 30 - r04 File Editor Match

Date: 2026-07-03

## Feedback Applied

User asked to compare the r05 `#file-editor` route against r04 again.

## Comparison Finding

r04 file editor shows a minimal right-panel editor:

- breadcrumbs row
- code pane
- line numbers
- code lines

The r05 route had added a save/revert toolbar to the base editor state, which
made the normal file editor diverge from r04. Save/revert still belongs in the
dirty and conflict routes, not in the default editor route.

## Correction Pass

- Removed the save/revert toolbar from base `#file-editor`.
- Kept the toolbar only for `#file-editor-dirty` and
  `#file-external-conflict`.
- Changed the File Editor panel grid to r04's two-row editor shape for the
  default route: 36px breadcrumbs plus code pane.
- Aligned code pane styling with r04: 13px monospace, tabular numbers, sticky
  line numbers, `max-content` code rows, and `white-space: pre`.

## Scope Boundary

No specs, durable handoff docs, or product implementation files were updated.
