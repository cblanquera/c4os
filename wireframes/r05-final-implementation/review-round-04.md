# Feedback-Loop Review Round 4

Phase: Wireframes
Revision: `r05-final-implementation`
Round: 4
Review target: `./index.html#settings-plugin-detail`

## What Changed

- Revised Settings > Plugin Detail so the rendered form is clearly schema-driven
  and shows possible field types: input, number, switch, select, sensitive
  input, and default-only value.
- Removed the separate Shell-reserved config block. Reserved shell settings
  such as panel placement and icon order now appear inside the same rendered
  form.
- Moved the Enabled control to the right side of the plugin-detail title row.
- Moved Repair states and Tool policy below the rendered form.
- Changed Settings > Configuration remembered-rule actions from text buttons
  to icon buttons for edit and revoke.
- Revised Settings > Plugin Marketplace to follow the r04/source screenshots:
  source filter, configured marketplace row, Add Marketplace row, and add
  marketplace modal fields.
- Revised Settings > Skills to stay closer to the r04 source structure with a
  search field and list panel instead of a looser custom layout.

## Please Review

- Does `#settings-plugin-detail` now communicate that fields are rendered from
  plugin config/schema and are not literal GitHub fields?
- Is it clear enough that a plugin can declare default-only settings with no
  user-editable input?
- Does moving Enabled into the plugin title row feel right?
- Do Repair states and Tool policy belong below the rendered form in this
  structure?
- Do `#settings-configuration` policy rows work better with icon-only edit and
  revoke actions?
- Does `#settings-plugin-marketplace` now match the r04/source screenshots
  closely enough?
- Does `#settings-skills` now stay close enough to the r04/source Skills
  surface?

## Approval Path

If this review round is approved, the next step is still to update specs 03,
04, and 11 with approved wireframe evidence and accepted UI behavior, then
update durable wireframe handoff docs only for approved Settings behavior. No
spec should be marked frozen unless explicitly requested.
