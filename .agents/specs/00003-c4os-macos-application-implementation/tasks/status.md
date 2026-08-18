# Spec 00003 Task Status

Plan state: Original phase accepted 2026-07-18; corrective r013 convergence phase accepted 2026-07-27. Historical Tasks 00001 through 00015C retain `open`, `started`, and `verified`. Corrective Tasks 00016 onward also use `accepted` after explicit user review of their human-reviewable output.

| Task | Status | Acceptance | Coverage focus |
| --- | --- | --- | --- |
| [00001 Foundation and dependency lock](00001-foundation-and-dependency-lock.md) | verified | passed | Foundation for all IDs |
| [00002 Durable core, configuration, and Workspace lifecycle](00002-durable-core-configuration-workspace.md) | verified | passed | UX-013, UX-014, SET-002 |
| [00003 Policy, credentials, Action Gateway, and execution environments](00003-policy-credentials-action-gateway.md) | verified | passed | UX-010, UX-012, UX-015, CHAT-009, CHAT-010, SET-011 |
| [00004 Runtime adapters, providers, and capability lifecycle](00004-runtime-provider-capability-lifecycle.md) | verified | passed | Provider/runtime support for CHAT and SET |
| [00005 Native macOS platform and semantic UI foundation](00005-native-macos-semantic-ui.md) | verified | passed | UX-003, UI-001 through UI-005, SET-003 |
| [00006 Stateful renderer shell and accessible component system](00006-renderer-shell-component-system.md) | verified | passed | UX-001, UX-004, UX-011 |
| [00007 Workspace, Chat, composer, and conversation](00007-workspace-chat-composer.md) | verified | passed | UX-005 through UX-007, CHAT-001 through CHAT-008, ART-006 |
| [00008 Artifact framework and File/Folder facilities](00008-artifact-file-folder.md) | verified | passed | UX-008, ART-001, ART-003, ART-004, ART-007 |
| [00009 Terminal facility](00009-terminal-facility.md) | verified | passed | ART-005 |
| [00010 Native Browser facility](00010-native-browser-facility.md) | verified | passed | ART-002, SET-010 |
| [00011 Plugin and Skill lifecycle](00011-plugin-skill-lifecycle.md) | verified | passed | SET-007, SET-008 |
| [00012 MCP lifecycle](00012-mcp-lifecycle.md) | verified | passed | SET-009 |
| [00013 Onboarding and Settings integration](00013-onboarding-settings-integration.md) | verified | passed | SET-001, SET-004 through SET-006 |
| [00014 Updates, recovery, diagnostics, and degraded-state integration](00014-updates-recovery-diagnostics.md) | verified | passed | UX-010, UX-013; update/recovery support |
| [00015 Deterministic QA and integrated Agent Acceptance](00015-integrated-agent-acceptance.md) | verified | passed | UX-002, UX-009, QA-001, QA-002 |
| [00015A Security audit](00015A-security-audit.md) | verified | passed | Cross-cutting security audit |
| [00015B Accessibility and native macOS audit](00015B-accessibility-native-audit.md) | verified | passed | Cross-cutting rendered/native audit |
| [00015C Coverage and closeout audit](00015C-coverage-closeout-audit.md) | verified | passed | All 50 IDs and final records |
| [00016 Practical direct intent and credentials](00016-practical-direct-intent-and-credentials.md) | verified | production-native secure-storage create/read/restart plus integrated verification passed; user review required | Practical direct actions, credentials, provider journey |
| [00017 r013 visual foundation and shell geometry](00017-r013-visual-foundation-and-shell-geometry.md) | verified | integrated renderer/native/responsive verification passed; user review required | Shared r013 geometry, density, shell composition |
| [00018 r013 onboarding and Workspace Start](00018-r013-onboarding-and-workspace-start.md) | verified | no-picker correction and refreshed journey evidence passed; user review required | First-run and returning-user launch journeys |
| [00019 r013 Settings information architecture](00019-r013-settings-information-architecture.md) | verified | integrated browser/native verification passed; user review required | Settings hierarchy and route responsibilities |
| [00020 r013 Workspace, Chat, and composer](00020-r013-workspace-chat-and-composer.md) | verified | integrated renderer/Playwright/native verification passed; user review required | Primary Workspace and capability-aware Chat |
| [00021 r013 artifact and responsive convergence](00021-r013-artifact-and-responsive-convergence.md) | verified | integrated provider/authority/responsive verification passed; user review required | Artifact shells, focus, contextual Chat, responsiveness |
| [00022 r013 integrated human acceptance](00022-r013-integrated-human-acceptance.md) | started | complete review package ready; explicit user acceptance required | Complete r013 production comparison and closeout |
| [00022A onboarding automatic model selection](00022A-onboarding-automatic-model-selection.md) | verified | implementation and refreshed evidence passed; explicit user review required | SET-001 no-picker onboarding and automatic strongest-model selection |
| [00022B onboarding Continue recovery](00022B-onboarding-continue-recovery.md) | verified | production-native Test/Continue, secure-storage restart read-back, fresh Chat, and durable post-restart retry passed; explicit parent review remains | SET-001 secure Continue persistence and retry recovery |

## Current Blockers

There is no remaining implementation blocker for Task 00022B. The user completed the production-native connection flow; secure storage survived its intentional recovery relaunch; the provider read back without a session-only warning; a fresh production Chat completed in one native step; and a previously failed durable Chat retried successfully after a full app/runtime restart. The corrective milestone remains open only for explicit visual/functional acceptance through Task 00022. Historical Tasks 00001 through 00015C remain verified. Signing, notarization, distribution, signed updater evidence, public marketplace governance, other-platform claims, and explicitly deferred product scope remain external gates.
