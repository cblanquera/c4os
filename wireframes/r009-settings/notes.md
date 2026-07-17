# R009 Settings Review Notes

## Review Round 1 — 2026-07-17 — Settings foundation

### Changed

- Copied forward the complete r008 workspace behavior and visual language.
- Added a separate desktop Settings window with persistent navigation and direct hashes for Providers, Models, Plugins, Skills, MCP Servers, and Configuration.
- Added provider connection presets for OpenRouter, Hugging Face, LiteLLM, and OpenAI Compatible endpoints.
- Added active-provider-derived model groups, model search, visibility controls, default selection, and refresh feedback.
- Added Installed/Directory plugin management with capability details and simulated installation.
- Added Installed/Catalog skill management with search, scope filtering, a secondary item rail, details, enablement, and simulated installation.
- Added MCP global settings, connected/stopped/error rows, and a transport-aware Add Server dialog.
- Added structured runtime/environment controls and a raw configuration editor with dirty, valid, and parse-error states.
- Expanded `workflows.html` with direct settings review entry points while preserving the existing workspace workflows.

### Feedback Applied

- Applied the user’s requirement that Settings is entered from the native OS application menu rather than from persistent main-app navigation.
- Applied the requested top-level categories without merging Plugins, Skills, or MCP Servers into one generic Extensions page.
- Applied research patterns from OpenCode, Jan, OpenChamber, and Codex while retaining r008 as the visual source of truth.

### Review Focus

- Whether the six-item settings taxonomy matches the intended product model.
- Whether the separate desktop window feels appropriate for an OS-menu Settings destination.
- Whether Providers and Models are separated clearly while still communicating that models derive from active providers.
- Whether Plugins and Skills need their current catalog views in this revision or should be reduced to installed-resource management.
- Whether Configuration should remain one destination with structured and raw controls.

### Browser Verification

- Verified the workflow launcher and all six destination hashes in Chromium.
- Verified provider dialog preset switching, required-field validation, and Escape closure.
- Verified model grouping, plugin installation into the Installed view, Skills split-detail rendering, MCP transport-aware fields, and raw configuration parse-error feedback.
- Verified Providers at 1440 × 960 and 720 × 900; the narrow desktop layout collapses navigation labels without horizontal document overflow.
- Captured Providers wide/narrow and Skills wide screenshots under `qa/`; detailed evidence is recorded in `qa/notes.md`.

### Simulated Or Deferred Behavior

- Native macOS/Windows application-menu invocation and secondary-window lifecycle are simulated through direct browser entry.
- Provider tests, model discovery, installation, MCP process connections, credential storage, and configuration writes are in-memory wireframe behavior.
- Plugin capability counts, skill contents, provider models, and MCP tools are illustrative product-shaped data.

### Open Questions

- Should Models expose per-model advanced parameters in a detail panel, or remain visibility/default management only?
- Should Plugins and Skills use the current top-level destinations or sit beneath an Extensions group label in the navigation?
- Should raw Configuration stay on the same page as runtime defaults or become a separate Advanced destination?

### Approval Path

- Approval of Review Round 1 unlocks a focused refinement round for any requested taxonomy, density, state, and interaction changes inside `wireframes/r009-settings/`.
- Wireframe phase approval remains pending until the complete requested settings scope and browser-visible behavior are accepted.

## Review Round 2 — 2026-07-17 — Peg-aligned settings navigation

### Changed

- Removed the window title bar selected in the browser review.
- Rebuilt the sidebar around the supplied peg: Back to C4OS, spacing, Settings label, Providers, Models, Runtimes, Configuration, divider, Plugins, Skills, and MCP Servers.
- Removed the sidebar search, destination descriptions, and version footer so the menu reads as one clear application-level settings index.
- Added a functional Runtimes destination with local, container, and remote execution states.

### Feedback Applied

- Mirrored the user-supplied information hierarchy and exact menu order.
- Used a white rounded selected row on the light gray navigation surface.
- Replaced the top-right close control with the explicit Back to C4OS action requested in the sidebar.

### Review Focus

- Whether the new sidebar proportions and selected state feel close enough to the peg while remaining consistent with C4OS.
- Whether Runtimes belongs between Models and Configuration as shown.
- Whether the first divider is sufficient or the lower extension group also needs a label.

### Browser Verification

