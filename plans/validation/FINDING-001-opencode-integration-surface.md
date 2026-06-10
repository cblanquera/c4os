# FINDING-001 OpenCode Integration Surface Evidence

Date: 2026-06-10

## Question

What supported OpenCode integration surface is available for MVP: library API,
server protocol, CLI subprocess protocol, filesystem state, fork, or another
path?

## Local Probe

Command:

```sh
command -v opencode
```

Result:

- Exit code: 1
- Interpretation: `opencode` is not installed in the current local shell, so no
  local runtime probe was performed in this pass.

## Documentation Evidence

Official OpenCode docs inspected on 2026-06-10:

- `https://opencode.ai/docs/server/`
- `https://opencode.ai/docs/sdk/`
- `https://opencode.ai/docs/cli/`
- `https://opencode.ai/docs/custom-tools/`
- `https://github.com/anomalyco/opencode`
- `https://github.com/anomalyco/opencode/blob/dev/packages/sdk/js/src/gen/types.gen.ts`

Observed supported surfaces:

1. Headless HTTP server.
   - `opencode serve` runs a headless HTTP server.
   - The server exposes an OpenAPI endpoint for clients.
   - Default host/port are `127.0.0.1:4096`.
   - The server can be protected with HTTP basic auth using
     `OPENCODE_SERVER_PASSWORD`.

2. OpenAPI 3.1 protocol.
   - The server publishes an OpenAPI 3.1 spec at `/doc`.
   - The spec is the basis for generated clients and request/response types.

3. JS/TS SDK.
   - `@opencode-ai/sdk` is documented as a type-safe client for the OpenCode
     server.
   - `createOpencode()` starts both a server and a client.
   - `createOpencodeClient()` connects to an existing server.
   - SDK APIs include sessions, messages, config, auth, files, TUI, and events.

4. CLI subprocess surface.
   - The CLI supports programmatic commands such as `opencode run`.
   - The CLI also supports `opencode serve`.
   - This is useful evidence of automation support, but CLI subprocess use is
     weaker than the HTTP/SDK surface for MVP control because it risks falling
     back to stdout/stderr observation.

5. Event stream surface.
   - The server exposes an SSE stream at `/event`.
   - The SDK exposes `event.subscribe()`.
   - Generated SDK types include event variants such as `server.connected`,
     `message.updated`, `message.part.updated`, `permission.updated`,
     `permission.replied`, `session.status`, `session.idle`, `file.edited`,
     `command.executed`, `pty.created`, `pty.updated`, and `pty.exited`.

6. Permission response surface.
   - The server documents `POST /session/:id/permissions/:permissionID` to
     respond to a permission request.
   - The generated SDK types include `permission.updated` and
     `permission.replied` events.

7. Tool-discovery surface.
   - The server documents experimental tool endpoints:
     `/experimental/tool/ids` and `/experimental/tool`.
   - Custom tools can be defined locally or globally, and a custom tool can
     replace a built-in tool by name.

## Classification

Status: supported, with limits.

OpenCode has a supported programmatic integration surface through a headless
HTTP server, OpenAPI 3.1, and the JS/TS SDK. This is enough to continue the
Phase 0 validation sequence into structured-events and permission/interception
probes.

This does not prove direct OpenCode is viable for MVP enforcement. The evidence
does not yet prove:

- pre-execution interception for file writes, shell commands, or Git state
  changes;
- denial semantics;
- stop behavior;
- provider/model override prevention;
- app-owned record mapping;
- config isolation;
- instruction-loading observability.

## Decision

Proceed to Workstream 1, Task 2: structured runtime event proof.

Do not mark FINDING-001 resolved. Only the integration-surface row has evidence.
