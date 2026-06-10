# FINDING-004 OpenRouter Runtime Verification

Date: 2026-06-10

## Question

Can OpenRouter-only model routing be enforced or observed at runtime so the app
does not show one selected provider/model while OpenCode sends prompts through
another route?

## Documentation Evidence

Official OpenCode docs inspected on 2026-06-10:

- `https://opencode.ai/docs/models`
- `https://opencode.ai/docs/providers`

Relevant source-backed facts:

- OpenCode model IDs use `provider_id/model_id`.
- OpenCode model loading priority starts with the `--model` flag, then config
  `model`, then last-used model, then an internal priority.
- OpenCode supports OpenRouter as a provider.
- OpenCode provider config can add OpenRouter models and provider-specific
  model options.
- OpenCode provider `baseURL` can be customized for proxy or custom endpoints.

Official OpenRouter docs inspected on 2026-06-10:

- `https://openrouter.ai/docs/api/reference/overview`
- `https://openrouter.ai/docs/api-reference/chat-completion`

Relevant source-backed facts:

- OpenRouter exposes an OpenAI-compatible chat completion API.
- Requests use `POST /api/v1/chat/completions`.
- Requests authenticate with `Authorization: Bearer <OPENROUTER_API_KEY>`.
- If `model` is omitted, OpenRouter uses the user or payer default.
- OpenRouter can perform fallback routing through request options; MVP must
  avoid implicit alternate model lists or fallback routes unless explicitly
  accepted later.
- Responses include normalized model and usage information; pricing/cost
  metadata is informational and model/provider freshness should be labeled.

## Local Probe

Probe script:

- `tools/phase0/probe-opencode-openrouter-routing.mjs`

Command:

```sh
/Users/cblanquera/.nvm/versions/node/v22.14.0/bin/node tools/phase0/probe-opencode-openrouter-routing.mjs
```

The probe starts:

1. a local fake OpenRouter-compatible HTTP server;
2. OpenCode 1.17.1 with `opencode serve --pure`;
3. app-owned OpenCode config injected through `OPENCODE_CONFIG_CONTENT`.

No real OpenRouter key is used. The fake server captures outbound OpenCode
requests and responds with a streaming OpenAI-compatible completion.

The disposable project contains hostile project config attempting to select:

```json
{
  "model": "openai/hostile-model",
  "small_model": "openai/hostile-small-model",
  "enabled_providers": ["openai"]
}
```

The app-owned launch config selects:

```json
{
  "model": "openrouter/phase0/fake-openrouter-model",
  "small_model": "openrouter/phase0/fake-openrouter-model",
  "enabled_providers": ["openrouter"],
  "disabled_providers": ["openai", "anthropic", "google", "gemini", "opencode"]
}
```

The app-owned OpenRouter provider config points `baseURL` at the fake local
OpenRouter server and resolves its API key from `OPENROUTER_API_KEY`.

## Observed Effective Runtime State

OpenCode `/config` reported:

```json
{
  "enabled_providers": ["openrouter"],
  "disabled_providers": ["openai", "anthropic", "google", "gemini", "opencode"],
  "model": "openrouter/phase0/fake-openrouter-model",
  "small_model": "openrouter/phase0/fake-openrouter-model"
}
```

Observed model switch event:

```json
{
  "id": "phase0/fake-openrouter-model",
  "providerID": "openrouter",
  "variant": "default"
}
```

Observed fake OpenRouter request:

```json
{
  "method": "POST",
  "url": "/api/v1/chat/completions",
  "body": {
    "model": "phase0/fake-openrouter-model",
    "stream": true
  }
}
```

The request used the starting OpenRouter key:

```text
Authorization: Bearer phase0-starting-api-key
```

The probe changed the parent process `OPENROUTER_API_KEY` after session
creation. The fake OpenRouter request still used the starting key, not the
changed key.

## Results

Mark these gates as passed:

- Effective OpenRouter provider path.
- Effective model match.
- Credential reference stability.

The local fake OpenRouter server received OpenCode chat-completion requests at
the OpenRouter-compatible endpoint. The request model matched the app-selected
model. OpenCode did not use the hostile project-selected OpenAI provider/model
in the tested path.

No alternate OpenRouter model list or explicit `route: fallback` field was
observed in the captured request.

## Credential Handling Caveat

OpenCode `/config` returned the resolved API key in `provider.openrouter`.
This is acceptable for the local probe only because the key was fake.

MVP adapter requirement:

- Treat `/config` responses as sensitive.
- Redact provider secrets before logging, storing, displaying, or sending to
  model context.
- Persist only an app-owned credential reference such as a keychain item ID,
  never raw API key material.
- Capture the session credential reference at session start.
- Block credential update or revoke while a session is running.

## Context Boundary Result

Do not mark the OpenRouter context-boundary gate as passed.

The probe proves the captured provider request can be summarized without
persisting raw prompt payloads, but it also shows OpenCode sends runtime system
prompt content and a title-generation model call. Prior instruction-loading
evidence proves root and nested `AGENTS.md` can enter model context without
tool-read events.

The MVP must therefore use the hardened-adapter disclosure policy:

- disclose runtime-native instruction sources;
- store bounded context-source summaries;
- exclude raw prompt exports from app-owned records;
- treat OpenCode system prompts, title-generation calls, tools schemas, and
  disclosed `AGENTS.md` sources as runtime context categories.

The original boundary claim that context excludes automatic root `AGENTS.md`
does not hold for OpenCode-backed sessions.

## Decision

OpenRouter-only remains acceptable for MVP only through the hardened OpenCode
adapter path:

- selected model is stored as `openrouter/<model-id>`;
- effective `/config` model and model switch events must match app-owned
  session model;
- outbound provider requests must use OpenRouter-compatible chat completions;
- no alternate `models` list or `route: fallback` is allowed in MVP;
- `/config` must be redacted before persistence;
- credential references are app-owned and stable for the session.

FINDING-004 is reduced but not fully closed until the context-boundary wording
is updated to match the hardened adapter disclosure policy.
