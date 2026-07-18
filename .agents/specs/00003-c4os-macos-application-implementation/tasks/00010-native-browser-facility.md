# Task 00010 — Native Browser Facility

Status: open

Coverage: ART-002, SET-010; support for UX-008, UX-010, ART-001, ART-006, ART-007.

## Summary

Integrate the selected Rust-owned native WKWebView controller with no page bridge, scoped Browser Environments, sanitized events, permissions, navigation, focus, clearing, isolation, and recovery.

## Implementation Steps

1. Reconfirm the exact compatible Tauri and objc2-web-kit lock and rerun the native proof before adopting its boundary constraints.
2. Implement the Rust-owned WKWebView lifecycle: attach, resize, focus, navigate, history, sanitized events, generation checks, crash recovery, and teardown without a page-to-app bridge.
3. Implement persistent and ephemeral Browser Environments, four accepted presets, guardrails, profile isolation, permission policy, downloads/popups states, scoped data clearing, and inactivation retention.
4. Implement shared Browser artifact shells, address/navigation controls, focus and accessibility, visible permission/failure/recovery states, and immutable Reply context.
5. Verify hostile-page isolation, credential exclusion, profile generation conflicts, storage clearing, relaunch, and native geometry across the acceptance matrix.

## Verification Process

- Exact-version Rust target tests and native proof rerun.
- Browser controller integration tests for lifecycle, profiles, permissions, generations, events, clearing, crashes, and teardown.
- Production native walkthroughs and screenshots across themes, sizes, focus states, failures, console, and overflow.

Acceptance criteria: none — implementation acceptance is delegated to the coordinator’s Agent Acceptance process for Spec 00003.

## Agent Acceptance

Result: failed — production native Browser evidence is absent.

Required evidence: exact dependency lock and proof rerun; controller tests; hostile-page and profile-isolation records; native screenshots; focus/accessibility/permission/recovery/overflow/console checks; evidence paths, commands, and limitations.

## Implementation Notes

Not started. Raw Wry and Tauri WebviewWindow approaches are rejected production boundaries.

## Verification Notes

Not run.

## Agent Acceptance Notes

A rendered fake web page or source-only inspection cannot pass this task.
