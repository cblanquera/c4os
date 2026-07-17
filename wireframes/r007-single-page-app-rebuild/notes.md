# r007 Single Page App Rebuild Notes

## Review Round 1 - r04 Baseline Reconstruction

Date: 2026-07-13

### Changed

- Created `r007-single-page-app-rebuild` as a new major wireframe revision.
- Reconstructed the r04 single-page application contract with one product entry point and sixteen hash-routed screen states.
- Added the required revision-local `specs.md`, `notes.md`, and `workflows.html` files.
- Copied the bundled wireframe library's tokens, reset, and base CSS as the r007 foundation.
- Remapped the C4OS-specific r04 stylesheet onto the bundled grayscale tokens while preserving the r04 shell, density, route structure, and component treatment.
- Preserved the r04 functional router, shared renderers, Lucide-guided inline SVG icons, panel controls, message disclosure, composer states, tool surfaces, and settings interactions.
- Added explicit white surface ownership to the center workbench and settings content after browser QA exposed transparent-region compositing.
- Replaced remaining near-black interactive control fills with the library's medium grayscale emphasis tokens.

### Feedback Applied

- Applied the instruction to start from r04 rather than r05 or r06.
- Applied the instruction to treat `wireframe/lib` as a guide and starting foundation rather than a rigid template or visual ceiling.
- Applied the instruction to make the SPA as close to product truth as a functional wireframe can be.
- Kept all r05 additions deferred until this r04 parity round is reviewed.
- Ignored `wireframes/r06-final-implementation/` completely.

### Verified

- `script.js` passes `node --check`.
- All product assets use document-relative paths.
- All sixteen r04 hash routes render in the same `index.html` document.
- Every route renders one application root and a focusable `#main` region.
- No route produced document-level horizontal overflow at the browser review viewport.
- No route exposed TODOs, review-round labels, coverage matrices, annotations, or implementation notes in the product UI.
- App Start navigates to New Session through client-side SPA routing.
- Browser Back and Forward restore App Start and New Session correctly.
- Left-panel collapse updates the shell state and `aria-pressed` value.
- Agent Show More reveals the additional message content and becomes Show Less.
- File Explorer navigation opens the File Editor with breadcrumbs and eight rendered code lines.
- Settings navigation opens Plugins without leaving the SPA.
- The composer approval control exposes both approval options.
- `workflows.html` contains seven document-relative workflow starting points and no horizontal overflow.
- Browser console inspection found no warnings or errors.
- Visual QA covered App Start, the New Session shell, and Settings > Plugins.

### Review Now

- Whether the r007 App Start remains faithful to the r04 trust-first entry.
- Whether the three-panel New Session shell preserves the correct r04 structure, density, and composer hierarchy.
- Whether the r04 settings structure remains recognizable after adopting the bundled grayscale foundation.
- Whether any r04 route or interaction is missing before the r05 additions begin.

### Simulated Or Deferred

- Folder opening, cloning, workspace-file opening, prompt submission, provider and model persistence, runtime persistence, Browser rendering, file editing and saving, Terminal execution, and settings persistence remain review simulations.
- r05 Batch 1 through Batch 5 additions are intentionally deferred.
- Production frontend behavior, backend integration, security enforcement, persistence, and native desktop behavior are outside this wireframe round.

### Open Questions

- None blocking the r04 parity review.

### Approval Path

If Review Round 1 is approved, the next step is Review Round 2 in this same r007 revision: apply the accepted r05 Batch 1 global-header and plugin-panel shell architecture to the r04 SPA foundation. If changes are requested, revise this r04-parity round before introducing any r05 behavior.

## Review Round 2 - Browser Annotation Refinements

Date: 2026-07-13

### Changed

- Removed the Send control's separate hover fill so it retains one stable emphasized treatment.
- Removed horizontal padding from the composer action row.
- Removed horizontal padding from the OpenRouter model-popover back row.
- Left-aligned and vertically centered the Providers popover title.
- Removed `Select source` from the Providers popover.
- Added `Built by C4OS` and a divider above `+ Add Marketplace` in the plugin source menu.
- Removed `Docs` from the custom MCP connection dialog.
- Removed the redundant `Choose runtime` status label from the unselected runtime row while preserving its radio control.
- Increased top padding in the Settings navigation.
- Made every Settings content section explicitly fill and paint the available center surface after visual QA exposed an empty-area compositing defect at the taller review viewport.

