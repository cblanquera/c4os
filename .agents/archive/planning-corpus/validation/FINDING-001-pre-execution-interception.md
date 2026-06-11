# FINDING-001 Pre-Execution Interception Evidence

Date: 2026-06-10

## Question

Can OpenCode prevent protected file writes, shell commands, and runtime-proposed
Git state changes from executing before backend approval?

## Probe

Probe script:

- `tools/phase0/probe-opencode-preexecution.mjs`

Command:

```sh
/Users/cblanquera/.nvm/versions/node/v22.14.0/bin/node tools/phase0/probe-opencode-preexecution.mjs
```

The probe starts an OpenCode 1.17.1 server in a disposable temp directory with:

- `permission.edit = ask`
- `permission.bash = ask`
- `permission.webfetch = ask`
- `permission.external_directory = deny`

For each scenario, the probe checks whether the target artifact exists:

1. before the prompt;
2. at the `permission.asked` event, before replying;
3. after the run completes or rejects.

## Scenarios

| Scenario | Protected action | Reply | Exists before prompt | Exists at permission request | Exists after run | Result |
| --- | --- | --- | --- | --- | --- | --- |
| `shell-deny` | `echo phase0-shell-denied > phase0-shell-denied.txt` | `reject` | false | false | false | Passed |
| `shell-allow` | `echo phase0-shell-allowed > phase0-shell-allowed.txt` | `once` | false | false | true | Passed |
| `git-deny` | `git init && git checkout -b phase0-denied-branch` | `reject` | false | false | false | Passed |
| `git-allow` | `git init && git checkout -b phase0-allowed-branch` | `once` | false | false | true | Passed |
| `file-write-deny` | `write` tool creates `phase0-edit-denied.txt` | `reject` | false | false | false | Passed |
| `file-write-allow` | `write` tool creates `phase0-edit-allowed.txt` | `once` | false | false | true | Passed |

## Representative Payloads

Shell denial request:

```json
{
  "type": "permission.asked",
  "properties": {
    "permission": "bash",
    "patterns": [
      "echo phase0-shell-denied > phase0-shell-denied.txt"
    ],
    "metadata": {
      "command": "echo phase0-shell-denied > phase0-shell-denied.txt",
      "description": "Write phase0-shell-denied to file"
    }
  },
  "targetExistsBeforeReply": false
}
```

Shell denial result:

```json
{
  "type": "message.part.updated",
  "properties": {
    "part": {
      "type": "tool",
      "tool": "bash",
      "state": {
        "status": "error",
        "error": "The user rejected permission to use this specific tool call."
      }
    }
  }
}
```

Git denial request:

```json
{
  "type": "permission.asked",
  "properties": {
    "permission": "bash",
    "patterns": [
      "git init",
      "git checkout -b phase0-denied-branch"
    ],
    "metadata": {
      "command": "git init && git checkout -b phase0-denied-branch",
      "description": "Initialize git repo and create branch"
    }
  },
  "targetExistsBeforeReply": false
}
```

File-write denial request:

```json
{
  "type": "permission.asked",
  "properties": {
    "permission": "edit",
    "patterns": [
      "private/var/folders/.../phase0-edit-denied.txt"
    ],
    "metadata": {
      "filepath": "/private/var/folders/.../phase0-edit-denied.txt",
      "diff": "+phase0-edit-denied"
    }
  },
  "targetExistsBeforeReply": false
}
```

File-write denial result:

```json
{
  "type": "message.part.updated",
  "properties": {
    "part": {
      "type": "tool",
      "tool": "write",
      "state": {
        "status": "error",
        "error": "The user rejected permission to use this specific tool call."
      }
    }
  }
}
```

Approval result:

```json
{
  "type": "message.part.updated",
  "properties": {
    "part": {
      "type": "tool",
      "tool": "write",
      "state": {
        "status": "completed",
        "output": "Wrote file successfully."
      }
    }
  }
}
```

## Findings

1. Shell execution waits for permission before executing.
   - The shell target did not exist at `permission.asked`.
   - Rejecting permission kept the target absent.
   - Approving once allowed the target to be created.

2. Git state changes through shell wait for permission before executing.
   - `.git` did not exist at `permission.asked`.
   - Rejecting permission kept `.git` absent.
   - Approving once allowed `git init` and branch creation to complete.

3. File writes through the `write` tool wait for permission before writing.
   - The write target did not exist at `permission.asked`.
   - Rejecting permission kept the target absent.
   - Approving once allowed the file to be written.

4. OpenCode emits `message.part.updated` with tool state `running` before
   permission is answered, but the protected side effect did not occur before
   the `permission.asked` event was handled. Adapter code must not treat
   `running` as proof of execution.

5. Rejection returns structured runtime state:
   - `permission.replied` with `reply: "reject"`;
   - tool state `error`;
   - stable error text: `The user rejected permission to use this specific tool call.`

## Decision

Mark these gates as passed:

- Pre-execution file-write interception.
- Pre-execution shell interception.
- Pre-execution Git state-change interception.
- Structured denial result.

FINDING-001 remains blocked until stop behavior, app-owned record mapping,
runtime config isolation, and instruction-loading observability are resolved.
