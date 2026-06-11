# Objective

Define post-MVP prerequisites for rich artifact previews and browser/web viewing.

# Context

MVP artifact scope is limited to plain text, Markdown, logs, diffs, and generated source or config files, with no active HTML rendering, browser-based rendering, Chromium dependency, rich media/document previews, duplicate/export workflows, or artifact execution. Reviews warn that generated HTML, documents, PDFs, browser pages, screenshots, and DOM extraction introduce security and prompt-injection risk.

# Questions To Answer

 - Which rich artifact types should be considered first after MVP?
 - Can generated HTML execute scripts in a future preview surface?
 - How are rich previews sandboxed?
 - Can browser content enter model context in a future browser surface, and under what explicit user action?
 - Are screenshots, DOM extraction, and text extraction different risk classes?
 - Which future features require Chromium consistency?

# Hypothesis

Post-MVP rich previews and browser features should not be added until renderer isolation, model-context ingestion, Chromium requirements, storage impact, and provenance are specified.

# Investigation Plan

 - Classify post-MVP artifact types by user value and risk.
 - Review Tauri, Electron, and controlled Chromium preview isolation options.
 - Threat-model generated HTML and remote browser pages.
 - Define safe post-MVP preview defaults.
 - Identify which browser capabilities belong in V1 or later.

# Success Criteria

 - Post-MVP rich artifact candidates are ranked.
 - Generated HTML execution policy is documented before active preview support.
 - Browser content ingestion policy is documented before browser support.
 - Chromium requirements and alternatives are identified.
 - V1 browser/artifact prerequisites are identified.

# Decisions Unlocked

 - ADR-017: Artifact Model.
 - ADR-018: Browser/Web Viewing.
 - ADR-002: Desktop Application Shell.

# Estimated Effort

2 to 4 engineering/security days.
