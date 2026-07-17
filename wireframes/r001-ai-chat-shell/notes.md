# Wireframe Review Notes

## Round 1 - 2026-07-15 - Initial AI chat shell

### Changed

- Created the first grayscale SPA wireframe with independently resizable left and right panels.
- Added a centered AI chat thread with its composer fixed to the bottom of the center workspace.
- Added interactive attach, approval, Git branch, model, microphone, and send controls.
- Added desktop panel collapse controls and narrow-screen overlay behavior.

### Review Focus

- Review the starting widths and overall balance of the three-panel shell.
- Review whether the central thread density and bottom composer placement feel appropriate for a desktop AI chat.
- Review the grouping, order, and relative emphasis of the prompt controls.

### Feedback And Annotations Applied

- Applied the direct request to keep both side panels visually blank.
- Applied the direct request to begin with the panel layout before defining side-panel content.

### Simulated Or Deferred

- Message submission and the AI reply are simulated in the browser.
- File selection displays a local filename but does not upload it.
- Microphone state is illustrative and captures no audio.
- Approval, branch, and model choices update only the local wireframe state.
- Side-panel content and navigation are deferred.

### Verification

- Opened `workflows.html` and followed its entry link to `index.html` in the in-app browser.
- Verified the wide desktop layout and a 720 by 900 narrow viewport.
- Verified keyboard panel resizing from 248px to 264px, panel collapse and restore, approval-mode selection, prompt submission, and the simulated AI reply.
- Verified no browser console warnings or errors.
- Saved review captures in `qa/desktop.png` and `qa/narrow.png`.

### Open Questions

- Should the left panel or the right panel be defined first after this shell is approved?
- Should either side panel start collapsed by default on desktop?

### Approval Path

If approved, the next step is defining the left panel's content and navigation in the next wireframe review round. If not, revise the shell proportions, thread density, or prompt-control arrangement in this revision.

## Round 2 - 2026-07-15 - Message treatment and prompt order

### Changed

- Removed the visible `You` label from all existing and newly submitted user messages.
- Replaced assistant `AI` labels with the active model name, initially `GPT-5`.
- Presented assistant responses as left-aligned chat bubbles.
- Added a collapsed `Worked for <duration>` disclosure above each assistant bubble.
- Added an expandable high-level work summary to each assistant response.
- Reordered composer controls to Attach, model, approval, and Git branch.

### Review Focus

- Review whether the left/right bubble treatment creates the desired conversation rhythm.
- Review the model label and duration hierarchy above assistant bubbles.
- Expand `Worked for 1 min 5 sec` and review whether the work-summary detail is useful without overwhelming the response.
- Review the revised prompt-control order.

### Feedback And Annotations Applied

- Applied browser comments 1 and 2 by removing both visible `You` labels.
- Applied browser comments 3 and 4 by using the model name, adding assistant bubbles, and adding expandable duration summaries.
- Applied browser comment 5 by moving model before approval and branch.

### Simulated Or Deferred

- Work durations and work-summary content are illustrative.
- The disclosure shows a concise user-facing summary of major actions, not private chain-of-thought reasoning.
- Model selection changes the label for newly generated illustrative responses but does not call a model backend.

### Verification

- Verified both existing assistant messages show `GPT-5`, both user labels are absent, and the toolbar DOM order is Attach, model, approval, branch.
- Verified the first work summary expands and reveals all three summary steps.
- Verified a newly submitted prompt creates an unlabeled user bubble plus a `GPT-5` assistant bubble with a duration disclosure.
- Verified the revised messages at the default desktop viewport and at 720 by 900 with no horizontal overflow.
- Verified no browser console warnings or errors.
- Saved review captures in `qa/round-2-desktop.png` and `qa/round-2-narrow.png`.

### Open Questions

- Is `Worked for <duration>` the preferred wording, or should the control use a shorter label such as `View work`?
- Should the expanded summary allow more than one assistant message to stay open at a time?

### Approval Path

If approved, the next step is defining the left panel's content and navigation in the next wireframe review round. If not, revise the bubble treatment, work-summary disclosure, or prompt-control ordering in this revision.

## Round 3 - 2026-07-15 - Unboxed thinking activity

### Changed

- Replaced each boxed work-summary panel with a plain, unboxed thinking-activity stream.
- Removed the visible `Work summary` heading and bulleted-list treatment.
- Added short progress paragraphs and muted icon-led activity rows based on the supplied reference image.
- Applied the same activity-stream pattern to newly submitted illustrative responses.

### Review Focus

- Expand `Worked for 1 min 5 sec` and review the rhythm between progress paragraphs and activity rows.
- Review whether the muted activity rows are distinct enough without becoming visually heavy.
- Review the spacing between the expanded thinking activity and the assistant response bubble.

### Feedback And Annotations Applied

- Applied the supplied thinking-process reference image.
- Applied the direct request to remove the thinking-process box and border.
- Applied the direct request to remove the visible work-summary treatment.

### Simulated Or Deferred

- Durations, progress narration, and activity rows are illustrative.
- The expanded stream represents user-facing progress and tool activity, not private chain-of-thought reasoning.

### Verification

- Verified the first duration disclosure expands to two progress paragraphs and two activity rows.
- Verified the expanded content has no border, background, box shadow, `Work summary` heading, or summary list.
- Verified a newly submitted response receives the same unboxed activity stream.
- Verified no browser console warnings or errors.
- Saved the expanded-state review capture in `qa/round-3-thinking.png`.

### Open Questions

- Should activity rows include elapsed timestamps, or remain as concise status lines?

### Approval Path

If approved, the next step is defining the left panel's content and navigation in the next wireframe review round. If not, revise the thinking-activity wording, spacing, or activity-row treatment in this revision.
