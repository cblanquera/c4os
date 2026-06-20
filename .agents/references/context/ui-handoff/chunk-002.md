# UI Handoff - Chunk 002

Parent: `index.md`
Focus: Sections 10-19: provider and model selection, chat session, Browser, Files, Terminal, and settings through configuration.

### 10. Provider And Model Selection

Routes:

- `#providers-popover`
- `#models-popover`

Both routes render the New Session shell with a composer popover anchored near
the model chip.

#### 10.1. Provider Popover

The provider popover title is `Providers`. Its subtitle is `Select source`.

Illustrative provider choices:

- OpenRouter
- OpenAI
- LiteLLM Local

Each provider row drills into the model list.

#### 10.2. Model Popover

The model popover shows a back bar with the selected provider, currently
`OpenRouter`. The back control returns to provider selection.

Illustrative visible models:

- Gemini 2.5 Flash
- ChatGPT o4
- Grok 2.0

The current model is visibly marked with a check icon. Provider and model names
are placeholders unless supplied by real configuration/provider data. Selecting
a model closes back to the new-session shell in the prototype. Real persistence
and provider fetching are not implemented in the wireframe.

Purpose and expected behavior:

- The model chip chooses which model will answer the next prompt or session.
- Models are grouped by provider so users understand where the model comes
  from.
- Opening the model selector can start at provider selection or at the active
  provider's model list. r04 demonstrates both states as separate routes.
- The provider list is a drill-in menu. Selecting a provider opens that
  provider's model list.
- The model list has a back control so the user can return to provider
  selection without closing the whole selector.
- The active/current model is visibly marked.
- Selecting a different model updates the composer model chip and closes the
  selector.
- If the chat is already active and model context is locked, the model chip is
  read-only and should not open the selector.
- Production implementation should source provider/model rows from configured
  provider connections, not from r04 placeholder values.

### 11. Chat Session Screen

Route: `#chat-session`

Chat Session represents an active session inside the trusted project.

The r04 active session title `Locate Tauri integration` is illustrative.

The visible thread uses illustrative content to demonstrate four item types:

1. User message.
2. Agent response.
3. Tool Call activity card.
4. Agent Run activity card waiting for sensitive action approval.

#### 11.1. Message Ownership

User messages align right and leave space on the left. Agent messages align
left and leave space on the right. This ownership model must be preserved.

Agent bubbles are not full width. The layout should make it clear who owns each
message without relying on color alone.

#### 11.2. Agent Message Controls

Agent responses can include:

- Icon-only collapse/expand affordance.
- Show More / Show Less content disclosure.

The first agent message includes hidden extra detail. Show More expands the
extra content inside the same message and changes to Show Less. Show Less
collapses it.

#### 11.3. Activity Cards

Activity cards represent agent work and approval boundaries. The wireframe
shows:

- Tool Call with status `Completed`.
- Agent Run with label `Sensitive action waiting`.
- Review Approval action.

Implementation should treat these as separate structured event surfaces, not
plain message text.

#### 11.4. Follow-Up Composer

The chat composer is docked under the thread and remains reachable. Its context
chips are read-only because the chat has locked branch and model context.

### 12. Browser Tool

Default right-panel state for New Session and Chat Session.

The browser tool contains:

- Address bar with a local preview URL.
- Preview surface.
- Helper text indicating that browser tabs are out of scope.

The r04 URL, title, and helper text are placeholders. MVP browser behavior is
one preview surface. Do not add browser tabs unless a future product decision
expands the scope.

### 13. Files Tool

Routes:

- `#file-explorer`
- `#file-editor`

Both render the shell with the right panel in Files state.

#### 13.1. File Explorer

The file explorer heading is the active project name. r04 uses `c4os2` as the
placeholder project name.

Illustrative rows:

- `backend` folder
- `frontend` folder
- `main.js` file
- `index.html` file
- `tests` folder

The root project folder is not repeated as the first row. The tree starts with
items inside the active project.

#### 13.2. File Editor

The editor state replaces the file tree with breadcrumbs and a code pane.

Illustrative breadcrumbs:

- c4os2
- frontend
- main.js

