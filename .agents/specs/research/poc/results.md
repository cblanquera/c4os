# Research POC Results

Status: active
Updated: 2026-06-20

## POC-001: Runtime Adapter Target

Status: completed
Confidence: evidence-backed
MVP: yes
Phase: poc
Source:
- `proofs/opencode-runtime/opencode-runtime-evidence-2026-06-20.md`
- `proofs/pi-runtime/pi-runtime-evidence-2026-06-20.md`
Related:
- Q-001
- ASM-001
- RISK-001
- TASK-002
- DEC-006

### Result

Both OpenCode and Pi can support meaningful parts of the C4OS runtime adapter
contract, but neither should become the visible product model. The MVP should
use a thin C4OS-owned runtime adapter boundary and pick an implementation
backend per capability.

### OpenCode Findings

- Pass: server startup, project/path inspection, session creation, session
  resume, server/session event streaming, permission configuration, and
  session abort.
- Partial: model-backed prompt execution was not run because the proof avoided
  provider credentials and token spend.
- Partial: permission response APIs exist, but a live permission request was
  not produced without a credentialed tool-call run.
- Gap: OpenCode defaults to user-level state/config paths unless C4OS supplies
  isolated runtime environment and config paths.

### Pi Findings

- Pass: session creation/resume through repository contracts, prompt execution
  through `Agent.prompt`, lifecycle/tool event streaming, pre-execution tool
  interception through `beforeToolCall`, approval denial through `block: true`,
  and run lifecycle control through `abort`/`waitForIdle`.
- Partial: artifact identity remains a C4OS concern layered on top of transcript
  and tool-result state.
- Gap: Pi does not provide a built-in filesystem/process/network/credential
  permission sandbox; C4OS must own those boundaries.

### Runtime Direction

Use a thin C4OS-owned adapter for MVP. Treat Pi as the better proof for
in-process approval interception and app-owned tool policy. Treat OpenCode as
the better proof for a server-backed coding runtime with sessions, events, and
CLI ecosystem compatibility. Do not expose either runtime's native concepts as
the primary UX before freeze.

### Follow-Up

Before implementation freeze, decide whether MVP starts with one backend behind
the adapter or keeps both as pluggable proof backends. If only one backend is
selected, choose based on the first MVP's highest-risk requirement:

- choose Pi if app-owned approval interception and tight embedding matter most
- choose OpenCode if server-mode coding workflow, existing CLI ecosystem, and
  session/event APIs matter most

## POC-002: Browser Isolation

Status: completed
Confidence: evidence-backed
MVP: yes
Phase: poc
Source:
- `proofs/native-browser-plugin/native-browser-plugin-evidence-2026-06-20.md`
- `proofs/native-browser-wry/native-browser-wry-evidence-2026-06-20.md`
- `proofs/native-browser-tauri/native-browser-tauri-evidence-2026-06-20.md`
Related:
- Q-002
- DEC-004
- DEC-007
- RISK-002
- TASK-003

### Result

Browser isolation is viable for a constrained preview/browser surface. Later
grill decisions promoted Browser to MVP product scope as a user-owned desktop
Browser with project-scoped profile, local file browsing, public web browsing,
and request-scoped agent browsing. The POC findings remain implementation risk
evidence, especially around direct Tauri `WebviewWindow` internals.

### Findings

- Pass: sandboxed headless browser proof blocked host bridge/secret access,
  denied `file:` and `javascript:` navigation, and modeled reload/stop state.
- Pass with warning: raw Wry proof blocked privileged protocols and did not
  expose Tauri globals or C4OS secrets, but macOS content observed `window.ipc`;
  no Wry IPC handler was registered.
- Fail for full unbridged Browser: Tauri WebviewWindow proof exposed
  `window.__TAURI_INTERNALS__`, even though the command invocation was rejected
  and did not leak the secret sentinel.

### Browser Risk Direction

Do not ignore the failed direct Tauri `WebviewWindow` proof. Implement Browser
as MVP product scope while preserving agent authority records, project-scoped
profile behavior, download exclusion, and explicit handling for local-file and
logged-in browsing risks.

## POC-003: Terminal Lifecycle

Status: completed
Confidence: evidence-backed
MVP: yes
Phase: poc
Source:
- `proofs/native-terminal-plugin/native-terminal-plugin-evidence-2026-06-20.md`
- `proofs/native-terminal-tauri/native-terminal-tauri-evidence-2026-06-20.md`
Related:
- Q-003
- DEC-008
- RISK-002
- TASK-004

### Result

Backend-owned terminal lifecycle is viable on this macOS host. Later grill
decisions promoted Terminal to MVP product scope with separate user terminal
and agent-owned command terminal surfaces, deterministic allowlist, approvals,
and audit records.

### Findings

- Pass: Python stdlib PTY proof validated project-scoped startup, output
  streaming, input, resize, trusted-root blocking, approval-gated high-impact
  command blocking, failure observation, cancellation, and cleanup.
- Pass: Rust/macOS PTY proof validated trusted-root scoping, approval-gated
  launch, PTY startup, streaming input/resize, cancellation/cleanup, and
  command failure observation.

### Terminal Risk Direction

Do not allow renderer shell spawn. Implementation still needs Tauri command
capability scoping, renderer backpressure/audit persistence, and non-macOS
PTY/ConPTY confirmation, but those are implementation risks rather than scope
deferrals.
