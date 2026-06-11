# Acceptance Criteria

## AC-001: MVP Acceptance Set

Status: accepted
Confidence: evidence-backed
MVP: yes
Source: `.agents/archive/planning-corpus/mvp/mvp-freeze.md`
Related:
- REQ-001

### Statement

MVP completion depends on the required acceptance documents listed in
`.agents/archive/planning-corpus/mvp/mvp-freeze.md`.

### Acceptance Links

- `.agents/archive/planning-corpus/acceptance/openrouter-integration.md`
- `.agents/archive/planning-corpus/acceptance/project-management.md`
- `.agents/archive/planning-corpus/acceptance/file-access.md`
- `.agents/archive/planning-corpus/acceptance/security-and-permissions.md`
- `.agents/archive/planning-corpus/acceptance/shell-execution.md`
- `.agents/archive/planning-corpus/acceptance/agent-execution.md`
- `.agents/archive/planning-corpus/acceptance/sessions.md`
- `.agents/archive/planning-corpus/acceptance/git-integration.md`
- `.agents/archive/planning-corpus/acceptance/artifacts.md`
- `.agents/archive/planning-corpus/acceptance/skills.md`
- `.agents/archive/planning-corpus/acceptance/settings-and-configuration.md`
- `.agents/archive/planning-corpus/acceptance/telemetry-and-diagnostics.md`

## AC-002: Progress Migration Complete

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.task-bank/`, `.agents/progress/`
Related:
- DEC-004

### Statement

The legacy progress bank is ported to `.agents/progress/`, including manifest,
brief, decisions, conventions, outputs, CSV progress sheet, batches, item
packets, and logs.

## AC-003: Source Retirement Decision Exists

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/specs/c4os-mvp/generated/source-retirement-review.md`
Related:
- DEC-004

### Statement

The source-retirement review explicitly classifies `.task-bank/` and other
planning sources before any removal.

## AC-004: Human Planning Corpus Archived

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/archive/planning-corpus/MIGRATION.md`
Related:
- EVD-005

### Statement

The detailed human planning corpus is archived under
`.agents/archive/planning-corpus/`, compact routing facts are available under
`.agents/specs/c4os-mvp/`, and future workers can use `.agents` without reading
the repository-root human planning directory.

## AC-005: Proposed V1 Raster Image Artifact Preview Tier

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/archive/planning-corpus/acceptance/artifacts.md`, `.agents/progress/items/item-039.md`
Related:
- TASK-007

### Statement

The accepted V1 rich artifact preview tier is
`raster_image_preview_only`: passive previews for local PNG, JPEG, WebP, and
GIF artifacts only. SVG, HTML, PDF, documents, spreadsheets, browser rendering,
remote URLs, execution, search, export, duplicate workflows, and automatic
model-context ingestion remain out of scope.

## AC-006: Accepted V1 Retention And Session Delete Tier

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/archive/planning-corpus/acceptance/retention-cleanup.md`, `.agents/progress/items/item-042.md`
Related:
- TASK-010

### Statement

The accepted V1 retention/delete tier is `archived_session_delete_only`:
explicit deletion for archived, unpinned sessions only. Active, latest,
running, pending-approval, and pinned sessions remain protected. Automatic
cleanup, quotas, message-level delete/redaction, memory, import, and
round-trip compatibility remain out of scope unless separately accepted.

## AC-007: Accepted V1 Long-Term Memory Tier

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/archive/planning-corpus/acceptance/memory.md`, `.agents/progress/items/item-044.md`
Related:
- TASK-012

### Statement

The accepted V1 long-term memory tier is `no_durable_memory_v1`: no
cross-session memory, learned preferences, automatic summaries, embeddings,
memory write prompts, or memory inspect/edit/delete UI beyond existing session
persistence, project JSON export, and archived-session delete.

## AC-008: Accepted V1 Broader Compatibility Claims Tier

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/archive/planning-corpus/acceptance/compatibility.md`, `.agents/progress/items/item-045.md`
Related:
- TASK-013

### Statement

The accepted V1 broader compatibility-claims tier is
`no_broader_compatibility_claims_v1`: V1 keeps compatibility language narrow
and feature-level, without claiming full AGENTS.md compatibility, full Agent
Skills compatibility, full MCP compatibility, Codex plugin compatibility,
OpenCode config compatibility, import compatibility, export/import round-trip
compatibility, browser compatibility, or durable-memory compatibility.

## AC-009: Accepted V1 Provider Expansion Tier

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/archive/planning-corpus/acceptance/provider-expansion.md`, `.agents/progress/items/item-046.md`
Related:
- TASK-014

### Statement

The accepted V1 provider expansion tier is
`openrouter_only_v1_no_direct_or_local_provider_expansion`: V1 keeps
OpenRouter as the only configurable provider path, while direct provider
integrations, local model providers, offline fallback, provider fallback
routing, automatic model switching, BYOK provider subconfiguration,
provider-specific settings, provider comparison workflows, and multi-provider
configuration remain deferred.

## AC-010: Accepted V1 Browser And Web Viewing Tier

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/archive/planning-corpus/acceptance/browser-web-viewing.md`, `.agents/progress/items/item-047.md`
Related:
- TASK-015

### Statement

The accepted V1 browser and web-viewing tier is
`no_browser_or_web_viewing_v1`: V1 keeps in-app browser panels, remote URL
viewing, DOM extraction, screenshots, browser testing, browser automation, web
content ingestion, Chromium-backed rendering, generated HTML execution, and
automatic browser-content model-context ingestion deferred.

## AC-011: Accepted V1 Plugin And Marketplace Tier

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/archive/planning-corpus/acceptance/plugin-marketplace.md`, `.agents/progress/items/item-048.md`
Related:
- TASK-016

### Statement

The accepted V1 plugin and marketplace tier is
`no_plugin_or_marketplace_v1`: V1 keeps plugin installation, enablement,
execution, scripts/hooks, permission grants, trusted plugin assets,
plugin-provided MCP servers, marketplace browsing, remote marketplace
manifests, plugin search/install/update, ratings, reviews, advisories, and
curation workflows deferred.
