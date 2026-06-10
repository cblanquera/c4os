# Validation Decision Log

This log records decisions made during the validation loop.

## Template

### DECISION-YYYY-MM-DD-N: Title

Date: YYYY-MM-DD

Related findings:

 - FINDING-000

Related spikes:

 - `plans/spikes/SPIKE-000-name.md`

Evidence:

 - TBD

Decision:

TBD.

Rationale:

TBD.

ADRs impacted:

 - ADR-000

Follow-up updates:

 - TBD

Readiness impact:

 - BLOCKER/HIGH/MEDIUM resolved, reduced, unchanged, or escalated.

## Decisions

### DECISION-2026-06-10-13: Acceptance Gates Separated From Product QA

Date: 2026-06-10

Related findings:

 - FINDING-011

Related spikes:

 - Decision cleanup

Evidence:

 - `plans/validation/acceptance-gate-separation.md`
 - `plans/validation/evidence-matrix.md`

Decision:

Separate Phase 0 architecture viability gates from product QA acceptance before
freeze-and-plan work begins.

Rationale:

The validation loop has enough evidence to enter implementation planning, but
some checks belong to later app-build or release validation. Runtime control,
approval cooperation, OpenRouter routing, instruction disclosure, shell policy,
fallback ordering, and the MVP platform matrix are Phase 0 gates. Tauri UI
validation, implemented shell-policy tests, adapter tests, and user-facing
journeys are implementation or release gates.

ADRs impacted:

 - ADR-022
 - ADR-003
 - ADR-004
 - ADR-010
 - ADR-019

Follow-up updates:

 - Keep deferred app-build validation in implementation QA.
 - Keep product acceptance documents focused on implemented user behavior.

Readiness impact:

 - FINDING-011 resolved for implementation planning.

### DECISION-2026-06-10-12: MVP Tauri Platform Matrix Is macOS-First

Date: 2026-06-10

Related findings:

 - FINDING-009
 - FINDING-015

Related spikes:

 - `plans/spikes/SPIKE-006-tauri-platform-validation.md`

Evidence:

 - `plans/validation/FINDING-009-015-tauri-platform-matrix.md`
 - Official Tauri prerequisites: `https://v2.tauri.app/start/prerequisites/`
 - Official Tauri WebView versions: `https://v2.tauri.app/reference/webview-versions/`

Decision:

Use a macOS-first Tauri MVP platform matrix. macOS Apple Silicon is the
mandatory launch-validation target. Windows 11 x64 is required before public
MVP release. macOS Intel is optional when hardware is available. Linux is
deferred unless explicitly pulled into MVP beta scope.

Rationale:

The MVP validates a local desktop coding control center, not broad platform
distribution. A macOS-first implementation keeps Phase 0 narrow while still
testing Tauri's core shell, Rust backend, process supervision, keychain, file
dialog, approval, transcript, tool ledger, Git diff, and text-like artifact
flows. Windows remains important for release compatibility, but does not need
to block initial implementation planning. Linux WebKitGTK and packaging
variability would add support scope before the core thesis is validated.

ADRs impacted:

 - ADR-002
 - ADR-017
 - ADR-018

Follow-up updates:

 - Update ADR-002 with the accepted matrix.
 - Keep MVP UI and text-like artifact validation open until an app build or
   prototype exists.

Readiness impact:

 - FINDING-015 resolved.
 - FINDING-009 remains partially open pending app-build UI validation.

### DECISION-2026-06-10-11: Shell Security Policy Is Concrete Enough For Implementation

Date: 2026-06-10

Related findings:

 - FINDING-003
 - FINDING-002

Related spikes:

 - `plans/spikes/SPIKE-003-shell-security-policy.md`

Evidence:

 - `plans/validation/FINDING-003-shell-security-policy.md`
 - `plans/adr/010-filesystem-and-shell-access-defaults.md`
 - `plans/acceptance/shell-execution.md`
 - `plans/acceptance/security-and-permissions.md`

