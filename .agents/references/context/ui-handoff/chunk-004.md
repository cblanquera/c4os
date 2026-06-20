# UI Handoff - Chunk 004

Parent: `index.md`
Focus: Sections 24-29: visual system, interaction boundaries, accessibility, acceptance notes, reconciliation, and open questions.

### 24. Visual System

The r04 visual system is intentionally restrained:

- White paper background.
- Neutral gray rails and borders.
- Black primary text and buttons.
- Rounded corners no larger than functional UI needs.
- System UI typography.
- Monospace only for code and terminal output.

This is not final brand identity. Use it as a behavior and layout reference.

Implementation rules:

- Keep the app dense enough for repeated technical work.
- Avoid oversized landing-page composition.
- Avoid decorative gradients, orbs, or marketing-style hero treatment.
- Use familiar icon buttons for panel, file, provider, model, terminal,
  settings, plugin, skill, and MCP controls.
- Preserve accessible labels for icon-only buttons.

### 25. Interaction Boundaries

The prototype demonstrates interactions but does not own production behavior.

Implemented in prototype:

- Hash route navigation.
- Show More / Show Less message expansion.
- Panel collapse controls.
- Panel resize handles.
- Terminal bottom-panel resize handle.
- Attachment chip rendering after file picker selection.
- Composer popover selection labels.
- Plugin connection dialog.
- Marketplace dialog.
- Skill detail dialog.
- MCP custom server dialog and transport mode toggle.

Simulated or out of scope:

- Real folder open.
- Real clone flow.
- Real workspace file open.
- Prompt submission.
- Provider/model persistence.
- Browser rendering.
- File editing/saving.
- Terminal execution.
- Approval policy enforcement.
- Sandbox configuration persistence.
- Plugin installation/authentication.
- Skill execution/uninstall.
- MCP validation/persistence.

### 26. Accessibility Expectations

The wireframe expresses these accessibility expectations:

- Icon-only buttons need accessible names.
- Focus-visible states should be present.
- Contenteditable composer needs textbox semantics.
- Popovers and modals should receive focus management in production.
- Dialogs should support Escape close.
- Toggles should expose state in production, even if r04 only shows static
  switch styling.
- Text must remain legible and must not overlap controls.

### 27. Implementation Acceptance Notes

Accepted implementation should preserve:

- Folder-first App Start entry.
- Trusted project shell before prompting.
- Three-panel desktop structure.
- Resizable/collapsible side panels.
- Project selection versus active chat selection distinction.
- Browser, Files, Terminal tab order.
- Files explorer and editor state split.
- Messenger-style user/agent ownership.
- Approval boundary surfaced as structured activity.
- Provider/model settings separation.
- Runtime selection as a distinct settings surface under Models.
- Plugins, Skills, and MCP Servers as distinct settings surfaces.

Do not treat r04 placeholder data as production fixtures. Do not hardcode
example project names, session names, file paths, provider names, model names,
plugin names, skill names, MCP server names, branch names, non-runtime URLs,
code snippets, terminal output, badges, statuses, or sample copy unless a
separate product record explicitly promotes them into requirements,
configuration, or final copy.
Use sample data only to understand layout, hierarchy, and state behavior.

When implementing frontend code from this handoff, the expected instruction is:
preserve the r04 behavior and information architecture, but replace illustrative
wireframe data and placeholder visuals with real application state, production
copy, and the accepted design system. If any of those production sources are
missing, leave a clear implementation note or follow-up rather than hardcoding
the r04 examples.

### 28. Required Reconciliation Before Freeze

Before MVP freeze, accepted UI behavior should be reconciled into:

- `.agents/specs/research/requirements.md`
- `.agents/specs/research/acceptance.md`
- `.agents/specs/research/traceability.md`

Reconcile as product behavior, not as prototype implementation detail.

### 29. Open Questions

These are not settled by r04:

- Final product copy.
- Mobile or responsive behavior beyond narrow desktop overflow.
- Production persistence for provider, model, runtime, plugin, skill, and MCP
  settings.
- Exact backend contract for file explorer and editor state.
- Exact terminal execution and approval policy integration.
- Whether Hooks returns to settings navigation after MVP scope changes.
