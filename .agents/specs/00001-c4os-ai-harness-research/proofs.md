# Existing Proof Applicability

The inventory below distinguishes historical feasibility evidence from the active, dated proof loop completed on 2026-07-18. Every claim remains bounded to its pinned dependencies, host platform, and acceptance criteria.

## Runtime and event model

| Proof | Applicable signal | Remaining gap |
| --- | --- | --- |
| `opencode-runtime` | Session lifecycle, event streaming, and abort are viable through an adapter. | Live provider credentials, permission flow, crash recovery, state isolation, and production packaging. |
| `pi-runtime` | Session, streaming, tool interception, denial, and abort are viable. | Artifact identity, sandbox ownership, persistence, secrets, packaging, and cross-platform behavior. |
| `pi-runtime-app-layer-proof` | App-layer approval denial/resume can preserve trace continuity. | Full C4OS state model and production runtime supervision. |
| `runtime-tool-discovery-without-plugin-view` | Runtime tool catalogs can hydrate independently of a visible plugin view. | Versioning, authorization, late failure, and remote runtime parity. |
| `tool-event-fanout` | One runtime event can feed multiple application consumers. | Backpressure, ordering across reconnects, and durable replay. |
| `prompt-tag-resolution` | Structured prompt references can resolve before runtime dispatch. | Complete file/browser/artifact identity rules and remote path semantics. |

## Approval, attachments, and policy

| Proof | Applicable signal | Remaining gap |
| --- | --- | --- |
| `approval-remember-policy` | Remembered approval decisions can be represented as policy. | Secure scope, expiry, revocation, remote execution, and audit requirements. |
| `approval-ui-flow` | An approval request can block and resume a user-visible flow. | Runtime-independent protocol, multi-window behavior, and recovery after restart. |
| `user-directed-file-access-policy` | User-selected paths can be distinguished from ambient filesystem authority. | Symlinks, moved roots, remote hosts, containers, and revoked grants. |
| `attachment-compatibility` | Attachment handling can be checked against model capability. | Provider drift, conversion policy, size limits, retry, and privacy. |
| `model-attachment-adapter` | Model-specific attachment conversion can sit behind an adapter. | Complete provider matrix and persistent artifact provenance. |
| `runtime-adapter-conformance` | Separate OCAdapter/PIAdapter manifests and pinned runtime evidence can carry versioned capability data. | Add model fixtures for modality, reasoning, tools, structured output, unknown metadata, contradictions, and effective send/model-switch preflight. |
| `runtime-tool-discovery-without-plugin-view` | Runtime tool inventory can exist independently of plugin presentation. | Resolve effective tool use as model tool-calling support plus runtime inventory, C4OS enablement, and policy. |

## Plugins, marketplaces, and skills

| Proof | Applicable signal | Remaining gap |
| --- | --- | --- |
| `extension-system` | Static manifest inventory, disabled/untrusted defaults, instruction-only skills, local MCP calls, and approval routing are viable. | Installation, signature/integrity, updates, remote MCP auth, hooks, and production trust. |
| `codex-marketplace-install-cache` | Metadata-first install, a C4OS-owned cache, uninstall removal, and reinstall from GitHub/ref are viable. | Marketplace schema compatibility, verification, ref mutability, offline behavior, update, and rollback. |
| `bundled-plugin-lifecycle` | Installed-disabled, enable, data-delete prompt, uninstall, and disabled reinstall states are representable. | Admin policy, dependency resolution, migration, revocation, and crash consistency. |
| `plugin-lifecycle-pending-restart-and-service-scope` | Live settings can be separated from restart-gated tools and shared/lazy services. | Process crashes, upgrades, dependency cycles, and cross-workspace isolation. |
| `plugin-migration-failure-handling` | Migration failures can map to recovery or disablement classes. | Transactional storage, backups, rollback, and multi-version fixtures. |
| `plugin-settings-renderer` | Schema-driven settings, conditional visibility, unknown-key warnings, reserved keys, and secret references are viable. | Full JSON Schema policy, migrations, accessibility, remote secrets, and malicious schemas. |
| `plugin-svg-sanitization` | Untrusted plugin artwork can be sanitized before display. | Complete media policy, decompression limits, raster handling, and cache poisoning. |
| `skills-settings-invalid-states` | Bundled, user, plugin, and project skills can remain metadata-only; invalid skills can expose repair reasons; only valid enabled skills enter suggestions/context. | Exact Agent Skills validator parity, precedence, collisions, workspace scope, and customization persistence. |

