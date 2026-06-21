# MVP Evidence

Status: ready-for-review
Updated: 2026-06-21

## Evidence Records

### EVD-001: Product Brief

Status: accepted
Source: `plans/product-brief.md`

Defines the greenfield restart, target users, product thesis, security model,
core workflows, feature areas, data model concepts, success metrics, and risks.

### EVD-002: Interface Brief

Status: accepted
Source: `plans/product-interface.md`

Defines the three-panel shell, composer, provider/model selectors, session
thread, Browser/Files/Terminal tab rules, and settings screens.

### EVD-003: Wireframe Pegs

Status: accepted
Source: `plans/pegs/*.png`, `wireframes/screens.md`

Provide visual references for app start, new session, chat session,
provider/model popovers, settings, file explorer/editor, and terminal surfaces.

### EVD-004: Runtime Adapter POCs

Status: accepted
Source:
- `proofs/opencode-runtime/opencode-runtime-evidence-2026-06-20.md`
- `proofs/pi-runtime/pi-runtime-evidence-2026-06-20.md`
- `.agents/specs/research/poc/results.md`

OpenCode and Pi were compared through a shared adapter lens. OpenCode is viable
for the first server-backed runtime adapter; Pi remains viable as a later
adapter target.

### EVD-005: Browser POCs And ADR

Status: accepted
Source:
- `proofs/native-browser-plugin/native-browser-plugin-evidence-2026-06-20.md`
- `proofs/native-browser-wry/native-browser-wry-evidence-2026-06-20.md`
- `proofs/native-browser-tauri/native-browser-tauri-evidence-2026-06-20.md`
- `docs/adr/0001-browser-and-agent-authority.md`

Browser isolation proofs identified implementation risks, and the ADR promotes
Browser into MVP as a user-owned desktop surface with audit boundaries.

### EVD-006: Terminal POCs

Status: accepted
Source:
- `proofs/native-terminal-plugin/native-terminal-plugin-evidence-2026-06-20.md`
- `proofs/native-terminal-tauri/native-terminal-tauri-evidence-2026-06-20.md`

Backend-owned terminal lifecycle is viable on macOS, including trusted-root
validation, sanitized ownership, streaming, resize observation, approval-gated
launches, failure reporting, cancellation, and cleanup.

### EVD-007: r04 UI Handoff

Status: accepted
Source:
- `wireframes/r04-single-page-app/README.md`
- `wireframes/r04-single-page-app/index.html`
- `wireframes/ui-handoff-spec.md`
- `.agents/context/ui-handoff.md`

r04 is accepted as behavioral handoff for the MVP shell, state model,
navigation, composer controls, structured events, right tool surfaces, settings
IA, and placeholder-data guard.

### EVD-008: Extension ADR

Status: accepted
Source: `docs/adr/0002-extension-enablements-and-prompt-tags.md`

Confirms plugin, skill, and MCP install/connect are MVP, runtime impact requires
explicit enablement, and prompt tags are `$skill`, `@plugin`, and `^mcp`.

### EVD-009: Frozen Research Closeout

Status: accepted
Source:
- `.agents/specs/research/research-freeze.md`
- `.agents/specs/research/viability-gaps.md`
- `.agents/specs/research/implementation-checkpoint-plan.md`

Confirms research is frozen for MVP specification, the full documented/r04
candidate scope remains included, and checkpoints are sequencing gates only.

