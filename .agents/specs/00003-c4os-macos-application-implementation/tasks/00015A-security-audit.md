# Side Quest 00015A — Security Audit

Status: verified

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

Result: passed — independent Rust/native/security review, hostile and concurrency regression, exact host/native tiers, redaction inspection, and cleanup evidence report P0 = 0 and P1 = 0.

Required evidence: threat-boundary trace; exact commands/results; finding ledger with resolution evidence; denial-before-effect records; race/hostile-input/secret-scan results; residual limitations and explicit external gates.

## Implementation Notes

Completed 2026-07-24 against the final Task 00015 tree and rebuilt bundle. The audit traced renderer intent through native command policy, Action Gateway, facilities, runtime peers, Plugins/Skills, MCP, Browser, Terminal, File/Folder, Git, configuration, persistence, recovery, and diagnostic boundaries. It specifically repaired direct action routing, lifecycle cancellation, Browser authentication classification, remote marketplace fail-closed behavior, and the saturated diagnostic transaction.

## Verification Notes

`npm run qa:security` passed 104 runnable tests with one explicit live-Keychain case ignored. The full serialized Rust matrix passed every product target after the two sandbox-only MCP STDIO tests passed 2/2 on the exact host. MCP HTTP passed 2/2; native OpenCode passed 7/7; streaming passed 3/3; packaged runtimes passed 4/4; private-TLS OpenCode/Pi passed 1/1. Saturated-journal, maximal-replacement, update lifecycle, and database persistence regressions passed. The final process, certificate/temp, isolated-home, structured diagnostic, database, and unified-log review found no secret value or surviving application, runtime, transport, Cargo, or rustc process.

## Agent Acceptance Notes

Independent result: P0 = 0, P1 = 0, P2 = 7, P3 = 0. Retained P2s are bounded fail-closed capacity at 4,096 security rows, generic rather than fully typed journal-payload validation, Terminal containment metadata overstatement, an encrypted and authority-free provider credential orphan crash window, normally unreachable backup restore when Database startup itself fails, direct-action lease loss after a post-effect journal completion fault, and an MCP cancellation window after final check but before Ready commit while the registry remains quiesced. The earlier MCP terminal-commit recovery finding is closed by worker termination plus recoverable projection. No retained item is an in-scope authority bypass; the audit passes.
