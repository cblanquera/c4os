# ADR-010: Filesystem And Shell Access Defaults

Status: Provisional, sandbox model unresolved.

## Context

The research and security specs recommend project-root file access by default, explicit approval for external paths, shell execution defaulting to ask, secret-file deny patterns, filtered inherited environment, and approval for destructive commands.

The review identifies missing requirements: no explicit sandboxing model, no statement on whether agents run as the user's OS account, no network egress policy, and no default-deny details.

## Decision

Use project-root scoped file access and approval-aware shell execution as the default local execution posture.

This is provisional because the sandbox and network model are unresolved.

## Alternatives Considered

 - Full home-directory access.
 - File picker only.
 - Root-scoped access with explicit external-directory approval.

## Alternatives That Should Be Considered

 - Separate OS user for agent commands.
 - Containerized or sandboxed shell execution.
 - Network-disabled command mode.
 - Read-only mode for untrusted projects.
 - No shell execution for non-technical onboarding mode.

## Tradeoffs

Root-scoped access is usable and understandable.

File picker only is safer but too restrictive for agent workflows.

Full home access is convenient but unacceptable for sensitive local data.

Strong sandboxing reduces blast radius but adds platform complexity and may break developer environments.

## Consequences

 - Secret-file denylist and external-directory prompts are mandatory.
 - Shell output logging must include redaction controls.
 - Network egress must be decided before remote tools and plugin scripts are safe.

## Follow-Up Questions

 - Do agent commands run as the user's OS account?
 - What network access is allowed by default?
 - What files are denied by default?
 - How are symlinks handled?
 - What commands are considered destructive?

## ADR Recommendation

Resolve before implementation planning because this is core to local trust.

