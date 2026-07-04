# Review Round 28 - Batch 4 Self-QA Correction

Date: 2026-07-03

## Trigger

User feedback identified that the Batch 4 review was still relying on the user
to find route-by-route problems after the root issue had already been named.
The artifact needed a batch-level consistency pass against the r04 workspace
shape and the approved r05 shell conventions.

## Correction Pass

- Left `#workspace-start` in its reviewed FS-left-panel shape.
- Reworked `#workspace-loaded` as the baseline loaded workspace state using the
  r04-style project list density, with sessions shown only under the active
  `c4os2` project.
- Tightened `#workspace-missing-project` so the missing project is its own
  muted, struck-through active project with read-only sessions beneath it and
  the relocate/reveal/copy/remove action surface still available.
- Tightened `#workspace-non-git` so `client-ops` is the active sidebar project
  without borrowing the `c4os2` chat list.
- Removed rendered explanatory labels from file explorer rows, including
  `Markdown icon`, `hidden file visible`, `hidden by rule`, and `text file`.
- Removed the rendered icon-theme explainer card from the file panel. Icon and
  hidden-file behavior remain represented by the actual explorer state instead
  of annotations.
- Hid `.git` from the active file explorer state while leaving `.env.example`
  visible as the hidden-file example.
- Changed file route shell titles from invented `Draft handoff` labels to the
  selected file or file state being shown.
- Removed unused workspace-manager/table/savebar functions that belonged to the
  discarded center-manager direction rather than the FS-left-panel model.
- Removed unused CSS for the discarded workspace-manager/table/savebar/card
  model so the active artifact no longer carries that obsolete surface.

## Scope Boundary

No specs, source docs, handoff docs, or product implementation files were
updated. This remains a grayscale HTML/CSS/JS review artifact correction.
