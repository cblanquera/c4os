# Research Risks

## RISK-001: Runtime Coupling

Status: mitigated
Confidence: evidence-backed
MVP: yes
Phase: poc
Source: `plans/product-brief.md`, `.agents/specs/research/poc/results.md`

If OpenCode or Pi concepts leak directly into the product model, C4OS may feel like a wrapper and become expensive to adapt. The runtime POCs mitigate this by choosing a thin C4OS-owned adapter boundary and keeping runtime-specific concepts behind that boundary.

## RISK-002: Security Boundary Drift

Status: mitigated
Confidence: evidence-backed
MVP: yes
Phase: poc
Source: `plans/product-brief.md`, `.agents/specs/research/poc/results.md`

Browser, terminal, artifact preview, file editing, and extensions can expand local authority unless trust records, approval policy, authority rules, and audit logs are explicit. Grill decisions promote Browser, Terminal, and extension execution into MVP scope, so implementation must preserve agent authority boundaries and audit records rather than treating earlier POC limits as scope deferrals.

## RISK-003: UI Scope Creep

Status: mitigated
Confidence: evidence-backed
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`, `wireframes/ui-handoff-spec.md`

The desktop workspace can sprawl into IDE, browser, terminal, marketplace, memory manager, and admin console all at once. TASK-005 mitigates this by accepting r04 as a behavioral handoff, fixing the MVP shell, composer, tool-tab, and settings boundaries, and treating extension-heavy surfaces as inventory or configuration unless separately promoted.

## RISK-004: Provider Complexity

Status: open
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

Provider-native APIs differ widely and could consume MVP scope if the first version does not stay OpenAI-compatible.

## RISK-005: Persistence Confusion

Status: open
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

Users may expect workspace files to carry all state unless `.c4os` descriptors and app-owned local storage boundaries are clear.
