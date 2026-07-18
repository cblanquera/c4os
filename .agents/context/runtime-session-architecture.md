# Runtime and Session Architecture

State: Accepted reusable truth
Accepted: 2026-07-18

## Product intent and wireframe authority

`wireframes/r012-cleanup/` remains the current product-intent and visual-interaction reference until a later revision is reviewed and accepted. A research-spec Freeze closes its decision and evidence set; it does not retire the wireframes or make visual design unnecessary. Wireframe behavior is not implementation evidence.

A future wireframe revision must express the accepted four-preset approval model and capability-aware model/session experience while preserving the established product surface. Research supplies architectural and behavioral constraints for that revision; the revised wireframes remain the human-reviewable UI contract.

## Authority boundary

The Tauri/Rust C4OS core owns authoritative workspace, session, turn, run, artifact, approval, credential-reference, and audit records. It also owns native windows and dialogs, secure-storage access, process supervision, updates, and the privileged tool gateway. The renderer submits user intent and receives no ambient authority.

Runtime adapters, managed runtime processes, MCP servers, and Local/Docker/SSH executors are supervised workers. They may retain native runtime state and request actions, but they do not decide C4OS policy, mint authorizations, read raw credentials, or mutate through unbrokered privileged paths. Remote executors may perform an exact authorized effect while the local C4OS core remains the decision and audit authority.

Computation may run outside Rust. Durable product state and security authority remain in the Rust-owned core so sidecars stay replaceable and recoverable.

## Runtime integrations

OpenCode and Pi are peer user-selectable runtimes behind separate adapters. C4OS manages OpenCode as an authenticated, loopback-bound server with isolated state and explicit health, restart, and shutdown behavior. C4OS integrates Pi through a C4OS-owned Node SDK sidecar; Pi RPC remains an alternative transport rather than the selected initial integration. Neither runtime owns C4OS policy or becomes architecturally primary.

## Plugin boundary

Plugins begin as declarative bundles of metadata, skills, MCP/app declarations, settings schemas, and explicitly reviewed hooks. A plugin does not receive native Rust or arbitrary shell authority by implication. Every executable surface remains subject to C4OS trust, approval, activation, and revocation boundaries.

Marketplaces are plugin catalog sources. Adding a marketplace fetches enough validated metadata to list and search its plugins; it does not install or execute them. C4OS owns the lifecycle of an individually selected plugin, including verification, disabled installation, enablement, transactional update, rollback, revocation, and removal.

## Browser boundary

Users may navigate to arbitrary ordinary websites. Every website remains unprivileged relative to C4OS and receives no Tauri internals, C4OS commands, credentials, filesystem authority, or page-accessible native IPC. Browser capabilities remain generally usable: native mediation may add per-origin security and remembered choices only while preserving normal browser behavior, and platform/browser defaults remain available rather than being replaced with a blanket denial. Each supported target must prove its real permission UX and isolation boundary.

## Compatibility scope

Codex specifications are standards research for C4OS. The current product does not import Codex plugins or `config.toml` and does not claim Codex compatibility. A future importer requires a separately accepted, versioned scope.

## Skill resolution

Every skill retains a stable source-qualified identity. Only valid, enabled, eligible skills enter suggestions or runtime context. Project/workspace skills require a trusted root; plugin skills require their plugin to be trusted and enabled.

The default collision precedence is `project-local > workspace-local > user-global > plugin-provided > bundled`, with an explicit workspace selection taking priority. C4OS shows the active source and collision state, preserves fully qualified identities, and never deletes another source during resolution. Customizing an immutable skill creates a user-owned copy and explicit override. Metadata loads for discovery; full instructions and referenced resources load progressively only after activation.

## Credential and runtime-state isolation

Only the Rust-authoritative core reads stored secrets. An installation master key lives in the operating system credential service and protects a C4OS-owned encrypted vault. Databases, renderer state, logs, diagnostics, workspace files, runtime configuration, and normal exports carry opaque references rather than raw credentials. Without an OS credential service, C4OS offers an explicit password-protected vault or session-only credentials—never plaintext fallback.

Workers receive only the current operation's secret through a short-lived channel, not command-line arguments or broad inherited environment. Remote execution prefers local references and SSH/OS agents; copying a reusable secret to a remote host requires a separately accepted feature boundary. Runtime state lives under C4OS application data and is namespaced by runtime kind, native version, workspace binding, and process generation. Reset is scoped, normal exports exclude secrets, imports require reauthentication, and diagnostics redact values and credential-bearing fields.

## Update lifecycle

The application, OpenCode/Pi runtimes, and plugins use separate versioned update channels. Updates stage and verify before activation, never replace an active component, check compatibility and health, and retain a last-known-good version.

Application updates require platform and Tauri update signatures plus a pre-migration data snapshot. Runtime updates remain pinned to the adapter compatibility matrix. Executable plugin or capability changes stage disabled for review; unchanged trusted capabilities may preserve enablement only after validation. Offline use continues with installed versions. Failures preserve current working state and permissions; air-gapped packages require equivalent signatures or pinned digests.

