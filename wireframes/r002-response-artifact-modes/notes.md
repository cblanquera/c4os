# Wireframe Review Notes

## Round 1 - 2026-07-15 - Response artifact modes

### Changed

- Added persistent Chat, Files, Browser, and Terminal modes above the composer input.
- Added one-file browsing, address entry, and terminal-command input treatments while preserving the existing Chat composer.
- Added Browser, File, and Terminal Response Artifacts to the conversation.
- Added contextual Copy and Reply actions to messages and artifacts, shown on hover or keyboard focus.
- Replaced the mode selector with a removable quoted-reference strip while replying, then restored the persisted mode after submission or cancellation.
- Added a tabbed right-pane viewer opened explicitly from artifact Expand controls.
- Added inline Browser navigation, File edit approval, expanded File editing, and one persistent Terminal session.

### Review Focus

- Review whether the four modes are discoverable without making the composer feel like a toolbar-heavy application shell.
- Review whether Browser and File artifacts communicate that follow-up replies update the original card.
- Review whether immutable Terminal command cards plus one continuous expanded session feel coherent.
- Review the balance between compact inline artifacts and richer right-pane tabs.

### Feedback And Decisions Applied

- Modes persist until manually changed; replies temporarily use natural-language Chat behavior.
- Files supports one file at a time and receives instructions through Reply.
- Browser previews remain compact inline and interactive in the expanded pane.
- Browser and File artifacts update in place; Terminal replies append new output cards in the same session.
- Artifacts do not open the right pane automatically.
- Right-pane tabs persist independently of their inline artifacts; closing a tab does not destroy the artifact or session.
- File approval saves immediately and increments the version.
- Copy uses contextual defaults for messages, URLs, file contents, and terminal command/output.

### Simulated Or Deferred

- File browsing, file reads and writes, webpage loading, browser navigation, shell execution, and AI responses use local illustrative state.
- Approval behavior is demonstrated for file edits; production command-risk classification is deferred.
- Browser tabs within a Browser artifact, multiple-file selection, credential-specific states, and durable persistence are deferred.
- The left panel remains blank.

### Verification

- Verified all four mode states and confirmed Browser mode persists after artifact creation.
- Verified direct Chat submission appends a user/assistant pair, direct Files opens one new File artifact, and direct Terminal appends command output.
- Verified Browser replies update the original artifact without adding a second Browser card.
- Verified File replies expose approval and approval saves immediately as Version 2.
- Verified Terminal replies preserve the original card and append a second output in the same session.
- Verified three artifact types can remain open as right-pane tabs and closing a tab preserves the transcript artifact.
- Verified an expanded File artifact begins read only, can enter Edit mode, and saves its revised contents back to the inline artifact.
- Verified Browser Copy places the current URL on the clipboard.
- Verified the 720 by 900 layout has no horizontal document overflow and collapses both panels into overlays.
- Verified no browser console warnings or errors.
- Saved review captures in `qa/round-1-desktop.png`, `qa/round-1-artifact-pane.png`, and `qa/round-1-narrow.png`.

### Open Questions

- Should the artifact cards use a shared fixed preview height, or continue sizing to the content type?
- Should the right pane preserve tab order by first-opened order or move the most recently focused tab to the front?

### Approval Path

If approved, the next round can refine one complete workflow end to end, including the empty/loading/error states for its Response Artifact. If not, revise the composer mode hierarchy, artifact density, or right-pane tab treatment in this revision.

## Round 2 - 2026-07-15 - Message hierarchy and artifact actions

### Changed

- Moved the `Worked for <duration>` thinking-activity disclosure above the model label on existing and newly generated assistant responses.
- Explicitly centered each Response Artifact type icon against its two-line title and metadata block.
- Moved artifact Copy and Reply controls from the header to an action row below the artifact card on the lower left.
- Kept the artifact Expand control in the upper-right header position.

### Review Focus

