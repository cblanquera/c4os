# Product Graphical User Interface

The following describes the look of the application.

## Overall Layout

The app uses a three-panel desktop shell:

- Left panel: global navigation, project list, project-level chat sessions, settings entry.
- Center panel: empty-state prompt or active session thread.
- Right panel: workspace tools with tabs for Browser, Files, and Terminal.

The left and right panels are resizable:

- Left panel resize handle sits along the right padding edge of the left sidebar.
- Right panel resize handle sits along the left padding edge of the right tool panel.
- Center content should flex between them.

Top center bar includes:

- Left-panel collapse affordance at the top left.
- Current project or session title.
- Right-panel collapse affordance at the top right.

## Left Panel

The left panel contains:

- Search input placeholder.
- Plugins placeholder.
- Projects section with an add-project `+` button.
- Project rows with folder icon, project name, pencil icon for new chat, and trash icon for remove project.
- Chat session rows nested under the active project.
- Settings link at the bottom.

Project/session rules:

- Project row can be selected on the main shell.
- On a session thread, the project row is not highlighted; the active chat session row is highlighted.
- Chat session examples do not use chevron icons.
- Chat session text aligns flush with the project name text, while the active chat highlight extends left to the project-row edge.

## Prompt Composer

The empty state asks:

`What should we build in c4os2?`

Composer controls:

- Attachment icon.
- Approval policy chip: `Approve for me`.
- Model selector chip: `gemini-flash-2.5`.
- Microphone icon.
- Send button.
- Context row only shows branch `main`.

Removed from the final wireframe:

- `c4os2` context placeholder in the composer.
- `Work locally` placeholder in the composer.
- `5.5 Medium` model/reasoning label.

Model selector behavior:

- Clicking `gemini-flash-2.5` opens a model popover.
- The popover opens directly to the currently selected provider's model list.
- The model-list header shows the active provider name, for example `OpenRouter`.
- The model-list header has a back/left control that returns to the provider list.
- The provider-list state shows provider rows such as `OpenRouter >` and `OpenAI >`.
- Clicking a provider drills into that provider's model list.
- Clicking a model updates the composer model chip and closes the popover.
- The current model is visibly marked in the active provider's model list.
- Example OpenRouter model options: `Gemini 2.5 Flash`, `ChatGPT o4`, `Grok 2.0`.

## Session Thread

Messages use a messenger-style layout:

- User messages align to the right and leave left margin.
- Agent messages align to the left and leave right margin.
- Agent bubbles are not full width.

Agent bubble controls:

- Each agent bubble has an icon-only collapse/expand toggle in the header.
- The first agent bubble also demonstrates a content-level `Show more` / `Show less` toggle.
- `Show more` reveals extra content inside the bubble and changes label to `Show less`.
- `Show less` hides that extra content and changes label back to `Show more`.

Tool/action examples:

- Tool-call block with completion status.
- Agent run block with sensitive action waiting and review approval action.

## Right Panel

Tab order is fixed:

1. Browser
2. Files
3. Terminal

The active tab controls the full right-panel body. Do not add extra gear/collapse tabs in the right tab strip.

### Browser Tab

Browser tab includes:

- A browser preview surface.
- A web address bar.
- No browser tab strip in MVP.

Browser preview is a single surface for local app previews.

### Files Tab

Files tab has two states:

1. File explorer tree.
2. Open file/code view.

Explorer state:

- Header title is the active project name, such as `c4os2`.
- The root project folder is not repeated as the first tree row.
- Tree begins directly with folders/files under the active project.
- File tree is shifted left and dense, similar to Codex explorer behavior.
- Clicking a file opens the file/code view.

Open file state:

- Header becomes breadcrumbs, for example `c4os2 > frontend > main.js`.
- Clicking a folder breadcrumb returns to the file explorer state.
- Code view fills the remaining right-panel tab body, including down to the bottom.
- Avoid padding that makes the code view feel like a floating card.

### Terminal Tab

Terminal tab includes:

- Main read-only terminal output area.
- Resizable bottom panel for command/results the AI will make.

Terminal rules:

- Remove the top label/header such as `>_ npm run build`.
- Bottom panel is read-only in the wireframe and represents pending AI command preview/results.
- Bottom panel is vertically resizable.

## Settings

Settings sidebar includes:

- `Back to app` at the top.
- No search settings input.
- Navigation order:
  - Providers
  - Models
  - Runtimes
  - Configuration
  - Plugins
  - Skills
  - MCP Servers

### Providers

Providers screen:

- Shows OpenAI-compatible provider connections.
- Each provider has a unique label.
- Existing providers show endpoint URL and saved API key status.
- Providers can be enabled/disabled.
- Add Provider button opens the provider form.

Add Provider form fields:

- Provider type
- Label
- API Base URL
- API key
- Auth
- Headers

The API key field belongs directly under the API Base URL field.

### Models

Models screen:

- Models are fetched from enabled provider connections when available.
- Each model row shows which provider it came from.
- Include model enable/disable affordance.
- Do not include a separate Manage button in MVP wireframe.

### Configuration

Configuration screen:

- Shows approval policy.
- Shows sandbox settings.
- Has an action to open `config.toml` externally.
- Do not include a redundant config-source row.

## Implementation Notes

Use these rules when translating the wireframe to production UI:

- Keep the MVP interface neutral and utilitarian.
- Use lucide icons.
- Do not introduce dark theme or brand styling during implementation unless the design phase explicitly approves it.
- Treat wireframe text as structural labels, not final copy.
- Preserve the left/right panel resize behavior.
- Preserve the distinction between project selection and active chat selection.
- Preserve the right panel tab order and full-body tab behavior.
- Preserve messenger-style ownership for user versus agent messages.
