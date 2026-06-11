# Tasks

## TASK-001: MVP Implementation Tasks Migrated

Status: accepted
Confidence: evidence-backed
MVP: yes
Source: `.agents/archive/planning-corpus/implementation/tasks.md`, `.agents/progress/manifest.md`
Related:
- AC-002

### Statement

Implementation tasks `TASK-001` through `TASK-026` are represented by migrated
progress items `item-001` through `item-026`.

## TASK-002: Stabilization And V1 Follow-On Tasks Migrated

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/progress/manifest.md`
Related:
- AC-002

### Statement

Stabilization and V1 follow-on work through nested `AGENTS.md` ordered display
guidance is represented by migrated progress items `item-027` through
`item-034`.

## TASK-003: V1 Agent Skills Scope Accepted

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/archive/planning-corpus/implementation/tasks.md`, `.agents/progress/items/item-035.md`
Related:
- AC-002

### Statement

`TASK-034`, represented by progress item `item-035`, resolves Agent Skills
discovery and invocation scope as `explicit_discovery_and_invocation_only`.

## TASK-004: V1 Agent Skills Implementation Ready

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/archive/planning-corpus/implementation/tasks.md`, `.agents/progress/items/item-036.md`
Related:
- DEC-005

### Statement

`TASK-035`, represented by progress item `item-036`, implemented explicit
project-local skill discovery and invocation and is verified.

## TASK-005: V1 Local Stdio MCP Scope Accepted

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/archive/planning-corpus/implementation/tasks.md`, `.agents/progress/items/item-037.md`
Related:
- RISK-001

### Statement

`TASK-036`, represented by progress item `item-037`, resolves local stdio MCP
scope as `local_stdio_explicit_approval_only`.

## TASK-006: V1 Local Stdio MCP Implementation Verified

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/archive/planning-corpus/implementation/tasks.md`, `.agents/progress/items/item-038.md`
Related:
- DEC-006

### Statement

`TASK-037`, represented by progress item `item-038`, implemented the explicit
approval-gated local stdio MCP surface and is verified.

## TASK-007: V1 Rich Artifact Preview Scope Accepted

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/archive/planning-corpus/implementation/tasks.md`, `.agents/progress/items/item-039.md`
Related:
- AC-005
- DEC-007

### Statement

`TASK-038`, represented by progress item `item-039`, should choose and document
the V1 rich artifact preview support tier before implementation. The accepted
tier is `raster_image_preview_only`.

## TASK-008: Passive Raster Image Artifact Preview Implementation Verified

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/archive/planning-corpus/implementation/tasks.md`, `.agents/progress/items/item-040.md`
Related:
- AC-005
- DEC-007

### Statement

`TASK-039`, represented by progress item `item-040`, implemented passive local
PNG, JPEG, WebP, and GIF artifact previews while keeping active HTML, SVG, PDF,
documents, spreadsheets, browser rendering, remote URLs, export, duplicate
workflows, search, OCR, image analysis, and automatic model-context ingestion
out of scope.

## TASK-009: V1 Project JSON Export Verified

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/archive/planning-corpus/implementation/tasks.md`, `.agents/progress/items/item-041.md`
Related:
- DEC-003

### Statement

`TASK-040`, represented by progress item `item-041`, implemented
`project_json_export_only`: selected-project JSON export is available while
import and round-trip compatibility remain deferred.

## TASK-010: V1 Retention And Session Delete Scope Accepted

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/progress/items/item-042.md`, `.agents/archive/planning-corpus/acceptance/retention-cleanup.md`
Related:
- AC-006
- DEC-008

### Statement

`TASK-041`, represented by progress item `item-042`, accepted
`archived_session_delete_only`: explicit deletion for archived, unpinned
sessions only, with automatic cleanup, quotas, message-level delete/redaction,
memory, import, and round-trip compatibility remaining out of scope.

## TASK-011: Archived Session Delete Implementation

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/progress/items/item-043.md`, `.agents/archive/planning-corpus/acceptance/retention-cleanup.md`
Related:
- AC-006
- DEC-008

### Statement

`TASK-042`, represented by progress item `item-043`, implemented the accepted
`archived_session_delete_only` tier and is verified.

## TASK-012: V1 Long-Term Memory Scope Accepted

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/progress/items/item-044.md`, `.agents/archive/planning-corpus/acceptance/memory.md`
Related:
- AC-007
- DEC-009

### Statement

`TASK-043`, represented by progress item `item-044`, accepted
`no_durable_memory_v1`: no cross-session memory, learned preferences,
automatic summaries, embeddings, memory write prompts, or memory
inspect/edit/delete UI beyond existing session persistence, project JSON
export, and archived-session delete.

## TASK-013: V1 Broader Compatibility Claims Scope Accepted

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/progress/items/item-045.md`, `.agents/archive/planning-corpus/acceptance/compatibility.md`
Related:
- AC-008
- DEC-010

### Statement

`TASK-044`, represented by progress item `item-045`, proposes
`no_broader_compatibility_claims_v1`: keep V1 compatibility claims narrow and
feature-level, with full AGENTS.md, Agent Skills, MCP, Codex plugin, OpenCode
config, import, round-trip, browser, and durable-memory compatibility claims
deferred.

## TASK-014: V1 Provider Expansion Scope Accepted

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/progress/items/item-046.md`, `.agents/archive/planning-corpus/acceptance/provider-expansion.md`
Related:
- AC-009
- DEC-011

### Statement

`TASK-045`, represented by progress item `item-046`, proposes
`openrouter_only_v1_no_direct_or_local_provider_expansion`: keep V1
OpenRouter-only, with direct providers, local model providers, offline
fallback, provider fallback routing, automatic switching, BYOK provider
subconfiguration, provider-specific settings, and multi-provider configuration
deferred.

## TASK-015: V1 Browser And Web Viewing Scope Accepted

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/progress/items/item-047.md`, `.agents/archive/planning-corpus/acceptance/browser-web-viewing.md`
Related:
- AC-010
- DEC-012

### Statement

`TASK-046`, represented by progress item `item-047`, proposes
`no_browser_or_web_viewing_v1`: keep browser panels, remote URL viewing, DOM
extraction, screenshots, browser testing, browser automation, Chromium-backed
rendering, generated HTML execution, and browser-content model-context
ingestion deferred.

## TASK-016: V1 Plugin And Marketplace Scope Accepted

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/progress/items/item-048.md`, `.agents/archive/planning-corpus/acceptance/plugin-marketplace.md`
Related:
- AC-011
- DEC-013

### Statement

`TASK-047`, represented by progress item `item-048`, accepts
`no_plugin_or_marketplace_v1`: keep plugin installation, enablement,
execution, scripts/hooks, permission grants, trusted plugin assets,
plugin-provided MCP servers, marketplace browsing, remote marketplace
manifests, plugin search/install/update, ratings, reviews, advisories, and
curation workflows deferred.