### Feedback Applied

- Applied all nine browser annotations supplied after Review Round 1.
- Kept these changes inside r007 because they are minor refinements to the same r04-parity artifact.
- Did not introduce any r05 Batch 1 behavior.

### Verified

- Composer action-row left and right padding both compute to `0px`.
- Send background remains `rgb(95, 95, 95)` before and during hover.
- The OpenRouter back row computes to `0px` left and right padding.
- The Providers header computes to left alignment and vertical centering, contains only `Providers`, and has no subtitle element.
- The plugin source menu opens with `Built by C4OS`, one separator, and `+ Add Marketplace` in that order.
- The custom MCP dialog opens without a `Docs` control.
- Runtime settings contain two radio controls, only the selected runtime retains a `Selected` pill, and `Choose runtime` is absent.
- Settings navigation top padding computes to `20px`.
- Visual QA covered the corrected provider popover and plugin source menu at the annotated review viewport.
- A seven-route affected-surface sweep found no horizontal overflow, missing main regions, removed-text regressions, console warnings, or console errors.

### Review Now

- Whether the Send control now feels correct without hover feedback.
- Whether composer action-row edge alignment matches the intended density.
- Whether both provider/model popover headers now have the intended alignment and spacing.
- Whether the plugin source menu hierarchy is correct.
- Whether Settings top spacing and the simplified MCP/runtime states match the annotations.

### Simulated Or Deferred

- The existing r04 simulation boundaries remain unchanged.
- r05 Batch 1 through Batch 5 additions remain deferred.

### Open Questions

- None blocking this correction round.

### Approval Path

If Review Round 2 is approved, the next step is Review Round 3 in this same r007 revision: apply the accepted r05 Batch 1 global-header and plugin-panel shell architecture to the corrected r04 SPA foundation. If changes are requested, revise this round before introducing r05 behavior.

## Review Round 3 - Exact Container Spacing Corrections

Date: 2026-07-13

### Changed

- Added `10px` right padding to the composer action row while keeping its left padding at `0px`.
- Removed horizontal padding from both the OpenRouter header container and its nested back row.
- Added `12px` top padding directly to the Settings kicker below Back to app.

### Feedback Applied

- Applied the three new browser annotations.
- Corrected the previous selector mistakes: the OpenRouter change now targets `.popover-backbar` as well as `.popover-back`, and the Settings spacing now targets `.settings-nav > .kicker` instead of only moving the entire sidebar.
- Kept the changes in r007 and did not introduce r05 Batch 1 behavior.

### Verified

- At `1588x846`, composer action-row padding computes to `0px` left and `10px` right, with a measured `10px` gap after the Send control.
- At `1588x846`, both the OpenRouter header container and nested back row compute to `0px` left and right padding.
- At `1068x964`, the Settings kicker computes to `12px` top padding and its text begins below Back to app with the requested visible separation.
- Visual QA covered File Explorer composer spacing, the OpenRouter popover, and Settings > Add Provider at the exact annotated viewport sizes.
- Browser console inspection found no warnings or errors.

### Review Now

- Whether the composer now has the intended right breathing room without restoring left padding.
- Whether the OpenRouter header is now truly flush horizontally.
- Whether the Settings label now has sufficient space above it.

### Simulated Or Deferred

- The existing r04 simulation boundaries remain unchanged.
- r05 Batch 1 through Batch 5 additions remain deferred.

### Open Questions

- None blocking this correction round.

### Approval Path

If Review Round 3 is approved, the next step is Review Round 4 in this same r007 revision: apply the accepted r05 Batch 1 global-header and plugin-panel shell architecture. If changes are requested, revise this round before introducing r05 behavior.

## Review Round 4 - r05 Batch 1 Plugin-First Shell

Date: 2026-07-13

### Changed

- Made `#shell-foundation` the default r007 route while retaining the earlier r04 routes as reference states in the same SPA.
- Added one global header with configured left-side and right-side plugin icon groups and no default open side panel.
- Added functional left and right plugin panels for Chats, File System, File Editor, Browser, Terminal, and Chat Debug shell placement.
- Added active-icon closure, inactive-icon opening, same-side replacement, and independent opposite-side coexistence.
- Added per-chat panel state for `Locate Tauri integration` and `Draft wireframes`.
- Added a Settings center route that closes plugin panels and restores the prior chat shell through Back to app.
- Added a 640px center-workspace constraint, panel resize handles, viewport normalization, and opposite-panel collision closure.
- Added closed-plugin activity indication that clears only when the matching panel opens.
- Added a repair state for plugin declarations that attempt to own reserved `panel`, `enabled`, or `iconOrder` shell fields.
- Replaced the workflow starting points with the seven accepted r05 Batch 1 review routes.

