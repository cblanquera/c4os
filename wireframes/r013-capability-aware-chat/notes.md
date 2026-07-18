# R013 Capability-Aware Chat Review Notes

## Review Round 7 — 2026-07-18 — Chat-session search case study

### Changed

- Added `#chat-search` as a deterministic workflow state seeded with the query `project`.
- Replaced the Projects heading and hierarchy with a flat Search results view whenever the query is non-empty.
- Added owning-project context to each matching chat-session result.
- Added an explicit inline clear control that empties the query, restores Projects unchanged, and returns focus to search.
- Added result activation with the query preserved, Escape clearing, and a no-results state.

### Feedback Applied

- Applied the user's request for searched chat sessions to appear in the left pane instead of filtering the Projects hierarchy in place.
- Applied the requirement that Projects remain temporarily replaced until the search input is cleared through an explicit `x` control.

### Review Focus

- Whether the flat result rows provide enough context through session title plus owning project.
- Whether replacing Projects feels clearer than filtering nested projects in place.
- Whether the inline clear control makes the return path to Projects obvious.

### Simulated Or Deferred Behavior

- Search is an in-memory case-insensitive title match over the seeded wireframe sessions; it does not query a persisted index.
- Result activation restores the prototype's seeded transcript snapshot and does not load external data.

### Open Questions

- None required for this review round.

### Verification

- Confirmed `#chat-search` seeds `project`, hides Projects, and shows exactly two flat matches: Design workspace projects in AI Desktop UI and Establish project knowledge base in quotable-ai.
- Confirmed the active session is marked in results and activating the other result updates the original session and center title without clearing the query.
- Confirmed the inline clear control restores project order `desktop-ui`, `quotable`, `legacy`, preserves expanded states `true`, `true`, `false`, clears the query, hides results, and returns focus to search.
- Confirmed Escape clears the query and restores Projects; unmatched text shows `0 results` and `No chat sessions found`.
- Confirmed the left-pane search/results state has no document-level horizontal overflow at the default viewport or 720 × 900.
- Confirmed the checked route reports no console warnings or errors.
- Confirmed `script.js` passes Node syntax validation and the scoped wireframe/KB changes pass `git diff --check`.

### Approval Path

- Approval of Review Round 7 accepts the chat-session search replacement, result, and clear behavior and returns the complete r013 wireframe phase to an implementation-handoff-ready state. Further minor feedback remains in r013; a materially different navigation model creates r014.

## Review Round 6 — 2026-07-18 — Annotated provider and branch refinements

### Changed

- Kept the provider-family label inline to the right of the back chevron in the model popover header.
- Removed `Personal` and `Work` secondary labels from every compact provider row.
- Added a divider and `+ Create New` beneath `codex/artifacts` in the branch menu.

### Feedback Applied

- Applied all three browser comments captured against the model header, provider chooser, and branch menu.
- Preserved the existing provider/model navigation and branch selection behavior while making only the annotated hierarchy changes.

### Review Focus

- Whether the inline back-chevron/provider label now reads as one compact navigation row.
- Whether provider-family-only rows are the right density for this chooser.
- Whether the divider gives `+ Create New` enough separation from selectable existing branches.

### Simulated Or Deferred Behavior

- `+ Create New` is a visible non-mutating wireframe affordance in this round; it does not create or switch a real Git branch.
- Provider/model data and selection remain illustrative in-memory wireframe behavior.

### Open Questions

- None introduced by these annotated refinements.

### Verification

- Confirmed the provider header computes as a two-column grid and the OpenAI label is horizontally inline and vertically centered beside the chevron.
- Confirmed the provider chooser contains exactly OpenRouter, OpenAI, and Hugging Face with zero secondary `small` labels.
- Confirmed the branch menu order is `main`, `codex/artifacts`, separator, and `+ Create New`, with the separator and action exposed through menu semantics.
- Confirmed the revised provider and branch menus remain inside the viewport with no document-level horizontal overflow at the default viewport and 720 × 900.
- Confirmed the checked route reports no console warnings or errors.
- Confirmed `script.js` passes Node syntax validation and the scoped wireframe/KB changes pass `git diff --check`.

### Approval Path

