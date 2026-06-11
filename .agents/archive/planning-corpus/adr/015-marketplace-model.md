# ADR-015: Marketplace Model

Status: Provisional.

## Context

The marketplace specification recommends Codex-style `marketplace.json`, local and team marketplace manifests for MVP, public publishing out of scope, and install separate from enablement.

The review notes that marketplace concepts imply trust, curation, team policy, and support obligations, even though team collaboration is out of MVP.

## Decision

Support manifest-based local and team plugin marketplaces, with public marketplace publishing out of MVP.

This decision is provisional.

## Alternatives Considered

 - Manifest-based marketplace.
 - Central hosted marketplace only.
 - Git repository only.

## Alternatives That Should Be Considered

 - Local plugin install only in MVP.
 - Signed marketplace manifests.
 - Trust-tiered marketplaces.
 - Read-only marketplace preview before install support.

## Tradeoffs

Manifest marketplaces support personal, team, local, and remote catalogs.

Central hosted marketplace enables curation and revocation but conflicts with local-first goals.

Git-only distribution is simple but lacks policy and UX metadata.

## Consequences

 - Install and enable must remain separate.
 - Source integrity and update verification must be specified.
 - Marketplace trust indicators are required if remote sources are allowed.

## Follow-Up Questions

 - Are unsigned remote marketplace entries allowed?
 - How are plugin updates verified?
 - Who is responsible for plugin-caused data loss?
 - Are team marketplaces meaningful without team policy administration?

## ADR Recommendation

Keep this ADR high priority if marketplace remains in MVP.