## Workspace and file semantics

| Proof | Applicable signal | Remaining gap |
| --- | --- | --- |
| `fs-workspace-file-and-relink` | Workspace identity can survive missing or moved local paths through relinking. | Remote/container roots, conflicts, symlinks, and concurrent changes. |
| `ide-file-operation-boundary` | Editor operations can be mediated instead of granting arbitrary filesystem mutation. | Complete command set, undo, conflict handling, permissions, and large files. |
| `project-and-chat-removal-semantics` | Removing application records can be separated from deleting user files. | Trash/recovery, shared data, remote roots, and plugin-owned data. |
| `project-chat-sharing-across-workspaces` | Chat/workspace relationships can be modeled without duplicating project data. | Authorization, concurrent moves, export, and sync. |

## Browser and document surfaces

| Proof | Applicable signal | Remaining gap |
| --- | --- | --- |
| `native-browser-tauri` | Negative evidence: the page could see `window.__TAURI_INTERNALS__`; secret command access was rejected. | A normal Tauri WebviewWindow did not satisfy the isolation criterion. |
| `native-browser-wry` | A raw-Wry alternative can create a narrower browser boundary. | Production navigation, downloads, profiles, permissions, crash recovery, and all target platforms. |
| `native-browser-plugin` | Browser behavior can be isolated behind a plugin boundary. | Native security architecture and production integration. |
| `browser-state-hydration-without-visible-view` | Browser state can exist without the view being mounted. | Durable recovery, multi-window synchronization, and crash semantics. |
| `browser-annotation-attachment-model` | Browser annotations can become structured prompt attachments. | Stable document identity, privacy, transforms, and runtime compatibility. |
| `browser-document-preview-boundary` | Previewable documents can be distinguished from privileged file operations. | Renderer hardening, active content, size limits, and unsupported formats. |

## Terminal and diagnostics

| Proof | Applicable signal | Remaining gap |
| --- | --- | --- |
| `native-terminal-tauri` | Rust PTY start/output/input/resize/cancel/cleanup and trusted-root approval are viable. | Tauri wiring, Windows/Linux, capability policy, backpressure, and production supervision. |
| `native-terminal-plugin` | Terminal behavior can be exposed through a plugin-style boundary. | Native host integration, trust, and process recovery. |
| `terminal-portable-pty` | A portable PTY abstraction can cover more than one host environment. | Full platform matrix and packaging. |
| `terminal-user-pty-lifecycle` | User terminal lifecycle can be separated from model command snapshots. | Persistence, reconnect, process trees, shell profiles, and remote terminals. |
| `terminal-xterm-pty-bridge` and `terminal-xterm-renderer` | xterm can render and bridge PTY data. | Accessibility, paste security, large-output pressure, and lifecycle recovery. |
| `terminal-inline-ui` | Inline terminal artifacts can coexist with chat. | Real PTY authority, persistence, and artifact provenance. |
| `shell-panel-resize-and-restore` | Panel size can be restored as user state. | Multi-window/workspace persistence and responsive constraints. |
| `chat-debug-redaction-history` | Diagnostic history can redact sensitive values. | Formal secret taxonomy, plugin/runtime fields, export policy, and retention. |

## Active Proof Loop

Approved: 2026-07-18

The user approved the complete loop. Each proof still records a bounded question and result; approval of the loop does not convert partial or unavailable platform evidence into a pass.

