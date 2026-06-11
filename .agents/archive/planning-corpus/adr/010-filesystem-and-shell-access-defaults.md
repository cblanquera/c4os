# ADR-010: Filesystem And Shell Access Defaults

Status: Finalized for MVP file boundaries, shell execution account, shell
network baseline, and shell safety policy.

## Context

The research and security specs originally recommended project-root file access by default, explicit approval for external paths, shell execution defaulting to ask, secret-file deny patterns, filtered inherited environment, and approval for destructive commands.

The review identifies missing requirements: no explicit sandboxing model, no statement on whether agents run as the user's OS account, no network egress policy, and no default-deny details.

## Decision

Use project-root scoped file access and approval-aware shell execution as the default local execution posture.

For MVP, file reads and writes outside the selected project root are blocked. There are no external-directory grants in MVP.

For MVP, shell commands run as the current OS user from the selected project root or an approved project subpath after explicit approval. MVP shell execution is not a strong sandbox and must not be described as one.

Approved MVP shell commands may use normal network access. MVP does not enforce network blocking for shell commands, and approval prompts must disclose that approved commands may access the network.

The file boundary, secret-deny behavior, shell execution account, shell network
baseline, symlink boundary behavior, filtered shell environment baseline,
destructive-command approval behavior, manual recovery posture, and bounded
shell output persistence are finalized for MVP. Exact shell environment,
redaction, truncation, destructive classification, and unclassifiable-command
rules are defined in
`.agents/archive/planning-corpus/validation/FINDING-003-shell-security-policy.md`.

For MVP, every file path is canonicalized before boundary checks. Traversal segments and symlinks are resolved before determining whether access is inside the selected project root. Symlinks are allowed only when the final resolved target remains inside the selected project root. Reads and writes through symlinks that resolve outside the selected project root are blocked, even when the symlink itself is inside the repository.

For MVP, likely secret files are blocked for agent reads and writes even when they are inside the selected project root. Secret-deny files have no approval override in MVP. The built-in deny patterns include `.env` files, private keys, token files, credential files, and common cloud or package-manager credential locations. The file browser may show that these files exist, but it must not preview contents.

For MVP, approved shell commands inherit a minimal backend-filtered
environment, not the full interactive user shell environment. The exact
always-keep, conditionally-keep, and always-strip environment variable policy
is defined in `.agents/archive/planning-corpus/validation/FINDING-003-shell-security-policy.md`.
Approval prompts must disclose that commands run with a filtered environment,
and logs must redact known secret values and secret-shaped environment values.

For MVP, obviously destructive shell commands are classified as high-risk and
require fresh one-time approval every time, even if the user has granted
session allow for ordinary shell commands. The destructive-command categories,
non-examples, and unclassifiable command handling are defined in
`.agents/archive/planning-corpus/validation/FINDING-003-shell-security-policy.md`. MVP uses conservative
pattern detection and warning, not perfect shell-expression understanding.

For MVP, there is no automatic rollback for bad approved local actions. The app provides prevention and review through approval prompts, tool logs, changed-file lists, Git diffs, and stop controls. Recovery is manual through Git or user action. Automatic rollback, snapshots, restore points, and undo stacks are post-MVP because arbitrary shell commands, generated files, package installs, and local toolchains are difficult to reverse reliably.

For MVP, shell output persistence is bounded. The app persists command
metadata, exit status, timestamps, working directory, approval decision,
affected-file summary, and redacted/truncated stdout/stderr summaries. The
exact byte, line, line-length, metadata, safe-label, and output-omission limits
are defined in `.agents/archive/planning-corpus/validation/FINDING-003-shell-security-policy.md`. Live
terminal output may be visible while a command is running or while the live
terminal buffer remains open, and an open drawer may remain temporarily visible
after completion in the same app session when labeled live/ephemeral. Live
buffers are not retained after navigation away, reload, app close, or session
restore. App-owned persisted records remain bounded summaries. If a safe
summary cannot be produced, the app persists command metadata and an explicit
output-omitted marker instead of raw stdout/stderr. Redacted substrings,
sensitive raw byte counts, offsets, hashes, and reconstruction hints are not
persisted. Normal OS text selection/copy of visible summaries is acceptable.
It does not persist unlimited raw stdout/stderr by default and does not provide
dedicated raw-output copy or export controls. Full raw output capture and
export are post-MVP because logs can contain secrets and grow quickly.

For MVP, session allow is available only for matching non-destructive shell commands inside the selected project root or approved project subpath. Session allow never covers destructive shell commands, outside-root paths, secret-deny files, or Git state changes.

## Alternatives Considered

 - Full home-directory access.
 - File picker only.
 - Root-scoped access with explicit external-directory approval.
 - Root-scoped access with no external-directory grants.

## Alternatives That Should Be Considered

 - Separate OS user for agent commands.
 - Containerized or sandboxed shell execution.
 - Current OS user with approval, environment filtering, logging, and no strong sandbox claim.
 - Network-disabled command mode.
 - Normal network access for approved shell commands.
 - Read-only mode for untrusted projects.
 - No shell execution for non-technical onboarding mode.

## Tradeoffs

Root-scoped access is usable and understandable.

File picker only is safer but too restrictive for agent workflows.

Full home access is convenient but unacceptable for sensitive local data.

External-directory approval grants may be useful later, but they create a second boundary and revocation model before MVP validates the core Git-backed workflow.

Strong sandboxing reduces blast radius but adds platform complexity and may break developer environments.

Running as the current OS user preserves local toolchain compatibility but increases damage potential and makes approval UX, filtered environment handling, redaction, and process termination mandatory.

Allowing normal network access preserves common test, build, package-manager, and toolchain workflows, but it leaves approved shell commands as a possible exfiltration path. The MVP mitigation is explicit approval and clear disclosure, not network isolation.

## Consequences

 - Secret-deny file blocking is mandatory and has no MVP override.
 - External-directory prompts are excluded from MVP.
 - Path enforcement must use resolved-target containment, not string-prefix checks.
 - Shell commands must not inherit the full interactive user shell environment.
 - Shell output logging must persist bounded, redacted/truncated summaries rather than unlimited raw stdout/stderr by default.
 - Destructive shell commands must bypass ordinary session allow and require fresh one-time approval.
 - Shell session allow must remain scoped to ordinary in-project shell commands.
 - Recovery after approved local actions is manual; the app must not promise automatic rollback in MVP.
 - Shell commands run as the current OS user and must be approval-gated.
 - The product must not claim strong shell sandboxing in MVP.
 - The product must not claim shell network isolation in MVP.
 - Network egress controls must be revisited before remote tools and plugin scripts are safe.

## ADR Recommendation

Implementation planning must include tests for the exact shell policy in
`.agents/archive/planning-corpus/validation/FINDING-003-shell-security-policy.md` because this is core to
local trust.
