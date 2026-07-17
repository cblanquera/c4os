# r06 Final Implementation Notes

## Review Round 3 - White Screen Startup Fix

Date: 2026-07-07

Changed screens, states, copy, layout, or behavior:

- Fixed the r06 startup white screen by moving the imported feature renderer
  helper to the correct IIFE scope.
- Bumped the document-relative CSS/JS cache token to
  `r06-white-screen-fix`.

Root cause:

- The r05 merge inserted `renderFeatureRoute()` inside
  `bindMarketplaceControls()` instead of at feature-renderer scope.
- The exported feature renderer referenced `renderFeatureRoute` before it was
  visible, causing a `ReferenceError` during script evaluation. Because the
  script failed before `renderRoute()` could run, `index.html` showed an empty
  white page.

What should be reviewed next:

- Reload `./index.html` and confirm the App Start screen appears instead of a
  blank page.
- Open `#debug` and `#browser-navigation` to confirm feature routes still
  render after the scope fix.

Feedback or annotations applied:

- Applied direct feedback that `index.html` loaded as a white screen.

Simulated or deferred behavior:

- No product behavior changed. This is a static wireframe runtime fix only.

Open questions:

- None blocking this fix.

Approval path:

- If the page now loads, continue with the minor visual/content changes you
  held back during the full r06 batch integration review.

## Review Round 2 - Batches 2 Through 5 Integration

Date: 2026-07-06

Changed screens, states, copy, layout, or behavior:

- Integrated the latest approved r05 feature renderer into r06 for all
  Batch 1 through Batch 5 final-implementation routes.
- Preserved r04-only routes in r06 for App Start, provider/model popovers,
  provider/model/runtime/MCP settings, file explorer, chat/session carry-forward
  states, and legacy terminal carry-forward states.
- Added Batch 2 Settings routes for Plugins, Configuration, Skills, plugin
  marketplace/detail/states/uninstall, config error fallback, skill detail,
  skill customize, and invalid skill states.
- Added Batch 3 prompt routes for suggestions, approvals, remembered rules,
  blocked suggestion repair, branch selection, attachments, and safe fallback.
- Added Batch 4 workspace/files routes for workspace start/load/missing/search,
  non-Git state, File Editor panel placement, file context menu, operations,
  dirty editor, external conflict, and empty states.
- Added Batch 5 visible runtime plugin routes for Terminal user PTY, Browser
  navigation/menu/page context menu/preview host, and Chat Debug timeline/event
  detail.
- Added `browser-annotations.md` as the r06 markdown-only record for Browser
  annotation behavior that should not render as a standalone product screen.

What should be reviewed next:

- Whether the complete r06 route set still feels like one functional SPA after
  combining the r04 carry-forward routes with the latest r05 feature states.
- Whether the Batch 2 through Batch 5 routes preserve the accepted r05 behavior
  without reintroducing annotation-like text in visible shell surfaces.
- Whether `#browser-navigation`, `#browser-menu`,
  `#browser-page-context-menu`, `#browser-preview-host`, `#debug`,
  `#debug-timeline`, and `#debug-event-detail` match the final r05 cleanup
  rounds.

Feedback or annotations applied:

- Applied the request to continue past Batch 1 and add the rest of the r05
  batches before collecting minor feedback.

Simulated or deferred behavior:

- This remains static wireframe behavior. No product code, `.agents` specs, or
  durable handoff docs were changed.
- Browser annotation behavior remains markdown-only in
  `./browser-annotations.md`.

Open questions:

- None blocking this full-route r06 review round.

Approval path:

- If this review round is approved, the next step is to apply any final minor
  visual/content corrections you held back, then decide whether r06 should be
  synced into durable wireframe/spec handoff records.

## Review Round 1 - Batch 0 Copy And Batch 1 Shell Foundation

Date: 2026-07-06

Changed screens, states, copy, layout, or behavior:

- Created `wireframes/r06-final-implementation/` from an exact copy of
  `wireframes/r04-single-page-app/`.
- Preserved the original r04 routes for app start, new session, chat session,
  provider/model popovers, file explorer/editor, terminal, and Settings.
- Added r05 Batch 1 final-shell routes: `#shell-foundation`,
  `#same-side-replacement`, `#per-chat-restore`, `#resize-collision`,
  `#hidden-activity`, `#debug`, `#repair-state`, `#settings`, and
  `#coverage`.
- Reworked non-start, non-Settings r04 shell routes into the r06 global-header
  model with left/right plugin icon groups.
- Kept review coverage isolated to `#coverage` so normal shell routes remain
  product-like.

What should be reviewed next:

- Whether r06 preserves enough r04 behavior while switching the active shell
  model to the approved r05 Batch 1 header/plugin-panel pattern.
- Whether `#shell-foundation` reads as the correct final baseline with no
  default right panel.
- Whether `#same-side-replacement`, `#per-chat-restore`,
  `#resize-collision`, `#hidden-activity`, `#debug`, and `#repair-state`
  cover the Batch 1 acceptance states clearly enough.

Feedback or annotations applied:

- Applied the requested r06 strategy: copy r04 exactly first, then slowly add
  r05 features in batches.
- Applied the established C4OS wireframe rule to keep explanatory notes out of
  the rendered shell surface.

Simulated or deferred behavior:

- Panel restore, resize persistence, hidden plugin fanout, repair decisions,
  Chat Debug events, terminal output, and provider/model actions are static
  review simulations.
- Batch 2 Settings upgrades, Batch 3 prompt/approval flows, Batch 4
  workspace/files, and Batch 5 runtime plugin panels remain deferred.
- No `.agents` specs, durable handoff docs, or production code were updated.

Open questions:

- None blocking this Batch 1 review round.

Approval path:

- If this review round is approved, the next step is Batch 2: merge the
  approved r05 Settings and Configuration states into this r06 SPA while
  keeping the r04 Settings routes functional.
