# Status

## Lifecycle

- Phase: Research complete
- Freeze state: Frozen 2026-07-18 after P-018 acceptance
- Last updated: 2026-07-18
- Context promotion: Complete for accepted P-001 through P-018 in `context/runtime-session-architecture.md` where reusable; P-008 records an explicit compatibility non-goal.
- Classification: Research-only. Post-Freeze clarification accepted 2026-07-18; implementation details require a new spec.

## Work items

| Work item | State | Notes |
| --- | --- | --- |
| Read r012 functionality | Complete | Read wireframe specification, implementation structure, QA notes, and representative rendered QA captures. |
| Inventory existing proofs | Complete | Applicability and remaining gaps are recorded in `proofs.md`; the active seven-part loop and supporting proofs were rerun on 2026-07-18. |
| Research Tauri | Complete for initial pass | Architecture, capabilities, sidecars, shell permissions, and updater security reviewed. |
| Research Agent Skills | Complete for initial pass | Current public `SKILL.md` schema and progressive-loading model reviewed. |
| Research Codex plugins and marketplaces | Complete; subset proof passed | Bundled Codex 0.145 loader validated metadata-only discovery; trust/update/rollback remain in the marketplace proof. |
| Research Codex `config.toml` | Complete; subset proof passed | Versioned one-way importer passed precedence, trust, secret, unknown-key, and managed-policy fixtures against the current strict parser. |
| Research Jan and Atomic Chat | Complete for initial pass | Current source inspected; their OpenCode relationships are materially different. |
| Research comparable runtime hosts | Complete for initial pass | OpenWork, OpenChamber, current OpenCode desktop, and Pi SDK/RPC reviewed. |
| Draft `OCAdapter` | Complete; tested boundary passed | OpenCode 1.18.3 authenticated control plane and real model-backed permission rejection passed. Per-request tool overrides are treated as authority-bearing. |
| Draft `PIAdapter` | Complete; proof passed for tested boundary | Pi 0.80.10 denied before execute. C4OS Node SDK sidecar selected; RPC retained as an alternative. Packaging remains separate. |
| Adapter and plugin architecture review | Accepted | P-002 through P-005 establish C4OS-owned policy, managed OpenCode, the Pi Node SDK sidecar, and declarative plugin bundles. |
| Derive runtime capability matrix | Complete; tested boundary passed | Both peer adapters passed the common contract and current-package denial-before-side-effect checks; optional capabilities remain explicit. |
| Simplify approval policy model | Accepted; proof passed | The pure policy boundary passed 7 tests against all 71 live r012 fixture identities. Runtime enforcement and OS sandboxing remain separate proofs. |
| User-journey pass | Complete | Twelve material journeys cover onboarding, workspaces, peer runtime binding, turns, approvals, artifacts, recovery, extensions, execution environments, and the explicitly deferred Codex-import path. |
| Journey-created decision review | Accepted | P-010 through P-014 establish the Rust-authoritative split plane, first-submit chat binding, explicit configuration activation, safe Retry semantics, and compact provenance. |
| Final architecture review | Accepted | P-015 through P-017 establish skill resolution, credential/runtime-state isolation, and independent recoverable update channels. |
| Model-capability research | Complete; P-018 accepted | Provider/runtime declarations, the four-layer C4OS normalization boundary, effective-capability rules, chat-session effects, wireframe implications, and proof reuse are recorded in `model-capabilities.md`. |
| Open-source runtime/capability comparison | Complete; informs P-018 | ACP, OpenCode ACP, pi-acp, Goose, OpenHands, Cline, Cherry Studio, and Continue were compared; native peer adapters remain recommended, with ACP as an optional compatibility surface. |
| New Proof Loop | Complete for locally available evidence | All seven approved proof areas have executable artifacts and dated results. Local/macOS boundaries passed, including OpenCode/Pi denial, Tauri `externalBin`, descendant cleanup, Ed25519 trust, hook sandboxing, raw-Wry isolation, Docker, and real loopback OpenSSH. Target-specific gates remain explicit. |

## Accepted decision review

P-001 through P-018 are accepted. All material research Gaps are resolved. GAP-008 and GAP-013 are explicitly deferred to public-marketplace/release work and do not block this Freeze.

## Remaining Freeze blockers

None.

## Feature and target release gates

These do not block research Freeze or unrelated features. Each blocks only the affected target or capability from being enabled or shipped:

- Windows/Linux runtime sidecar packaging and descendant cleanup require tests on those operating systems.
- Distributable macOS builds require a Developer ID identity and successful notarization; the local ad-hoc integrity proof is not a shipping signature.
- Arbitrary remote-content browsing on Windows/Linux requires raw-Wry-equivalent isolation evidence on those targets.
- Browser capability shipping requires real camera/microphone, download, popup, external-protocol, and user-selected-file UX checks against the chosen permission-capable Wry release. C4OS mediation must preserve `Default` platform/browser behavior and must not introduce blanket denial.
- Executable hooks on Windows/Linux require equivalent filesystem, process, environment, network, output, timeout, and revocation enforcement.
- Each real Remote SSH host profile requires host-specific authentication, host-key, filesystem, shell, credential-reference, and cancellation validation.
- Public marketplace launch requires an accepted signing/origin authority and moderation/revocation operating policy.

## Evidence state

The approved Proof Loop now provides executable, dated evidence for the locally available macOS boundary. It proves real OpenCode/Pi denial-before-side-effect paths, macOS supervisor/sidecar packaging, local marketplace trust and hook isolation, macOS raw-Wry isolation, and Local/Docker/OpenSSH transport parity. It does not convert target-specific signing, other operating systems, named remote hosts, or public marketplace governance into completed evidence.

## Implementation handoff

Do not create `tasks/` or add production sequencing under this Frozen research package. A new implementation spec must begin from `context/index.md`, inherit the accepted runtime/session architecture, and use this package only for research provenance, deferred gates, and Proof evidence.
