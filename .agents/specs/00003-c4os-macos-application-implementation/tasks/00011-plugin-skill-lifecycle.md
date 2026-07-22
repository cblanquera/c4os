# Task 00011 — Plugin And Skill Lifecycle

Status: verified

Coverage: SET-007, SET-008; support for UX-010, UX-012, UX-015, ART-001.

## Summary

Implement signed, immutable C4OS Plugin and Skill packages; discovery and marketplaces; declarative contributions; progressive Skill loading; sandboxed hooks; trust, revocation, update, rollback, uninstall, and truthful Settings states.

## Implementation Steps

1. Define versioned C4OS package manifests, strict schemas, signature/digest verification, trust authorities, content-addressed storage, quarantine, and disabled transactional installation.
2. Implement Plugin activation, declarative contributions, schema-driven settings, sensitive references, hook worker protocol, supervision, policy mediation, and no arbitrary renderer code.
3. Implement Skill frontmatter/resource validation, metadata-only discovery, stable source-qualified identity, precedence/collision handling, progressive loading, eligibility, customization, and Try in Chat.
4. Implement activation, update, revocation propagation, rollback, uninstall, cleanup, recovery, and audit with immutable selectors and last-known-good state.
5. Integrate real service-backed Settings catalogs/details/states and hostile package fixtures.

## Verification Process

- Rust/package tests for signatures, schemas, immutable storage, transactions, hostile archives, hooks, revocation, rollback, recovery, and audit.
- Renderer tests and production walkthroughs for Plugin/Skill discovery, details, settings, invalid states, lifecycle, Try in Chat, accessibility, responsive behavior, console, and overflow.
- Source inspection proving no executable plugin metadata or untrusted renderer code path.

Acceptance criteria: none — implementation acceptance is delegated to the coordinator’s Agent Acceptance process for Spec 00003.

## Agent Acceptance

Result: passed — P0=0, P1=0, P2=0, P3=0.

Required evidence is complete: package schemas and exact local dependency lock; hostile, transaction, lifecycle, persistence, renderer, native sandbox, bundle, restart, and independent-review results; immutable-store/audit inspection; and production Plugin/Skill lifecycle evidence.

## Implementation Notes

Completed 2026-07-22 from verified Task 00010 checkpoint `8ef2387`. The historical Codex-shaped executable-plugin POC remains rejected production scope. The production-composed golden path uses one user-added local catalog exposing immutable signed declarative Plugin and Skill packages. Rust validates bounded catalog/package paths, strict manifests/frontmatter, complete content inventory/digests, Ed25519 origin/content trust, SemVer compatibility, and source-qualified identity; stages verified bytes in quarantine; installs disabled into a private content-addressed store; explicitly activates for the next turn; progressively loads one eligible Skill; and disables, revokes, rolls back, or uninstalls without package-defined removal code or renderer/peer authority.

The completed path also covers all five Skill sources, deterministic precedence/collisions, private customization, portable Workspace Skills, exact Try-in-Chat selection, declarative Plugin settings/apps/MCP metadata, reviewed immutable hook arguments, Action-Gateway-only proposals, bounded native sandbox execution, immediate disable during execution, stale completion rejection, key/package revocation, update staging and activation, byte-reverified rollback, visible restart recovery, and atomic database/selector/Skill publication. True pre-publication selector failures compensate to prior working semantics; atomic rename is the selector publication commit point so post-rename directory-sync degradation cannot be misreported as a failed CAS that rolls durable state backward.

## Verification Notes

- `npm run typecheck`, `npm run lint`, and `npm run format:check`: passed.
- `npm test -- --run`: 53 files and 271 tests passed.
- Focused Rust: selector post-rename regression 1/1; `extension_transactions` 10/10; `extension_persistence` 4/4; `extension_lifecycle` 16 passed with the native test intentionally ignored; `protocol_contract` 20/20.
- `cargo test --workspace`: passed; library 192 passed and 4 expected ignores, followed by the complete integration and doc-test matrix with no failures.
- `npm run tauri:build`: passed and produced the rebuilt debug `C4OS.app` with verified bundled sidecars.
- Native ignored hook acceptance: 1/1 passed outside the outer sandbox with the project-owned pinned Node runtime.
- Native macOS UI/restart acceptance against `/private/tmp/c4os-task11-acceptance.iCeZD4`: the source-qualified selected Skill survived restart; the signed Plugin remained enabled; immediate Disable was available; signature, reviewed-hook argument, last-known-good, and recovery detail were visible.
- Production evidence: `output/native/task-00011-plugin-details.jpg`, `task-00011-plugin-update-staged.jpg`, `task-00011-hook-review-invalidated.jpg`, `task-00011-skills-collisions.jpg`, and `task-00011-plugins-directory.jpg`.

## Agent Acceptance Notes

Final independent frozen-tree review passed with P0=0, P1=0, P2=0, P3=0. Review explicitly closed portable Workspace Skill archiving, customization collision cleanup, immutable-manifest hook-argument migration, interrupted execution recovery, rollback re-verification, immediate executing/update-staged Disable, stale completion rejection, pre-publication compensation, post-rename selector publication semantics, restart authority, Try-in-Chat visibility, and truthful executing-state copy. SET-007 and SET-008 remain open only because their named supporting Settings/audit tasks are not yet verified.
