# C4OS

C4OS is a coding-first desktop AI workspace centered on a narrow MVP validation
loop before expanding into broader agent-assisted workflows.

Product scope, architecture intent, implementation sequencing, active progress,
and verification evidence now live under `.agents/`. Use
`.agents/specs/c4os-mvp/` for compact durable records, `.agents/progress/` for
active execution state, and `.agents/archive/planning-corpus/` for detailed
historical planning context.

The singular `.agent/` path is deprecated in this repository. Some reusable
ChrisAI skills still mention `.agent/` as their default path, but C4OS uses
`.agents/` as the canonical planning and progress layer.

## Language

**MVP**:
The smallest coding-first desktop product needed to validate one local technical user controlling one active agent session in one selected local Git project at a time, with persistent session state, visible tool activity, explicit approvals, reviewable file changes, OpenCode execution, and OpenRouter model access.
_Avoid_: General-purpose workspace, plugin ecosystem, marketplace, multi-agent workspace, enterprise platform

**Selected Project**:
The one registered local Git project that is visible and active in the MVP workspace at a given time.
_Avoid_: Workspace, project group, multi-project context

**Project Selector**:
The minimal MVP UI for listing registered local Git projects and choosing the selected project. It is not project management and does not include search, grouping, archive, delete, favorites, metadata editing, cross-project views, or multi-project operations.
_Avoid_: Project manager, workspace switcher, project dashboard

**Session**:
A durable conversation and tool-activity record attached to one selected project. An MVP session can be resumed after app restart, but only one session can be active or running at a time.
_Avoid_: Child session, archived session, multi-session workspace, handoff

**Stop Active Run**:
The MVP control that cancels the current runtime/model stream and any supervised child process owned by the app, records a stopped state, preserves transcript and tool history, and allows the user to continue or retry later. It is not session deletion and does not support pause/resume stream, background continuation, partial-response regeneration controls, or killing arbitrary external processes.
_Avoid_: Pause stream, resume stream, background continuation, delete session, kill arbitrary process

**App Lifecycle Execution**:
The MVP rule that active runs may continue while the app window is minimized, as long as the app process remains running and the user can restore the window to see status and stop controls. Closing/quitting the app stops the active run using Stop Active Run behavior and preserves session history. MVP does not include a system tray daemon, background agent service, OS notifications, scheduled runs, or wake/resume automation.
_Avoid_: Closed-app execution, tray daemon, background service, scheduled run, wake automation

**Crash Recovery**:
The MVP behavior after the app crashes or is force-quit during an active run. On next launch, the app marks the previous run as interrupted/crashed, preserves the last persisted transcript and tool records, discards any pending approval prompts, and requires the user to retry or send a new prompt. MVP does not reconnect, continue automatically, replay crash recovery, resend unsent prompts, approve after restart, replay stale approval prompts, or reattach to prior runtime processes.
_Avoid_: Auto-reconnect, automatic continuation, crash replay, unsent prompt resend, approve after restart, stale approval prompt, process reattachment

**Stopped Assistant Message**:
A partial assistant response preserved when the user stops the active run. MVP marks it visibly as interrupted or stopped and does not auto-regenerate, auto-delete, silently mark it complete, allow partial-response editing, show branch/regenerate controls, or automatically continue it.
_Avoid_: Complete response, auto-regenerate, auto-delete, response branch, edit partial response

**Append-Only Transcript**:
The MVP session-history rule that user and assistant messages are appended and retained locally. Prior messages cannot be edited or deleted in MVP. Runtime-generated records may receive status updates, such as complete, stopped, or failed. Users correct prior input by sending follow-up messages or starting a new session. A retry command may exist, but it creates a new appended retry action/status and does not rewrite, replace, or regenerate the failed assistant record in place.
_Avoid_: Message edit, message delete, transcript pruning, message redaction UI, branch from message, conversation rewrite, regenerate in place