- Verified the exact seven-item menu order, Back to C4OS target, 260px desktop sidebar, selected-row treatment, and absence of the removed title bar in Chromium.
- Verified every destination by direct hash and confirmed each active heading and selected menu item match.
- Verified the Runtimes Configure action produces feedback and the new destination renders its execution states.
- Verified Providers at 1170 × 919 and 720 × 900 without horizontal document overflow.
- Captured Providers wide/narrow and Runtimes wide screenshots under `qa/`; detailed evidence is recorded in `qa/notes.md`.

### Simulated Or Deferred Behavior

- Runtime detection, Docker setup, and Remote SSH connection remain in-memory wireframe behavior.
- Native OS application-menu invocation remains outside the static browser artifact.

### Open Questions

- None blocking; this round follows the supplied menu peg directly.

### Approval Path

- Approval of Review Round 2 locks the settings navigation hierarchy and leaves destination-detail refinements for subsequent focused rounds.

## Review Round 3 — 2026-07-17 — Compact C4OS menu scale

### Changed

- Reduced the sidebar typography, icon size, row height, and spacing to match the compact interface scale established by r008.
- Reduced the navigation column from 260px to 224px so it aligns with the original Settings proportion and the rest of the wireframes.
- Preserved the Round 2 menu order, Back to C4OS action, grouping divider, and selected-row treatment.

### Feedback Applied

- Applied the user’s feedback that the Round 2 menu icons and fonts were oversized.
- Used local r008 CSS as the design source of truth instead of the larger type scale from the external peg.

### Review Focus

- Whether the menu now feels native to the existing C4OS wireframes.
- Whether the compact row density remains readable without returning to the subtitle-heavy Round 1 navigation.

### Browser Verification

- Confirmed the compact CSS contract in source: 224px sidebar, 13px Back/menu labels, 10px section label, 17–18px icons, and 40px menu rows.
- JavaScript syntax and whitespace checks pass.
- Fresh in-app browser verification could not be performed because automated access to the local `file://` preview was blocked by the browser security policy; manual refresh remains the visual review path for this round.

### Simulated Or Deferred Behavior

- No behavior changes are introduced in this visual refinement round.

### Open Questions

- None blocking; this round only normalizes the left-menu scale.

### Approval Path

- Approval of Review Round 3 locks the settings navigation hierarchy and compact visual scale.

## Review Round 4 — 2026-07-17 — Provider connection profiles

### Changed

- Reframed Providers as uniquely labeled saved connection profiles with Edit and enable controls.
- Replaced the preset-button form with a Provider Type selector and type-dependent fields.
- Added preset flows for OpenRouter, Hugging Face, and OpenAI with implicit standard endpoints.
- Added an OpenAI Compatible flow with API Base URL, API Key, Auth, optional API-key header name, and JSON Headers.
- Treated LiteLLM as an OpenAI-compatible server profile rather than a separate provider preset.
- Reused the same form for Add and Edit, including preserved-key behavior during edits.

### Feedback Applied

- Applied the supplied provider pegs as workflow and field references only; the existing C4OS scale, buttons, borders, spacing, and grayscale visual system remain authoritative.
- Applied the user’s clarification that LiteLLM may be installed on any server and therefore needs the OpenAI-compatible fields.
- Applied the accepted authentication options: Bearer token, API key header, and None.

### Review Focus

- Whether the saved-profile list communicates provider type, unique label, endpoint, credential status, and enabled state clearly.
- Whether each Provider Type exposes the correct minimum fields.
- Whether LiteLLM is understandable as a labeled OpenAI-compatible profile without its own preset.

### Browser Verification

- Confirmed the saved-profile list, four Provider Type options, three authentication options, conditional field hooks, unique-label validation, JSON-header validation, and Add/Edit state paths in source.
- JavaScript syntax and whitespace checks pass.
- Fresh automated in-app browser verification remains unavailable for this local `file://` preview because of browser security policy; manual refresh is the visual review path for this round.

### Simulated Or Deferred Behavior

- Connection tests, credential persistence, uniqueness checks, model discovery, and server communication remain simulated in memory.

### Open Questions

- None blocking; the accepted provider and authentication model is represented directly.

### Approval Path

- Approval of Review Round 4 locks the Providers list and Add/Edit form model for this wireframe phase.

## Review Round 5 — 2026-07-17 — Provider list simplification and interaction repair

### Changed

