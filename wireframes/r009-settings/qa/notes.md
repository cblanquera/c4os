# R009 Settings Browser QA

## Round 1 — 2026-07-17

### Environment

- Local preview: `http://127.0.0.1:4179/`
- Browser automation: Playwright CLI, Chromium
- Wide viewport: 1440 × 960
- Narrow viewport: 720 × 900

### Verified

- `workflows.html` loads and the Providers and Models workflow opens `settings.html#providers`.
- All six destination buttons update the active panel and URL hash.
- Providers renders connected, inactive, and error states without console errors.
- Add Provider opens as an accessible dialog; OpenAI Compatible changes the seeded fields; missing API-key validation is visible; Escape closes the dialog.
- Models renders OpenRouter and Hugging Face groups from active providers with search, default, and visibility controls present.
- Plugins switches between Installed and Directory; installing Analytics Workspace adds it to the Installed list with an enabled switch.
- Skills renders the Installed/Catalog tabs, searchable scoped rail, selected item state, detail view, related files, and enable switch.
- MCP renders global controls and connected, stopped, and error rows; Add Server changes from command fields to URL when HTTP is selected.
- Configuration marks edits Unsaved and renders a Parse error with an invalid token after Validate.
- The wide Providers view remains centered and readable at 1440 × 960.
- The narrow Providers view collapses navigation labels to icons and remains readable without horizontal document overflow at 720 × 900.
- Final page loads reported no browser console errors. Chromium emitted a non-blocking autocomplete suggestion for a form input.

### Screenshots

- `round-1-providers-wide.png`
- `round-1-providers-narrow.png`
- `round-1-skills-wide.png`

### Deferred

- Native OS application-menu invocation cannot be tested in a static browser artifact.
- Real provider authentication, model discovery, plugin or skill installation, MCP process execution, persistence, and filesystem-backed configuration writes remain simulated.

## Round 2 — 2026-07-17

### Environment

- Local preview: `http://127.0.0.1:4179/`
- Browser automation: Playwright CLI, Chromium
- Peg-review viewport: 1170 × 919
- Narrow desktop viewport: 720 × 900

### Verified

- The selected window title bar is absent from the DOM and no empty 48px region remains.
- The sidebar reads, in order: Back to C4OS; Settings; Providers; Models; Runtimes; Configuration; divider; Plugins; Skills; MCP Servers.
- The desktop sidebar is 260px wide, uses a light gray surface, and gives the selected destination a white rounded row.
- Back to C4OS points to `./index.html#chat`.
- All seven destination hashes render the matching heading and selected navigation item.
- Runtimes renders Local Desktop, Docker, and Remote SSH states; Configure produces the expected notifier feedback.
- At 1170 × 919 and 720 × 900, the full peg-style menu remains visible and document width equals viewport width.
- Final page loads reported no browser console errors.

### Screenshots

- `round-2-navigation-wide.png`
- `round-2-navigation-narrow.png`
- `round-2-runtimes-wide.png`

### Deferred

- Native OS application-menu invocation cannot be tested in a static browser artifact.
- Runtime discovery, container setup, remote connections, and persistence remain simulated.

## Round 3 — 2026-07-17

### Source Verification

- Sidebar width: 224px.
- Back/menu labels: 13px.
- Settings group label: 10px.
- Back icon: 18px; destination icons: 17px.
- Destination row minimum height: 40px.
- The Round 2 order, divider, hashes, and interaction code are unchanged.
- JavaScript syntax and whitespace checks pass.

### Browser Limitation

- Automated in-app browser access to the local `file://` preview was blocked by browser security policy, so this round requires a manual refresh for visual confirmation.

## Round 4 — 2026-07-17

### Source Verification

- Providers renders four uniquely labeled saved profiles with Edit and enable controls.
- Provider Type includes OpenRouter, Hugging Face, OpenAI, and OpenAI Compatible; LiteLLM is represented by a compatible profile with its own URL.
- Preset types expose Label and API Key while keeping their standard endpoints implicit.
- OpenAI Compatible exposes API Base URL, Auth, API Key, API key header name when applicable, and JSON Headers.
- Auth includes Bearer token, API key header, and None; None removes the API-key requirement.
- Add and Edit use the same form; Edit preserves a saved key when the field remains blank.
- Duplicate labels and invalid JSON headers produce inline validation feedback.
- JavaScript syntax and whitespace checks pass.