- Review the new assistant hierarchy: activity disclosure, model label, then response bubble.
- Review whether the artifact identity icon now feels optically centered against the title and metadata.
- Hover or focus an artifact and review whether its lower-left Copy and Reply controls align with the ordinary chat-bubble action pattern.
- Review whether keeping Expand in the header makes its pane-opening behavior sufficiently distinct from conversational actions.

### Feedback And Annotations Applied

- Applied browser comment 1 by vertically centering the artifact icon.
- Applied browser comment 2 by moving `Worked for 18 sec` above `GPT-5`.
- Applied browser comment 3 by moving artifact Copy and Reply actions to the lower left beneath the response artifact.

### Simulated Or Deferred

- The content and timing of the thinking-activity disclosure remain illustrative.
- Artifact Copy, Reply, and Expand behavior remains locally simulated as described in Round 1.

### Verification

- Verified the static and dynamically generated assistant DOM order is thinking activity, model label, response bubble, then message actions.
- Verified the Browser artifact icon, identity block, and header share the same measured vertical center.
- Verified Browser Copy still places the artifact URL on the clipboard from its relocated action row.
- Verified newly created Browser artifacts place Expand in the header and Copy/Reply immediately after the artifact footer.
- Verified the revised 720 by 900 layout has no horizontal document overflow.
- Verified no browser console warnings or errors.
- Saved review captures in `qa/round-2-message-hierarchy.png` and `qa/round-2-artifact-actions.png`.

### Open Questions

- Should artifact actions align to the card edge as shown, or indent to the preview content edge?

### Approval Path

If approved, the next round can refine one complete Response Artifact workflow through its empty, loading, success, and error states. If not, revise the assistant hierarchy or artifact action placement within this revision.

## Round 3 - 2026-07-15 - Unified assistant preamble

### Changed

- Moved each ordinary response’s `Worked for <duration>` disclosure into a full-width row above both the assistant icon and model label.
- Added the same thinking disclosure, assistant icon, and active model label above every Browser, File, and Terminal Response Artifact.
- Separated artifact assistant metadata from the bordered artifact surface so the hierarchy matches an ordinary assistant response.
- Corrected the artifact type-icon selector so the SVG is centered inside its square on both axes.
- Applied the unified hierarchy to newly generated chat responses and artifacts as well as the initial examples.

### Review Focus

- Review the repeated hierarchy across ordinary responses and artifacts: `Worked for…`, assistant icon plus model, then response content.
- Review whether the repeated preamble adds useful provenance without making the thread too tall.
- Review the Browser, File, and Terminal type icons inside their artifact headers for exact optical centering.

### Feedback And Annotations Applied

- Applied browser comment 1 by correcting the selector conflict that pinned the artifact SVG to the top of its square.
- Applied browser comment 2 by moving the thinking disclosure above the entire icon/model row.
- Applied browser comment 3 by adding thinking disclosure, assistant icon, and `GPT-5` to all Response Artifacts.

### Simulated Or Deferred

- Work durations and thinking-activity content remain illustrative.
- Each artifact currently has one collapsed thinking disclosure; artifact-specific progress detail can be refined with the detailed state workflow.

### Verification

- Verified ordinary assistant responses render thinking disclosure, then assistant icon/model, then the response bubble for both initial and generated messages.
- Verified all three initial artifact types and newly generated File artifacts render thinking disclosure, assistant icon/model, bordered artifact surface, and action row in that order.
- Measured the Browser type-icon SVG and its square container at identical horizontal and vertical center coordinates with a zero-pixel delta.
- Verified the 720 by 900 layout remains free of horizontal document overflow.
- Verified no browser console warnings or errors.
- Saved the review capture in `qa/round-3-response-hierarchy.png`.

### Open Questions

- Should all artifact durations use the same compact format, such as `8 sec`, or switch to `0:08` when artifact activity becomes more detailed?

### Approval Path

If approved, the next round can refine one complete Response Artifact workflow through its empty, loading, success, and error states. If not, revise the unified response preamble or artifact density within this revision.

