# User Journeys

This pass translates `r012-cleanup`, the adapter contracts, the approval model, and the completed Proof Loop into reviewable end-to-end behavior. It does not revise the wireframes; accepted reusable architecture is promoted separately into Context.

## Classification

Every numbered step carries two labels:

- Scope: `in-scope`, `out-of-scope`, `deferred`, `external`, or `unknown`.
- Implementability: `implementable`, `not-implementable`, `blocked`, or `evidence-only`.

“Implementable” means the current spec and evidence are sufficient to plan the behavior. It does not mean the behavior exists in `r012-cleanup` or production code.

## J-001 — Configure the first provider

- Actor: C4OS user
- Goal: Reach a usable start screen with one tested model provider.
- Trigger: Launch C4OS with no configured provider.
- Preconditions: The app can inspect provider configuration without exposing credentials to the renderer.
- Steps:
  1. `[in-scope · implementable]` C4OS detects that no provider is configured and opens the standalone Add Provider flow.
  2. `[in-scope · implementable]` The user selects a preset or OpenAI-compatible provider and supplies the required endpoint, authentication, and model fields.
  3. `[in-scope · blocked]` C4OS stores secret values in an OS-backed credential namespace; the exact keychain, fallback, reset, and export rules require GAP-010.
  4. `[external · blocked]` C4OS tests the provider connection and reports authentication, endpoint, model, or network errors without discarding the form.
  5. `[in-scope · implementable]` On success, C4OS records only redacted provider metadata and continues to the workspace start screen.
- Scope notes: Live provider authentication and model discovery are deferred by r012, but their product states are in scope.
- Implementable steps: Detection, form validation, redacted persistence boundaries, error states, and transition to Start.
- Evidence links: `research.md` §Product intent; `wireframes/r012-cleanup/specs.md`; GAP-010.
- Gaps created/resolved: No new gap; credential storage remains GAP-010.
- Acceptance impact: The onboarding UI is usable only when test failures preserve entered non-secret state and never echo secrets.

## J-002 — Open, recover, or add a workspace

- Actor: C4OS user
- Goal: Establish the trusted project root used by sessions and tools.
- Trigger: The user reaches Start or selects another project from the sidebar.
- Preconditions: At least one provider is configured.
- Steps:
  1. `[in-scope · implementable]` The user chooses Open folder, Open workspace, Clone Repository, or one of exactly three recent entries.
  2. `[in-scope · implementable]` A native picker or clone form resolves an explicit path; C4OS does not infer a broad ambient root.
  3. `[in-scope · implementable]` C4OS records the workspace identity and trusted-root grant before any runtime or tool acts on it.
  4. `[in-scope · implementable]` A missing recent path is shown as missing and offers Relocate, Copy Path, Rename, or Remove rather than opening silently.
  5. `[in-scope · implementable]` The workspace opens in Chat and restores its project/session list without starting a runtime turn.
  6. `[external · blocked]` Clone authentication uses credential references and approval-aware network execution; delivery rules remain GAP-010.
- Scope notes: Native pickers and real clone/filesystem behavior are deferred in r012 but required by the product journey.
- Implementable steps: Trusted-root establishment, missing-path recovery, navigation, and non-executing restore.
- Evidence links: `research.md` §Product intent; `approval-policy-model.md`; runtime adapter ownership tables.
- Gaps created/resolved: No new gap.
- Acceptance impact: No session, adapter, or plugin may expand the trusted root implicitly.

## J-003 — Start a chat with a peer runtime and execution environment

- Actor: C4OS user
- Goal: Create a chat bound to Pi or OpenCode and an execution environment.
- Trigger: The user selects New chat or sends the first valid prompt/attachment in a blank chat.
- Preconditions: A trusted workspace, provider, model, runtime default, and environment default exist.
- Steps:
  1. `[in-scope · implementable]` New chat opens a provisional blank thread that is not yet added to the sidebar.
  2. `[in-scope · implementable]` C4OS resolves the selected peer runtime through `OCAdapter` or `PIAdapter`; neither runtime is treated as primary.
  3. `[in-scope · implementable]` C4OS resolves Local, Docker, or Remote SSH as an environment-qualified execution target independent of the runtime brand.
  4. `[in-scope · implementable]` The first valid submission captures the chat's runtime/environment binding and persists the session; later default changes affect only new chats.
  5. `[in-scope · implementable]` The first prompt text or attachment name supplies the initial session title.
  6. `[deferred · blocked]` Explicit migration of an existing chat to another runtime/environment is unavailable until transcript, tool-state, and capability compatibility are defined.
