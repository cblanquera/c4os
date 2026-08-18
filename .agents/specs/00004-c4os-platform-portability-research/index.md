# C4OS Cross-Platform Architecture Research

Spec ID: `00004-c4os-platform-portability-research`

Freeze state: Proposed 2026-07-27; materially revised 2026-07-28; research-only; not Frozen

Classification: Current-build audit, primary-source architecture comparison, and controlled-restart planning package. It authorizes no production implementation and makes no Linux or Windows support claim.

## Purpose

Compare the current Tauri/Rust/macOS-first implementation with an Electron shell, a Node application-service layer, and peer Pi and OpenCode SDK integrations. Identify what a restart would improve, what remains OS-specific, what C4OS behavior should survive, and which Proofs must pass before a new implementation contract is accepted.

## Files

- [Brief](brief.md) — user goal, scope, non-goals, and authority boundary.
- [Status](status.md) — lifecycle, completed research, open work, and Freeze blockers.
- [Architecture comparison](architecture-comparison.md) — current-vs-proposed structure, recommendation, tradeoffs, and restart boundary.
- [Decisions and gaps](decisions.md) — preserved constraints, proposed decisions, and unresolved choices.
- [Current implementation inventory](inventory.md) — source-backed macOS/Unix coupling and reuse classification.
- [Future spec projections](future-specs.md) — Proof-first restart, implementation, native acceptance, and distribution boundaries.
- [Research](research.md) — repository research synthesis and current primary-source findings.
- [Proofs](proofs.md) — Electron/Node/SDK and exact-target Proof matrix; every new-architecture row remains `not run`.

## Reading Order

Read `brief.md`, `architecture-comparison.md`, and `inventory.md` first. Use `decisions.md` for choices that require acceptance. Use `research.md` to audit the evidence and `proofs.md` before authorizing a restart implementation spec.

## Authority Boundary

Accepted reusable truth remains in [C4OS Context](../../context/index.md). Context currently defines the C4OS Core as Tauri/Rust, and Frozen [Spec 00003](../00003-c4os-macos-application-implementation/index.md) remains the governing implementation contract. This Proposed research package exposes a possible replacement architecture; it does not silently override either source.

The source audit uses commit `336dd5e8f53fed85f02adf692afb0e9b53cc59b0` plus a read-only observation of the 2026-07-28 working tree. Uncommitted implementation work is evidence under review, not accepted behavior.

## Implementation Gate

Closed. Do not replace production code, delete the current implementation, edit Frozen Specs, or promote the proposed architecture to Context from this package. A restart requires user acceptance, resolved critical Gaps, successful architecture Proofs on named targets, Context changes, and a new Frozen implementation spec.
