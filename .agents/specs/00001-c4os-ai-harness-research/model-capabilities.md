# Model Capabilities and Chat Experience

State: Research complete; P-018 accepted 2026-07-18
Researched: 2026-07-18

## Finding

Models do have capabilities that materially change a chat session, but there is no universal cross-provider capability specification. A capability may describe accepted media, output types, reasoning controls, tool calling, structured output, context limits, sampling controls, caching, streaming, or a provider-native tool. Providers expose different subsets and runtimes normalize them differently.

C4OS therefore needs its own model-capability descriptor. It must keep model ability separate from runtime ability and C4OS authority:

- A model may support function calling, but it does not execute a C4OS tool itself.
- A runtime may be able to translate tool calls, but that does not mean a particular tool is installed or enabled.
- A tool may be available, but C4OS policy may still ask, constrain, or deny its proposed action.
- A model may accept images through one provider endpoint while a runtime adapter or gateway path cannot deliver them correctly.

The effective experience is the intersection of those boundaries, not the union of every advertised feature.

## What current sources expose

| Source | Capability surface | Important limitation |
| --- | --- | --- |
| OpenAI model catalog | Per-model modalities, endpoints, context and output limits, reasoning levels, streaming, function calling, and structured-output support | Features vary by exact model and endpoint; model aliases may move while snapshots provide a more stable behavioral target. |
| Anthropic model catalog/API | Text/image input, text output, context/output limits, and a programmatic `capabilities` object; current reasoning modes and defaults vary by model | Adaptive, manual, always-on, and disabled thinking are not interchangeable, and some models reject sampling controls. |
| Gemini model catalog/API | Input/output token limits, generation methods, thinking support, and sampling metadata; separate documentation identifies media, function calling, structured output, built-in tools, and Live API support | Thinking controls differ between model generations; the base Models API does not describe the entire usable feature surface. |
| OpenRouter Models API | Input/output modalities, context length, endpoint/provider limits, and `supported_parameters` such as tools, reasoning, and structured outputs | It is gateway metadata. The selected upstream endpoint may differ, and unsupported request parameters can otherwise be ignored unless routing requires them. |
| OpenCode `1.18.3` SDK | `temperature`, `reasoning`, `attachment`, `toolcall`, text/audio/image/video/PDF input and output, interleaving, context/input/output limits, cost, status, variants, and provider/API identity | The descriptor is OpenCode's normalized view and can depend on catalog or custom-provider declarations; it is not proof that the end-to-end provider path works. |
| Pi `0.80.10` SDK | Provider/API identity, reasoning, thinking-level mapping, text/image input, context window, max output, cost, and provider-specific compatibility settings | The core model type is intentionally smaller. Tool support is expressed through the conversation/tool API rather than a `toolcall` model flag, and additional provider behavior lives in compatibility settings. |

