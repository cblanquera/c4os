# MVP Decisions

## DEC-001: Greenfield Restart

Status: accepted
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

C4OS is being treated as a greenfield restart from current plan documents. Existing prior planning records are product intent and evidence, not a required implementation sequence.

## DEC-002: Folder-First Trust Gate

Status: accepted
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

The first-run and main workspace experience is folder-first. Prompting is disabled until a trusted folder is available.

## DEC-003: Product Model Is App-Owned

Status: accepted
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

The product model centers app-owned workspaces, sessions, runs, approvals, artifacts, providers, files, browser, terminal, and settings rather than exposing runtime internals as the primary UX.

## DEC-004: Browser Promotion Requires Isolation Proof

Status: accepted
Confidence: imported
MVP: unknown
Phase: mvp
Source: `plans/product-brief.md`

Native Browser implementation should wait until cross-platform raw Wry behavior and privileged IPC isolation are proven.

## DEC-005: Extensions Disabled Before Runtime Impact

Status: accepted
Confidence: imported
MVP: no
Phase: feature-development
Source: `plans/product-brief.md`

Extensions should be disabled by default before they can affect runtime execution, model context, tools, hooks, or app-owned state.
