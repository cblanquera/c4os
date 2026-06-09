# Open Questions And Research Gaps

## Runtime Integration

 - What is the most stable way to embed or control OpenCode Runtime from a Tauri app?
 - Can OpenCode expose all needed events without scraping terminal output?
 - Can the app intercept tool calls before execution for custom approval UI?
 - How does OpenCode persist sessions, and should the GUI mirror or own session metadata?

## OpenCode And Workspaces

 - What exact on-disk OpenCode config paths should be shared versus isolated per project?
 - How should global OpenCode settings interact with app-managed settings?
 - Can multiple runtime instances safely run in parallel against separate worktrees?

## Plugin Compatibility

 - Which Codex plugin manifest fields are stable public contract versus current implementation detail?
 - Should the app support Codex deep links for plugin viewing/sharing?
 - How should plugin hooks map to OpenCode hooks if the semantics differ?

## Skills

 - Should skills be auto-invoked by the model, explicitly selected by users, or both?
 - How should conflicting skill versions be resolved across global, project, and plugin scopes?
 - Should app-specific skill metadata live in `agents/openai.yaml`, a namespaced file, or both?

## MCP

 - Which MCP spec version should be the minimum supported baseline?
 - Should remote MCP servers be allowed in MVP, or only local stdio servers?
 - How should MCP sampling requests be represented in the approval UI?
 - Should MCP resources become artifacts automatically?

## Browser And Artifacts

 - Should the in-app browser use Tauri WebView only or a controlled external browser for richer automation?
 - Which document/spreadsheet/slide preview renderers are acceptable for MVP?
 - How should generated artifacts be deduplicated and garbage-collected?

## Security

 - What default secret-file denylist should ship?
 - Should shell commands run under the user's account or a separate local sandbox account?
 - What command patterns should be considered destructive by default?
 - What telemetry, if any, is acceptable in a local-first product?

## Provider And BYOK

 - Should BYOK keys be stored directly in OpenRouter workspace settings, locally, or both?
 - Should the app support direct provider calls if OpenRouter is unavailable?
 - How should provider fallback and routing be surfaced to users?

## Tauri Versus Electron

 - Does OpenCode Runtime or browser automation require Node/Chromium capabilities that would materially favor Electron?
 - Are system WebView differences acceptable for the artifact and browser viewer requirements?
 - What platform support is required at launch: macOS only, macOS/Windows, or macOS/Windows/Linux?

## Product Scope

 - Is the initial audience developers, general knowledge workers, or mixed teams?
 - Are cloud sync and team policy administration required soon after MVP?
 - Should operations/research/writing workflows have built-in agents at launch or be delivered as plugins?