**No Global Search**:
The MVP product-UI boundary that users can view the latest session transcript/activity and browse files in the selected project, but cannot search across sessions, transcripts, tool logs, artifacts, projects, or project-wide file contents. Runtime file search tools such as glob/grep may still run inside the selected project during an agent run under normal file-access rules.
_Avoid_: Cross-session search, full-text transcript search, tool-log search, artifact search, project-wide file content search UI

**MVP Shell Command**:
An approval-gated local command run as the current OS user from the selected project root or an approved project subpath. MVP shell execution is not a strong sandbox and does not block normal network access for approved commands.
_Avoid_: Sandboxed command, remote shell, background automation

**Filtered Shell Environment**:
The minimal environment passed to approved MVP shell commands from the app backend, not a full interactive login shell environment. It keeps safe basics such as `PATH`, `HOME`, `USER`, `SHELL`, `LANG`, `LC_*`, `TERM`, and necessary toolchain variables, while stripping obvious credentials and secret-bearing variables such as `*_KEY`, `*_TOKEN`, `*_SECRET`, `*_PASSWORD`, cloud credentials, GitHub tokens, npm tokens, and provider API keys.
_Avoid_: Full shell environment, unfiltered environment, credential inheritance

**Destructive Shell Command**:
An MVP shell command classified as obviously high-risk because it can delete, overwrite, re-permission, reset, prune, uninstall, format, or broadly mutate local state. Destructive shell commands require fresh one-time approval every time and are not covered by session allow.
_Avoid_: Ordinary shell command, session-allowed command

**Shell Session Allow**:
The MVP approval scope that can re-allow matching non-destructive shell commands within the selected project root or approved project subpath for the current session only. It never covers destructive shell commands, outside-root paths, secret-deny files, or Git state changes.
_Avoid_: Broad local automation grant, always allow, project-wide shell approval

**Manual Recovery**:
The MVP rollback posture after a bad approved local action: the app provides prevention and review through approvals, tool logs, changed-file lists, diffs, and stop controls, but recovery is performed manually by the user through Git or local action. MVP does not provide automatic rollback, snapshots, restore points, or undo stacks.
_Avoid_: Automatic rollback, snapshot restore, undo stack

**Shell Output Summary**:
The MVP-persisted shell output record: command metadata, exit status, timestamps, working directory, approval decision, affected-file summary, and redacted/truncated stdout/stderr summary sufficient for common test and build debugging. Live terminal output may be visible while the command is running or while the live terminal buffer remains open. After command completion, an open live terminal drawer may remain temporarily available in the same app session only if labeled as live/ephemeral, but live terminal buffers are not retained after navigation away, reload, app close, or session restore. Post-run durable views use only the bounded persisted summary or explicit output-omitted marker. If a safe redacted/truncated summary cannot be produced, MVP persists the metadata plus an explicit output-omitted marker instead of storing raw stdout/stderr. Shell summaries may expose safe reason labels such as `truncated_by_size`, `redacted_secret_pattern`, or `output_omitted_safe_summary_failed`. Normal OS text selection/copy of visible redacted/truncated shell summaries is acceptable. MVP does not persist unlimited raw stdout/stderr by default and does not provide dedicated raw-output copy/export controls.
_Avoid_: Unlimited raw log, retained live terminal buffer, raw stdout/stderr export, copy raw output button, shell output export, raw fallback when summary fails, redacted substring disclosure, reconstructable truncation metadata

**Structured Denial Result**:
The MVP tool result returned to the runtime when the Approval Gateway denies or blocks an action. It is not silent failure and not fake success. It includes a denial category, such as `user_denied`, `outside_project_root`, `secret_denied`, `destructive_requires_one_time`, or `mvp_scope_blocked`, plus a short user-safe message.
_Avoid_: Silent denial, fake success, raw policy internals

**Git Inspection**:
Read-only Git activity used to show branch, status, changed files, and diffs for the selected project. MVP Git inspection does not require approval, but it is visible in tool activity when performed by the runtime.
_Avoid_: Git action, repository mutation

**Git State Change**:
A Git operation that can change local or remote repository state. MVP gates runtime-proposed Git state changes with approval, but does not provide product workflows for commits, branches, pull requests, merges, rebases, tags, pushes, or worktrees.
_Avoid_: Git inspection, commit workflow, branch management, pull request workflow

