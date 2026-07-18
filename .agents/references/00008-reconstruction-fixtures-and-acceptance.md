# Reconstruction Fixtures And Acceptance

## Purpose

Use this reference to create a deterministic review build equivalent to the accepted r013 baseline. Values are illustrative QA fixtures unless another Context File promotes them to product configuration.

For historical r012 parity, use the literal profile preserved in `00007-launch-and-settings-contract.md`. Current reconstruction applies the forward approval, policy, capability, and provenance requirements in `runtime-session-architecture.md` and r013.

## Required Artifact Files

An equivalent static review artifact may use different internal organization, but must provide:

- one directly openable application entry
- one workflow launcher with all sixteen destinations
- platform/system theme support
- styles and behavior requiring no build step for review
- a source-preserving Markdown editor runtime
- companion reconstruction spec and review notes when it is emitted as a wireframe revision

All document links/imports are relative. Product UI contains no review annotations.

## Workflow Launcher Destinations

1. No Provider → onboarding
2. Start Screen → workspace start
3. Workspace Projects → Chat
4. Search Chat Sessions → seeded `#chat-search` results
5. Chat
6. Files
7. Browser
8. Terminal
9. Providers
10. Model Capabilities → Models
11. Runtimes
12. Plugins
13. Skills
14. MCP Servers
15. Configuration
16. Advanced Policies

## Project Fixtures

| Project | Path | State | Sessions |
|---|---|---|---|
| AI Desktop UI | `~/Documents/AI Desktop UI` | found, expanded, active | Build onboarding start screens; Design workspace projects; Refine response artifacts |
| quotable-ai | `~/Projects/shoppable/quotable-ai` | found, collapsed/available | Establish project knowledge base; Review provider settings |
| legacy-ui | `~/Projects/archive/legacy-ui` | missing, lighter italic | Recover legacy workspace |

The exact active session can vary by workflow route, but the fixture must exercise found/missing, expanded/collapsed, active/inactive, search, menus, sorting, pending chat, promotion, and removal fallback.

The `#chat-search` fixture seeds `project`, replaces Projects with flat matches for `Design workspace projects` and `Establish project knowledge base`, shows each owning project, preserves the query when a result opens, exposes a no-results state for unmatched text, and restores the unchanged Projects hierarchy through the clear control or Escape. The explicit clear control returns focus to search.

## Provider And Model Fixtures

| Label | Family | Endpoint | Enabled | Model count |
|---|---|---|---|---|
| OpenRouter - Personal | OpenRouter | `https://openrouter.ai/api/v1` | yes | 5 |
| Hugging Face - Personal | Hugging Face | `https://router.huggingface.co/v1` | yes | 3 |
| OpenAI - Work | OpenAI | `https://api.openai.com/v1` | yes | 4 |
| LiteLLM Local | OpenAI Compatible | `http://127.0.0.1:4000/v1` | no | 4 |

Model rows include at least: `anthropic/claude-opus-4.1`, `anthropic/claude-sonnet-4.5`, `moonshotai/kimi-k2`, `openai/gpt-5`, `openai/gpt-5-mini`, `Qwen/Qwen3-Coder`, and `deepseek-ai/DeepSeek-R1`. Fixtures must span providers and enabled/disabled states so filtering and bulk behavior are reviewable.

Capability fixtures span Vision, Tools, Reasoning, Audio absence, and 128K/200K/400K context sizes. `GPT-5` supports Vision, Tools, and Reasoning with a 400K context; `Kimi K2` is Tools-only with 128K in the deterministic fixture. Model details include effective support, restrictions, evidence source, and checked time.

The composer model fixture begins on OpenAI when GPT-5 is active. Its inline back-chevron/OpenAI header opens enabled OpenRouter, OpenAI, and Hugging Face family choices without profile subtitles. Choosing a provider returns to only that provider's model rows without changing GPT-5 until a model is selected; closing and reopening restores the active model's provider. The fixture includes at least two composer-selectable models for each listed provider so the provider-to-model flow and capability filters remain reviewable.

## Plugin, Skill, And Runtime Fixtures