Decision:

Keep shell execution in MVP under the current-user, normal-network posture, but
require the concrete policy in `FINDING-003-shell-security-policy.md`.

Rationale:

The policy defines exact environment keep/strip/conditional rules, required
redaction matcher categories, truncation limits, safe reason labels, forbidden
persisted metadata, destructive-command categories, unclassifiable command
handling, and approval prompt disclosures. It preserves the broad ADR-010
decision while making the unresolved implementation details testable.

ADRs impacted:

 - ADR-010
 - ADR-009
 - ADR-004
 - ADR-019

Follow-up updates:

 - Update ADR-010 to point at this policy for exact shell limits.
 - Add implementation tests for environment filtering, redaction, truncation,
   output omission, destructive classification, and unclassifiable handling.

Readiness impact:

 - FINDING-003 policy gap resolved for planning.
 - Implementation still needs code-level tests for these mandatory rules.

### DECISION-2026-06-10-10: OpenRouter Routing Passes With Context-Boundary Caveat

Date: 2026-06-10

Related findings:

 - FINDING-004
 - FINDING-005

Related spikes:

 - `plans/spikes/SPIKE-002-openrouter-verification.md`

Evidence:

 - `tools/phase0/probe-opencode-openrouter-routing.mjs`
 - `plans/validation/FINDING-004-openrouter-runtime-verification.md`
 - Official OpenCode docs: `https://opencode.ai/docs/models`
 - Official OpenCode docs: `https://opencode.ai/docs/providers`
 - Official OpenRouter docs: `https://openrouter.ai/docs/api/reference/overview`

Decision:

OpenRouter provider routing, model matching, and starting credential-reference
stability pass for the tested hardened-adapter path. The original context
boundary claim does not pass as written.

Rationale:

The probe configured OpenCode with app-owned OpenRouter provider settings and
a fake local OpenRouter-compatible endpoint. A hostile project config attempted
to select OpenAI, but effective `/config`, model-switch events, and outbound
requests used the app-selected OpenRouter model. The captured request went to
`/api/v1/chat/completions`, used the selected model, and did not include an
alternate model list or explicit fallback route.

The probe changed the parent `OPENROUTER_API_KEY` after session creation. The
outbound request still used the starting key. This supports the MVP rule that
running sessions keep their starting credential reference, provided the app
blocks credential update/revoke while sessions are active.

Adapter caveat:

OpenCode `/config` returned the resolved provider API key. The app must treat
`/config` as sensitive, redact secrets immediately, and persist only app-owned
credential references.

Context caveat:

OpenCode sends runtime system prompt content and title-generation requests, and
previous instruction-loading evidence shows root/nested `AGENTS.md` can enter
model context. The MVP must update context-boundary language from strict
exclusion to disclosed bounded context-source categories for OpenCode-backed
sessions.

ADRs impacted:

 - ADR-003
 - ADR-013
 - ADR-019

Follow-up updates:

 - Update ADR-019/context acceptance wording to require disclosure and bounded
   summaries rather than strict exclusion of runtime-native contexts.
 - Add adapter tasks for `/config` redaction and session credential-reference
   capture.

Readiness impact:

 - Effective OpenRouter provider path passed.
 - Effective model match passed.
 - Credential reference stability passed.
 - OpenRouter context boundary failed as originally worded and needs scope
   wording cleanup before freeze.

### DECISION-2026-06-10-9: Select Hardened OpenCode Adapter Fallback

Date: 2026-06-10

Related findings:

 - FINDING-001
 - FINDING-005
 - FINDING-014

Related spikes:

 - `plans/spikes/SPIKE-001-opencode-runtime-control.md`
 - `plans/spikes/SPIKE-004-instruction-loading.md`
 - `plans/spikes/SPIKE-005-runtime-fallback-strategy.md`

Evidence:

 - `plans/validation/FINDING-001-runtime-config-isolation.md`
 - `plans/validation/FINDING-001-instruction-loading-observability.md`
 - `plans/validation/FINDING-014-runtime-fallback-decision.md`

