# item-047: TASK-046 Resolve V1 Browser And Web Viewing Scope

## Status

verified

## Objective

Choose and document whether V1 should add browser or web-viewing features now
that raster artifact previews, provider scope, and compatibility claims have
been bounded.

## Decision

Accepted `no_browser_or_web_viewing_v1` on 2026-06-12.

V1 should not add an in-app browser, remote URL viewing, DOM extraction,
screenshots, browser testing, browser automation, web content ingestion,
Chromium-backed rendering, generated HTML execution, or automatic
browser-content model-context ingestion.

## Dependencies

- item-040
- item-045
- item-046

## Inputs

- `CONTEXT.md`
- `.agents/archive/planning-corpus/acceptance/browser-web-viewing.md`
- `.agents/archive/planning-corpus/acceptance/artifacts.md`
- `.agents/archive/planning-corpus/decisions/deferred-decisions.md`
- `.agents/archive/planning-corpus/decisions/non-goals.md`
- `.agents/archive/planning-corpus/spikes/SPIKE-013-artifact-browser-preview-security.md`

## Deliverables

- Accepted, revised, or rejected V1 browser/web-viewing support tier.
- Updated acceptance criteria for browser surfaces, web content ingestion, and
  active rendering.
- Follow-on implementation item only if V1 app status or UI needs code changes
  after acceptance.

## Acceptance Criteria

- User explicitly accepts, revises, or rejects the support tier.
- Browser panels, remote URL viewing, DOM extraction, screenshots, browser
  testing, browser automation, and browser-content ingestion are explicitly
  accepted or deferred.
- Chromium-backed rendering and generated HTML execution are explicitly
  accepted or deferred.
- Model-context behavior for browser/web content remains explicit.

## Verification

- User accepted `no_browser_or_web_viewing_v1`.
- App status reports the accepted browser and web-viewing tier.
- `cargo test --manifest-path src-tauri/Cargo.toml app_status_reports_no_browser_or_web_viewing_tier`.
- `npm test -- tests/backend-command-boundary.test.mjs`.
- `npm test`.
- `npm run build`.
- `cargo test --manifest-path src-tauri/Cargo.toml`.
- `npm run tauri -- build`.

## Follow-On

- No browser or web-viewing implementation is required because the accepted
  tier keeps those surfaces deferred.