- Installed plugins: GitHub Workflow, Web Research.
- Directory plugins: Analytics Workspace, Calendar Assistant.
- At least one installed and one unavailable/disabled skill; row and dialog state must synchronize.
- Runtime choices: OpenCode and Pi, with one saved and the other usable as a dirty draft.
- MCP starts with an empty or small seeded list, but both STDIO and Streamable HTTP forms must be fully reviewable.

## Conversation Fixtures

Seed a transcript that demonstrates without replaying animation:

- ordinary user and assistant Markdown messages
- collapsed and expanded work disclosure
- Browser artifact with navigation history
- File artifact in read state and a long file adequate for scrolling/line-number QA
- File Explorer root plus at least `docs`, `src`, and `wireframes` child paths with distinct contents
- Terminal session `shell-1` with completed, running, and interruptible entries
- contextual Copy and Reply actions
- enough height to exercise scroll-to-latest
- Chat information with runtime, Local environment, workspace, model, Healthy status, and illustrative context-window utilization; adapter identity remains in per-run details
- capability-review draft with `concept-board.png` against Kimi K2, marked Needs Vision with explicit compatible-model, convert, remove, and cancel resolutions

Generated fixture actions must additionally demonstrate a new Chat stream, Browser update-in-place, File proposed edit/approve/reject, Terminal new-card reply, attachments, and attachment-only pending-chat promotion.

## State Matrix

Every relevant component is checked in default, hover, focus-visible, active/selected, disabled, loading/busy, success, warning, error, empty, and dark-mode states. Additionally:

- project: found, missing, expanded, collapsed, dragging, pending chat
- message: user, assistant, streaming, completed, with/without attachments, replied-to
- session controls: reasoning supported/unsupported, model-switch compatible/incompatible, attachment ready/converted/incompatible
- artifact: response, compact-pane, focused, selected placeholder
- file: view, editing, dirty, proposed, approved, rejected, saved
- browser: first history entry, middle, last, refreshing
- terminal: completed, running, stdin-ready, interrupted, command-ready
- dialog: closed, opening, valid, invalid, submitting
- Settings resource: enabled, disabled, dirty, installed, uninstalled, empty

## Full Reconstruction Acceptance

### Launch and navigation

- No-provider and configured-provider routes resolve correctly.
- All sixteen workflow destinations render directly.
- Settings visit/return preserves workspace state.
- Platform theme and live light/dark changes affect every route.

### Workspace

- Project/session CRUD-like prototype actions, path-dependent menus, search replacement/results/activation/clear/no-results, sorting, pending promotion, attachment-title fallback, and safe active fallback work.
- Panel resize works repeatedly; overlay dismissal and collapsed focus behavior are correct.
- Transcript hierarchy, Markdown source/render, shortcuts, paste, attachments/drop, actions, reply, and scroll-to-latest work.
- Composer modes show only their allowed controls and restore correctly after reply/focus.
- Composer model selection supports provider-list entry, provider-scoped return, capability filtering within the chosen provider, active-model preservation while browsing, model-dependent control recomputation after selection, and Escape/focus restoration.
- The branch fixture shows `main`, `codex/artifacts`, a divider, and `+ Create New` in that order without performing a real branch mutation.

### Artifacts

- Browser, File, Folder, and Terminal work inline and focused; compact-pane behavior never competes with focused controls.
- Artifact bodies scroll without covering header/footer.
- Focus swap moves one real transcript, supports direct Close, preserves left-panel state, and restores exact artifact state.
- Terminal Stop produces interrupted exit 130 and a reusable prompt without destroying the session.

### Settings

- All resource interactions, conditional forms, validation, capability filters/details, visible-result bulk action, dirty/save states, dialogs, transport panels, four approval presets, seven policy groups, and concrete exceptions are reviewable.
- Historical r012 parity additionally verifies its nine policy groups and 71 fixture identities without treating them as current product settings.

### Quality

- No console errors or warnings attributable to the artifact.
- No document-level horizontal overflow at wide and narrow review sizes.
- Keyboard-only completion is possible for every workflow.
- Screen-reader names, roles, states, live regions, dialog focus, and resizers are coherent.
- Reduced motion produces the same final content with no typing delay.
- Light and dark contrast and platform-specific chrome/control behavior pass visual review.
