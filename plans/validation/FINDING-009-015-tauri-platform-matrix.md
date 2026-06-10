# FINDING-009/015 Tauri Platform Matrix

Date: 2026-06-10

## Question

Which platforms must pass Tauri validation before MVP implementation planning,
and what evidence is still required after an app build exists?

## Source Evidence

Official Tauri docs inspected on 2026-06-10:

- `https://v2.tauri.app/start/prerequisites/`
- `https://v2.tauri.app/reference/webview-versions/`

Relevant source-backed facts:

- Tauri v2 prerequisites list desktop support for macOS Catalina 10.15 and
  later, Windows 7 and later, and Linux distributions with required system
  dependencies.
- Tauri uses Microsoft Edge WebView2 on Windows. WebView2 is Chromium-based,
  can update itself, is preinstalled on Windows 11, and Tauri installers can
  ensure installation on older supported Windows versions.
- Tauri uses WKWebView on macOS. WKWebView is a macOS system component updated
  with OS updates; unsupported macOS versions do not receive WebKit updates.
- Tauri uses WebKitGTK on Linux. The Tauri docs warn Linux WebKitGTK version
  information varies by distribution and should be checked against the distro's
  repositories.
- Linux development dependencies include WebKitGTK 4.1 packages on common
  distributions.

## MVP Platform Matrix

| Tier | Platform | Minimum | Status | Rationale |
| --- | --- | --- | --- | --- |
| Mandatory launch validation | macOS Apple Silicon | macOS 13 Ventura or newer | Required before MVP implementation freeze | Primary local development and validation environment; modern WKWebView; validates desktop control-center thesis with the least platform spread. |
| Compatibility validation | Windows 11 x64 | WebView2 Evergreen runtime | Required before public MVP release, not before initial implementation planning | Important coding-user platform; WebView2 is Chromium-based and updateable; validates packaging, keychain-equivalent storage, file dialogs, shell process supervision, and clipboard/focus behavior. |
| Optional smoke validation | macOS Intel | macOS 13 Ventura or newer | Optional before MVP release | Useful compatibility check if hardware is available; not required to start implementation because Apple Silicon covers macOS Tauri/WKWebView behavior for the first MVP loop. |
| Deferred platform | Linux x64 | Ubuntu 22.04+ or equivalent WebKitGTK 4.1 distro | Deferred until post-MVP or explicit beta scope | Linux WebKitGTK and distro packaging variability add support burden; not required to validate the first desktop control-center thesis. |
| Out of scope | iOS/Android | N/A | Post-MVP | Mobile does not validate local desktop coding workflow. |

## Required Validation Checklist

The mandatory macOS validation build must prove:

- app shell opens and restores correctly;
- dense session transcript renders without overlap;
- tool activity timeline renders and scrolls correctly;
- approval prompts receive focus and cannot be bypassed by UI state;
- file browser handles long paths and large directory lists;
- Git diff viewer renders added/removed lines, long lines, and whitespace;
- text-like artifacts render for plain text, Markdown, logs, diffs, and
  generated source/config files;
- normal text selection/copy works for visible summaries;
- no dedicated raw shell-output export/copy control exists;
- keyboard shortcuts do not conflict with approval safety;
- OS file dialogs can select project roots;
- OS credential storage path is available or provider setup blocks with a
  clear error;
- Rust backend can supervise OpenCode and shell child processes;
- stop behavior remains available from minimized or backgrounded window while
  the app process is running;
- accessibility basics pass for focus order, labels, contrast, and keyboard
  navigation on approval prompts and ledger rows.

Windows compatibility validation must repeat the same checklist before public
MVP release, with special attention to:

- WebView2 runtime availability;
- path separator and long-path behavior;
- PowerShell/cmd shell command display;
- process-tree termination;
- Windows credential storage;
- file dialog project-root selection;
- clipboard/focus behavior in WebView2.

Linux validation, when later accepted, must additionally verify:

- distro WebKitGTK version and package availability;
- Wayland/X11 clipboard and focus behavior;
- Secret Service/keyring availability;
- terminal/shell process supervision differences;
- packaging target selection.

## Blocker Versus Caveat Classification

Blockers for a mandatory platform:

- Tauri app cannot launch.
- WebView cannot render the main app UI.
- approval prompt focus or state can allow protected actions without a clear
  user decision.
- file dialogs cannot select a local project root.
- credential storage is unavailable and cannot fail closed.
- child process stop/supervision is unreliable for OpenCode or shell commands.
- text-like logs/diffs/artifacts are unreadable or overlap in normal viewport
  sizes.

Acceptable MVP caveats:

- minor WebView visual differences that do not affect workflow or safety;
- non-critical keyboard shortcut differences when alternate controls exist;
- platform-specific packaging caveats documented in release notes;
- optional Linux support deferred;
- macOS Intel not tested before the first implementation cycle.

## Chromium/Electron Reconsideration Triggers

Reopen ADR-002 and reconsider Chromium/Electron if MVP or validated near-term
scope requires any of these:

- browser panels;
- browser automation;
- screenshots or DOM extraction;
- active HTML/site rendering;
- rich artifact previews that require Chromium parity;
- web extension integration;
- cross-platform rendering bugs in approval, diff, or artifact views that
  cannot be fixed in Tauri without disproportionate work.

## Decision

Mark the Tauri platform matrix gate as passed.

Tauri remains the selected MVP shell with this scoped matrix:

- implementation planning may proceed after macOS Apple Silicon validation is
  scheduled as the mandatory launch validation target;
- Windows 11 x64 is required before public MVP release;
- Linux is deferred unless explicitly pulled into MVP beta scope;
- actual UI validation remains open until an app build exists.

## Readiness Impact

FINDING-015 is resolved by naming the platform matrix.

FINDING-009 remains partially open because MVP UI and text-like artifact
preview validation cannot be completed until an implementation or prototype
exists.
