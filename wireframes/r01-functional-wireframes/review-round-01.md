# Wireframe Review Round 01

Phase: wireframes
Revision: r01-functional-wireframes
Status: ready-for-review
Created: 2026-06-18

## Scope Covered

- App start review hub.
- Main three-panel app shell.
- New session empty state.
- Chat session with messenger-style ownership, show-more behavior, tool call, and approval block.
- Provider and model popovers.
- Right-panel Browser, Files, File Editor, and Terminal states.
- Settings Providers, Add Provider, Models, and Configuration screens.

## Simulated Behavior

- Links navigate between peg-backed screens.
- Model chip links to provider/model popover states.
- Show more/Show less toggles the first agent bubble.
- File tree opens the code-view state.
- Settings navigation moves between settings screens.

## Icon Source

- Interface icons use inline Lucide-style SVGs based on `https://lucide.dev/icons/` names.
- Icons are embedded in `script.js` so the static review artifact remains portable and does not require a CDN.

## Source Notes

- Based on `../../../plans/product-interface.md`.
- Screen list cross-checks `../screens.md`.
- Visual peg references remain in `../../../plans/pegs/`.

## Deferred

- Mobile responsive wireframe variants.
- Approval modal details.
- Project creation, clone, and trust setup flow.
- Exact empty/loading/error/success states outside the visible pegs.