- Removed provider badges from every saved-profile row.
- Replaced endpoint and credential-state metadata with the provider type only.
- Removed the Saved profiles heading and its supporting sentence.
- Removed the AI Runtime eyebrow from Providers and Models.
- Added the shared dialog-open hook to Add Provider and every Edit action, including dynamically saved rows.
- Versioned the Settings script import so local-file refreshes load the current interaction code instead of a stale cached copy.

### Feedback Applied

- Applied all six browser annotations from the provider review.
- Kept profile labels, provider icons, Edit actions, enable switches, and the accepted dynamic Add/Edit form model.

### Review Focus

- Whether the simplified rows now contain the correct minimum information.
- Whether Add Provider and all Edit actions open the appropriate form after refreshing the local page.

### Browser Verification

- Confirmed in source that all annotated copy and badges are removed, every initial Add/Edit control owns both the shared dialog hook and provider-specific hook, and the Settings CSS/JS imports carry the Round 5 cache version.
- JavaScript syntax and whitespace checks pass.
- Automated inspection of the existing local `file://` tab was blocked by browser security policy; refreshing that tab is required for final interaction confirmation.

### Simulated Or Deferred Behavior

- Provider connection tests and persistence remain simulated in memory.

### Open Questions

- None blocking; this round follows the annotations directly.

### Approval Path

- Approval of Review Round 5 locks the provider list density and Add/Edit entry behavior.

## Review Round 6 — 2026-07-17 — Compatible-field selector repair

### Changed

- Scoped every conditional provider-field lookup to the Add/Edit Provider form.
- Corrected the selector collision that caused the LiteLLM profile row to be mistaken for the Auth and Headers form fields.
- Versioned the Settings assets so the corrected field behavior loads after refreshing the local page.

### Feedback Applied

- Applied the browser annotation that Auth and Headers were missing when OpenAI Compatible was selected.

### Review Focus

- Confirm that selecting OpenAI Compatible shows API Base URL, Auth, API Key, and Headers.
- Confirm that selecting API key header additionally shows API key header name.
- Confirm that selecting None hides the API Key field.

### Browser Verification

- Confirmed the selector collision and corrected all conditional-field selectors in source.
- JavaScript syntax and whitespace checks pass.
- Automated inspection of the local `file://` tab remains blocked; refreshing the tab is required for visual interaction confirmation.

### Simulated Or Deferred Behavior

- Connection tests and persistence remain simulated in memory.

### Open Questions

- None blocking; this is a targeted behavior repair.

### Approval Path

- Approval of Review Round 6 locks the OpenAI-compatible conditional field behavior.

## Review Round 7 — 2026-07-17 — Filterable model availability

### Changed

- Replaced provider-grouped model sections with one aligned resource list.
- Added a provider-profile filter beside model search.
- Set every seeded model to enabled by default and retained individual switches.
- Added Disable results to bulk-disable only the currently visible search/filter results.
- Added the Bulk enable results state when every visible result is disabled.
- Retained Refresh Models in the page header.

### Feedback Applied

- Applied the supplied Models peg as a workflow reference while retaining C4OS typography, buttons, switches, borders, spacing, and grayscale colors.
- Applied the confirmed mixed-state rule: if any visible result is enabled, the bulk action remains Disable results.

### Review Focus

- Whether search, provider filter, and bulk action align as one compact toolbar.
- Whether the flattened list communicates model identity, provider profile, source, and availability clearly.
- Whether bulk actions affect only visible results and leave filtered-out models unchanged.

### Browser Verification

- Confirmed seven seeded models start enabled, the toolbar contains search/provider/bulk controls, filtering composes search with provider selection, and bulk state derives only from visible rows.
- Confirmed the `opus` scenario in source: filtering to the Opus row and disabling results changes that visible set to disabled and changes the action to Bulk enable results; clearing the search leaves the other rows enabled.
- JavaScript syntax and whitespace checks pass.
- Automated inspection of the local `file://` tab remains blocked; refreshing the Models tab is required for visual interaction confirmation.

### Simulated Or Deferred Behavior

- Model discovery, persistence, and provider refresh remain simulated in memory.

### Open Questions

- None blocking; the accepted visible-result behavior is represented directly.

### Approval Path

- Approval of Review Round 7 locks the Models list, filtering, and bulk availability behavior.

## Review Round 8 — 2026-07-17 — Model bulk-action copy

### Changed