Decision:

Select the hardened OpenCode adapter with mandatory instruction/config
disclosure as the MVP fallback path. Do not proceed with unconstrained direct
OpenCode, and do not claim strict runtime config isolation.

Rationale:

OpenCode passed the tested structured events, pre-execution interception,
structured denial, stop behavior, and app-owned record mapping gates. It failed
strict runtime config isolation because project config instructions still merge
into effective config. Instruction loading is not disabled, but it is
disclosable if the app performs preflight scanning and records effective
instruction sources.

The selected fallback preserves the MVP trust loop while avoiding open-ended
runtime replacement. It requires honest scope language: OpenCode-backed
sessions may include runtime-native instructions, and those sources must be
disclosed before session start.

Fallback order:

 - Hardened OpenCode adapter with mandatory disclosure.
 - App-controlled shadow workspace.
 - Tool proxy or runtime wrapper.
 - OpenCode fork.
 - Alternate runtime or direct provider mini-runtime.
 - MVP scope reduction.

ADRs impacted:

 - ADR-003
 - ADR-004
 - ADR-008
 - ADR-009
 - ADR-013
 - ADR-019

Follow-up updates:

 - Update ADR-003 fallback language.
 - Update ADR-013 and acceptance wording for OpenCode-backed instruction
   disclosure.
 - Add implementation tasks for instruction preflight scanning, `/config`
   capture, disclosure UI, and context-source persistence.

Readiness impact:

 - FINDING-014 resolved.
 - FINDING-001 is bounded by an accepted fallback path, but implementation
   freeze still waits on remaining high-risk prework.

### DECISION-2026-06-10-8: Instruction Loading Is Disclosable, Not Disabled

Date: 2026-06-10

Related findings:

 - FINDING-001
 - FINDING-005

Related spikes:

 - `plans/spikes/SPIKE-001-opencode-runtime-control.md`
 - `plans/spikes/SPIKE-004-instruction-loading.md`

Evidence:

 - `tools/phase0/probe-opencode-instruction-loading.mjs`
 - `plans/validation/FINDING-001-instruction-loading-observability.md`

Decision:

Mark instruction-loading observability as passed only with a mandatory
disclosure policy. Do not claim OpenCode-native instruction loading is
disabled.

Rationale:

The probe launched OpenCode with `--pure`, app-owned `instructions: []`, and
read/list/search tools denied. Project config `instructions` still appeared in
`/config`. Root and nested `AGENTS.md` contents affected assistant output
without tool events, permission events, or explicit file-read visibility.

This means direct OpenCode sessions cannot preserve the prior display-only
`AGENTS.md` claim. They can be made truthful only if the app preflights and
discloses instruction sources before session start.

Required MVP policy:

 - Scan and disclose `AGENTS.md` files from project root to session working
   directory.
 - Read `/config` and disclose config-provided instruction fields.
 - Store disclosed instruction sources in the app-owned session/context-source
   summary.
 - Block session start when instruction-bearing sources cannot be enumerated,
   classified, or disclosed.
 - Update MVP scope language so root `AGENTS.md` is not described as
   display-only for OpenCode-backed sessions.

ADRs impacted:

 - ADR-003
 - ADR-004
 - ADR-013
 - ADR-019

Follow-up updates:

 - Decide whether to accept disclosed native OpenCode instructions, use a
   shadow workspace, block projects with OpenCode instruction/config files, or
   choose a fallback runtime strategy.
 - Update acceptance wording if direct OpenCode remains selected.

Readiness impact:

 - Instruction-loading observability gate passed under disclosure policy.
 - FINDING-001 remains blocked because runtime config isolation failed.

### DECISION-2026-06-10-7: Runtime Config Isolation Fails On Project Instruction Bleed-Through

Date: 2026-06-10

Related findings:

 - FINDING-001
 - FINDING-004
 - FINDING-005

Related spikes:

 - `plans/spikes/SPIKE-001-opencode-runtime-control.md`

