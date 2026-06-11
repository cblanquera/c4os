# Browser And Web Viewing Acceptance Criteria

## Current Decision

Status: accepted on 2026-06-12.

Accepted tier: `no_browser_or_web_viewing_v1`.

V1 should not add an in-app browser, remote URL viewing, DOM extraction,
screenshots, browser testing, browser automation, web content ingestion,
Chromium-backed rendering, generated HTML execution, or automatic
browser-content model-context ingestion.

## Acceptance Criteria

- User explicitly accepts, revises, or rejects `no_browser_or_web_viewing_v1`.
- Browser panels and remote URL viewing remain unavailable.
- DOM extraction, screenshots, browser automation, and browser testing remain
  unavailable.
- Browser and web content are not automatically ingested into model context.
- Chromium-backed rendering and active generated HTML execution remain
  deferred until renderer isolation, prompt-injection risk, storage impact,
  provenance, and model-context ingestion policy are specified.

## Out Of Scope

- In-app browser panels.
- Remote web page previews.
- DOM extraction or text extraction from browser pages.
- Screenshots or screenshot understanding.
- Browser automation and browser testing.
- Generated HTML execution.
- Chromium-backed rendering.
- Automatic browser-content model-context ingestion.