Illustrative code:

```js
import { startWorkspace } from "./runtime";

const project = "c4os2";
const trustedRoot = true;

startWorkspace({ project, trustedRoot });

// Code view fills the full tool body.
```

The code snippet is mock content. The code pane fills the right-panel body. It
should not look like a decorative card. Line numbers are visible. Folder
breadcrumbs return to explorer state in the prototype.

### 14. Terminal Tool

Route: `#terminal`

The Terminal state contains:

- Read-only terminal output area.
- Horizontal resize handle.
- Bottom AI command preview/results panel.

Illustrative terminal output:

```text
$ npm run dev
ready in 614ms
local preview available at 127.0.0.1:3000
```

The terminal output and helper copy are placeholders. The bottom panel title
expresses the intended function: AI command preview/results. The bottom panel
is read-only in the wireframe and the resize handle sits on the top boundary.

The wireframe does not implement terminal input, command execution, or process
management. It only establishes the layout for terminal output and AI command
preview/results.

Purpose of the bottom panel:

- The top terminal area represents terminal output or a live terminal stream.
- The bottom panel represents agent-owned command preview/results, separate
  from the terminal stream.
- The bottom panel is where the app can show what the agent proposes to run,
  why it wants to run it, approval state, pending command details, execution
  results, or command-related explanation.
- Keeping this panel separate prevents agent command review from being confused
  with raw terminal output.
- The bottom panel is read-only in r04. Production may add review actions, but
  command editing or execution controls must be designed deliberately.
- The horizontal resize handle lets the user allocate more or less space to
  command preview/results without leaving the Terminal tab.

### 15. Settings Shell

Settings screens use a two-column settings layout:

- Left settings navigation rail.
- Main settings content.

The left rail contains:

- Back to app.
- Kicker: Settings.
- Providers.
- Models.
- Configuration.
- Divider.
- Plugins.
- Skills.
- MCP Servers.

There is no settings search input. Back to app returns to the project shell.

### 16. Settings Providers

Route: `#settings-providers`

The Providers screen manages OpenAI-compatible provider connections.

Header:

- Title: Providers
- Summary: `Manage OpenAI-compatible provider connections. Labels must be unique.`
- Add Provider primary action.

Illustrative provider rows:

- OpenRouter - Personal
- OpenAI - Work
- LiteLLM Local

Each provider row shows:

- Provider label.
- Endpoint URL.
- API key saved status.
- Edit button.
- Enabled switch.

Provider labels, URLs, and saved-key statuses are placeholders. The wireframe
does not persist provider toggles or edits.

### 17. Add Provider

Route: `#settings-add-provider`

The Add Provider form saves an OpenAI-compatible connection profile.

Fields:

- Provider Type select.
- Label input.
- API Base URL input.
- API Key password input.
- Auth select.
- Headers textarea.

Important ordering rule: API Key belongs directly under API Base URL.

Actions:

- Save Provider.
- Cancel.

Both actions return to Providers in the prototype. Real validation, secure key
storage, and provider connection testing are outside the wireframe.

### 18. Settings Models

Route: `#settings-models`

The Models screen presents models fetched from enabled provider connections.

Illustrative model rows:

- Gemini 2.5 Flash from OpenRouter - Personal, marked Current.
- ChatGPT o4 from OpenRouter - Personal, marked Enabled.
- Grok 2.0 from OpenRouter - Personal, marked Enabled.
- gpt-4.1 from OpenAI - Work, marked Enabled.
- local/coder-small from LiteLLM Local, marked Enabled.

Model names and provider labels are placeholders. Each row has an enable
switch. The wireframe does not include a separate Manage button for models.

### 19. Settings Configuration

Route: `#settings-configuration`

The Configuration screen exposes runtime control areas.

Header summary:

`Configure approval policy and sandbox settings.`

Header action:

- Open config.toml externally.

Cards:

- Approval Policy: choose when the app asks before high-impact actions. Action
  label: `On request`.
- Sandbox Settings: choose how much agents can do when running commands.
  Action label: `Workspace write`.

The wireframe intentionally avoids a redundant config-source row.
