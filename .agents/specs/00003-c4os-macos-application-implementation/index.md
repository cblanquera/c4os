# C4OS macOS Application Implementation

Spec ID: `00003-c4os-macos-application-implementation`

Freeze state: Frozen 2026-07-18; accepted for implementation planning

Classification: Production implementation contract. This is the only current spec intended to own implementation requirements derived from the accepted runtime/session, usability, and macOS platform contracts.

## Purpose

Define how the accepted C4OS product contracts become a secure, maintainable Tauri application for the currently evidenced macOS target. Frozen Specs 00001 and 00002 remain research-only evidence packages and must not receive implementation tasks or production sequencing.

## Files

- [Brief](brief.md) — goal, scope, non-goals, authority, and source boundaries.
- [Status](status.md) — lifecycle, Freeze readiness, open work, and implementation-planning gate.
- [Decisions and gaps](decisions.md) — inherited constraints, proposed implementation decisions, and unresolved choices.
- [Questions](questions.md) — lossless implementation-clarification ledger with accepted answers and delegated technical follow-ups; no user-owned question is currently queued.
- [Implementation contract](implementation-contract.md) — required production boundaries, records, components, integrations, and verification gates.
- [Implementation selections](implementation-selections.md) — concrete renderer, persistence, Browser, extension, archive, configuration, and storage choices.
- [Feature coverage](feature-coverage.md) — normative mapping from accepted usability features to implementation ownership, verification, and visual acceptance.
- [Research](research.md) — completed bounded implementation-selection evidence, rejected candidates, and residual risk.
- [Proofs](proofs.md) — disposition of inherited feasibility evidence and the passing pre-Freeze macOS Browser Proof.

## Reading Order

Read `brief.md`, then `decisions.md`, `questions.md`, `implementation-contract.md`, and `feature-coverage.md`. Use `research.md` and `proofs.md` only when resolving a named Gap or checking an evidence boundary. After this spec is Frozen and the user requests implementation planning, create sequencing and task records under `tasks/` using the Spec Task Implementation Workflow.

## Authority Boundary

Accepted reusable truth comes from [C4OS Context](../../context/index.md), especially [Runtime and session architecture](../../context/runtime-session-architecture.md) and [Usability and interface contract](../../context/usability-and-interface.md). This spec may narrow that truth for an implementation slice but may not override it.

The following Frozen packages are provenance only:

- [C4OS AI harness research](../00001-c4os-ai-harness-research/index.md) — adapter, policy, capability, extension, environment, recovery, and Proof evidence.
- [Platform visual and theme feasibility](../00002-platform-visual-theme-feasibility/index.md) — macOS theme, menu, window, and representative visual evidence.

## Implementation Gate

All Gaps, bounded research, required pre-Freeze Proofs, context promotion, and lifecycle acceptance are closed. Use the Spec Task Implementation Workflow to create `tasks/sprint.md` before production implementation. The task plan must cover every normative Feature Coverage ID and preserve the complete-contract delivery boundary.
