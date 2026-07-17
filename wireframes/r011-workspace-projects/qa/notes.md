# R011 Workspace Projects QA

## Review Round 7 — 2026-07-17

### Source Verification

- Stacked workspace-action cards no longer carry a forced 118-pixel minimum height.
- Existing 20-pixel card padding now determines equal top and bottom space around the 36-pixel icon and copy block.
- Icon and title top alignment remains unchanged.
- Repository whitespace checks pass.

### Browser Limitation

- The in-app browser control is unavailable, so the revised card height requires manual refresh and visual confirmation in the existing local `file://` preview.

## Review Round 6 — 2026-07-17

### Source Verification

- Stacked workspace-action cards use explicit auto-sized rows with top alignment rather than distributing the title and description across the card height.
- Each action icon spans the two copy rows and begins at the same top edge as its title.
- The desktop three-column card layout remains unchanged.
- Repository whitespace checks pass.

### Browser Limitation

- The in-app browser control is unavailable, so the narrow layout requires manual refresh and visual confirmation in the existing local `file://` preview.

## Review Round 5 — 2026-07-17

### Source Verification

- Pending chats promote on text-only, attachment-only, and combined first submissions.
- Starting a project chat dispatches one shared composer transition that restores Chat, cancels Reply, clears stale drafts, and exits artifact focus.
- Removing active or pending projects and sessions clears detached state and activates a remaining session or stable empty workspace.
- Project rename no longer places an input inside a button; rename and relocation refresh project action accessible names.
- Copy path uses the shared Clipboard API plus legacy fallback and reports failure rather than claiming success.
- Onboarding and Settings password controls use explicit labels outside their input/button groups.
- HTML Tidy no longer reports the two missing/implicit label-boundary warnings found by the audit.
- JavaScript syntax and repository whitespace checks pass.

### Browser Limitation

- The in-app browser control is unavailable, so the corrected interaction combinations require manual refresh and confirmation in the existing local `file://` preview.

## Review Round 4 — 2026-07-17

### Source Verification

- The workspace header contains only the active thread title.
- “Chat, files, browser, and terminal” is absent from the r011 product interface.
- Subtitle-specific CSS is removed from both the base thread-title rules and narrow-viewport media query.
- JavaScript syntax and repository whitespace checks pass.

### Browser Limitation

- The updated local `file://` header requires manual refresh and browser confirmation.

## Review Round 3 — 2026-07-17

### Source Verification

- Project New chat enters an unsaved state without inserting a session row.
- The blank state centers the selected project name in the requested question.
- A capture-phase composer hook promotes the pending thread before the existing Chat submit handler renders its first exchange.
- Empty submissions do not promote the pending thread.
- The first prompt becomes the new session title and is truncated after 48 characters.
- The new session is inserted under, expanded within, and activated for the originating project.
- JavaScript syntax and repository whitespace checks pass.

### Browser Limitation

- The updated local `file://` interaction requires manual refresh and browser confirmation.

## Review Round 2 — 2026-07-17

### Source Verification

- The seeded missing project no longer renders a `Missing` text label.
- Missing project names use the lighter `--gray-400` token with italic type.
- The project list is a flexible child with `min-height: 0` and `max-height: none`, so it fills the remaining panel height and scrolls only when its contents exceed that space.
- JavaScript syntax and repository whitespace checks pass.

### Browser Review

- The user supplied browser screenshots for the pre-change state. The updated local `file://` artifact requires refresh for post-change confirmation.

## Review Round 1 — 2026-07-17

### Source Verification

- r011 contains every file present in r010 before this revision's additions and edits.
- The left panel renders Search chat sessions before the Projects heading and Add action.
- Three seeded projects cover expanded, collapsed, found-path, and missing-path states.
- Found-path menus expose Reveal, Copy path, Rename, and Remove.
- The missing-path menu exposes Relocate, Copy path, Rename, and Remove.
- Project More and New chat actions are hover/focus-only; chat rows expose only a hover/focus Remove action.
- The Add and Relocate paths invoke the directory input; a selection appends a project or repairs the targeted missing project.
- Search filters nested chat-session titles, New chat appends and activates a session, and session Remove deletes only its chat row.
- Native drag events reorder complete project items together with their nested sessions.
- `script.js` and `markdown.js` pass Node syntax checks.
- The repository whitespace check passes.

### Browser Limitation

- Automated inspection of the open local `file://` preview is blocked by browser security policy, so Review Round 1 requires a manual refresh for visual and interaction confirmation.

### Inherited Assets

- Existing screenshots were copied forward with the full r010 revision. They document inherited screens and are not presented as current r011 project-panel verification.

### Deferred

- Durable project/session persistence, true filesystem paths, OS Reveal behavior, native rename/remove operations, and backend chat creation remain simulated.
