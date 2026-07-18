# PIAdapter

State: Proposed design; tested transport selected against Pi 0.80.10
Research refreshed: 2026-07-18

## Purpose

`PIAdapter` exposes Pi to C4OS without making Pi's SDK objects, session files, tools, extensions, or configuration the C4OS product contract. Pi and OpenCode remain peer user-selectable runtimes.

## Transport candidates

Pi currently provides two viable integration surfaces:

| Candidate | Strength | Cost |
| --- | --- | --- |
| SDK in a C4OS-owned Node sidecar | Direct typed access to sessions, state, custom tools, resource loading, and extension APIs. | C4OS must build and version its own sidecar protocol and supervise the Node process. |
| Pi RPC process | Existing newline-delimited JSON command/event protocol and process isolation. | Events do not carry IDs, some extension UI behavior is degraded, and adapter control is limited to the exposed protocol. |

Tauri's Rust process should not embed Node directly. Even the SDK option therefore implies a supervised C4OS Node sidecar. The adapter-conformance proof should select the PIAdapter transport on fidelity, interception safety, recovery, and packaging—not compare Pi's product value against OpenCode.

## Native Pi capabilities

| Area | Current upstream surface | Adapter use |
| --- | --- | --- |
| Sessions | `AgentSession`, `AgentSessionRuntime`, session managers, new/resume/fork/import, tree navigation, names, and session files | Map native sessions and branches to C4OS runtime bindings while keeping C4OS identity authoritative. |
| Prompting | Prompt, steer, follow-up, queued messages, and images | Send chat input and expose steer/follow-up as Pi-specific capability flags. |
| Streaming | Session subscription or RPC JSON-line events | Translate agent, turn, message, tool, compaction, retry, queue, and extension-error events. |
| Model control | Model runtime/registry, model selection, thinking level, and custom providers | Populate runtime-native model choices while C4OS owns credential and provider policy. |
| Tools | Selectable built-ins, custom tools, extension-registered tools, tool-call events, and active-tool control | Prefer C4OS-brokered custom tools; disable or constrain native side-effecting tools unless proven safe. |
| Compaction | Manual and automatic compaction events plus abort | Normalize compaction state and preserve summary provenance. |
| Cancellation | Abort prompt and compaction; dispose session | Produce idempotent C4OS cancellation and cleanup states. |
| Resources | ResourceLoader for skills, extensions, prompts, themes, and `AGENTS.md` context | Supply C4OS-approved resources explicitly and retain diagnostics/source provenance. |
| Extensions | Event handlers, custom tools, commands, shortcuts, flags, and an event bus | Use only for adapter plumbing reviewed by C4OS; do not equate Pi extensions with C4OS plugins. |
| RPC UI | Dialog request/response and fire-and-forget UI sub-protocol | Translate supported prompts/notifications; capability-gate degraded or TUI-only behavior. |

## Proposed C4OS operations

| PIAdapter operation | Native mapping | Required normalization |
| --- | --- | --- |
| `probe()` | Package/CLI version plus minimal process or SDK initialization | Installed, compatible, unavailable, degraded, or update-required. |
| `start(binding)` | Start RPC or C4OS SDK sidecar and initialize services | Workspace cwd, isolated agent/state paths, approved resource loader, sanitized logs. |
| `stop()` | Dispose session/services or terminate RPC/sidecar | Graceful timeout, forced termination, descendant cleanup, final status. |
| `describeCapabilities()` | Transport and loaded resource/tool inventory | Required and optional capability flags plus degraded RPC UI features. |
| `createSession()` | AgentSessionRuntime new-session flow | C4OS ID to Pi ID/file mapping and persisted runtime binding. |
| `resumeSession()` | Runtime resume plus session subscription/state query | Rehydrate normalized history and restart event sequence safely. |
| `send()` | Prompt, with approved images and resources | C4OS attachments, model selection, active tools, and correlation identifiers. |
| `steer()` / `followUp()` | Native queue APIs or RPC commands | Capability-gated queue state and normalized delivery status. |
| `cancel()` | Abort prompt/compaction | Idempotent terminal state and late-event handling. |
| `fork()` | AgentSessionRuntime fork or tree branch behavior | Preserve C4OS ancestry and native node provenance. |
| `compact()` | Session compact | Report compaction lifecycle and retain C4OS-visible summary provenance. |
| `respondToApproval()` | C4OS custom tool continuation or extension/RPC response | Resume the same trace only after an app-owned decision; never execute on denial. |
| `listModels()` | Model runtime/registry | Stable C4OS provider/model IDs mapped to Pi models. |
| `reloadResources()` | ResourceLoader reload | Apply approved skill/context changes with diagnostics and active-session policy. |

## Event translation

`PIAdapter` must create C4OS-owned envelopes containing:

- C4OS runtime, workspace, session, turn, message, tool-call, and artifact identifiers where applicable;
- original Pi event type and any native identifiers;
- adapter-generated correlation IDs and monotonic sequence, especially because RPC events do not include IDs;
- normalized lifecycle, text/thinking delta, tool request/progress/result, approval request, usage, compaction, retry, completion, cancellation, and error categories; and
- transport/process generation so events from a replaced process cannot mutate the current session.

Unknown extension and runtime events remain diagnostics and must not break the stream.

## C4OS-owned boundaries

- Runtime selection and per-session runtime binding.
- Trusted roots, sandbox rules, approval records, remembered scopes, and tool execution.
- Stable workspace, session, message, tool-call, and artifact identity.
- Provider credentials; Pi auth files are not the product source of truth.
- The approved skill/context set and its precedence across C4OS scopes.
- Plugin, Codex bundle, MCP, and marketplace policy.
- Sidecar/RPC supervision, version pinning, state isolation, diagnostics, redaction, and recovery.

The preferred safety shape is to expose C4OS-brokered custom tools and omit Pi's native side-effecting tools. If native tools remain enabled, conformance must prove that C4OS policy intercepts them before execution.

Before execution, `PIAdapter` must emit the action intent defined in `approval-policy-model.md`. C4OS evaluates category rules and concrete exceptions, then returns a single-use authorization or denial. Pi extensions, RPC UI requests, and session state are not the canonical remembered-policy store.

## Runtime-specific capabilities

The following should remain capability-gated rather than mandatory OCAdapter behavior: steer, follow-up queues, session-tree navigation, custom resource-loader overrides, Pi extensions, extension UI requests, thinking-level cycling, active-tool mutation, and Pi prompt templates.

## Proof gaps

- Pi `0.80.10` passed a current-package pre-tool denial in which the tool body executed zero times.
- PIAdapter selects a C4OS-owned Node SDK sidecar; RPC remains a documented alternative rather than an undecided primary transport.
- Persistent session recovery, crash restart, event correlation, and duplicate suppression are unproven.
- Durable trace resume after a real process crash remains unproven.
- Artifact identity, provider secret delivery, resource precedence, and reload behavior remain app-owned and unproven.
- Windows/Linux packaging and descendant-process cleanup are unproven.

## Sources

- [Pi SDK](https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/sdk.md)
- [Pi RPC protocol](https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/rpc.md)
- [Pi extensions](https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/extensions.md)
- Repository proofs: `proofs/pi-runtime/` and `proofs/pi-runtime-app-layer-proof/`
