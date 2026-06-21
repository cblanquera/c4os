# UI Handoff Context Chunks

Status: active
Created: 2026-06-20
Source Note: Derived from the r04 wireframe review and accepted handoff decisions.

## Summary

These chunks preserve the detailed r04 UI handoff inside `.agents/references/`
so `.agents/context/creative-specs.md` can stay compact while still giving future
agents enough detail to implement or spec from the wireframes without opening
the prototype.

## Chunks

| Chunk | Focus |
| --- | --- |
| `chunk-001.md` | Sections 1-9: usage, normative versus illustrative content, sources, product frame, artifact contract, global layout, App Start, New Session, and composer behavior. |
| `chunk-002.md` | Sections 10-20: provider/model selection, chat session, Browser, Files, Terminal, Runtimes, and Configuration. |
| `chunk-003.md` | Sections 21-23: Plugins, Skills, and MCP Servers settings surfaces. |
| `chunk-004.md` | Sections 24-29: visual system, interaction boundaries, accessibility, acceptance notes, reconciliation, and open questions. |

## Lookup Notes

- Start with `.agents/context/creative-specs.md`.
- Read all chunks before converting r04 UI behavior into implementation tasks.
- Treat r04 example data as illustrative unless another product record promotes it.
- Do not treat simulated prototype behavior as completed implementation.