## Round 4 - 2026-07-15 - Flush response alignment and constrained artifacts

### Changed

- Grouped the assistant icon and model label into one identity row beneath the thinking disclosure.
- Moved the ordinary assistant bubble onto its own row beneath that identity and aligned it flush with the thinking disclosure and icon.
- Constrained all Browser, File, and Terminal Response Artifacts to the same left-aligned reading width as assistant bubbles.
- Applied the same hierarchy and width rules to newly generated assistant messages and artifacts.

### Review Focus

- Review the shared left edge across `Worked for…`, the assistant icon/model row, and the response bubble.
- Review whether the narrower artifact width feels consistent with normal responses while remaining usable for browser, file, and terminal content.
- Review whether the browser address field and terminal output remain comfortably readable at the constrained width.

### Feedback And Annotations Applied

- Applied browser comment 1 by moving the response text beneath the icon/model row and making all three rows flush left.
- Applied browser comment 2 by shortening Response Artifacts so they no longer stretch to the thread’s right edge.

### Simulated Or Deferred

- Artifact content remains illustrative and does not yet adapt its width based on specific output length.
- The right-pane expanded view remains full-width within the pane and is unchanged by this inline-width refinement.

### Verification

- Measured the ordinary response thinking disclosure, icon/model row, and bubble at the same 256px left coordinate.
- Verified a newly generated assistant response uses the same three-part hierarchy and identical left coordinates.
- Measured initial and newly generated Terminal artifacts at approximately 81% of the available desktop thread width and aligned to the response left edge.
- Verified the 720 by 900 layout remains free of horizontal document overflow.
- Verified no browser console warnings or errors.
- Saved the review capture in `qa/round-4-aligned-widths.png`.

### Open Questions

- Should very short Terminal output be allowed to shrink below the shared artifact width, or should artifact cards retain this consistent width across types?

### Approval Path

If approved, the next round can refine one complete Response Artifact workflow through its empty, loading, success, and error states. If not, revise the shared left alignment or inline artifact width within this revision.

## Round 5 - 2026-07-15 - Shared response container cleanup

### Changed

- Applied the assistant bubble’s compact 5px lower-left corner to every inline Response Artifact surface.
- Added shared `--response-max` and `--response-radius` tokens for response geometry.
- Reused the shared response-width token for thinking disclosures, assistant bubbles, and artifact cards.
- Added one reusable `.response-container` class to initial and generated assistant bubbles and artifact surfaces.
- Consolidated the duplicated message/artifact model-row selectors.
- Removed obsolete assistant-grid, artifact-radius, and visibility declarations that were superseded by the current layout.

### Review Focus

- Review the lower-left corner treatment across assistant bubbles and Browser, File, and Terminal artifacts.
- Confirm the asymmetric response shape feels consistent at both compact bubble and larger artifact sizes.

### Feedback And Annotations Applied

- Applied the direct request for matching lower-left border radii.
- Applied the direct request to reuse CSS selectors for response bubbles and Response Artifact containers and clean up related CSS.

### Simulated Or Deferred

- No interaction behavior changed in this cleanup round.
- The right-pane artifact viewer retains its existing independent container treatment.

### Verification

- Computed styles confirmed initial assistant and artifact containers both use `16px 16px 16px 5px`, a 5px lower-left radius, and the same 1px border.
- Verified newly generated Chat and Browser responses inherit the same shared radius through `.response-container`.
- Verified all initial response surfaces and generated surfaces render through the shared class.
- Verified the 720 by 900 layout remains free of horizontal document overflow and retains the 5px lower-left radius.
- Verified no browser console warnings or errors.
- Saved the review capture in `qa/round-5-shared-response-containers.png`.

### Open Questions

- None for this final CSS refinement.

### Approval Path

If approved, this closes the response-container refinement rounds and the next step can be a detailed artifact-state round or a local commit when requested. If not, revise the shared response geometry within this revision.
