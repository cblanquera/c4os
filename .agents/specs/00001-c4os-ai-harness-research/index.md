# C4OS AI Harness Research

Spec ID: `00001-c4os-ai-harness-research`

Freeze state: Frozen 2026-07-18 after P-018 acceptance

Classification: Research-only evidence package. Do not add implementation tasks, production sequencing, or implementation acceptance records here; create a new spec for implementation work.

This package records completed research for the Tauri-based C4OS desktop AI harness represented by `wireframes/r012-cleanup/`. It was reopened to research model capabilities and their effect on chat-session experience, then refrozen after P-018 acceptance. The wireframes remain current product intent and a visual contract; research Freeze does not retire them.

Accepted reusable architecture has been promoted to [Runtime and session architecture](../../context/runtime-session-architecture.md). That Context File takes precedence over historical evidence wording in this frozen package.

## Files

- [Brief](brief.md) — goal, scope, source boundaries, and non-goals.
- [Status](status.md) — current phase, completed work, and freeze blockers.
- [Decisions](decisions.md) — accepted conclusions plus resolved and explicitly deferred release gaps.
- [Research](research.md) — product reading and external technical research.
- [Codex config](codex-config.md) — `config.toml` layers, schema surfaces, managed policy, and C4OS import implications.
- [OCAdapter](opencode-adapter.md) — OpenCode-native capabilities, C4OS mappings, ownership, and proof gaps.
- [PIAdapter](pi-adapter.md) — Pi-native capabilities, SDK/RPC choices, C4OS mappings, ownership, and proof gaps.
- [Runtime capability matrix](runtime-capability-matrix.md) — side-by-side capability map and the derived C4OS adapter baseline.
- [Model capabilities](model-capabilities.md) — provider/runtime capability research, normalized C4OS descriptor, effective resolution, and chat-session effects.
- [Open-source runtime and capability patterns](open-source-runtime-capability-patterns.md) — ACP, Goose, OpenHands, Cline, and related patterns applied to the peer-adapter design.
- [Approval policy model](approval-policy-model.md) — accepted simplified user model, internal scenario corpus, policy resolution, and deferred wireframe revision.
- [Proofs](proofs.md) — applicability of existing proof artifacts and results from the approved seven-part Proof Loop.
- [User journeys](journeys.md) — primary, permission, recovery, compatibility, extension, and environment flows with scope and implementability labels.
- [Acceptance archive](acceptance/index.md) — historical user acceptance records for P-001 through P-018 and the wireframe clarification at Freeze.

## Reading order

Read `brief.md`, then `research.md`. For runtime research, read both adapter files before the runtime capability matrix; read `model-capabilities.md` and `open-source-runtime-capability-patterns.md` before scoping model selection, dynamic session controls, attachments, reasoning, tools, or response rendering in a new implementation spec. Use `journeys.md` for end-to-end research, `decisions.md` for accepted and deferred choices, and `proofs.md` for provenance and implementation-risk inputs. Accepted reusable truth comes from Context, not directly from this package.
