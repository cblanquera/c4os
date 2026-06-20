# C4OS r02 Functional Interface Draft

Draft stage: functional creative wireframe for desktop app review.

Source inputs:

- `plans/product-interface.md`
- `wireframes/screens.md`
- `plans/pegs/*.png`
- `wireframes/r01-functional-wireframes/`

Design strategy:

- Preserve the peg-backed three-panel desktop shell and settings flows.
- Move the visual system from grayscale boxes to a scalable workbench vocabulary: warm neutral canvas, graphite code surfaces, amber decision/approval accents, dense rails, and explicit resize handles.
- Prefer CSS layout and state styling over JavaScript. JavaScript remains only for screen assembly and the simulated show-more interaction from the prior draft.
- Optimize for desktop app windows first. Narrow-window rules compress the workbench but do not convert it into a mobile website.

What is clickable:

- Review hub screen links.
- App start recent workspace rows.
- Left project/session navigation.
- Right Browser, Files, and Terminal tabs.
- Provider and model popover navigation.
- File explorer rows and breadcrumbs.
- Settings navigation and provider add/cancel links.
- Show More / Show Less controls in the chat session.

What is simulated:

- Folder opening, cloning, and workspace-file opening.
- Real chat submission.
- Model/provider persistence.
- Browser rendering.
- File editing and saving.
- Terminal execution.
- Provider/model fetching.
- Approval and sandbox changes.

Known limitations:

- Icons are embedded as inline Lucide-style SVG paths for draft portability.
- The draft does not use app runtime state, persistence, credentials, shell commands, file writes, or backend calls.
- Screen assembly is still centralized in `script.js`; production implementation should split this into app components while preserving the CSS token and layout concepts.

QA artifacts:

- `qa/notes.md`

Safe to delete: yes. This is a design-review artifact, not production app code.
