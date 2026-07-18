# Platform Visual And Theme Feasibility

Spec ID: `00002-platform-visual-theme-feasibility`

Freeze state: Frozen 2026-07-18; macOS contract accepted; Windows/Linux deferred to a separate spec

Classification: Research-only feasibility and evidence package. Do not add implementation tasks, production sequencing, or implementation acceptance records here; create a new spec for implementation work.

This package preserves the feasibility work introduced by the accepted usability KB. It does not reopen the product direction that C4OS follows the host platform and system theme. It tests whether the concrete theme, native integration, startup, fallback, and cross-platform acceptance claims can be implemented honestly in Tauri.

## Files

- [Brief](brief.md) — accepted goal, scope, non-goals, and source boundaries.
- [Status](status.md) — Frozen closeout, Proof disposition, context promotion, and next action.
- [Decisions and gaps](decisions.md) — accepted macOS decisions and explicit Windows/Linux deferral.
- [Research findings](research.md) — bounded primary-source results and proposed target matrix.
- [Proof plan](proofs.md) — executed macOS results and explicit `not run` rows for Windows and Linux.

## Reading Order

Read `brief.md`, then `decisions.md`. Use `research.md` for source provenance and deferred non-macOS findings; use `proofs.md` for the accepted macOS evidence boundary. The usability Context and linked platform contract contain the promoted reusable truth and must seed any new implementation spec. Do not implement from this package directly or extend its evidence to Windows or Linux.