### Browser Limitation

- Automated in-app browser access to the local `file://` preview remains blocked by browser security policy, so this round requires manual refresh and interaction review.

## Round 5 — 2026-07-17

### Source Verification

- Removed Saved profiles and its supporting sentence.
- Removed the AI Runtime eyebrow from Providers and Models.
- Removed badges, endpoints, and credential-state copy from all four provider rows.
- Provider row secondary copy is now OpenRouter, Hugging Face, OpenAI, or OpenAI Compatible.
- Add Provider and initial Edit controls include the shared dialog-open hook plus provider-specific initialization.
- Newly saved rows retain provider-specific Edit initialization.
- Settings CSS and JavaScript imports use the Round 5 cache version to prevent stale local-file assets.
- JavaScript syntax and whitespace checks pass.

### Browser Limitation

- Automated inspection of the open local `file://` tab was blocked by browser security policy; manual refresh and click verification remain required.

## Round 6 — 2026-07-17

### Source Verification

- Root cause identified: global conditional-field selectors matched `data-provider-auth` and `data-provider-headers` on the LiteLLM profile row before the dialog fields.
- All conditional provider-field selectors are now scoped to `data-provider-form`.
- OpenAI Compatible can reveal Auth and Headers without colliding with profile metadata.
- Settings assets use the Round 6 cache version.
- JavaScript syntax and whitespace checks pass.

### Browser Limitation

- Automated inspection of the local `file://` tab remains blocked by browser security policy; manual refresh and interaction confirmation remain required.

## Round 12 — 2026-07-17

### Source Verification

- The Configuration Runtime card no longer contains Default model or its select.
- Default Approval Policy is now the first Runtime row, followed by Restore last workspace.
- The page description no longer refers to runtime defaults.
- Settings assets use the Round 12 cache version.
- JavaScript syntax and whitespace checks pass.

### Browser Limitation

- Automated inspection of the local `file://` tab remains blocked by browser security policy; manual refresh remains required.

## Round 13 — 2026-07-17

### Source Verification

- Configuration includes an Advanced link beside Default Approval Policy.
- Advanced Policies uses the shared Settings shell with Configuration selected and a direct Back to Configuration link.
- Nine group controls represent Terminal, Git, Filesystem, Browser, Network, Credentials, Processes and apps, Desktop facilities, and C4OS authority.
- The policy dataset contains all 71 supplied identities.
- Every identity select offers Use default, Always allow, Always ask for permission, and Never allow.
- Search matches across all groups; group selection clears search and restores group context.
- Changing a select enables Save Policies; returning all values to the saved state disables it; saving promotes the draft and emits confirmation.
- `workflows.html` includes an Advanced Policies entry.
- JavaScript syntax and whitespace checks pass.

### Browser Limitation

- Automated inspection of local `file://` pages remains blocked by browser security policy; manual visual and interaction confirmation remain required.

## Round 14 — 2026-07-17

### Source Verification

- The Advanced link is stacked beneath the Default Approval Policy select.
- The standalone Configuration back-link is removed from Advanced Policies.
- The Configuration sidebar item remains selected and links back to the Configuration screen.
- Asset cache versions advance for both affected pages.
- JavaScript syntax and whitespace checks pass.

### Browser Limitation

- Automated inspection of local `file://` pages remains blocked by browser security policy; manual refresh remains required.

## Round 15 — 2026-07-17

### Source Verification

- The Advanced Policies page header centers Save Policies against the title block.
- The page-specific stylesheet uses the Round 15 cache version.
- JavaScript syntax and whitespace checks pass.

### Browser Limitation

- Automated inspection of the local `file://` page remains blocked by browser security policy; manual refresh remains required.

## Round 20 — 2026-07-17

### Source Verification

- The Plugins page header contains only its title and supporting description.
- The Extensions eyebrow is absent from Plugins.
- Settings assets use the Round 20 cache version.
- JavaScript syntax and whitespace checks pass.

