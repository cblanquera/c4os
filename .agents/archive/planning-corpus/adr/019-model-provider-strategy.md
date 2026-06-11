# ADR-019: Model Provider Strategy

Status: Finalized for MVP.

## Context

The preferred architecture routes model access through OpenRouter. Functional requirements include OpenRouter credential setup, model selection before session start, OpenRouter credits where available, and provider route visibility when available.

The review identifies OpenRouter lock-in risks: routing, pricing, BYOK fees, model slugs, provider behavior, privacy disclosures, and future direct-provider migration.

## Decision

Use OpenRouter as the only provider gateway for MVP.

Store the OpenRouter API key only in the OS keychain or platform credential store. If secure platform credential storage is unavailable or fails, provider setup is blocked with an actionable error.

OpenRouter API key update and revoke are allowed only when no session is running. A running session keeps the credential reference it started with until stopped or complete. Hot key rotation, per-session credential switching, and mid-call credential retry are post-MVP.

Direct provider integrations, local model providers, fallback routing, provider-specific app settings, and BYOK provider subconfiguration inside the app are post-MVP.

MVP uses one selected OpenRouter model per session. Mid-session model switching, per-agent model overrides, fallback routes, and budget controls are post-MVP.

App-owned provider and model settings are authoritative for MVP. The runtime effective model must match the selected OpenRouter model for the session. If OpenCode config can override provider or model routing and the app cannot detect or prevent that override, OpenCode is not viable as the direct MVP runtime until an adapter, wrapper, configuration strategy, or scope change resolves the mismatch.

Model metadata is best effort in MVP. The app may show OpenRouter or runtime-provided context limits, tool support, streaming support, pricing, route, source, and freshness timestamp when available. Stale, missing, or unknown pricing, capability, or route metadata is labeled but does not block session start when provider credentials and the selected model route are viable.

MVP model context sent through OpenRouter is bounded by disclosed
context-source categories rather than raw prompt capture. For OpenCode-backed
sessions, the app records selected model/routing metadata, active transcript
summary, approved or policy-allowed tool result summaries, explicitly read file
sources, runtime system/title-generation categories, and disclosed
runtime-native instruction sources such as root or nested `AGENTS.md` when
OpenCode loads them.

The app does not add its own hidden root `AGENTS.md` injection layer. OpenCode
runtime-native instruction loading is allowed only under the hardened adapter
policy: preflight inventory, user-visible disclosure, effective config capture,
and app-owned context-source persistence. Shell tool context uses only the
persisted bounded redacted/truncated summary, explicit output-omitted marker,
and safe output summary reason labels. Whole-repo indexing, hidden background
file ingestion by the app, implicit artifact ingestion, secret-deny file
contents, live raw terminal buffers, omitted raw shell output, redacted
substrings, sensitive raw byte counts, offsets, hashes, and reconstruction
metadata are excluded.

MVP uses standing disclosure rather than per-call context approval. Provider setup discloses that prompts and bounded context leave through OpenRouter, and session activity records bounded context-source summaries for model calls. Raw prompt export, token-by-token context inspection, per-call approval modals, and editable context composers are post-MVP.

OpenRouter or network outages fail closed for new model calls. The app preserves app-owned transcript, tool history, approvals, and artifacts, then allows explicit retry after connectivity or provider recovery. Offline model fallback, direct-provider fallback, automatic model switching, queued background resend, and synthetic assistant continuation are post-MVP or out of scope.

## Alternatives Considered

 - OpenRouter primary.
 - Direct provider calls.
 - OpenCode provider abstraction.

## Alternatives That Should Be Considered

 - Direct provider fallback for critical providers.
 - Local model provider support.
 - Provider-neutral model capability registry.
 - OpenRouter as optional gateway rather than required gateway.

## Tradeoffs

OpenRouter simplifies multi-provider access, routing, and model switching.

It introduces dependency on external routing behavior, model IDs, pricing, and policy changes.

Direct providers reduce intermediary lock-in but increase credential, routing, pricing, and capability complexity.

## Consequences

 - Provider-specific fields should be isolated from canonical model records.
 - SQLite stores only provider readiness state or keychain references, never raw OpenRouter keys.
 - Session start must capture a stable provider credential reference.
 - The Runtime Adapter must verify or enforce that the effective model matches the app-owned selected OpenRouter model.
 - The Runtime Adapter must make OpenRouter-bound context auditable through
   bounded source-category summaries and explicit disclosure of runtime-native
   instruction sources.
 - Model-call records need context-source summaries, not full raw prompt payload storage.
 - Users need visibility into data leaving the machine.
 - Basic model cost and routing information should be surfaced when available and labeled with source or freshness.
 - Provider and network failures need recoverable session state and explicit retry semantics.
 - Direct-provider migration remains a post-MVP risk.

## Follow-Up Questions

 - What OpenRouter routing and cost information is available through OpenCode?
 - Can OpenCode config override provider or model routing, and can the app detect or prevent it?
 - Which OpenCode runtime-native context categories must be disclosed for every
   OpenRouter-routed model call?
 - Can OpenCode expose context-source categories without requiring raw prompt capture?
 - What data-retention disclosures are required for OpenRouter-routed model calls?
 - What provider/network errors should be normalized for retry UX?
 - How does the runtime represent the provider credential reference used by a session without exposing secret material?
 - What migration path is needed if MVP validation users reject OpenRouter-only setup?

## ADR Recommendation

Validate platform keychain integration, routing visibility, cost disclosure, metadata freshness labeling, and privacy messaging before implementation planning.
