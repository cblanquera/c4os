# FINDING-001 Stop Behavior Evidence

Date: 2026-06-10

## Question

Can OpenCode stop active model/runtime work, terminate app-supervised child
processes, and preserve records and partial output status?

## Probe

Probe script:

- `tools/phase0/probe-opencode-stop-behavior.mjs`

Command:

```sh
/Users/cblanquera/.nvm/versions/node/v22.14.0/bin/node tools/phase0/probe-opencode-stop-behavior.mjs
```

The probe starts OpenCode 1.17.1 in a disposable temp directory with bash
permission set to `ask`. It approves one long-running shell command, waits for
the command to write its PID marker, calls `client.session.abort()`, and checks
whether the process is still alive.

Long-running command:

```sh
printf "$$" > phase0-long-running.pid; touch phase0-long-running.started; trap "echo trapped > phase0-long-running.trap; exit 0" TERM INT HUP; sleep 60; echo done > phase0-long-running.done
```

## Result

Observed process state:

```json
{
  "startedBeforeAbort": true,
  "doneBeforeAbort": false,
  "trapBeforeAbort": false,
  "aliveBeforeAbort": true,
  "aliveAfterAbort": false,
  "doneAfterAbort": false,
  "trapAfterAbort": true,
  "aliveAfterCleanup": false
}
```

Observed abort response:

```json
{
  "status": 200
}
```

Observed session error:

```json
{
  "name": "MessageAbortedError",
  "data": {
    "message": "Aborted"
  }
}
```

Observed tool output:

```text
(no output)

<shell_metadata>
User aborted the command
</shell_metadata>
```

Observed tool state:

```json
{
  "tool": "bash",
  "status": "completed",
  "input": {
    "command": "printf \"$$\" > phase0-long-running.pid; touch phase0-long-running.started; trap \"echo trapped > phase0-long-running.trap; exit 0\" TERM INT HUP; sleep 60; echo done > phase0-long-running.done",
    "description": "Run the long-running process command",
    "timeout": 120000
  },
  "output": "(no output)\n\n<shell_metadata>\nUser aborted the command\n</shell_metadata>"
}
```

Observed status events:

- `busy`
- `idle`
- `session.idle`

## Interpretation

1. `session.abort()` returned HTTP 200.
2. The shell process was alive before abort.
3. The shell process was not alive after abort.
4. The command's `TERM INT HUP` trap wrote `phase0-long-running.trap`, proving
   the child process received a termination signal.
5. The command did not finish naturally: `phase0-long-running.done` was absent
   after abort.
6. OpenCode emitted `session.error` with `MessageAbortedError`.
7. OpenCode returned the session to idle state.
8. OpenCode preserved tool/message records, but the shell tool state was
   `completed` with shell metadata saying `User aborted the command`.

## Adapter Requirements

The app adapter must not treat shell tool state `completed` as ordinary success
when the output contains OpenCode shell abort metadata or the session has a
corresponding abort/error event.

Recommended canonical app mapping:

- `session.error.name = MessageAbortedError` -> app session status `stopped`.
- shell output containing `<shell_metadata>User aborted the command</shell_metadata>` -> app tool status `stopped`, not `succeeded`.
- OpenCode `session.status: idle` after abort -> runtime settled, not proof of
  successful completion.

## Decision

Mark stop behavior as passed for Phase 0.

FINDING-001 remains blocked until app-owned record mapping, runtime config
isolation, and instruction-loading observability are resolved.
