# MVP Feature List

Status: active
Created: 2026-06-21
Updated: 2026-06-21
Source Note: Derived from the current MVP contract and accepted r04 handoff.

## Summary

This is the implementation-facing MVP feature list for C4OS. The MVP is the
full documented/r04 desktop product slice unless a later accepted decision
explicitly moves something out. Checkpoints are sequencing gates only; they do
not reduce MVP scope.

Use this file before MVP implementation so the work starts from a complete
feature inventory instead of a partial task list.

## Customer Workflow

The MVP must let a technical local-project user complete this workflow:

1. Open the app and select, clone, create, open, or resume a local project.
2. Trust a project folder before local file, Git, shell, instruction, skill,
   extension, or approval-scoped workflows become available.
3. Configure an OpenAI-compatible provider and model with raw keys stored only
   in secure storage.
4. Start or resume a persistent session backed by the C4OS-owned runtime
   adapter.
5. Submit a request with visible attachment, approval policy, branch,
   provider, and model context.
6. Watch structured user messages, agent messages, tool activity, run activity,
   approval waits, and final responses.
7. Inspect and edit trusted-root files, preview artifacts, browse local/public
   pages, and use user or agent terminal surfaces under app-owned policy.
8. Install or connect plugins, skills, and MCP servers, then explicitly enable
   any extension before it affects runtime behavior.
9. Quit and restart with transcript, runtime references, approvals, artifacts,
   Browser/file/terminal/extension action records, and run state available for
   continuation.

## Feature Surfaces

### App Start And Workspace Trust

- App starts on App Start, not a review menu or marketing screen.
- App Start provides Open Folder, Clone Repository, Open Workspace File, and
  recent folder-backed workspace entry points.
- Prompt submission and local agent actions stay unavailable until a project
  folder is trusted.
- Workspace descriptors may store non-secret project references and
  preferences only; they must not store raw provider keys, full transcripts,
  artifact archives, or private operational state by default.

### Desktop Shell

- Shell has three regions: left project/session navigation, center workbench,
  and right workspace tools.
- Left and right panels are resizable and collapsible; center workbench absorbs
  remaining width.
- Left navigation includes project search, add-project action, project rows,
  nested chat session rows, and Settings.
- Project selection and active chat selection are distinct states.
- Center workbench supports empty New Session, Chat Session, and provider/model
  popover states.
- Right workspace tool tab order is Browser, Files, Terminal. The active tab
  owns the full right-panel body.

### Composer And Session Thread

- Composer exposes attachment, approval policy, branch, provider, and model
  context before submission.
- Attachments appear as removable chips and must not imply generic upload or
  deletion of the underlying file.
- Branch and model context are visible before submission; in an active chat
  they become read-only unless a later product decision changes that behavior.
- User messages align right; agent messages align left.
- Agent messages can have collapse/expand and Show More / Show Less behavior.
- Tool calls, run activity, and approval waits are structured event surfaces,
  not plain message text.

### Provider, Model, And Runtime

- Users can configure OpenAI-compatible providers with secure BYOK storage.
- Built-in provider profile types include OpenAI, OpenRouter, Hugging Face
  router, LiteLLM proxy, and custom OpenAI-compatible endpoint.
- Provider rows show label, endpoint, saved key status, edit action, and
  enable/disable state.
- Add Provider collects provider type, label, API base URL, API key, auth, and
  headers, with API key directly under API base URL.
- Models are fetched from enabled provider connections where available and can
  be enabled or disabled.
- Model selector groups models by provider, shows the current model, supports
  provider drill-in, and updates the composer chip on selection.
- Runtime settings include a distinct Runtimes surface. OpenCode and Pi are
  visible runtime options, with exactly one selected at a time.
- OpenCode is the first runtime backend behind a thin C4OS-owned adapter; Pi is
  visible as a later adapter target, not the first implementation target.
- C4OS owns workspace, session, run, approval, artifact, provider, extension,
  UI, persistence, and audit records even when a runtime executes work.

### Sessions, Runs, Resume, And Concurrency

- Sessions persist transcript, message index, run status, selected agent,
  provider/model, approvals, artifacts, memory references, runtime references,
  branch/worktree context, Browser state, terminal state, extension state, and
  action records needed for restart/resume.
- Multiple trusted project folders and sessions can exist.
- Concurrent runs are supported across trusted folders or sessions, with one
  main active run per chat session.
- Concurrent runs isolate cwd, approvals, outputs, artifacts, runtime events,
  cancellation, and action records.

### Approvals, Audit, And Policy

- Sensitive actions are evaluated by app-owned policy before execution.
- Sensitive categories include shell commands, file writes/deletes, MCP
  mutation, credential use, networked tools, worktree mutation, share/export,
  Browser agent actions, extension runtime impact, and destructive actions.
- Approval policy supports allow, ask, and deny behavior, plus visible pending
  approval states and audit records.
- Saved or remembered approval rules must stay scoped and auditable.
- "Approve for me" in the composer remains subject to stricter global,
  workspace, sandbox, or app policy.

### Files

