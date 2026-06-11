# item-036: TASK-035 Build Explicit Project-Local Skill Discovery And Invocation

## Status

verified

## Objective

Implement the accepted `explicit_discovery_and_invocation_only` Agent Skills
surface for project-local `SKILL.md` files.

## Dependencies

- item-035

## Scope

- Discover project-local `SKILL.md` files only inside the selected project
  root.
- Display bounded skill metadata: name, description, source path, and
  unsupported-capability warnings.
- Require explicit user selection before skill text can affect the current
  session.
- Keep project-root and secret-deny rules authoritative for discovery and
  invocation.
- Keep scripts, install steps, references, assets, global catalogs,
  plugin-provided skills, auto-invocation, version conflict resolution,
  marketplace workflows, export, import, and round-trip compatibility out of
  scope.
- Skills must not affect permissions, approvals, file access, shell access, Git
  access, model routing, or automatic app-owned model context.

## Inputs

- `.agents/archive/planning-corpus/acceptance/skills.md`
- `.agents/archive/planning-corpus/spikes/SPIKE-006-standards-conformance-matrix.md`
- `.agents/archive/planning-corpus/specs/security-specification.md`
- `.agents/progress/items/item-035.md`

## Deliverables

- Backend discovery for project-local `SKILL.md` files.
- Skill metadata projection and support warnings.
- Explicit invocation path for selected skill text.
- UI/status surface for accepted and unavailable skill behavior.
- Automated tests for discovery, invocation, containment, secret-deny blocking,
  and excluded behavior.

## Implementation

- Added the Rust `skills` module for project-local `SKILL.md` discovery,
  metadata parsing, path containment, secret-deny enforcement, unsupported
  capability warnings, and explicit invocation.
- Added Tauri commands `list_project_skills` and `invoke_project_skill`.
- Added app status capability metadata for the accepted explicit-only skills
  tier.
- Added JS smoke command support and a compact UI status label.
- Explicit invocation appends selected skill text to the latest session only
  through the user-invoked command when a latest session exists.

## Verification

- Rust tests for discovery, containment, secret-deny blocking, and invocation
  policy.
- JS tests for capability/status projection and UI unavailable-behavior labels.
- `cargo test --manifest-path src-tauri/Cargo.toml skills` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml invoking_project_skill_appends_explicit_session_context`
  passed.
- `npm test` passed.
- `npm run build` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed.
- `npm run tauri -- build` passed.

## Handoff

Sprint 13 is complete. Continue to the next documented V1 roadmap item: local
stdio MCP server support requires a scope decision before implementation.
