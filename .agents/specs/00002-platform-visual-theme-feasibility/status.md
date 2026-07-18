# Status

## Lifecycle

- Phase: Setup and research planning
- Freeze state: Draft
- Last updated: 2026-07-18
- Context promotion: Skipped; no feasibility result is accepted yet.

## Work Items

| Work item | State | Next action |
| --- | --- | --- |
| Preserve accepted intent and scope | Complete | Keep intent separate from feasibility evidence. |
| Scan KB intersections | Complete for setup | Recheck intersections if research contradicts a current contract. |
| Record source reconnaissance | Complete for setup | Treat it as triage, not completed research. |
| Research Tauri theme/platform APIs | Planned | Verify exact support and limitations by target. |
| Research webview and OS theme inputs | Planned | Map native, CSS, portal, and fallback sources. |
| Research native menu/window behavior | Planned | Define safe per-platform chrome and Settings entry choices. |
| Define supported target matrix | Pending | Decide which OS versions and Linux environments require proof. |
| Run technical Proofs | Proposed; not started | Review the proof queue before creating prototypes. |
| Run visual/platform QA | Proposed; not started | Execute only on named available targets; gate missing targets honestly. |
| Resolve Gaps and Freeze | Blocked by research and Proofs | Accept supported behavior and fallbacks, then promote reusable truth. |

## Freeze Blockers

- GAP-001 through GAP-006 in `decisions.md` remain open.
- Research topics R-001 through R-006 are not complete.
- The proposed target-specific Proofs have not been reviewed or executed.
- The required macOS/Windows/Linux target matrix is not yet accepted.

## User-Journey Loop

Not required for setup. Existing usability and reconstruction contracts already enumerate affected surfaces and theme-change acceptance. Revisit only if research introduces a visible failure, recovery, or configuration journey.

## Next Action

Review the recorded gaps, research topics, and Proof queue. Then begin the bounded primary-source Research Loop without changing Context or wireframes.