Evidence:

 - `tools/phase0/probe-opencode-config-isolation.mjs`
 - `plans/validation/FINDING-001-runtime-config-isolation.md`

Decision:

Do not mark runtime config isolation as passed for the current direct OpenCode
launch strategy.

Rationale:

The probe created hostile ambient OpenCode config that attempted to override
model, small model, permissions, and instructions. App-owned launch config
injected through `OPENCODE_CONFIG_CONTENT` protected the tested model and
permission values: the effective config used `opencode/big-pickle`, bash
remained `ask`, webfetch and external directory remained `deny`, and a hostile
`permission.bash: allow` did not bypass pre-execution approval.

However, project-level `instructions` still merged into effective config even
when OpenCode was launched directly with `opencode serve --pure` and app-owned
`instructions: []`. Because the gate requires existing OpenCode config not to
silently override provider, model, tool, permission, session, or instruction
behavior, the current evidence fails the gate as written.

ADRs impacted:

 - ADR-003
 - ADR-004
 - ADR-008
 - ADR-019

Follow-up updates:

 - Run instruction-loading observability next.
 - Decide whether MVP blocks on detected OpenCode instruction/config files,
   discloses them as part of the session contract, launches against an
   app-controlled shadow workspace, or uses a fallback runtime path.

Readiness impact:

 - Runtime config isolation gate failed.
 - FINDING-001 remains blocked.

### DECISION-2026-06-10-6: App-Owned Records Remain Canonical Over OpenCode References

Date: 2026-06-10

Related findings:

 - FINDING-001

Related spikes:

 - `plans/spikes/SPIKE-001-opencode-runtime-control.md`

Evidence:

 - `plans/specs/data-model-specification.md`
 - `plans/adr/008-unified-tool-invocation-and-ledger.md`
 - `plans/validation/FINDING-001-structured-events.md`
 - `plans/validation/FINDING-001-pre-execution-interception.md`
 - `plans/validation/FINDING-001-stop-behavior.md`
 - `plans/validation/FINDING-001-app-owned-record-mapping.md`

Decision:

The app-owned record mapping gate passes Phase 0. OpenCode session IDs,
message IDs, part IDs, call IDs, permission IDs, logs, and persistence paths
can be mapped into app-owned SQLite session, message, tool-call, approval, and
artifact records without becoming canonical.

Rationale:

The data model already establishes that app-owned SQLite records are the source
of truth. The live OpenCode probes provide the runtime references needed to
correlate sessions, messages, tool calls, permission requests, denials,
approvals, aborts, and terminal status updates. ADR-008 requires all
executable runtime actions to normalize into the unified tool ledger.

Implementation notes:

OpenCode identifiers must be stored only in explicit runtime reference fields
or metadata JSON. The app must generate canonical IDs. Duplicate or out-of-order
SSE events must update the same app-owned records through runtime-reference
idempotency keys. OpenCode `completed` shell output containing abort metadata
must map to app status `stopped`, and permission rejection must map to
canonical denial rather than generic failure.

ADRs impacted:

 - ADR-003
 - ADR-008
 - ADR-009

Follow-up updates:

 - Use the mapping as the runtime-adapter persistence contract.
 - Continue with runtime config isolation.
 - Continue with instruction-loading observability.

Readiness impact:

 - App-owned record mapping gate passed.
 - BLOCKER unchanged until remaining FINDING-001 gates pass or fallback is
   selected.

### DECISION-2026-06-10-5: Stop Behavior Passes With Adapter Mapping Caveat

Date: 2026-06-10

Related findings:

 - FINDING-001

Related spikes:

 - `plans/spikes/SPIKE-001-opencode-runtime-control.md`

Evidence:

 - `tools/phase0/probe-opencode-stop-behavior.mjs`
 - `plans/validation/FINDING-001-stop-behavior.md`

Decision:

OpenCode 1.17.1 stop behavior passes the Phase 0 runtime-control gate for the
tested path.