- Changed the all-visible-results-disabled action from Bulk enable results to Enable results.
- Preserved the existing visible-result scope and all enable/disable behavior.
- Versioned the Settings script import for the copy update.

### Feedback Applied

- Applied the browser annotation exactly: `Enable results`.

### Review Focus

- Confirm the bulk action alternates between Disable results and Enable results.

### Browser Verification

- Confirmed the dynamic label and current specification use Enable results.
- JavaScript syntax and whitespace checks pass.
- Automated inspection of the local `file://` tab remains blocked; refreshing the Models tab is required for visual confirmation.

### Simulated Or Deferred Behavior

- No behavior changes were introduced.

### Open Questions

- None.

### Approval Path

- Approval of Review Round 8 locks the Models bulk-action language.

## Review Round 9 — 2026-07-17 — Runtime selection and save

### Changed

- Replaced Local Desktop, Docker, and Remote SSH execution-environment controls with OpenCode and Pi runtime choices.
- Added compact radio-style selection rows with runtime name, project URL, and Selected state.
- Added Save Runtime in the page header.
- Save Runtime remains disabled until the draft selection differs from the saved runtime, then disables again after saving.

### Feedback Applied

- Applied the supplied runtime peg as a structure reference while retaining C4OS typography, buttons, borders, spacing, and grayscale colors.
- Applied the user’s requirement for an explicit save step when changing runtime.

### Review Focus

- Whether the two runtime rows are aligned and clearly mutually exclusive.
- Whether the difference between selecting a draft runtime and saving it is understandable.
- Whether Save Runtime belongs in the upper-right page header.

### Browser Verification

- Confirmed OpenCode is seeded as the saved and selected runtime, Pi is unselected, and Save Runtime is initially disabled.
- Confirmed choosing Pi moves the Selected state and enables Save Runtime; returning to OpenCode before saving disables it again.
- Confirmed saving promotes the draft choice to the saved runtime, disables Save Runtime, and emits a confirmation notice.
- Settings assets use the Round 9 cache version; JavaScript syntax and whitespace checks pass.
- Automated inspection of the local `file://` tab remains blocked; refresh the Runtimes tab for visual and interaction confirmation.

### Simulated Or Deferred Behavior

- Runtime persistence and workspace execution remain simulated in memory.

### Open Questions

- None blocking; OpenCode is the seeded saved runtime and Pi is the alternate choice.

### Approval Path

- Approval of Review Round 9 locks the Runtimes selection and save workflow.

## Review Round 10 — 2026-07-17 — Runtime radio alignment

### Changed

- Assigned every runtime radio to the same fixed right-hand grid column.
- Kept the Selected badge in its own adjacent column so showing or hiding it no longer shifts the radio.
- Advanced the settings asset cache version for a clean browser refresh.

### Feedback Applied

- Applied both browser annotations requesting that the runtime radios align flush on the right.

### Review Focus

- Confirm the OpenCode and Pi radios share the same right edge in both selection states.

### Browser Verification

- Source inspection confirms the Selected badge occupies column 2 and every radio occupies column 3.
- Settings assets use the Round 10 cache version; JavaScript syntax and whitespace checks pass.
- Automated inspection of the local `file://` tab remains blocked; refresh the Runtimes tab for visual confirmation.

### Simulated Or Deferred Behavior

- Runtime persistence and workspace execution remain simulated in memory.

### Open Questions

- None.

### Approval Path

- Approval of Review Round 10 locks runtime-row alignment and completes the current Runtimes review.

## Review Round 11 — 2026-07-17 — Configuration simplification

### Changed

- Renamed Approval policy to Default Approval Policy.
- Added Browser Environment below the login-shell control with All browsers, Per project, Per chat session, and None options.
- Replaced Configuration scope and its file selector with Customize Configuration and an Open config.toml externally button.
- Removed the complete Raw configuration editor, including its source header, validity state, textarea, and editor actions.
- Updated the page description to reflect runtime, permission, and environment behavior.

### Feedback Applied

- Applied all six Configuration browser annotations.
- Preserved the existing compact C4OS card, row, select, and button styling.

### Review Focus

- Confirm Browser Environment appears directly below the shell-environment row and its four scopes are clear.
- Confirm Customize Configuration reads as a single external-editor action.
- Confirm the page feels complete without the embedded raw editor.

### Browser Verification

