# C4OS Agent Routing

This repository uses `.agents/` as its project-local operating workspace for agent-readable rules, accepted product knowledge, planning records, supporting evidence, and reusable workflows.

## Start Here

1. Read [`.agents/AGENTS.md`](.agents/AGENTS.md) for the local operating contract.
2. Read [`.agents/context/index.md`](.agents/context/index.md) before project-specific planning, implementation, review, or documentation work.
3. Load [`.agents/TERMS.md`](.agents/TERMS.md) when C4OS or Agent Workspace terminology affects the task.

## Source Routing

- `.agents/context/` is the Knowledge Base and the only accepted agent source-of-truth folder.
- `.agents/specs/` contains research, planning, evidence, gap resolution, and Frozen implementation contracts; it does not override accepted context.
- `.agents/references/` contains supporting detail linked from Context Files, Spec Files, and workflows.
- `.agents/resources/` preserves Raw Source and other non-agent supporting material.
- `.agents/workflows/` contains reusable maintenance and delivery procedures.
- `proofs/` contains isolated technical experiments and their executable evidence; a passing proof is not a production guarantee.
- `wireframes/` contains human-reviewable product-intent and visual-interaction artifacts; consult the KB for the currently accepted revision and authority boundary.

## Task Routing

- For runtime, security, adapters, sessions, capabilities, approvals, plugins, skills, credentials, updates, or provenance, load [Runtime and Session Architecture](.agents/context/runtime-session-architecture.md).
- For screens, navigation, interaction, responsive behavior, accessibility, native presentation, or wireframes, load [Usability and Interface Contract](.agents/context/usability-and-interface.md) and its task-specific references.
- After a new wireframe revision or material wireframe change, follow the [Wireframe-to-KB Sync Workflow](.agents/workflows/wireframe-kb-sync.md).
- Before running or changing a proof, read [`proofs/README.md`](proofs/README.md) and the proof-local README.

## Operating Boundaries

- Preserve source-of-truth boundaries: accepted reusable truth in `.agents/context/`, unresolved work in `.agents/specs/`, implementation evidence in `proofs/`, and visual review artifacts in `wireframes/`.
- Do not treat wireframes as implementation evidence or isolated proofs as accepted product behavior.
- Do not start implementation from a planning record unless the user asks for implementation and the governing spec is Frozen or the user explicitly authorizes reopening or bypassing that state.
- Keep unrelated worktree changes untouched and scope validation to the files and surfaces affected by the task.
