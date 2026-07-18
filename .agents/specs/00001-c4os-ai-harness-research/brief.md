# Brief

## User goal

Create an initial research spec for this project: an AI harness desktop built with Tauri. Determine the intended functionality from `wireframes/r012-cleanup/`, note existing proofs that may apply, and research:

- Tauri;
- the Agent Skills `SKILL.md` specification;
- OpenAI Codex plugin specifications;
- OpenAI Codex plugin marketplace specifications;
- Jan.ai's current Tauri architecture and any demonstrated OpenCode integration;
- Atomic Chat's current Tauri architecture and OpenCode integration; and
- other relevant projects using Tauri, OpenCode, Pi, or a useful combination of them.

## In scope

- A source-grounded reading of the r012 product surface.
- Current public documentation and repository evidence, accessed 2026-07-17.
- The current Codex `config.toml` schema, configuration layers, profiles, and managed-policy boundary, reviewed 2026-07-18.
- Runtime topology, process ownership, transport, policy, and extension packaging implications.
- A reusable-evidence inventory of the existing `proofs/` directory.
- Explicit gaps that must be resolved before the spec can freeze.

## Out of scope

- Implementing the desktop application or changing the wireframe.
- Treating simulated wireframe behavior as implemented behavior.
- Implementing the production application; the approved research Proof Loop is now included as bounded evidence.
- Claiming full Codex plugin or marketplace compatibility.
- Submitting a plugin to the public Codex directory.
- Treating a proof result as production or cross-platform certification.
- Promoting findings into `.agents/context/` before they are accepted as reusable project truth.

## Source boundaries

1. Current repository files define local product intent and proof evidence.
2. Current official specifications and current upstream source define external behavior.
3. Repository READMEs are supporting evidence; current code takes priority when they disagree.
4. Proof artifacts are dated feasibility evidence, not production specifications.
5. A missing integration in a public repository tree is recorded as “not found,” not as proof that it cannot exist elsewhere.

## Accepted context

- [Runtime and session architecture](../../context/runtime-session-architecture.md) is the reusable source of truth for accepted P-001 through P-018 where applicable. This frozen spec retains their evidence and decision history.

## Deliverable standard

This deliverable is research-only. It supplies evidence and accepted inputs to Context; it is not an implementation contract, task plan, or production acceptance ledger. Those details belong in a new implementation spec.

The research is complete enough for an initial review when it:

- distinguishes embedded runtimes from external CLI launchers;
- separates Tauri host responsibilities from AI-runtime and policy responsibilities;
- records the Codex plugin and marketplace surfaces without overstating compatibility;
- distinguishes portable standards research from Codex-host-specific behavior without implying a current Codex importer;
- identifies directly applicable and cautionary proofs; and
- leaves every material unresolved decision visible in `decisions.md`.
