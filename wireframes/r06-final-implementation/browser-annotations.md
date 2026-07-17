# Browser Annotation Notes

Date: 2026-07-06

Status: markdown-only r06 Batch 5 note. The rendered r06 wireframe no longer
includes `#browser-annotations`.

## Scope

Browser annotation behavior is not represented as a standalone right-panel
wireframe route in r06 Batch 5. The visible Browser review surface is limited to:

- `./index.html#browser-navigation`
- `./index.html#browser-menu`
- `./index.html#browser-page-context-menu`
- `./index.html#browser-preview-host`

## Annotation Behavior

- Annotation capture may still be a product capability behind Browser controls.
- Multi-marker capture, comments, selectors, viewport metadata, screenshot
  evidence, prompt evidence bundles, and clear-after-send behavior should be
  handled as spec/acceptance behavior rather than rendered as a separate
  wireframe panel.
- `./index.html#browser-preview-host` should show only document preview hosting
  and document-family plugin output boundaries. It should not show annotation
  markers, annotation comments, or an annotation toolbar control.
- `./index.html#attachment-states` should stay focused on file and Browser
  screenshot attachment records in the rendered review artifact.
- `./index.html#debug-event-detail` should use non-annotation Browser evidence
  sample data in the rendered review artifact.

## Preview Host Helper Text

The following explanatory text was removed from
`./index.html#browser-preview-host` because it read like callouts instead of
product UI:

- Browser-native PDF preview
- DOCX/XLSX rendered by document-family plugins
- Browser hosts rendered output

Keep these as review notes only unless the product later needs real document
status metadata in the Browser chrome or document viewer.

## Review Boundary

Do not treat this Markdown note as an approved product specification update.
Specs 08, 09, 10, and directly affected 04 documentation stay unchanged until
this r06 Batch 5 review is explicitly approved.