### Feedback Applied

- Applied only r05 Batch 1 after the user approved advancing beyond the corrected r04 foundation.
- Kept r05 Batch 2 through Batch 5 deferred so each addition remains independently reviewable.
- Continued using the bundled wireframe library as the grayscale and accessibility foundation without treating its examples as the product ceiling.

### Verified

- `script.js` passes `node --check`.
- All seven Batch 1 routes render through the same `index.html`, with one global header, one `#main`, no review annotations, and no document-level horizontal overflow at `1588x846`.
- Shell Foundation opens with zero plugin panels.
- Same-Side Replacement opens File System and Terminal with a measured `928px` center workspace; selecting Chats replaces File System while Terminal remains open.
- Per-Chat Restore returns from a panel-free `Locate Tauri integration` state to the prior Chats and Browser panels for `Draft wireframes`.
- Settings closes both panels, changes the hash to `#settings`, and Back to app restores the prior Chats and Terminal panels.
- Hidden Activity begins with zero panels and a Browser activity dot; opening Browser clears the dot without affecting another plugin.
- Dragging the left panel into the collision boundary closes the right panel first and leaves a measured `1108px` center workspace.
- Browser diagnostics reported no console warnings or errors.
- Visual QA covered the panel-free Shell Foundation and the two-sided File System/Terminal shell.

### Review Now

- Whether the global header and plugin icon grouping match the accepted r05 shell direction.
- Whether same-side replacement and two-sided coexistence feel correct.
- Whether the panel widths, center workspace, and collision behavior preserve the intended working density.
- Whether per-chat restoration and Settings closure/restoration match the expected mental model.
- Whether hidden activity and invalid-layout repair are restrained but understandable.

### Simulated Or Deferred

- Plugin service fan-out, actual plugin runtime isolation, disk persistence, real worker execution, and production layout validation remain simulated.
- r05 Batch 2 settings, Batch 3 prompt/workspace behavior, Batch 4 file/editor changes, and Batch 5 Browser/Terminal/Chat Debug detail remain deferred.

### Open Questions

- None blocking Batch 1 review.

### Approval Path

If Review Round 4 is approved, the next step is Review Round 5 in this same r007 revision: apply accepted r05 Batch 2 Settings surfaces. If changes are requested, revise Batch 1 before introducing later r05 behavior.

## Review Round 5 - r04 Body Restoration Correction

Date: 2026-07-13

### Changed

- Corrected the Review Round 4 composition boundary: r05 continues to own the global header, plugin placement, panel state, restoration, and collision behavior, while r04 now owns every visible body.
- Kept Chats as the distinct r05 main-shell chat plugin.
- Replaced the simplified File System body with the r04 project search, five local project folders, and two chats nested under `c4os2`.
- Replaced the simplified File Editor body with the r04 folder/file tree and added an in-panel transition to the r04 editor after selecting a file.
- Replaced the custom Browser body with the r04 address bar and rendered-preview body.
- Split the r04 Terminal body correctly: Terminal now contains only the top terminal-output region, and Chat Debug contains only the bottom command-results region.
- Replaced the simplified center messages with the complete r04 thread, activity, approval, disclosure, and composer body.
- Replaced the simplified Settings list with the complete r04 Settings navigation and content surfaces while preserving r05 panel closure and restoration.
- Removed the extra generic plugin-body headings so the restored r04 bodies render without r05-only wrappers.

### Feedback Applied

- Applied the user's correction that File System is local-project navigation, including project search, folders, and chats nested per project.
- Applied the clarification that Chats remains a correct, separate main-shell plugin.
- Applied the instruction that File Editor begins as the r04 file tree and opens the r04 editor when a file is selected.
- Applied the instruction to use the r04 center, Browser, Terminal-top, Debug-bottom, and Settings bodies.

### Verified

