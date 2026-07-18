# Task 00013 — Onboarding And Settings Integration

Status: open

Coverage: SET-001, SET-004, SET-005, SET-006; integrated verification for SET-002 through SET-011 and UX-009.

## Summary

Complete onboarding, C4OS Home, and every Settings destination against real Provider, Credential, Capability, Runtime, Extension, MCP, Configuration, Policy, Workspace, and update services.

## Implementation Steps

1. Implement onboarding connection tests whose latest success gates Continue and is invalidated by relevant edits, including zero-model handling and recommended model/runtime confirmation.
2. Implement real provider CRUD/test/enable flows with opaque secret references, conditional fields, secure preservation, persistence, and failure recovery.
3. Implement model search/filter/details/bulk/refresh from CapabilityService freshness and authoritative availability.
4. Implement runtime defaults with persistence and explicit existing-versus-new Chat binding behavior.
5. Integrate C4OS Home, Settings shell, Plugins, Skills, MCP, Configuration, Advanced Policies, updates, notices, Back/round-trip state, native entry, and compressed navigation.
6. Verify all normal, loading, empty, error, degraded, dirty, conflict, revoked, and recovery states across themes, widths, keyboard, and restart.

## Verification Process

- Service integration tests for test invalidation, credentials, provider/model/runtime persistence, capability refresh, and cross-domain generations.
- Renderer tests and production Playwright/native walkthroughs for onboarding, Home, every Settings route, dialogs, notices, Back, compression, focus, accessibility, console, and overflow.
- Restart tests proving secrets never enter renderer state or logs and Settings changes reach authoritative services.

Acceptance criteria: none — implementation acceptance is delegated to the coordinator’s Agent Acceptance process for Spec 00003.

## Agent Acceptance

Result: failed — real-service onboarding and Settings evidence is absent.

Required evidence: automated service and UI results; secret-boundary inspection; production screenshots for every Settings destination and material state; native menu/shortcut round trip; keyboard/accessibility/overflow/console checks; evidence paths, commands, and limitations.

## Implementation Notes

Not started. Hardcoded r013 records and simulated connection success cannot be reused as authority.

## Verification Notes

Not run.

## Agent Acceptance Notes

The coordinator must exercise real persisted changes and their visible consequences.
