# R010 Onboarding And Start Review Notes

## Review Round 1 — 2026-07-17 — Full copy-forward SPA integration

### Changed

- Rebuilt r010 by copying every file from the updated `wireframes/r009-settings/` revision.
- Retained the r009 workspace, Settings SPA, HTML, CSS, JavaScript, supporting files, and workflow destinations inside r010.
- Added first-provider onboarding and provider-present workspace start as top-level views inside the existing `index.html` SPA.
- Added all onboarding/start presentation rules to the copied `styles.css` and all new behavior and routing to the copied `script.js`.
- Added `#onboarding` and `#start` review routes without creating separate onboarding HTML, CSS, or JavaScript files.
- Removed the earlier standalone `provider-setup.html` and `start.html` interpretation.
- Updated `workflows.html` with direct launch-state entry points.

### Feedback Applied

- Applied the correction that a new revision must copy forward the full previous revision folder rather than link back to it.
- Applied the correction that onboarding and start screens belong in the existing SPA files: `index.html`, `styles.css`, and `script.js`.
- Preserved the established main application and Settings behavior as the base of this revision.

### Review Focus

- Confirm `#onboarding` appears as a standalone provider form with no Settings navigation.
- Confirm successful provider setup transitions to `#start` without leaving `index.html`.
- Confirm `#start` contains the three required actions and exactly three recent workspaces.
- Confirm a workspace choice transitions into the retained r009 main app without document replacement or visual drift.

### Simulated Or Deferred Behavior

- Provider testing, credentials, file/folder selection, workspace parsing, and repository cloning remain simulated.
- One localStorage flag simulates whether a provider exists; direct hashes keep both launch branches independently reviewable.

### Open Questions

- None blocking this corrected architecture.

### Approval Path

- Approval of Review Round 1 locks the copy-forward SPA architecture and the onboarding/start flow structure. Requested visual or interaction changes remain in this revision as another review round.

## Review Round 2 — 2026-07-17 — Launch-screen annotation fixes

### Changed

- Restored vertical document scrolling while either `#onboarding` or `#start` is active, without changing the fixed workspace or Settings overflow contracts.
- Removed the Settings action from the start-screen header.
- Removed text decoration from every link rendered inside the launch SPA, including action-card and recent-workspace text.

### Feedback Applied

- Applied Browser Comment 1: the taller OpenAI Compatible onboarding form can now scroll to its remaining fields and actions.
- Applied Browser Comment 2: the start-screen Settings action is removed.
- Applied Browser Comment 3: start-screen link text no longer shows underlines.
- Applied Browser Comment 4: the start screen now scrolls when its content exceeds the available viewport.

### Review Focus

- Confirm both launch routes scroll vertically at the annotated 1217 × 701 viewport.
- Confirm the start header contains only the C4OS identity.
- Confirm action cards, workspace names, paths, and dates have no text decoration.

### Simulated Or Deferred Behavior

- Provider and workspace actions remain simulated as described in Round 1.

### Open Questions

- None.

### Approval Path

- Approval of Review Round 2 locks these launch-screen refinements. Additional feedback remains in r010 as another review round.

## Review Round 3 — 2026-07-17 — New terminal response per command

### Changed

- Changed the Terminal-mode composer so every submitted command creates a new user command message and a new Terminal response artifact.
- Removed the global composer behavior that appended typed input to the currently active Terminal artifact.
- Kept process-specific stdin inside the expanded Terminal session’s inline prompt.
- Kept the global Terminal composer’s prefix, placeholder, and action consistently set to `$`, `Enter a command`, and `Run command`.

### Feedback Applied

- Applied the browser annotation that entering `ls` must create a new Terminal response rather than appearing inside the last Terminal response.

### Review Focus

- Enter `ls` in the global Terminal composer and confirm a new `$ ls` user message and a separate Terminal artifact appear at the end of the thread.
- Confirm the previous running, completed, or interrupted Terminal artifact remains unchanged.
- Confirm expanded-process input remains separate from the global command composer.

### Simulated Or Deferred Behavior

- Command execution and output remain simulated; the artifact and shared-session behavior represent the intended frontend contract.

### Open Questions

- None.

### Approval Path

- Approval of Review Round 3 locks the one-command-per-response-artifact behavior. Additional feedback remains in r010 as another review round.

## Review Round 4 — 2026-07-17 — File edit-state continuity

### Changed

- Added one shared draft-content value to each File artifact alongside its existing editing flag.
- Made the focused File renderer reflect the shared editing state instead of always opening in view mode.
- Synchronized draft text from both inline and focused editors into the shared File artifact state.
- Preserved editing mode and unsaved draft text when expanding a File response and when closing the focused File workspace.
- Kept Cancel and Save as the only actions that intentionally leave editing mode.

### Feedback Applied

- Applied Browser Comment 1: expanding a File response while editing now opens the focused File workspace in edit mode.
- Applied Browser Comment 2: closing a focused File workspace while editing now restores the inline File response in edit mode.

### Review Focus

- Begin editing the inline `notes.md` response, change its text, and expand it; confirm the focused surface remains editable with the same draft.
- Change the draft in focused mode, press Close, and confirm the inline response remains editable with the latest draft.
- Confirm Cancel discards the draft and Save commits it in either context.

### Simulated Or Deferred Behavior

- File writes remain simulated and in-memory; this round changes only edit-state and draft continuity between render contexts.

### Open Questions

- None.

### Approval Path

- Approval of Review Round 4 locks File edit-state continuity across inline and focused contexts. Additional feedback remains in r010 as another review round.

## Review Round 5 — 2026-07-17 — File Explorer state restoration

### Changed

- Made focused File Explorer state flow back into the inline File Explorer response when X closes focus.
- Preserved the current folder and opened child file instead of restoring the response’s earlier state.
- Preserved whether the opened Explorer file is in view or edit mode.
- Added shared draft synchronization so unsaved Explorer-file text survives the focused-to-inline transition.
- Kept X as a close-focus action that returns to Chat rather than a nested back button.

### Feedback Applied

- Applied the clarified annotation that X should return from a focused Explorer child file to the inline File Explorer response while retaining the child file’s exact view/edit state.

### Review Focus

- Expand a File Explorer response, navigate to a folder, open a file, and press X; confirm the inline response shows that same opened file.
- Repeat in edit mode after changing text; confirm the inline response remains editing with the unsaved draft intact.
- Confirm the main workspace returns to Chat and does not remain in the focused Explorer.

### Simulated Or Deferred Behavior

- Explorer edits remain in-memory; Save and Cancel retain their existing simulated behavior.

### Open Questions

- None.

### Approval Path

- Approval of Review Round 5 locks focused-to-inline File Explorer state restoration. Additional feedback remains in r010 as another review round.