- `script.js` passes `node --check`.
- At `1588x846`, File System renders five project rows and two nested chat rows beside the r04 center thread, with Terminal on the opposite side and a measured `928px` center workspace.
- File Editor opens with five r04 file-tree rows; selecting `main.js` replaces them with the r04 breadcrumbs and eight-line editor.
- Browser renders the r04 `http://127.0.0.1:13000` address bar and `Rendered page mock` surface.
- Terminal renders the r04 terminal output without the bottom region.
- Chat Debug renders the r04 `AI command preview/results` region without terminal output.
- Settings opens with zero plugin panels, all seven r04 navigation items, and the full Providers body; Plugins renders nine catalog cards and the source menu.
- Back to app restores the prior File System and Chat Debug panels.
- The corrected shell has one `#main`, no document-level horizontal overflow, and no browser console warnings or errors.
- Visual QA covered the File System/Terminal shell and the complete r04 Plugins Settings body inside the r05 global shell.

### Review Now

- Whether File System now matches the r04 local-project and nested-chat structure.
- Whether Chats remains visibly distinct as the main-shell chat plugin.
- Whether File Editor correctly changes from the file tree to the editor inside one plugin panel.
- Whether Browser, Terminal, and Chat Debug now reproduce the correct r04 body regions.
- Whether the center thread and Settings surfaces now look like r04 while retaining the r05 shell behavior.

### Simulated Or Deferred

- Project creation/removal, file loading and saving, Browser rendering, terminal execution, debug event streaming, and Settings persistence remain simulated.
- r05 Batch 2 additions and all later r05 batches remain deferred; restoring their r04 baseline bodies does not apply those later additions.

### Open Questions

- None blocking this correction review.

### Approval Path

Approval of Review Round 5 confirms the corrected r05 Batch 1 composition boundary. The exact next step is Review Round 6: apply accepted r05 Batch 2 additions over the restored r04 Settings body.

## Review Round 6 - New Chat And Add Project Actions

Date: 2026-07-13

### Changed

- Added a full-width `+ New Chat` action immediately below Search chats in the Chats plugin.
- Made the File System Add Project control functional.
- Routed both actions to the r04 empty-session center state with the `What should we build in c4os2?` heading, `Do anything` composer, and `c4os2` global-header title.
- Preserved the currently open Chats or File System panel while the center changes to the empty-session state.
- Made existing chat selection leave the empty-session state and restore the r04 thread body.
- Updated the chat workflow starting point to include both starting and restoring a chat.

### Feedback Applied

- Applied the annotation to place `+ New Chat` directly below chat search.
- Applied the annotation that Add Project must work and load the solo r04 chat prompt.

### Verified

- `script.js` passes `node --check`.
- At the exact `1068x964` annotation viewport, Chats contains one `+ New Chat` action directly below search.
- Selecting `+ New Chat` preserves the Chats panel, removes the thread, changes the global title to `c4os2`, and renders the r04 empty prompt with `Do anything`.
- File System contains one accessible Add Project control.
- Selecting Add Project preserves the File System panel and renders the same r04 empty prompt state.
- Selecting `Draft wireframes` after either empty-prompt action restores the r04 thread and `Draft wireframes` global title.
- Browser diagnostics reported no console warnings or errors.
- Visual QA covered the new Chats action and its r04 empty-prompt result at `1068x964`.

### Review Now

- Whether `+ New Chat` has the correct placement and visual weight below Search chats.
- Whether both `+ New Chat` and Add Project land in the correct r04 empty-prompt state.
- Whether retaining the initiating plugin panel while the center changes feels correct.

### Simulated Or Deferred

- New chat persistence, folder selection, project creation, and prompt submission remain simulated.
- r05 Batch 2 and later additions remain deferred.

### Open Questions

- None blocking this action correction.

### Approval Path

Approval of Review Round 6 confirms these final r05 Batch 1 action corrections. The exact next step is Review Round 7: apply accepted r05 Batch 2 additions over the restored r04 Settings body.

## Review Round 7 - New Chat Model Popover Shell Fix

Date: 2026-07-13

### Changed

- Replaced the New Chat model chip's r04 route link with a local composer popover trigger.
- Kept the r04 OpenRouter model-list appearance inside the r05 plugin-first shell.
- Added a local Providers back-step with OpenRouter, OpenAI, and LiteLLM Local choices.
- Made model selection update the composer model chip without changing the hash route, closing the Chats panel, or replacing the r05 shell.
- Kept the historical r04 model/provider routes available only as reference routes outside this New Chat flow.