**Root AGENTS.md Display**:
The MVP app-owned behavior of detecting and showing the selected project's root `AGENTS.md` as user-visible guidance only. The app does not automatically inject it into model context and does not let it affect permissions. If the user or runtime explicitly reads root `AGENTS.md`, it behaves like any other allowed project-root file read: scoped, logged, and included in model context only because it was explicitly read. If the agent proposes editing root `AGENTS.md`, it behaves like any other project-root file write: approval-gated, logged, and not automatically reloaded into model context or permission state. MVP does not include editable system prompts, project prompts, prompt templates, instruction composers, or hidden app-authored instruction layers. A future skill creator workflow may create explicit, reviewable instruction artifacts, but that is post-MVP and not a generic prompt editor.
_Avoid_: Instruction stack, nested AGENTS.md resolution, policy source, automatic context injection, editable system prompt, project prompt editor, hidden app instructions

**Runtime-Native Instruction Loading**:
Any OpenCode behavior that automatically loads `AGENTS.md` or other instruction files into model context outside the app-owned display-only behavior. For MVP, this must be observable and disclosed, or disabled. Invisible runtime-native instruction loading blocks direct OpenCode MVP use.
_Avoid_: Invisible instruction injection, undisclosed nested instructions

**MVP Compatibility Claim**:
The narrow set of external compatibility statements the MVP can safely make: OpenRouter-backed model access, local Git project support, root `AGENTS.md` display, and app-owned text-like artifact records. MVP does not claim full AGENTS.md compatibility, Agent Skills compatibility, MCP compatibility, Codex plugin compatibility, OpenCode config compatibility, import/export compatibility, or round-trip compatibility.
_Avoid_: Standards-compatible, Codex-compatible, MCP-compatible, full AGENTS.md support, round-trip support

**OpenCode Runtime Settings**:
App-owned adapter settings and references required to launch or map OpenCode during MVP. They are runtime plumbing, not OpenCode config import, mirror, edit, export, or round-trip compatibility. Any existing OpenCode config behavior that affects runtime execution must be discovered in Phase 0 and disclosed.
_Avoid_: OpenCode config compatibility, OpenCode config editor, config round-trip

**Canonical Record**:
An app-owned SQLite or artifact-store record that defines user-facing MVP history, including sessions, messages, tool calls, approvals, artifacts, projects, models, and settings. Runtime-native records may be referenced, but they are not the source of truth for the MVP UI.
_Avoid_: OpenCode session as source of truth, runtime-owned history

**MVP Artifact**:
An app-owned record or file link for text-like session output: plain text, Markdown, logs, diffs, and generated source or config files. MVP artifacts do not include active HTML rendering, rich media/document previews, export workflows, duplication workflows, or executable artifact behavior.
_Avoid_: Browser artifact, rich preview, active HTML, PDF preview, image preview, document preview, spreadsheet preview

**Chromium Strategy**:
A post-MVP option for browser panels, browser automation, screenshots, DOM extraction, generated HTML previews, or rich artifact rendering that need Chromium consistency. MVP does not require bundled Chromium unless Tauri WebView cannot support the basic app UI and text-like previews.
_Avoid_: MVP dependency, default desktop shell requirement

**OpenRouter-Only Provider Path**:
The MVP model-provider boundary: users configure OpenRouter, and all model-backed sessions route through OpenRouter. MVP validates credential presence and selected model route viability, but does not expose OpenRouter credit balance, billing links, top-up flows, invoice links, spend warnings, or account diagnostics. Direct provider integrations, local model providers, fallback routing, and provider-specific app settings are post-MVP.
_Avoid_: Credit balance, billing UI, top-up flow, direct provider setup, local model setup, provider fallback, multi-provider configuration

**OpenRouter Credential Storage**:
The MVP rule that the OpenRouter API key is stored only in the OS keychain or platform credential store. If keychain storage is unavailable or fails, provider setup is blocked with an actionable error. MVP does not store OpenRouter keys in SQLite, project files, env files, shell environments, custom encrypted vaults, or export/import bundles.
_Avoid_: Plaintext key, env-file write, project secret file, custom vault, secret export/import