- Source inspection confirms the raw editor and all associated interaction hooks are absent.
- Source inspection confirms the browser-sharing select contains all four requested options.
- Open config.toml externally emits simulated confirmation feedback.
- Settings assets use the Round 11 cache version; JavaScript syntax and whitespace checks pass.
- Automated inspection of the local `file://` tab remains blocked; refresh the Configuration tab for visual confirmation.

### Simulated Or Deferred Behavior

- Opening `config.toml` in an external editor remains simulated through the notifier.
- Configuration values remain in-memory wireframe controls.

### Open Questions

- None.

### Approval Path

- Approval of Review Round 11 locks the Configuration structure and external customization workflow.

## Review Round 12 — 2026-07-17 — Remove default model

### Changed

- Removed the complete Default model row from the Configuration Runtime card.
- Tightened the Configuration page description to approval defaults and environment behavior.
- Advanced the settings asset cache version for a clean browser refresh.

### Feedback Applied

- Applied the browser annotation requesting removal of the Default model control.

### Review Focus

- Confirm the Runtime card now begins with Default Approval Policy and remains balanced with two rows.

### Browser Verification

- Source inspection confirms Default model and its provider/model select are absent from the Configuration page.
- Settings assets use the Round 12 cache version; JavaScript syntax and whitespace checks pass.
- Automated inspection of the local `file://` tab remains blocked; refresh the Configuration tab for visual confirmation.

### Simulated Or Deferred Behavior

- Configuration values remain in-memory wireframe controls.

### Open Questions

- None.

### Approval Path

- Approval of Review Round 12 locks the reduced Runtime card and completes the current Configuration review.

## Review Round 13 — 2026-07-17 — Advanced Policies screen

### Changed

- Added an Advanced link beside Default Approval Policy on Configuration.
- Added `advanced-policies.html` as a dedicated per-tool policy screen inside the existing Settings shell.
- Added nine policy groups containing all 71 supplied authority identities.
- Added a four-option select to every identity: Use default, Always allow, Always ask for permission, and Never allow.
- Added group navigation, cross-group search, empty search results, changed-row state, and Save Policies draft behavior.
- Added a Back to Configuration route and a workflow-launcher entry.

### Feedback Applied

- Applied the Configuration annotation requesting an Advanced link.
- Used the attached permission scope as the content and classification source of truth.
- Preserved the compact C4OS typography, grayscale controls, navigation, spacing, and aligned trailing fields.

### Review Focus

- Confirm the group rail is the right way to navigate 71 policy identities.
- Confirm the policy row density keeps identity, description, and override understandable.
- Confirm the Advanced link and Back to Configuration route make the relationship to Default Approval Policy clear.
- Confirm search, changed-row state, and Save Policies communicate draft versus saved state.

### Browser Verification

- Source inspection confirms all nine supplied groups, 71 identities, and four policy values are present.
- JavaScript syntax and whitespace checks pass.
- Automated inspection of local `file://` pages remains blocked; open the Advanced link from Configuration for visual and interaction confirmation.

### Simulated Or Deferred Behavior

- Policy changes and persistence remain simulated in memory.
- The global Default Approval Policy is not persisted across the two static HTML files.

### Open Questions

- None blocking.

### Approval Path

- Approval of Review Round 13 locks the Advanced Policies information architecture, density, and draft-save workflow.

## Review Round 14 — 2026-07-17 — Advanced navigation cleanup

### Changed

- Stacked the Advanced link directly beneath the Default Approval Policy select.
- Removed the separate Configuration back-link above the Advanced Policies page header.
- Retained the selected Configuration sidebar item as the return route.

### Feedback Applied

- Applied both Advanced Policies browser annotations.

### Review Focus

- Confirm the Advanced link now reads as a subordinate action for the select.
- Confirm the Advanced Policies header has the intended top alignment without the extra back-link.

### Browser Verification

- Source inspection confirms the setting action uses a right-aligned vertical stack and the page-level back-link is absent.
- JavaScript syntax and whitespace checks pass.
- Automated inspection of local `file://` pages remains blocked; refresh both pages for visual confirmation.

### Simulated Or Deferred Behavior

- Policy changes and persistence remain simulated in memory.

### Open Questions

- None.

### Approval Path

- Approval of Review Round 14 locks Advanced Policies entry and header alignment.

## Review Round 15 — 2026-07-17 — Save action alignment

### Changed

- Vertically centered Save Policies against the complete Advanced Policies title block.
- Advanced the page-specific stylesheet cache version for a clean refresh.

