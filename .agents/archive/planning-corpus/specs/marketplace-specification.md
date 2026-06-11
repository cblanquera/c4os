# Marketplace Specification

## Goals

The marketplace should distribute plugins, skills, and workflow bundles without locking users into a proprietary ecosystem.

## Marketplace Manifest

Support Codex-style `marketplace.json`:

```json
{
  "name": "personal",
  "interface": {
    "displayName": "Personal"
  },
  "plugins": []
}
```

Each plugin entry should include:
 - `name`
 - `source`
 - `policy.installation`
 - `policy.authentication`
 - `category`
 - optional product or compatibility gates

Recommendation: use Codex marketplace metadata as the base and add namespaced extensions only when required.

Alternatives considered:
 - Central hosted marketplace only: better curation, weaker local/team workflows.
 - Git repository only: simple distribution, weak UI and policy metadata.

Why selected: manifest-based marketplaces support personal, team, local, and remote catalogs.

## Source Types

 - Local path.
 - Git URL and revision.
 - Archive URL with checksum.
 - Future signed registry entry.

## Policy Values

Installation policy:
 - `NOT_AVAILABLE`
 - `AVAILABLE`
 - `INSTALLED_BY_DEFAULT`

Authentication policy:
 - `ON_INSTALL`
 - `ON_USE`

## Marketplace UX

Marketplace views should show:
 - Plugin name and description.
 - Publisher/source.
 - Category.
 - Requested permissions.
 - Included skills, MCP servers, and app connectors.
 - Authentication requirements.
 - Version and update status.
 - Trust indicators and signatures when available.

## Publishing Workflow

MVP should support local and team marketplace manifests. Public publishing is out of scope, but the data model should allow later signed submissions, review states, ratings, and vulnerability advisories.

## Security

Marketplace install must verify source integrity where possible. Remote plugin install should require explicit confirmation and show requested permissions before enabling.

Recommendation: separate install from enablement.

Alternatives considered:
 - Enable immediately after install: faster, but unsafe.
 - Require manual file copy only: safe, but poor UX.

Why selected: install-then-enable matches desktop extension safety patterns.
