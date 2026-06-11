# item-048: TASK-047 Resolve V1 Plugin And Marketplace Scope

## Status

verified

## Objective

Choose and document whether V1 should add plugin or marketplace workflows now
that skills, local stdio MCP, compatibility claims, provider scope, and browser
scope have been bounded.

## Accepted Decision

Accepted tier: `no_plugin_or_marketplace_v1`.

V1 should not add plugin installation, plugin enablement, plugin execution,
plugin scripts or hooks, plugin permission grants, trusted plugin assets,
plugin-provided MCP servers, plugin marketplace browsing, remote marketplace
manifests, plugin updates, plugin search, ratings, reviews, advisories, or
curation workflows.

## Dependencies

- item-036
- item-038
- item-045
- item-047

## Inputs

- `CONTEXT.md`
- `.agents/archive/planning-corpus/acceptance/plugin-marketplace.md`
- `.agents/archive/planning-corpus/acceptance/plugin-system.md`
- `.agents/archive/planning-corpus/acceptance/marketplace.md`
- `.agents/archive/planning-corpus/decisions/deferred-decisions.md`
- `.agents/archive/planning-corpus/decisions/non-goals.md`

## Deliverables

- Accepted, revised, or rejected V1 plugin and marketplace support tier.
- Updated acceptance criteria for plugin execution, trust, permissions, and
  marketplace acquisition.
- Follow-on implementation item only if V1 app status or UI needs code changes
  after acceptance.

## Acceptance Criteria

- User explicitly accepts, revises, or rejects the support tier.
- Plugin install, enable, execution, scripts, hooks, permissions, trusted
  assets, and plugin-provided MCP servers are explicitly accepted or deferred.
- Marketplace browsing, remote manifests, plugin search/install/update, and
  curation workflows are explicitly accepted or deferred.
- Project-local plugin metadata remains untrusted unless separately accepted.

## Verification Evidence

- User accepted `no_plugin_or_marketplace_v1` on 2026-06-12.
- App status exposes the accepted no-plugin/no-marketplace V1 tier.
- Verification passed: `cargo test --manifest-path src-tauri/Cargo.toml app_status_reports_no_plugin_or_marketplace_tier`.
- Verification passed: `npm test -- tests/backend-command-boundary.test.mjs`.
- Verification passed: `npm test`.
- Verification passed: `npm run build`.
- Verification passed: `cargo test --manifest-path src-tauri/Cargo.toml`.
- Verification passed: `npm run tauri -- build`.
