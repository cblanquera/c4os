# FINDING-001 Structured Runtime Events Evidence

Date: 2026-06-10

## Question

Can OpenCode emit structured events for sessions, assistant output, model calls,
tool proposals, tool results, approvals, denials, errors, and stop/interruption
state without terminal scraping?

## Local Probe

Initial local check:

```sh
command -v opencode
```

Result:

- Exit code: 1
- Interpretation: OpenCode is not installed in the current shell.

Initial install attempts by Codex before the user installed packages:

```sh
npm install opencode-ai @opencode-ai/sdk
```

```sh
npm install opencode-ai @opencode-ai/sdk --fetch-timeout=30000 --fetch-retries=1 --no-audit --no-fund
```

Initial result:

- Both install attempts produced no output and did not create `node_modules` or
  `package-lock.json` within the bounded wait.
- Both npm processes were terminated to avoid leaving background work running.
- No live OpenCode server was started.
- No live SSE payload was observed in this environment.

The user then installed OpenCode locally:

- `opencode-ai` version: 1.17.1
- `@opencode-ai/sdk` version: 1.17.1
- `npx opencode --version`: 1.17.1

Codex used the machine-local preferred Node path from the `local-environment`
skill:

```sh
/Users/cblanquera/.nvm/versions/node/v22.14.0/bin/node --version
```

Result:

- v22.14.0

Server startup probe:

```sh
/usr/bin/env XDG_CONFIG_HOME=/private/tmp/c4os-opencode-probe/config XDG_DATA_HOME=/private/tmp/c4os-opencode-probe/data XDG_STATE_HOME=/private/tmp/c4os-opencode-probe/state XDG_CACHE_HOME=/private/tmp/c4os-opencode-probe/cache ./node_modules/.bin/opencode serve --hostname=127.0.0.1 --port=4196 --print-logs --log-level=DEBUG --pure
```

Sandboxed result:

- Failed with `ServeError`.

Escalated localhost result:

- Server started successfully and printed:
  `opencode server listening on http://127.0.0.1:4196`

Authenticated server note:

- Starting with `OPENCODE_SERVER_PASSWORD=phase0-event-probe` succeeded, but
  the client probe initially returned HTTP 401. The exact SDK auth convention
  was not resolved in this pass.
- For the local-only event payload probe, the server was restarted without
  `OPENCODE_SERVER_PASSWORD`, bound only to `127.0.0.1`.

Probe scripts:

- `tools/phase0/probe-opencode-events-client.mjs`
- `tools/phase0/probe-opencode-events.mjs`

No-reply command:

```sh
/Users/cblanquera/.nvm/versions/node/v22.14.0/bin/node tools/phase0/probe-opencode-events-client.mjs
```

Observed event types:

- `server.connected`
- `session.created`
- `plugin.added`
- `reference.updated`
- `session.next.agent.switched`
- `session.next.model.switched`
- `message.updated`
- `message.part.updated`
- `session.updated`

Reply-mode command:

```sh
/usr/bin/env OPENCODE_PROBE_NO_REPLY=false /Users/cblanquera/.nvm/versions/node/v22.14.0/bin/node tools/phase0/probe-opencode-events-client.mjs
```

Observed event types:

- `server.connected`
- `session.created`
- `plugin.added`
- `reference.updated`
- `session.next.agent.switched`
- `session.next.model.switched`
- `message.updated`
- `message.part.updated`
- `session.updated`
- `session.status`
- `session.diff`
- `message.part.delta`

Representative live payload excerpts:

```json
{
  "type": "session.created",
  "properties": {
    "sessionID": "ses_14e61cba1ffe93oXSJ14Psq8iN",
    "info": {
      "id": "ses_14e61cba1ffe93oXSJ14Psq8iN",
      "version": "1.17.1",
      "projectID": "global",
      "directory": "/private/var/folders/.../c4os-opencode-events-5h1SbH"
    }
  }
}
```

```json
{
  "type": "session.next.model.switched",
  "properties": {
    "sessionID": "ses_14e61cba1ffe93oXSJ14Psq8iN",
    "model": {
      "id": "big-pickle",
      "providerID": "opencode",
      "variant": "default"
    }
  }
}
```

```json
{
  "type": "message.updated",
  "properties": {
    "sessionID": "ses_14e611153ffeuwbyPiG7B5YCIu",
    "info": {
      "role": "assistant",
      "modelID": "big-pickle",
      "providerID": "opencode",
      "sessionID": "ses_14e611153ffeuwbyPiG7B5YCIu"
    }
  }
}
```

```json
{
  "type": "message.part.delta",
  "properties": {
    "sessionID": "ses_14e611153ffeuwbyPiG7B5YCIu",
    "messageID": "msg_eb19ef19c001e1ItXGybMRD8q2",
    "partID": "prt_eb19efb80001weSjRh7Hn2ulzH",
    "field": "text",
    "delta": "The"
  }
}
```

```json
{
  "type": "session.status",
  "properties": {
    "sessionID": "ses_14e611153ffeuwbyPiG7B5YCIu",
    "status": {
      "type": "busy"
    }
  }
}
```

## Documentation And Source Evidence

Official OpenCode docs and generated SDK source inspected on 2026-06-10:

- `https://opencode.ai/docs/server/`
- `https://opencode.ai/docs/sdk/`
- `https://github.com/anomalyco/opencode/blob/dev/packages/sdk/js/src/gen/types.gen.ts`

Source-backed findings:

1. Server-sent event stream exists.
   - The server docs document `GET /event`.
   - The endpoint is described as a server-sent events stream.
   - The first event is documented as `server.connected`, followed by bus
     events.