### Feedback Applied

- Applied the reported regression that opening the model popover from New Chat returned the application to the r04 shell layout.

### Verified

- `script.js` passes `node --check`.
- New Chat contains one model button and zero links to `#models-popover`.
- Opening the model picker keeps `#per-chat-restore`, the `per-chat-restore` SPA state, the r04 empty prompt, and the Chats plugin panel unchanged.
- The local picker displays Gemini 2.5 Flash, ChatGPT o4, and Grok 2.0 across the full `348px` inner popover width.
- The Providers back-step shows all three r04 provider choices and returns to the model list without route navigation.
- Selecting ChatGPT o4 updates the chip label while the r05 shell and empty prompt remain mounted.
- No document-level horizontal overflow or browser console warnings/errors were found.

### Review Now

- Whether the model picker now opens in the correct r05 shell context.
- Whether the local Providers back-step and selected-model update behave as expected.
- Whether any other New Chat composer control still escapes to an r04 reference route.

### Simulated Or Deferred

- Provider-specific model fetching and persisted model selection remain simulated.
- r05 Batch 2 and later additions remain deferred.

### Open Questions

- None blocking this regression fix.

### Approval Path

Approval of Review Round 7 confirms the New Chat model flow remains inside the r05 shell. The exact next step is Review Round 8: apply accepted r05 Batch 2 additions over the restored r04 Settings body.

## Review Round 8 - Batch 2 Configuration Only

Date: 2026-07-13

### Changed

- Split Batch 2 into independently reviewable parts and applied only Configuration.
- Promoted `#settings-configuration` to the canonical direct-review route inside the r05 shell.
- Replaced the r04 Configuration cards inside the r05 Settings route with four accepted registered server-tool policy rows: `terminal.run`, `files.read`, `git.worktree`, and `credentials.use`.
- Added default and maximum policy summaries to every row.
- Added functional edit actions with inline Default policy and Maximum policy selectors plus Save and Cancel.
- Added functional revoke actions that keep the registered tool visible and mark its remembered rule revoked.
- Added the accepted `config.toml` parse-error state and last-valid fallback with loaded timestamp, runtime, tool-policy summary, and blocked-save explanation.
- Kept the complete r04 Settings navigation and r05 global shell behavior.
- Left Plugins and Skills on their restored r04 bodies without applying their Batch 2 additions.

### Feedback Applied

- Applied the user's request to perform Batch 2 by parts and update only Configuration now.
- Preserved the accepted r05 correction that normal policy rows show policy summaries and icon-only edit/revoke actions without persistent remembered/session-rule prose.

### Verified

- `script.js` passes `node --check`.
- At `1068x964`, Configuration renders four policy rows with the expected server-tool identifiers and no plugin panels while Settings is active.
- Direct navigation to `#settings-configuration` opens this Configuration part without falling back to the historical r04 shell.
- Editing `terminal.run`, choosing Default Allow and Max Deny, and saving updates the visible row pills and closes the editor.
- Revoking `files.read` preserves the row and adds `Rule revoked` state.
- Parse-error state renders `config.toml parse error`, three last-valid fields, and the blocked-save explanation; Return to policies restores all four rows.
- Plugins remains the restored r04 nine-card marketplace body with no Batch 2 Configuration nodes.
- Skills remains the restored r04 seven-row list with no Batch 2 Configuration nodes.
- No document-level horizontal overflow or browser console warnings/errors were found.
- Visual QA covered the normal Configuration policy view at the exact annotation viewport.

### Review Now

- Whether the four policy rows have the right density and summaries inside the r04 Settings layout.
- Whether edit/save and revoke communicate the remembered-rule controls clearly enough.
- Whether the parse-error and last-valid fallback state explains the config source boundary correctly.
- Whether Plugins and Skills correctly remain unchanged for this part.

### Simulated Or Deferred

- Policy persistence, config.toml writes, parser recovery, and native external-file opening remain simulated.
- Batch 2 Plugins and Skills remain deferred, along with all later batches.

### Open Questions

- None blocking Configuration review.

### Approval Path

Approval of Review Round 8 confirms the Batch 2 Configuration part only. The exact next step is Review Round 9: apply the accepted Batch 2 Plugins additions while leaving Skills deferred.

