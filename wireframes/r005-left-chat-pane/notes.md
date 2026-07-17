# r005 Review Notes

## Round 1 — Left Chat Pane

- Date: 2026-07-15
- Phase: Conceptual wireframes.
- Revision: `r005-left-chat-pane`.
- Feedback applied:
  - Replace the floating Chat frame with a bottom pane inside the left panel.
  - Show the pane only while Browser or File owns the center workspace.
  - Default pane height to 40% and allow vertical resizing up to 60%.
  - Do not reopen the left panel when an artifact is expanded while it is collapsed.
  - Repair left-panel resizing so it works repeatedly and changes center width.
- Changed behavior:
  - The real thread DOM moves between the center host and the contextual left Chat pane.
  - Restore Chat returns the thread to center and the focused artifact to its original bubble.
  - The pane’s top separator supports pointer and keyboard resizing and retains session height.
  - Left-panel width uses a responsive maximum: 55% of the workspace while preserving at least 420px for center where possible.
  - Resize cleanup is reset after every drag, so later drags begin from fresh geometry.
  - Collapsing or expanding the left panel does not change artifact focus.
- Simulated behavior: File, Browser, Terminal, AI, and persistence behavior remain in-memory wireframe simulations.
- Review focus: Left-pane proportions, pane-height range, center-space tradeoff, and collapsed-panel behavior.
- Open questions: None blocking this round.
- Approval path: Approval keeps `r005` as the active wireframe revision; additional visual feedback remains a new review round inside this folder unless it materially changes the layout again.

## Round 2 — Artifact Focus Controls

- Date: 2026-07-15
- Phase: Conceptual wireframes.
- Revision: `r005-left-chat-pane`.
- Feedback applied:
  - Keep inline File artifacts read-only while they are inside the artifact-focused Chat pane.
  - Lock the fixed composer to Chat and remove the mode chooser during artifact focus.
  - Remove inline Browser back, forward, and refresh controls while retaining its read-only address.
  - Let the full-workspace File surface fill all center height available above the composer.
  - Add a direct Close control to focused Browser and File surfaces for collapsed-left-panel recovery.
  - Replace the focused File text Edit control with a pencil icon.
- Changed behavior:
  - Entering artifact focus remembers the prior composer mode and switches to Chat; restoring Chat restores that prior mode.
  - Browser/File cards remain useful as compact read-only transcript context while their full surfaces own the center workspace.
  - Both focused surfaces can restore the centered Chat thread without relying on the left Chat pane header.
- Simulated behavior: Browser navigation, file editing/saving, and mode persistence remain in-memory prototype interactions.
- Review focus: Full-height File workspace, reduced inline controls, Chat-only composer, and direct close behavior with the left panel collapsed.
- Open questions: None blocking this round.
- Approval path: Approval keeps `r005` as the active wireframe revision; further layout changes can begin a new revision.