- Scope notes: r012 exposes runtime selection in Settings, while the adapter model requires per-session identity. The activation boundary is not yet specified.
- Implementable steps: Provisional session behavior, peer adapter resolution, environment qualification, first-submit persistence, and initial naming.
- Evidence links: `opencode-adapter.md`; `pi-adapter.md`; `runtime-capability-matrix.md`; `wireframes/r012-cleanup/specs.md`.
- Gaps created/resolved: GAP-015 is resolved by accepted P-011.
- Acceptance impact: The UI must distinguish a default for new chats from the immutable or migratable binding of an existing chat.

## J-004 — Send a turn and receive a normalized response

- Actor: C4OS user
- Goal: Submit text and/or attachments and receive one coherent C4OS response.
- Trigger: Send from the Chat composer.
- Preconditions: The chat has a valid runtime/environment binding and provider/model reference.
- Steps:
  1. `[in-scope · implementable]` C4OS accepts prompt text, attachments, or an attachment-only submission and validates referenced paths against explicit grants.
  2. `[in-scope · implementable]` C4OS creates stable session, turn, run, and correlation identities before dispatch.
  3. `[in-scope · implementable]` The chosen adapter publishes its versioned runtime manifest and normalizes the selected provider/model capability evidence.
  4. `[in-scope · implementable]` Before dispatch, C4OS preflights attachments, requested response mode, reasoning controls, context limits, and tool needs against the effective model/runtime/policy intersection; incompatible content remains visible with a specific change-model, convert, remove, or continue-without-feature choice and is never silently dropped.
  5. `[in-scope · implementable]` C4OS snapshots the exact model/provider/endpoint/revision and effective capabilities for the run, then sends the normalized request.
  6. `[in-scope · implementable]` C4OS translates runtime-native deltas, reasoning summaries, tool requests, media, errors, and completion into normalized events and rejects stale or mismatched correlations.
  7. `[in-scope · implementable]` The transcript presents one response identified as C4OS using streaming, complete-response, Markdown, or media rendering as supported, while preserving runtime/model/environment provenance outside the assistant identity.
  8. `[in-scope · implementable]` Cancel aborts the active run and does not delete the chat, completed messages, terminal session, or existing artifacts.
- Scope notes: Backend responses and attachment persistence are deferred in r012; the product states are in scope.
- Implementable steps: The full normalized turn lifecycle is contract-ready; production packaging and provider integration remain separate work.
- Evidence links: `runtime-capability-matrix.md`; `model-capabilities.md`; `proofs.md` §Runtime adapter contract proof; `proofs.md` §Execution-environment parity proof.
- Gaps created/resolved: Provenance presentation is resolved by accepted P-014; model-dependent chat behavior is resolved by accepted P-018.
- Acceptance impact: Runtime- and model-specific optional features may enrich a turn but may not change baseline event meaning or disappear from a draft silently.

## J-005 — Approve, remember, deny, or constrain an action

- Actor: C4OS user or managed policy administrator
- Goal: Authorize only the intended action at the intended scope.
- Trigger: A runtime proposes an action through the C4OS tool boundary.
- Preconditions: The proposed action has a canonical identity, arguments, workspace, session, runtime, environment, and requested capability.
- Steps:
  1. `[in-scope · implementable]` C4OS classifies the action into one of the seven policy groups and applies maximum-authority, managed-policy, sandbox, and trusted-root constraints.
  2. `[in-scope · implementable]` The active preset (`Ask for approval`, `Approve safe actions`, `Approve for me`, or `Custom`) resolves to allow, ask, or deny; runtime-native approval cannot bypass this result.
  3. `[in-scope · implementable]` When interaction is required, the approval surface shows the concrete action, target, environment, risk, and proposed scope.
  4. `[in-scope · implementable]` The user may approve once, deny, or create a narrow remembered exception that cannot exceed higher constraints.
  5. `[in-scope · implementable]` C4OS issues a single-use authorization bound to canonical arguments and rejects replay, mutation, stale session use, or environment substitution.
  6. `[in-scope · implementable]` C4OS records the decision and result with secrets redacted; denial returns a normalized result without side effects.
