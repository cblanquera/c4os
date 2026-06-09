# Proposed ADR Priority Order

This priority order is based on architectural blast radius, security impact, and unresolved risk identified in the review.

## Priority 0: Must Resolve Before Architecture Freeze

1. ADR-001: Product Scope And MVP Audience
2. ADR-003: Agent Runtime Strategy
3. ADR-004: Policy Enforcement Authority
4. ADR-010: Filesystem And Shell Access Defaults
5. ADR-011: MCP Integration Strategy
6. ADR-014: Plugin System Compatibility
7. ADR-016: Plugin Trust And Execution
8. ADR-005: Standards-First Interoperability

## Priority 1: Must Resolve Before Detailed Implementation Planning

9. ADR-002: Desktop Application Shell
10. ADR-008: Unified Tool Invocation And Ledger
11. ADR-009: Permission And Approval Model
12. ADR-019: Model Provider Strategy
13. ADR-007: Local-First Storage Model
14. ADR-015: Marketplace Model

## Priority 2: Resolve Before MVP Scope Lock

15. ADR-006: Project, Session, And Worktree Model
16. ADR-017: Artifact Model
17. ADR-018: Browser/Web Viewing
18. ADR-022: MVP Roadmap Sequencing
19. ADR-023: Telemetry And Privacy

## Priority 3: Resolve During Product Definition

20. ADR-012: Agent Skills Support
21. ADR-013: AGENTS.md Instruction Support
22. ADR-020: Session And Memory Management
23. ADR-021: UX Layout And Interaction Model

## Rationale

The top group contains decisions that can invalidate major parts of the architecture if answered differently. Runtime strategy and policy enforcement are especially coupled: if OpenCode cannot provide pre-execution control, the current security model changes materially. Product scope is equally important because the current documents say "general-purpose" while the concrete MVP is mostly coding-agent shaped.

