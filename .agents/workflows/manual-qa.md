# Manual QA Workflow

Use this for acceptance testing against the built C4OS desktop app. Do not treat
the browser harness as acceptance evidence unless the user explicitly asks for a
browser-only smoke check.

Manual QA is an agent responsibility before stakeholder acceptance. Do not send
the user a checklist as the first verification path for behavior the agent can
exercise locally. Use the checklist yourself, record the result, then ask the
user for stakeholder acceptance only after automated verification and built-app
manual QA have current evidence. If the app cannot be launched, the folder
picker cannot be operated, credentials are unavailable, or the UI cannot be
inspected, mark the item blocked or partially verified with the exact reason.

## Entry Commands

1. Build the desktop backend:
   `npm run backend:build`
2. Launch the debug desktop app with logs attached:
   `npm run backend:run`
   For agent-only QA with an isolated session store, use:
   `npm run backend:run:qa`
3. If a built app must be opened directly, use:
   `open -n backend/target/debug/C4OS.app`
   Only use this after confirming the wrapper binary reflects the current
   build; prefer `npm run backend:run` or `npm run backend:run:qa` otherwise.
   A stale wrapper can keep serving old frontend/cache-token behavior even
   after `npm run backend:build` updates `backend/target/debug/c4os-backend`.
   Before using Computer Use against `backend/target/debug/C4OS.app`, verify
   the wrapper executable or refresh it from the rebuilt backend binary.

Prefer `npm run backend:run` when backend stdout/stderr logs are required.
Use `npm run backend:run:qa` when QA should avoid the normal session store while
keeping backend stdout/stderr attached.
The existing `acceptance:desktop-wrapper` package script
currently points at a missing `scripts/create-desktop-app-wrapper.js`; do not
use it as the acceptance path until that script is repaired.

## App Navigation

1. Confirm the window title is `C4OS` and the URL is `tauri://localhost`.
2. Confirm the start screen heading says `Open a folder to start working`.
3. Use a deterministic path first: click a recent workspace such as
   `Mock Workspace Alpha`.
4. Confirm the app reaches `#new-session` and the heading says
   `What should we build in <workspace>?`.
5. To load the main chat screen, click an existing session in the left sidebar,
   or enter a prompt and click `Send Prompt` when the task requires creating a
   first session.
6. Confirm the app reaches `#chat-session`, the center panel shows session
   messages, and Browser / Files / Terminal remain in the right-panel contract.

Use `Open Folder` only when the acceptance question requires the native trusted
folder flow. Record the selected folder and any `.c4os/workspace.json` side
effect in the QA notes.

## Logs And Inspector

Keep the `npm run backend:run` terminal open throughout the pass and copy any
backend errors, panics, command failures, or stderr output into the QA result.

Renderer inspector logs are required only when available from the built desktop
app. The current repo build does not compile Tauri/Wry with the `devtools`
feature, so backend/stdout logs and visible desktop behavior are the reliable
acceptance evidence today. If devtools are enabled later, capture console errors
from the C4OS WebView inspector and record how the inspector was opened.

## Evidence

Record:

- command used to build and launch the app
- whether credentials were supplied through environment, secure settings, or
  not supplied; never record raw keys or secrets
- route/screen sequence reached
- visible pass/fail observations
- exact folders/projects/sessions used when testing workspace isolation
- backend/stdout log result, even when empty
- inspector availability and any renderer console errors
- screenshots only when the UI behavior is in dispute or hard to describe

Stop when the built desktop app behavior is proved, disproved, blocked, or
requires a product decision.

## Acceptance Handoff

Only ask the user for acceptance after the QA notes state what the agent
checked, what passed, what failed, and what remains out of scope. The handoff
must distinguish agent verification from stakeholder acceptance and must not
imply the user is responsible for finding first-pass regressions.