- Scope notes: The 71 r012 identities remain test fixtures, not settings rows. The four-preset simplification belongs to a future wireframe revision.
- Implementable steps: All listed policy steps passed at the pure policy boundary; live runtime and OS enforcement retain their own proof requirements.
- Evidence links: `approval-policy-model.md`; `proofs.md` §Policy classification proof; P-007.
- Gaps created/resolved: Resolves the user-facing granularity question; does not resolve credential storage or OS sandbox implementation.
- Acceptance impact: “Approve for me” is never unrestricted access.

## J-006 — Read, edit, save, and focus file artifacts

- Actor: C4OS user
- Goal: Work with project files without giving the renderer or runtime ambient filesystem authority.
- Trigger: The user opens Files, selects a file/folder, or expands a file artifact.
- Preconditions: A trusted workspace exists; any outside path has a separate grant.
- Steps:
  1. `[in-scope · implementable]` A native picker or trusted explorer returns a capability-scoped path, not arbitrary renderer filesystem access.
  2. `[in-scope · implementable]` Reads are brokered through C4OS and rendered as a file or read-only explorer artifact.
  3. `[in-scope · implementable]` A proposed write is classified and approved before mutation, then applied atomically with conflict/error reporting.
  4. `[in-scope · implementable]` Direct artifact edits remain local until Save; Cancel restores the last persisted content.
  5. `[in-scope · implementable]` Expanding the artifact focuses it in the center workspace while keeping its originating chat context available.
  6. `[in-scope · implementable]` Reopening or updating the same logical file updates the artifact in place with auditable provenance.
- Scope notes: Real filesystem behavior is deferred in r012, but the artifact interaction is specified.
- Implementable steps: Capability-scoped reads, approval-gated writes, save/cancel semantics, and focus behavior.
- Evidence links: `research.md` §Product intent; `approval-policy-model.md`; `proofs.md` §Execution-environment parity proof.
- Gaps created/resolved: Provenance presentation is resolved by accepted P-014.
- Acceptance impact: Files mode may hide model/approval selectors, but policy enforcement remains active.

## J-007 — Run and recover a persistent terminal session

- Actor: C4OS user
- Goal: Use one persistent shell per chat while retaining C4OS authorization over commands.
- Trigger: A user or runtime requests terminal execution.
- Preconditions: The chat has an environment binding and the terminal capability is supported.
- Steps:
  1. `[in-scope · implementable]` C4OS creates or reuses one environment-qualified shell session for the chat.
  2. `[in-scope · implementable]` Each command request passes policy classification and single-use authorization before bytes reach the shell.
  3. `[in-scope · implementable]` Output streams through normalized events; each completed interaction creates a terminal snapshot artifact tied to the same shell identity.
  4. `[in-scope · implementable]` Stop interrupts only the active foreground long-running process, records interruption, and preserves the shell when the platform permits.
  5. `[in-scope · implementable]` Expanded Terminal accepts stdin in the terminal body while the fixed AI composer remains Chat input.
  6. `[in-scope · implementable]` Process exit, transport loss, or adapter failure produces an explicit terminal state and offers a fresh shell without falsifying continuity.
  7. `[deferred · not-implementable]` Full-screen terminal applications and password-entry flows remain outside the current wireframe contract.
- Scope notes: Real PTY and shell behavior are deferred in r012; full-screen apps are explicitly deferred.
- Implementable steps: Persistent identity, command approval, streaming/snapshots, bounded interruption, stdin separation, and honest recovery.
- Evidence links: `research.md` §Product intent; adapter contracts; `proofs.md` §Tauri sidecar supervisor proof.
- Gaps created/resolved: Remote transport recovery remains part of GAP-011.
- Acceptance impact: Replying to a terminal artifact always returns to Chat and never executes terminal input.

## J-008 — Browse untrusted content in an isolated surface

