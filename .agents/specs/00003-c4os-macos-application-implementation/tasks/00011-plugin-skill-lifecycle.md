# Task 00011 — Plugin And Skill Lifecycle

Status: open

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

Result: failed — production Plugin and Skill evidence is absent.

Required evidence: package schemas and exact crypto/library lock; automated hostile and lifecycle results; immutable-store/audit inspection; production screenshots for all lifecycle states; accessibility/overflow/console checks; evidence paths, commands, and limitations.

## Implementation Notes

Not started. The historical Codex-shaped executable-plugin POC is not the selected production package model.

## Verification Notes

Not run.

## Agent Acceptance Notes

Catalog-only UI cannot close either coverage ID.
