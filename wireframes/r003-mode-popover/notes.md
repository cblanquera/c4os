# Wireframe Review Notes

## Round 1 - 2026-07-15 - Composer mode popover

### Changed

- Replaced the persistent Chat, Files, Browser, and Terminal row with one compact active-mode control in the lower composer toolbar.
- Positioned the mode control immediately to the left of the paperclip in Chat mode.
- Added an upward popover with icons, short descriptions, and a checked current mode.
- Kept the active mode visible on the trigger and preserved the existing hash, input, artifact, and submission behaviors.
- Temporarily hides the mode control while replying, then restores the previously selected mode after cancel or submit.
- Added Escape and arrow-key behavior to the mode popover.

### Review Focus

- Review whether the compact trigger makes mode switching sufficiently discoverable without occupying a permanent composer row.
- Review the popover width, option descriptions, and selected-state emphasis.
- Review whether hiding the mode trigger during a quoted reply keeps the reply context clear.

### Simulated Or Deferred

- Filesystem, browser, terminal, and AI operations remain illustrative as defined by `r002`.
- No artifact, right-pane, message, or panel behavior changed in this revision.

### Verification

- Verified the trigger appears before the paperclip and the Chat layout has no horizontal document overflow at 1280 by 720.
- Verified the popover opens upward with all four modes and a checked current option.
- Verified Files, Browser, and Terminal selections update the trigger, visible input, and URL hash, then close the popover.
- Verified Files mode hides Chat-only attachment and microphone controls while retaining the mode trigger.
- Verified replying hides the mode control and cancel restores the previously selected Files mode.
- Verified Escape closes the popover and returns focus to the trigger.
- Verified `script.js` syntax and repository diff whitespace checks.
- Saved review captures in `qa/round-1-chat-mode.png` and `qa/round-1-mode-popover.png`.

### Open Questions

- Should the trigger retain its text label at very narrow widths, or collapse to an icon-only treatment?
- Are the one-line option descriptions helpful, or should the popover be denser with labels only?

### Approval Path

If approved, use `r003-mode-popover` as the active composer-mode direction. If not, revise the trigger density, popover placement, or option treatment in this revision.

## Round 2 - 2026-07-15 - Artifact controls and inline file editing

### Changed

- Consolidated Browser artifact controls into one header: boxed browser icon, back, forward, refresh, read-only current address, and far-right Expand.
- Removed the separate Browser navigation row and title/metadata block.
- Added File artifact Edit, Cancel, and Save icon controls.
- Added a line-numbered inline File editor with scrolling and vertical resizing; Cancel and Save return to the compact read-mode card.
- Moved each Terminal command into the artifact header and removed the repeated command from its output body.
- Removed the visible Files `Browse` label and normalized Files, Browser, and Terminal composer prefixes to the same 30px boxed icon treatment.

### Review Focus

- Review whether the consolidated Browser toolbar reads clearly without a page-title label.
- Review whether File Edit, Cancel, Save, and Expand remain distinguishable as adjacent icon controls.
- Review the editor’s starting height, line-number gutter, and vertical resize affordance.
- Review whether command-in-header plus output-only body improves Terminal scanability.

### Feedback And Annotations Applied

- Applied comments 1 and 2 by combining the Browser icon, navigation, read-only address, and Expand control into one header and removing the old secondary row.
- Applied comment 3 by adding inline File read/edit modes with line numbers, scrolling, vertical resizing, Cancel, and Save.
- Applied comments 4 and 5 by moving `$ ls` to the Terminal header and removing it from the output body.
- Applied comment 6 by making Files Browse icon-only.
- Applied comments 7 and 8 by boxing the Browser and Terminal input prefixes to match Files.

### Simulated Or Deferred

- Browser navigation, file persistence, and terminal execution remain local illustrative behaviors.
- Inline File Save increments the illustrative version in memory; reloading resets the prototype.
- Expanded right-pane File editing remains a separate existing interaction.

### Verification

- Verified Browser header order, read-only address state, navigation update to `https://example.com/dashboard`, and enabled Back state.
- Verified File edit mode exposes Cancel/Save, five generated line numbers, `overflow: auto`, and `resize: vertical`.
- Measured the initial File surface at 226.19px, edit mode at 312px, and the restored Cancel state at 226.19px.
- Verified Save updates the preview and advances the illustrative version.
- Verified initial and generated Terminal artifacts show `$ <command>` in the header and output only in the body.
- Verified all three non-chat composer prefix boxes measure 30px by 30px and Files contains no visible Browse text.
- Verified newly generated Browser, File, and Terminal artifacts inherit the revised structures.
- Verified `script.js` syntax and repository diff whitespace checks.
- Saved review captures in `qa/round-2-browser-header.png`, `qa/round-2-file-editor.png`, and `qa/round-2-terminal-command.png`.

