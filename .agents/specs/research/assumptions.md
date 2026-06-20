# Research Assumptions

## ASM-001: Runtime Can Be Adapted

Status: accepted
Confidence: user-accepted
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`, `.agents/specs/research/poc/results.md`, `.agents/references/research/grill-session.md`

OpenCode can provide stable enough session control, streaming events, connector support, tools, permissions, and configuration APIs behind an app-owned adapter for the first implementation. Pi remains viable as a later adapter target.

## ASM-002: Provider Profiles Are Enough For MVP

Status: proposed
Confidence: imported
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`

OpenAI-compatible provider profiles can satisfy the first provider/model management needs without provider-native APIs.

## ASM-003: Concurrent Runs Can Be Isolated

Status: accepted
Confidence: user-accepted
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`, `.agents/references/research/grill-session.md`

The app can support concurrent agent runs across different trusted project folders without mixing approvals, tool output, working directories, artifacts, runtime events, or cancellation state. A single chat session remains single-main-run; subagent child runs are a future provisioned model concern, not an MVP execution requirement.
