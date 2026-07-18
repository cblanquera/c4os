# C4OS Usability And Interface Contract

## Status And Authority

- Status: Accepted reusable truth.
- Behavioral baseline: the complete accepted `r013-capability-aware-chat` single-page artifact, including preserved behavior inherited from r012 and the accepted capability-aware delta.
- Visual-interaction baseline: the structure, density, hierarchy, and interaction geometry captured by r013. `r012-cleanup` remains the historical parity profile.
- Visual direction that supersedes r012/r013 grayscale wireframe styling: C4OS follows the host operating system's default visual language and current light/dark preference.
- The KB contract is sufficient to reconstruct an equivalent accepted r013 experience without the `wireframes/` folder. Historical HTML, CSS, JavaScript, screenshots, and notes remain evidence, not required runtime inputs.
- “Equivalent” means the same screens, hierarchy, workflows, state transitions, responsive behavior, accessibility semantics, and reviewable content. It does not require preserving cleanup-only class names or illustrative IDs.
- The accepted Runtime and Session Architecture remains authoritative over forward product behavior. r013 now expresses its four approval presets, capability-aware model/session behavior, on-demand provenance, and simplified policy presentation. The literal r012 policy UI remains documented only as a historical reconstruction fixture.

## Product Model

C4OS is a desktop AI workspace organized around projects and chat sessions. A chat supports ordinary Markdown conversation and three structured response-artifact modes: Files, Browser, and Terminal. A response artifact can stay inline or, when its provider supports focus, replace the center conversation while the real thread moves into a contextual pane in the left sidebar.

The application is one stateful shell. Onboarding, workspace start, workspace modes, artifact focus, Settings, and Advanced Policies are views of the same application state, not unrelated products.

## Primary Actors And Goals

- New user: connect the first AI provider and open a workspace.
- Returning user: resume a recent workspace or open/clone another one.
- Builder: work through chat, attachments, files, browser pages, and a persistent terminal session without losing conversational context.
- Administrator/power user: manage providers, models, runtimes, plugins, skills, MCP servers, environment behavior, and per-tool approval policies.

## Top-Level Surfaces

| Surface | Stable route/state | Primary goal |
|---|---|---|
| First-provider onboarding | `#onboarding` | Configure one provider before use |
| Workspace start | `#start` | Open, clone, or resume a workspace |
| Chat workspace | `#chat` | Continue ordinary AI conversation |
| Capability-aware Chat fixture | `#chat-capabilities` | Review incompatible-attachment preflight and model-dependent controls |
| Files mode | `#files` | Open a file or browse a folder |
| Browser mode | `#browser` | Open and operate a webpage |
| Terminal mode | `#terminal` | Run a command in the chat's shell session |
| Providers | `#settings/providers` | Manage provider connections |
| Models | `#settings/models` | Choose available models |
| Runtimes | `#settings/runtimes` | Choose the execution runtime |
| Plugins | `#settings/plugins` | Discover and manage plugins |
| Skills | `#settings/skills` | Manage installed skills and availability |
| MCP Servers | `#settings/mcp` | Configure external tool servers |
| Configuration | `#settings/configuration` | Set approval and environment defaults |
| Advanced Policies | `#settings/advanced-policies` | Override policy per tool identity |

Production routing need not use URL hashes, but every surface must remain directly addressable for development and QA.

## Application Invariants

- Preserve in-memory workspace state while visiting Settings and returning.
- Never open an artifact automatically; focus is an explicit user action.
- Never reopen a collapsed left panel merely because an artifact receives focus.
- Keep the global composer fixed and usable while the transcript or a focused artifact scrolls.
- During artifact focus, lock the global composer to Chat, hide its mode chooser, and restore the prior mode after returning to Chat.
- Reply is always natural-language Chat, even when quoting a File, Browser, or Terminal artifact.
- User-facing work details are concise progress and tool activity, never private chain-of-thought.
- Keep historical assistant responses and artifacts static; animate only newly generated work.
- Keep artifact headers and footers outside the artifact body's scroll container.
- Use contextual actions that become visible on hover or focus without moving surrounding layout.
- Render intended product UI only; review notes, TODOs, annotations, and implementation commentary never appear in the product surface.