- Approval of Review Round 6 accepts the annotated provider and branch refinements and returns the complete r013 wireframe phase to an implementation-handoff-ready state. Further minor feedback remains in r013; a materially different flow creates r014.

## Review Round 5 — 2026-07-18 — Provider-to-model navigation

### Changed

- Added a provider header above the capability filters in the composer model popover.
- Added an in-place Providers view using the configured OpenRouter, OpenAI, and Hugging Face fixture routes.
- Made each provider choice return to a model list scoped to that provider while preserving the active model until a model is selected.
- Added provider-aware model fixtures and retained the existing capability filtering, attachment preflight, dependent-control recomputation, and selected-state behavior.
- Added Escape dismissal with focus restoration to the model trigger.

### Feedback Applied

- Applied the user's request for a header conceptually like the supplied “OpenRouter” peg and a provider-list state conceptually like the supplied provider peg.
- Treated the supplied screenshots as interaction references only. The initial header uses OpenAI because GPT-5 belongs to the OpenAI fixture, and the layout retains the C4OS wireframe language.

### Review Focus

- Whether the provider header is discoverable as navigation without competing with the model capability filters.
- Whether replacing the popover body with Providers and returning to a provider-scoped model list feels clear and lightweight.
- Whether provider-family labels are sufficient here or should expose full provider-profile labels in a later refinement.

### Simulated Or Deferred Behavior

- Provider availability, provider/model membership, capability data, and model changes remain illustrative in-memory wireframe behavior; no live provider discovery or model call occurs.
- The static grayscale artifact still does not implement the accepted platform-native light/dark contract; this round does not change platform presentation.

### Open Questions

- None required to review this interaction round.

### Verification

- Confirmed the default GPT-5 menu opens on OpenAI and shows only GPT-5 and GPT-5 fast.
- Confirmed the header opens Providers, marks OpenAI as the active model's provider, and shows OpenRouter, OpenAI, and Hugging Face fixture choices.
- Confirmed choosing OpenRouter returns to Claude Opus 4.1 and Kimi K2 while GPT-5 remains active until a model is selected.
- Confirmed selecting Kimi K2 closes the menu, marks Kimi K2 selected, and hides the unsupported Reasoning control.
- Confirmed closing and reopening restores the model view for the selected model's provider.
- Confirmed Escape closes the provider chooser, clears `aria-expanded`, and restores focus to the model trigger.
- Confirmed no document-level horizontal overflow at the default wide viewport or at 720 × 900; the narrow popover remained inside the viewport.
- Confirmed the checked route reported no console warnings or errors.
- Confirmed `script.js` passes Node syntax validation and the scoped wireframe changes pass `git diff --check`.

### Approval Path

- Approval of Review Round 5 accepts provider-to-model navigation and returns the complete r013 wireframe phase to an implementation-handoff-ready state. Requested refinements remain in r013 when minor or create r014 if they materially change the flow.

## Review Round 4 — 2026-07-18 — Context-window provenance

### Changed

- Removed the Adapter row from Chat information.
- Added a Context window row with used and remaining proportions, token usage against the effective model limit, and a compact utilization bar.
- Kept the illustrative limit aligned with the wireframe's GPT-5 model profile rather than copying the supplied peg's capacity literally.

### Feedback Applied

- Applied the browser comment requesting less adapter detail and a user-relevant Context window provision.
- Used the supplied peg only to identify the information types the row should report: used percentage, remaining percentage, used tokens, and total tokens.

### Review Focus

- Whether Context window is the right level of runtime detail for Chat information.
- Whether the percentage, token total, and utilization bar scan clearly without making the popover feel diagnostic-heavy.

### Simulated Or Deferred Behavior

- The 114K / 400K token usage and 29% utilization are illustrative wireframe values; they are not read from a live conversation runtime.

### Open Questions

- None introduced by this refinement round.

### Verification

- Confirmed at 889 × 964 that Chat information contains no Adapter row and exactly one Context window row.
- Confirmed the popover remains within the viewport with no document-level horizontal overflow.
- Confirmed the utilization bar renders at 29% and exposes 114 of 400 through progressbar semantics.
- Confirmed the visible row reports 29% used, 71% left, and 114K / 400K tokens.
- Confirmed the checked Chat route reports no console warnings or errors.
- Confirmed `script.js` passes Node syntax validation and the r013 file set passes `git diff --check`.