Rationale:

The probe approved a long-running shell command, confirmed the process was
alive, called `session.abort()`, then confirmed the process was no longer
alive. The command's signal trap fired, its natural completion marker was not
written, OpenCode emitted `MessageAbortedError`, and the session returned to
idle.

Adapter caveat:

OpenCode reported the shell tool state as `completed` while embedding shell
metadata that the user aborted the command. The app adapter must map this case
to canonical tool status `stopped`, not `succeeded`.

ADRs impacted:

 - ADR-003
 - ADR-008
 - ADR-009

Follow-up updates:

 - Use this behavior in the app-owned record mapping.
 - Add canonical status mapping for `MessageAbortedError` and shell abort
   metadata.

Readiness impact:

 - Stop behavior gate passed.
 - BLOCKER unchanged until remaining FINDING-001 gates pass or fallback is
   selected.

### DECISION-2026-06-10-4: Pre-Execution Interception Passes For Shell, Git, And File Writes

Date: 2026-06-10

Related findings:

 - FINDING-001
 - FINDING-002

Related spikes:

 - `plans/spikes/SPIKE-001-opencode-runtime-control.md`

Evidence:

 - `tools/phase0/probe-opencode-preexecution.mjs`
 - `plans/validation/FINDING-001-pre-execution-interception.md`

Decision:

OpenCode 1.17.1 provides pre-execution interception for the tested protected
action classes when permissions are configured as `ask`: shell commands,
runtime-proposed Git state changes through shell, and file writes through the
`write` tool.

Structured denial is also accepted as passed for these tested paths.

Rationale:

For all tested deny scenarios, the target side effect was absent before the
prompt, absent at `permission.asked` before the probe replied, and still absent
after replying `reject`. For all corresponding approve-once scenarios, the
target was absent at `permission.asked` and present only after replying `once`.

Rejected tool calls emitted `permission.replied` with `reply: "reject"` and a
`message.part.updated` tool state of `error`.

Implementation note:

OpenCode emits a tool state of `running` before permission is answered. The
adapter must treat `permission.asked` and final tool state as the enforcement
boundary, not the word `running`.

ADRs impacted:

 - ADR-003
 - ADR-004
 - ADR-008
 - ADR-009

Follow-up updates:

 - Continue to stop behavior and child-process termination proof.
 - Confirm app-owned record mapping for permission/tool/session IDs.
 - Confirm runtime config isolation so ambient config cannot silently change
   permission behavior.

Readiness impact:

 - Pre-execution interception gate passed.
 - Structured denial result gate passed.
 - BLOCKER unchanged until remaining FINDING-001 gates pass or fallback is
   selected.

### DECISION-2026-06-10-3: Structured Event Surface Passes Phase 0 Gate

Date: 2026-06-10

Related findings:

 - FINDING-001
 - FINDING-002

Related spikes:

 - `plans/spikes/SPIKE-001-opencode-runtime-control.md`

Evidence:

 - `plans/validation/FINDING-001-structured-events.md`
 - Official OpenCode server docs: `https://opencode.ai/docs/server/`
 - Official OpenCode SDK docs: `https://opencode.ai/docs/sdk/`
 - OpenCode generated SDK types:
   `https://github.com/anomalyco/opencode/blob/dev/packages/sdk/js/src/gen/types.gen.ts`

Decision:

OpenCode has a documented, typed, and locally observed structured event surface
through server-sent events and SDK event subscription. The `structured-events`
gate is passed.

Rationale:

The server docs describe `GET /event` as an SSE stream, the SDK exposes
`event.subscribe()`, and generated SDK types include structured event variants.
After local installation of OpenCode 1.17.1, the live probes observed
structured events for server connection, session creation/update, model switch,
user message, assistant message creation, assistant text streaming, session
status, session diff, tool pending/running/completed/error states, permission
ask/reply, approval, denial, abort errors, and idle recovery.

