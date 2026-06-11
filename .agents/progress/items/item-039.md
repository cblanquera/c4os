# item-039: TASK-038 Scope V1 Rich Artifact Preview Support

## Status

verified

## Objective

Choose and document the next accepted V1 artifact preview support tier before
implementation.

## Dependencies

- item-038

## Scope

- Revisit richer artifact previews after the basic text/log/diff artifact
  surface and local stdio MCP sprint.
- Rank candidate preview types by user value and risk.
- Define whether any V1 rich preview tier is allowed, and if so which content
  types it includes.
- Define sandboxing, active-content, browser-content ingestion, model-context,
  provenance, storage, and Chromium/WebView requirements.
- Keep implementation deferred until acceptance criteria are explicit and
  accepted.

## Inputs

- `.agents/archive/planning-corpus/acceptance/artifacts.md`
- `.agents/archive/planning-corpus/decisions/deferred-decisions.md`
- `.agents/archive/planning-corpus/spikes/SPIKE-013-artifact-browser-preview-security.md`
- `.agents/archive/planning-corpus/specs/security-specification.md`
- `.agents/archive/planning-corpus/specs/architecture-specification.md`
- `.agents/progress/items/item-032.md`

## Deliverables

- Accepted or deferred V1 support tier for richer artifact previews.
- Updated artifact acceptance criteria.
- Updated deferred-decision and standards-conformance notes.
- Follow-on implementation item only if a narrow tier is accepted.

## Verification

- User accepted `raster_image_preview_only` on 2026-06-12.
- Acceptance criteria explicitly exclude active renderers, browser-content
  ingestion, Chromium requirements, export, search, OCR, image analysis, and
  automatic model-context behavior.

## Handoff

Proposed tier: `raster_image_preview_only`.

Accepted by the user on 2026-06-12. Implementation proceeds through
`item-040` and must not broaden beyond `raster_image_preview_only`.
