# Plugin And Marketplace Acceptance Criteria

## Current Decision

Status: accepted.

Accepted tier: `no_plugin_or_marketplace_v1`.

V1 should not add plugin installation, plugin enablement, plugin execution,
plugin scripts or hooks, plugin permission grants, trusted plugin assets,
plugin-provided MCP servers, plugin marketplace browsing, remote marketplace
manifests, plugin updates, plugin search, ratings, reviews, advisories, or
curation workflows.

## Acceptance Criteria

- User explicitly accepts `no_plugin_or_marketplace_v1`.
- Plugin installation, enablement, execution, scripts, hooks, permissions, and
  trusted assets remain unavailable.
- Plugin-provided MCP servers remain unavailable.
- Marketplace browsing, remote manifests, plugin search/install/update,
  ratings, reviews, advisories, and curation remain unavailable.
- Project-local plugin metadata is not trusted, executed, or granted
  permissions.

## Out Of Scope

- Plugin installation or enablement.
- Plugin scripts or hooks.
- Plugin permissions and trust review.
- Plugin-provided MCP servers.
- Marketplace browsing or remote manifests.
- Plugin search, install, updates, ratings, reviews, advisories, or curation.
