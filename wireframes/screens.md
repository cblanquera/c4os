# Wireframe Screens

Status: active
Updated: 2026-07-13
Source:
- `plans/product-interface.md`
- `plans/pegs/*.png`
- `.agents/specs/00002-shell-plugin-architecture-refactor/`
- `.agents/specs/00003-core-app-shell-ux/`
- `.agents/specs/00007-file-system-plugin/`
- `.agents/specs/00008-file-editor-plugin/`
- `.agents/specs/00009-terminal-plugin/`
- `.agents/specs/00010-chat-debug-plugin/`
- `.agents/specs/00011-browser-plugin/`

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
| Settings runtimes | `wireframes/r04-single-page-app/` r04 addition; no peg image |
| Settings configuration | `plans/pegs/C4OS-Settings-Configuration-Wireframe.png` |

## Imported UI Rules

- r04 remains the accepted MVP baseline for the original three-panel desktop
  shell and route set.
- r05 final-implementation Batch 1 supersedes the r04 shell model for specs
  00002 and 00003 only: use one global header, left/right plugin icon groups, no default
  right panel, and icon-owned plugin panel toggle/close behavior.
- Left and right plugin panels are resizable when visible.
- The final shell allows one visible plugin panel per side. Same-side plugin
  icon clicks replace the visible panel; left and right panels can coexist.
- Plugin panel visibility persists per chat session and is restored on chat
  switch.
- Settings opens as a center route, closes plugin panels while active, and
  restores the chat's prior panels when leaving Settings.
- Resize behavior preserves a 640px center-pane minimum by closing the
  opposite-side panel before clamping expansion.
- Hidden compatible plugins may update per-chat state and unread/activity
  indicators, but must not open panels, steal focus, prompt the user, or
  trigger duplicate backend calls.
- Invalid shell layout declarations must surface repair/disable states instead
  of corrupting shell layout.
- r05 final-implementation Batch 4 supersedes r04 only for focused workspace,
  project, file explorer, and file editor states in specs 00007 and 00008.
- Workspace start, loaded workspace, missing project, search, and non-Git
  workspace states belong inside the FS plugin panel. The center pane remains
  the normal new-chat prompt surface.
- Missing projects use muted strike-through project rows and read-only chat
  rows until relocated. Missing project menus show Relocate, Copy path, Rename,
  and Remove. Found project menus show Reveal, Copy path, Rename, and Remove.
- File Editor is a separate plugin from File System. It may mount on the left
  or right, uses r04-style file explorer density, and opens files into the File
  Editor code view through file-row clicks.
- File Editor dirty, conflict, create/rename/delete-to-trash, context-menu,
  icon-theme, hidden-file, and non-code empty states are represented in r05
  Batch 4 and should not be inferred from stale r04 fixed-tab behavior.
- r05 final-implementation Batch 5 supersedes r04 only for focused runtime
  plugin panel states in specs 00009, 00010, and 00011, plus directly affected spec 00005
  overlap. Terminal is the user PTY panel only; runtime terminal tool output
  belongs to thread context and Chat Debug. Browser visible routes cover
  navigation/actions, toolbar menu, page context menu, and preview hosting,
  while annotation capture/state behavior is documented in companion Markdown
  and spec evidence. Chat Debug visible routes cover command/tool result
  history, run history, and redacted event detail without export controls.
- The prompt composer is disabled until a trusted project exists.
- The empty-state prompt asks `What should we build in c4os2?` until final product copy is decided.
- User messages align right and agent messages align left.
- File explorer begins directly with folders/files under the active project.
- Code view fills the right-panel body and should not look like a floating card.
- Settings navigation order is Providers, Models, Runtimes, Configuration, Plugins, Skills, MCP Servers.
- MVP interface should remain neutral and utilitarian, use lucide icons, and avoid dark theme or brand styling unless a later design phase approves it.

## Promotion Notes

Before a current spec is Frozen, accepted wireframe behavior should be reconciled into that spec's requirements, acceptance, evidence, and traceability records.

The implementation-facing UI handoff is `wireframes/ui-handoff-spec.md`.

## Functional Drafts

| Revision | Path | Status | Notes |
| --- | --- | --- | --- |
| r01-functional-wireframes | `r01-functional-wireframes/index.html` | ready-for-review | HTML/CSS/JS grayscale draft covering the peg-backed shell, session, right-panel, popover, and settings screens. |
| r02-functional-interface-draft | `r02-functional-interface-draft/index.html` | ready-for-review | HTML/CSS/JS desktop interface draft with scalable CSS tokens, peg-backed screen coverage, distinct app-start state, and review-only simulated interactions. |
| r03-functional-frontend-architecture | `r03-functional-frontend-architecture/index.html` | ready-for-review | Fresh HTML/CSS/JS desktop interface draft with route-level screen declarations, component-style DOM helpers, layered CSS, trust-spine state treatment, and review-only simulated interactions. |
| r04-single-page-app | `r04-single-page-app/index.html` | accepted MVP baseline | HTML/CSS/JS single-page app baseline for original MVP shell, settings, right-tool tabs, and implementation handoff. |
| r05-final-implementation | `r05-final-implementation/index.html` | approved Batches 1-5 | Final-implementation review artifact. Batch 1 covers shell foundation, Batch 2 covers Settings, Batch 3 covers prompt/approval flows, Batch 4 covers workspace/files states for specs 00007 and 00008, and Batch 5 covers runtime plugin panel states for specs 00009, 00010, and 00011 with direct spec 00005 overlap. |
