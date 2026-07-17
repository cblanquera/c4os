# R011 Workspace Projects Review Notes

## Review Round 7 — 2026-07-17 — Compact start actions

### Changed

- Removed the forced minimum height from stacked workspace-action cards so their bottom padding matches their top padding.
- Retained the icon-to-title top alignment accepted from the previous refinement.

### Feedback Applied

- Applied the browser annotation that the top-aligned cards had too much empty space beneath their content.

### Review Focus

- Whether the stacked action cards now feel evenly padded above and below their icon and copy.

### Simulated Or Deferred Behavior

- None for this spacing refinement.

### Open Questions

- None blocking this review round.

### Approval Path

- Approval of Review Round 7 accepts the compact stacked workspace-action spacing. Further product refinements remain in r011 as another review round.

## Review Round 6 — 2026-07-17 — Start action alignment

### Changed

- Top-aligned each workspace-action icon with its title in the stacked start-screen layout.
- Kept the title and description together with a consistent five-pixel gap.

### Feedback Applied

- Applied the browser annotation requesting matching icon and text alignment on the “Open a folder” card to all three workspace actions.

### Review Focus

- Whether the folder, workspace, and repository icons align with the first line of their respective text blocks at narrow window widths.

### Simulated Or Deferred Behavior

- None for this visual alignment change.

### Open Questions

- None blocking this review round.

### Approval Path

- Approval of Review Round 6 accepts the stacked workspace-action alignment. Further product refinements remain in r011 as another review round.

## Review Round 5 — 2026-07-17 — Coding audit fixes

### Changed

- Promoted pending project chats on attachment-only first submissions as well as text prompts.
- Used the first attachment filename as the initial chat title when no prompt text exists.
- Added a shared start-new-thread transition that exits Reply, restores Chat, clears stale input and attachments, and closes focused artifacts before showing the blank thread.
- Reconciled active and pending thread state when projects or sessions are removed, including fallback-session selection and a no-session workspace state.
- Moved the inline project rename input outside the project toggle button and synchronized project action accessible names after rename or relocation.
- Routed project-path copying through the shared clipboard controller, including a real legacy fallback and failure feedback.
- Replaced the two password-field labels that wrapped buttons with explicit labels and separate control groups; updated hidden-field validation to match.

### Feedback Applied

- Applied all P1, P2, and P3 findings from the requested `chrisai-coding` audit pass.

### Review Focus

- Whether New chat cleanly exits an in-progress Reply and starts with an empty composer.
- Whether an attachment-only first submission creates the sidebar chat under the correct project.
- Whether removing the active session, active project, or pending project always lands on a valid remaining state.
- Whether project rename, relocation, and Copy path retain their expected interaction and keyboard behavior.

### Simulated Or Deferred Behavior

- Filesystem changes, durable project/session persistence, and OS Reveal remain simulated.
- Clipboard behavior uses the browser Clipboard API with a legacy selection fallback in the static preview.

### Open Questions

- None blocking this review round.

### Approval Path

- Approval of Review Round 5 accepts the coding-audit repairs. Further product refinements remain in r011 as another review round.

## Review Round 4 — 2026-07-17 — Thread header simplification

### Changed

- Removed “Chat, files, browser, and terminal” from the workspace header.
- Removed the subtitle’s unused desktop and narrow-viewport CSS.
- Kept the active or pending thread title centered as the header’s only text.

### Feedback Applied

- Applied both browser annotations requesting removal of the repeated workspace-header subtitle.

### Review Focus

- Whether the single-line active thread title provides the right header density in both blank and saved thread states.

### Simulated Or Deferred Behavior

- None introduced by this copy removal.

### Open Questions

- None blocking this review round.

### Approval Path

- Approval of Review Round 4 accepts the title-only workspace header. Further refinements remain in r011 as another review round.

## Review Round 3 — 2026-07-17 — Deferred new-chat creation

### Changed

