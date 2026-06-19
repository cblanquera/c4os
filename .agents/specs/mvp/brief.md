# MVP Restart Brief

Status: ready-for-review
Created: 2026-06-18
Updated: 2026-06-18
Source:
- `plans/product-brief.md`
- `plans/product-interface.md`
- `plans/pegs/*.png`

## Summary

This spec imports the C4OS greenfield restart brief into compact records. The proposed product is a local-first desktop AI workspace where users trust project folders, run persistent agent sessions, manage providers and approvals, inspect artifacts, edit files, and use safe browser and terminal surfaces.

## Current Scope Status

The plan is broad and ambitious. It contains enough product direction to review and narrow an MVP, but several implementation-sensitive areas still need validation or explicit sequencing before freeze:

- runtime adapter choice and lifecycle control
- browser isolation
- terminal lifecycle and renderer transport
- concurrent session isolation
- extension inventory versus execution timing
- file editor mutation boundaries

## Recommended Next Step

Run `workflows/review.md` against this MVP import, then route blocker findings to `workflows/validation.md` or `workflows/poc.md` before freeze.