1. **Policy classification and resolution** — convert the 71 r012 identities into an extensible scenario corpus for composable action facts. Prove preset/category/exception precedence, multi-category most-restrictive resolution, unknown handling, user-directed scope, safety ceilings, and narrow remembered rules.
2. **Adapter conformance** — implement separate `OCAdapter` and `PIAdapter` expectation manifests against pinned current releases. Test the derived required C4OS baseline, action-intent emission, each adapter's declared optional capabilities, denial before tool side effects, session recovery, cancellation, crash/stale-event handling, and diagnostics. Compare Pi SDK-sidecar and Pi RPC only to select PIAdapter's transport, not to rank Pi against OpenCode.
3. **Tauri runtime supervisor** — prove signed/bundled sidecar discovery, loopback authentication, isolated state, health checks, restart limits, log redaction, shutdown, and version mismatch behavior.
4. **Codex compatibility fixtures** — validate a small matrix of skills-only, MCP/app, settings, and hook bundles plus layered `config.toml` fixtures against a pinned Codex release and the proposed C4OS importer. Cover user/project/profile precedence, untrusted-project suppression, machine-local key rejection, unknown fields, secrets, and managed-policy non-import. Record accepted, translated, rejected, and ignored fields.
5. **Marketplace trust and rollback** — prove immutable-source resolution, digest verification, metadata-before-code review, disabled install, cache ownership, update, failed migration rollback, uninstall, and revocation.
6. **Untrusted browser boundary** — prove that remote content cannot reach Tauri internals or C4OS commands, then cover profiles, downloads, navigation, popups, permissions, crash recovery, and supported platforms.
7. **Execution-environment parity** — run the same file, terminal, approval, and artifact journey in Local Desktop, Docker, and Remote SSH with explicit path and credential boundaries.

### Proof 1 — Policy classification and resolution

**Result: Passed (2026-07-18).** Seven deterministic tests passed. The proof compares its fixture keys directly with the unchanged r012 source, converts all 71 identities, and verifies presets, restrictive precedence, ceilings, explicit outside-root grants, resolved authenticated publishing, narrow expiring exceptions, and exact single-use authorization. This is application-layer evidence only; runtime enforcement and OS sandboxing remain outside this result.

- **Gap:** The accepted four-preset and seven-group model has not been exercised against all 71 r012 scenarios or cross-category actions.
- **Hypothesis:** Composable action facts plus safety ceilings, category rules, and narrow exceptions can deterministically resolve every fixture without making the legacy identity strings part of the permanent policy API.
- **Expected signal:** All 71 identities convert into known or explicitly ambiguous action facts; `Deny > Ask > Allow`; ceilings cannot be bypassed; unknown actions never silently allow; remembered exceptions remain bound to operation, target, runtime/plugin, and duration.
- **Failure signal:** A fixture disappears or duplicates, an ambiguous action resolves to allow, a broader exception matches, or an allow overrides a stricter applicable decision.
- **Scope:** Pure policy evaluation, legacy-fixture conversion, rule matching, and single-use decision evidence.
- **Non-goals:** Final UI copy, OS sandbox enforcement, runtime-native permission configuration, organizational policy distribution, or cryptographic authorization tokens.
- **Prototype:** `proofs/policy-classification-and-resolution/`

### Proof 2 — Adapter conformance

**Result: Passed for the tested adapter boundary (2026-07-18).** Eleven deterministic/current-package tests passed against pinned OpenCode `1.18.3` and Pi `0.80.10`. Actual Pi denied a current-package tool before its execute body. Actual OpenCode proved authenticated loopback control and emitted a real model-backed permission request; C4OS rejected it before the proposed file existed, and it remained absent. A diagnostic also established that per-request OpenCode tool overrides are authority-bearing and must not bypass OCAdapter policy. PIAdapter selects a C4OS Node SDK sidecar; RPC remains an available alternative, not a competing primary runtime.