## Chat binding

Runtime and execution-environment settings are defaults for new chats. A provisional blank chat captures its runtime kind, adapter binding, environment identity, workspace mapping, and capability baseline on its first valid prompt or attachment submission. Later default changes affect only new chats.

Existing chats never migrate implicitly. Adapter and native versions are recorded per run. Initially, moving a conversation to another runtime or environment means duplicating its user-visible context into a new chat; true native-session migration remains deferred until compatibility is proven.

## Configuration activation

- Display-only metadata updates immediately.
- Policy tightening, disablement, revocation, and credential removal take effect immediately, invalidate outstanding authorizations, and stop affected executable services when required.
- Skills, prompt resources, ordinary plugin enablement, MCP tool availability, and provider configuration are snapshotted at turn start and affect the next turn, never an active run.
- Executable plugin or MCP changes activate only after validation and process readiness; failure preserves the prior working state.
- Runtime and environment defaults affect only new chats.
- Adapter or native-runtime updates wait for active runs to finish or be cancelled, pass compatibility checks, and record the new version on the next run.

Every run retains the effective configuration and resource provenance needed to explain what was available when it executed.

## Retry behavior

Retry creates a new run attempt under the same immutable user turn. The failed or interrupted attempt, its output, completed tool effects, and audit records remain intact. The new attempt reuses the original user prompt and attachment snapshot, captures current approved configuration, and requires fresh single-use authorizations for every proposed side effect.

C4OS does not claim cross-runtime checkpoint continuation. If a prior effect has unknown completion, the user must review it before retry. Automatic retry is allowed only before any side effect or when C4OS can prove the operation was idempotent and did not complete.

## Provenance presentation

The assistant identity remains C4OS. Provenance appears where it changes user understanding or authority:

- The chat header shows runtime, execution environment or host alias, workspace, and health; the model control shows the current model.
- Each turn retains provider/model, adapter/native version, environment, and effective capability/configuration provenance in expandable details.
- Approval surfaces show the action, target, workspace, environment/host, requesting runtime, and effective policy result.
- File, Terminal, and Browser artifacts expose their source run and relevant path, shell, environment, or URL in compact details.
- Errors identify the failing runtime/environment/version boundary and recovery action.
- Audit and diagnostics retain the full redacted provenance and correlation chain.

Use compact chips or expandable details rather than changing the response author from C4OS or crowding every message.

## Approval presentation

The Advanced Policies user model has four presets: `Ask for approval`, `Approve safe actions`, `Approve for me`, and `Custom`. `Approve for me` remains bounded by sandbox, trusted-root, maximum-authority, and managed-policy constraints. The detailed r012 authority identities are an internal scenario corpus, not permanent settings rows.

## Model and session capabilities

C4OS owns a versioned model-capability descriptor instead of adopting an OpenCode, Pi, provider, or ACP schema as its product contract. `OCAdapter` and `PIAdapter` preserve raw evidence and normalize identity and lifecycle, input/output modalities, context/output limits, reasoning, tool calling, structured output, streaming, generation controls, and caching/session requirements.

Capability state has declared, adapter-normalized, observed, and effective layers. Material fields use `supported`, `unsupported`, `unknown`, or `degraded`, retain their source, time, constraints, and reason, and are scoped to the complete provider/endpoint/model-revision/adapter/runtime route. The effective set is the narrow intersection with the execution environment, installed resources, configuration, and C4OS policy. Numeric limits use the narrowest confirmed value.

Adapter-declared `runtimeSessionOptions` remain separate from normalized `modelCapabilities`. Changing a model or dependent option atomically replaces the complete recomputed session-control snapshot before C4OS intersects it with effective capabilities and policy. Every run snapshots its exact route and effective descriptor. Unsupported or unknown state affects only the dependent experience and is explained; C4OS does not silently drop input, fabricate reasoning, mislabel prompted JSON, or equate model tool calling with installed or authorized tools.

ACP is an optional runtime transport or future compatibility surface. It does not replace the separate OCAdapter and PIAdapter contracts, establish semantic parity, or move authority out of C4OS.

## Provenance

- [Runtime and session acceptance record](../specs/00001-c4os-ai-harness-research/acceptance/2026-07-18-runtime-session-architecture-acceptance.md)
- [Adapter and plugin acceptance record](../specs/00001-c4os-ai-harness-research/acceptance/2026-07-18-adapter-plugin-acceptance.md)
- [Skill, credential, and update acceptance record](../specs/00001-c4os-ai-harness-research/acceptance/2026-07-18-skill-credential-update-acceptance.md)
- [Model-capability acceptance record](../specs/00001-c4os-ai-harness-research/acceptance/2026-07-18-model-capability-acceptance.md)
- [Research decision and gap ledger](../specs/00001-c4os-ai-harness-research/decisions.md)
- [Approval policy model and scenario corpus](../specs/00001-c4os-ai-harness-research/approval-policy-model.md)