### Approval Path

- Approval of Review Round 4 accepts the Chat information content model and returns the complete r013 wireframe phase to an implementation-handoff-ready state.

### Approval

- Approved by the user on 2026-07-18 with “looks good now”; r013 is the accepted forward wireframe baseline and is eligible for KB synchronization and implementation handoff.

### KB Sync

- Promoted r013 as the accepted forward behavioral and visual-interaction baseline while preserving r012 as the historical parity profile.
- Synchronized capability-aware Chat, Models, approvals, policy groups/exceptions, honest activity, provenance, deterministic fixtures, and superseded decisions into the narrow owning Context and Reference Files.
- Classified provider/evidence data, conversions, context usage, service behavior, and persistence as simulated-only; deferred model price/latency signals and an exception detail drawer to a later accepted revision.
- Agent Workspace validation passed with the pre-existing preferred-length warning for `specs/00001-c4os-ai-harness-research/journeys.md`.
- Wide and narrow rendered checks, checked responsive breakpoints, and console state are recorded in Rounds 1–4. The static grayscale wireframe does not implement the accepted platform-native light/dark contract, so macOS, Windows, Linux, and live theme propagation remain unverified production acceptance work rather than r013 claims.

## Review Round 3 — 2026-07-18 — Resize containment and attachment alignment

### Changed

- Shortened the Models bulk action from Disable results / Enable results to Disable / Enable.
- Reduced the Models search column minimum width so the four-control toolbar remains contained near the 992 px breakpoint.
- Top-aligned the attachment file icon, metadata, compatibility status, and remove control inside the taller capability-aware attachment card.

### Feedback Applied

- Applied both browser comments captured at the 995 × 964 Models and capability-aware Chat states.

### Review Focus

- Whether the shorter bulk action remains clear in the Models context while eliminating right-edge overflow.
- Whether top alignment gives the attachment card a cleaner hierarchy without crowding the Needs Vision status.

### Simulated Or Deferred Behavior

- Bulk enable/disable and attachment compatibility remain illustrative in-memory wireframe behavior.

### Open Questions

- None introduced by this refinement round.

### Verification

- Confirmed at 995 × 964 that the Models toolbar and model list share the same 945 px right edge with no document-level horizontal overflow.
- Confirmed the compact bulk action renders at 70 px wide, changes from Disable to Enable, and updates all seven visible model rows.
- Confirmed the attachment file icon, metadata container, and remove control share the same 723 px top coordinate with computed `align-items: start`.
- Confirmed the Needs Vision status retains bottom breathing room and the checked routes report no console warnings or errors.
- Confirmed `script.js` passes Node syntax validation and the r013 file set passes `git diff --check`.

### Approval Path

- Approval of Review Round 3 accepts these r013 refinements and returns the complete r013 wireframe phase to an implementation-handoff-ready state.

## Review Round 2 — 2026-07-18 — Browser feedback refinements

### Changed

- Replaced persistent Chat header provenance chips with an information icon and on-demand metadata popover.
- Removed Models Details buttons and made each model name the details trigger with a hover underline.
- Kept the Models toolbar inline at 992 px and wider; below 992 px the search spans the first row and both filters plus the bulk toggle share the second row.
- Left-aligned the Advanced Policy tabs and stretched the current-preset guardrail across the policy content width.
- Added breathing room below attachment compatibility labels.

### Feedback Applied

- Applied all seven browser comments captured against the Chat, Models, Advanced Policies, and capability-aware attachment states.

### Review Focus

- Whether Chat metadata is discoverable without occupying persistent header space.
- Whether model-name links make details access clear without a repeated action column.
- Whether the two responsive Models toolbar arrangements remain balanced around the 992 px breakpoint.
- Whether the policy controls and attachment status now align and breathe correctly.

### Simulated Or Deferred Behavior

- Runtime, model, environment, adapter, and health values in Chat information remain illustrative in-memory metadata.
- Capability evidence and attachment conversion remain prototype behavior as described in Review Round 1.

### Open Questions

- None introduced by this refinement round.

### Verification

