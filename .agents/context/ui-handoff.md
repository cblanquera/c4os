# UI Handoff Context

Status: active
Updated: 2026-06-20

## Summary

This context entry captures the r04 C4OS UI handoff for future specs and
implementation planning. The full handoff is split into reference chunks
because it is too detailed to keep directly in `.agents/context/`.

The chunked reference index is
`.agents/references/context/ui-handoff/index.md`.

## Key Facts

- r04 is the current wireframe handoff baseline.
- The latest review artifact is `wireframes/r04-single-page-app/index.html`.
- The app starts on App Start, not a review menu.
- Prompting requires trusted workspace context.
- C4OS uses a three-panel desktop shell: project/session navigation, center
  workbench, and workspace tools.
- The right workspace tool tab order is Browser, Files, Terminal.
- Settings navigation is Providers, Models, Runtimes, Configuration, Plugins,
  Skills, MCP Servers.

## Chunked Context

| Chunk | Focus |
| --- | --- |
| `.agents/references/context/ui-handoff/chunk-001.md` | Sections 1-9: usage, normative versus illustrative content, sources, product frame, artifact contract, global layout, App Start, New Session, and composer behavior. |
| `.agents/references/context/ui-handoff/chunk-002.md` | Sections 10-20: provider/model selection, chat session, Browser, Files, Terminal, Runtimes, and Configuration. |
| `.agents/references/context/ui-handoff/chunk-003.md` | Sections 21-23: Plugins, Skills, and MCP Servers settings surfaces. |
| `.agents/references/context/ui-handoff/chunk-004.md` | Sections 24-29: visual system, interaction boundaries, accessibility, acceptance notes, reconciliation, and open questions. |

## Project Implications

- Future implementation work should not infer UI behavior from screenshots
  alone; it should read the chunked handoff context first.
- Example data in r04 is illustrative unless promoted into requirements,
  configuration, or final product copy.
- Accepted wireframe behavior has been reconciled into MVP requirements,
  acceptance, and traceability.
- Simulated r04 behavior must not be mistaken for completed product behavior.

## Related Context

- `.agents/context/index.md`
- `wireframes/screens.md`
- `.agents/specs/research/requirements.md`
- `.agents/specs/research/acceptance.md`