- Changed the project New chat action so it opens an unsaved blank thread without inserting a sidebar chat item.
- Added the centered prompt “What do you want to build in <project name>?” to the blank-thread state.
- Deferred session creation until the first non-empty Chat prompt is submitted.
- Derived the new sidebar chat title from the first prompt, truncating titles longer than 48 characters.
- Preserved the previous active thread while the unsaved blank thread is open and restored saved thread markup when selecting an existing session.

### Feedback Applied

- Applied the confirmed state transition: project New chat creates a blank draft; the first prompt promotes it into a saved project chat.
- Applied the approved title behavior: use the first prompt as the initial chat title.

### Review Focus

- Whether the blank-thread message is positioned and weighted appropriately in the center workspace.
- Whether no sidebar item appears before the first prompt.
- Whether the first prompt creates the item under the correct project with a useful title.

### Simulated Or Deferred Behavior

- Session persistence remains in-memory for the static wireframe. Reloading restores the seeded state.

### Open Questions

- None blocking this review round.

### Approval Path

- Approval of Review Round 3 accepts the deferred new-chat creation flow. Further project-panel refinements remain in r011 as another review round.

## Review Round 2 — 2026-07-17 — Missing state and panel height

### Changed

- Removed the visible `Missing` label from the `legacy-ui` project row.
- Changed the missing project name to a lighter gray, italic treatment.
- Let the project list flex through the full remaining height of the left panel with no maximum height.

### Feedback Applied

- Applied browser annotation 1: communicate the missing path through the project-name styling instead of explicit status copy.
- Applied browser annotation 2: remove the apparent project-list height cap.

### Review Focus

- Whether the lighter italic `legacy-ui` name communicates an unavailable path without becoming too faint.
- Whether the project list now occupies the expected full panel height.

### Simulated Or Deferred Behavior

- The missing path remains illustrative; Relocate still uses the browser directory picker to simulate choosing a replacement folder.

### Open Questions

- None blocking this review round.

### Approval Path

- Approval of Review Round 2 accepts the missing-project treatment and full-height project list. Further refinements remain in r011 as another review round.

## Review Round 1 — 2026-07-17 — Project and chat-session navigation

### Changed

- Copied the complete accepted r010 revision into r011 before modification.
- Added a left-panel project and chat-session navigation specification based on the supplied peg.
- Added the search-first Projects panel to the existing `index.html`, `styles.css`, and `script.js` SPA files.
- Added three seeded projects with nested sessions, including found, missing, expanded, collapsed, and active states.
- Added native directory selection for adding a project and relocating a missing project.
- Added hover/focus-only project More, New chat, and chat Remove controls.
- Added path-dependent project menus, inline rename, copy/reveal feedback, removal, chat creation, session activation/removal, title search, and native drag sorting.
- Added a Workspace Projects review entry to `workflows.html`.

### Feedback Applied

- Applied the requested search-first left-panel order, Projects add action, nested chat sessions, project hover actions, path-dependent menus, sorting, and session removal behavior.

### Review Focus

- Whether the hierarchy and density match the supplied peg while remaining consistent with C4OS.
- Whether project and chat-session hover actions are discoverable without adding persistent clutter.
- Whether found and missing project states are distinct enough.
- Whether sorting and folder-selection behavior communicate the intended desktop app model.

### Simulated Or Deferred Behavior

- Real filesystem persistence, durable session storage, and OS reveal operations remain simulated; the browser directory picker represents the intended native folder chooser.

### Verification

- Confirmed r011 retains every file copied from r010.
- Confirmed three projects, six chat sessions, and twelve seeded project-menu actions are present.
- Confirmed `script.js` and `markdown.js` pass Node syntax checks.
- Confirmed the repository whitespace check passes.
- Automated inspection of the local `file://` preview is blocked by browser security policy; manual refresh and interaction review remain required.

### Open Questions

- None blocking the first review round.

### Approval Path

- Approval of Review Round 1 locks the left-panel project/session structure and interactions. Requested refinements remain in r011 as another review round.