## Required Reading By Task

- Load [platform visual and theme contract](../references/00004-platform-visual-and-theme-contract.md) for styling, native integration, light/dark behavior, typography, controls, focus, motion, and responsive rules.
- Load [workspace shell and conversation contract](../references/00005-workspace-shell-and-conversation-contract.md) for projects, sessions, transcript, composer, attachments, Markdown, reply, resizing, and focus behavior.
- Load [response artifact contract](../references/00006-response-artifact-contract.md) for File, Folder, Browser, Terminal, shared artifact shells, scrolling, streaming, and artifact state transitions.
- Load [launch and settings contract](../references/00007-launch-and-settings-contract.md) for onboarding, start, Providers, Models, Runtimes, Plugins, Skills, MCP, Configuration, and Advanced Policies.
- Load [reconstruction fixtures and acceptance contract](../references/00008-reconstruction-fixtures-and-acceptance.md) when recreating the full review artifact or building deterministic QA fixtures.
- Load [wireframe decision ledger](../references/00009-wireframe-decision-ledger.md) when resolving why a behavior exists, identifying superseded directions, or syncing a later revision.
- Follow [Wireframe-to-KB Sync Workflow](../workflows/wireframe-kb-sync.md) for every new revision or material wireframe update.

## Accessibility Baseline

- All operations are keyboard reachable; hidden-on-idle actions remain in the focus order.
- Use semantic landmarks, headings, articles, forms, labels, dialogs, menus, tabs, switches, status regions, and live regions.
- Icon-only controls require accessible names and tooltips where the icon is not universally understood.
- Focus is always visibly indicated using a platform-appropriate ring with sufficient contrast.
- Popovers and dialogs close by explicit control and Escape; dialogs also support backdrop close unless data loss would result.
- Resizers support pointer and keyboard operation and expose current/min/max values to assistive technology.
- Generated responses expose busy state and announce completion without reading every streamed character.
- Reduced-motion preference removes typing and transition animation while preserving the final state and order.
- Color never carries the only indication of selected, active, missing, error, dirty, or disabled state.

## Production Boundaries

The wireframe simulated credentials, provider calls, filesystem access, file writes, browser navigation, shell execution, process signaling, AI generation, plugin/skill installation, MCP execution, persistence, native menus, and detached windows. The product must implement these through the accepted runtime/security architecture. The usability contract specifies the visible behavior and state transitions, not permission to bypass approvals, trust boundaries, or platform security.

Full-screen terminal programs, password-entry flows, browser sub-tabs, multiple simultaneous reply targets, and detached Chat windows remain outside the reconstructed r012 scope. `Detach Chat` is a visible future affordance and must remain non-destructive until a native window contract is accepted.

## Reconstruction Versus Forward Design

- To reproduce r012 for historical parity testing, use its three default approval choices and nine-group/71-identity policy browser as specified in the reconstruction reference.
- To reproduce the accepted forward baseline or create production UI, use r013's four presets (`Ask for approval`, `Approve safe actions`, `Approve for me`, `Custom`), seven user-facing policy groups, concrete exceptions, capability-aware model/attachment behavior, dynamic controls, honest activity presentation, and on-demand provenance.
- A later pending wireframe may demonstrate another forward model, but it does not replace r013 until reviewed and accepted.

## Source Provenance

- Processed sources: every revision-root `notes.md` and nested `qa/notes.md` from r001 through r013, plus the complete r012 and r013 specs and rendered artifact sources.
- r012 was browser-verified with no console errors across the workspace, Providers, Plugins, onboarding, and all fourteen workflow launcher destinations.
- r013 was browser-verified across capability-aware Chat, Models, Configuration, Advanced Policies, responsive toolbar states, and Chat information with no reported console errors or document-level overflow at the checked widths.
- Earlier review limitations are preserved in the decision ledger but do not override later verified behavior.
- The platform-native theme and system-following light/dark requirement comes from the user's accepted direction on 2026-07-18 and supersedes grayscale-only presentation requirements in structural wireframes.
