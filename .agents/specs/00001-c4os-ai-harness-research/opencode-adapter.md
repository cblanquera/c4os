# OCAdapter

State: Proposed design; tested boundary passed against OpenCode 1.18.3
Research refreshed: 2026-07-18

## Purpose

`OCAdapter` exposes OpenCode to C4OS without making OpenCode's API, storage, permissions, or configuration the C4OS product contract. OpenCode and Pi remain peer user-selectable runtimes.

## Proposed topology

```text
C4OS UI
  -> C4OS application and policy layer
    -> OCAdapter
      -> authenticated loopback OpenCode server
```

C4OS supervises a pinned OpenCode process running `opencode serve`. The server binds to loopback, uses a random per-launch password, and exposes an OpenAPI-generated client plus server-sent events. C4OS owns process installation, startup, health checks, compatibility checks, restart limits, shutdown, and isolated state directories.

Connecting to a user-managed or remote OpenCode server is a separate future mode. It must not silently inherit the trust granted to the C4OS-managed local server.

## Native OpenCode capabilities

| Area | Current upstream surface | Adapter use |
| --- | --- | --- |
| Health and version | `GET /global/health` | Readiness, compatibility, and diagnostics. |
| Events | `/event` and `/global/event` SSE streams | Streaming responses, tool activity, permissions, status, and reconnect detection. |
| Project context | Project, path, VCS, file, find, symbol, and LSP endpoints | Bind native requests to the selected C4OS workspace; expose optional native search/VCS capabilities. |
| Sessions | List, create, status, get, update title, delete, children, fork, abort, share, diff, summarize, revert, and unrevert | Map OpenCode session IDs to C4OS runtime bindings; advertise optional operations through capability flags. |
| Messages | List/get messages, synchronous message, asynchronous prompt, command, and shell endpoints | Send normalized C4OS input and translate native message parts into C4OS events and artifacts. |
| Permissions | Session permission-response endpoint with response and remember behavior | Surface native permission requests, but route the actual user decision through C4OS policy and audit. |
| Providers | Config/provider inventory, provider auth methods, OAuth, and credential endpoint | Populate runtime-native model/provider choices without making OpenCode config the C4OS source of truth. |
| Extensibility | Commands, agents, MCP, tools, and OpenCode configuration | Expose as runtime-specific capabilities; do not merge them automatically with C4OS plugins or tools. |
| Instance lifecycle | Instance dispose | Reload or dispose native state after controlled configuration changes. |

## Proposed C4OS operations

| OCAdapter operation | Native mapping | Required normalization |
| --- | --- | --- |
| `probe()` | Executable version plus global health | Installed, compatible, unavailable, degraded, or update-required. |
| `start(binding)` | Spawn server and wait for health | Loopback address, random credentials, workspace binding, isolated state, sanitized logs. |
| `stop()` | Dispose then terminate supervised process | Graceful timeout, forced termination, descendant cleanup, final status. |
| `describeCapabilities()` | Versioned endpoint/SDK inventory | Required and optional capability flags. |
| `createSession()` | `POST /session` | C4OS ID to OpenCode ID mapping and persisted runtime binding. |
| `resumeSession()` | Session get, messages, status, then SSE | Rehydrate normalized messages and reconnect without duplicating events. |
| `send()` | Async prompt or message endpoint | C4OS attachments, model selection, tool exposure, and correlation identifiers. |
| `cancel()` | Session abort | Idempotent terminal state and late-event handling. |
| `fork()` | Session fork | Preserve C4OS ancestry and native parent/message provenance. |
| `compact()` | Session summarize | Report native compaction as a capability; retain C4OS-visible summary provenance. |
| `respondToApproval()` | Permission response endpoint | C4OS decision, scope, audit record, native response, and remember-policy translation. |
| `listModels()` | Provider/config endpoints | Stable C4OS provider/model IDs mapped to OpenCode IDs. |
| `disposeWorkspace()` | Instance dispose | Controlled reload with active-session warning and recovery state. |

## Event translation

`OCAdapter` must translate SSE events into C4OS-owned envelopes containing:

- C4OS runtime, workspace, session, turn, message, tool-call, and artifact identifiers where applicable;
- the original OpenCode event type and native identifiers for diagnostics;
- monotonic adapter sequence and received timestamp;
- normalized lifecycle, text/thinking delta, tool request/progress/result, permission request, usage, completion, cancellation, and error categories; and
- replay/reconnect metadata so duplicated native events do not duplicate UI state.

Unknown native events remain observable diagnostics. They must not crash the stream or be silently reclassified.

## C4OS-owned boundaries

- Runtime selection and per-session runtime binding.
- Trusted roots, sandbox rules, tool exposure, approval records, and remembered decision scopes.
- Stable workspace, session, message, tool-call, and artifact identity.
- Credential storage and provider-to-runtime credential delivery.
- Plugin, Codex bundle, skill, and MCP policy outside OpenCode's own extension system.
- Process supervision, version pinning, updates, state isolation, diagnostics, and redaction.
- Normalized persistence and recovery independent of OpenCode's native database or config.

OpenCode must not execute a privileged native tool merely because its own configuration permits it. The conformance proof must show that every C4OS-governed tool call is intercepted or constrained before side effects.

Before execution, `OCAdapter` must emit the action intent defined in `approval-policy-model.md`. OpenCode's tool-pattern permissions remain defense-in-depth. C4OS evaluates category rules and concrete exceptions, then sends a single-request native permission response; OpenCode's native remember behavior is not canonical.

## Runtime-specific capabilities

The following should remain capability-gated rather than becoming mandatory PIAdapter behavior: native todos, share/unshare, session diff, revert/unrevert, OpenCode commands, native agents, native shell endpoint, provider OAuth, file/symbol search, and OpenCode plugins.

## Proof result and remaining gates

- OpenCode `1.18.3` authenticated/unauthenticated health, isolated session creation, abort, and controlled OCAdapter conformance passed.
- A live model-backed write produced a native permission request; C4OS rejected it before the file existed, and the file remained absent.
- A diagnostic per-request `tools: { write: true }` override bypassed the loaded wildcard `ask` policy. OCAdapter must forbid such overrides or route them through an independent C4OS authorization before dispatch.
- Event reconnect, ordering, duplication, backpressure, and server crash recovery are unproven.
- Supervisor tests passed per-launch state isolation and diagnostic redaction on macOS; configuration reload and version migration remain production work.
- macOS Tauri `externalBin`, ad-hoc bundle integrity, and descendant cleanup passed. Windows/Linux packaging and cleanup gate those targets.
- Remote OpenCode is not part of the initial adapter proof.

## Sources

- [OpenCode server API](https://opencode.ai/docs/server/)
- [OpenWork architecture](https://github.com/different-ai/openwork/blob/dev/ARCHITECTURE.md)
- Repository proofs: `proofs/opencode-runtime/`, `proofs/runtime-adapter-conformance/`, and `proofs/tauri-runtime-supervisor/`
