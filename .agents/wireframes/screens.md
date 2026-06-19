# Wireframe Screens

Status: active
Updated: 2026-06-18
Source:
- `plans/product-interface.md`
- `plans/pegs/*.png`

## Screen Pegs

| Screen | Source |
| --- | --- |
| App start | `plans/pegs/C4OS-App-Start-Wireframe.png` |
| New session | `plans/pegs/C4OS-New-Session-Wireframe.png` |
| Chat session | `plans/pegs/C4OS-Chat-Session-Wireframe.png` |
| Providers popover | `plans/pegs/C4OS-Providers-Popover-Wireframe.png` |
| Models popover | `plans/pegs/C4OS-Models-Popover-Wireframe.png` |
| File explorer right panel | `plans/pegs/C4OS-File-Explorer-Right-Wireframe.png` |
| File editor right panel | `plans/pegs/C4OS-File-Editor-Right-Wireframe.png` |
| Terminal right panel | `plans/pegs/C4OS-Terminal-Right-Wireframe.png` |
| Settings providers | `plans/pegs/C4OS-Settings-Providers-Wireframe.png` |
| Add provider | `plans/pegs/C4OS-Settings-Add-Provider-Wireframe.png` |
| Settings models | `plans/pegs/C4OS-Settings-Models-Wireframe.png` |
| Settings configuration | `plans/pegs/C4OS-Settings-Configuration-Wireframe.png` |

## Imported UI Rules

- Use a three-panel desktop shell.
- Left and right panels are resizable.
- Right tool tab order is Browser, Files, Terminal.
- The prompt composer is disabled until a trusted project exists.
- The empty-state prompt asks `What should we build in c4os2?` until final product copy is decided.
- User messages align right and agent messages align left.
- File explorer begins directly with folders/files under the active project.
- Code view fills the right-panel body and should not look like a floating card.
- Settings navigation order is Providers, Models, Configuration, Skills, MCP Servers, Hooks.
- MVP interface should remain neutral and utilitarian, use lucide icons, and avoid dark theme or brand styling unless a later design phase approves it.

## Promotion Notes

Before freeze, accepted wireframe behavior should be promoted into `.agents/specs/mvp/requirements.md` and `.agents/specs/mvp/acceptance.md`.

## Functional Drafts

| Revision | Path | Status | Notes |
| --- | --- | --- | --- |
| r01-functional-wireframes | `r01-functional-wireframes/index.html` | ready-for-review | HTML/CSS/JS grayscale draft covering the peg-backed shell, session, right-panel, popover, and settings screens. |