Primary sources: [OpenAI model catalog](https://developers.openai.com/api/docs/models), [OpenAI model comparison](https://developers.openai.com/api/docs/models/compare), [Anthropic models overview](https://platform.claude.com/docs/en/about-claude/models/overview), [Anthropic adaptive thinking](https://platform.claude.com/docs/en/build-with-claude/adaptive-thinking), [Gemini Models API](https://ai.google.dev/api/models), [Gemini capabilities](https://ai.google.dev/gemini-api/docs), [Gemini function calling](https://ai.google.dev/gemini-api/docs/function-calling), [Gemini structured output](https://ai.google.dev/gemini-api/docs/structured-output), [OpenRouter Models API](https://openrouter.ai/docs/guides/overview/models), [OpenCode provider source](https://github.com/anomalyco/opencode/blob/dev/packages/opencode/src/provider/provider.ts), and [Pi model types](https://github.com/earendil-works/pi/blob/main/packages/ai/src/types.ts).

Local pinned evidence: `proofs/runtime-adapter-conformance/node_modules/@opencode-ai/sdk/dist/v2/gen/types.gen.d.ts` and `proofs/runtime-adapter-conformance/node_modules/@earendil-works/pi-ai/dist/types.d.ts`.

## Four capability layers

C4OS should retain four layers instead of flattening them into one Boolean:

1. **Declared** — raw provider, gateway, runtime catalog, or user configuration metadata, with source and retrieval time.
2. **Adapter-normalized** — the `OCAdapter` or `PIAdapter` translation into a versioned C4OS descriptor.
3. **Observed** — results from a probe, conformance fixture, or actual run. Observation may confirm, contradict, or leave a declaration untested.
4. **Effective** — what this run can use after model, provider endpoint, adapter, runtime, execution environment, installed resources, C4OS configuration, and policy are resolved.

Every material field should use `supported`, `unsupported`, `unknown`, or `degraded`, not a default-false Boolean. `degraded` means C4OS can provide a disclosed conversion or narrower path, such as extracting text from an otherwise unsupported document. It must not mean silent feature emulation.

## Proposed normalized descriptor

The first contract should normalize only fields that change product behavior:

| Group | Fields | Purpose |
| --- | --- | --- |
| Identity | C4OS model ID, provider ID, provider model ID, endpoint/API, revision or snapshot, adapter kind/version | Prevent an alias or gateway route from obscuring what ran. |
| Lifecycle | active, preview, deprecated, unavailable; release/deprecation data when known | Warn, filter, and prevent selection of unusable models. |
| Input | text, image, audio, video, PDF/file; MIME/size/count constraints when known | Control attachments and preflight the draft. |
| Output | text, image, audio, video; streaming support | Select the response renderer and interaction mode. |
| Context | input/context limit, maximum output, token-count confidence | Drive budget warnings, compaction, and output controls. |
| Reasoning | none/fixed/adaptive/manual/always-on; supported effort levels; summary/opaque/visible forms; interleaved tool reasoning | Present only valid controls and preserve provider-required reasoning state. |
| Tools | function calling, parallel calls, strict schemas, streamed arguments, provider-native tools | Determine whether tool-capable turns can be formed; it does not grant tool authority. |
| Structured output | JSON mode and JSON Schema support/subset | Enable schema-constrained response workflows. |
| Generation | temperature, top-p/top-k, stop, seed, verbosity, and other supported controls | Avoid presenting controls that a model rejects or ignores. |
| Caching/session | prompt-cache modes, retention choices, session affinity, and continuity requirements | Explain latency/cost changes and preserve valid multi-turn state. |

Provider-specific fields remain namespaced raw metadata until two adapters need the same product behavior. C4OS should not turn every provider parameter into a permanent cross-runtime field.

Each normalized value retains:

- the raw source and source priority;
- `checkedAt` and optional expiry;
- the provider/model/endpoint/revision to which it applies;
- declared and observed state;
- an unsupported or degraded reason; and
- constraints or allowed values, not just availability.

Capability scope includes the complete usable route: provider, endpoint/API, provider model ID and revision, adapter kind/version, runtime kind/version, and relevant session configuration. A base-model declaration may seed this route, but it does not overrule narrower evidence that the same model behaves differently through another gateway, CLI wrapper, or runtime.

The adapter also exposes `runtimeSessionOptions` separately from `modelCapabilities`. Session options describe the model, mode, reasoning level, or other controls the current runtime says it can present. They are not capability truth. C4OS intersects them with the effective descriptor and policy, and any model or dependent-option change atomically replaces the complete recomputed control snapshot so stale choices cannot remain visible or active.

## Effective capability resolution

For a run, `effectiveCapabilities` are snapshotted with its existing provenance. Resolution follows these rules:

- A feature is usable only if every required layer supports it. For example, image input requires model acceptance, provider-endpoint transport, adapter translation, and an allowed attachment path.
- Numeric limits use the narrowest applicable confirmed limit.
- `unknown` does not silently become `supported`. It disables only the dependent control or blocks the incompatible draft with a specific explanation; it does not disable ordinary text chat.
- A provider-native tool is not automatically a C4OS tool. It must either pass through the C4OS action/approval boundary or be explicitly classified as an upstream service with disclosed data, cost, and policy behavior.
- Runtime tool availability is resolved independently from model tool-calling ability. Effective tool use requires both, plus C4OS enablement and policy.
- An adapter or provider contradiction produces `degraded` or `unsupported` with diagnostics. The most optimistic declaration never wins silently.
- User overrides may correct incomplete custom-provider metadata, but are labeled unverified until probed; they do not bypass C4OS safety policy.

## Chat-session effects

| Capability outcome | Required experience |
| --- | --- |
| Input modalities | The attachment picker may remain generally available, but each draft is preflighted against the selected model path. Unsupported items remain visible with a reason and an explicit conversion/remove/change-model choice; they are never silently dropped. |
| Model switching | Before switching, C4OS checks the current draft, attachments, requested response mode, and tool needs. A conflict is explained before the switch or send. Existing transcript content remains intact. |
| Reasoning | Show only the modes and effort levels valid for that exact model path. Never fabricate a thinking transcript. Display provider summaries or opaque activity honestly and preserve required signatures privately. |
| Tool calling | If the model cannot call tools, the chat may still answer normally. Tool-required actions are disabled or require a compatible model. If supported, only the C4OS-approved tool inventory is exposed. |
| Structured output | Schema-driven UI is available only when the effective path supports it; ordinary prompted JSON is not labeled schema-guaranteed. |
| Context/output limits | Show approaching-limit and compaction states, reserve room for reasoning/output, and explain truncation or `max_tokens` completion. |
| Output modalities | Route generated image/audio/video through an appropriate artifact/player rather than assuming every response is Markdown text. |
| Streaming | Use progressive rendering only when supported. A complete-response path remains a valid, visibly different experience. |
| Sampling controls | Hide or disable unsupported settings instead of sending values that may be rejected or ignored. |
| Caching/provider-native services | Surface relevant privacy, retention, cost, and upstream-service consequences without treating them as model intelligence. |

The model selector should show compact, high-value capability chips or filters—such as Vision, Tools, Reasoning, Audio, and context size—with a details view for source and restrictions. It should not expose the full internal descriptor in the normal composer.

Capability support is also not a quality score. A model that technically accepts tool definitions or images may be unreliable at using them. Suitability, latency, price, and evaluated quality are separate selection signals.

## Wireframe cross-reference

`r012-cleanup` currently:

- lists and enables models by provider but shows no model capabilities;
- offers a small model menu in Chat;
- accepts arbitrary draft attachments and previews images;
- renders a thinking/activity disclosure for every assistant response; and
- assumes Markdown text as the normal final response.

A future wireframe revision should preserve the simple composer while adding capability-aware model details and send-time incompatibility handling. The universal “Thinking…” animation should become generic work/activity presentation unless the effective model path actually exposes a reasoning summary. The attachment picker should not be reduced to the current model's types because switching models or applying a disclosed conversion can still resolve the draft.

No edits to `wireframes/r012-cleanup/` are made by this research addendum.

## Proof applicability

Existing proofs already establish useful pieces:

- `attachment-compatibility` proves explicit attachment adaptation/degradation records and no silent loss.
- `model-attachment-adapter` proves a model-specific translation boundary.
- `runtime-adapter-conformance` proves separate adapter capability manifests and versioned provenance.
- `runtime-tool-discovery-without-plugin-view` reinforces that tool inventory and plugin UI are different concerns.

Before production enablement, adapter conformance should add a model-capability fixture matrix covering at least text-only, image input, reasoning controls, tool calling, structured output, an unknown custom provider, a declaration contradicted by a probe, and the same model through two routes with different capabilities. That proof should verify the normalized descriptor, effective intersection, model-switch/send preflight, atomic replacement of dependent session controls, and persisted run snapshot. It is an implementation acceptance test, not a blocker to accepting this research decision.

The open-source comparison supporting the route scope, dynamic session-state rule, and optional ACP compatibility direction is in `open-source-runtime-capability-patterns.md`.

## Accepted direction

P-018 is accepted: C4OS owns a versioned, evidence-bearing model-capability descriptor and snapshots the effective descriptor per run. `OCAdapter` and `PIAdapter` populate it from their different native surfaces; neither runtime's schema becomes the product standard. Unknown or incompatible capabilities produce narrow, explicit UX changes, and model capabilities never bypass runtime availability or C4OS policy.
