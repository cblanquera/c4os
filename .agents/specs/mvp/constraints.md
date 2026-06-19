# MVP Constraints

## CON-001: Local-First Storage

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

Project metadata, session metadata, approvals, artifacts, audit logs, local memory, and workspace state are stored locally by default.

## CON-002: Runtime Adapter Boundary

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

OpenCode Runtime or Pi Runtime should sit behind an app-owned adapter. Runtime details should not dominate the product UI.

## CON-003: OpenAI-Compatible Providers First

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

The first provider strategy uses OpenAI-compatible profiles before direct provider-native APIs.

## CON-004: Extension Execution Deferred

Status: proposed
Confidence: imported
MVP: no
Phase: feature-development
Source: `plans/product-brief.md`

Marketplace readiness and executable plugin/hook behavior are architecture goals, not MVP execution scope.

## CON-005: Neutral Utility UI

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-interface.md`

The MVP interface should be neutral and utilitarian, use lucide icons, and avoid dark theme or brand styling unless explicitly approved in a design phase.