2. SDK event subscription exists.
   - The SDK docs expose `event.subscribe()`.
   - The documented SDK example iterates `events.stream` and reads
     `event.type` and `event.properties`.

3. Generated event types are structured.
   - The generated SDK type file defines named event variants with literal
     `type` strings and typed `properties`.
   - Event types found include:
     - `permission.updated`
     - `permission.replied`
     - `session.status`
     - `session.idle`
     - `session.compacted`
     - `file.edited`
     - `command.executed`
     - `session.created`
     - `session.updated`
     - `session.deleted`
     - `session.diff`
     - `session.error`
     - `file.watcher.updated`
     - `vcs.branch.updated`
     - `pty.created`
     - `pty.updated`
     - `pty.exited`

4. Some required categories are source-evidenced.
   - Session state is represented by `session.created`, `session.updated`,
     `session.status`, `session.idle`, `session.error`, and related events.
   - Permission state is represented by `permission.updated` and
     `permission.replied`.
   - File activity is represented by `file.edited` and
     `file.watcher.updated`.
   - Command activity is represented by `command.executed`.
   - PTY lifecycle is represented by `pty.created`, `pty.updated`, and
     `pty.exited`.

## Classification

Status: evidence recorded, not passed.

The docs, generated source, and local probe prove that OpenCode has a
structured SSE event surface with live payloads for server connection, session
creation/update, model switch, user message, assistant message creation,
assistant text streaming, session status, and session diff.

This still does not satisfy the full Task 2 done criteria because complete
coverage has not been observed for every required MVP event category:

- tool proposals;
- tool results;
- approval request timing;
- denial result timing;
- errors;
- stop/interruption state during active runtime work.

## Required Follow-Up Probe

Completed follow-up probe:

Probe script:

- `tools/phase0/probe-opencode-permission-stop.mjs`

Denial-mode command:

```sh
/Users/cblanquera/.nvm/versions/node/v22.14.0/bin/node tools/phase0/probe-opencode-permission-stop.mjs
```

Approval-mode command:

```sh
/usr/bin/env OPENCODE_PROBE_PERMISSION_RESPONSE=once /Users/cblanquera/.nvm/versions/node/v22.14.0/bin/node tools/phase0/probe-opencode-permission-stop.mjs
```

Observed denial-mode event types:

- `permission.asked`
- `permission.replied`
- `message.part.updated` with tool state `pending`
- `message.part.updated` with tool state `running`
- `message.part.updated` with tool state `error`
- `session.error` with `MessageAbortedError`
- `session.status` from `busy` to `idle`
- `session.idle`

Representative denial payloads:

```json
{
  "type": "permission.asked",
  "properties": {
    "id": "per_eb1a56791001N6GR1JfCgU6CU3",
    "sessionID": "ses_14e5aa8b7ffehxPkTcqlyGeueh",
    "permission": "bash",
    "patterns": [
      "echo phase0-denied > phase0-denied.txt"
    ],
    "metadata": {
      "command": "echo phase0-denied > phase0-denied.txt",
      "description": "Write phase0-denied to a file"
    },
    "tool": {
      "messageID": "msg_eb1a55b24001Qfht5UtiyN45n1",
      "callID": "call_00_m2YcRf6m2k4CIGgZRjNO5334"
    }
  }
}
```

```json
{
  "type": "permission.replied",
  "properties": {
    "sessionID": "ses_14e5aa8b7ffehxPkTcqlyGeueh",
    "requestID": "per_eb1a56791001N6GR1JfCgU6CU3",
    "reply": "reject"
  }
}
```

```json
{
  "type": "message.part.updated",
  "properties": {
    "part": {
      "type": "tool",
      "tool": "bash",
      "state": {
        "status": "error",
        "input": {
          "command": "echo phase0-denied > phase0-denied.txt",
          "description": "Write phase0-denied to a file"
        },
        "error": "The user rejected permission to use this specific tool call."
      }
    }
  }
}
```

Denial-mode outcome:

- Permission reply API returned HTTP 200.
- `phase0-denied.txt` did not exist after rejection.

Representative approval payload:

```json
{
  "type": "message.part.updated",
  "properties": {
    "part": {
      "type": "tool",
      "tool": "bash",
      "state": {
        "status": "completed",
        "input": {
          "command": "echo phase0-denied > phase0-denied.txt",
          "description": "Write phase0-denied to a file"
        },
        "output": "(no output)"
      }
    }
  }
}
```

Approval-mode outcome:

- Permission reply API returned HTTP 200 with response `once`.
- Tool state reached `completed`.
- The temp file existed only in the disposable probe directory and was removed
  by the probe cleanup.

Representative stop payloads:

```json
{
  "type": "session.error",
  "properties": {
    "sessionID": "ses_14e5a8575ffeYokT426J4hAGUi",
    "error": {
      "name": "MessageAbortedError",
      "data": {
        "message": "Aborted"
      }
    }
  }
}
```

```json
{
  "type": "message.updated",
  "properties": {
    "info": {
      "role": "assistant",
      "error": {
        "name": "MessageAbortedError",
        "data": {
          "message": "Aborted"
        }
      }
    }
  }
}
```

Stop-mode outcome:

- `client.session.abort()` returned HTTP 200.
- The stream emitted `session.error`, `session.status` `idle`, and
  `session.idle`.

## Decision

Mark `structured-events` as passed.

The current evidence proves live structured events for sessions, messages,
assistant output, model selection, status, tool proposal lifecycle,
tool-completion result, permission request/reply, denial result, error payloads,
and stop/interruption state.

FINDING-001 remains blocked because the other mandatory gates are not yet
resolved.
