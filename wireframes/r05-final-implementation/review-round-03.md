# Feedback-Loop Review Round 3

Phase: Wireframes
Revision: `r05-final-implementation`
Round: 3
Review target: `./index.html#settings-plugins`

## What Changed

- Added Batch 2 Settings routes while preserving the approved r05 shell model
  from Batch 1.
- Added focused Settings > Plugins routes for plugin list/detail, settings
  renderer fields, sensitive value redaction, reserved shell keys, marketplace
  source/install review, blocked/incompatible/pending-restart/repair states,
  icon fallback, and uninstall data prompt.
- Added Settings > Configuration routes for per-server-tool policy rows,
  remembered approval rule review/edit/revoke controls, and
  config.toml parse-error with last-valid fallback.
- Added Settings > Skills routes for list/detail/customize/invalid states,
  bundled read-only customization copy, source/status visibility, and `$`
  suggestion filtering.
- Updated `#coverage` to cover specs 03, 04, and 11 only.
- Kept all Batch 2 behavior inside static grayscale HTML/CSS/JS review routes.

## Please Review

- Does `#settings-plugins` show the right plugin-management entry point for
  enabled, disabled, dependency-blocked, incompatible, pending-restart, repair,
  icon fallback, panel side, and icon-order review?
- Does `#settings-plugin-detail` express the simple settings renderer,
  shell-reserved fields, and sensitive setting redaction clearly enough?
- Does `#settings-plugin-marketplace` make metadata-first source/install review
  clear without implying passive code execution or auto-enablement?
- Does `#settings-plugin-uninstall` ask the right data-retention question before
  uninstall?
- Does `#settings-configuration` show one policy item per registered server
  tool, with remembered approval rules reviewable, editable, and revocable?
- Does `#settings-config-error` communicate that config.toml is the source while
  C4OS keeps the last valid config active on parse errors?
- Does `#settings-skills` plus the detail/customize/invalid routes cover the
  Skills list, bundled customization copy, plugin-provided source boundary,
  invalid states, and `$` suggestion filtering?
- Is the `#coverage` matrix sufficient for Batch 2 review against specs 03, 04,
  and 11?

## Approval Path

If this review round is approved, the next step is to update specs 03, 04, and
11 with approved wireframe evidence and accepted UI behavior, then update
wireframe handoff docs only for durable approved Settings behavior. No spec
should be marked frozen unless explicitly requested.
