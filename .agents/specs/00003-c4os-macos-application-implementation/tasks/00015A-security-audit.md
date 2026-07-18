# Side Quest 00015A — Security Audit

Status: open

Coverage: independent cross-cutting audit of all 50 IDs, with emphasis on UX-010, UX-012, UX-015, CHAT-009, CHAT-010, ART-002 through ART-005, SET-004, SET-007, SET-009, SET-011.

## Summary

Independently audit the integrated build for authority boundaries, denial before side effects, least privilege, secret isolation, hostile input, races, provenance, audit durability, revocation, and recovery.

## Implementation Steps

1. Trace renderer intent through core services, PolicyService, ActionGateway, executors, facilities, runtimes, extensions, MCP, Browser, Terminal, File/Folder, and Git.
2. Test token binding, TTL, workspace/session/environment/process/risk binding, one-shot consumption, concurrency, revocation, and persisted pre-effect audit.
3. Test path traversal, symlinks, time-of-check races, hostile archives/packages/MCP/page content, process injection, network trust, secret channels, and redaction.
4. Inspect direct-operation and per-runtime request configuration for authority bypasses.
5. Record findings, require fixes, and rerun affected plus broad security suites before passing.

## Verification Process

- Independent source/diff review plus deterministic hostile and concurrency tests.
- Native execution inspection proving denial and audit precede effects.
- Secret scans of renderer state, logs, databases, diagnostics, evidence, environments, and process arguments.

Acceptance criteria: none — implementation acceptance is delegated to the coordinator’s Agent Acceptance process for Spec 00003.

## Agent Acceptance

Result: failed — an integrated security target does not yet exist.

Required evidence: threat-boundary trace; exact commands/results; finding ledger with resolution evidence; denial-before-effect records; race/hostile-input/secret-scan results; residual limitations and explicit external gates.

## Implementation Notes

Not started. Proof weaknesses identified during inventory are required negative scenarios.

## Verification Notes

Not run.

## Agent Acceptance Notes

Any unresolved in-scope high- or medium-impact authority bypass fails the audit.