## Review Round 9 - Batch 2 Plugins Only

Date: 2026-07-14

### Changed

- Replaced the r04 marketplace placeholder catalog with the five actual C4OS app plugins: File System, File Editor, Browser, Terminal, and Chat Debug.
- Kept Chats out of Settings > Plugins because Chats is part of the main shell rather than an app plugin.
- Added an Advanced settings entry for every plugin with the accepted field shapes: select, text, textarea, switch, number, and sensitive password.
- Added visible dependency health for each plugin, including Browser repair and Chat Debug dependency-blocked examples.
- Added functional Repair and Check again actions that return the simulated plugin and dependency state to Ready.
- Added two uninstall choices: remove the plugin while retaining its data, or also delete plugin-owned settings, cache, and secure-secret references.
- Redacted the sensitive credential value and explained that raw secret use remains behind governed C4OS calls.
- Added search filtering and made `#settings-plugins` a direct review route inside the r05 shell.
- Left Skills and later Batch 2 work unchanged.

### Feedback Applied

- Applied the approved Plugins-only Batch 2 scope.
- Applied the correction that Chats is part of the shell and must not appear in the plugin inventory.
- Applied the request to show advanced settings, dependency and repair states, uninstall choices, sensitive-value redaction, and the real C4OS plugin list.

### Verified

- `script.js` passes `node --check`.
- `git diff --check` passes.
- The rendered plugin inventory is sourced only from the five C4OS plugin records; the previous marketplace products are no longer part of the rendered list.
- Repair, dependency recheck, settings save, uninstall choice, uninstall confirmation, and search-filter controls have local SPA handlers.
- Direct `#settings-plugins` routing stays in the r05 plugin-first shell and selects Plugins in the r04 Settings navigation.
- The local preview server responds on port `4173`.
- Automated in-app browser navigation could not complete because its existing connection-error page was blocked from returning to localhost by browser URL policy; visual QA remains for human review.

### Review Now

- Whether the five-plugin list has the right density and hierarchy inside the r04 Settings layout.
- Whether the advanced settings form reads as one plugin-owned schema without looking like a marketplace connection form.
- Whether dependency-blocked and repairable states are distinct and actionable enough.
- Whether the two uninstall choices make data retention consequences clear.
- Whether the redacted sensitive field and secure-use explanation are appropriately explicit.

### Simulated Or Deferred

- Plugin settings persistence, dependency resolution, native repair, secure storage, uninstall execution, and plugin-data deletion remain simulated.
- Skills and all later Batch 2 additions remain deferred.

### Open Questions

- None blocking this Plugins review.

### Approval Path

Approval of Review Round 9 confirms the Batch 2 Plugins part only. The exact next step is to update the next Batch 2 part the user selects; Skills remains unchanged until explicitly requested.

## Review Round 10 - Plugin Hierarchy Correction

Date: 2026-07-14

### Changed

- Restored the accepted two-column r04 plugin list layout instead of presenting Batch 2 status rows in the list.
- Restored the Built by C4OS source control and its existing marketplace menu.
- Restored the plugin overview dialog as the first surface opened from a plugin card.
- Kept Advanced settings as a link at the bottom of that overview dialog.
- Moved all Batch 2 work behind the Advanced settings link: dependencies, repair and blocked states, form fields, sensitive-value redaction, and uninstall entry.
- Kept uninstall data-retention choices one level below Advanced settings.
- Added Back transitions from uninstall to Advanced settings and from Advanced settings to the plugin overview.

### Feedback Applied

- Applied the correction that the accepted plugin list layout and overview dialog must remain intact.
- Applied the clarification that the existing Advanced settings link, not the plugin list, owns the new Batch 2 work.

### Verified

- `script.js` passes `node --check`.
- `git diff --check` passes.
- In-app browser review confirms the list renders all five C4OS plugins in the restored two-column layout.
- Opening Browser first renders the restored overview dialog with its Advanced settings link.
- Selecting Advanced settings renders Browser dependency and repair state, the settings form, and the redacted sensitive credential.
- Selecting Uninstall renders both Keep plugin data and Delete plugin data choices.
- Back transitions restore Advanced settings and then the overview dialog.
- The reviewed viewport has no document-level horizontal overflow.

### Review Now

- Whether the plugin list and overview modal now match the previously accepted structure.
- Whether Advanced settings is the correct and only entry into dependency, repair, redaction, and uninstall controls.
- Whether the Back hierarchy between uninstall, advanced settings, and overview feels correct.

