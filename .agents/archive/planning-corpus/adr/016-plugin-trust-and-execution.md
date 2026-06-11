# ADR-016: Plugin Trust And Execution

Status: Provisional, MVP restrictions unresolved.

## Context

The plugin and security specs state that plugins are untrusted until installed and enabled, indexing must not execute plugin code, scripts execute only as approved tools, and remote plugin sources should support checksums or signatures in future phases.

The review highlights that malicious plugins can still use dangerous instructions, misleading MCP tool descriptions, broad shell permissions, obfuscated scripts, hostile assets, and dependency install steps.

## Decision

Treat plugins as untrusted packages. Do not execute plugin code during indexing. Separate install from enablement.

Whether plugin scripts, hooks, and remote plugin sources are allowed in MVP is unresolved.

## Alternatives Considered

 - Trust all local plugins.
 - Ban scripts in plugins.
 - Allow scripts with explicit permission.

## Alternatives That Should Be Considered

 - Disable plugin scripts in MVP.
 - Disable plugin hooks in MVP.
 - Static plugin linting and trust scoring.
 - Quarantine mode for new plugins.
 - Mandatory checksums for remote plugin sources.

## Tradeoffs

Allowing scripts and hooks enables powerful workflows and reusable automation.

Disabling scripts and hooks materially lowers MVP risk but reduces plugin capability.

Trust scoring helps users reason about risk but can create false confidence.

## Consequences

 - Plugin permission prompts must show requested filesystem, shell, network, credential, MCP, and write capabilities.
 - Plugin assets and previews may need sandboxed rendering.
 - Marketplace support is unsafe without source integrity and trust posture.

## Follow-Up Questions

 - Should plugin scripts be disabled in MVP?
 - Should plugin hooks be disabled in MVP?
 - What warnings are shown before enablement?
 - What plugin contents are linted?
 - Are dependency install steps ever run automatically?

## ADR Recommendation

Resolve before any plugin or marketplace implementation planning.