- **Gap:** The separate OCAdapter and PIAdapter designs have not been tested against one C4OS-owned contract or pinned current runtime declarations.
- **Hypothesis:** Both peer adapters can satisfy the required baseline while declaring different optional capabilities and transports.
- **Expected signal:** Contract fixtures pass for lifecycle, events, action intent, denial-before-side-effect, cancellation, recovery, stale-event rejection, and diagnostics for both adapters.
- **Failure signal:** Either adapter requires canonical policy ownership, cannot block a tool before side effects, or cannot distinguish stale runtime events.
- **Scope:** Adapter contract and controlled fakes plus pinned upstream surface evidence where locally reproducible.
- **Non-goals:** Ranking runtimes, live billable model calls, production packaging, or full provider compatibility.
- **Prototype:** `proofs/runtime-adapter-conformance/`

### Proof 3 — Tauri runtime supervisor

**Result: Passed for the macOS supervisor and packaging boundary (2026-07-18).** Four real-process supervisor tests passed for canonical bundled discovery, SHA-256 verification, trust/version rejection, authenticated loopback health, isolated launch state, bounded restart, redacted logs, and parent/descendant shutdown. A separate real Tauri `2.11.4` build bundled and executed the target-qualified `externalBin`; ad-hoc signing passed strict deep bundle verification. Developer ID/notarization and Windows/Linux descendant cleanup remain target-specific release gates.

- **Gap:** Existing runtime proofs do not establish production-like sidecar supervision.
- **Hypothesis:** A Tauri-owned supervisor boundary can authenticate loopback traffic, isolate state, enforce version compatibility and restart limits, redact logs, and shut down cleanly.
- **Expected signal:** Controlled sidecars prove discovery, auth rejection, separate state roots, health/restart behavior, redaction, shutdown, and mismatch failure.
- **Failure signal:** Unauthenticated control, shared state leakage, unbounded restart, secret-bearing logs, orphan processes, or silent version mismatch.
- **Scope:** Supervisor process boundary and deterministic local fixtures.
- **Non-goals:** Code signing on every target OS, installer notarization, live runtime provider calls, or updater rollout.
- **Prototype:** `proofs/tauri-runtime-supervisor/`

### Proof 4 — Codex compatibility fixtures

**Result: Passed for declared subset (2026-07-18).** Eight tests passed against the official manual fetched that day and bundled `codex-cli 0.145.0-alpha.18`. The real strict parser and marketplace loader validated fixture behavior. The versioned one-way importer proves documented precedence, untrusted-project suppression, accepted/translated/rejected/ignored dispositions, environment-key handling, raw-secret and managed-policy rejection, and disabled-by-default plugin/hook intake. It does not claim full Codex parity or lossless round-trip export.

- **Gap:** “Codex-compatible” and `config.toml` import behavior are not yet a versioned contract.
- **Hypothesis:** A declared importer subset can classify bundle/config fields as accepted, translated, rejected, or ignored without importing machine-local secrets or managed policy.
- **Expected signal:** Pinned fixtures demonstrate deterministic precedence, trust suppression, unknown-key handling, secret references, and explicit compatibility results.
- **Failure signal:** Silent field loss, secret ingestion, managed-policy import, untrusted-project activation, or version-dependent ambiguity without diagnostics.
- **Scope:** Skills, MCP/app, settings, hook metadata, and layered config fixtures.
- **Non-goals:** Claiming full Codex implementation parity, installing marketplace code, or executing arbitrary hooks.
- **Prototype:** `proofs/codex-compatibility-fixtures/`

### Proof 5 — Marketplace trust and rollback

**Result: Passed for the local lifecycle and macOS executable-trust boundary (2026-07-18).** Five filesystem-backed lifecycle tests passed for immutable selectors, digest verification, metadata-before-code review, disabled install, transactional update, failed-migration rollback, uninstall, and revocation. Four additional tests passed with real Ed25519 origin/content verification and a real macOS hook sandbox: explicit trust, sanitized environment, denied network/outside writes, bounded output, timeout, process-group termination, and key/content revocation. Remote registry availability, selection of public signing authorities, moderation, billing, and Windows/Linux hook sandboxes remain outside the result.

