# ADR-001: Product Scope And MVP Audience

Status: Unresolved.

## Context

The requirements describe the product as a general-purpose desktop AI workspace for coding, writing, research, analysis, operations, documentation, and other agent-assisted workflows. The review challenges this because most concrete MVP requirements are coding-oriented: OpenCode Runtime, Git integration, worktrees, shell execution, file patches, AGENTS.md, diffs, and coding-agent conventions.

This creates a product and architecture ambiguity. A coding-first workspace can make strong use of OpenCode, Git, worktrees, AGENTS.md, and shell tools. A true general-purpose workspace needs first-class primitives for documents, spreadsheets, research citations, knowledge ingestion, operations workflows, and non-code artifact lifecycles.

## Decision

No final decision has been made.

The documents currently imply a broad general-purpose ambition with a coding-heavy MVP implementation path. This should be treated as unresolved, not final.

## Alternatives Considered

 - General-purpose workspace from day one.
 - Coding workspace first, generalized through plugins later.
 - Operations/research/document workspace first.

## Alternatives That Should Be Considered

 - Technical power-user MVP: coding plus local knowledge work, explicit about its developer bias.
 - Two-lane onboarding: "developer project" and "document/research folder".
 - Plugin-first general workflows: core remains projects/sessions/tools/artifacts, while writing/research/operations are delivered as bundled plugins.

## Tradeoffs

General-purpose from day one broadens appeal but makes requirements vague and increases UX complexity.

Coding-first aligns with OpenCode, Git, worktrees, and shell tools, but risks contradicting the stated product goal and limiting differentiation if OpenCode or Codex expands its GUI.

Document/research-first would better serve non-developers, but weakens the rationale for OpenCode Runtime as the central dependency.

## Consequences

Until the audience and first-class workflows are decided:

 - UX complexity cannot be judged.
 - Artifact priorities remain unclear.
 - Plugin strategy may overcompensate for missing core product definition.
 - Security defaults may be wrong for non-technical users.
 - The roadmap may validate the wrong runtime.

## Follow-Up Questions

 - Who is the first user persona?
 - Who is the buyer, if different from the user?
 - Is the MVP a coding workspace, a general local workspace, or a workflow shell?
 - What are the first five workflows the product must make excellent?
 - Which non-coding workflows are first-class in MVP?
 - Which workflows are intentionally excluded?

## ADR Recommendation

Keep this ADR open and resolve it before architecture freeze.