One important implementation note: OpenCode 1.17.1 emitted
`permission.asked`, while the generated SDK type list inspected earlier named
`permission.updated`. The runtime event stream should be treated as canonical
for adapter handling.

ADRs impacted:

 - ADR-003
 - ADR-004
 - ADR-008
 - ADR-009

Follow-up updates:

 - Use the observed event names and payloads when designing the runtime adapter.
 - Continue to pre-execution interception validation; structured events alone
   do not prove enforcement authority.
 - Keep FINDING-001 unresolved until observed event payloads and the remaining
   mandatory gates pass.

Readiness impact:

 - Structured-events gate passed.
 - BLOCKER unchanged until all FINDING-001 mandatory gates pass or fallback is
   selected.

### DECISION-2026-06-10-2: OpenCode Has A Supported Programmatic Integration Surface

Date: 2026-06-10

Related findings:

 - FINDING-001
 - FINDING-002
 - FINDING-004
 - FINDING-005

Related spikes:

 - `plans/spikes/SPIKE-001-opencode-runtime-control.md`

Evidence:

 - `plans/validation/FINDING-001-opencode-integration-surface.md`
 - Official OpenCode server docs: `https://opencode.ai/docs/server/`
 - Official OpenCode SDK docs: `https://opencode.ai/docs/sdk/`
 - Official OpenCode CLI docs: `https://opencode.ai/docs/cli/`
 - OpenCode generated SDK types:
   `https://github.com/anomalyco/opencode/blob/dev/packages/sdk/js/src/gen/types.gen.ts`

Decision:

OpenCode has a supported programmatic integration surface through a headless
HTTP server, OpenAPI 3.1, and the JS/TS SDK. This is enough to continue direct
OpenCode validation into structured-events and interception probes.

Rationale:

The official docs describe `opencode serve`, OpenAPI documentation at `/doc`,
SDK client creation, session/message/config/auth APIs, SSE event subscription,
permission response APIs, and generated event types. This proves that direct
integration is not limited to terminal scraping.

This does not prove MVP runtime-control viability. Pre-execution interception,
denial semantics, stop behavior, provider/model control, app-owned record
mapping, config isolation, and instruction-loading observability remain
unproven.

ADRs impacted:

 - ADR-003
 - ADR-004
 - ADR-008
 - ADR-009
 - ADR-019

Follow-up updates:

 - Run structured event proof.
 - Run permission/interception probes after a local OpenCode install is
   available.
 - Keep FINDING-001 unresolved until all mandatory gates pass or fallback is
   selected.

Readiness impact:

 - BLOCKER unchanged.

### DECISION-2026-06-10-1: Phase 0 Harness Does Not Resolve FINDING-001

Date: 2026-06-10

Related findings:

 - FINDING-001
 - FINDING-002
 - FINDING-004
 - FINDING-005

Related spikes:

 - `plans/spikes/SPIKE-001-opencode-runtime-control.md`

Evidence:

 - `tools/phase0/runtime-control-harness.mjs`
 - `tests/phase0-runtime-control.test.mjs`
 - `plans/validation/FINDING-001-phase0-harness-run.md`

Decision:

The Phase 0 harness is accepted as a validation scaffold only. It does not
resolve FINDING-001 and does not mark direct OpenCode as viable.

Rationale:

The test result proves that the evaluator requires all mandatory
runtime-control gates before recommending direct OpenCode viability. The test
does not include real OpenCode integration evidence for structured events,
pre-execution interception, denial handling, stop behavior, app-owned record
mapping, config isolation, or instruction-loading observability.

ADRs impacted:

 - ADR-003
 - ADR-004
 - ADR-008
 - ADR-009
 - ADR-019

Follow-up updates:

 - Collect concrete OpenCode evidence for each missing FINDING-001 gate.
 - Keep `plans/validation/evidence-matrix.md` runtime-control rows as not
   passed until source references or probe transcripts are recorded.
 - Enter the fallback decision tree if any mandatory gate fails.

Readiness impact:

 - BLOCKER unchanged.
