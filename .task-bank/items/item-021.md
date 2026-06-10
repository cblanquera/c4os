# item-021: TASK-021 Build Approval Prompt UI

## Status

verified

## Objective

Display action type, target, risk category, preview/summary state, and allowed
approval choices before execution.

## Dependencies

- item-014
- item-019

## Deliverables

- Approval prompt projection.
- File write preview states.
- Shell command prompt states.
- Destructive command prompt with no session allow.

## Verification

- Rust tests for file write, batch write, shell, destructive shell, Git state
  change, deny choice, and policy block passed.
- JS smoke test for status surface passed.
- Native Tauri build passed.