- Files surface includes trusted-root file explorer and open file/code view.
- File tree starts with items inside the active project; it does not repeat the
  root folder as the first row.
- Hidden files and folders are visible except `.git`.
- File operations include browsing, loading, editing, saving, reverting,
  creating, renaming, guarded deleting, external change handling, and active
  file search.
- File operations reject traversal outside trusted roots and casual `.git`
  mutation.
- Editor view uses breadcrumbs and fills the right-panel body; it must not look
  like a floating decorative card.

### Artifacts And Preview

- Artifacts are product records associated with project, session, originating
  run, creation time, safe viewer type, and reveal/export options.
- Supported artifact types include Markdown, text, code, diffs, generated HTML,
  images, PDFs, downloadable files, command logs, research, and analysis
  outputs.
- Generated or untrusted HTML must render without provider credentials,
  arbitrary workspace file access, shell state, or privileged app APIs.
- External web browsing and local artifact preview remain visually and
  technically separate.

### Browser

- Browser is a user-owned desktop surface with project-scoped profile state.
- Browser supports one preview surface only; Browser tabs and downloads are out
  of MVP scope.
- Browser supports local file browsing, public web browsing, request-scoped
  agent browsing, and logged-in use when clearly requested.
- Agent Browser actions are recorded.
- Browser content must not expose provider credentials, arbitrary workspace
  files, shell state, or privileged app APIs.

### Terminal

- Terminal includes a user terminal and a backend-owned agent command terminal.
- Terminal process lifecycle, cwd validation, sanitized environment, transport,
  output streaming, cancellation, failure reporting, cleanup, and audit records
  are backend-owned.
- Agent command execution uses a deterministic read-only allowlist with exact
  pattern matching.
- Non-allowlisted or outside-project commands require approval before
  execution.
- r04 terminal UI has top terminal output plus a separate resizable bottom AI
  command preview/results panel.
- The bottom panel is read-only in the wireframe and is for proposed commands,
  reasoning, approval state, pending details, results, or command explanation.

### Settings, Extensions, And Invocation

- Settings navigation order is Providers, Models, Runtimes, Configuration,
  Plugins, Skills, MCP Servers.
- Settings has Back to app and no settings search input.
- Configuration exposes approval policy, sandbox settings, and an action to
  open config externally.
- Plugins surface includes catalog controls, plugin cards, Add action, plugin
  connect dialog, and marketplace dialog.
- Skills surface includes search, skill rows, per-project scope labels,
  enabled switches, and skill detail dialog.
- MCP Servers surface includes server rows, enabled switches, settings action,
  Add server action, and custom MCP dialog with STDIO and Streamable HTTP
  modes.
- Plugin, skill, and MCP install/connect records include provenance, scopes,
  workspace/project scope, shared data, runtime/tool access, enabled state, and
  audit log.
- Extensions have no runtime effect until explicitly enabled.
- Prompt tags are `$skill` for skills, `@plugin` for plugins, and `^mcp` for
  MCP servers. MCP invocation is explicit only.

### Visual, Accessibility, And Placeholder Rules

- MVP UI is neutral, utilitarian, dense enough for repeated technical work, and
  not a marketing-style hero or dashboard.
- Use familiar icon buttons and accessible labels for icon-only controls.
- Focus-visible states, dialog focus management, Escape close, textbox
  semantics, toggle state semantics, and non-overlapping legible text are
  expected.
- r04 monochrome styling, placeholder labels, sample records, sample copy, and
  example data are illustrative unless promoted into product records.
- Do not hardcode sample project names, session names, file paths, provider
  names, model names, plugin names, skill names, MCP server names, branch names,
  URLs, code snippets, terminal output, badges, statuses, or sample copy.

## Excluded From MVP

- Browser downloads.
- Final brand styling or permanent product copy beyond accepted behavioral
  handoff.
- Provider-native API integrations beyond OpenAI-compatible provider profiles.
- Pi as the first runtime adapter.
- Treating proof code, mock server behavior, or wireframe simulation as
  production-complete behavior.
- Browser tab strip, secondary right-panel tabs, gear tabs, or panel-management
  strips inside the Browser/Files/Terminal tab bar.
- Arbitrary renderer shell spawn, remote shells, SSH, containers, terminal
  multiplexing, and agent auto-run without deliberate future scope.

## Implementation Guardrails

- Production implementation belongs in `backend/`, `frontend/`, and
  `tests/server/`.
- Do not create or use a separate `src-tauri` implementation path.
- Proposed MVP tasks are not active work until the MVP spec is frozen and
  converted into progress items.
- Every mock-backed phase must state exactly what is mocked.
- Do not claim product completion until acceptance passes with real behavior or
  explicitly accepted remaining mocks.

## Context References

- `.agents/references/context/source-provenance.md`

## Related Context

- `.agents/context/index.md`
- `.agents/context/product.md`
- `.agents/context/product-experience.md`
- `.agents/context/product-model.md`
- `.agents/context/feature-surfaces.md`
- `.agents/context/interface.md`
- `.agents/context/runtime-adapter.md`
