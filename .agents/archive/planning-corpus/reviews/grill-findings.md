# Grill Findings

## Summary

- BLOCKER: 1
- HIGH: 5
- MEDIUM: 5
- LOW: 2
- QUESTION: 2

Implementation planning should not freeze while the blocker remains open. The high findings must be resolved, accepted, or explicitly deferred before freeze.

## Findings

| ID | Severity | Title |
| --- | --- | --- |
| FINDING-001 | BLOCKER | OpenCode Runtime Control Is An Unproven MVP Gate |
| FINDING-002 | HIGH | Backend Approval Gateway Depends On Runtime Cooperation |
| FINDING-003 | HIGH | Shell Safety Still Lacks Concrete Redaction And Classification Limits |
| FINDING-004 | HIGH | OpenRouter-Only Model Routing Needs Runtime-Level Verification |
| FINDING-005 | HIGH | Runtime-Native Instruction Loading Can Contradict AGENTS.md Display-Only Scope |
| FINDING-006 | HIGH | Old Readiness Gap Rollup Conflicts With Later Finalized Decisions |
| FINDING-007 | MEDIUM | Plugin And Marketplace Specs Remain Broader Than MVP |
| FINDING-008 | MEDIUM | Local Indefinite Retention Has No Cleanup Or Deletion Story |
| FINDING-009 | MEDIUM | Tauri Choice Is Reasonable But Still Needs Platform Validation |
| FINDING-010 | MEDIUM | Append-Only Transcript Integrity Is Strong But UX Recovery Is Thin |
| FINDING-011 | MEDIUM | Acceptance Criteria Are Strong But Some Are Phase-0 Gates, Not Product QA |
| FINDING-012 | LOW | Preplan Brief Still Says To Create Plans Even Though Corpus Exists |
| FINDING-013 | LOW | ADR Numbering And Rollup Naming Are Hard To Follow |
| FINDING-014 | QUESTION | What Is The Accepted Runtime Path If Direct OpenCode Fails? |
| FINDING-015 | QUESTION | Which Target Platforms Must Pass Tauri Validation? |

## Blocker

The direct OpenCode runtime path is not implementation-ready until the runtime-control matrix proves structured events, pre-execution interception, stop semantics, model/provider control, app-owned record mapping, and instruction-loading observability.

## High-Risk Themes

- The Approval Gateway is correctly designed as backend-authoritative, but that authority still depends on runtime cooperation.
- Shell execution is intentionally powerful and must have concrete redaction, truncation, destructive classification, and environment-filtering evidence.
- OpenRouter-only provider scope is acceptable only if OpenCode cannot silently override model or credential routing.
- `AGENTS.md` display-only scope is acceptable only if runtime-native instruction loading is observable, disclosed, disabled, or otherwise controlled.
- The readiness gap rollup needs reconciliation with later finalized and deferred decisions.

## Medium-Risk Themes

- Plugin and marketplace documents should remain post-MVP reference and not leak into MVP implementation.
- Local indefinite retention is acceptable for MVP only if treated as an explicit tradeoff.
- Tauri remains reasonable, but target platform validation is still needed.
- Append-only transcript semantics are strong for integrity but need UX validation around failed and stopped records.
- Phase 0 gates should be separated from ordinary product acceptance criteria.

## Recommended Readiness Decision

Current state: Not Ready.

Freeze-and-plan may begin only after FINDING-001 is resolved and the high findings are either resolved, accepted as explicit MVP risk, or deferred with documented rationale.
