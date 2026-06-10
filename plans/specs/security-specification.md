# Security Specification

## Security Model

The application must assume that model output, generated files, project files, and shell output can be malicious or misleading. Remote web content, MCP tool descriptions, and plugins remain post-MVP untrusted surfaces.

## Trust Boundaries

 - Tauri frontend: untrusted UI surface relative to backend capabilities.
 - Tauri Rust backend: trusted execution boundary and owner of the MVP Approval Gateway.
 - OpenCode runtime: semi-trusted execution engine constrained by the backend Approval Gateway.
 - Project files: user data, not automatically safe instructions.
 - Runtime-native instruction loading: allowed for MVP only if observed and disclosed, or disabled.
 - MCP servers, plugins, and remote web pages: post-MVP untrusted surfaces.

## MVP Approval Gateway

The MVP Approval Gateway is a backend-owned control point for file write, shell, and Git state-changing actions. It classifies the requested action, asks the user when required, records the decision, and blocks denied actions before execution. Project-root file reads and Git inspection do not require per-read approval, but runtime reads and runtime-performed Git inspection are logged in tool activity.

Denied or blocked actions return a structured denial result to the runtime. Denial results must not be silent failures or fake successes. MVP denial categories include `user_denied`, `outside_project_root`, `secret_denied`, `destructive_requires_one_time`, and `mvp_scope_blocked`.

Post-MVP Policy Engine concepts include workspace trust, project policy, agent policy, plugin policy, MCP policy, network policy, and data-flow-aware policy.

## Approval UX

Approvals must show:
 - Tool name and provider.
 - Arguments and target resources.
 - Risk category.
 - Approval source.
 - Whether the call reads, writes, executes, sends network data, or uses credentials.
 - Scope of approval.

MVP approval scopes:
 - Once.
 - Session, only for matching non-destructive shell commands inside the selected project root or approved project subpath.
 - Deny.

## Filesystem Security

 - Default access is project root only.
 - Every file path must be canonicalized before boundary checks.
 - Traversal segments and symlinks are resolved before determining whether access is inside the selected project root.
 - Symlinks are allowed only when the final resolved target remains inside the selected project root.
 - Reads and writes through symlinks that resolve outside the selected project root are blocked, even when the symlink itself is inside the repository.
 - Project-root reads are allowed without per-read approval and are logged.
 - File reads outside the selected project root are blocked for MVP.
 - Secret-deny files are blocked for agent reads and writes even when they are inside the selected project root.
 - MVP secret-deny files have no approval override.
 - Built-in secret-deny patterns include `.env` files, private keys, token files, credential files, and common cloud or package-manager credential locations.
 - The read-only file browser may show that a secret-deny file exists, but it must not preview contents.
 - File writes require approval and are blocked outside the selected project root.
 - Generated artifacts are stored in app-managed artifact directories unless explicitly exported.

## Shell Security

 - Shell execution defaults to `ask`.
 - MVP shell commands run as the current OS user.
 - MVP shell execution is not a strong sandbox and must not be described as one.
 - Approved MVP shell commands may use normal network access; MVP does not enforce network blocking.
 - Approval prompts must make clear that shell commands may access the network.
 - Shell session allow applies only to matching non-destructive shell commands inside the selected project root or approved project subpath.
 - Shell session allow never covers destructive shell commands, outside-root paths, secret-deny files, or Git state changes.
 - Destructive shell commands require fresh one-time approval every time, even when ordinary shell commands have session allow.
 - MVP destructive command classification is conservative pattern detection, not a complete shell interpreter.
 - High-risk examples include `rm -rf`, recursive deletion, `chmod -R`, `chown -R`, `dd`, disk formatting, force Git cleanup/reset, package-manager uninstall/prune commands, and commands targeting broad paths such as `/`, `$HOME`, or the project root.
 - Commands run with project working directory and a backend-filtered environment, not the full interactive user shell environment.
 - The filtered environment keeps safe basics such as `PATH`, `HOME`, `USER`, `SHELL`, `LANG`, `LC_*`, and `TERM`.
 - Toolchain variables may be passed only when needed for common local build and test workflows.
 - The backend strips obvious credentials and secret-bearing variables, including `*_KEY`, `*_TOKEN`, `*_SECRET`, `*_PASSWORD`, cloud credentials, GitHub tokens, npm tokens, and provider API keys.
 - Secrets must not be injected into shell environment in MVP.
 - Approval prompts must state that commands run with a filtered environment.
 - Live terminal output may be visible while a command is running or while the live terminal buffer remains open.
 - An open live terminal drawer may remain temporarily visible after completion in the same app session only when labeled live/ephemeral.
 - Live terminal buffers are not retained after completion, navigation away, reload, app close, or session restore.
 - PTY output is persisted as a redacted/truncated shell output summary by default.
 - Shell output summaries include command metadata, exit status, timestamps, working directory, approval decision, affected-file summary, and bounded stdout/stderr excerpts.
 - If a safe shell output summary cannot be produced, the persisted record must include command metadata and an explicit output-omitted marker instead of raw stdout/stderr.
 - Shell output summaries may expose safe reason labels such as `truncated_by_size`, `redacted_secret_pattern`, and `output_omitted_safe_summary_failed`.
 - Shell output summary metadata must not expose redacted substrings, sensitive raw byte counts, offsets, hashes, or other reconstruction paths to raw output.
 - Model context may include only the persisted shell output summary, output-omitted marker, and safe reason labels, never live raw terminal buffers, omitted raw shell output, or reconstructable metadata.
 - Normal OS text selection/copy of visible shell output summaries is acceptable.
 - MVP does not persist unlimited raw stdout/stderr by default.
 - MVP does not provide dedicated raw shell output copy or export controls.
 - Shell output summaries use redaction for known secret values and secret-shaped environment values.
 - The backend must support termination for supervised child processes owned by the active run when the user stops the run.
 - MVP stop controls must not kill arbitrary external processes outside the app-supervised runtime/process tree.
 - MVP does not provide automatic rollback after approved local actions.
 - Recovery after a bad approved local action is manual through Git or user action, supported by logs, changed-file lists, diffs, and stop controls.

