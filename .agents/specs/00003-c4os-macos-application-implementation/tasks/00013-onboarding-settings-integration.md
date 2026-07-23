# Task 00013 — Onboarding And Settings Integration

Status: verified

Coverage: SET-001, SET-004, SET-005, SET-006; integrated verification for SET-002 through SET-011 and UX-009.

## Summary

Complete onboarding, C4OS Home, and every Settings destination against real Provider, Credential, Capability, Runtime, Extension, MCP, Configuration, Policy, Workspace, and update services.

## Implementation Steps

1. Implement onboarding connection tests whose latest success gates Continue and is invalidated by relevant edits, including zero-model handling and recommended model/runtime confirmation.
2. Implement real provider CRUD/test/enable flows with opaque secret references, conditional fields, secure preservation, persistence, and failure recovery.
3. Implement model search/filter/details/bulk/refresh from CapabilityService freshness and authoritative availability.
4. Implement runtime defaults with persistence and explicit existing-versus-new Chat binding behavior.
5. Integrate C4OS Home, Settings shell, Plugins, Skills, MCP, Configuration, Advanced Policies, updates, notices, Back/round-trip state, native entry, and compressed navigation.
6. Verify all normal, loading, empty, error, degraded, dirty, conflict, revoked, and recovery states across themes, widths, keyboard, and restart.

## Verification Process

- Service integration tests for test invalidation, credentials, provider/model/runtime persistence, capability refresh, and cross-domain generations.
- Renderer tests and production Playwright/native walkthroughs for onboarding, Home, every Settings route, dialogs, notices, Back, compression, focus, accessibility, console, and overflow.
- Restart tests proving secrets never enter renderer state or logs and Settings changes reach authoritative services.

Acceptance criteria: none — implementation acceptance is delegated to the coordinator’s Agent Acceptance process for Spec 00003.

## Agent Acceptance

Result: passed — three independent reviewers reported P0=0 and P1=0.

Required evidence: complete in `output/native/task-00013-acceptance.md` and the 21 final Task 00013 native captures.

## Implementation Notes

Started 2026-07-23 from verified Task 00012 checkpoint `eedc3e8`. Hardcoded r013 records and simulated connection success cannot be reused as authority.

The smallest production-composed golden path starts from a first launch with no configured provider, tests one real provider connection through the Rust-owned provider/capability/credential services, requires at least one authoritative usable model, confirms an explicit recommended model plus OpenCode and Local defaults, persists only an opaque credential reference and the selected provider/default identities, enters Workspace Start, and round-trips through native Settings without replacing the application document or losing the authoritative state. Provider/model/runtime CRUD, all Settings destinations, degraded states, restart, responsive/accessibility behavior, and the complete integration matrix extend this same route.

Completed 2026-07-23. The Rust core now owns provider CRUD/test/enable state, exact credential binding and re-entry, model evidence, runtime defaults, app/workspace configuration, all 28 policy categories, concrete exceptions, policy-aware Workspace activation, and generation-safe compensation. Failed provider-route reloads quarantine both host and exact-generation dispatch peers. Policy/configuration transitions remain serialized with runtime/provider writers, and failed application or Workspace activation restores the prior durable and in-memory authority.

The production renderer now composes real provider onboarding, Workspace Start, Providers, Models, Runtimes, Configuration, Advanced Policies, Plugins, Skills, and MCP Servers in one Settings router. Provider Save & Test continues across two serialized approvals, Models and provider writers freeze while approval is pending, QA-only diagnostics stay off production routes, and provider approval focus returns to `Provider profiles`. The native Conversation draft is serialized before Terminal or Browser mutation so authoritative rebases cannot erase the selected mode.

## Verification Notes

- Renderer format, lint, typecheck, 69-file/361-test unit suite, production and QA builds, and the complete 43/43 Playwright matrix passed.
- `cargo check --lib` and `cargo fmt --all -- --check` passed. The serialized Rust workspace regression passed across the library (251 passed, 4 expected ignored tiers), every integration target, and doc tests; the two sandbox-only STDIO fixtures passed 2/2 in their exact host rerun.
- `npm run protocol:generate` passed Rust export 1/1, and generated protocol output was clean after excluding the unrelated `MarketplaceSnapshot.ts` whitespace diff.
- `npm run tauri:build` rebuilt the exact debug `C4OS.app`. Three packaged-tree verifiers passed 1/1 each; ignored MCP, OpenCode native, OpenCode stream, and packaged-runtime tiers passed 2/2, 7/7, 3/3, and 4/4; the private-TLS OpenCode/Pi native golden passed 1/1.
- Native macOS acceptance passed real provider onboarding, two approval continuations, two-model discovery, recommended defaults, Workspace Start, every Settings destination, native menu and `Cmd+,`, Back restoration, 1100/762/622 px containment, deterministic provider degradation/recovery, controlled restart, session credential re-entry, exact secret/log/process scans, and final cleanup.
- Production evidence: `output/native/task-00013-acceptance.md` plus 21 Task 00013 JPG captures.

## Agent Acceptance Notes

PASS 2026-07-23. Rust/security reported P0=0, P1=0, P2=1, P3=0; renderer/integration reported P0=0, P1=0, P2=0, P3=0; policy/configuration reported P0=0, P1=0, P2=0, P3=0.

The retained non-blocking P2 is a crash-consistency window that can leave an encrypted orphan provider credential if durable profile publication fails after credential creation. It does not persist plaintext or grant authority. Task 00015A owns final classification.
