# Validation Evidence Matrix

This matrix tracks evidence required to resolve the remaining findings.

Status values:

 - Not started.
 - Investigating.
 - Evidence recorded.
 - Passed.
 - Failed.
 - Fallback required.
 - Deferred by explicit decision.

## Matrix

| Gate | Finding | Spike | Required Evidence | Status | Evidence Location | Decision |
| --- | --- | --- | --- | --- | --- | --- |
| External PoC review: Odysseus | FINDING-001 | SPIKE-001 | Review Odysseus only as a proof-of-concept reference for OpenCode-backed workspace integration. It may inform probe design but cannot resolve the blocker by itself. | Not started | https://github.com/pewdiepie-archdaemon/odysseus | Research input only |
| OpenCode integration surface | FINDING-001 | SPIKE-001 | Supported API/protocol/source evidence for direct runtime control. | Evidence recorded | `plans/validation/FINDING-001-opencode-integration-surface.md` | Supported HTTP/OpenAPI/SDK surface exists; continue to structured-events and interception probes before viability decision |
| Structured events | FINDING-001 | SPIKE-001 | Session, assistant, model, tool proposal, tool result, approval, denial, error, stop/interruption events without terminal scraping. | Passed | `plans/validation/FINDING-001-structured-events.md` | Live SSE payloads observed for session, model selection, user/assistant messages, assistant streaming, tool lifecycle, permission ask/reply, approval, denial, error, stop, status, idle, and diff |
| Pre-execution file-write interception | FINDING-001, FINDING-002 | SPIKE-001 | File write cannot execute before backend approval. | Passed | `plans/validation/FINDING-001-pre-execution-interception.md` | `write` tool target absent at permission request; reject kept it absent; approve-once created it |
| Pre-execution shell interception | FINDING-001, FINDING-002 | SPIKE-001 | Shell command cannot execute before backend approval. | Passed | `plans/validation/FINDING-001-pre-execution-interception.md` | Shell target absent at permission request; reject kept it absent; approve-once created it |
| Pre-execution Git state-change interception | FINDING-001, FINDING-002 | SPIKE-001 | Runtime-proposed Git state change cannot execute before backend approval. | Passed | `plans/validation/FINDING-001-pre-execution-interception.md` | `.git` absent at permission request; reject kept it absent; approve-once ran `git init` and branch creation |
| Structured denial result | FINDING-002 | SPIKE-001 | Denied or blocked action is returned to runtime as structured denial, not success or silent failure. | Passed | `plans/validation/FINDING-001-pre-execution-interception.md` | Reject emitted `permission.replied` with `reply: "reject"` and tool state `error` |
| Stop behavior | FINDING-001 | SPIKE-001 | Active runtime/model stream cancels; app-supervised child processes terminate; app-owned records survive. | Passed | `plans/validation/FINDING-001-stop-behavior.md` | `session.abort()` returned 200; long-running shell process alive before abort and dead after abort; TERM trap fired; session emitted `MessageAbortedError` and returned idle |
| App-owned record mapping | FINDING-001 | SPIKE-001 | Runtime IDs/logs/persistence map to app-owned session and tool records without becoming source of truth. | Passed | `plans/validation/FINDING-001-app-owned-record-mapping.md` | SQLite app records are canonical; OpenCode session, message, part, call, permission, log, and persistence references are adapter metadata only |
| Runtime config isolation | FINDING-001, FINDING-004, FINDING-005 | SPIKE-001 | Existing OpenCode config cannot silently override provider, model, tool, permission, session, or instruction behavior. | Failed | `plans/validation/FINDING-001-runtime-config-isolation.md` | App-owned model and permission config overrode hostile ambient config, but project-level `instructions` still merged into effective config even with `--pure` and `instructions: []` |
| Instruction-loading observability | FINDING-001, FINDING-005 | SPIKE-001, SPIKE-004 | Runtime-native instruction loading is disabled, observable, or disclosed. | Passed | `plans/validation/FINDING-001-instruction-loading-observability.md` | Direct OpenCode loads config instructions plus root/nested `AGENTS.md`; not disabled, but disclosable through `/config` plus mandatory app-side preflight scan |
| Effective OpenRouter provider path | FINDING-004 | SPIKE-002 | Runtime model calls use OpenRouter. | Passed | `plans/validation/FINDING-004-openrouter-runtime-verification.md` | Fake OpenRouter server received OpenCode `POST /api/v1/chat/completions` requests through configured OpenRouter provider path |
| Effective model match | FINDING-004 | SPIKE-002 | Runtime effective model matches app-owned selected model at session start and during run. | Passed | `plans/validation/FINDING-004-openrouter-runtime-verification.md` | Effective `/config`, model switch event, and outbound request model matched app-owned `openrouter/phase0/fake-openrouter-model` selection |
| Credential reference stability | FINDING-004 | SPIKE-002 | Running session keeps starting credential reference; key update/revoke cannot hot-swap it. | Passed | `plans/validation/FINDING-004-openrouter-runtime-verification.md` | Probe changed parent `OPENROUTER_API_KEY` after session creation; outbound request still used starting key; adapter must persist only keychain reference |
| OpenRouter context boundary | FINDING-004, FINDING-005 | SPIKE-002 | Context excludes whole-repo ingestion, hidden background files, automatic root `AGENTS.md`, implicit artifacts, secret-deny contents, and raw shell output. | Failed | `plans/validation/FINDING-004-openrouter-runtime-verification.md` | Original exclusion claim fails for OpenCode-backed sessions because runtime system/title contexts and root/nested `AGENTS.md` can enter model context; hardened adapter must disclose context-source categories instead |
| Instruction-source inventory | FINDING-005 | SPIKE-004 | All OpenCode instruction sources and trigger points are identified. | Evidence recorded | `plans/validation/FINDING-001-instruction-loading-observability.md` | Project config instructions, root `AGENTS.md`, and nested `AGENTS.md` were observed; tool defaults and full agent/persona config remain implementation-time inventory inputs |
| Instruction disable/observe/disclose path | FINDING-005 | SPIKE-004 | Runtime-native instruction loading is disabled, observable, or disclosed. | Passed | `plans/validation/FINDING-001-instruction-loading-observability.md` | Not disabled; acceptable only through mandatory preflight inventory and disclosure of effective OpenCode instruction sources |
| Fallback decision tree | FINDING-014 | SPIKE-005 | Accepted fallback order if direct OpenCode fails. | Passed | `plans/validation/FINDING-014-runtime-fallback-decision.md` | Select hardened OpenCode adapter with mandatory instruction/config disclosure as MVP fallback; shadow workspace, proxy, fork, runtime replacement, and scope reduction remain ordered fallback paths |
| Shell environment policy | FINDING-003 | SPIKE-003 | Exact keep/strip/conditional environment variable policy. | Passed | `plans/validation/FINDING-003-shell-security-policy.md` | Backend-built environment has explicit keep, conditional-keep, and strip lists |
| Shell redaction policy | FINDING-003 | SPIKE-003 | Exact matcher categories and known limitations. | Passed | `plans/validation/FINDING-003-shell-security-policy.md` | Required matcher categories and best-effort limitations are defined; failure omits output |
| Shell truncation policy | FINDING-003 | SPIKE-003 | Exact stdout/stderr/summary/line limits and omission rules. | Passed | `plans/validation/FINDING-003-shell-security-policy.md` | Persisted shell summaries are capped by bytes, lines, line length, and metadata limits; raw fallback is forbidden |
| Destructive command policy | FINDING-003 | SPIKE-003 | High-risk command categories, examples, non-examples, and one-time approval behavior. | Passed | `plans/validation/FINDING-003-shell-security-policy.md` | Destructive categories require fresh one-time approval and cannot use session allow |
| Unclassifiable command handling | FINDING-003 | SPIKE-003 | Conservative handling for shell indirection, encoded scripts, eval-like execution, aliases/functions, substitutions, broad globs, and nested interpreters. | Passed | `plans/validation/FINDING-003-shell-security-policy.md` | Opaque command shapes are conservative high risk; block when target scope cannot be determined |
| Tauri platform matrix | FINDING-015, FINDING-009 | SPIKE-006 | Mandatory, optional, and deferred launch platforms are named. | Passed | `plans/validation/FINDING-009-015-tauri-platform-matrix.md` | Mandatory launch validation is macOS Apple Silicon; Windows 11 x64 is pre-public-release compatibility validation; macOS Intel optional; Linux deferred |
| Tauri UI validation | FINDING-009 | SPIKE-006 | MVP UI and text-like artifact previews pass on mandatory platforms. | Not started | TBD | TBD |
| Acceptance gate separation | FINDING-011 | Decision cleanup | Phase 0 gates are separated from product acceptance before QA planning. | Passed | `plans/validation/acceptance-gate-separation.md` | Phase 0 architecture gates, accepted fallback/scope-change gates, deferred app-build validation, and product QA acceptance are separated before freeze-and-plan |

## Evidence Rules

Evidence must be concrete enough for an implementation-readiness reviewer to reproduce or trust the decision.

Acceptable evidence includes:

 - Source references.
 - Official documentation references.
 - Minimal probe command transcript.
 - Observed structured payload.
 - Config precedence matrix.
 - Failure-mode reproduction.
 - Explicit decision record with rationale.

Unacceptable evidence includes:

 - Assumption without source.
 - UI-only observation after execution.
 - Terminal scraping as proof of structured control.
 - Post-execution audit as proof of pre-execution control.
 - "Should be possible" statements without a tested or documented integration path.

## Harness Runs

| Date | Finding | Harness | Result | Evidence Location | Readiness Impact |
| --- | --- | --- | --- | --- | --- |
| 2026-06-10 | FINDING-001 | `tools/phase0/runtime-control-harness.mjs` | Harness tests passed, but no OpenCode gate evidence was supplied; evaluator returns `ready: false`. | `plans/validation/FINDING-001-phase0-harness-run.md` | BLOCKER unchanged |
