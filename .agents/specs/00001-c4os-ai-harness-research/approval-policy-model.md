# Approval Policy Model

State: Accepted direction; pure policy proof passed
Accepted: 2026-07-18

## Decision

C4OS will keep the 71 authority identities in `wireframes/r012-cleanup/` as an internal policy-scenario corpus, not as the permanent end-user settings schema.

The corpus remains useful for threat modeling, classifier fixtures, approval examples, adapter coverage, and discovering missing scenarios. It may grow without requiring a matching settings row for every scenario.

The current r012 wireframe remains unchanged. A later wireframe revision must replace its 71-row Advanced Policies presentation with the simplified model in this file. That work must happen in a new revision folder rather than updating `r012-cleanup` in place.

## Default presets

The Configuration screen should offer:

1. **Ask for approval** — prompt before actions that are not intrinsically read-only and trusted.
2. **Approve safe actions** — automatically approve low-risk, confidently classified actions within the trusted workspace; ask for broader or sensitive actions.
3. **Approve for me** — allow C4OS to resolve ordinary approval requests automatically inside the active sandbox and maximum-authority boundary.
4. **Custom** — use user-defined category rules and concrete exceptions.

“Approve for me” does not mean unrestricted access, disabled sandboxing, or bypassed managed policy. The UI must describe the active guardrails near the selector.

## User-facing policy groups

Advanced Policies should expose a compact set of understandable controls rather than every hypothetical identity:

| Group | Initial controls |
| --- | --- |
| Workspace files | Read, modify, delete, access outside the workspace. |
| Commands and processes | Inspect, execute inside the workspace, execute outside/system, start/control processes. |
| Version control | Local read, local change, remote read, remote publish or mutation. |
| Network and sharing | Retrieve, submit/publish, listen, upload or export local data. |
| Browser and desktop | View/capture, interact, use authenticated sessions, control desktop applications. |
| Credentials | Use securely, add/change/delete, reveal raw values. |
| Extensions and C4OS | Read metadata, use skills/plugins/MCP, install/configure, change C4OS policy, export artifacts. |

Each control supports `Use default`, `Allow`, `Ask`, and `Deny`. The exact row count and copy should be visually validated in the future wireframe revision.

## Internal action facts

Policy evaluation uses composable facts rather than one exhaustive identity string:

| Dimension | Example values |
| --- | --- |
| Surface | terminal, file, git, browser, network, credential, process, desktop, c4os |
| Effect | read, create, modify, delete, execute, control, publish, reveal, listen |
| Scope | workspace, external-local, remote, system |
| Initiator | user, agent, plugin, runtime |
| Sensitivity | ordinary, authenticated, credential, private |
| Reversibility | reversible, destructive, unknown |
| Confidence | known, ambiguous |

One action may produce multiple facts and match multiple rules. The policy engine evaluates the complete action, not only the runtime's native tool name.

## Resolution rules

1. Managed requirements and non-bypassable safety ceilings apply first.
2. Applicable `Deny` rules override `Ask` and `Allow` rules.
3. Any applicable `Ask` rule overrides `Allow`.
4. Ambiguous or unresolved effects use the appropriate unknown scenario and cannot be silently classified as safe.
5. A default preset supplies the result only when no more specific category rule or concrete exception decides it.
6. The most restrictive result wins when an action spans multiple categories, scopes, destinations, or effects.
7. Runtime-native permission rules are defense-in-depth; they are not the C4OS policy source of truth.

## Safety ceilings

An `Allow` rule remains bounded by the active sandbox, trusted roots, execution environment, managed requirements, and runtime/plugin authority. At minimum, C4OS must retain an ask-or-deny boundary for:

- unknown or unclassifiable actions;
- raw credential revelation;
- unresolved authenticated publishing or external sharing;
- destructive system actions;
- writes outside granted roots without an explicit scope;
- plugin or MCP behavior that exceeds its declaration; and
- actions prohibited by organizational policy.

## Concrete exceptions

Approval prompts may create narrow session or persistent exceptions, such as:

- allow `git status` in one workspace;
- allow reads under one additional directory;
- allow one MCP tool for one project; or
- allow uploads to one named host for the current session.

Exceptions record decision, operation/tool, matched action facts, normalized target scope, runtime or plugin identity where relevant, duration, creation source, and audit metadata. Advanced Policies shows only concrete exceptions that exist, with review, edit, and revoke actions.

Static category policies and prompt-created exceptions are separate records. A remembered approval must not silently broaden its path, command, host, plugin, runtime, or duration.

## Adapter requirement

`OCAdapter` and `PIAdapter` must emit a pre-execution action intent with native tool and arguments, resolved targets where available, runtime/workspace/session identity, execution environment, initiator, plugin/MCP involvement, credential/authenticated-session involvement, expected effects, and classification uncertainty.

C4OS classifies and evaluates the action. The adapter receives a single-use authorization bound to the exact runtime generation, tool call, arguments, and scope. The runtime does not persist the canonical remembered decision.

## Future wireframe revision

The next policy wireframe revision should:

- preserve the three existing default choices while renaming `Approve automatically` to `Approve for me`;
- add `Custom` when category rules or exceptions differ from the selected preset;
- replace the 71-row browser with the seven policy groups above;
- add an Exceptions view containing only concrete saved rules;
- explain the sandbox and safety ceiling for `Approve for me`;
- offer action-classification details from an approval or audit record, not as permanent settings rows; and
- preserve r012 as the historical accepted artifact.

## Evidence references

- `wireframes/r012-cleanup/script.js` — original nine groups, 71 identities, and four values.
- `proofs/approval-remember-policy/` — narrow session and persistent rule evidence.
- `proofs/approval-ui-flow/` — decision, resume, and settings-routing evidence.
- `proofs/user-directed-file-access-policy/` — initiator and trusted-root distinctions.
- `proofs/policy-classification-and-resolution/` — all 71 live r012 identities, presets, precedence, ceilings, exceptions, and single-use authorization.
