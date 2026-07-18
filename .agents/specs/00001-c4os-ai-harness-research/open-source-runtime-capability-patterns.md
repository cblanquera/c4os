# Open-Source Runtime and Model-Capability Patterns

State: Research complete; incorporated into accepted P-018
Researched: 2026-07-18

## Scope

This pass looked for open-source AI projects that do both of the following:

- place an application or client boundary around an agent runtime such as OpenCode or Pi; and
- change session behavior based on model, agent, transport, or execution capabilities.

Projects that are only multi-provider chat clients are secondary evidence. They can demonstrate useful capability registries, but they do not prove C4OS's runtime supervision, policy, or session-ownership architecture.

## Closest runtime-boundary references

| Project or standard | Runtime boundary | Capability handling | C4OS lesson |
| --- | --- | --- | --- |
| [Agent Client Protocol](https://agentclientprotocol.com/) | A client starts or connects to an agent and exchanges versioned JSON-RPC messages. Initialization negotiates client and agent capabilities. | Agent capabilities cover prompt media, session operations, authentication, MCP transports, and other protocol behavior. Session Config Options let an agent advertise model, mode, reasoning level, and model-related controls dynamically. | Strongest open standard for agent/client negotiation. It complements, but does not replace, C4OS's model-capability truth and policy boundary. |
| [OpenCode ACP](https://opencode.ai/docs/acp/) | `opencode acp` runs OpenCode as an ACP-compatible stdio subprocess. | OpenCode exposes its tools, MCP servers, rules, agents, permissions, and most other behavior through ACP. | A future ACP-compatible transport is realistic for OCAdapter, but the proven authenticated server/SSE path remains richer C4OS evidence today. |
| [pi-acp](https://github.com/svkozak/pi-acp) | A separately maintained adapter exposes Pi through ACP. It is listed in the official ACP registry. | It translates Pi into ACP rather than adding a native ACP surface to Pi itself. | ACP can normalize Pi transport, but an extra community-maintained layer adds compatibility and supply-chain risk. PIAdapter should retain its selected C4OS-owned Node SDK sidecar. |
| [Goose](https://github.com/aaif-goose/goose) | Desktop, CLI, and API clients can use direct LLM providers, CLI providers, or external coding agents through ACP. | Provider configuration exposes models, context limits, streaming, thinking and generation settings; ACP agents supply their own session behavior and MCP tools. | Closest broad host reference. Its “ACP agent as provider” design is flexible, but C4OS should not conflate agent runtime, model provider, and execution environment. |
| [OpenHands](https://github.com/OpenHands/OpenHands) | The agent/backend emits actions to an isolated sandbox and receives observations. Local Docker and remote sandboxes share a workspace interface. | It maintains verified-model and feature-specific model sets, plus configuration overrides for vision, prompt caching, and unsupported parameters. | Strong evidence for keeping OpenCode/Pi runtime selection separate from Local/Docker/SSH execution environments. Its curated model lists are useful observed evidence, not a complete product contract. |

The [ACP registry](https://github.com/agentclientprotocol/registry) currently includes both OpenCode and `pi-acp`, along with Cline, Goose, Codex, Gemini, and other agents. Registry presence establishes distribution and handshake compatibility, not that every agent exposes equivalent semantics or deserves the same trust.

## Strong model-capability references

| Project | Capability representation | C4OS lesson |
| --- | --- | --- |
| [Cline](https://github.com/cline/cline/blob/main/src/shared/api.ts) | Provider-specific `ModelInfo` records include context/output limits, image input, prompt caching, reasoning and effort controls, tools, streaming, and prices. The same underlying model can have different flags through different provider or CLI paths. | Capabilities must be scoped to the actual provider endpoint and runtime path, not inferred from a model name alone. |
| [Cherry Studio](https://github.com/CherryHQ/cherry-studio) | Model entries and user overrides govern features such as function calling. Releases include capability corrections for particular model families. | Capability metadata needs observable corrections and explicit overrides. Function calling is a model/path capability; MCP availability is a separate tool-inventory concern. |
| [Continue](https://github.com/continuedev/continue/blob/main/extensions/vscode/config_schema.json) | Model configuration includes provider, endpoint, context length, maximum tokens, templates, and generation options. | User-configured or custom endpoints require an `unknown`/unverified state instead of optimistic feature inference. |

These projects validate useful fields and UX patterns. They are not evidence that a hard-coded Boolean registry alone can safely drive C4OS.

## Patterns worth adopting

### 1. Separate protocol capabilities from model capabilities

ACP's initialization describes what an agent/client connection can exchange: images, audio, embedded context, session loading, filesystem or terminal methods, and similar protocol behavior. That is different from whether the selected model can actually use an image, reason at a requested level, or produce schema-constrained output.

C4OS should therefore retain two adjacent but independent surfaces:

- `runtimeSessionOptions`: dynamic controls and values currently offered by the runtime or agent; and
- `modelCapabilities`: C4OS-normalized, evidence-bearing facts for the exact model/provider/adapter path.

The visible control set is their intersection with the execution environment, installed resources, configuration, and C4OS policy.

### 2. Replace dependent option state atomically

ACP requires a session-option change to return the complete current option list. This prevents stale UI when changing a model adds or removes reasoning levels, modes, or other dependent controls.

C4OS should adopt the same state rule even when an adapter does not use ACP: a model or runtime-option change returns a complete recomputed session-control snapshot. The adapter-provided option list remains declared runtime state; C4OS still validates it against effective model capabilities and policy.

### 3. Scope evidence to the complete route

Cline demonstrates why `modelId -> capabilities` is insufficient. A base model reached through a native API, gateway, CLI wrapper, or agent runtime may expose different media, caching, streaming, tools, or reasoning behavior.

Each material C4OS value should therefore record a capability scope containing the provider, endpoint/API, provider model ID and revision, adapter kind/version, runtime kind/version, and relevant session configuration. Broader declarations may seed a narrower route but cannot silently override contradictory route evidence.

### 4. Combine declarations with verified exceptions

OpenHands' feature-specific model sets and Cherry Studio's release corrections show that practical clients maintain tested allowlists, denylists, or overrides even when providers publish metadata. C4OS's existing declared/normalized/observed/effective layers already support this pattern without turning exceptions into unexplained hard-coded truth.

Observed exceptions need provenance, the tested route, a timestamp, and optional expiry. A later runtime/provider update invalidates or lowers confidence in route-specific evidence until compatibility checks pass again.

### 5. Keep runtime and execution environment orthogonal

OpenHands treats the workspace/sandbox as the command and file execution location. Goose can host an external ACP agent. Neither pattern implies that an agent runtime and its execution target are the same selection.

C4OS should preserve the existing product axes:

```text
OCAdapter or PIAdapter
        x
Local Desktop, Docker, or Remote SSH
        x
model/provider endpoint and effective capabilities
```

This is more explicit than treating an ACP agent, LLM provider, and sandbox as one provider-shaped object.

## ACP recommendation

Do not replace the accepted OCAdapter and PIAdapter designs with a generic ACPAdapter now.

- OpenCode's native ACP support makes ACP a credible optional OCAdapter transport or future compatibility surface.
- Pi currently reaches ACP through a separate community adapter, whereas the selected C4OS Node SDK sidecar directly owns Pi session integration.
- ACP normalizes client/agent messages and dynamic session options, but it does not define C4OS approval authority, complete model capability truth, persistence, credential isolation, or execution-environment semantics.
- A future third-party runtime adapter may use ACP as its baseline transport while still satisfying the same C4OS-owned adapter contract and policy gateway.

## P-018 refinement

P-018 should retain the four capability layers and add these requirements:

1. Capability evidence is scoped to the complete provider/endpoint/adapter/runtime route.
2. Adapter-declared session options and C4OS-normalized model capabilities are stored separately.
3. Changing a model or dependent runtime option atomically replaces the complete session-control snapshot.
4. Runtime/provider version changes invalidate or lower confidence in affected observed evidence until rechecked.
5. ACP compatibility is optional transport compatibility, not a new authority boundary or proof of semantic parity.

## Proof implication

The planned model-capability acceptance fixture should add:

- one model whose capabilities differ between two provider/runtime routes;
- one model switch that removes an available reasoning option and returns a complete new control snapshot;
- one runtime-declared option contradicted by effective model or policy evidence; and
- native OpenCode/Pi results compared with any future ACP transport without assuming identical optional semantics.

No new root Proof Loop item is required. These are extensions of adapter conformance and the planned model-capability fixture matrix.
