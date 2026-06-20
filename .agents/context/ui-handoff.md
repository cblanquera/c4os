# UI Handoff Context

Status: active
Updated: 2026-06-20

## Summary

The implementation-facing UI handoff lives at
`wireframes/ui-handoff-spec.md`. It translates the current wireframes into
layout, behavior, state, and implementation guidance without duplicating the
full wireframe artifact inside `.agents/context/`.

## Durable Product Facts

- The latest review target is `wireframes/r04-single-page-app/index.html`.
- The app starts on App Start, not on a wireframe menu.
- C4OS uses a three-panel desktop shell: left project/session navigation,
  center workbench, and right workspace tools.
- The right workspace tool tab order is Browser, Files, Terminal.
- Prompting requires trusted workspace context; folder-backed setup comes
  before active chat use.
- Chat uses messenger ownership: user messages right, agent messages left.
- Settings navigation in the latest wireframe is Providers, Models,
  Configuration, Plugins, Skills, MCP Servers.
- Plugin, skill, and MCP management screens are functional review surfaces, but
  persistence and external integrations are simulated in the wireframe.

## Context Boundary

Keep the detailed layout and interaction contract in
`wireframes/ui-handoff-spec.md`. Keep `.agents/context/` limited to reusable
product facts and pointers so future specs can find the handoff without
duplicating it.

## Next Reconciliation

Before MVP freeze, accepted handoff behavior should be reconciled into:

- `.agents/specs/mvp/requirements.md`
- `.agents/specs/mvp/acceptance.md`
- `.agents/specs/mvp/traceability.md`
