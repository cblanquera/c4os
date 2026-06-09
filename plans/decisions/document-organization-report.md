# Document Organization Report

This report records the information architecture applied to the `plans/` directory. Existing document contents were not rewritten as part of this reorganization.

## 1. Proposed Directory Tree

```text
plans/
  adr/
    00-adr-candidate-list.md
    00-proposed-adr-priority-order.md
    001-product-scope-and-mvp-audience.md
    002-desktop-application-shell.md
    003-agent-runtime-strategy.md
    004-policy-enforcement-authority.md
    005-standards-first-interoperability.md
    007-local-first-storage-model.md
    008-unified-tool-invocation-and-ledger.md
    009-permission-and-approval-model.md
    010-filesystem-and-shell-access-defaults.md
    011-mcp-integration-strategy.md
    014-plugin-system-compatibility.md
    015-marketplace-model.md
    016-plugin-trust-and-execution.md
    019-model-provider-strategy.md
  decisions/
    document-organization-report.md
    implementation-readiness-gaps.md
  mvp/
  research/
    industry-standards-research.md
    open-questions-and-research-gaps.md
  reviews/
    adr-architecture-review-report.md
    architecture-review.md
    risk-analysis.md
  roadmap/
    implementation-roadmap.md
  specs/
    architecture-specification.md
    data-model-specification.md
    functional-specification.md
    marketplace-specification.md
    plugin-system-specification.md
    requirements-specification.md
    security-specification.md
    ux-specification.md
  spikes/
```

## 2. File Move Plan

Completed moves:

 - `plans/01-industry-standards-research.md` -> `plans/research/industry-standards-research.md`
 - `plans/12-open-questions-and-research-gaps.md` -> `plans/research/open-questions-and-research-gaps.md`
 - `plans/02-requirements-specification.md` -> `plans/specs/requirements-specification.md`
 - `plans/03-functional-specification.md` -> `plans/specs/functional-specification.md`
 - `plans/04-architecture-specification.md` -> `plans/specs/architecture-specification.md`
 - `plans/05-plugin-system-specification.md` -> `plans/specs/plugin-system-specification.md`
 - `plans/06-marketplace-specification.md` -> `plans/specs/marketplace-specification.md`
 - `plans/07-security-specification.md` -> `plans/specs/security-specification.md`
 - `plans/08-data-model-specification.md` -> `plans/specs/data-model-specification.md`
 - `plans/09-ux-specification.md` -> `plans/specs/ux-specification.md`
 - `plans/10-implementation-roadmap.md` -> `plans/roadmap/implementation-roadmap.md`
 - `plans/11-risk-analysis.md` -> `plans/reviews/risk-analysis.md`
 - `plans/gaps.md` -> `plans/decisions/implementation-readiness-gaps.md`

Files retained in place:

 - `plans/adr/*` remained under `plans/adr/`.
 - `plans/reviews/architecture-review.md` remained under `plans/reviews/`.
 - `plans/reviews/adr-architecture-review-report.md` remained under `plans/reviews/`.

Directories created:

 - `plans/research/`
 - `plans/specs/`
 - `plans/reviews/`
 - `plans/adr/`
 - `plans/spikes/`
 - `plans/decisions/`
 - `plans/mvp/`
 - `plans/roadmap/`

## 3. Duplicate Document Report

Exact duplicates:

 - No exact duplicate Markdown documents were found by checksum.

Overlapping documents:

 - `plans/research/open-questions-and-research-gaps.md` and `plans/decisions/implementation-readiness-gaps.md` overlap on unresolved questions. The research document is broad discovery; the decisions document is an implementation-readiness gate checklist.
 - `plans/reviews/architecture-review.md` and `plans/reviews/adr-architecture-review-report.md` overlap on architecture risk. The first reviews the original specs; the second reviews the ADR set adversarially.
 - `plans/reviews/risk-analysis.md`, `plans/reviews/architecture-review.md`, and `plans/reviews/adr-architecture-review-report.md` overlap on risk themes. The risk-analysis document is a concise first-pass risk list; the review reports are deeper critique artifacts.
 - `plans/adr/00-adr-candidate-list.md` overlaps with individual ADR drafts because it summarizes the same decisions. It should remain as an index and extraction ledger.
 - `plans/adr/00-proposed-adr-priority-order.md` overlaps with `plans/adr/00-adr-candidate-list.md` on priority metadata. It should remain separate as an execution-order view.

Recommended duplicate handling:

 - Do not delete any current documents yet.
 - Treat overlapping documents as different review layers until content consolidation is explicitly requested.
 - Future consolidation candidates are the two review reports and the two gap/open-question documents.

## 4. Missing Document Report

Missing from `plans/spikes/`:

 - OpenCode runtime control spike.
 - Policy enforcement gateway spike.
 - Tauri versus Electron browser/artifact spike.
 - Shell sandboxing spike.
 - MCP local and remote security spike.
 - Plugin validation and non-execution indexing spike.
 - SQLite concurrency and crash-recovery spike.
 - Large workspace performance spike.

Missing from `plans/mvp/`:

 - MVP scope definition.
 - MVP audience and persona definition.
 - MVP acceptance criteria.
 - MVP non-goals.
 - MVP security baseline.
 - MVP workflow list.
 - MVP release readiness checklist.

Missing from `plans/decisions/`:

 - Architecture freeze checklist.
 - Standards conformance matrix.
 - Policy authority decision summary.
 - Runtime feasibility decision summary.
 - Privacy and data movement decision summary.

Missing from `plans/specs/`:

 - Operational specification.
 - Telemetry and privacy specification.
 - Testing and verification specification.
 - Runtime adapter specification.
 - Data portability/export specification.
 - Accessibility specification.
 - Error recovery specification.

Missing from `plans/research/`:

 - OpenCode runtime API research.
 - Codex plugin schema stability research.
 - MCP threat-model research.
 - Desktop browser automation research.
 - Local shell sandboxing research.

Missing from `plans/roadmap/`:

 - Research and spike roadmap.
 - Decision-gate roadmap.
 - MVP release roadmap.

## 5. Recommended Naming Conventions

Directory conventions:

 - `plans/research/`: discovery, source notes, standards research, open research questions.
 - `plans/specs/`: normative product, functional, architecture, security, data, UX, and subsystem specifications.
 - `plans/reviews/`: critique, risk analysis, architecture reviews, adversarial reviews.
 - `plans/adr/`: ADR indexes, priority lists, and individual ADR drafts.
 - `plans/spikes/`: time-boxed technical validation plans and spike results.
 - `plans/decisions/`: decision summaries, readiness gates, matrices, and organization records that are not ADRs.
 - `plans/mvp/`: MVP scope, acceptance criteria, release gates, workflow lists, and non-goals.
 - `plans/roadmap/`: phase sequencing, release plans, decision-gate plans, and research/spike sequencing.

File naming conventions:

 - Use lowercase kebab-case for Markdown files.
 - Prefer descriptive names over numeric prefixes once files live inside typed directories.
 - Keep ADR files numbered with three digits: `001-title.md`.
 - Use `00-` only for ADR indexes or ordering documents that must sort before numbered ADRs.
 - Use singular subsystem names where possible: `security-specification.md`, not `security-specifications.md`.
 - Use `*-specification.md` only for normative specs.
 - Use `*-review.md` or `*-review-report.md` only for critique artifacts.
 - Use `*-gaps.md` or `*-readiness-gaps.md` for blocker/gap documents.
 - Use `*-roadmap.md` for sequencing documents.