### Simulated Or Deferred

- Plugin opening, settings persistence, dependency resolution, native repair, secure storage, and uninstall execution remain simulated.
- Skills and all later Batch 2 additions remain deferred.

### Open Questions

- None blocking this correction review.

### Approval Path

Approval of Review Round 10 replaces the rejected Review Round 9 hierarchy while retaining its approved plugin inventory and advanced-state scope. The exact next step is the next Batch 2 part the user explicitly selects.

## Review Round 11 - Advanced Modal Toggle Placement

Date: 2026-07-14

### Changed

- Removed the Enabled status pill from the Advanced settings modal header.
- Removed the labeled Enable plugin field and switch from the form body.
- Moved the plugin enable switch into the top-right modal controls beside Close.
- Preserved the accessible plugin-specific enabled label on the relocated switch without rendering visible label copy.

### Feedback Applied

- Applied Browser Comment 1 to remove the Enabled pill.
- Applied Browser Comment 2 to remove the Enable plugin form label.
- Applied Browser Comment 3 to move the switch beside the top-right Close control.

### Verified

- `script.js` passes `node --check`.
- `git diff --check` passes.
- In-app browser inspection finds zero header status pills, zero Enable plugin form labels, and exactly one top-right plugin switch.
- The File System Advanced settings modal retains its dependency state, remaining form fields, sensitive-value redaction, and footer actions.
- Browser diagnostics show no warnings or errors and no document-level horizontal overflow.

### Review Now

- Whether the top-right switch has the correct spacing and visual relationship to Close.
- Whether removing the duplicate Enabled and Enable plugin labels makes the modal hierarchy sufficiently clear.

### Simulated Or Deferred

- Enablement persistence and all plugin lifecycle actions remain simulated.
- Skills and later Batch 2 additions remain deferred.

### Open Questions

- None blocking this correction review.

### Approval Path

Approval of Review Round 11 confirms the Advanced settings toggle placement. The exact next step remains the next Batch 2 part the user explicitly selects.

## Review Round 12 - Batch 2 Closeout

Date: 2026-07-14

### Decision

- Accepted Review Round 11 as the final Batch 2 Plugins state.
- Closed Batch 2 completely with Configuration and Plugins as its accepted additions.
- Rejected the remaining proposed Batch 2 additions as mistaken scope; they are not deferred work and must not be carried into a later round.
- Kept the existing r04 Skills body unchanged as part of the inherited baseline rather than as unfinished Batch 2 work.

### Verified

- Updated the current r007 specification boundary to match this closeout decision.
- No product UI changed in this closeout entry.

### Approval Path

Batch 2 is closed. The next authorized work is the accepted r05 Batch 3 prompt and workspace behavior applied over the approved r007 shell and r04 chat-session body.

## Review Round 13 - Batch 3 Prompt And Workspace Behavior

Date: 2026-07-14

### Changed

- Applied the complete accepted r05 Batch 3 set as seven hash-addressable routes in the existing r007 SPA.
- Kept Chats in the main shell and retained the r04 chat-session body shape for the center workspace.
- Added prompt reference resolution for `$` Skills, `@` resources, and `/` commands with active-query and resolved-token states.
- Added explicit approval, remembered approval, dependency-blocked suggestion, Git branch, structured attachment, and safe fallback states.
- Kept Browser and Chat Debug as plugin surfaces on the accepted routes while preserving the Batch 1 center-minimum collision behavior.
- Added Batch 3 starting points to `workflows.html`.

### Batch 2 Boundary Preserved

- Batch 2 remains complete and closed.
- No rejected Batch 2 Skills or other proposed states were revived or reclassified as deferred work.

### Verified

- All seven Batch 3 routes render inside the single `index.html` document.
- Route titles, left Chats ownership, route-specific state bodies, and narrow-viewport collision behavior match the accepted r05 route map.
- Approval Advanced expands to four decision-context rows.
- Browser diagnostics report no warnings or errors.
- No document-level horizontal overflow appears across the seven routes at the active review viewport.
- `script.js` passes `node --check` and `git diff --check` passes.

### Review Now

