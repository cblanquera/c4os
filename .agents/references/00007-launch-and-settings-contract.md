# Launch And Settings Contract

## Top-Level Routing

Resolve exactly one of launch, workspace, or Settings at a time. A first launch without a configured provider enters onboarding. A configured provider enters workspace start. Development/QA direct routes may force either state. Workspace and Settings transitions preserve the application document and in-memory state.

Production Settings entry belongs in the native application menu. A visible workspace Settings control is acceptable only in development/review builds.

## First-Provider Onboarding

Use a centered, standalone provider form with compact C4OS branding and no Settings navigation. Heading: `Connect your AI provider`; support: `Add a provider to choose the models C4OS can use.`

Fields:

- Provider type: OpenRouter, Hugging Face, OpenAI, OpenAI Compatible.
- Profile label, required and unique.
- API key with reveal control when auth requires it.
- OpenAI Compatible additionally exposes API base URL; Authentication (Bearer token, API key header, None); conditional API key header name; JSON additional headers.
- None removes the API-key requirement.

Actions: Test Connection and Continue. Validate only visible required fields, URL shape, unique label, and JSON headers. Expose testing, failure, success, discovered-model availability, and submitted status in a live region. Continue remains unavailable until the latest connection test succeeds and at least one usable model is available; changing a tested endpoint or authentication field invalidates that success.

The successful-test state remains inside the compact provider form and does not add a model picker or default-confirmation panel. C4OS automatically selects the production-ready model with the most normalized features in the explicit `supported` state; ties use discovery recommendation rank and then stable model identity. Continue stores the provider through the credential architecture, persists that model with OpenCode and Local as the initial defaults, and enters Workspace Start. If a retryable pre-commit Continue attempt fails, the successful transient test remains current, the failure stays visible, and Continue remains available without repopulating or requesting the raw key. Model changes remain available through the normal post-onboarding model controls before the first valid Chat submission binds the route.

## Workspace Start

Heading: `What would you like to open?`; support: `Start something new or continue where you left off.`

Show exactly three primary actions:

- Open a folder — use a local folder as the project.
- Open a workspace — open a saved C4OS workspace.
- Clone Repository — clone from a Git repository URL.

Then show exactly three recent-workspace rows in the deterministic reconstruction fixture. Selecting any action/row shows brief opening progress and enters Chat without replacing the application document. At narrow width, cards stack, icon aligns with title, title/description stay grouped by a 5px gap, and vertical padding is balanced with no forced excess height.

A saved Workspace is a portable C4OS zip archive containing its ordered Project references, Workspace-level configuration, per-Project configuration, and per-Chat configuration/cache/archive records. Loading it reconstructs the main-screen state from an unpacked working copy under `~/.c4os`. Project folders remain external trusted roots, so the same folder can have different overlays and Chats in different Workspaces.

## Settings Shell

Desktop layout is a 224px persistent navigation column and independently scrolling content capped near 920px. Navigation order:

1. Providers
2. Models
3. Runtimes
4. Configuration
5. divider
6. Plugins
7. Skills
8. MCP Servers

`Back to C4OS` returns to Chat and restores the existing workspace. Advanced Policies keeps Configuration selected. Below 680px, compress navigation to icons with accessible names.

Shared patterns: concise page header, scan-oriented rows/cards, search fields, switches with `role=switch`, aligned actions, shared dialogs, transient notifier, explicit/backdrop/Escape dialog close, and focus restoration.

## Providers

- List provider profiles with provider family, Edit, and availability switch.
- Add and Edit share one dialog and field rules with onboarding.
- Presets: OpenRouter `https://openrouter.ai/api/v1`; Hugging Face `https://router.huggingface.co/v1`; OpenAI `https://api.openai.com/v1`; compatible is user-defined.
- LiteLLM is represented as OpenAI Compatible.
- Blank API key during Edit preserves an existing secret.
- Toggling a provider updates provider-derived model availability.
- Do not restore removed “Saved profiles”, endpoint copy, credential-state copy, badges, or the `AI Runtime` eyebrow.

## Models

- Search by model identity and filter by provider and effective capability: Vision, Tools, Reasoning, or Audio.
- Rows show a clickable provider/model identity, compact effective-capability chips, context size, and availability switch. Hover/focus underlines the model name; activation opens route-specific capability details and evidence.
- Models default enabled when discovered unless policy says otherwise.
- Bulk action affects only visible results: `Disable`, switching to `Enable` when every visible row is disabled.
- At 992px and wider, search, both filters, and the bulk action share one row. Below 992px, search spans the first row and both filters plus the bulk action share the second row without document overflow.
- Refresh exposes progress and completion feedback.

## Runtimes

Show OpenCode and Pi as mutually exclusive choices. A draft selection does not take effect until Save Runtime. Save is disabled when draft equals saved value.

