# Runtime Capability Matrix

State: Proposed design; tested conformance boundary passed
Compared: 2026-07-18

## Interpretation

- **Native** means the upstream runtime currently documents a direct surface.
- **Adapter** means C4OS must translate, correlate, constrain, or supervise that surface.
- **C4OS-owned** means the runtime must not be treated as the product authority.
- A documented capability is not marked proven until the repository Proof Loop verifies the pinned integration.
- This matrix describes runtime/adapter behavior. Model-specific modalities, reasoning, tool-calling, structured-output, and limit metadata are normalized separately in `model-capabilities.md`; a runtime capability does not imply that every model supports it.

## Capability comparison

| Capability | OCAdapter / OpenCode | PIAdapter / Pi | C4OS responsibility |
| --- | --- | --- | --- |
| Runtime topology | Managed HTTP server with OpenAPI SDK and SSE | Node SDK sidecar or Pi RPC process | Install, pin, isolate, authenticate where applicable, supervise, recover, and report health. |
| Workspace binding | Native project/path/VCS context | Session `cwd` and resource/tool path resolution | Authoritative workspace identity, trusted root, relink, Local/Docker/SSH mapping. |
| Session create/resume | Native session CRUD and status | AgentSessionRuntime and SessionManager | Stable C4OS session ID and persisted runtime/native binding. |
| Session branching | Native children and fork endpoint | Runtime fork and session-tree navigation | Common ancestry/provenance; advertise exact native operations. |
| Streaming | SSE server and session events | SDK subscriptions or RPC JSON-line events | Normalized envelopes, IDs, ordering, reconnect, duplicate suppression, backpressure. |
| Prompt delivery | Sync message and async prompt endpoints | Prompt, steer, and follow-up | Common send operation; steer/follow-up remain Pi capability flags. |
| Cancellation | Session abort | Prompt/compaction abort and dispose | Idempotent UI state, late-event rejection, process fallback. |
| Compaction | Session summarize | Native compact and compaction events | Common user-visible state and summary provenance; semantics may differ. |
| Messages/history | Native message parts and session storage | Session messages and files | Product history, redaction, export, retention, and recovery remain C4OS-owned. |
| Tool inventory | Native/experimental tool surfaces plus commands, shell, files | Built-in, custom, and extension tools | Decide exposed tools; use C4OS broker for privileged execution. |
| Tool interception | Live native permission rejection passed; per-request tool overrides are authority-bearing | Live custom-tool `beforeToolCall` rejection passed | C4OS approval and trusted-root decision must happen before side effects. |
| Permission response | Native session permission response | Adapter continuation around brokered tool or extension interaction | Classify action facts, evaluate category policy and concrete exceptions, issue single-use authorization, and audit. |
| Providers/models | Provider/config/auth/OAuth endpoints plus a rich normalized model descriptor | Model runtime/registry and custom providers plus a smaller core model descriptor and provider compatibility settings | Credential storage, stable IDs, model-capability normalization, effective per-run snapshots, policy, and cross-runtime presentation. |
| Skills/context | OpenCode skills/config conventions | ResourceLoader for skills, prompts, extensions, and `AGENTS.md` | Scope precedence, validation, enablement, provenance, and runtime translation. |
| MCP/extensions | Native OpenCode MCP/plugins/agents | Pi extensions; MCP may be adapter or C4OS supplied | C4OS plugins and marketplace remain separate; expose runtime-native extensions honestly. |
| Artifacts/diffs | Native message parts, session diff, file APIs | Tool results and edit patch/diff details | Stable artifact IDs, storage, preview, privacy, and cross-runtime rendering. |
| Runtime-native UI | Todos, share, revert, commands, native agents | Steer/follow-up, tree navigation, extension UI, thinking controls | Capability-gated UI; do not emulate solely for superficial parity. |
| Remote connection | OpenCode server can be connected by URL; trust model unresolved | RPC/SDK can be hosted remotely only through C4OS infrastructure | Remote identity, TLS/auth, workspace mapping, reconnect, secrets, and policy. |
| Configuration reload | Instance dispose and restart/reload patterns | ResourceLoader reload or process/session recreation | Warn about active work, persist intent, rollback, and show errors. |

## Derived C4OS baseline

Both adapters must provide these C4OS-facing behaviors, even when their native implementations differ:

1. **Identity** — runtime kind, adapter version, native version, process generation, and stable native-ID mappings.
2. **Availability** — probe, compatibility result, start, readiness, degraded state, stop, and diagnostics.
3. **Capabilities** — versioned required/optional flags with unsupported reasons; no silent fallback to the other runtime.
4. **Workspace binding** — one explicit trusted workspace/execution environment per active binding.
5. **Sessions** — create, resume, enumerate mapped sessions, send, receive, cancel, and close.
6. **Events** — normalized lifecycle, content, tool, approval, usage, compaction, completion, cancellation, and error envelopes.
7. **Tool safety** — C4OS-approved tool inventory and denial-before-side-effect behavior.
8. **Provider/model mapping** — list usable choices, normalize their declared/observed capability evidence, and resolve stable C4OS IDs to native IDs without exposing secrets.
9. **Persistence and recovery** — restore mappings, detect stale processes, reconcile native state, and reject late-generation events.
10. **Observability** — redacted logs, native error details, source event types, health/version data, and exportable diagnostics.

Both adapters must emit the pre-execution action intent in `approval-policy-model.md`. The 71 r012 scenarios are classifier/proof fixtures, not adapter method names or required user-facing settings.

The baseline is derived from the two adapters. It is not a requirement that their internal APIs, event counts, session models, extensions, or optional features match.

## Candidate optional capability flags

These optional capability names remain implementation-level candidates even though the current conformance proof passed:

- `session.fork`, `session.treeNavigation`, `session.share`, `session.revert`
- `prompt.steer`, `prompt.followUp`, `prompt.async`
- `context.compact`, `context.skills`, `context.agentsFiles`, `context.promptTemplates`
- `tools.dynamic`, `tools.progress`, `permissions.nativeRequests`
- `artifacts.nativeDiff`, `workspace.fileSearch`, `workspace.symbolSearch`, `workspace.vcs`
- `providers.oauth`, `extensions.native`, `extensions.uiRequests`
- `runtime.remoteConnect`, `runtime.resourceReload`

## Deferred implementation choices

These choices do not block research Freeze. Resolve them in the affected implementation or UI specification:

- Accepted P-011 establishes that saved defaults affect only new chats and that the first valid submission binds runtime/environment.
- Which optional capabilities appear in shared UI, and which live in runtime-specific settings or actions?
- Does C4OS permit native OpenCode/Pi extensions in addition to C4OS plugins, and under what trust model?
- Is remote OpenCode a first-class initial mode, or is Remote SSH implemented only through C4OS execution infrastructure?
- Which native history is imported when a runtime is first connected, if any?

## Conformance result

The conformance harness now uses separate expectation manifests for `OCAdapter` and `PIAdapter`:

- one required baseline shared by both;
- one declared optional-capability set per adapter;
- runtime-specific fixtures for native operations; and
- identical safety assertions for tool denial, trusted-root enforcement, secrets, cancellation, stale events, and crash recovery.

Both adapters passed the tested baseline and their pinned current-package denial paths. This does not mean Pi and OpenCode produce identical native behavior, and future package versions must rerun the pins and contract fixtures.
