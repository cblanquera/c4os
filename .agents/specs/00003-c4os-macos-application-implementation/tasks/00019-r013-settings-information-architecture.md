# Task 00019 — r013 Settings Information Architecture

Status: verified

Coverage: corrective ownership for SET-003 through SET-011; support for UI-004, UI-005, UX-009, and UX-012.

## Summary

Converge every production Settings destination on r013's compact, scan-oriented information architecture while retaining real services and the accepted forward four-preset/seven-group policy model. Remove duplicated headings, misplaced controls, security implementation prose, and cross-route responsibility leaks.

## Implementation Steps

1. Use one Settings title context and one concise page header; remove duplicated route titles and oversized generic heading composition.
2. Keep the 224px persistent/compressed navigation, 920px content cap, independent scrolling, native Settings entry, Back restoration, and shared responsive behavior.
3. Limit Providers rows to provider identity/family, availability, and Edit. Place connection Test and destructive removal inside the shared Edit flow where context is explicit; remove selected-model controls and routine credential-architecture prose from the list.
4. Keep model selection, capability evidence, provider/capability filters, availability, details, bulk actions, and refresh feedback in Models.
5. Recompose Runtimes, Configuration, Plugins, Skills, and MCP Servers with r013/r012 row, card, toolbar, dialog, empty, loading, failure, and notifier geometry while retaining real lifecycle behavior.
6. Preserve exactly four approval presets and seven Advanced Policy groups plus separate Exceptions. Do not reintroduce historical r012 policy identities as product rows.
7. Apply Task 00016's direct-intent behavior so explicit Settings actions do not create duplicate prompts; preserve managed denial, explicit `Ask`, destructive confirmation, and truthful pending state.
8. Verify all route transitions, dialogs, focus restoration, writer locking, long content, and narrow navigation without implementation annotations or QA copy in production.

## Verification Process

- Route/component tests for responsibility boundaries, heading hierarchy, provider/model separation, CRUD placement, policy presets/groups, dialogs, notices, and focus.
- Service-backed tests proving the visual reorganization does not weaken credential, provider, model, runtime, extension, MCP, configuration, or policy authority.
- Playwright wide/narrow Settings matrix with Light/Dark screenshots, console checks, and overflow assertions.
- Native menu/`Cmd+,`/Back walkthrough and r013 comparison for every Settings destination.

## Acceptance Criteria

The user reviews a production Settings board and walkthrough covering Providers, Models, Runtimes, Configuration, Advanced Policies, Plugins, Skills, and MCP Servers at wide and narrow widths, including representative dialogs and failure states.

## Implementation Notes

Started after Tasks 00016 and 00017. Work is restricted to Settings composition and responsibility boundaries while the existing provider, model, runtime, configuration, policy, extension, and MCP services remain authoritative.

- Provider rows now contain only identity/family, availability, and Edit. Exact connection testing and destructive removal live in the Edit dialog, and Provider forms no longer render model selection.
- Models retains capability evidence, filtering, availability, details, bulk action, and refresh ownership. Runtime selection and its explicit Save boundary remain isolated in Runtimes.
- Configuration no longer embeds the Updates and Diagnostics surfaces. Its four accepted approval presets, app defaults, Advanced Policies transition, and external configuration-file action remain intact.
- Plugin tabs now precede their content; marketplace management lives within Directory. Skill scan rows retain identity, state, availability, and Details while qualified identity, precedence, collisions, invalid frontmatter, and instructions stay in Details.
- MCP uses a concise header and compact service/row copy while preserving trust, testing, enable/disable, recovery, revocation, deletion, and transport configuration.
- Advanced Policies keeps exactly seven groups, four policy values, separate Exceptions, and header-level Save/Revert state. At narrow widths, its group selector becomes a horizontal scroll rail.
- Shared Settings pages now start at one consistent content edge without the former route-specific 26px top offset.

## Verification Notes

Automated component verification passed on 2026-07-27:

- `npm run typecheck -- --pretty false`
- `npm run lint -- --max-warnings=0`
- 75 focused Settings/shell checks across all eight destinations, including provider/model responsibility, service-backed mutations, dialog behavior, exactly four presets/seven policy groups, and the horizontal narrow policy rail. Existing MCP tests emit known React `act(...)` advisories but pass.

Task 00022 completed current wide/narrow captures for all eight accepted Settings destinations, the shared provider dialog, renderer and native Light/Dark Providers views, exact Settings Back-state restoration, all direct route checks, and the final heading/responsibility review. Independent audit found one duplicate Providers title; the component-owned title/support were removed, route ownership is now exclusive, and a regression test passed inside the final 472-test renderer gate. The 47/47 Playwright matrix passed; see `.agents/resources/native/task-00022-review-package.md`.

## Acceptance Notes

Integrated browser and native verification passed in Task 00022, including exact Settings Back restoration and the final Providers heading hierarchy. Explicit user review remains pending.
