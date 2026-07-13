# Research Questions

## Q-001: First Runtime Target

Status: answered
Confidence: user-accepted
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`, `.agents/specs/research/poc/results.md`, `.agents/references/research/grill-session.md`

The first implementation should target OpenCode behind a thin C4OS-owned runtime adapter because OpenCode supports connectors and is less raw than Pi for the first product slice. Pi remains a later adapter target only.

## Q-002: Browser MVP Boundary

Status: answered
Confidence: user-accepted
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`, `.agents/specs/research/poc/results.md`, `.agents/references/research/grill-session.md`

The MVP Browser tab is a user-owned desktop Browser with project-scoped profile, public web access, local file browsing, request-scoped agent browsing, and logged-in Browser use when the user's request clearly requires it. Agent file and Browser actions must be recorded. Downloads are not MVP.

## Q-003: Terminal MVP Boundary

Status: answered
Confidence: user-accepted
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`, `.agents/specs/research/poc/results.md`, `.agents/references/research/grill-session.md`

The MVP Terminal tab should expose separate user terminal and agent-owned command terminal surfaces. The agent-owned command terminal may run read-only allowlisted commands without prompting, may run non-allowlisted commands inside the active project after per-command approval, and may run outside-project commands only with explicit permission. C4OS must record command text, working directory, permission decisions, exit status, and output summary.

## Q-004: Concurrent Runs In First MVP

Status: answered
Confidence: user-accepted
MVP: yes
Phase: mvp
Source: `plans/product-brief.md`, `.agents/references/research/grill-session.md`

The first MVP requires concurrent agent runs across different trusted project folders. Each chat session has only one main run active at a time, but multiple chat sessions can run at the same time. Future subagent child runs should be provisioned in the model, but subagent spawning does not need to ship in MVP.

## Q-005: Wireframe Acceptance Boundary

Status: answered
Confidence: evidence-backed
MVP: yes
Phase: mvp
Source: `wireframes/ui-handoff-spec.md`

r04 should be treated as behavioral implementation handoff for MVP UI scope. Layout regions, state ownership, interaction placement, trust boundaries, tool surfaces, and settings IA are accepted; sample content, exact copy, placeholder data, and grayscale visual styling are illustrative.

## Q-006: Agent Authority Boundary

Status: answered
Confidence: user-accepted
MVP: yes
Phase: mvp
Source: `.agents/references/research/grill-session.md`

The active project folder is the default agent work boundary. Inside the active project, an agent may read and write files freely when the user's request implies file work. Outside-project file reads and writes require explicit permission. Public web browsing and logged-in Browser use are allowed when clearly within the user's request, and C4OS must record agent file and Browser actions.

## Q-007: Extension MVP Boundary

Status: answered
Confidence: user-accepted
MVP: yes
Phase: mvp
Source: `.agents/references/research/grill-session.md`

The first MVP includes install/connect flows for plugins, skills, and MCP servers. Extensions can affect agent execution only after explicit per-extension enablement. Skills and plugins may be invoked implicitly by enabled "use when" routing or explicitly with `$skill` and `@plugin` tags. MCP servers require explicit `^mcp` invocation.