- Actor: C4OS user
- Goal: Navigate web content without exposing Tauri or C4OS authority to the page.
- Trigger: A browser action or browser artifact is opened.
- Preconditions: The platform provides a raw-Wry-equivalent or stronger isolated surface.
- Steps:
  1. `[in-scope · implementable]` C4OS validates navigation through a browser controller and opens it in a surface with no page-accessible Tauri/C4OS bridge.
  2. `[in-scope · implementable]` Back, forward, refresh, and address navigation update one browser artifact in place.
  3. `[in-scope · implementable]` New windows, custom schemes, downloads, file access, clipboard, camera, microphone, location, and authentication challenges use normal platform/browser behavior or a browser-like C4OS permission decision; unavailable C4OS mediation does not become blanket denial.
  4. `[in-scope · implementable]` Expanding the artifact focuses the isolated browser while retaining its originating chat context.
  5. `[in-scope · implementable]` Closing or navigating the artifact never grants the page access to chat state, credentials, filesystem handles, or runtime control.
  6. `[in-scope · blocked]` Windows and Linux must prove an equivalent boundary before the browser is production-ready on those platforms.
- Scope notes: r012 simulates browser behavior and defers the real untrusted browser.
- Implementable steps: macOS architecture and controller policy are evidenced; production capability integrations and other platforms remain blocked.
- Evidence links: P-006; `proofs.md` §Untrusted browser isolation proof.
- Gaps created/resolved: GAP-009 is resolved by accepted P-006; real capability UX remains feature-gated per target.
- Acceptance impact: A normal Tauri `WebviewWindow` is not an acceptable fallback for untrusted pages.

## J-009 — Recover from runtime, transport, provider, or version failure

- Actor: C4OS user
- Goal: Understand failure and resume without duplicated actions or corrupted session state.
- Trigger: Health check failure, adapter crash, lost transport, expired credentials, incompatible version, or app shutdown during a run.
- Preconditions: C4OS persisted session/run identities and owns supervisor state.
- Steps:
  1. `[in-scope · implementable]` The supervisor detects failure and marks the active run interrupted rather than completed.
  2. `[in-scope · implementable]` C4OS rejects late events from the failed generation and prevents replay of prior tool authorizations.
  3. `[in-scope · implementable]` Bounded restart restores only the adapter/service; session, transcript, artifacts, and audit state remain C4OS-owned.
  4. `[in-scope · implementable]` After provider or credential repair, Retry creates a new run attempt under the same immutable user turn and requires review if a prior effect has unknown completion.
  5. `[in-scope · implementable]` An incompatible runtime fails closed with the pinned version, detected version, capability mismatch, and required remediation.
  6. `[in-scope · implementable]` If recovery cannot preserve a terminal or browser instance, C4OS creates a visibly new instance and does not claim continuity.
- Scope notes: This journey combines recovery states distributed across the adapter, supervisor, provider, terminal, and browser contracts.
- Implementable steps: Detection, stale-event rejection, authorization invalidation, bounded restart, honest incompatibility, and continuity labeling.
- Evidence links: `proofs.md` §Tauri sidecar supervisor proof; both adapter lifecycle sections; GAP-012.
- Gaps created/resolved: GAP-017 is resolved by accepted P-013.
- Acceptance impact: Retry must never duplicate a previously authorized side effect silently.

## J-010 — Install and govern plugins, skills, and MCP servers

- Actor: C4OS user or managed policy administrator
- Goal: Add capabilities without transferring C4OS authority to extension code.
- Trigger: Open Plugins, Skills, or MCP Servers in Settings.
- Preconditions: C4OS can resolve immutable content, calculate a digest, and display declared capabilities.
- Steps:
  1. `[in-scope · implementable]` C4OS discovers metadata and shows origin, version, digest, declarations, permissions, and compatibility before installation.
  2. `[in-scope · implementable]` Installation verifies immutable content and stages it disabled in a C4OS-owned cache.
  3. `[in-scope · implementable]` Enabling executable hooks, MCP processes, or sensitive settings requires explicit trust, sandbox, secret, timeout, and revocation rules; the macOS hook boundary passed and other targets remain feature-gated.
  4. `[in-scope · implementable]` Skills are parsed and validated before enablement; invalid packages do not enter prompt context.
  5. `[in-scope · implementable]` Security revocations apply immediately; ordinary resources and tools activate at the next turn after validation; runtime/environment defaults affect only new chats.
  6. `[in-scope · implementable]` Update is transactional and preserves the prior version on failure; disable, uninstall, and revocation remove active availability without deleting audit history.
  7. `[deferred · blocked]` A public C4OS marketplace requires the signing/origin authority, availability, moderation, and compromise-response work deferred in GAP-008; user-added catalogs do not depend on that service.
