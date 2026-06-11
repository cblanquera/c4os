# item-045: TASK-044 Resolve V1 Broader Compatibility Claims Scope

## Status

verified

## Objective

Choose and document whether V1 should make any broader compatibility claims
after explicit skills, local stdio MCP, raster artifact previews, project JSON
export, archived-session delete, and no durable memory have been scoped.

## Decision

Accepted `no_broader_compatibility_claims_v1` on 2026-06-12.

V1 should keep compatibility language narrow and feature-level. It should not
claim full AGENTS.md compatibility, full Agent Skills compatibility, full MCP
compatibility, Codex plugin compatibility, OpenCode config compatibility,
import compatibility, export/import round-trip compatibility, browser
compatibility, or durable-memory compatibility.

## Dependencies

- item-035
- item-037
- item-039
- item-041
- item-044

## Inputs

- `CONTEXT.md`
- `.agents/archive/planning-corpus/acceptance/compatibility.md`
- `.agents/archive/planning-corpus/decisions/deferred-decisions.md`
- `.agents/archive/planning-corpus/decisions/non-goals.md`
- `.agents/archive/planning-corpus/reviews/final-implementation-readiness-review.md`

## Deliverables

- Accepted, revised, or rejected V1 compatibility-claim tier.
- Updated acceptance criteria for allowed and forbidden compatibility claims.
- Follow-on implementation item only if product-facing status or docs need
  code/docs changes after acceptance.

## Acceptance Criteria

- User explicitly accepts, revises, or rejects the support tier.
- Allowed V1 compatibility claims are listed explicitly.
- Forbidden full-compatibility claims are listed explicitly.
- Import, round-trip compatibility, plugin compatibility, browser
  compatibility, durable memory, and runtime config ownership stay deferred
  unless separately accepted.

## Verification

- User accepted `no_broader_compatibility_claims_v1`.
- App status reports the accepted compatibility-claims tier.
- `cargo test --manifest-path src-tauri/Cargo.toml app_status_reports_no_broader_compatibility_claims_tier`.
- `npm test -- tests/backend-command-boundary.test.mjs`.
- `npm test`.
- `npm run build`.
- `cargo test --manifest-path src-tauri/Cargo.toml`.
- `npm run tauri -- build`.

## Follow-On

- No broader compatibility implementation is required because the accepted
  tier keeps claims narrow and feature-level.
