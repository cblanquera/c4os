# Pi Runtime POC Evidence

Date: 2026-06-20
Runtime: Pi
Packages:
- @earendil-works/pi-agent-core: 0.79.8
- @earendil-works/pi-coding-agent: 0.79.8

## Command

```sh
node proofs/pi-runtime/pi_runtime_poc.mjs
```

## Summary

Pi is a viable in-process runtime candidate for the C4OS adapter contract. The proof created and reopened a session, ran a prompt through the Agent loop with a faux provider, streamed lifecycle/tool events, intercepted a write-file tool call before execution, denied it through app-owned policy, and confirmed the tool body did not execute.

## Adapter Contract Results

| Adapter method | Result | Evidence |
| --- | --- | --- |
| createSession | pass | InMemorySessionRepo creates an app-selectable session ID; JsonlSessionRepo can replace it for durable storage. |
| resumeSession | pass | Session metadata reopens through the repository contract. |
| sendUserMessage | pass | Agent.prompt accepts a user message and runs the agent loop. |
| streamEvents | pass | Agent.subscribe emits lifecycle, message, and tool execution events suitable for app-owned records. |
| requestApproval | pass | beforeToolCall receives validated tool arguments before execution. |
| resolveApproval | pass | Returning block:true from beforeToolCall prevents the proof write tool from executing and emits an error tool result. |
| stopRun | pass | Agent.abort and waitForIdle are available for run cancellation/lifecycle control. |
| listArtifacts | partial | Pi exposes transcript/tool-result state; C4OS must define artifact identity and persistence outside the core runtime. |

## Approval Observation

```json
[
  {
    "toolName": "write_file",
    "args": {
      "path": "blocked.txt",
      "content": "should not be written"
    },
    "decision": "deny",
    "reason": "C4OS app-owned approval policy denied proof write."
  }
]
```

## Gaps

- Pi's public README says it has no built-in permission sandbox for filesystem, process, network, or credential access; C4OS must own those boundaries.
- This proof uses Pi's faux provider to avoid credentials and token spend; provider-backed streaming should be repeated after provider policy is decided.
- Pi is lower-level than OpenCode: session persistence, artifact identity, trusted-root policy, and provider secret storage remain app-owned integration work.

## Recommendation

Use Pi as a viable in-process runtime candidate only if C4OS owns the sandbox, approval policy, provider configuration, and persistence boundaries around it.