### Browser Limitation

- Automated inspection of the local `file://` page remains blocked by browser security policy; manual refresh remains required.

## Round 19 — 2026-07-17

### Source Verification

- The Configuration page header contains only its title and supporting description.
- The Advanced eyebrow is absent.
- Settings assets use the Round 19 cache version.
- JavaScript syntax and whitespace checks pass.

### Browser Limitation

- Automated inspection of the local `file://` page remains blocked by browser security policy; manual refresh remains required.

## Round 16 — 2026-07-17

### Source Verification

- The Plugins segmented tabs sit inside a full-width wrapper and align left with plugin content.
- Directory search and the Built by C4OS marketplace chooser share one aligned toolbar.
- Add Marketplace opens a dialog with Source, Git ref, and Sparse paths; Source is required.
- The plugin details dialog no longer renders a Plugin details eyebrow.
- Uninstall is visible only for an installed plugin, removes its card, updates the Installed count, and restores an available directory Install action.
- Marketplace and plugin dialogs close through existing explicit controls and Escape.
- Settings assets use the Round 16 cache version.
- JavaScript syntax and whitespace checks pass.

### Browser Limitation

- Automated inspection of the local `file://` page remains blocked by browser security policy; manual refresh and interaction confirmation remain required.

## Round 18 — 2026-07-17

### Source Verification

- Every seeded plugin detail provides a compact identity mark beside its title.
- Website, Terms, and Privacy Policy appear for installed and uninstalled plugins because they share the same detail renderer.
- Policy-link activation emits simulated confirmation without navigating away from the static artifact.
- Install, Uninstall, and Done states are unchanged.
- Settings assets use the Round 18 cache version.
- JavaScript syntax and whitespace checks pass.

### Browser Limitation

- Automated inspection of the local `file://` page remains blocked by browser security policy; manual refresh remains required.

## Round 17 — 2026-07-17

### Source Verification

- Uninstalled plugin details expose Install and hide Uninstall.
- Installed plugin details expose Uninstall and hide Install.
- Dialog Install invokes the existing directory install workflow and closes the dialog.
- Done remains available in both states.
- Settings assets use the Round 17 cache version.
- JavaScript syntax and whitespace checks pass.

### Browser Limitation

- Automated inspection of the local `file://` page remains blocked by browser security policy; manual refresh and interaction confirmation remain required.

## Round 10 — 2026-07-17

### Source Verification

- The Selected badge is explicitly placed in runtime grid column 2.
- Both runtime radios are explicitly placed in the fixed-width grid column 3, preventing selection-state content from shifting them.
- Settings assets use the Round 10 cache version.
- JavaScript syntax and whitespace checks pass.

### Browser Limitation

- Automated inspection of the local `file://` tab remains blocked by browser security policy; manual refresh remains required.

## Round 11 — 2026-07-17

### Source Verification

- Default Approval Policy replaces Approval policy.
- Browser Environment follows the login-shell row and offers All browsers, Per project, Per chat session, and None.
- Customize Configuration replaces Configuration scope and exposes Open config.toml externally.
- The Raw configuration header, textarea, state, error, and editor actions are removed from markup, styling, and JavaScript.
- The external-open button emits simulated confirmation feedback.
- Settings assets use the Round 11 cache version.
- JavaScript syntax and whitespace checks pass.

### Browser Limitation

- Automated inspection of the local `file://` tab remains blocked by browser security policy; manual refresh and interaction confirmation remain required.

## Round 8 — 2026-07-17

### Source Verification

- The all-visible-results-disabled action is Enable results.
- The mixed/default action remains Disable results.
- Visible-result scope and switch behavior are unchanged.
- Settings JavaScript uses the Round 8 cache version.
- JavaScript syntax and whitespace checks pass.

### Browser Limitation

- Automated inspection of the local `file://` tab remains blocked by browser security policy; manual refresh remains required.

## Round 9 — 2026-07-17

### Source Verification