### Feedback Applied

- Applied the annotation requesting that Save Policies align with the middle of the Advanced Policies label.

### Review Focus

- Confirm Save Policies now appears inline with the title instead of the Permissions eyebrow.

### Browser Verification

- Source inspection confirms the Advanced Policies page header overrides the shared start alignment with centered cross-axis alignment.
- JavaScript syntax and whitespace checks pass.
- Automated inspection of the local `file://` page remains blocked; refresh Advanced Policies for visual confirmation.

### Simulated Or Deferred Behavior

- Policy changes and persistence remain simulated in memory.

### Open Questions

- None.

### Approval Path

- Approval of Review Round 15 locks the Advanced Policies header action alignment.

## Review Round 16 — 2026-07-17 — Plugin marketplaces and uninstall

### Changed

- Left-aligned the Installed and Directory segmented tabs with the plugin content column.
- Added a compact marketplace chooser to the right of Directory search with Built by C4OS and Add Marketplace actions.
- Added an Add plugin marketplace dialog with Source, Git ref, and Sparse paths fields.
- Removed the Plugin details eyebrow from the plugin detail dialog.
- Added Uninstall to installed-plugin details and kept it hidden for plugins that are not installed.
- Added simulated uninstall behavior that removes the installed card, updates the count, and restores a matching Install action.

### Feedback Applied

- Applied all three plugin browser annotations.
- Used the supplied pegs for marketplace structure and fields while retaining C4OS typography, spacing, controls, grayscale surfaces, and compact dialog sizing.

### Review Focus

- Confirm the tabs and directory toolbar share a clean left edge.
- Confirm the marketplace chooser feels like a secondary directory source control rather than a primary navigation element.
- Confirm the Add Marketplace dialog field density and action placement match the rest of Settings.
- Confirm Uninstall is discoverable without overpowering Done.

### Browser Verification

- Source inspection confirms marketplace menu, dialog validation, installed-only Uninstall state, and uninstall list mutation are wired.
- JavaScript syntax and whitespace checks pass.
- Automated inspection of the local `file://` page remains blocked; refresh Plugins for visual and interaction confirmation.

### Simulated Or Deferred Behavior

- Marketplace registration, plugin installation, and plugin removal remain simulated in memory.
- The added marketplace is acknowledged but is not persisted as another directory source after reload.

### Open Questions

- None blocking.

### Approval Path

- Approval of Review Round 16 locks the Plugins directory toolbar, marketplace flow, and uninstall behavior.

## Review Round 17 — 2026-07-17 — Install from plugin details

### Changed

- Added Install to the plugin details footer for plugins that are not installed.
- Reused the same left-side action slot for Install and Uninstall based on current installed state.
- Wired dialog installation through the existing directory install workflow, including card creation, count updates, switch state, and confirmation feedback.

### Feedback Applied

- Applied the annotation requesting an Install action for uninstalled plugin details.

### Review Focus

- Confirm Install and Uninstall feel like mutually exclusive state-aware actions.
- Confirm Done remains a clear non-mutating close action.

### Browser Verification

- Source inspection confirms uninstalled details show Install, installed details show Uninstall, and only one state-changing action is visible at once.
- JavaScript syntax and whitespace checks pass.
- Automated inspection of the local `file://` page remains blocked; refresh Plugins for interaction confirmation.

### Simulated Or Deferred Behavior

- Plugin installation and removal remain simulated in memory.

### Open Questions

- None.

### Approval Path

- Approval of Review Round 17 locks the state-aware plugin details actions.

## Review Round 18 — 2026-07-17 — Plugin identity and policy links

### Changed

- Added a compact lettermark logo placeholder to the left of every plugin detail title.
- Added Website, Terms, and Privacy Policy links below the capability list for both installed and uninstalled plugin details.
- Added simulated link feedback without navigating away from the review artifact.

### Feedback Applied

- Applied both plugin detail browser annotations.

### Review Focus

- Confirm the logo placeholder and title form a balanced identity header.
- Confirm Website, Terms, and Privacy Policy are discoverable without competing with installation actions.

### Browser Verification

- Source inspection confirms every seeded plugin supplies an identity mark and all detail states render the three requested links.
- JavaScript syntax and whitespace checks pass.
- Automated inspection of the local `file://` page remains blocked; refresh Plugins for visual confirmation.

