# Objective

Determine the safe MVP scope for artifact previews and browser/web viewing.

# Context

Specs include artifact viewing and browser/web content. Reviews warn that generated HTML, documents, PDFs, browser pages, screenshots, and DOM extraction introduce security and prompt-injection risk.

# Questions To Answer

 - Which artifact types are required for MVP?
 - Can generated HTML execute scripts?
 - How are previews sandboxed?
 - Can browser content enter model context automatically?
 - Are screenshots, DOM extraction, and text extraction different risk classes?
 - Is an in-app browser needed for MVP?

# Hypothesis

MVP should limit artifacts to text, Markdown, logs, diffs, and generated files without active HTML execution or browser ingestion.

# Investigation Plan

 - Classify artifact types by MVP need and risk.
 - Review Tauri preview isolation options.
 - Threat-model generated HTML and remote browser pages.
 - Define safe preview defaults.
 - Identify which browser capabilities belong in V1 or later.

# Success Criteria

 - MVP artifact preview set is defined.
 - Generated HTML execution policy is documented.
 - Browser content ingestion policy is documented.
 - V1 browser/artifact prerequisites are identified.

# Decisions Unlocked

 - ADR-017: Artifact Model.
 - ADR-018: Browser/Web Viewing.
 - ADR-002: Desktop Application Shell.

# Estimated Effort

2 to 4 engineering/security days.