## Plugins

- Tabs: Installed and Directory, left aligned with content.
- Installed header count updates after install/uninstall.
- Directory toolbar aligns search with marketplace chooser.
- Seed directory/installed cards have identity, summary, Details, and state-appropriate action.
- Add Marketplace dialog fields: required Source, optional Git ref, optional sparse paths.
- Detail dialog: identity mark, title, summary/capabilities, Website, Terms, Privacy Policy, Install or Uninstall according to state, and Done.
- Install/Uninstall updates the card, directory action, detail action, and count without reload.
- Do not restore removed page eyebrows.

## Skills

- Search installed skills.
- Each row uses a document-like identity, name/summary, availability switch, and details access.
- Row and dialog switches share one state.
- Detail supports Uninstall and Try in chat. Uninstall removes the row; Try in chat returns a clear simulated/production transition.
- Close/Done action aligns consistently with other dialogs.

## MCP Servers

Start with server list and empty state. Add and Configure share a dialog with transport selection:

- STDIO: command, repeatable arguments, repeatable environment variables, repeatable environment passthrough, working directory.
- Streamable HTTP: URL, bearer-token environment variable, repeatable literal headers, repeatable environment-backed headers.

Repeaters add/remove rows. Validate transport-specific required fields. Save updates list state; production connection/execution follows the runtime/security contract.

## Configuration

Forward product Runtime card order:

1. Default Approval Policy: Ask for approval, Approve safe actions, Approve for me, Custom.
2. `Advanced` link beneath the policy select.
3. Restore last workspace switch.

Explain near `Approve for me` that it remains bounded by the active sandbox, trusted roots, maximum authority, and managed policy. `Custom` is selected when category rules or concrete exceptions differ from a preset.

Environment card:

- Shell environment switch.
- Browser Environment: All browsers, Per project, Per chat session, None.
- Open `config.toml` externally action.

Browser Environment sharing covers every applicable browser storage category—cookies, `sessionStorage`, `localStorage`, and IndexedDB—while preserving normal web-origin and storage semantics. All browsers uses one persistent app-wide profile; Per project and Per chat session use persistent Workspace+Project- and Chat-keyed profiles; None is per-artifact ephemeral and destroyed on close. `sessionStorage` follows page/tab lifetime. Inactivation retains profiles, while explicit Clear Browser Data targets app-wide, Project, Chat, or ephemeral scope. C4OS Home owns the protected profile registry and lifecycle. On macOS, raw website data stays in WebKit's platform-managed application container; it never enters portable Workspace archives, normal exports, diagnostics, or model context.

The action opens C4OS Home at `~/.c4os/config.toml`, not a Codex configuration file. C4OS Home also owns the unpacked last-loaded Workspace plus app-level MCP, skill, plugin, and marketplace configuration. Workspace-, Project-, and Chat-scoped configuration remains in the Workspace archive. None of these files stores raw credentials.

Do not restore removed Default model or the `Advanced` eyebrow.

## Advanced Policies

The forward product uses seven understandable policy groups plus concrete remembered exceptions. The detailed r012 tool identities remain an internal classifier/proof corpus, not permanent settings rows. Preserve search, dirty/revert/save behavior, clear action/target/scope descriptions, and the ability to inspect the effective policy result.

The seven groups are Workspace files, Commands and processes, Version control, Network and sharing, Browser and desktop, Credentials, and Extensions and C4OS. Category rules expose Use default, Allow, Ask, and Deny. Keep concrete Exceptions in a separate left-aligned tab view; revoking an exception does not change category defaults. The current-preset guardrail aligns with and spans the policy content width.

### Literal r012 reconstruction profile

For exact r012 parity only, render the historical header/support, centered/aligned Save Policies, search, 190px group rail, result header/list, and these nine groups:

1. Terminal
2. Git
3. Filesystem
4. Browser
5. Network
6. Credentials
7. Processes and apps
8. Desktop facilities
9. C4OS authority

The r012 fixture contains 71 supplied tool identities. Each row shows identity, description, and select with Use default, Always allow, Always ask for permission, Never allow. Group selection clears search and restores group context. Search spans all groups. Any draft difference marks dirty and enables Save; returning all values to saved state disables it; Save promotes the draft and notifies. Do not use this historical schema for new product work.

## Settings Acceptance

Verify direct entry and Back behavior, state preservation, all navigation destinations, narrow navigation, provider conditional fields and validation, automatic onboarding model selection without a picker, model search/filter/bulk, runtime dirty state, plugin marketplace/install/uninstall/details, skills shared switch/uninstall, both MCP transports and repeaters, current Configuration presets/guardrail copy, current policy groups/exceptions, search, dirty/revert/save, dialog dismissal, and theme changes while any dialog is open. When the task is literal r012 reconstruction, additionally verify all nine historical groups and 71 fixture identities.
