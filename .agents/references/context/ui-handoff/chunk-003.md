# UI Handoff - Chunk 003

Parent: `index.md`
Focus: Sections 21-23: plugins, skills, MCP servers, and their dialogs.

### 21. Settings Plugins

Route: `#settings-plugins`

The Plugins screen presents a plugin catalog and plugin connection flow.

Header:

- Title: Plugins.
- Summary: `Manage installed plugins and extension surfaces.`

Catalog controls:

- Search plugins input.
- Filter button labeled `Built by C4OS`.
- Filter menu with `+ Add Marketplace`.

Illustrative plugin cards:

- GitHub
- Slack
- Notion
- Linear
- Gmail
- Google Calendar
- Google Drive
- Figma
- Vercel

Plugin names and descriptions are placeholders for catalog shape. Each card
shows a logo mark, plugin name, description, and Add button.

#### 21.1. Plugin Connect Dialog

Clicking Add opens a plugin connection dialog. The dialog content changes to
the selected plugin.

Dialog surfaces:

- Source app and target plugin visual connection.
- Title such as `Connect GitHub`.
- Status: `Approved by your admin`.
- Copy blocks for control, risk, and data shared with plugin.
- Continue action.
- Advanced settings action.

The wireframe does not authenticate, install, or persist plugin connection.

#### 21.2. Marketplace Dialog

The marketplace dialog is opened from `+ Add Marketplace`.

Fields:

- Source.
- Git ref.
- Sparse paths.

Actions:

- Cancel.
- Add marketplace.

This is a simulated review form.

### 22. Settings Skills

Route: `#settings-skills`

The Skills screen manages installed skills and per-project availability.

Visible controls:

- Search skills input.
- Skill list rows.
- Scope labels.
- Enabled switches.

Illustrative skill rows:

- Ab Testing
- Ad Creative
- Ads
- Ai Seo
- Analytics
- Aso
- ChrisAI Agents

Clicking a skill opens a skill detail dialog.

Skill detail dialog surfaces:

- Skill icon.
- Skill name.
- Type label `Skill`.
- Skill summary.
- Enabled switch.
- More actions button.
- Detail body split from the selected skill description.
- Uninstall action.
- Try in chat action.

Skill names, descriptions, scopes, and details are placeholders. The wireframe
does not execute skills, uninstall skills, or persist skill availability.

### 23. Settings MCP Servers

Route: `#settings-mcp`

The MCP Servers screen manages external tool/data source connections.

Header:

- Title: MCP Servers.
- Summary: `Connect external tools and data sources.`
- Learn more button.
- Add server action.

Illustrative server rows:

- node_repl
- openaiDeveloperDocs
- stackpress_blog_mcp

Each row has:

- Server name.
- Settings button.
- Enabled switch.

#### 23.1. Custom MCP Dialog

Clicking Add server opens `Connect to a custom MCP`.

Top controls:

- Close button.
- Docs link.
- Transport segmented control.

Transport modes:

- STDIO
- Streamable HTTP

STDIO fields:

- Command to launch.
- Arguments.
- Environment variables.
- Environment variable passthrough.
- Working directory.

Streamable HTTP fields:

- URL.
- Bearer token env var.
- Headers.
- Headers from environment variables.

Footer actions:

- Cancel.
- Save.

Server names are placeholders. The wireframe does not connect to or validate
MCP servers.
