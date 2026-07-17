# r004 Review Notes

## Round 1 — Artifact Focus Swap

- Date: 2026-07-15
- Source: Approved user decisions following the r003 mode-popover revision.
- Implemented:
  - Removed the right panel, right toggle, right resize handle, artifact tabs, and pane-close behavior.
  - Browser and File Expand now replace the center thread with a full-workspace artifact view.
  - The actual conversation thread moves into a draggable, resizable Chat frame while the composer remains fixed at the bottom.
  - Chat frame geometry persists during the active session and across artifact-to-artifact swaps.
  - Restoring Chat returns the thread to the center and restores the focused artifact to its original bubble.
  - Terminal responses no longer expose Expand, including newly generated Terminal artifacts.
  - Expanded Files retain Edit, Cancel, and Save behavior and synchronize the inline artifact state.
- Review state: Ready for browser annotation.

## Validation

- Browser and File focus swaps verified in the rendered SPA.
- Chat frame drag, resize, 320px minimum width, geometry persistence, and restore verified.
- Fixed composer verified usable while an artifact is focused.
- Terminal Expand count verified as zero for the initial card; dynamic construction also omits the control.
- No right-panel DOM, horizontal document overflow, or console errors detected.