## Post-MVP MCP Security

Follow MCP trust guidance:
 - User consent for data access and operations.
 - Human-in-the-loop tool invocation.
 - Treat tool annotations as untrusted unless the server is trusted.
 - Require approval for sampling requests.
 - Scope roots to approved project folders.

## Post-MVP Plugin Security

 - Indexing must not execute plugin code.
 - Plugin scripts execute only as approved tools.
 - Plugins declare capabilities before enablement.
 - Remote plugin sources should support checksum or signatures in future phases.

## Secret Storage

Store the MVP OpenRouter API key only in the OS keychain or platform credential store. SQLite may store a provider-ready flag or keychain reference, but it must not store the raw key.

If OS keychain or platform credential storage is unavailable or fails, provider setup is blocked with an actionable error. MVP does not fall back to plaintext SQLite, project files, env-file writes, shell environment injection, custom encrypted vaults, or secret export/import.

OpenRouter API key update and revoke operations are blocked while a session is running. A running session uses the credential reference captured at session start until it is stopped or complete. MVP does not support hot key rotation, per-session credential switching, or mid-call credential retry.

## Model Context And Provider Data Flow

MVP model calls through OpenRouter may include only the active session transcript, selected model and routing metadata, user-approved or policy-allowed tool results and summaries, and file contents explicitly read by the runtime inside the selected project root. For shell command tool results, model context may include only the persisted bounded redacted/truncated summary, explicit output-omitted marker, and safe output summary reason labels.

MVP must not send whole-repo indexes, hidden background file ingestion, automatic app-owned root `AGENTS.md` injection, artifact contents unless referenced or explicitly read, secret-deny file contents, live raw terminal buffers, omitted raw shell output, redacted substrings, sensitive raw byte counts, offsets, hashes, or reconstruction metadata. Runtime-native instruction loading is allowed only if observed and disclosed, or disabled, under the runtime viability gate.

MVP does not prompt for per-call approval before sending bounded model context through OpenRouter. Provider setup must provide standing disclosure that prompts and bounded context leave the machine. Model-call activity must retain a context-source summary sufficient to identify source categories, such as transcript, explicit file reads, and tool summaries, without requiring raw prompt export or a token-by-token context inspector.

When OpenRouter or network access fails, new model calls must fail closed with an explicit provider or network error. MVP must not use offline model fallback, direct-provider fallback, automatic model switching, queued background resend, or synthetic assistant continuation.

## Telemetry And Diagnostics

 - MVP sends no product telemetry.
 - MVP diagnostics are local-only and stored on device.
 - MVP does not automatically upload crash reports, logs, transcripts, tool outputs, artifacts, screenshots, or support bundles.
 - OpenRouter model traffic is required product behavior and must be disclosed separately from product telemetry.
 - Local diagnostics must redact provider credentials and known secret values.

## Desktop Security

Use Tauri capabilities to restrict frontend access. Disable remote web content access to backend commands unless explicitly required. Browser panels must not receive privileged IPC.

Recommendation: MVP security enforcement belongs in the Rust backend Approval Gateway, not only frontend UI or runtime-native checks.

Alternatives considered:
 - Frontend-only checks: easy to bypass.
 - Runtime-only checks: misses app-owned approval records, local storage boundaries, and future plugin/browser surfaces.

Why selected: defense-in-depth is required for local shell and file access.