**OpenRouter Credential Mutation**:
The MVP rule that OpenRouter API key update or revoke is allowed only when no session is running. A running session keeps the credential reference it started with until stopped or complete. MVP does not support hot key rotation, per-session credential switching, or mid-call credential retry.
_Avoid_: Hot key rotation, active-session key swap, per-session credential switch, mid-call credential retry

**Model Context Payload**:
The data sent through OpenRouter for a model call. In MVP, this is limited to the active session transcript, selected model/routing metadata, user-approved or policy-allowed tool results and summaries, and file contents explicitly read by the runtime inside the selected project root. For shell commands, model context may include only the persisted bounded redacted/truncated summary, explicit output-omitted marker, and safe output summary reason labels from the tool record; live raw terminal buffers and omitted raw output are never sent. MVP does not send whole-repo indexes, hidden background file ingestion, app-injected root `AGENTS.md`, artifact contents unless referenced or read, or secret-deny file contents.
_Avoid_: Whole-repo context, background ingestion, automatic AGENTS.md injection, implicit artifact upload, secret file content, live raw terminal context, omitted raw shell output context, reconstructable shell metadata

**Model Context Disclosure**:
The MVP user-facing treatment of OpenRouter-bound context. Provider setup gives a standing disclosure that prompts and bounded context leave through OpenRouter, and model-call activity records a context-source summary such as transcript, explicit file reads, and tool summaries. MVP does not require per-call context approval, raw prompt export, token-by-token context inspection, or an editable context composer.
_Avoid_: Per-call approval modal, raw prompt export, token inspector, editable context composer

**Provider Outage Handling**:
The MVP behavior when OpenRouter or network access fails during model use. Once a user message is submitted, it remains appended even if the model call fails. The failed assistant/run status is persisted, the session remains recoverable with transcript and tool history preserved, and the user may explicitly retry after connectivity or provider recovery. MVP does not delete submitted prompts, silently resend prompts, use offline model fallback, direct-provider fallback, automatic model switching, queued background resend, or synthetic assistant continuation.
_Avoid_: Prompt deletion, silent resend, offline fallback, direct-provider fallback, auto-switch, queued resend, fake continuation

**Session Model**:
The one OpenRouter model selected for an MVP session. It is fixed for the whole session after creation, including when the session is idle or has no active run. Project default model changes may be made while a session is running, but they apply only to future sessions and do not reconfigure, restart, or migrate the active runtime. Mid-session model switching, retry-with-different-model, per-message overrides, per-agent overrides, fallback routes, and budget controls are post-MVP.
_Avoid_: Hot model swap, model switch, retry with different model, per-message model, agent model override, fallback route, budget control

**Effective Session Model**:
The actual provider and model route used by the runtime for a session. In MVP, it must match the app-owned selected OpenRouter model. If OpenCode config can override provider or model routing and the app cannot detect or prevent it, direct OpenCode runtime use is blocked for MVP.
_Avoid_: Runtime-selected model, hidden provider override

**Model Metadata**:
Best-effort OpenRouter or runtime-provided information about a model, such as context limits, tool support, streaming support, pricing source, provider route, source, and freshness timestamp. Stale, missing, or unknown metadata must be labeled but does not block MVP sessions when provider credentials and the selected model route are viable. MVP shows selected model and metadata source/freshness when available, but does not expose per-call token usage, cost estimates, budget meters, spend history, or enforcement.
_Avoid_: Token accounting, cost estimate, spend history, app-owned live catalog sync, cost dashboard, budget enforcement, freshness-gated session start

**Default Coding Agent**:
The single runtime persona used for MVP sessions. The app may store a runtime agent reference if OpenCode requires one, but only as adapter metadata, not as user-facing project default agent or persona configuration. MVP does not expose project default agent UI, custom agents, agent switching, retry-with-different-agent, per-message persona, subagents, or persona management.
_Avoid_: Project default agent UI, custom agent, agent switch, retry with different agent, per-message persona, subagent, persona manager, handoff

