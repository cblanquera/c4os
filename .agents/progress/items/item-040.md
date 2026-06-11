# item-040: TASK-039 Build Passive Raster Image Artifact Previews

## Status

verified

## Objective

Implement the accepted `raster_image_preview_only` artifact preview tier.

## Dependencies

- item-039

## Scope

- Support passive local PNG, JPEG, WebP, and GIF artifact previews.
- Preserve text, Markdown, log, diff, source, and config artifact behavior.
- Keep active HTML, SVG, PDF, document, spreadsheet, browser rendering, remote
  URL previews, execution, export, duplicate workflows, search, OCR, image
  analysis, and automatic model-context ingestion unavailable.

## Inputs

- `.agents/progress/items/item-039.md`
- `.agents/archive/planning-corpus/acceptance/artifacts.md`
- `.agents/archive/planning-corpus/spikes/SPIKE-013-artifact-browser-preview-security.md`

## Deliverables

- Raster image artifact capture API for accepted MIME/signature pairs.
- Artifact viewer projection for passive local image previews.
- App status fields for `raster_image_preview_only`.
- Tests proving passive image previews and excluded SVG/HTML/remote inputs.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml artifact`.
- `npm test -- tests/backend-command-boundary.test.mjs`.
- `npm test`.
- `npm run build`.
- `cargo test --manifest-path src-tauri/Cargo.toml`.
- `npm run tauri -- build`.

## Result

- Added passive local raster image artifact capture and viewer projection.
- App status now reports accepted image preview support and unsupported rich
  preview types.
- UI status summarizes artifacts as text plus raster images.
- No active renderer, browser surface, export, search, OCR, image analysis, or
  automatic model-context path was added.
