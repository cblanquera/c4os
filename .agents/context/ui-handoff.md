# UI Handoff Context

Status: active
Updated: 2026-06-21

## Summary

This context entry captures the r04 C4OS UI handoff for future specs and
implementation planning. The full handoff is split into reference chunks
because it is too detailed to keep directly in `.agents/context/`.

The chunked reference index is
`.agents/references/context/ui-handoff/index.md`.

## Key Facts

- r04 is the current wireframe handoff baseline.
- The latest review artifact is recorded in the context source provenance note.
- The app starts on App Start, not a review menu.
- Prompting requires trusted workspace context.
- C4OS uses a three-panel desktop shell: project/session navigation, center
  workbench, and workspace tools.
- The right workspace tool tab order is Browser, Files, Terminal.
- Current r04 settings navigation is Providers, Models, Runtimes,
  Configuration, Plugins, Skills, and MCP Servers.

## Chunked Context

| Chunk | Focus |
| --- | --- |
| `.agents/references/context/ui-handoff/chunk-001.md` | Sections 1-9: usage, normative versus illustrative content, sources, product frame, artifact contract, global layout, App Start, New Session, and composer behavior. |
| `.agents/references/context/ui-handoff/chunk-002.md` | r04 sections 10-20: provider/model selection, chat session, Browser, Files, Terminal, Runtimes, and Configuration. |
| `.agents/references/context/ui-handoff/chunk-003.md` | r04 sections 21-23: built-out Plugins, Skills, and MCP Servers settings surfaces. |
| `.agents/references/context/ui-handoff/chunk-004.md` | Sections 24-29: visual system, interaction boundaries, accessibility, acceptance notes, reconciliation, and open questions. |

## Project Implications

- Future implementation work should not infer UI behavior from screenshots
  alone; it should read `.agents/context/interface.md` and then the chunked
  handoff context for older r04 detail.
- If this file, `.agents/context/interface.md`, or the chunked r04 handoff
  conflicts, treat the root UI handoff provenance reference as the source to
  reconcile against until a later accepted design review changes it.
- Example data in r04 is illustrative unless promoted into requirements,
  configuration, or final product copy.
- Accepted wireframe behavior has been reconciled into MVP requirements,
  acceptance, and traceability.
- Simulated r04 behavior must not be mistaken for completed product behavior.

## Related Context

- `.agents/context/index.md`
- `.agents/references/context/source-provenance.md`
