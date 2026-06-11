# Document Organization Report

This report records the information architecture applied to the `.agents/archive/planning-corpus/` directory. Existing document contents were not rewritten as part of this reorganization.

## 1. Proposed Directory Tree

```text
.agents/archive/planning-corpus/
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

 - `.agents/archive/planning-corpus/01-industry-standards-research.md` -> `.agents/archive/planning-corpus/research/industry-standards-research.md`
 - `.agents/archive/planning-corpus/12-open-questions-and-research-gaps.md` -> `.agents/archive/planning-corpus/research/open-questions-and-research-gaps.md`
 - `.agents/archive/planning-corpus/02-requirements-specification.md` -> `.agents/archive/planning-corpus/specs/requirements-specification.md`
 - `.agents/archive/planning-corpus/03-functional-specification.md` -> `.agents/archive/planning-corpus/specs/functional-specification.md`
 - `.agents/archive/planning-corpus/04-architecture-specification.md` -> `.agents/archive/planning-corpus/specs/architecture-specification.md`
 - `.agents/archive/planning-corpus/05-plugin-system-specification.md` -> `.agents/archive/planning-corpus/specs/plugin-system-specification.md`
 - `.agents/archive/planning-corpus/06-marketplace-specification.md` -> `.agents/archive/planning-corpus/specs/marketplace-specification.md`
 - `.agents/archive/planning-corpus/07-security-specification.md` -> `.agents/archive/planning-corpus/specs/security-specification.md`
 - `.agents/archive/planning-corpus/08-data-model-specification.md` -> `.agents/archive/planning-corpus/specs/data-model-specification.md`
 - `.agents/archive/planning-corpus/09-ux-specification.md` -> `.agents/archive/planning-corpus/specs/ux-specification.md`
 - `.agents/archive/planning-corpus/10-implementation-roadmap.md` -> `.agents/archive/planning-corpus/roadmap/implementation-roadmap.md`
 - `.agents/archive/planning-corpus/11-risk-analysis.md` -> `.agents/archive/planning-corpus/reviews/risk-analysis.md`
 - `.agents/archive/planning-corpus/gaps.md` -> `.agents/archive/planning-corpus/decisions/implementation-readiness-gaps.md`

Files retained in place:

 - `.agents/archive/planning-corpus/adr/*` remained under `.agents/archive/planning-corpus/adr/`.
 - `.agents/archive/planning-corpus/reviews/architecture-review.md` remained under `.agents/archive/planning-corpus/reviews/`.
 - `.agents/archive/planning-corpus/reviews/adr-architecture-review-report.md` remained under `.agents/archive/planning-corpus/reviews/`.

Directories created:

 - `.agents/archive/planning-corpus/research/`
 - `.agents/archive/planning-corpus/specs/`
 - `.agents/archive/planning-corpus/reviews/`
 - `.agents/archive/planning-corpus/adr/`
 - `.agents/archive/planning-corpus/spikes/`
 - `.agents/archive/planning-corpus/decisions/`
 - `.agents/archive/planning-corpus/mvp/`
 - `.agents/archive/planning-corpus/roadmap/`

## 3. Duplicate Document Report

Exact duplicates:

 - No exact duplicate Markdown documents were found by checksum.

Overlapping documents:

 - `.agents/archive/planning-corpus/research/open-questions-and-research-gaps.md` and `.agents/archive/planning-corpus/decisions/implementation-readiness-gaps.md` overlap on unresolved questions. The research document is broad discovery; the decisions document is an implementation-readiness gate checklist.
 - `.agents/archive/planning-corpus/reviews/architecture-review.md` and `.agents/archive/planning-corpus/reviews/adr-architecture-review-report.md` overlap on architecture risk. The first reviews the original specs; the second reviews the ADR set adversarially.
 - `.agents/archive/planning-corpus/reviews/risk-analysis.md`, `.agents/archive/planning-corpus/reviews/architecture-review.md`, and `.agents/archive/planning-corpus/reviews/adr-architecture-review-report.md` overlap on risk themes. The risk-analysis document is a concise first-pass risk list; the review reports are deeper critique artifacts.
 - `.agents/archive/planning-corpus/adr/00-adr-candidate-list.md` overlaps with individual ADR drafts because it summarizes the same decisions. It should remain as an index and extraction ledger.
 - `.agents/archive/planning-corpus/adr/00-proposed-adr-priority-order.md` overlaps with `.agents/archive/planning-corpus/adr/00-adr-candidate-list.md` on priority metadata. It should remain separate as an execution-order view.

Recommended duplicate handling:

 - Do not delete any current documents yet.
 - Treat overlapping documents as different review layers until content consolidation is explicitly requested.
 - Future consolidation candidates are the two review reports and the two gap/open-question documents.

## 4. Missing Document Report

Missing from `.agents/archive/planning-corpus/spikes/`:

 - OpenCode runtime control spike.
 - Policy enforcement gateway spike.
 - Tauri versus Electron browser/artifact spike.
 - Shell sandboxing spike.
 - MCP local and remote security spike.
 - Plugin validation and non-execution indexing spike.
 - SQLite concurrency and crash-recovery spike.
 - Large workspace performance spike.

Missing from `.agents/archive/planning-corpus/mvp/`:

 - MVP scope definition.
 - MVP audience and persona definition.
 - MVP acceptance criteria.
 - MVP non-goals.
 - MVP security baseline.
 - MVP workflow list.
 - MVP release readiness checklist.

Missing from `.agents/archive/planning-corpus/decisions/`:

 - Architecture freeze checklist.
 - Standards conformance matrix.
 - Policy authority decision summary.
 - Runtime feasibility decision summary.
 - Privacy and data movement decision summary.

Missing from `.agents/archive/planning-corpus/specs/`:

 - Operational specification.
 - Telemetry and privacy specification.
 - Testing and verification specification.
 - Runtime adapter specification.
 - Data portability/export specification.
 - Accessibility specification.
 - Error recovery specification.

Missing from `.agents/archive/planning-corpus/research/`:

 - OpenCode runtime API research.
 - Codex plugin schema stability research.
 - MCP threat-model research.
 - Desktop browser automation research.
 - Local shell sandboxing research.

Missing from `.agents/archive/planning-corpus/roadmap/`:

 - Research and spike roadmap.
 - Decision-gate roadmap.
 - MVP release roadmap.

## 5. Recommended Naming Conventions

Directory conventions:

 - `.agents/archive/planning-corpus/research/`: discovery, source notes, standards research, open research questions.
 - `.agents/archive/planning-corpus/specs/`: normative product, functional, architecture, security, data, UX, and subsystem specifications.
 - `.agents/archive/planning-corpus/reviews/`: critique, risk analysis, architecture reviews, adversarial reviews.
 - `.agents/archive/planning-corpus/adr/`: ADR indexes, priority lists, and individual ADR drafts.
 - `.agents/archive/planning-corpus/spikes/`: time-boxed technical validation plans and spike results.
 - `.agents/archive/planning-corpus/decisions/`: decision summaries, readiness gates, matrices, and organization records that are not ADRs.
 - `.agents/archive/planning-corpus/mvp/`: MVP scope, acceptance criteria, release gates, workflow lists, and non-goals.
 - `.agents/archive/planning-corpus/roadmap/`: phase sequencing, release plans, decision-gate plans, and research/spike sequencing.

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

