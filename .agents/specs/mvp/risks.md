# MVP Risks

## RISK-001: Runtime Coupling

Status: open
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

If OpenCode or Pi concepts leak directly into the product model, C4OS may feel like a wrapper and become expensive to adapt.

## RISK-002: Security Boundary Drift

Status: open
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

Browser, terminal, artifact preview, file editing, and extensions can expand local authority unless trust records, approval policy, isolation, and audit logs are explicit.

## RISK-003: UI Scope Creep

Status: open
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

The desktop workspace can sprawl into IDE, browser, terminal, marketplace, memory manager, and admin console all at once.

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
