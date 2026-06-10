# item-023: TASK-023 Build Changed Files And Diff Viewer

## Status

verified

## Objective

Show changed files and file-level diffs tied to the active project and session
activity.

## Dependencies

- item-006
- item-019

## Deliverables

- Changed-file list.
- Diff viewer.
- Refresh behavior after tool execution.
- Tool-to-change context where available.

## Verification

- Temporary Git repository tests for modified, added, deleted, and untracked
  files passed.
- JS smoke test for status surface passed.
- Native Tauri build passed.
