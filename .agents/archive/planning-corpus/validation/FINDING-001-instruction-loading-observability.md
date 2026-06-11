# FINDING-001 Instruction-Loading Observability Evidence

Date: 2026-06-10

## Question

Is OpenCode-native instruction loading disabled, observable, or disclosable
well enough for MVP sessions?

## Probe

Probe script:

- `tools/phase0/probe-opencode-instruction-loading.mjs`

Command:

```sh
/Users/cblanquera/.nvm/versions/node/v22.14.0/bin/node tools/phase0/probe-opencode-instruction-loading.mjs
```

The probe launches OpenCode 1.17.1 directly as:

```sh
opencode serve --hostname=127.0.0.1 --port=4202 --pure
```

with app-owned config injected through `OPENCODE_CONFIG_CONTENT`.

The app-owned config sets `instructions: []` and denies read, list, grep,
glob, bash, edit, webfetch, and external-directory permissions.

The disposable project contains these sentinel instruction sources:

- `<project>/opencode.json`
- `<project>/AGENTS.md`
- `<project>/nested/AGENTS.md`

The probe checks effective config at the root and nested directories, then
runs assistant prompts from each directory while watching for tool or
permission events.

## Observed Effective Config

Root and nested `/config` responses both included the project config
instruction sentinel:

```json
{
  "instructions": [
    "PHASE0_CONFIG_INSTRUCTION_SENTINEL"
  ],
  "permission": {
    "read": "deny",
    "edit": "deny",
    "glob": "deny",
    "grep": "deny",
    "list": "deny",
    "bash": "deny",
    "external_directory": "deny",
    "webfetch": "deny"
  }
}
```

Interpretation:

- OpenCode project config instructions are observable through `/config`.
- App-supplied `instructions: []` does not disable project config
  instructions.

## Observed Root AGENTS.md Behavior

Root run summary:

```json
{
  "containsConfigInstruction": false,
  "containsRootAgents": true,
  "containsNestedAgents": false,
  "toolCount": 0,
  "permissionCount": 0,
  "sessionErrors": []
}
```

The assistant response referenced the root `AGENTS.md` instruction and included
the root sentinel even though read/list/search permissions were denied and no
tool events were emitted.

Interpretation:

- Root `AGENTS.md` is loaded into model context outside normal tool-read
  visibility.
- This contradicts the prior MVP claim that root `AGENTS.md` is display-only
  when using direct OpenCode against the project directory.

## Observed Nested AGENTS.md Behavior

Nested run summary:

```json
{
  "containsConfigInstruction": false,
  "containsRootAgents": true,
  "containsNestedAgents": true,
  "toolCount": 0,
  "permissionCount": 0,
  "sessionErrors": []
}
```

The assistant response referenced both root and nested `AGENTS.md` contents
without tool-read events.

Interpretation:

- Nested `AGENTS.md` is also loaded into model context outside normal
  tool-read visibility.
- The nested run saw both root and nested instructions.
- Full OpenCode instruction precedence cannot be treated as absent in MVP.

## Event Visibility

Observed event types included assistant/session/model lifecycle events such as:

- `session.created`
- `session.next.agent.switched`
- `session.next.model.switched`
- `message.updated`
- `message.part.updated`
- `message.part.delta`
- `session.status`
- `session.diff`

Observed event types did not include structured file-read or instruction-load
events for `AGENTS.md`.

There were no tool events and no permission events in either prompt run.

## Instruction Source Inventory For MVP

| Source | Observed by probe | Runtime effect | App observability path |
| --- | --- | --- | --- |
| Project `opencode.json` `instructions` | Yes | Appears in effective config | `/config` plus app-side config-file scan |
| App `OPENCODE_CONFIG_CONTENT.instructions` | Yes | Does not erase project config instructions | App launch record plus `/config` comparison |
| Root `AGENTS.md` | Yes | Enters model context | App-side preflight file scan; not visible as tool read |
| Nested `AGENTS.md` | Yes | Enters model context | App-side preflight file scan from project root to session cwd; not visible as tool read |
| Tool descriptions/runtime defaults | Implied by runtime | Part of runtime behavior | Not fully inventoried by this probe |
| Agent/persona config | Typed in SDK config schema | Possible via config | `/config` plus app-side config-file scan |

## Decision

Mark instruction-loading observability as passed only under a disclosure-based
MVP policy.

Instruction loading is not disabled. It is partially observable through
OpenCode `/config` and fully disclosable for MVP only if the app performs its
own preflight instruction-source inventory before session start.

Required MVP policy if direct OpenCode remains selected:

- Do not claim root `AGENTS.md` is display-only for OpenCode-backed sessions.
- Before session start, scan and disclose all `AGENTS.md` files from project
  root to session working directory.
- Read effective `/config` and disclose config-provided `instructions`, agent
  prompts, and other instruction-bearing config fields.
- Record disclosed instruction sources in the app-owned session/context-source
  summary.
- Block session start if the app detects an instruction-bearing source it
  cannot enumerate, classify, or disclose.
- Keep full instruction editing, precedence visualization, and compatibility
  management out of MVP unless a later ADR explicitly expands scope.

## Readiness Impact

The instruction-loading observability gate passes with the disclosure policy
above.

FINDING-001 remains blocked because runtime config isolation already failed for
the current direct OpenCode launch strategy. The next decision must choose a
mitigation or fallback:

- accept disclosed native OpenCode instructions and update MVP scope language;
- use an app-controlled shadow workspace to prevent project-level config and
  instruction bleed-through;
- block projects with OpenCode instruction/config files in MVP;
- choose a fallback runtime strategy with a stricter instruction boundary.
