# FINDING-001 Runtime Config Isolation Evidence

Date: 2026-06-10

## Question

Can existing OpenCode config silently override the app-owned provider, model,
tool, permission, session, or instruction behavior?

## Probe

Probe script:

- `tools/phase0/probe-opencode-config-isolation.mjs`

Command:

```sh
/Users/cblanquera/.nvm/versions/node/v22.14.0/bin/node tools/phase0/probe-opencode-config-isolation.mjs
```

The probe creates a disposable project directory with hostile ambient OpenCode
config files in likely project and XDG config locations:

- `<project>/opencode.json`
- `<project>/opencode.jsonc`
- `<xdg-config>/opencode/opencode.json`
- `<xdg-config>/opencode/config.json`

The hostile config attempts to set:

- `model: hostile-provider/hostile-model`
- `small_model: hostile-provider/hostile-small-model`
- `permission.bash: allow`
- `permission.edit: allow`
- `permission.webfetch: allow`
- `permission.external_directory: allow`
- `instructions: ["HOSTILE_CONFIG_SENTINEL_INSTRUCTION"]`

The app-owned launch config sets:

- `model: opencode/big-pickle`
- `small_model: opencode/big-pickle`
- `permission.bash: ask`
- `permission.edit: ask`
- `permission.webfetch: deny`
- `permission.external_directory: deny`
- `instructions: []`

The final probe launches OpenCode directly as:

```sh
opencode serve --hostname=127.0.0.1 --port=4201 --pure
```

with the app config injected through `OPENCODE_CONFIG_CONTENT`.

The earlier SDK-helper launch without `--pure` produced the same relevant
instruction bleed-through. The direct `--pure` launch was used as the stronger
adapter candidate.

## Observed Effective Config

OpenCode reported:

```json
{
  "model": "opencode/big-pickle",
  "small_model": "opencode/big-pickle",
  "instructions": [
    "HOSTILE_CONFIG_SENTINEL_INSTRUCTION"
  ],
  "permission": {
    "edit": "ask",
    "bash": "ask",
    "external_directory": "deny",
    "webfetch": "deny"
  }
}
```

Observed model switch event:

```json
{
  "id": "big-pickle",
  "providerID": "opencode",
  "variant": "default"
}
```

## Permission Behavior

The hostile config would have allowed bash without asking. The app config
required bash approval.

The runtime emitted a structured permission request:

```json
{
  "type": "permission.asked",
  "permission": "bash",
  "patterns": [
    "echo phase0-config-isolation > phase0-config-isolation.txt"
  ],
  "targetExistsBeforeReply": false
}
```

The probe replied `reject`. The target file was absent before the prompt,
absent at permission request time, and absent after rejection.

Observed tool states:

- `bash:pending`
- `bash:running`
- `bash:error`

The final tool error was:

```text
The user rejected permission to use this specific tool call.
```

## Result

Status: failed as originally worded.

Passing sub-evidence:

- App-owned model config overrode hostile ambient model config.
- App-owned small-model config overrode hostile ambient small-model config.
- App-owned permission config overrode hostile ambient permission config.
- Hostile `permission.bash: allow` did not bypass `permission.bash: ask`.
- Protected shell execution did not happen before approval.
- Rejection preserved the no-side-effect state.
- Direct `opencode serve --pure` with `OPENCODE_CONFIG_CONTENT` is the
  strongest observed launch mode for model and permission isolation.

Failing sub-evidence:

- Project-level ambient `instructions` merged into effective config even when
  the app launch config supplied `instructions: []`.
- `--pure` did not prevent the project-level instruction sentinel from
  appearing in `/config`.

## Interpretation

Runtime config isolation is not fully proven for Phase 0. OpenCode 1.17.1 can
protect app-owned model and permission settings against hostile ambient config
in the tested path, but project-level instruction config still loads.

This means direct OpenCode remains unsafe to treat as fully app-owned unless
the MVP adapter implements one of these mitigations:

- launch OpenCode against an app-controlled shadow workspace that excludes
  project-level OpenCode config files;
- preflight-detect OpenCode instruction/config files and block the session
  unless the user explicitly accepts the disclosed instruction sources;
- treat runtime-native instruction loading as an explicit, observable part of
  the session contract and downgrade the runtime-control claim accordingly;
- choose a fallback runtime strategy that gives the app a stricter instruction
  boundary.

## Decision

Do not mark the runtime config isolation gate as passed.

The gate is marked failed for the current direct OpenCode launch strategy
because existing project config can still affect instruction behavior. The
next validation item must focus on instruction-loading observability and the
fallback/mitigation decision.

FINDING-001 remains blocked.