- OpenCode is the seeded saved runtime and Pi is the alternate runtime.
- The runtime rows expose a mutually exclusive radio state with the selected marker aligned to the right.
- Save Runtime is disabled while the draft matches the saved runtime.
- Choosing another runtime updates the draft selection and enables Save Runtime.
- Saving promotes the draft choice to the saved state, disables the action, and emits a confirmation notice.
- Settings assets use the Round 9 cache version.
- JavaScript syntax and whitespace checks pass.

### Browser Limitation

- Automated inspection of the local `file://` tab remains blocked by browser security policy; manual refresh and interaction confirmation remain required.

## Round 7 — 2026-07-17

### Source Verification

- Models renders one aligned seven-row list with every switch enabled by default.
- The compact toolbar contains model search, provider-profile filter, and Disable results.
- Search and provider filter combine to define visible rows and the empty state.
- Disable results affects only visible rows; filtered-out rows retain their state.
- When all visible rows are disabled, the control becomes Bulk enable results.
- A mixed visible set keeps Disable results.
- Individual switches update their Enabled/Disabled label and recalculate the bulk action.
- Refresh Models remains in the upper-right page header.
- Settings assets use the Round 7 cache version.
- JavaScript syntax and whitespace checks pass.

### Browser Limitation

- Automated inspection of the local `file://` tab remains blocked by browser security policy; manual refresh and interaction confirmation remain required.

## Round 21 — 2026-07-17

### Source Verification

- Skills renders a full-width search field and one installed-skill resource list.
- Every row uses the same circular document icon beside the name, a truncated one-line description, and a flush trailing availability switch.
- Row identity controls open a dedicated details dialog without changing availability.
- The dialog repeats the circular icon beside the skill name without a redundant `Skill` label.
- Row and dialog switches update the same in-memory availability value.
- Uninstall removes the selected skill and Try in chat reports the simulated handoff.
- Empty search results replace the list with a concise empty state.
- Settings assets use the Round 21 cache version.
- JavaScript syntax and whitespace checks pass.

### Browser Limitation

- Automated inspection of the local `file://` tab was attempted and blocked by browser security policy; manual refresh and interaction confirmation remain required.

## Round 22 — 2026-07-17

### Source Verification

- The Skill details dialog header vertically centers the close control against the icon-and-title identity row.
- The rule is scoped to the Skill dialog and does not change other dialog headers.
- Settings assets use the Round 22 cache version.
- JavaScript syntax and whitespace checks pass.

### Browser Limitation

- Automated inspection of the local `file://` tab remains blocked by browser security policy; manual refresh remains required.

## Round 23 — 2026-07-17

### Source Verification

- MCP Servers renders three seeded names with aligned configure controls and availability switches.
- Add Server opens a clean custom MCP dialog in STDIO mode.
- STDIO includes command, repeatable arguments, environment variables, passthrough variables, and working directory.
- Streamable HTTP includes URL, bearer-token environment variable, repeatable headers, and repeatable environment-backed headers.
- Every repeated field type supports add and remove controls.
- Configure opens the selected server data for editing; Save adds or updates the list after required-field and unique-name validation.
- The old permission card, status metadata, transport select, test action, retry behavior, and SSE mode are absent.
- Settings assets use the Round 23 cache version.
- JavaScript syntax and whitespace checks pass.

### Browser Limitation

- Automated inspection of the local `file://` tab remains blocked by browser security policy; manual refresh and interaction confirmation remain required.

## Round 24 — 2026-07-17

### Source Verification

- The complete app and Settings surface now import only `styles.css`, `markdown.js`, and `script.js` from `index.html`.
- Every Settings workflow link targets an `index.html#settings/...` route.
- Settings and Advanced Policies selectors are scoped under `.settings-spa` to avoid overriding workspace components.
- JavaScript syntax and static route/reference checks pass.
- The redundant settings-specific and policy-specific page and asset files are absent.

### Browser Verification

- Served the static artifact from `http://127.0.0.1:8765/` to avoid the earlier `file://` inspection restriction.
- Verified direct Providers rendering, Models navigation, Configuration navigation, Advanced Policies routing, Configuration-selected state on Advanced Policies, Back to C4OS, workspace restoration, and workspace-header Settings entry.
- Visually inspected the Providers destination at the active desktop viewport; alignment and theme remained consistent with the approved Settings rounds.
- Browser console contained no errors or warnings.