- Whether prompt suggestions feel native to the composer rather than like a separate settings surface.
- Whether the approval and remembered-rule hierarchy is sufficiently clear inside the r04-style chat body.
- Whether blocked repair, branch choice, structured attachments, and the safe fallback match the accepted r05 behavior without changing shell ownership.

### Simulated Or Deferred

- Prompt execution, persisted approval decisions, dependency repair, Git creation, attachment transport, and adapter serialization remain simulated wireframe interactions.
- Batch 4 file/editor changes and Batch 5 Browser/Terminal/Chat Debug detail remain deferred.

### Approval Path

Approval of Review Round 13 closes Batch 3 and makes Batch 4 the next available revision boundary.

## Review Round 14 - r05 Prompt Resolver Parity

Date: 2026-07-14

### Changed

- Replaced the simplified r007 prompt suggestion handler with the final accepted r05 caret-scoped resolver behavior.
- The `$`, `@`, and `/` menus now open from the unresolved token containing the caret rather than from the last trigger in the prompt.
- Added Arrow Up, Arrow Down, and Enter selection; pointer selection; active-row ARIA state; focus-preserving caret restoration; and whitespace-boundary closure.
- Added input normalization so multiple recognized references remain blue, edited references return to unresolved text, and backspacing to a pending trigger immediately reruns resolution.
- Restored the complete r05 suggestion catalog metadata used by the review route.
- Removed the r007-only behavior that left the suggestion menu open before the composer had an active caret token.

### Feedback Applied

- Applied the user's direction that `r05-final-implementation/#prompt-suggestions` is the correct functionality.
- Ported the final r05 behavior without reinterpreting or simplifying its interaction model.

### Review Now

- Batch 3 is approved and ready to commit; this correction is a parity fix inside that approved boundary.

### Simulated Or Deferred

- Suggestion catalogs and reference serialization remain wireframe simulations; production resolution remains backend-authoritative.
- Batch 4 File System and File Editor behavior remains the next unstarted batch.

### Approval Path

This round closes Batch 3. The next revision activity is Batch 4 File System and File Editor behavior.

## Review Round 15 - Batch 4 Workspace And Files

Date: 2026-07-14

### Changed

- Applied the final corrected r05 Batch 4 workspace and files set as thirteen SPA routes.
- Preserved the accepted ownership boundary: File System owns folder-backed workspaces, local projects, and project chats; File Editor owns the folder/file tree and editor.
- Added workspace start, loaded, missing-project recovery, center search takeover, and non-Git folder states.
- Added File Editor left/right placement, r04-density explorer, click-to-editor navigation, context menu, create/delete-to-trash state, dirty save/revert state, external-change conflict, and empty/non-code state.
- Applied the later r05 corrections that keep the center r04 prompt visible while File Editor is open and keep the normal editor limited to compact breadcrumbs plus code.
- Kept Save/Revert controls out of the normal editor and limited them to dirty/conflict routes.

### Feedback Applied

- Kept File System distinct from File Editor, following the user's earlier correction that project folders/chats and the folder/file tree are different product surfaces.
- Used the final r05 Review Rounds 29-31 corrections instead of the earlier Batch 4 explanatory drafts.

### Verified

- All thirteen Batch 4 routes render inside the single `index.html` document.
- File System routes show the correct project/chat navigation or workspace-specific state.
- File Editor renders on the configured side and its header icon remains active even when mounted left.
- Clicking `main.js` from the explorer routes to the minimal editor with eight code lines and no default Save/Revert toolbar.
- Project action menus expose Reveal, Copy path, Rename, and Remove; missing projects expose Relocate.
- Dirty, conflict, context-menu, operations, and empty states render without document-level horizontal overflow.
- Browser diagnostics report no warnings or errors.
- `script.js` passes `node --check` and `git diff --check` passes.

### Review Now

- Whether File System now clearly owns workspaces/projects/chats without absorbing File Editor's tree.
- Whether the File Editor explorer and normal editor are close enough to r04 in density and padding.
- Whether the missing-project, context menu, guarded Trash, dirty, conflict, and empty states represent the accepted r05 additions without changing the main shell.

### Simulated Or Deferred

- Native folder pickers, workspace persistence, file-system operations, Trash, editor writes, external-change detection, and Add to chat insertion remain simulated.
- Batch 5 Browser, Terminal, and Chat Debug detail remains deferred.

### Approval Path

Approval of Review Round 15 closes Batch 4 and makes Batch 5 the next available revision boundary.
