# Creative Specs

Status: active
Created: 2026-06-21
Updated: 2026-07-02
Source Note: Normalized from interface, product-experience, and UI handoff context. Detailed UI handoff material is preserved under `.agents/references/context/creative-specs/` and `.agents/references/context/ui-handoff/`.

## Purpose

Use this as the interface and creative gate. It summarizes C4OS experience direction, layout contract, interaction rules, settings surfaces, visual tone, and accessibility expectations before detailed UI handoff references are loaded.

## Load When

- You need UI layout, interaction behavior, visual tone, accessibility rules, or wireframe handoff context.
- You are implementing or reviewing frontend behavior, screen structure, settings navigation, Browser/Files/Terminal panel behavior, or composer/session presentation.
- You need to decide whether to open the detailed r04 UI handoff chunks.

## Skip When

- You only need product thesis, users, goals, or vocabulary; load `product-brief.md`.
- You only need feature inventory or user-facing product behavior without layout detail; load `product-specs.md`.
- You only need runtime, security, persistence, Browser isolation, or Terminal ownership; load `technical-specs.md`.
- You only need sequencing, accepted work packages, validation needs, or deferred work; load `work-orders.md`.

## Owns

- Experience direction, interface contract, layout, interaction behavior, visual/accessibility rules, and UI handoff routing.

## Does Not Own

- Product thesis, full MVP feature inventory, technical enforcement details, active progress, or work-order status.

## Reference Routing

- `.agents/references/context/creative-specs/interface.md`
  Purpose: Current shell, composer, model selector, session thread, panel, and settings interface details.
  Load when: checking current interface contract details.
  Skip when: the compact creative context already answers the layout rule.

- `.agents/references/context/creative-specs/ui-handoff.md`
  Purpose: Historical r04 handoff summary and current-vs-historical caution.
  Load when: checking inherited wireframe decisions or historical UI caution.
  Skip when: working on final shell rules that supersede r04 details.

- `.agents/references/context/ui-handoff/index.md`
  Purpose: Full r04 UI chunks for implementation or detailed spec conversion.
  Load when: detailed handoff behavior is required.
  Skip when: the task only needs accepted compact UI guidance.

- `.agents/references/context/product-specs/product-experience.md`
  Purpose: Product-experience flow detail for first-run, workspace, session, prework, and composer behavior.
  Load when: UX flow matters more than screen layout.
  Skip when: the task is visual-only or already covered by creative context.

- `.agents/references/context/source-provenance.md`
  Purpose: Source and artifact provenance for UI facts.
  Load when: tracing where UI facts came from.
  Skip when: current context already supplies the accepted fact.

## Summary

C4OS should feel neutral, utilitarian, dense enough for repeated technical work, and closer to a local desktop workspace than a landing page, dashboard, or decorative chat surface. The MVP interface contract is a three-panel desktop shell with project/session navigation on the left, session workbench in the center, and Browser, Files, and Terminal tools on the right.

## Overall Layout

- Left panel: global navigation, project list, project-level chat sessions, settings entry.
- Center panel: empty-state prompt or active session thread.
- Right panel: workspace tools with Browser, Files, and Terminal tabs.
- Left and right panels are resizable and collapsible; center content flexes between them.
- Top center bar includes left-panel collapse, current project/session title, and right-panel collapse.

## Final Implementation Shell Layout

The final implementation shell removes the start screen, right-panel tabs, and
right-panel collapse icon. The persistent shell has one header, a center chat
pane, Settings, and no right panel by default. Plugin icons render in fixed
header slots on the side configured by plugin settings; primary click toggles
the plugin panel, and icon order is controlled by header reorder.

Only one plugin panel can be visible per side. Same-side plugin icons replace
the visible panel. Left and right plugin panels can be visible together, with a
640px minimum center pane. If panel resizing would shrink the center below
640px, close the opposite-side panel first, then clamp expansion. Visible plugin
panels persist per chat session and are restored when switching chats. Settings
opens as a center route, closes all visible plugin panels, and restores the
chat's panel state when leaving Settings.

Plugin settings are rendered in Settings > Plugins from plugin-declared schema
fields: `string`, `text`, `boolean`, `number`, and enum arrays. Shell-reserved
keys include `panel`, `enabled`, and `iconOrder`. Plugin SVG icons must come
from installed bundle assets by relative path and render as sanitized static SVG
in fixed-size slots with fallback on failure.

## Native App Menu

The desktop shell also owns OS-level app menu commands. File menu items are
Open Workspace, Save Workspace, and Save File; Save File is active only when
the file editor can save. Edit menu items are Undo, Redo, Select All, Cut,
Copy, and Paste; they are enabled according to focus in editable contexts such
as the chat prompt, Browser address bar, file editor, and settings input
fields. These commands should not be duplicated as new in-app toolbar controls
unless a later accepted UI change explicitly revises the contract.

## Left Panel

The left panel contains project search, add-project action, project rows, nested chat session rows, and Settings. Project selection and active chat selection are distinct states. Chat session rows do not use chevrons, and active chat highlighting should align with the project row edge.

## Composer And Model Selector

The composer exposes attachment, approval policy, branch, provider, and model context before submission. Empty state asks `What should we build in c4os2?` in the inherited r04 handoff, but sample names and copy should not be hardcoded into production behavior.

Clicking the model chip opens the model popover directly to the active provider model list. Provider rows drill into model lists. Selecting a model updates the composer chip and closes the popover.

## Session Thread

Messages use messenger-style layout. User messages align right. Agent messages align left and are not full width. Agent messages can collapse/expand and support content-level Show more/Show less. Tool calls, run activity, and approval waits are structured event surfaces, not plain message text.

## Right Panel

The right-panel tab order is Browser, Files, Terminal. The active tab owns the full right-panel body. Do not add extra gear, collapse, secondary tab, or panel-management strips inside that tab bar.

Browser has one preview surface and an address bar, with no Browser tab strip in MVP. Files has explorer and open file/code view states. Terminal has main terminal output plus a resizable bottom AI command preview/results panel.

## Settings

Settings navigation order is Providers, Models, Runtimes, Configuration, Plugins, Skills, MCP Servers. Settings has Back to app and no settings search input. Runtimes, Plugins, Skills, and MCP Servers are built-out settings surfaces, not placeholders.

## Visual And Accessibility Rules

- Keep MVP interface neutral and utilitarian.
- Use lucide icons where available.
- Do not introduce dark theme or brand styling unless design phase explicitly approves it.
- Use familiar icon buttons and accessible labels for icon-only controls.
- Preserve focus-visible states, dialog focus management, Escape close, textbox semantics, toggle semantics, and non-overlapping legible text.
- Do not hardcode sample project names, session names, file paths, provider names, model names, plugin names, skill names, MCP server names, branch names, URLs, code snippets, terminal output, badges, statuses, or sample copy.
