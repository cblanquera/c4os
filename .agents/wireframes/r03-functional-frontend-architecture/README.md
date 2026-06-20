# C4OS r03 Functional Frontend Architecture

Draft stage: functional creative draft for desktop app interface review.

This revision is intentionally not based on the code structure of `r01` or
`r02`. It uses the same product inputs and peg-backed screen list, but the
artifact source was written fresh.

Source inputs:

- `plans/product-interface.md`
- `.agents/wireframes/screens.md`
- `plans/pegs/*.png`

Frontend strategy:

- Thin HTML route files declare the active screen with `data-route`.
- `script.js` uses explicit screen data, small DOM/component helpers, and route-level render functions.
- `styles.css` is organized as layers: tokens, base, primitives, review hub, app start, shell, workspace content, tool panel, settings, and desktop-narrow behavior.
- JavaScript is limited to static screen composition and the simulated Show More state.
- Layout uses CSS grid/flex and fixed desktop app surfaces instead of JS measurement.

Visual strategy:

- Subject: local-first desktop command center for technical users.
- Signature element: the trust spine, a narrow green vertical marker that appears on shell boundaries and selected/trusted states.
- Palette: paper `#fbfbf6`, ink `#1e2320`, rail `#e9ece4`, trust green `#0f8b6f`, decision amber `#b8692f`, code black-green `#151b18`.
- Type: system UI for application controls, restrained display face fallback for major screen titles, monospace for code and terminal panes.

What is clickable:

- Review hub cards.
- App start recent workspace rows.
- Sidebar project, session, and settings navigation.
- Composer model chip.
- Provider/model popover rows.
- Right panel Browser, Files, and Terminal tabs.
- File explorer rows and breadcrumbs.
- Settings navigation and provider form actions.
- Show More / Show Less in the chat session.

What is simulated:

- Folder opening, cloning, and workspace-file opening.
- Prompt submission.
- Provider/model persistence.
- Browser rendering.
- File editing and saving.
- Terminal execution.
- Approval, sandbox, and provider toggles.

Safe to delete: yes. This is a review artifact, not production code.
