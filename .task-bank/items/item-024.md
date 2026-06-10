# item-024: TASK-024 Implement Text-Like Artifact Records And Viewer

## Status

verified

## Objective

Capture and reopen MVP artifacts for text, logs, diffs, and generated source or
config files without rich rendering.

## Dependencies

- item-002
- item-019

## Deliverables

- Artifact record model.
- Artifact file storage layout.
- Text-like artifact viewer.
- Artifact link from session/tool context.

## Verification

- Persistence tests passed.
- UI tests for text/log/diff/generated-file artifacts passed.
- Native Tauri build passed.
