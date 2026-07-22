# Task 00010 — Native Browser Facility

Status: verified

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

Result: passed — P0=0, P1=0, P2=0.

The final independent exact-source and evidence re-audit confirmed that production App < Workspace < Project < Chat Browser Environment precedence, exact persistent/ephemeral profile selection, public-WebKit ownership, sanitized request/history/Reply boundaries, explicit denial disposal, clear/relaunch recovery, destructive Close, full automated gates, persisted audits, secret scans, 13 clean native captures, and recorded limitations close every Task 00010 P0/P1 without a residual P2.

## Implementation Notes

Started 2026-07-22 from verified Task 00009 checkpoint `ff59999`. Raw Wry and Tauri `WebviewWindow` website surfaces remain rejected production boundaries. The smallest production-composed golden path is one active Chat submitting one direct Browser `Open`: Rust normalizes and policy-checks the address, creates one durable versioned Browser artifact, mounts one public-API native `WKWebView` child inside the existing focused artifact geometry with no page-accessible bridge, navigates one ordinary loopback page through a stable generation-bound controller, and publishes only bounded sanitized URL/title/loading/history state back through the artifact snapshot. Persistent/ephemeral Browser Environments, permission mediation, popups/downloads, clearing, crash recovery, immutable Reply context, and complete compact/focused UI must extend that path without making website JavaScript, Wry, or the renderer an authority.

## Verification Notes

The exact production dependency lock is Tauri 2.11.5, tauri-build 2.6.3, and objc2-web-kit 0.3.2. The isolated public-WebKit Proof passed all eight checks twice consecutively, then the production controller passed 24/24 focused Rust tests, 9/9 profile/configuration integration tests, 12/12 focused renderer tests, the complete 50-file/249-test renderer matrix, 41/41 serialized browser acceptance tests, clippy with denied warnings, the complete serialized Rust matrix, exact protocol/bundle verification, zero-vulnerability npm and Cargo advisories, and a final non-QA macOS bundle. Native evidence covers explicit Open/navigation approval and denial, Back/Forward/Refresh, redirects, popups, downloads, binary/non-GET blocks, camera denial, persistent and `None` environments, clearing, restart Recovery, immutable secret-free Reply context, destructive Close, Light/Dark themes, compact/wide geometry, focus/accessibility, console, and overflow. Full commands, timings, persisted identities, secret scans, screenshots, and limitations are recorded in `output/native/task-00010-acceptance.md`.

## Agent Acceptance Notes

A rendered fake web page or source-only inspection cannot pass this task. The independent reviewer directly inspected the final source, exact precedence test, all 13 accepted native captures, both persisted acceptance homes, profile/focus/security records, and secret-sentinel results. The prior configuration-precedence and evidence-completeness P1s are closed; final result P0=0, P1=0, P2=0.
