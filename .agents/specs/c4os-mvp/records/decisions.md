# Decisions

## DEC-001: Use Frozen MVP As Implementation Contract

Status: accepted
Confidence: evidence-backed
MVP: yes
Source: `.agents/archive/planning-corpus/mvp/mvp-freeze.md`
Related:
- REQ-001
- CON-001

### Statement

`.agents/archive/planning-corpus/mvp/mvp-freeze.md` is the implementation contract for the MVP.

## DEC-002: Keep Runtime Adapter Hardened

Status: accepted
Confidence: evidence-backed
MVP: yes
Source: `.agents/archive/planning-corpus/mvp/mvp-freeze.md`
Related:
- REQ-004

### Statement

Phase 1 must implement the hardened OpenCode adapter path, not unconstrained
direct OpenCode.

## DEC-003: Defer Worktree Lifecycle

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/progress/items/item-032.md`, `.agents/archive/planning-corpus/roadmap/implementation-roadmap.md`
Related:
- RISK-002

### Statement

Worktree creation and cleanup are deferred beyond current V1.

## DEC-004: Promote `.agents` As Active Agent Layer

Status: accepted
Confidence: evidence-backed
MVP: no
Source: user request, `.task-bank/DEPRECATED.md`,
`.agents/specs/c4os-mvp/generated/source-retirement-review.md`
Related:
- CON-004

### Statement

Use `.agents/specs/` for durable planning records and `.agents/progress/` for
execution state. `.task-bank/` is deprecated and retained only as a historical
source until deletion is confirmed. `.agent/` is a deprecated compatibility
redirect only; do not write new C4OS planning or progress state there.

## DEC-005: Accept Explicit-Only V1 Agent Skills Scope

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/progress/items/item-035.md`, `.agents/archive/planning-corpus/acceptance/skills.md`
Related:
- RISK-001

### Statement

V1 Agent Skills support is limited to explicit project-local discovery and
explicit user-selected invocation. Skills must not auto-invoke, execute
scripts, load references or assets as trusted content, affect permissions, or
enter app-owned model context without explicit user action.

## DEC-006: Accept Explicit-Approval V1 Local Stdio MCP Scope

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/progress/items/item-037.md`, `.agents/archive/planning-corpus/acceptance/mcp-integration.md`, `https://modelcontextprotocol.io/specification/2025-11-25`
Related:
- RISK-001

### Statement

V1 MCP support is limited to local stdio servers that are explicitly added by
the user and launched only through approval-gated local process execution.
Remote MCP, automatic project-file startup, automatic resource or prompt
context ingestion, unapproved tools, unapproved sampling, and full MCP
compatibility remain out of scope.

## DEC-007: Accept Passive Raster Image Artifact Preview Scope

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/progress/items/item-039.md`, `.agents/archive/planning-corpus/acceptance/artifacts.md`, `.agents/archive/planning-corpus/spikes/SPIKE-013-artifact-browser-preview-security.md`
Related:
- AC-005
- RISK-003

### Statement

The accepted V1 artifact preview tier is
`raster_image_preview_only`: local passive PNG, JPEG, WebP, and GIF previews
with no active rendering, SVG, HTML, PDF, document, spreadsheet, browser,
remote URL, export, duplicate, search, execution, OCR, image analysis, or
automatic model-context behavior.

## DEC-008: Accept V1 Retention And Session Delete Scope

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/progress/items/item-042.md`, `.agents/archive/planning-corpus/acceptance/retention-cleanup.md`, `.agents/archive/planning-corpus/spikes/SPIKE-014-memory-session-retention.md`
Related:
- AC-006
- TASK-010

### Statement

The accepted V1 retention/delete scope is `archived_session_delete_only`.
Implementation may add explicit deletion for archived, unpinned sessions only.
Automatic cleanup, quotas, message-level delete/redaction, memory, import, and
round-trip compatibility remain deferred.

## DEC-009: Accept V1 Long-Term Memory Scope

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/progress/items/item-044.md`, `.agents/archive/planning-corpus/acceptance/memory.md`, `.agents/archive/planning-corpus/spikes/SPIKE-014-memory-session-retention.md`
Related:
- AC-007
- TASK-012

### Statement

The accepted V1 long-term memory scope is `no_durable_memory_v1`. V1 does not
add cross-session memory, learned preferences, automatic summaries, embeddings,
memory write prompts, or memory inspect/edit/delete UI beyond existing session
persistence, project JSON export, and archived-session delete.

## DEC-010: Accept V1 Broader Compatibility Claims Scope

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/progress/items/item-045.md`, `.agents/archive/planning-corpus/acceptance/compatibility.md`, `CONTEXT.md`
Related:
- AC-008
- TASK-013

### Statement

The accepted V1 broader compatibility-claims scope is
`no_broader_compatibility_claims_v1`. V1 keeps claims narrow and feature-level
rather than advertising full compatibility with AGENTS.md, Agent Skills, MCP,
Codex plugins, OpenCode config, import/export round trips, browser surfaces,
or durable memory.

## DEC-011: Accept V1 Provider Expansion Scope

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/progress/items/item-046.md`, `.agents/archive/planning-corpus/acceptance/provider-expansion.md`, `.agents/archive/planning-corpus/adr/019-model-provider-strategy.md`
Related:
- AC-009
- TASK-014

### Statement

The accepted V1 provider expansion scope is
`openrouter_only_v1_no_direct_or_local_provider_expansion`. V1 keeps the
existing OpenRouter-only provider path and defers direct providers, local model
providers, offline fallback, provider fallback routing, automatic
provider/model switching, BYOK provider subconfiguration, and multi-provider
settings.

## DEC-012: Accept V1 Browser And Web Viewing Scope

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/progress/items/item-047.md`, `.agents/archive/planning-corpus/acceptance/browser-web-viewing.md`, `.agents/archive/planning-corpus/spikes/SPIKE-013-artifact-browser-preview-security.md`
Related:
- AC-010
- TASK-015

### Statement

The accepted V1 browser and web-viewing scope is
`no_browser_or_web_viewing_v1`. V1 keeps browser panels, remote URL viewing,
DOM extraction, screenshots, browser testing, browser automation,
Chromium-backed rendering, generated HTML execution, and browser-content
model-context ingestion deferred.

## DEC-013: Accept V1 Plugin And Marketplace Scope

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/progress/items/item-048.md`, `.agents/archive/planning-corpus/acceptance/plugin-marketplace.md`, `.agents/archive/planning-corpus/acceptance/plugin-system.md`, `.agents/archive/planning-corpus/acceptance/marketplace.md`
Related:
- AC-011
- TASK-016

### Statement

The accepted V1 plugin and marketplace scope is
`no_plugin_or_marketplace_v1`. V1 keeps plugin installation, enablement,
execution, scripts/hooks, permission grants, trusted plugin assets,
plugin-provided MCP servers, marketplace browsing, remote marketplace
manifests, plugin search/install/update, ratings, reviews, advisories, and
curation workflows deferred.
