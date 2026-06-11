# item-035: TASK-034 Resolve V1 Agent Skills Discovery And Invocation Scope

## Status

verified

## Objective

Resolve the Agent Skills conformance and trust model before exposing any skill
catalog or invocation behavior in the app.

## Dependencies

- item-034

## Scope Decision

Current V1 supports `explicit_discovery_and_invocation_only`, accepted by the
user on 2026-06-12.

- Project-local `SKILL.md` files may be inventoried and displayed.
- Displayed metadata is limited to skill name, description, source path, and
  unsupported-capability warnings.
- A skill may affect a session only after explicit user selection.
- Skill text may enter app-owned model context only as an explicit
  user-directed context addition or explicit project-root file read.
- Skills are never auto-invoked.
- Skill scripts are not executed.
- Skill references and assets are not loaded as trusted content.
- Skills have no permission effect and cannot alter file, shell, Git, model,
  or approval policy.
- Global skill catalogs, plugin-provided skills, version conflict resolution,
  marketplace distribution, export, import, and round-trip compatibility remain
  unsupported.

## Evidence

- `.agents/archive/planning-corpus/roadmap/implementation-roadmap.md` lists
  explicit skill discovery and invocation after nested `AGENTS.md` resolution.
- `.agents/archive/planning-corpus/decisions/deferred-decisions.md` keeps Agent
  Skills deferred until conformance and trust are defined.
- `.agents/archive/planning-corpus/acceptance/skills.md` excludes skills from
  MVP and now records proposed V1 acceptance criteria.
- `.agents/archive/planning-corpus/spikes/SPIKE-006-standards-conformance-matrix.md`
  now records the proposed `SKILL.md` tier.

## Deliverables

- Product decision for the V1 skill conformance tier.
- Standards conformance tier for `SKILL.md`.
- Acceptance criteria for explicit skill discovery and invocation.
- Out-of-scope boundaries for auto-invocation, scripts, references, assets,
  permissions, and model-context effects.
- Handoff for the implementation item that follows acceptance.

## Verification

- Planning review against roadmap, deferred decisions, standards matrix,
  security specification, and skills acceptance criteria.
- User accepted the proposed scope on 2026-06-12.

## Decision

The proposed scope is accepted.

## Handoff

Continue with item-036 / TASK-035 to implement explicit project-local skill
discovery and invocation.