**Local Diagnostics**:
On-device logs and diagnostic records used for user-visible troubleshooting. MVP local diagnostics are not uploaded automatically and are not product telemetry.
_Avoid_: Product telemetry, automatic crash upload, support bundle upload, analytics

**Product Telemetry**:
Product usage, analytics, crash, diagnostic, or support data sent from the app to the product operator. MVP does not send product telemetry.
_Avoid_: Local diagnostics, OpenRouter model traffic

**MVP Retention**:
The MVP default that app-owned sessions, messages, tool calls, approvals, local diagnostics, logs, and MVP artifacts remain on device indefinitely. MVP retention is not long-term memory and does not include session delete.
_Avoid_: Session delete, automatic cleanup, quota management, export/import, cross-session memory, durable memory

**Read-Only File Browser**:
The MVP project-root scoped UI for browsing and opening files for inspection. It is not an editor; file changes occur through agent-proposed writes that pass the Approval Gateway.
_Avoid_: Built-in editor, external editor workflow, file manager, outside-root browser

**Approval Gateway**:
The MVP backend control point that classifies file write, shell, and Git state-changing actions, asks the user when required, records approval decisions, and blocks denied actions before execution. Approval prompts are runtime state, not durable decisions; pending prompts are discarded on app close, crash, force-quit, or restart, and the agent must repropose the action later. Answered approval decisions are durable ledger records visible only in the local activity ledger, with structured metadata, decision, timestamp, resulting action status, and bounded redacted prompt summary or diff reference, but no full prompt replay blob or raw secret values. Normal OS text selection/copy of visible ledger text is acceptable, but MVP does not provide dedicated approval copy/export/share controls. File-write approvals show target path, action type, and a bounded diff or summary when available; if a safe diff cannot be produced, the prompt says so clearly and still requires explicit approval. Multiple file writes may be approved in one explicit atomic batch only when every target path, action type, and per-file diff/summary state is visible. MVP caps batch approvals to a fixed product threshold, initially 10 files plus bounded total preview size. If any item is blocked by policy or the batch exceeds caps, the whole batch is blocked and the agent must propose a smaller valid batch.
_Avoid_: Dedicated approval copy button, approval export, approval copy-all, approval support bundle, JSON download, share approval, durable pending approval, approve after restart, stale approval replay, full prompt replay blob, raw command output, raw secret in approval record, approval edit, approval deletion, user-configurable approval threshold, per-project cap, advanced safety setting, policy engine, enterprise policy, governance layer, oversized approval prompt, approve by summary only, approve all future writes, project-wide write approval, checkbox-per-file partial approval, hidden partial execution, full editor, merge UI

**Runtime Viability Gate**:
The Phase 0 requirement that the chosen agent runtime must support structured events and reliable pre-execution interception for MVP file writes, shell commands, and Git state changes. If this cannot be proven through OpenCode directly, a wrapper, proxy, fork, or runtime strategy change is required before MVP buildout continues.
_Avoid_: UI-only approval, post-execution audit, terminal scraping as control

**Project-Root Read**:
A runtime file read whose resolved path stays inside the selected project root. MVP project-root reads, including explicit reads of root `AGENTS.md`, do not need per-read approval, but they are recorded in tool activity.
_Avoid_: External-directory read, unlogged read

**Resolved-Target Containment**:
The MVP path-boundary rule: canonicalize the requested path and resolve traversal segments and symlinks before checking whether the final target is inside the selected project root. Symlinks are allowed only when the final resolved target remains inside the selected project root.
_Avoid_: String-prefix path check, symlink escape

**Secret-Deny File**:
A project file whose path matches the built-in MVP secret-deny patterns, such as `.env` files, private keys, token files, credential files, and common cloud or package-manager credential locations. MVP blocks agent reads and writes for these files with no override. The file browser may show that the file exists, but it does not preview contents.
_Avoid_: Promptable secret read, approval override, secret preview

**Policy Engine**:
A post-MVP policy system for broader project, agent, plugin, MCP, network, and data-flow rules.
_Avoid_: Approval Gateway
