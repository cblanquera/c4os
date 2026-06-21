# Interface Context

Status: active
Created: 2026-06-21
Updated: 2026-06-21
Source Note: Imported from human interface planning sources.

## Summary

C4OS uses a neutral, utilitarian, three-panel desktop shell. The current interface plan is the active source for this entry; visual pegs are references. Treat wireframe text as structural labels, not final product copy.

## Overall Layout

- Left panel: global navigation, project list, project-level chat sessions, settings entry.
- Center panel: empty-state prompt or active session thread.
- Right panel: workspace tools with Browser, Files, and Terminal tabs.
- Left and right panels are resizable; center content flexes between them.
- Top center bar includes left-panel collapse, current project/session title, and right-panel collapse.

## Left Panel

- Contains search input placeholder, Projects section with add-project `+`, project rows, nested chat session rows, and Settings link at bottom.
- Project rows show folder icon, project name, pencil icon for new chat, and trash icon for remove project.
- Project row can be selected on the main shell.
- In a session thread, the project row is not highlighted; the active chat session row is highlighted.
- Chat session rows do not use chevrons.
- Chat text aligns flush with project name text while active chat highlight extends left to the project-row edge.

## Prompt Composer

- Empty state asks `What should we build in c4os2?`
- Composer controls are attachment icon, approval policy chip `Approve for me`, model selector chip `gemini-flash-2.5`, microphone icon, send button, and context row showing only branch `main`.
- Removed from final wireframe: `c4os2` context placeholder, `Work locally` placeholder, and `5.5 Medium` model/reasoning label.

## Model Selector

- Clicking the model chip opens the model popover directly to the currently selected provider's model list.
- Model-list header shows active provider name such as `OpenRouter`.
- Header back/left control returns to provider list.
- Provider list shows rows such as `OpenRouter >` and `OpenAI >`.
- Clicking provider drills into that provider's model list.
- Clicking model updates composer model chip and closes the popover.
- Current model is visibly marked in the active provider's model list.
- Example OpenRouter options are `Gemini 2.5 Flash`, `ChatGPT o4`, and `Grok 2.0`.

## Session Thread

- Messages use messenger-style layout.
- User messages align right and leave left margin.
- Agent messages align left and leave right margin.
- Agent bubbles are not full width.
- Each agent bubble has icon-only collapse/expand in the header.
- First agent bubble demonstrates content-level Show more/Show less.
- Tool/action examples include completed tool-call block and agent run block with sensitive action waiting plus review approval action.

## Right Panel

- Tab order is Browser, Files, Terminal.
- Active tab controls the full right-panel body.
- Do not add extra gear/collapse tabs in the right tab strip.

### Browser

- Browser tab includes preview surface and web address bar.
- No Browser tab strip in MVP.
- Browser preview is a single surface for local app previews.

### Files

- Files tab has file explorer tree and open file/code view states.
- Explorer header title is active project name.
- Root folder is not repeated as first tree row; tree begins with folders/files under active project.
- File tree is dense and shifted left, like Codex explorer behavior.
- Clicking a file opens file/code view.
- Open file header becomes breadcrumbs such as `c4os2 > frontend > main.js`.
- Clicking a folder breadcrumb returns to explorer state.
- Code view fills the remaining right-panel tab body to the bottom.
- Avoid padding that makes code view feel like a floating card.

### Terminal

- Terminal tab includes main read-only terminal output area.
- Bottom panel is resizable, read-only in the wireframe, and represents pending AI command preview/results.
- Remove top terminal labels such as `>_ npm run build`.

## Settings

- Settings sidebar has `Back to app` at top and no search settings input.
- Navigation order is Providers, Models, Runtimes, Configuration, Plugins, Skills, MCP Servers.
- Providers screen shows OpenAI-compatible provider connections, unique labels, endpoint URL, saved API key status, enable/disable, and Add Provider form.
- Add Provider form fields are Provider type, Label, API Base URL, API key, Auth, and Headers. API key belongs directly under API Base URL.
- Models screen fetches models from enabled provider connections where available; each model row shows provider and enable/disable. Do not include a separate Manage button in MVP wireframe.
- Runtimes screen selects the workspace execution runtime and shows OpenCode and Pi as the r04 runtime options, with one selected at a time.
- Configuration shows approval policy, sandbox settings, and action to open `config.toml` externally. Do not include redundant config-source row.
- Plugins screen includes plugin catalog controls, plugin cards, plugin connect dialog, and marketplace dialog.
- Skills screen includes skill search, skill rows, per-project scope labels, enabled switches, and skill detail dialog.
- MCP Servers screen includes server rows, enabled switches, settings action, Add server action, and custom MCP dialog with STDIO and Streamable HTTP modes.

## Implementation Notes

- Keep MVP interface neutral and utilitarian.
- Use lucide icons.
- Do not introduce dark theme or brand styling unless design phase explicitly approves it.
- Preserve left/right panel resize behavior.
- Preserve project selection versus active chat selection distinction.
- Preserve right-panel tab order and full-body tab behavior.
- Preserve messenger-style ownership for user versus agent messages.
- Preserve Runtimes, Plugins, Skills, and MCP Servers as built-out settings surfaces, not placeholders.