- Confirmed the 1021 × 964 review viewport renders the Chat information icon without persistent provenance chips; the popover opens with runtime, environment, workspace, model, adapter, and health metadata.
- Confirmed Models renders no Details buttons, exposes seven clickable model names, includes the hover-underline rule, and opens the correct details dialog from the first model name.
- Confirmed all four Models toolbar controls share one row at 1021 px; at 900 px search spans the first row while both filters and the bulk toggle share the second row.
- Confirmed the Advanced Policy tabs, current-preset guardrail, search, and policy browser share the same left and right edges at the review viewport.
- Confirmed the Needs Vision label has a visible 4 px gap below it inside the attachment card.
- Confirmed checked routes have no document-level horizontal overflow and the in-app browser reported no console warnings or errors.
- Confirmed `script.js` passes Node syntax validation and the r013 file set passes `git diff --check`.

### Approval Path

- Approval of Review Round 2 accepts these r013 refinements and returns the complete r013 wireframe phase to an implementation-handoff-ready state.

## Review Round 1 — 2026-07-18 — Research-to-wireframe translation

### Changed

- Copied the accepted r012 SPA forward into a new major revision.
- Replaced approval choices with Ask for approval, Approve safe actions, Approve for me, and Custom.
- Replaced the internal scenario list with seven user-facing policy groups and a concrete Exceptions view.
- Added compact capability filters and details to model selection and Models settings.
- Added capability-aware attachment status, model-switch/send preflight, and explicit resolution actions.
- Added model-dependent reasoning controls that are recomputed when the model changes.
- Reframed universal Thinking presentation as honest Activity or an available provider reasoning summary.
- Added compact runtime, environment, workspace, health, and expandable per-run provenance.

### Feedback Applied

- Applied the frozen `00001-c4os-ai-harness-research` recommendation to create a later wireframe revision without modifying r012.
- Applied the user-requested coverage list from 2026-07-18.

### Review Focus

- Whether the four approval presets and `Approve for me` guardrail copy feel simple without hiding meaningful safety boundaries.
- Whether model capability chips, filters, and details expose enough information without making the composer heavy.
- Whether incompatible attachments remain understandable through switch, convert, remove, and cancel choices.
- Whether activity, reasoning summaries, and compact provenance are honest and appropriately placed.

### Simulated Or Deferred Behavior

- Provider/model capability data, evidence timestamps, health, conversions, response generation, and provenance snapshots are illustrative in-memory wireframe behavior.
- Attachment conversion changes only the prototype compatibility state; it does not transcode a real file.
- Runtime, environment, policy, and model changes do not contact real services.
- The capability-review route seeds an illustrative image draft so incompatibility behavior can be reviewed without choosing a local file.

### Open Questions

- Should capability details include price and latency in this structural phase, or remain focused on compatibility and limits?
- Should concrete exceptions open in a detail drawer, or is the compact list sufficient for the first round?

### Verification

- Confirmed `script.js`, `markdown.js`, and `markdown-source.js` pass Node syntax checks.
- Confirmed the workflow launcher exposes capability-aware Chat, Models, Configuration, and Advanced Policies entry points.
- Confirmed the seeded Chat route shows Kimi K2, a visible image attachment marked Needs Vision, hidden reasoning controls, and an explicit no-silent-drop alert.
- Confirmed Use compatible model changes to GPT-5, marks the attachment Ready, reveals the reasoning control, and closes the alert.
- Confirmed a conflicting switch to Kimi K2 leaves GPT-5 active until resolution, and Convert then applies Kimi K2, marks the attachment Converted · Ready, and hides unsupported reasoning controls.
- Confirmed Models settings renders seven capability-enriched rows, filters Reasoning to four matching rows, and opens route-specific model details.
- Confirmed Configuration exposes exactly four approval presets and the Approve for me safety ceiling.
- Confirmed Advanced Policies renders seven groups, switches to the Exceptions view, and exposes two narrow saved rules.
- Confirmed a 1440 × 900 wide viewport and 720 × 900 narrow viewport render without document-level horizontal overflow after transitions settle.
- Confirmed the in-app browser reported no console errors or warnings on the checked routes.

### Approval Path

- Approval of Review Round 1 accepts the complete r013 wireframe phase and unlocks an implementation handoff package. Requested structural or interaction changes remain in r013 when minor, or create r014 when they materially change navigation, screen inventory, or flow.
