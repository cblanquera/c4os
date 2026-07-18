# Response Artifact Contract

## Shared Provider Model

A Response Artifact is a structured assistant output. Each provider registers:

- type, label, icon, and accessible identity
- whether it supports focus/Expand
- response-body renderer
- optional focused-body renderer
- contextual Copy value
- Reply behavior
- provider-specific state and controls

Unknown types use a generic icon and remain inline-only. Shared shells own transcript placement, C4OS preamble, frame, actions, scrolling boundaries, focus transitions, and compact-pane adaptation. Providers own the contents inside the frame.

## Shared Shells

### Inline response

Order: work disclosure, C4OS/type identity row, constrained provider frame, then Copy/Reply action row. Expand stays in the frame header when supported. The frame header and footer are static; only the body scrolls. Default maximum body height is responsive between 260px and 440px.

### Focused workspace

The artifact fills the center region above the fixed composer. It may substantially differ from the inline body. It mirrors the same provider state and exposes direct Close. The source artifact becomes a compact selected placeholder in the contextual transcript.

### Contextual-pane response

Reuse the original inline DOM/state with a compact modifier. Browser and File cards become read-oriented while their focused surface owns operation. Copy, Reply, identity, state, and supported Expand remain available.

## Browser Artifact

- Input: address from Browser mode. Normalize a missing scheme for display/navigation.
- Inline header: Back, Forward, Refresh, read-only current address, far-right Expand.
- Body: compact page preview and navigation/history feedback.
- Copy: current URL.
- Reply: append the instruction to Chat, update the original Browser state in place, and add an activity/navigation entry.
- Focused state: full center browser surface with navigation, read-only address, and Close.
- Contextual-pane state: keep read-only address and Expand; remove Back/Forward/Refresh to avoid competing with the focused browser.
- Back/Forward disabled state follows history position. Refresh preserves address and reports activity.
- Browser sub-tabs, real networking, credentials, and embedded production webviews were not specified by r012.

## Direct File Artifact

- Input: one file path or native file selection from Files mode.
- Default: breadcrumbs, version/type metadata where useful, and a read view.
- Inline Edit enters a line-numbered editor with Cancel/Discard and Save. The editor grows to content where appropriate, remains vertically resizable inline, and scrolls internally for long content.
- Save updates in-memory/production file state through approval policy, increments the displayed version when versioning is shown, and returns to compact read mode.
- Cancel/Discard restores the unchanged content and compact height; trash means discard edits, never delete the file.
- Reply creates a proposed change/diff with Approve and Reject. Approve saves immediately through the required approval path; Reject returns to unchanged read state.
- Copy: current visible file contents.
- Focused view: full-height document surface, clickable folder breadcrumbs, pencil Edit, fixed line-number gutter, reduced editor padding, internal vertical/horizontal scrolling, Close.
- Preserve editing flag, draft, scroll-relevant content, and line numbering between inline and focused contexts.
- In the contextual Chat pane, hide Edit/Discard/Save; editing belongs to the focused surface.

## File Explorer Artifact

- Files Browse offers File or Folder. A folder creates a read-only explorer artifact.
- Show clickable breadcrumbs plus folder and file rows with icon, name, and relevant metadata.
- Folder selection replaces the current listing in place; it does not force focus.
- File selection replaces the listing with that file's normal view state and retains Edit capability.
- Parent breadcrumbs return to that folder. A breadcrumb from a direct File may convert the artifact in place into its containing Folder view.
- Focused Explorer fills center and omits a redundant `Browsing / item count` footer.
- Inline Folder may retain compact status metadata.
- Closing focus synchronizes current folder, opened file, view/edit state, and unsaved draft back to the inline artifact.
- Listings, file reads, and writes use the runtime/security architecture; the r012 fixture data was illustrative.

## Terminal Artifact And Session

Each chat owns one persistent shell session, represented as `shell-1` in the deterministic fixture. Every command creates a new immutable response artifact snapshot associated with that session.

### Inline card

- Header title is the executed command; do not repeat the command in output.
- Show working directory/session metadata, output, and status/exit result.
- Every completed, running, interrupted, seeded, and newly generated card exposes Expand.
- Only the active long-running foreground process exposes Stop.
- Copy includes command plus output.
- Reply creates a new conversational instruction and new result card; it does not mutate the target.

### Expanded session

- Show the selected command or process as one continuous terminal body, not stacked history cards.
- Earlier commands remain accessible through transcript artifacts.
- The selected running process streams output in place and may expose contextual stdin when it requests ordinary input.
- Stop conceptually sends `Ctrl+C`, appends `^C`, marks Interrupted with exit 130, preserves working directory/history, and returns an inline `$` prompt.
- Completion also returns an inline `$` prompt.
- Submitting at the idle prompt creates the next command/artifact in the same chat/session and makes it the selected expanded body.
- The global composer remains visible but is locked to Chat. Terminal-local input owns commands and process stdin.
- Full-screen TUIs, password entry, and production PTY details are outside this visual contract.

## Streaming Contract

Only newly generated assistant turns animate:

1. Open the work disclosure.
2. Show/type generic starting activity and concise progress.
3. Show/type working/tool activity; show a reasoning summary only when supplied by the effective route.
4. Collapse to `Worked for <duration>`.
5. Reveal the identity and final output.

Never present private chain-of-thought or fabricate a reasoning summary. Ordinary Chat final Markdown may progressively render with a cursor, followed by one clean final render. Artifact frames remain hidden until work completes, then reveal as a unit; do not type provider-owned body content character-by-character. Set `aria-busy` during generation and remove it at completion. Reduced motion jumps to the same final state.

## Artifact Acceptance

Verify every artifact in inline, contextual-pane, and focused contexts; internal scrolling; static chrome; Copy; Reply; focus/Close/Restore; artifact-to-artifact swap; left-panel collapsed behavior; file edit continuity; Explorer navigation continuity; Browser history; terminal completed/running/interrupted flows; Stop; stdin; returned prompt; and creation of the next command card.