- **Gap:** The install-cache proof does not cover immutable acquisition, verification, update failure, or revocation.
- **Hypothesis:** C4OS-owned acquisition can resolve immutable sources, verify digests, review metadata before activation, install disabled, roll back failed updates, uninstall, and revoke.
- **Expected signal:** Tampered content is rejected, failed migrations preserve the prior version, disabled-by-default is maintained, and revocation prevents activation.
- **Failure signal:** Mutable or mismatched content activates, rollback loses the prior version, uninstall leaves executable state, or revoked content runs.
- **Scope:** Local marketplace/cache fixtures and transactional lifecycle model.
- **Non-goals:** A public marketplace service, legal review, real signing authorities, billing, or moderation operations.
- **Prototype:** `proofs/marketplace-trust-and-rollback/` and `proofs/extension-trust-and-hooks/`

### Proof 6 — Untrusted browser boundary

**Result: Passed for the macOS isolation architecture; real permission UX remains feature-gated (2026-07-18).** The real raw-Wry hostile-page proof passed again and five controller tests passed for navigation, popups, profiles, modeled permissions, downloads, crash cleanup, and stale events. The safe architecture is raw Wry/native isolation with no registered page IPC handler. The normal Tauri `WebviewWindow` remains rejected. The pinned Wry `0.55.1` predates the merged unified permission handler, so camera/microphone and other real permission prompts still require a permission-capable Wry release plus target-specific UX evidence. C4OS must preserve browser/platform `Default` behavior rather than blanket-deny unavailable mediation.

- **Gap:** The Tauri WebviewWindow proof failed because remote content could see Tauri internals; raw Wry remains only promising evidence.
- **Hypothesis:** An isolated browser process/webview boundary can render untrusted content without exposing Tauri internals or C4OS commands and can mediate sensitive browser capabilities.
- **Expected signal:** Hostile fixtures cannot reach privileged globals or commands; navigation, popup, permission, download, profile, and crash behavior remain governed.
- **Failure signal:** Remote content observes/invokes privileged bridges, escapes profile boundaries, downloads without mediation, or recovers with stale authority.
- **Scope:** Locally reproducible hostile-content and boundary tests, plus explicit platform evidence limits.
- **Non-goals:** A production browser engine, exhaustive web-platform security audit, or unsupported-platform certification.
- **Prototype:** `proofs/untrusted-browser-boundary/`

### Proof 7 — Execution-environment parity

**Result: Passed for Local, Docker, and real loopback OpenSSH transports (2026-07-18).** Seven tests passed. Local Desktop performed a real file write and child cancellation. A digest-pinned, network-disabled Docker container performed the same write/reference journey and was cancelled and removed. An isolated real OpenSSH server used generated Ed25519 keys, strict host-key verification, credential references, a remote write, disconnect, and an explicit authenticated cancellation control channel. Each named external SSH host still requires host-specific validation before enablement.

- **Gap:** Local, Docker, and Remote SSH semantics have not been compared under one journey.
- **Hypothesis:** Environment adapters can preserve action-intent, approval, artifact, and cancellation semantics while making path and credential boundaries explicit.
- **Expected signal:** The same fixture journey produces equivalent contract events and environment-qualified identities in every available environment.
- **Failure signal:** Host paths/credentials leak, approvals bind to the wrong environment, artifacts lose provenance, or unavailable environments are presented as passed.
- **Scope:** Local execution plus Docker/SSH where safely available; deterministic simulations may validate contract logic but remain separately labeled.
- **Non-goals:** Cloud fleet orchestration, arbitrary SSH server administration, container image distribution, or cross-platform certification.
- **Prototype:** `proofs/execution-environment-parity/`

## Recommended order

The initial proof sequence is complete for the locally available macOS boundary. Next, convert the accepted decisions into an implementation plan and carry the proof invariants into production acceptance tests. Platform-specific browser, signing, hook-sandbox, and process-tree checks gate only the affected target/feature; they do not block unrelated implementation or research freeze.