### Simulated Or Deferred Behavior

- Link destinations are simulated through the notifier because publisher URLs are not part of the supplied plugin data.
- Plugin installation and removal remain simulated in memory.

### Open Questions

- None.

### Approval Path

- Approval of Review Round 18 locks plugin identity and policy-link placement.

## Review Round 19 — 2026-07-17 — Configuration header cleanup

### Changed

- Removed the Advanced eyebrow from the Configuration page header.
- Advanced the Settings asset cache version for a clean browser refresh.

### Feedback Applied

- Applied the Configuration header annotation requesting removal of Advanced.

### Review Focus

- Confirm Configuration now aligns with the simpler title-and-description treatment used by the approved settings pages.

### Browser Verification

- Source inspection confirms the Configuration header begins directly with its title.
- JavaScript syntax and whitespace checks pass.
- Automated inspection of the local `file://` page remains blocked; refresh Configuration for visual confirmation.

### Simulated Or Deferred Behavior

- No behavior changes were introduced.

### Open Questions

- None.

### Approval Path

- Approval of Review Round 19 locks the Configuration header treatment.

## Review Round 20 — 2026-07-17 — Plugins header cleanup

### Changed

- Removed the Extensions eyebrow from the Plugins page header.
- Advanced the Settings asset cache version for a clean browser refresh.

### Feedback Applied

- Applied the Plugins header annotation requesting removal of Extensions.

### Review Focus

- Confirm Plugins now begins directly with its title and supporting description.

### Browser Verification

- Source inspection confirms the Plugins header contains no eyebrow label.
- JavaScript syntax and whitespace checks pass.
- Automated inspection of the local `file://` page remains blocked; refresh Plugins for visual confirmation.

### Simulated Or Deferred Behavior

- No behavior changes were introduced.

### Open Questions

- None.

### Approval Path

- Approval of Review Round 20 locks the Plugins header treatment.

## Review Round 21 — 2026-07-17 — Skills list and details dialog

### Changed

- Replaced the Installed/Catalog split-detail browser with one searchable installed-skill list.
- Added a circular document icon directly beside every skill name and kept all availability switches in one flush trailing column.
- Added a focused skill details dialog with the same circular icon beside the name, synchronized per-project availability, a longer overview, Uninstall, and Try in chat.
- Removed scope labels, catalog controls, related-file metadata, and the redundant `Skill` type label from the skill identity treatment.
- Added empty search results and advanced the Settings asset cache version for a clean browser refresh.

### Feedback Applied

- Applied the supplied list and dialog pegs as structural references while retaining the current C4OS wireframe typography, grayscale controls, spacing, and compact desktop density.
- Applied the explicit request for circular icons beside skill names and no redundant `Skill` label.

### Review Focus

- Confirm the list density and right-aligned switches feel consistent with Models and the rest of Settings.
- Confirm the circular document icon reads clearly beside each skill name without becoming visually dominant.
- Confirm the detail dialog contains the right amount of context and gives Uninstall and Try in chat the correct action hierarchy.

### Browser Verification

- Source inspection confirms the Skills page contains no eyebrow, Installed/Catalog tabs, scope filter, detail rail, or redundant skill-type label.
- JavaScript syntax and whitespace checks pass.
- Automated inspection of the local `file://` page was attempted and blocked by the in-app browser security policy; refresh Skills for visual and interaction confirmation.

### Simulated Or Deferred Behavior

- Per-project availability, uninstall, and Try in chat are simulated in memory and reset on page reload.
- Try in chat reports the intended handoff through the notifier rather than creating a real conversation.

### Open Questions

- None.

### Approval Path

- Approval of Review Round 21 locks the Skills list, icon, and detail-dialog structure.

## Review Round 22 — 2026-07-17 — Skill dialog close alignment

### Changed

- Vertically centered the Skill details close control against the circular icon and title row.
- Kept the alignment rule scoped to the Skill dialog so other dialog headers remain unchanged.
- Advanced the Settings asset cache version for a clean browser refresh.

### Feedback Applied

- Applied the annotation requesting middle alignment for the Skill dialog close control.

### Review Focus

- Confirm the close control now aligns with the visual center of the skill identity row.

### Browser Verification

- Source inspection confirms the alignment rule targets only `#skill-dialog > header`.
- JavaScript syntax and whitespace checks pass.
- Automated inspection of the local `file://` page remains blocked by the in-app browser security policy; refresh the open dialog for visual confirmation.