### Open Questions

- Should File Save remain an icon-only immediate action, or should it use the existing approval policy before persisting?
- Should the Browser artifact footer retain status and navigation count now that the header is more compact?

### Approval Path

If approved, treat the revised Browser, File, Terminal, and composer-input controls as the active `r003` artifact patterns. If not, revise their control density, editor sizing, or metadata treatment within this revision.

## Round 3 - 2026-07-15 - Artifact type identity placement

### Changed

- Replaced the generic stars icon beside `GPT-5` with the appropriate Browser, File, or Terminal icon for each Response Artifact.
- Removed the duplicate Browser icon from the Browser toolbar.
- Removed the duplicate File and Terminal icons from their artifact headers.
- Expanded the Terminal composer’s prefix grid column from 18px to 30px so the boxed `$` prefix no longer collides with the command input.
- Applied the same icon hierarchy to newly generated Browser, File, and Terminal artifacts.

### Review Focus

- Review whether the artifact-type icon beside the model provides enough provenance without repeating it in the card header.
- Review the newly simplified text-only File and Terminal headers.
- Review the Terminal composer spacing between the `$` box and command field.

### Feedback And Annotations Applied

- Applied comment 1 by reserving the full 30px width of the Terminal prefix box.
- Applied comments 2 and 3 by removing the File header icon and moving the File icon to the model row.
- Applied comments 4 and 5 by removing the Browser toolbar icon and moving the Browser icon to the model row.
- Applied comments 6 and 7 by removing the Terminal header icon and moving the Terminal icon to the model row.

### Simulated Or Deferred

- No artifact behavior changed in this round; only identity placement and composer spacing changed.
- Type icons remain grayscale wireframe SVGs.

### Verification

- Verified all three initial artifacts have their unique type SVG in the model row and no `.artifact__icon` inside the artifact header.
- Verified newly generated Browser, File, and Terminal artifacts inherit the same icon hierarchy.
- Measured an 8px Terminal prefix/input gap; the prefix right edge is 417px and the input left edge is 425px with no overlap.
- Verified `script.js` syntax and repository diff whitespace checks.
- Saved review captures in `qa/round-3-browser-icon.png`, `qa/round-3-file-icon.png`, and `qa/round-3-terminal-icon.png`.

### Open Questions

- Should ordinary chat responses retain the stars icon while artifacts use type icons, as shown, or should all assistant identity icons follow one universal treatment?

### Approval Path

If approved, use type-specific model-row icons and text-only artifact headers as the final `r003` identity pattern. If not, revise the icon sizing or identity placement within this revision.

## Round 4 - 2026-07-15 - Expanded Browser pane header repair

### Changed

- Repaired the expanded right-pane Browser header with a dedicated `.pane-browser-bar` layout.
- Replaced unstyled navigation glyphs with the same SVG Back, Forward, and Refresh icon language used by the inline Browser artifact.
- Added a flexible, styled address field that remains editable in the expanded Browser pane.
- Preserved the two user-authored style fixes already present in `styles.css`; changes were limited to the pane Browser selectors and viewer markup.

### Review Focus

- Review the expanded Browser pane’s horizontal header alignment and density.
- Confirm Back, Forward, Refresh, and the address field remain readable at the current right-pane width.

### Feedback And Annotations Applied

- Applied comment 1 by restoring the broken expanded Browser header as one horizontal control row.
- Preserved the user’s two manual style fixes instead of normalizing or replacing unrelated CSS.

### Simulated Or Deferred

- Expanded Browser navigation remains locally simulated and shares state with the inline artifact.
- No other artifact, composer, or panel styling changed in this round.

### Verification

- Verified the right pane opens with a selected Browser tab and a semantic Browser tabpanel.
- Measured the toolbar at 379px wide and 47px high with three 30px by 30px navigation buttons.
- Verified the toolbar computes to `display: grid` and the address field occupies the remaining width without wrapping.
- Verified entering `https://example.com/dashboard` updates the expanded address and connected Browser state.
- Verified `script.js` syntax and repository diff whitespace checks.
- Saved the review capture in `qa/round-4-browser-pane-header.png`.

### Open Questions

- None blocking; this round is a targeted repair.

### Approval Path

If approved, retain the dedicated expanded Browser header as the final `r003` right-pane treatment. If not, revise only its control spacing or address-field density within this revision.