- Scope notes: r012 renders these Settings flows but defers real installation, MCP execution, and config writes.
- Implementable steps: Local discovery, digest verification, disabled install, transactional rollback, uninstall, and revocation passed.
- Evidence links: P-005; P-009; `proofs.md` §Codex plugin marketplace lifecycle proof; GAP-006 through GAP-008.
- Gaps created/resolved: GAP-005 and GAP-006 are resolved by P-009 and P-005; GAP-007 maps to proposed P-015; GAP-008 is deferred to public-marketplace launch.
- Acceptance impact: Install does not imply enable, trust, execution, or access to active chats.

## J-011 — Deferred future Codex import

- Actor: C4OS user
- Goal: Reuse a declared Codex-compatible subset without claiming exact emulation.
- Trigger: A future spec explicitly reopens and accepts a Codex importer.
- Preconditions: The source layer and workspace trust state are known.
- Steps:
  1. `[out-of-scope · blocked]` The current product exposes no Codex import action or compatibility claim.
  2. `[deferred · implementable]` If intentionally reopened, the proven versioned subset can classify fields as accepted, translated, rejected, or ignored while excluding raw secrets and managed policy.
  3. `[out-of-scope · not-implementable]` Export back to Codex remains unsupported until lossless round-tripping is independently proven.
- Scope notes: Codex is standards research only under accepted P-008; this journey is retained as deferred evidence, not current product scope.
- Implementable steps: None in the current scope.
- Evidence links: P-008; `codex-config.md`; `proofs.md` §Codex compatibility subset proof.
- Gaps created/resolved: GAP-004 and GAP-014 are resolved by excluding Codex import from the current product scope.
- Acceptance impact: The UI must name the exact tested subset/version and show per-field diagnostics.

## J-012 — Execute the same baseline in Local, Docker, or Remote SSH

- Actor: C4OS user
- Goal: Preserve C4OS policy and event semantics when execution location changes.
- Trigger: A new chat resolves a Local, Docker, or Remote SSH environment.
- Preconditions: The runtime supports the chosen environment topology and C4OS can establish a qualified connection.
- Steps:
  1. `[in-scope · implementable]` C4OS resolves a stable environment identity, workspace mapping, runtime endpoint, and capability manifest before the first turn.
  2. `[in-scope · implementable]` Docker and OpenSSH transport fixtures passed connectivity, credential-reference, host-verification, workspace mapping, write, and cancellation checks; each named external host still requires its own validation.
  3. `[in-scope · implementable]` Every action request carries the environment identity through policy resolution and single-use authorization.
  4. `[in-scope · implementable]` Normalized lifecycle events keep the same meaning across environments while preserving environment-specific limitations.
  5. `[in-scope · implementable]` Compact runtime, environment/host, workspace, and source-run provenance appears in the chat header and at consequential approval, artifact, error, and audit surfaces.
  6. `[in-scope · implementable]` Disconnect or cancellation invalidates outstanding authorizations and marks affected runs/artifacts honestly.
  7. `[deferred · not-implementable]` Moving an existing active chat between environments remains intentionally unsupported; P-011 initially duplicates visible context into a new chat instead of claiming native-session migration.
- Scope notes: Local, a digest-pinned network-disabled Docker container, and a real isolated loopback OpenSSH server passed. External host profiles remain feature-scoped deployment gates.
- Implementable steps: Environment-qualified contracts, policy binding, event parity, capability differences, cancellation, and failure labeling.
- Evidence links: `runtime-capability-matrix.md`; `proofs.md` §Execution-environment parity proof; GAP-011.
- Gaps created/resolved: GAP-018 and GAP-015 are resolved by accepted P-014 and P-011; the shared GAP-011 transport proof passed, while named host validation remains deployment-specific.
- Acceptance impact: Parity means stable C4OS semantics, not identical native capabilities.

## Journey conclusions

- The end-to-end product remains coherent with peer `OCAdapter` and `PIAdapter` implementations under C4OS-owned policy, persistence, credentials, artifacts, and normalized events.
- The approval model is sufficiently detailed internally while the accepted four-preset UI avoids exposing the 71-scenario corpus as permanent settings structure.
- r012 covers the primary interaction language, but it does not yet establish activation timing, recovery semantics, or provenance visibility for the runtime architecture.
- Accepted P-010 through P-014 resolve GAP-003 and GAP-015 through GAP-018 as one coherent boundary between the Tauri host, C4OS services, adapters, and user-visible session state.