### Simulated Or Deferred Behavior

- No behavior changes were introduced.

### Open Questions

- None.

### Approval Path

- Approval of Review Round 22 locks the Skill dialog header alignment.

## Review Round 23 — 2026-07-17 — MCP server management

### Changed

- Replaced the permission-heavy MCP page with a compact server list containing only server name, configure control, and aligned availability switch.
- Removed the MCP eyebrow and updated the supporting copy to focus on external tools and data sources.
- Added a transport-aware Connect to a custom MCP dialog with STDIO and Streamable HTTP modes.
- Added repeatable arguments, environment variables, passthrough variables, headers, and environment-backed headers with add/remove controls.
- Added working-directory, URL, and bearer-token environment variable fields in their relevant transport states.
- Added functional add, edit, enable/disable, required-field validation, duplicate-name validation, and empty-list behavior.
- Advanced the Settings asset cache version for a clean browser refresh.

### Feedback Applied

- Applied the supplied MCP list, STDIO form, and Streamable HTTP form pegs as structural references while retaining the current C4OS type scale, grayscale controls, spacing, and dialog treatment.

### Review Focus

- Confirm the simplified server rows provide enough information at the page level.
- Confirm the configure icon and switches form two clean trailing columns.
- Confirm the STDIO and Streamable HTTP forms expose the right fields with comfortable density inside the existing dialog scale.
- Confirm repeated-value add/remove controls are discoverable without visually overpowering primary Save.

### Browser Verification

- Source inspection confirms the old permission card, status-heavy rows, Test Connection, HTTP/SSE select, and retry behavior are removed.
- JavaScript syntax and whitespace checks pass.
- Automated inspection of the local `file://` page remains blocked by the in-app browser security policy; refresh MCP Servers for visual and interaction confirmation.

### Simulated Or Deferred Behavior

- Server add, edit, availability, and repeated-field values are simulated in memory and reset on page reload.
- Learn more reports the intended documentation handoff through the notifier because a final documentation URL is not part of the supplied scope.
- No MCP process is launched and no connection is tested in the wireframe.

### Open Questions

- None.

### Approval Path

- Approval of Review Round 23 locks the MCP list and custom connection dialog structure.

## Review Round 24 — 2026-07-17 — SPA architecture correction

### Changed

- Folded Providers, Models, Runtimes, Configuration, Plugins, Skills, MCP Servers, all settings dialogs, and Advanced Policies into `index.html`.
- Folded the approved settings and policy styles into `styles.css`, scoped under the Settings SPA wrapper so workspace controls retain their existing treatment.
- Folded settings behavior, policy rendering, and top-level hash routing into `script.js`.
- Added linkable `#settings/<destination>` routes, a workspace-header Settings entry for browser review, and a Back to C4OS transition that restores `#chat` without unloading the workspace DOM.
- Updated every Settings workflow entry to target `index.html`.
- Removed the redundant settings-specific and Advanced Policies HTML/CSS/JS files.

### Feedback Applied

- Applied the user correction that this revision must retain the original single-page application architecture and use the existing `index.html`, `styles.css`, and `script.js` files.

### Review Focus

- Confirm the workspace Settings control opens Providers without a document reload.
- Confirm all sidebar destinations and Advanced Policies remain inside `index.html` and Back to C4OS restores the workspace.
- Confirm previously approved settings layouts, forms, dialogs, and interactions did not visually drift during consolidation.

### Browser Verification

- Verified the corrected artifact through `http://127.0.0.1:8765/index.html#settings/providers` in the in-app browser.
- Confirmed workspace-header Settings entry, Models navigation, Configuration, Advanced Policies, Configuration-selected state, Back to C4OS, and workspace restoration without document replacement.
- Confirmed the Providers layout retained its approved spacing, typography, navigation density, row alignment, and control treatment.
- Confirmed the browser console reported no errors or warnings.
- Static structure, syntax, route, and reference checks are recorded in `qa/notes.md`.

### Simulated Or Deferred Behavior

- The native OS application-menu entry remains simulated by the workspace-header Settings control for static browser review.
- Provider calls, model discovery, installs, external-editor launch, MCP connections, and durable persistence remain simulated.

### Open Questions

- None for this architecture correction.

### Approval Path

- Approval closes the r009 Settings SPA correction. Requested changes create another review round in this revision.
