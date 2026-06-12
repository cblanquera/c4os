# C4OS General Workspace Roadmap Brief

Spec ID: c4os-general-workspace-roadmap
Type: planned-brownfield
Status: proposed

## Objective

Plan the post-V1 path from the verified coding-first C4OS MVP into the broader
general-purpose desktop AI workspace described by the original project prompt.

## Audience

- Product and architecture agents deciding what scope to accept next.
- Implementation agents that need phase boundaries before creating progress
  items.
- Project stakeholders evaluating whether deferred capabilities should now be
  promoted.

## Source Material

- Original AI Workspace Specification Project prompt provided by the user.
- `CONTEXT.md`
- `.agents/specs/c4os-mvp/`
- `.agents/progress/manifest.md`
- `.agents/archive/planning-corpus/`

## Scope

- Multi-project and general workspace workflows.
- Import/export and portability.
- Browser, web content, and richer artifact viewing.
- Provider expansion.
- Worktrees and concurrent agents.
- Plugin and marketplace systems.
- Durable memory and knowledge workflows.
- Advanced policy and audit surfaces.

## Non-Goals

- Do not start implementation from this spec.
- Do not promote deferred V1 features without accepted scope and acceptance
  criteria.
- Do not claim broad compatibility with AGENTS.md, Agent Skills, MCP, Codex
  plugins, OpenCode config, import/export, browser workflows, or memory until
  each claim has evidence.
- Do not weaken the existing backend-owned safety boundary.

## Current State

The MVP and current V1 follow-on items through `item-048` are verified in
`.agents/progress/manifest.md`. V1 explicitly deferred browser/web viewing,
plugin/marketplace workflows, direct/local providers, durable memory, broad
compatibility claims, and worktree lifecycle.

## Intended State

C4OS becomes a local-first desktop AI workspace that supports coding, writing,
research, analysis, operations, documentation, and other agent-assisted
workflows through explicit, inspectable, standards-aligned capabilities.

## Definition Of Ready

- Roadmap phase records exist with dependencies and acceptance criteria.
- Current MVP/V1 decisions remain traceable.
- High-risk phases have validation tasks before implementation tasks.
- Open questions are recorded instead of silently assumed.
- Readiness review resolves or explicitly accepts blocker/high findings before
  active progress items are created.
