# C4OS r04 Single Page App Wireframe

Draft stage: functional creative draft for desktop app interface review.

This revision is based on `r03-functional-frontend-architecture`, including
the current r03 settings/plugin/skill/MCP surfaces, but turns the review
artifact into a single page app.

Source inputs:

- `wireframes/r03-functional-frontend-architecture/`
- `plans/product-interface.md`
- `wireframes/screens.md`
- `plans/pegs/*.png`

Frontend strategy:

- `index.html` is the only page entry point.
- The default entry screen is App Start, not a review menu.
- `script.js` keeps r03's route-level render functions and explicit screen data.
- All generated internal links are converted to hash routes such as `#chat-session`.
- A small client-side router intercepts internal navigation, swaps the active screen, updates the URL hash, and supports direct hash entry.
- `styles.css` is carried forward from r03 so the visual review target stays stable.

What is clickable:

- App start recent workspace rows.
- Sidebar project, session, and settings navigation.
- Composer model chip.
- Provider/model popover rows.
- Right panel Browser, Files, and Terminal tabs.
- File explorer rows and breadcrumbs.
- Settings navigation and provider form actions.
- Show More / Show Less in the chat session.
- Plugin, marketplace, skill, and MCP modal controls.

What is simulated:

- Folder opening, cloning, and workspace-file opening.
- Prompt submission.
- Provider/model persistence.
- Browser rendering.
- File editing and saving.
- Terminal execution.
- Approval, sandbox, provider, plugin, skill, and MCP toggles.

Review URL:

- Open `index.html` directly, or serve this folder and open `/`; both start on App Start.
- Deep links use hash routes, for example `index.html#settings-mcp`.

Safe to delete: yes. This is a review artifact, not production code.
